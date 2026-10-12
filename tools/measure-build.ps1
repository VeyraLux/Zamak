param(
    [string]$ZamPath = "zam\target\debug\zam.exe",
    [int]$Modules = 6,
    [int]$Repetitions = 3,
    [int]$Warmup = 1,
    [string]$OutDir = ""
)
$ErrorActionPreference = 'Stop'
if ($Modules -lt 1) { throw 'Modules must be at least 1.' }
if ($Repetitions -lt 1) { throw 'Repetitions must be at least 1.' }
if ($Warmup -lt 0) { throw 'Warmup must be at least 0.' }
$root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$zam = (Resolve-Path (Join-Path $root $ZamPath)).Path
$stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
if ([string]::IsNullOrWhiteSpace($OutDir)) { $OutDir = Join-Path $root "results\build-measure-$stamp" }
New-Item -ItemType Directory -Force $OutDir | Out-Null

# 生成固定形状的多模块工程：main 调用 m1..mN 的 pub fn，编辑任一模块只应重编该模块。
function New-Template([string]$dir, [int]$modules) {
    New-Item -ItemType Directory -Force $dir | Out-Null
    $project = Join-Path $dir 'buildmeasure'
    & $zam new $project *> $null
    if ($LASTEXITCODE -ne 0) { throw "zam new failed for $project" }
    for ($i = 1; $i -le $modules; $i++) {
        Set-Content -Path (Join-Path $project "src\m$i.zm") -Value "pub fn value() -> i64 {`n    return $i`n}`n" -NoNewline
    }
    $main = @()
    for ($i = 1; $i -le $modules; $i++) { $main += "use m$i" }
    $main += ''
    $main += 'fn main() {'
    $main += '    let mut total = 0'
    for ($i = 1; $i -le $modules; $i++) { $main += "    total = total + m$i.value()" }
    $main += '    println(total)'
    $main += '}'
    Set-Content -Path (Join-Path $project 'src\main.zm') -Value (($main -join "`n") + "`n") -NoNewline
    return $project
}

$previousCache = [Environment]::GetEnvironmentVariable('ZAMAK_CACHE_DIR', 'Process')
$capture = Join-Path $OutDir 'captures'
New-Item -ItemType Directory -Force $capture | Out-Null

function Invoke-Zam([string]$workDir, [string]$cacheDir, [string[]]$arguments, [string]$tag) {
    $out = Join-Path $capture "$tag.out.txt"
    $err = Join-Path $capture "$tag.err.txt"
    [Environment]::SetEnvironmentVariable('ZAMAK_CACHE_DIR', $cacheDir, 'Process')
    $sw = [Diagnostics.Stopwatch]::StartNew()
    Push-Location $workDir
    try { & $zam @arguments 1> $out 2> $err } finally { Pop-Location }
    $code = $LASTEXITCODE
    $sw.Stop()
    $stderr = if (Test-Path $err) { [string](Get-Content $err -Raw) } else { '' }
    [pscustomobject]@{
        ExitCode = $code
        Ms = [math]::Round($sw.Elapsed.TotalMilliseconds, 3)
        Stderr = $stderr
        ObjectMiss = @($stderr -split "`r?`n" | Where-Object { $_ -like 'OBJECT MISS *' }).Count
        ObjectHit = @($stderr -split "`r?`n" | Where-Object { $_ -like 'OBJECT HIT *' }).Count
        LinkMiss = @($stderr -split "`r?`n" | Where-Object { $_ -like 'LINK MISS *' }).Count
        LinkHit = @($stderr -split "`r?`n" | Where-Object { $_ -like 'LINK HIT *' }).Count
        Output = @($stderr -split "`r?`n" | Where-Object { $_ -like 'OUTPUT *' } | Select-Object -Last 1)
    }
}

function Get-DirectoryBytes([string]$path) {
    if (-not (Test-Path $path)) { return 0 }
    return (Get-ChildItem $path -Recurse -File -ErrorAction SilentlyContinue | Measure-Object -Property Length -Sum).Sum
}

$template = New-Template (Join-Path $OutDir 'template') $Modules

# 预热：先在丢弃的副本上跑一轮冷/热构建，让 C 工具链和头文件进入系统文件缓存，
# 避免把首次读取编译器的开销算进被记录的样本。用 -Warmup 0 关闭。
if ($Warmup -gt 0) {
    $warmupDir = Join-Path $OutDir 'warmup'
    New-Item -ItemType Directory -Force $warmupDir | Out-Null
    Copy-Item -Recurse -Force (Join-Path $template '*') $warmupDir
    $warmupCache = Join-Path $OutDir 'cache-warmup'
    New-Item -ItemType Directory -Force $warmupCache | Out-Null
    for ($w = 1; $w -le $Warmup; $w++) {
        $null = Invoke-Zam $warmupDir $warmupCache @('build', '.') "warmup$w-cold"
        $null = Invoke-Zam $warmupDir $warmupCache @('build', '.') "warmup$w-warm"
    }
}

