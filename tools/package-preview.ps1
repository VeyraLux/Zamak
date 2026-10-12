param([string]$OutDir = '', [switch]$SkipBuild)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
if (-not $OutDir) { $OutDir = Join-Path $root 'dist' }
$OutDir = [IO.Path]::GetFullPath($OutDir)
if (-not $SkipBuild) {
    & cargo build --release --locked --manifest-path (Join-Path $root 'zam/Cargo.toml')
    if ($LASTEXITCODE -ne 0) { throw 'release build failed' }
}
$exe = Join-Path $root 'zam/target/release/zam.exe'
if (-not (Test-Path $exe)) { throw 'Windows release compiler is missing' }
$version = (& $exe --version).Trim()
if ($LASTEXITCODE -ne 0 -or $version -notmatch '^\d+\.\d+\.\d+$') { throw 'invalid compiler version' }
$stage = Join-Path $OutDir ('stage-' + [guid]::NewGuid().ToString('N'))
$package = Join-Path $stage "zamak-$version-windows-x64"
New-Item -ItemType Directory -Force -Path (Join-Path $package 'bin') | Out-Null
Copy-Item $exe (Join-Path $package 'bin/zam.exe')
foreach ($name in @('README.md','TUTORIAL.md','PREVIEW.md','CONTRIBUTING.md')) {
    Copy-Item (Join-Path $root $name) $package
}
foreach ($directory in @('docs','ico')) {
    Copy-Item (Join-Path $root $directory) $package -Recurse
}
# Copy source examples only; exclude locally generated executables and build caches.
Get-ChildItem (Join-Path $root 'examples') -Recurse -File | Where-Object {
    $_.Extension -in '.zm','.toml' -and $_.FullName -notmatch '[\\/]build[\\/]'
} | ForEach-Object {
    $relative = [IO.Path]::GetRelativePath($root, $_.FullName)
    $destination = Join-Path $package $relative
    New-Item -ItemType Directory -Force -Path (Split-Path $destination) | Out-Null
    Copy-Item $_.FullName $destination
}
$zip = Join-Path $OutDir "zamak-$version-windows-x64.zip"
Compress-Archive -Path $package -DestinationPath $zip -Force
$unpacked = Join-Path $stage 'unpacked'
Expand-Archive -Path $zip -DestinationPath $unpacked
$package = Join-Path $unpacked "zamak-$version-windows-x64"
$smoke = Join-Path $stage 'smoke'
New-Item -ItemType Directory -Path $smoke | Out-Null
$packagedExe = Join-Path $package 'bin/zam.exe'
$project = Join-Path $smoke 'hello'
$env:ZAMAK_CACHE_DIR = Join-Path $smoke 'cache'
foreach ($cmd in @('new','check','build','run','fmt')) {
    $output = & $packagedExe $cmd $project
    if ($LASTEXITCODE -ne 0) { throw "packaged compiler $cmd failed" }
    if ($cmd -eq 'run' -and ($output -join "`n").Trim() -ne 'hello from zamak') { throw 'unexpected smoke output' }
}
$cases = Join-Path $package 'examples/tutorial_cases/main.zm'
$output = & $packagedExe run $cases
if ($LASTEXITCODE -ne 0) { throw 'packaged algorithm examples failed' }
$expected = "1`n1`n3628800`n0`n1`n55`n6`n6`n9`n42`nfalse`ntrue`ntrue`nfalse`n0`n8`n3`n6`n2`n问候：世界`n世界"
if (($output -join "`n").Trim() -ne $expected) { throw 'packaged algorithm output mismatch' }
$artifact = Get-ChildItem (Join-Path $project 'build') -Filter *.exe | Select-Object -First 1
$output = & $artifact.FullName
if ($LASTEXITCODE -ne 0 -or ($output -join "`n").Trim() -ne 'hello from zamak') { throw 'standalone program failed' }
$hash = (Get-FileHash $zip -Algorithm SHA256).Hash.ToLowerInvariant()
Set-Content -Path "$zip.sha256" -Value "$hash  $([IO.Path]::GetFileName($zip))" -Encoding utf8
Write-Output "PASS packaged new/check/build/run/fmt, algorithm outputs and standalone program"
Write-Output "ZIP $zip"
Write-Output "SHA256 $hash"
