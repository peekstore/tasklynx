$ErrorActionPreference = 'Stop'
$Repo = 'peekstore/tasklynx'
if ($env:TASKLYNX_VERSION) { $Version = $env:TASKLYNX_VERSION } else { $Release = Invoke-RestMethod "https://api.github.com/repos/$Repo/releases/latest"; $Version = $Release.tag_name }
$Asset = "tasklynx-windows-amd64.exe"
$Url = "https://github.com/$Repo/releases/download/$Version/$Asset"
$Dir = Join-Path $env:LOCALAPPDATA 'Tasklynx'
New-Item -ItemType Directory -Force $Dir | Out-Null
$Exe = Join-Path $Dir 'tasklynx.exe'
$Ico = Join-Path $Dir 'tasklynx.ico'
Invoke-WebRequest $Url -OutFile $Exe
try { Invoke-WebRequest "https://raw.githubusercontent.com/$Repo/main/assets/tasklynx.ico" -OutFile $Ico } catch {}
$UserPath=[Environment]::GetEnvironmentVariable('Path','User'); if($UserPath -notlike "*$Dir*"){[Environment]::SetEnvironmentVariable('Path',(($UserPath.TrimEnd(';')+';'+$Dir).Trim(';')),'User')}
try {
  $WshShell = New-Object -ComObject WScript.Shell
  $LnkDir = Join-Path $env:APPDATA 'Microsoft\Windows\Start Menu\Programs'
  New-Item -ItemType Directory -Force $LnkDir | Out-Null
  $Shortcut = $WshShell.CreateShortcut((Join-Path $LnkDir 'Tasklynx.lnk'))
  $Shortcut.TargetPath = $Exe
  if (Test-Path $Ico) { $Shortcut.IconLocation = $Ico }
  $Shortcut.WorkingDirectory = $Dir
  $Shortcut.Save()
} catch {}
Write-Host "Installed Tasklynx $Version to $Dir"
