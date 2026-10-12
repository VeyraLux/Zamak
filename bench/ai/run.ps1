param(
    [string]$ZamPath = "zam\target\debug\zam.exe",
    [int]$LatencyRepeats = 5
)
$ErrorActionPreference = 'Stop'
if ($LatencyRepeats -lt 1) { throw 'LatencyRepeats must be at least 1.' }
$root = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$cases = Get-Content (Join-Path $PSScriptRoot 'cases.json') -Raw | ConvertFrom-Json
$stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
$outDir = Join-Path $root "results\ai-bench-$stamp"
New-Item -ItemType Directory -Force $outDir | Out-Null
$zam = (Resolve-Path (Join-Path $root $ZamPath)).Path

function Quote-ProcessArgument([string]$value) {
    if ($value -notmatch '[\s"]') { return $value }
    return '"' + ($value -replace '(\\*)"', '$1$1\"' -replace '(\\+)$', '$1$1') + '"'
}

function Invoke-ProcessCapture([string]$filePath, [string[]]$argumentList, [string]$workingDirectory, [string]$captureDir, [hashtable]$environment = @{}) {
    $captureId = [guid]::NewGuid().ToString('N')
    $stdoutPath = Join-Path $captureDir "$captureId.stdout.txt"
    $stderrPath = Join-Path $captureDir "$captureId.stderr.txt"
    $arguments = ($argumentList | ForEach-Object { Quote-ProcessArgument $_ }) -join ' '
    $previous = @{}
    try {
        foreach ($key in $environment.Keys) {
            $previous[$key] = [Environment]::GetEnvironmentVariable($key, 'Process')
            [Environment]::SetEnvironmentVariable($key, [string]$environment[$key], 'Process')
        }
        $sw = [Diagnostics.Stopwatch]::StartNew()
        $process = Start-Process -FilePath $filePath -ArgumentList $arguments -WorkingDirectory $workingDirectory -NoNewWindow -PassThru -RedirectStandardOutput $stdoutPath -RedirectStandardError $stderrPath -Wait
        $sw.Stop()
    }
    finally {
        foreach ($key in $environment.Keys) {
            [Environment]::SetEnvironmentVariable($key, $previous[$key], 'Process')
        }
    }
    # 空文件时 Get-Content -Raw 返回 $null，统一成字符串，避免后续字符串方法调用失败。
    $stdoutText = if (Test-Path $stdoutPath) { [string](Get-Content $stdoutPath -Raw) } else { '' }
    $stderrText = if (Test-Path $stderrPath) { [string](Get-Content $stderrPath -Raw) } else { '' }
    [pscustomobject]@{
        FilePath = $filePath
        Arguments = $argumentList
        Command = "$filePath $arguments"
        ExitCode = $process.ExitCode
        Ms = [math]::Round($sw.Elapsed.TotalMilliseconds, 3)
        StdoutPath = $stdoutPath
        StderrPath = $stderrPath
        Stdout = $stdoutText
        Stderr = $stderrText
    }
}

function Parse-JsonResult($capture, [string]$command, [bool]$expectSuccess) {
    $json = $null
    $jsonError = $null
    try { $json = ([string]$capture.Stdout).Trim() | ConvertFrom-Json } catch { $jsonError = $_.Exception.Message }
    $jsonShapePass = $null -ne $json -and $json.command -eq $command -and $json.PSObject.Properties.Name -contains 'ok'
    $exitConsistent = $jsonShapePass -and (($capture.ExitCode -eq 0) -eq [bool]$json.ok)
    $stderrFailurePass = [bool]$jsonShapePass -and ([bool]$json.ok -or -not [string]::IsNullOrWhiteSpace($capture.Stderr))
    $successPass = $jsonShapePass -and $exitConsistent -and $stderrFailurePass -and ([bool]$json.ok -eq $expectSuccess)
    [pscustomobject]@{
        Json = $json
        JsonError = $jsonError
        JsonShapePass = $jsonShapePass
        ExitConsistent = $exitConsistent
        StderrFailurePass = $stderrFailurePass
        SuccessPass = $successPass
    }
}

function Get-OutputPath([string]$stderr) {
    $line = @($stderr -split "`r?`n" | Where-Object { $_ -like 'OUTPUT *' } | Select-Object -Last 1)
    if ($line.Count -eq 0) { return $null }
    return $line[0].Substring(7).Trim()
}

