# hanfix installer for Windows:
#   irm https://github.com/NaNIsaac/hanfix/releases/latest/download/install.ps1 | iex
# Puts hanfix.exe in $env:HANFIX_INSTALL_DIR (default %LOCALAPPDATA%\Programs\hanfix) and adds it to
# the user PATH. Pin a release with $env:HANFIX_VERSION = 'v0.2.3'. Run it again (or hanfix --update) to update.
$ErrorActionPreference = 'Stop'
[Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12

$repo = 'NaNIsaac/hanfix'
$dir = if ($env:HANFIX_INSTALL_DIR) { $env:HANFIX_INSTALL_DIR } else { Join-Path $env:LOCALAPPDATA 'Programs\hanfix' }
$version = if ($env:HANFIX_VERSION) { $env:HANFIX_VERSION } else { 'latest' }
$arch = if ($env:PROCESSOR_ARCHITECTURE -eq 'ARM64' -or $env:PROCESSOR_ARCHITEW6432 -eq 'ARM64') { 'aarch64' } else { 'x86_64' }
$asset = "hanfix-$arch-pc-windows-msvc.exe"
$base = if ($version -eq 'latest') { "https://github.com/$repo/releases/latest/download" } else { "https://github.com/$repo/releases/download/$version" }

Write-Host "hanfix: downloading $asset ($version)"
New-Item -ItemType Directory -Force -Path $dir | Out-Null
$tmp = Join-Path ([IO.Path]::GetTempPath()) ([Guid]::NewGuid().ToString() + '.exe')
try {
    Invoke-WebRequest -UseBasicParsing "$base/$asset" -OutFile $tmp
    # GitHub serves the .sha256 as binary, so read it from a file rather than from .Content
    Invoke-WebRequest -UseBasicParsing "$base/$asset.sha256" -OutFile "$tmp.sha256"
    $want = ((Get-Content -Raw "$tmp.sha256").Trim() -split '\s+')[0]
    # .NET instead of Get-FileHash, which is missing when Windows PowerShell inherits PowerShell 7's module path
    $fs = [IO.File]::OpenRead($tmp)
    try { $got = -join ([Security.Cryptography.SHA256]::Create().ComputeHash($fs) | ForEach-Object { $_.ToString('X2') }) } finally { $fs.Dispose() }
    if ($want -ne $got) { throw "hanfix: checksum mismatch (expected $want, got $got)" }
    # A running hanfix.exe (hanfix --update) cannot be overwritten, but it can be renamed.
    $exe = Join-Path $dir 'hanfix.exe'
    Remove-Item -Force "$exe.old" -ErrorAction SilentlyContinue
    if (Test-Path $exe) { Move-Item -Force $exe "$exe.old" }
    Move-Item -Force $tmp $exe
    Remove-Item -Force "$exe.old" -ErrorAction SilentlyContinue   # stays only while the old one runs
} finally {
    Remove-Item -Force $tmp, "$tmp.sha256" -ErrorAction SilentlyContinue
}

$userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
if (($userPath -split ';') -notcontains $dir) {
    [Environment]::SetEnvironmentVariable('Path', ($userPath.TrimEnd(';') + ";$dir").TrimStart(';'), 'User')
    Write-Host "hanfix: added $dir to your user PATH"
}
if (($env:Path -split ';') -notcontains $dir) { $env:Path += ";$dir" }

Write-Host "hanfix: installed $(& (Join-Path $dir 'hanfix.exe') --version) to $dir"
if ($env:HANFIX_UPDATE) { return }
Write-Host 'hanfix: to fix the previous command with hf, run this once and open a new terminal:'
Write-Host "    Add-Content `$PROFILE 'Invoke-Expression (hanfix --init powershell | Out-String)'"
