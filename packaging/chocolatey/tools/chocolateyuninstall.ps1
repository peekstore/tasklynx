$ErrorActionPreference = 'SilentlyContinue'

$toolsDir = "$(Split-Path -parent $MyInvocation.MyCommand.Definition)"
Remove-Item "$toolsDir\tasklynx.exe" -Force
Remove-Item "$toolsDir\tx.exe" -Force
