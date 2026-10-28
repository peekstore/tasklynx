# installer.ps1 - Installer and Uninstaller for Tasklynx on Windows
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$ProjectName    = "tasklynx"
$PublisherName  = "rkriad585"
$GitHubRepo     = "peekstore/tasklynx"

$ConfigDir      = Join-Path $env:USERPROFILE ".config\peekstore\$ProjectName"
$BinDir         = Join-Path $ConfigDir "bin"
$BinaryPath     = Join-Path $BinDir "${ProjectName}.exe"
$TxPath         = Join-Path $BinDir "tx.exe"

$IsUninstall = $false
if ($args -contains "--selfuninstall" -or $args -contains "-selfuninstall" -or $args -contains "--uninstall" -or $args -contains "-u") {
    $IsUninstall = $true
}

if ($IsUninstall) {
    Write-Host ""
    Write-Host "Tasklynx Uninstaller" -ForegroundColor Yellow
    Write-Host ""
    if (Test-Path $BinaryPath) { Remove-Item $BinaryPath -Force; Write-Host "  Removed $BinaryPath" -ForegroundColor Green }
    if (Test-Path $TxPath) { Remove-Item $TxPath -Force; Write-Host "  Removed $TxPath" -ForegroundColor Green }
    if (Test-Path $BinDir) { Remove-Item $BinDir -Recurse -Force -ErrorAction SilentlyContinue }
    if (Test-Path $ConfigDir) { Remove-Item $ConfigDir -Recurse -Force -ErrorAction SilentlyContinue }
    $userPath = [Environment]::GetEnvironmentVariable("Path", [EnvironmentVariableTarget]::User)
    if ($userPath) {
        $pathEntries = $userPath -split ';'
        $cleanEntries = $pathEntries | Where-Object { $_.Trim().TrimEnd('\') -ine $BinDir.TrimEnd('\') }
        [Environment]::SetEnvironmentVariable("Path", ($cleanEntries -join ';'), [EnvironmentVariableTarget]::User)
        $env:PATH = ($env:PATH -split ';' | Where-Object { $_.Trim().TrimEnd('\') -ine $BinDir.TrimEnd('\') }) -join ';'
    }
    Write-Host ""
    Write-Host "Tasklynx uninstalled." -ForegroundColor Green
    Write-Host ""
    exit 0
}

Write-Host ""
Write-Host "Tasklynx Installer" -ForegroundColor Cyan
Write-Host ""

Write-Host "  Checking latest version from GitHub ... " -NoNewline
$versionUrl = "https://raw.githubusercontent.com/${GitHubRepo}/main/.version"
try {
    $version = (Invoke-RestMethod -Uri $versionUrl -UseBasicParsing).Trim()
    if ($version -notmatch '^v') { $version = "v$version" }
    Write-Host "$version" -ForegroundColor Green
} catch {
    $version = "v0.0.1-beta-1"
    Write-Host "$version" -ForegroundColor Yellow
}

Write-Host "  Detecting system architecture ... " -NoNewline
$arch = "amd64"
try {
    $procArch = (Get-CimInstance Win32_Processor -ErrorAction SilentlyContinue | Select-Object -First 1).Architecture
    if ($procArch -eq 12) { $arch = "arm64" }
} catch {}
Write-Host "windows-$arch" -ForegroundColor Green

$downloadUrl = "https://github.com/${GitHubRepo}/releases/download/${version}/tasklynx-windows-${arch}.exe"
Write-Host "  Downloading binary from $downloadUrl`n"

if (-not (Test-Path $BinDir)) {
    New-Item -ItemType Directory -Path $BinDir -Force | Out-Null
}

try {
    $oldProgressPreference = $ProgressPreference
    $ProgressPreference = 'SilentlyContinue'
    Invoke-WebRequest -Uri $downloadUrl -OutFile $BinaryPath -UseBasicParsing
    $ProgressPreference = $oldProgressPreference
    $size = [math]::Round((Get-Item $BinaryPath).Length / 1MB, 2)
    Write-Host "  OK   Downloaded tasklynx.exe (${size} MB)" -ForegroundColor Green
    Copy-Item $BinaryPath $TxPath -Force
} catch {
    $ProgressPreference = $oldProgressPreference
    Write-Host "  Failed to download binary." -ForegroundColor Red
    exit 1
}

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

Write-Host ""
Write-Host "OK   Tasklynx successfully installed!" -ForegroundColor Green
Write-Host "  Installation Path: $BinaryPath" -ForegroundColor Cyan
Write-Host "  Version          : $version" -ForegroundColor Cyan
Write-Host ""
Write-Host "  Run 'tasklynx --help' or 'tx --help' to get started" -ForegroundColor Yellow
Write-Host "  Restart your terminal if commands not found." -ForegroundColor Yellow
Write-Host ""
