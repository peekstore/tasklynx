$ErrorActionPreference = 'Stop'

$toolsDir     = "$(Split-Path -parent $MyInvocation.MyCommand.Definition)"
$version      = '0.0.1-beta-1'
$url64        = "https://github.com/peekstore/tasklynx/releases/download/v${version}/tasklynx-windows-amd64.exe"
$urlArm64     = "https://github.com/peekstore/tasklynx/releases/download/v${version}/tasklynx-windows-arm64.exe"

# Detect architecture: ARM64 gets the ARM64 binary, everything else gets AMD64
$isArm64 = $false
try {
    $procArch = (Get-CimInstance Win32_Processor -ErrorAction SilentlyContinue | Select-Object -First 1).Architecture
    if ($procArch -eq 12) { $isArm64 = $true }
} catch {}
if (-not $isArm64) {
    if ($env:PROCESSOR_ARCHITEW6432 -eq 'ARM64' -or $env:PROCESSOR_ARCHITECTURE -eq 'ARM64') {
        $isArm64 = $true
    }
}

$downloadUrl = if ($isArm64) { $urlArm64 } else { $url64 }
$arch        = if ($isArm64) { 'arm64' } else { 'amd64' }

Write-Host "Detected architecture: windows-$arch"
Write-Host "Downloading: $downloadUrl"

$destExe = Join-Path $toolsDir 'tasklynx.exe'
$destTx  = Join-Path $toolsDir 'tx.exe'

$packageArgs = @{
  packageName   = 'tasklynx'
  fileFullPath  = $destExe
  url           = $downloadUrl
  url64bit      = $url64
  checksum64    = 'c6b92ef1448d12dac8ed66ba51f594a5ff65f85e311f5c0fcc955ec161be03ec'
  checksumType64= 'sha256'
}

# Download the correct binary for detected arch
Get-ChocolateyWebFile -PackageName 'tasklynx' `
    -FileFullPath $destExe `
    -Url $downloadUrl `
    -Url64bit $url64 `
    -Checksum $packageArgs.checksum64 `
    -ChecksumType 'sha256'

# Verify it is a real Windows PE binary (MZ header)
$bytes = [System.IO.File]::ReadAllBytes($destExe)
if ($bytes.Length -lt 1024 -or $bytes[0] -ne 0x4D -or $bytes[1] -ne 0x5A) {
    throw "Downloaded file is not a valid Windows executable. Check the release URL."
}

# Create tx alias shim
Copy-Item $destExe $destTx -Force
Write-Host "Installed tasklynx.exe and tx.exe to $toolsDir" -ForegroundColor Green