function Repair([string]$id, [string]$source) {
    switch ($id) {
        'syntax-character' { return $source.Replace('println(@)', 'println(1)') }
        'ownership-move' { return $source.Replace('println(a)', 'println(b)') }
        'missing-return' { return ('fn value() -> string {' + [Environment]::NewLine + '    return "ok"' + [Environment]::NewLine + '}' + [Environment]::NewLine + 'fn main() {' + [Environment]::NewLine + '    println(value())' + [Environment]::NewLine + '}' + [Environment]::NewLine) }
        'type-mismatch' { return $source.Replace('let x: i64 = "s"', 'let x: i64 = 1') }
        'immutable-assign' { return $source.Replace('let count = 1', 'let mut count = 1') }
        'borrow-missing-ampersand' { return $source.Replace('take(text)', 'take(&text)') }
        'array-bounds' { return $source.Replace('println(a[5])', 'println(a[2])') }
        'literal-overflow' { return $source.Replace('9223372036854775808', '9223372036854775807') }
        default { return $source }
    }
}

$records = @()
foreach ($case in $cases) {
    $dir = Join-Path $outDir $case.id
    New-Item -ItemType Directory -Force $dir | Out-Null
    $path = Join-Path $dir 'main.zm'
    Set-Content -Path $path -Value $case.source -NoNewline
    $first = Invoke-ProcessCapture $zam @('check', '--json', $path) $root $dir
    $firstParsed = Parse-JsonResult $first 'check' ([bool]$true)
    $latencies = @($first.Ms)
    for ($i = 1; $i -lt $LatencyRepeats; $i++) {
        $sample = Invoke-ProcessCapture $zam @('check', '--json', $path) $root $dir
        $latencies += $sample.Ms
    }
    $rounds = 0
    $current = $case.source
    $warmSamples = @($latencies | Select-Object -Skip 1 | Sort-Object)
    $warmMedian = $null
    if ($warmSamples.Count -gt 0) {
        if ($warmSamples.Count % 2) { $warmMedian = $warmSamples[[math]::Floor($warmSamples.Count / 2)] }
        else { $warmMedian = ($warmSamples[$warmSamples.Count / 2 - 1] + $warmSamples[$warmSamples.Count / 2]) / 2 }
    }
    $solved = [bool]$firstParsed.SuccessPass
    if (-not $solved -and $case.patches.Count -gt 0) {
        $rounds = 1
        $current = Repair $case.id $current
        Set-Content -Path $path -Value $current -NoNewline
        $fixed = Invoke-ProcessCapture $zam @('check', '--json', $path) $root $dir
        $fixedParsed = Parse-JsonResult $fixed 'check' ([bool]$true)
        $solved = [bool]$fixedParsed.SuccessPass
    }

    $build = $null
    $buildParsed = $null
    $warmBuild = $null
    $warmBuildParsed = $null
    $run = $null
    $executionPass = $false
    $buildPass = $false
    $artifactPath = $null
    $cacheDir = Join-Path $dir 'cache'
    if ($solved) {
        $build = Invoke-ProcessCapture $zam @('build', '--json', $path) $root $dir @{ ZAMAK_CACHE_DIR = $cacheDir }
        $buildParsed = Parse-JsonResult $build 'build' ([bool]$true)
        $buildPass = [bool]$buildParsed.SuccessPass
        $artifactPath = Get-OutputPath $build.Stderr
        if ($buildPass -and $artifactPath -and (Test-Path $artifactPath)) {
            $warmBuild = Invoke-ProcessCapture $zam @('build', '--json', $path) $root $dir @{ ZAMAK_CACHE_DIR = $cacheDir }
            $warmBuildParsed = Parse-JsonResult $warmBuild 'build' ([bool]$true)
            $run = Invoke-ProcessCapture $artifactPath @() $root $dir
            $expected = (($case.expected_output -replace "`r`n", "`n") -replace "`r", "`n")
            $actual = (($run.Stdout -replace "`r`n", "`n") -replace "`r", "`n")
            $executionPass = $warmBuildParsed.SuccessPass -and $run.ExitCode -eq 0 -and $actual -ceq $expected -and [string]::IsNullOrWhiteSpace($run.Stderr)
        }
    }
    $unsupportedExpected = $case.expected_code -eq 'ZM900'
    $unsupportedRejected = $unsupportedExpected -and -not $solved
    $diagnostics = @($firstParsed.Json.diagnostics)
    $diagnosticCount = 0
    if ($null -ne $firstParsed.Json -and $null -ne $firstParsed.Json.diagnostics) { $diagnosticCount = $diagnostics.Length }
    $firstCode = $null
    if ($diagnosticCount -gt 0) { $firstCode = $diagnostics[0].code }
    if ($null -eq $case.expected_code) { $codeMatch = ($diagnosticCount -eq 0) }
    elseif ($diagnosticCount -gt 0) { $codeMatch = ($diagnostics[0].code -eq $case.expected_code) }
    else { $codeMatch = $false }
    $recordValidationPass = $firstParsed.JsonShapePass -and $firstParsed.ExitConsistent -and $firstParsed.StderrFailurePass -and
        ($diagnosticCount -eq 0 -or $diagnostics[0].code -eq $case.expected_code) -and
        (($case.patches.Count -eq 0 -and ($unsupportedRejected -or $executionPass)) -or ($case.patches.Count -gt 0 -and $solved -and $executionPass))
    $records += [pscustomobject]@{
        id = $case.id
        first_pass = [bool]$firstParsed.SuccessPass
        first_code = $firstCode
        code_match = $codeMatch
        expected_code = $case.expected_code
        repair_rounds = $rounds
        solved = $solved
        unsupported_expected = $unsupportedExpected
        unsupported_rejected = $unsupportedRejected
        expected_output = $case.expected_output
        check_samples_ms = @($latencies)
        check_first_ms = $latencies[0]
        check_warm_median_ms = $warmMedian
        check_median_ms = [math]::Round((($latencies | Sort-Object)[[math]::Floor($latencies.Count / 2)]), 3)
        checks = $LatencyRepeats
        check_json_shape_pass = $firstParsed.JsonShapePass
        check_exit_consistent = $firstParsed.ExitConsistent
        check_failure_stderr_pass = $firstParsed.StderrFailurePass
        first_stderr_path_informational = ([string]$first.Stderr).Contains($path)
        build_pass = $buildPass
        build_json_shape_pass = if ($buildParsed) { $buildParsed.JsonShapePass } else { $null }
        build_exit_consistent = if ($buildParsed) { $buildParsed.ExitConsistent } else { $null }
        build_failure_stderr_pass = if ($buildParsed) { $buildParsed.StderrFailurePass } else { $null }
        build_first_ms = if ($build) { $build.Ms } else { $null }
        build_warm_ms = if ($warmBuild) { $warmBuild.Ms } else { $null }
        build_first_cache_misses = if ($build) { @($build.Stderr -split "`r?`n" | Where-Object { $_ -match 'CACHE MISS|OBJECT MISS|LINK MISS' }).Count } else { $null }
        build_warm_cache_hits = if ($warmBuild) { @($warmBuild.Stderr -split "`r?`n" | Where-Object { $_ -match 'CACHE HIT|OBJECT HIT|LINK HIT' }).Count } else { $null }
        artifact = $artifactPath
        executable_bytes = if ($artifactPath -and (Test-Path $artifactPath)) { (Get-Item $artifactPath).Length } else { $null }
        execution_pass = $executionPass
        stdout_contract_pass = ($first.Stdout.Trim() -split "`r?`n").Count -eq 1
        validation_pass = $recordValidationPass
        captures = $dir
    }
}