$runs = @()
for ($rep = 1; $rep -le $Repetitions; $rep++) {
    $repDir = Join-Path $OutDir "rep$rep"
    New-Item -ItemType Directory -Force $repDir | Out-Null
    Copy-Item -Recurse -Force (Join-Path $template '*') $repDir
    $project = $repDir
    $cacheDir = Join-Path $OutDir "cache$rep"
    New-Item -ItemType Directory -Force $cacheDir | Out-Null

    $cold = Invoke-Zam $project $cacheDir @('build', '.') "rep$rep-cold"
    if ($cold.ExitCode -ne 0) { throw "cold build failed in repetition $rep" }
    if ($cold.ObjectMiss -ne ($Modules + 1) -or $cold.ObjectHit -ne 0) {
        throw "cold build expected $($Modules + 1) misses, saw $($cold.ObjectMiss) misses / $($cold.ObjectHit) hits"
    }

    $warm = Invoke-Zam $project $cacheDir @('build', '.') "rep$rep-warm"
    if ($warm.ExitCode -ne 0) { throw "warm build failed in repetition $rep" }
    if ($warm.ObjectHit -ne ($Modules + 1) -or $warm.ObjectMiss -ne 0) {
        throw "warm build expected $($Modules + 1) hits, saw $($warm.ObjectHit) hits / $($warm.ObjectMiss) misses"
    }

    $check = Invoke-Zam $project $cacheDir @('check', '.') "rep$rep-check"
    if ($check.ExitCode -ne 0) { throw "check failed in repetition $rep" }

    # 实现编辑：改叶子模块的函数体，接口不变，只应重编该模块。
    Set-Content -Path (Join-Path $project 'src\m1.zm') -Value "pub fn value() -> i64 {`n    return 2`n}`n" -NoNewline
    $leaf = Invoke-Zam $project $cacheDir @('build', '.') "rep$rep-leaf"
    if ($leaf.ExitCode -ne 0) { throw "leaf edit build failed in repetition $rep" }
    if ($leaf.ObjectMiss -ne 1) { throw "leaf edit expected 1 miss, saw $($leaf.ObjectMiss)" }

    # 接口编辑：重命名 m2 的导出函数并同步调用方，应重编 m2 与 main。
    Set-Content -Path (Join-Path $project 'src\m2.zm') -Value "pub fn value2() -> i64 {`n    return 2`n}`n" -NoNewline
    $mainPath = Join-Path $project 'src\main.zm'
    Set-Content -Path $mainPath -Value ((Get-Content $mainPath -Raw).Replace('m2.value()', 'm2.value2()')) -NoNewline
    $signature = Invoke-Zam $project $cacheDir @('build', '.') "rep$rep-signature"
    if ($signature.ExitCode -ne 0) { throw "signature edit build failed in repetition $rep" }
    if ($signature.ObjectMiss -ne 2) { throw "signature edit expected 2 misses, saw $($signature.ObjectMiss)" }

    $artifactPath = $null
    if ($cold.Output) {
        $relative = ($cold.Output.Substring(7).Trim()) -replace '^\.\\', '' -replace '^\./', ''
        $artifactPath = Join-Path $project $relative
    }
    $runs += [pscustomobject]@{
        repetition = $rep
        cold_build_ms = $cold.Ms
        warm_build_ms = $warm.Ms
        leaf_edit_build_ms = $leaf.Ms
        signature_edit_build_ms = $signature.Ms
        check_ms = $check.Ms
        cold_object_misses = $cold.ObjectMiss
        warm_object_hits = $warm.ObjectHit
        leaf_object_misses = $leaf.ObjectMiss
        signature_object_misses = $signature.ObjectMiss
        artifact = $artifactPath
        artifact_bytes = if ($artifactPath -and (Test-Path $artifactPath)) { (Get-Item $artifactPath).Length } else { $null }
        cache_bytes = Get-DirectoryBytes $cacheDir
    }
}
[Environment]::SetEnvironmentVariable('ZAMAK_CACHE_DIR', $previousCache, 'Process')

function Get-Median($values) {
    $sorted = @($values | Sort-Object)
    if ($sorted.Count % 2) { return $sorted[[math]::Floor($sorted.Count / 2)] }
    return [math]::Round((($sorted[$sorted.Count / 2 - 1] + $sorted[$sorted.Count / 2]) / 2), 3)
}

$version = (& $zam --version) | Select-Object -First 1
$report = [pscustomobject]@{
    generated_at = (Get-Date).ToString('o')
    os = (Get-CimInstance Win32_OperatingSystem).Caption
    powershell = $PSVersionTable.PSVersion.ToString()
    zam = $zam
    zam_version = "$version".Trim()
    modules = $Modules
    repetition_count = $Repetitions
    warmup_cycles = $Warmup
    note = '时间包含进程启动与 C 工具链调用；不测量峰值内存。冷构建使用全新缓存目录，热构建复用同一缓存。正式样本前默认先跑一轮丢弃的预热构建，降低 C 工具链首次读取系统缓存的偏差；仍受机器负载影响，绝对值只用于同机对比。'
    medians = [pscustomobject]@{
        cold_build_ms = Get-Median ($runs.cold_build_ms)
        warm_build_ms = Get-Median ($runs.warm_build_ms)
        leaf_edit_build_ms = Get-Median ($runs.leaf_edit_build_ms)
        signature_edit_build_ms = Get-Median ($runs.signature_edit_build_ms)
        check_ms = Get-Median ($runs.check_ms)
        artifact_bytes = Get-Median ($runs.artifact_bytes)
        cache_bytes = Get-Median ($runs.cache_bytes)
    }
    runs = $runs
}
$reportPath = Join-Path $OutDir 'report.json'
$report | ConvertTo-Json -Depth 6 | Set-Content $reportPath
Get-Content $reportPath
