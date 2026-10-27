$ErrorActionPreference = 'Stop'
$Repo = 'rkriad585/tasklynx'
$Release = Invoke-RestMethod "https://api.github.com/repos/$Repo/releases/latest"
$Version = $Release.tag_name
$Asset = "tasklynx-windows-amd64.exe"
$Url = "https://github.com/$Repo/releases/download/$Version/$Asset"
$Dir = Join-Path $env:LOCALAPPDATA 'Tasklynx'
New-Item -ItemType Directory -Force $Dir | Out-Null
Invoke-WebRequest $Url -OutFile (Join-Path $Dir 'tasklynx.exe')
$UserPath=[Environment]::GetEnvironmentVariable('Path','User'); if($UserPath -notlike "*$Dir*"){[Environment]::SetEnvironmentVariable('Path',(($UserPath.TrimEnd(';')+';'+$Dir).Trim(';')),'User')}
Write-Host "Installed Tasklynx $Version to $Dir"
