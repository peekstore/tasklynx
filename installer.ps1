# installer.ps1 - Installer and Uninstaller for Tasklynx on Windows
# Usage:
#   irm https://raw.githubusercontent.com/peekstore/tasklynx/main/installer.ps1 | iex
#   & ([scriptblock]::Create((irm .../installer.ps1))) --selfuninstall
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$ErrorActionPreference = "Stop"

$ProjectName   = "tasklynx"
$PublisherName = "rkriad585"
$GitHubRepo    = "peekstore/tasklynx"
$RepoBranch    = "main"

$ConfigDir  = Join-Path $env:USERPROFILE ".config\peekstore\$ProjectName"
$BinDir     = Join-Path $ConfigDir "bin"
$BinaryPath = Join-Path $BinDir "${ProjectName}.exe"
$TxPath     = Join-Path $BinDir "tx.exe"
$Shortcut   = Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs\Tasklynx.lnk"

# ── Uninstall ────────────────────────────────────────────────────────────────
$IsUninstall = $false
foreach ($a in $args) {
    if ($a -in @("--selfuninstall", "-selfuninstall", "--uninstall", "-u")) { $IsUninstall = $true }
}

if ($IsUninstall) {
    Write-Host ""
    Write-Host "Tasklynx Uninstaller" -ForegroundColor Yellow
    Write-Host ""
    if (Test-Path $BinaryPath) { Remove-Item $BinaryPath -Force; Write-Host "  Removed $BinaryPath" -ForegroundColor Green }
    if (Test-Path $TxPath)     { Remove-Item $TxPath -Force;     Write-Host "  Removed $TxPath" -ForegroundColor Green }
    if (Test-Path $Shortcut)   { Remove-Item $Shortcut -Force;   Write-Host "  Removed Start Menu shortcut" -ForegroundColor Green }
    if (Test-Path $BinDir)     { Remove-Item $BinDir -Recurse -Force -ErrorAction SilentlyContinue }
    if (Test-Path $ConfigDir)  { Remove-Item $ConfigDir -Recurse -Force -ErrorAction SilentlyContinue }

    $userPath = [Environment]::GetEnvironmentVariable("Path", [EnvironmentVariableTarget]::User)
    if ($userPath) {
        $clean = ($userPath -split ';' | Where-Object { $_.Trim().TrimEnd('\') -ine $BinDir.TrimEnd('\') -and $_.Trim() -ne '' }) -join ';'
        [Environment]::SetEnvironmentVariable("Path", $clean, [EnvironmentVariableTarget]::User)
        $env:PATH = (($env:PATH -split ';' | Where-Object { $_.Trim().TrimEnd('\') -ine $BinDir.TrimEnd('\') -and $_.Trim() -ne '' }) -join ';')
    }
    Write-Host ""
    Write-Host "Tasklynx uninstalled. Restart your terminal to apply PATH changes." -ForegroundColor Green
    Write-Host ""
    exit 0
}

# ── Install ──────────────────────────────────────────────────────────────────
Write-Host ""
Write-Host "Tasklynx Installer" -ForegroundColor Cyan
Write-Host ""

Write-Host "  Checking latest version from GitHub ... " -NoNewline
$version = $null
try {
    $raw = Invoke-RestMethod -Uri "https://raw.githubusercontent.com/${GitHubRepo}/${RepoBranch}/.version" -UseBasicParsing
    $version = "$raw".Trim()
} catch {}
if (-not $version) {
    try {
        $version = (Invoke-RestMethod -Uri "https://api.github.com/repos/${GitHubRepo}/releases/latest" -UseBasicParsing).tag_name
    } catch {}
}
if (-not $version) { $version = "v0.0.1-beta-2" }
if ($version -notmatch '^v') { $version = "v$version" }
Write-Host "$version" -ForegroundColor Green

Write-Host "  Detecting system architecture ... " -NoNewline
$arch = $null
try {
    $procArch = (Get-CimInstance Win32_Processor -ErrorAction SilentlyContinue | Select-Object -First 1).Architecture
    switch ($procArch) {
        12 { $arch = "arm64" }
        9  { $arch = "amd64" }
    }
} catch {}
if (-not $arch) {
    if ($env:PROCESSOR_ARCHITEW6432 -eq "ARM64" -or $env:PROCESSOR_ARCHITECTURE -eq "ARM64") {
        $arch = "arm64"
    } else {
        $arch = "amd64"
    }
}
Write-Host "windows-$arch" -ForegroundColor Green

$downloadUrl = "https://github.com/${GitHubRepo}/releases/download/${version}/tasklynx-windows-${arch}.exe"
Write-Host "  Downloading binary from $downloadUrl`n"

New-Item -ItemType Directory -Path $BinDir -Force | Out-Null

$oldProgressPreference = $ProgressPreference
try {
    $ProgressPreference = 'SilentlyContinue'
    Invoke-WebRequest -Uri $downloadUrl -OutFile $BinaryPath -UseBasicParsing
} catch {
    $ProgressPreference = $oldProgressPreference
    Write-Host "  Failed to download binary for $version ($arch)." -ForegroundColor Red
    Write-Host "  Check https://github.com/${GitHubRepo}/releases" -ForegroundColor Yellow
    exit 1
} finally {
    $ProgressPreference = $oldProgressPreference
}

# Verify we downloaded a real PE executable, not an HTML error page.
$bytes = [System.IO.File]::ReadAllBytes($BinaryPath)
if ($bytes.Length -lt 1024 -or $bytes[0] -ne 0x4D -or $bytes[1] -ne 0x5A) {
    Remove-Item $BinaryPath -Force -ErrorAction SilentlyContinue
    Write-Host "  Downloaded file is not a valid Windows executable (bad version/arch?)." -ForegroundColor Red
    exit 1
}

$size = [math]::Round((Get-Item $BinaryPath).Length / 1MB, 2)
Write-Host "  OK   Downloaded tasklynx.exe (${size} MB)" -ForegroundColor Green
Copy-Item $BinaryPath $TxPath -Force

Write-Host "  Configuring PATH environment variable ... " -NoNewline
$userPath = [Environment]::GetEnvironmentVariable("Path", [EnvironmentVariableTarget]::User)
$pathEntries = if ($userPath) { $userPath -split ';' } else { @() }
$alreadyInPath = $false
foreach ($entry in $pathEntries) {
    if ($entry.Trim().TrimEnd('\') -ieq $BinDir.TrimEnd('\')) { $alreadyInPath = $true; break }
}
if (-not $alreadyInPath) {
    [Environment]::SetEnvironmentVariable("Path", (($pathEntries + $BinDir) -join ';'), [EnvironmentVariableTarget]::User)
    $env:PATH = $env:PATH + ";" + $BinDir
    Write-Host "Added to PATH" -ForegroundColor Green
} else {
    Write-Host "Already in PATH" -ForegroundColor Yellow
}

# Optional Start Menu shortcut
try {
    New-Item -ItemType Directory -Force -Path (Split-Path $Shortcut) | Out-Null
    $ws = New-Object -ComObject WScript.Shell
    $lnk = $ws.CreateShortcut($Shortcut)
    $lnk.TargetPath = $BinaryPath
    $lnk.Description = "Tasklynx developer task manager"
    $lnk.Save()
} catch {}

Write-Host ""
Write-Host "OK   Tasklynx successfully installed!" -ForegroundColor Green
Write-Host "  Publisher        : $PublisherName" -ForegroundColor Cyan
Write-Host "  Installation Path: $BinaryPath" -ForegroundColor Cyan
Write-Host "  Version          : $version" -ForegroundColor Cyan
Write-Host ""
Write-Host "  Run 'tasklynx --help' or 'tx --help' to get started" -ForegroundColor Yellow
Write-Host "  Restart your terminal if commands are not found." -ForegroundColor Yellow
Write-Host ""
