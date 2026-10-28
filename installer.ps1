# installer.ps1 - Installer and Uninstaller for tasklynx
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$ProjectName = "tasklynx"
$GitHubRepo  = "peekstore/tasklynx"
$BinDir      = Join-Path $env:LOCALAPPDATA "Programs\tasklynx"
$BinaryPath  = Join-Path $BinDir "tasklynx.exe"

$IsUninstall = $false
if ($args -contains "--selfuninstall" -or $args -contains "-u" -or $args -contains "--uninstall") { $IsUninstall = $true }

if ($IsUninstall) {
    Write-Host "`ntasklynx Uninstaller`n" -ForegroundColor Yellow
    if (Test-Path $BinaryPath) { Remove-Item $BinaryPath -Force; Write-Host "  Removed" -ForegroundColor Green }
    if (Test-Path $BinDir) { Remove-Item $BinDir -Recurse -Force -ErrorAction SilentlyContinue }
    $userPath = [Environment]::GetEnvironmentVariable("Path", [EnvironmentVariableTarget]::User)
    if ($userPath) {
        $clean = ($userPath -split ';' | Where-Object { $_.Trim().TrimEnd('\') -ine $BinDir.TrimEnd('\') }) -join ';'
        [Environment]::SetEnvironmentVariable("Path", $clean, [EnvironmentVariableTarget]::User)
    }
    Write-Host "`nUninstalled.`n" -ForegroundColor Green
    exit 0
}

Write-Host "`ntasklynx Installer`n" -ForegroundColor Cyan
$version = "latest"
try { $v = (Invoke-RestMethod -Uri "https://raw.githubusercontent.com/$GitHubRepo/main/.version" -UseBasicParsing).Trim(); if ($v) { $version = $v } } catch {}
if ($version -eq "latest") { try { $r = Invoke-RestMethod "https://api.github.com/repos/$GitHubRepo/releases/latest"; $version = $r.tag_name } catch {} }
if ($version -and $version -notmatch '^v') { $version = "v$version" }

$arch = "amd64"
try { $pa = (Get-CimInstance Win32_Processor -ErrorAction SilentlyContinue | Select-Object -First 1).Architecture; if ($pa -eq 12) { $arch = "arm64" } } catch {}
New-Item -ItemType Directory -Force -Path $BinDir | Out-Null
$Url = "https://github.com/$GitHubRepo/releases/download/$version/tasklynx-windows-$arch.exe"
try {
    $old = $ProgressPreference; $ProgressPreference = 'SilentlyContinue'
    Invoke-WebRequest $Url -OutFile $BinaryPath -UseBasicParsing
    $ProgressPreference = $old
    Write-Host "  Downloaded" -ForegroundColor Green
} catch {
    Write-Error "Download failed: $Url"; exit 1
}

$userPath = [Environment]::GetEnvironmentVariable("Path", [EnvironmentVariableTarget]::User)
if ($userPath -notlike "*$BinDir*") {
    [Environment]::SetEnvironmentVariable("Path", ($userPath.TrimEnd(';')+";"+$BinDir).Trim(';'), [EnvironmentVariableTarget]::User)
    $env:PATH += ";$BinDir"
}
try {
    $WshShell = New-Object -ComObject WScript.Shell
    $LnkDir = Join-Path $env:APPDATA 'Microsoft\Windows\Start Menu\Programs'
    New-Item -ItemType Directory -Force $LnkDir | Out-Null
    $Shortcut = $WshShell.CreateShortcut((Join-Path $LnkDir 'Tasklynx.lnk'))
    $Shortcut.TargetPath = $BinaryPath; $Shortcut.Save()
} catch {}
Write-Host "`ntasklynx installed to $BinaryPath`nRestart terminal.`n" -ForegroundColor Green
