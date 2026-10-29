$ErrorActionPreference = 'Stop'

$toolsDir     = "$(Split-Path -parent $MyInvocation.MyCommand.Definition)"
$version      = '0.0.1-beta-3'
$url64        = "https://github.com/peekstore/tasklynx/releases/download/v${version}/tasklynx-windows-amd64.exe"
$urlArm64     = "https://github.com/peekstore/tasklynx/releases/download/v${version}/tasklynx-windows-arm64.exe"

$packageArgs = @{
  packageName   = 'tasklynx'
  unzipLocation = $toolsDir
  fileType      = 'exe'
  url64bit      = $url64
  url           = $url64
  softwareName  = 'tasklynx*'
  checksum64    = '61e8ecaa553e75baba1282d953acd174b3667c5bbe0d2b5558d6bb34214e4794'
  checksumType64= 'sha256'
}

# Download standalone binary
Get-ChocolateyWebFile -PackageName 'tasklynx' -FileFullPath "$toolsDir\tasklynx.exe" -Url $url64 -Url64bit $url64 -Checksum $packageArgs.checksum64 -ChecksumType 'sha256'

# Create tx shim copy
Copy-Item "$toolsDir\tasklynx.exe" "$toolsDir\tx.exe" -Force