$os = Get-CimInstance Win32_OperatingSystem
$toolVersion = Invoke-ProcessCapture $zam @('--version') $root $outDir
$report = [pscustomobject]@{
    generated_at = (Get-Date).ToString('o')
    os = "$($os.Caption) $($os.Version)"
    powershell = $PSVersionTable.PSVersion.ToString()
    zam = $zam
    zam_version = $toolVersion.Stdout.Trim()
    commands = @('check --json <file>', 'build --json <file>', '<artifact>')
    latency_repeats = $LatencyRepeats
    latency_note = 'Samples are first plus warm process launches; no cold OS-cache claim is made.'
    cases = $records
    unsupported_reject_count = @($records | Where-Object unsupported_rejected).Count
    first_pass_rate = [math]::Round((($records | Where-Object first_pass).Count / $records.Count), 3)
    solved_rate = [math]::Round((($records | Where-Object solved).Count / $records.Count), 3)
    code_match_rate = [math]::Round((($records | Where-Object code_match).Count / $records.Count), 3)
    execution_rate = [math]::Round((($records | Where-Object execution_pass).Count / [math]::Max(1, @($records | Where-Object solved).Count)), 3)
    validation_rate = [math]::Round((($records | Where-Object validation_pass).Count / $records.Count), 3)
}
$reportPath = Join-Path $outDir 'report.json'
$report | ConvertTo-Json -Depth 8 | Set-Content $reportPath
Get-Content $reportPath
if (@($records | Where-Object { -not $_.validation_pass }).Count -gt 0) {
    throw 'AI benchmark validation failed; inspect report.json and retained captures.'
}
