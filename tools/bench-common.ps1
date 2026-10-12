<#
.SYNOPSIS
Compare seven tutorial integer algorithms in release Zamak, optimized C, Python, Rust, and Go.
.DESCRIPTION
Windows runner, no third-party dependencies. Run from any directory:
  pwsh -File tools/bench-common.ps1
Requires cargo, Python 3, and MSVC (or -CCompiler gcc). Rustc and Go are autodetected.
Builds the release zam tool before measurement. Toolchain setup/build is excluded from run time.
The selected C compiler also selects Zamak's own C backend, because Zamak reads CC and an unset CC
means the MSVC default on Windows (see zam/src/native.rs tool()). MSVC and GNU runs are therefore
separate batches, and their timings and peak-memory figures are never mixed.
Five samples per language/kernel, exact seeded checksum assertions, calibrated duration >= 500ms,
median wall-clock time normalized by repeat count. Results are saved in ignored results/.
Checked Rust separately diagnoses arithmetic-checking costs. Rust release uses opt-level 3
with overflow checks off; Go uses its default optimized compiler pipeline.
The legacy screening criterion is each Zamak median <= 3x optimized C. A separate modern criterion
reports whether Zamak beats both Rust release and Go. These are machine-local measurements.
Do not run concurrently with builds, tests, or other CPU-intensive jobs.
#>
[CmdletBinding()]
param(
    [string]$CCompiler = '',
    [ValidateRange(0.5, 60.0)][double]$MinSeconds = 0.5,
    [switch]$SkipReleaseBuild,
    [switch]$SkipRust,
    [switch]$SkipGo
)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot

function Import-MsvcEnvironment {
    if (Get-Command cl.exe -ErrorAction SilentlyContinue) { return }
    $vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/Installer/vswhere.exe'
    if (-not (Test-Path $vswhere)) { throw 'MSVC not found; install Visual Studio C++ tools or specify -CCompiler gcc.' }
    $installation = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
    if ($LASTEXITCODE -ne 0 -or -not $installation) { throw 'vswhere could not find Visual Studio C++ tools.' }
    $vcvars = Join-Path $installation.Trim() 'VC/Auxiliary/Build/vcvars64.bat'
    if (-not (Test-Path $vcvars)) { throw "Missing vcvars64.bat: $vcvars" }
    $originalPath = $env:Path
    $lines = & $env:ComSpec /d /c "call `"$vcvars`" >nul && set"
    if ($LASTEXITCODE -ne 0) { throw 'Failed to initialize MSVC environment.' }
    foreach ($line in $lines) {
        $index = $line.IndexOf('=')
        if ($index -le 0) { continue }
        $key = $line.Substring(0, $index)
        $value = $line.Substring($index + 1)
        if ($key -ieq 'Path') { $value = "$value;$originalPath" }
        Set-Item -Path "env:$key" -Value $value
    }
    if (-not (Get-Command cl.exe -ErrorAction SilentlyContinue)) { throw 'MSVC cl.exe was not imported.' }
}

if (-not $CCompiler -or [System.IO.Path]::GetFileNameWithoutExtension($CCompiler) -eq 'cl') {
    Import-MsvcEnvironment
    $CCompiler = (Get-Command cl.exe).Source
} else {
    $CCompiler = (Get-Command $CCompiler -ErrorAction Stop).Source
}
$python = (Get-Command python -ErrorAction Stop).Source
$zam = Join-Path $root 'zam/target/release/zam.exe'
if (-not $SkipReleaseBuild) {
    $buildWatch = [System.Diagnostics.Stopwatch]::StartNew()
    & cargo build --release --locked --manifest-path (Join-Path $root 'zam/Cargo.toml')
    if ($LASTEXITCODE -ne 0) { throw 'Release zam build failed.' }
    $buildWatch.Stop()
    Write-Host ('Release compiler build (excluded): {0:F3} s' -f $buildWatch.Elapsed.TotalSeconds)
}
if (-not (Test-Path $zam)) { throw "Release zam binary missing: $zam" }
$arguments = @((Join-Path $PSScriptRoot 'common-kernel.py'), '--zam', $zam, '--cc', $CCompiler,
    '--min-seconds', $MinSeconds.ToString([System.Globalization.CultureInfo]::InvariantCulture), '--samples', '5')
# Propagate the chosen compiler to Zamak's own backend. Zamak on Windows falls back to MSVC whenever
# CC is unset, so clear CC for the MSVC default and set it for everything else; an unset CC is the only
# way Zamak finds MSVC, and a bare 'cl.exe' would be rejected by its GNU argument list.
if ([System.IO.Path]::GetFileNameWithoutExtension($CCompiler) -eq 'cl') {
    Remove-Item Env:CC -ErrorAction SilentlyContinue
    Write-Host 'Zamak backend: MSVC default (CC unset)'
} else {
    if ($env:CC -and $env:CC -ne $CCompiler) { Write-Host "Overriding CC=$env:CC with $CCompiler for the Zamak backend." }
    $env:CC = $CCompiler
    Write-Host "Zamak backend: $CCompiler"
}
if (-not $SkipRust) {
    $rust = Get-Command rustc -ErrorAction SilentlyContinue
    if ($rust) { $arguments += @('--rustc', $rust.Source) }
}
if (-not $SkipGo) {
    $go = Get-Command go.exe -ErrorAction SilentlyContinue
    if ($go) { $arguments += @('--go', $go.Source) }
}
& $python @arguments
if ($LASTEXITCODE -ne 0) { throw 'Tutorial benchmark failed; inspect the command output above.' }
