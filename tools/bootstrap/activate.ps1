#!/usr/bin/env pwsh
<#
Activate the Runenwerk host toolchain paths in the current PowerShell session.

File: tools/bootstrap/activate.ps1
Module: activate
#>

[CmdletBinding()]
param()

$ErrorActionPreference = "Stop"

$CargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
$NodeBin = Join-Path $env:ProgramFiles "nodejs"
$WinGetLinks = Join-Path $env:LOCALAPPDATA "Microsoft\WinGet\Links"
$WindowsApps = Join-Path $env:LOCALAPPDATA "Microsoft\WindowsApps"
$RunenwerkBin = Join-Path $env:USERPROFILE ".runenwerk\bin"

$PythonScripts = @()
$PythonRoot = Join-Path $env:APPDATA "Python"
if (Test-Path $PythonRoot) {
    $PythonScripts = Get-ChildItem -Path $PythonRoot -Directory -Filter "Python*" |
        ForEach-Object { Join-Path $_.FullName "Scripts" }
}

function Add-SessionPath {
    param([Parameter(Mandatory = $true)][string]$PathEntry)
    if ((Test-Path $PathEntry) -and ($env:Path -notlike "*$PathEntry*")) {
        $env:Path = "$PathEntry;$env:Path"
    }
}

foreach ($PathEntry in @($RunenwerkBin, $CargoBin, $NodeBin, $WinGetLinks, $WindowsApps) + $PythonScripts) {
    Add-SessionPath -PathEntry $PathEntry
}

$Tools = @("uv", "task", "cargo", "plantuml", "lychee", "ast-grep")
$Missing = @()
foreach ($Tool in $Tools) {
    if (-not (Get-Command $Tool -ErrorAction SilentlyContinue)) {
        $Missing += $Tool
    }
}

if ($Missing.Count -gt 0) {
    Write-Host "Runenwerk toolchain paths loaded, but tools are still missing:" -ForegroundColor Yellow
    foreach ($Tool in $Missing) {
        Write-Host "- $Tool"
    }
    Write-Host "Run tools/bootstrap/bootstrap.ps1 to install missing tools." -ForegroundColor Yellow
    $global:LASTEXITCODE = 1
    return
}

Write-Host "Runenwerk toolchain paths loaded." -ForegroundColor Green
