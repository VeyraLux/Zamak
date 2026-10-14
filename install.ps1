# Install the latest Zamak compiler on Windows x64:
#   irm https://raw.githubusercontent.com/rayL-K/Zamak/main/install.ps1 | iex
# Environment: ZAMAK_HOME (install directory, default %LOCALAPPDATA%\zamak), ZAMAK_VERSION (e.g. 0.1.0),
# ZAMAK_NO_MODIFY_PATH=1 to leave the user PATH untouched.
$ErrorActionPreference = 'Stop'
$repo = 'rayL-K/Zamak'
$archive = 'zamak-windows-x64.zip'
if ($env:ZAMAK_DOWNLOAD_BASE) {
    $base = $env:ZAMAK_DOWNLOAD_BASE
} elseif ($env:ZAMAK_VERSION) {
    $base = "https://github.com/$repo/releases/download/v$($env:ZAMAK_VERSION)"
} else {
    $base = "https://github.com/$repo/releases/latest/download"
}
$zamakHome = if ($env:ZAMAK_HOME) { $env:ZAMAK_HOME } else { Join-Path $env:LOCALAPPDATA 'zamak' }

$tmp = Join-Path ([IO.Path]::GetTempPath()) ('zamak-install-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $tmp | Out-Null
try {
    Write-Host "zamak: downloading $base/$archive"
    Invoke-WebRequest -UseBasicParsing "$base/$archive" -OutFile (Join-Path $tmp $archive)
    Invoke-WebRequest -UseBasicParsing "$base/$archive.sha256" -OutFile (Join-Path $tmp "$archive.sha256")
    $expected = ((Get-Content (Join-Path $tmp "$archive.sha256") -Raw).Trim() -split '\s+')[0].ToLowerInvariant()
    $actual = (Get-FileHash (Join-Path $tmp $archive) -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($expected -ne $actual) { throw "checksum mismatch for $archive" }

    Expand-Archive -Path (Join-Path $tmp $archive) -DestinationPath $tmp
    if (Test-Path $zamakHome) { Remove-Item -Recurse -Force $zamakHome }
    New-Item -ItemType Directory -Force -Path (Split-Path $zamakHome) | Out-Null
    Move-Item (Join-Path $tmp 'zamak-windows-x64') $zamakHome
} finally {
    Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
}

$bin = Join-Path $zamakHome 'bin'
$version = & (Join-Path $bin 'zam.exe') --version
Write-Host "zamak: installed $version to $zamakHome"

$userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
if ($env:ZAMAK_NO_MODIFY_PATH) {
    Write-Host "zamak: add $bin to PATH to use zam"
} elseif (-not (($userPath -split ';') -contains $bin)) {
    [Environment]::SetEnvironmentVariable('Path', ($bin + ';' + $userPath).TrimEnd(';'), 'User')
    Write-Host "zamak: added $bin to your user PATH; open a new terminal to use zam"
}
if (-not $env:Path.Contains($bin)) { $env:Path = "$bin;$env:Path" }

$vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
$hasMsvc = (Test-Path $vswhere) -and (& $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath)
if (-not $hasMsvc -and -not (Get-Command gcc -ErrorAction SilentlyContinue) -and -not (Get-Command clang -ErrorAction SilentlyContinue)) {
    Write-Host 'zamak: no C compiler found. Install Visual Studio Build Tools ("Desktop development with C++") or MinGW-w64 gcc.'
}
Write-Host 'zamak: try it with: zam new hello; zam run hello'
