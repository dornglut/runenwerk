#!/usr/bin/env pwsh
<#
Bootstrap or verify the Runenwerk developer toolchain on Windows.

File: tools/bootstrap/bootstrap.ps1
Module: bootstrap
#>

[CmdletBinding()]
param(
    [switch]$CheckOnly,
    [switch]$SkipRustTools,
    [switch]$SkipNodeTools
)

$ErrorActionPreference = "Stop"
$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..")
Set-Location $RepoRoot

$CargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
$NodeBin = Join-Path $env:ProgramFiles "nodejs"
$WinGetLinks = Join-Path $env:LOCALAPPDATA "Microsoft\WinGet\Links"
$WindowsApps = Join-Path $env:LOCALAPPDATA "Microsoft\WindowsApps"
$RunenwerkBin = Join-Path $env:USERPROFILE ".runenwerk\bin"
$WinGetPackages = @(
    Join-Path $env:LOCALAPPDATA "Microsoft\WinGet\Packages"
    Join-Path $env:ProgramFiles "WinGet\Packages"
)
$PythonScripts = @()
$PythonRoot = Join-Path $env:APPDATA "Python"
if (Test-Path $PythonRoot) {
    $PythonScripts = Get-ChildItem -Path $PythonRoot -Directory -Filter "Python*" |
        ForEach-Object { Join-Path $_.FullName "Scripts" }
}

foreach ($PathEntry in @($RunenwerkBin, $CargoBin, $NodeBin, $WinGetLinks, $WindowsApps) + $PythonScripts) {
    if ((Test-Path $PathEntry) -and ($env:Path -notlike "*$PathEntry*")) {
        $env:Path = "$PathEntry;$env:Path"
    }
}

$Missing = New-Object System.Collections.Generic.List[string]
$Installed = New-Object System.Collections.Generic.List[string]

function Test-Tool {
    param([Parameter(Mandatory = $true)][string]$Name)
    return $null -ne (Get-Command $Name -ErrorAction SilentlyContinue)
}

function Test-ToolProbe {
    param(
        [Parameter(Mandatory = $true)][string]$Name,
        [string[]]$ProbeArgs = @("--version")
    )
    if (-not (Test-Tool $Name)) {
        return $false
    }
    $PreviousErrorActionPreference = $ErrorActionPreference
    try {
        $ErrorActionPreference = "Continue"
        $global:LASTEXITCODE = 0
        & $Name @ProbeArgs *> $null
        return $LASTEXITCODE -eq 0
    } catch {
        return $false
    } finally {
        $ErrorActionPreference = $PreviousErrorActionPreference
    }
}

function Write-Ok {
    param([string]$Message)
    Write-Host "[ok] $Message" -ForegroundColor Green
}

function Write-Warn {
    param([string]$Message)
    Write-Host "[warn] $Message" -ForegroundColor Yellow
}

function Write-Fail {
    param([string]$Message)
    Write-Host "[missing] $Message" -ForegroundColor Red
}

function Install-WingetPackage {
    param(
        [Parameter(Mandatory = $true)][string]$Id,
        [Parameter(Mandatory = $true)][string]$DisplayName
    )
    if (-not (Test-Tool "winget")) {
        throw "winget is required to install $DisplayName. Install App Installer / Windows Package Manager first."
    }
    winget install --id $Id --exact --source winget --accept-source-agreements --accept-package-agreements
}

function Find-WinGetPackageExecutable {
    param([Parameter(Mandatory = $true)][string]$ExecutableName)
    foreach ($PackageRoot in $WinGetPackages) {
        if (-not (Test-Path $PackageRoot)) {
            continue
        }
        $Match = Get-ChildItem -Path $PackageRoot -Recurse -Filter $ExecutableName -File -ErrorAction SilentlyContinue |
            Select-Object -First 1
        if ($Match) {
            return $Match.FullName
        }
    }
    return $null
}

function Ensure-UserBinOnPath {
    if (-not (Test-Path $RunenwerkBin)) {
        New-Item -ItemType Directory -Force -Path $RunenwerkBin | Out-Null
    }
    if ($env:Path -notlike "*$RunenwerkBin*") {
        $env:Path = "$RunenwerkBin;$env:Path"
    }

    $UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ([string]::IsNullOrWhiteSpace($UserPath)) {
        [Environment]::SetEnvironmentVariable("Path", $RunenwerkBin, "User")
        return
    }
    $PathParts = $UserPath -split ";" | Where-Object { -not [string]::IsNullOrWhiteSpace($_) }
    if ($PathParts -notcontains $RunenwerkBin) {
        [Environment]::SetEnvironmentVariable("Path", "$RunenwerkBin;$UserPath", "User")
    }
}

function Ensure-CmdShim {
    param(
        [Parameter(Mandatory = $true)][string]$Name,
        [Parameter(Mandatory = $true)][string]$Target
    )
    Ensure-UserBinOnPath
    $ShimPath = Join-Path $RunenwerkBin "$Name.cmd"
    $Shim = "@echo off`r`n`"$Target`" %*`r`n"
    Set-Content -Path $ShimPath -Value $Shim -Encoding ASCII -NoNewline
}

function Ensure-Tool {
    param(
        [Parameter(Mandatory = $true)][string]$Name,
        [Parameter(Mandatory = $true)][string]$DisplayName,
        [Parameter(Mandatory = $true)][scriptblock]$Install,
        [string[]]$ProbeArgs = @("--version")
    )

    if (Test-ToolProbe -Name $Name -ProbeArgs $ProbeArgs) {
        Write-Ok "$DisplayName is available"
        return
    }

    if ($CheckOnly) {
        Write-Fail "$DisplayName ($Name)"
        $Missing.Add($DisplayName) | Out-Null
        return
    }

    Write-Warn "Installing $DisplayName"
    & $Install
    $Installed.Add($DisplayName) | Out-Null

    if (Test-ToolProbe -Name $Name -ProbeArgs $ProbeArgs) {
        Write-Ok "$DisplayName is available"
    } else {
        Write-Fail "$DisplayName did not appear on PATH. Restart the shell and run tools/bootstrap/bootstrap.ps1 -CheckOnly."
        $Missing.Add($DisplayName) | Out-Null
    }
}

function Ensure-CargoTool {
    param(
        [Parameter(Mandatory = $true)][string]$Binary,
        [Parameter(Mandatory = $true)][string]$DisplayName,
        [Parameter(Mandatory = $true)][string[]]$InstallArgs
    )

    Ensure-Tool -Name $Binary -DisplayName $DisplayName -Install {
        if (-not (Test-Tool "cargo")) {
            throw "cargo is required to install $DisplayName."
        }
        cargo install @InstallArgs
    }
}

Ensure-Tool -Name "git" -DisplayName "Git" -Install {
    Install-WingetPackage -Id "Git.Git" -DisplayName "Git"
}

Ensure-Tool -Name "uv" -DisplayName "uv" -Install {
    powershell -ExecutionPolicy ByPass -NoProfile -Command "irm https://astral.sh/uv/install.ps1 | iex"
}

Ensure-Tool -Name "task" -DisplayName "Task" -Install {
    Install-WingetPackage -Id "Task.Task" -DisplayName "Task"
}

Ensure-Tool -Name "cargo" -DisplayName "Rust Cargo" -Install {
    Install-WingetPackage -Id "Rustlang.Rustup" -DisplayName "Rustup"
    if (Test-Tool "rustup") {
        rustup toolchain install stable
        rustup default stable
    }
}

if (-not $SkipRustTools) {
    Ensure-CargoTool -Binary "cargo-nextest" -DisplayName "cargo-nextest" -InstallArgs @("cargo-nextest", "--locked")
    Ensure-CargoTool -Binary "cargo-deny" -DisplayName "cargo-deny" -InstallArgs @("cargo-deny", "--locked")
    Ensure-CargoTool -Binary "cargo-machete" -DisplayName "cargo-machete" -InstallArgs @("cargo-machete")
}

Ensure-Tool -Name "lychee" -DisplayName "lychee" -Install {
    $LycheeExe = Find-WinGetPackageExecutable -ExecutableName "lychee.exe"
    if (-not $LycheeExe) {
        Install-WingetPackage -Id "lycheeverse.lychee" -DisplayName "lychee"
        $LycheeExe = Find-WinGetPackageExecutable -ExecutableName "lychee.exe"
    }
    if (-not $LycheeExe) {
        throw "lychee installed but lychee.exe was not found under WinGet package roots."
    }
    Ensure-CmdShim -Name "lychee" -Target $LycheeExe
}

Ensure-Tool -Name "plantuml" -DisplayName "PlantUML" -Install {
    throw "Install PlantUML manually, for example with Chocolatey: choco install plantuml -y"
} -ProbeArgs @("-version")

Ensure-Tool -Name "node" -DisplayName "Node.js" -Install {
    Install-WingetPackage -Id "OpenJS.NodeJS.LTS" -DisplayName "Node.js LTS"
}

Ensure-Tool -Name "npm" -DisplayName "npm" -Install {
    Install-WingetPackage -Id "OpenJS.NodeJS.LTS" -DisplayName "Node.js LTS"
}

if (-not $SkipNodeTools) {
    Ensure-Tool -Name "ast-grep" -DisplayName "ast-grep" -Install {
        if (-not (Test-Tool "npm")) {
            throw "npm is required to install ast-grep."
        }
        npm install -g "@ast-grep/cli"
    }

    Ensure-Tool -Name "renovate" -DisplayName "Renovate CLI" -Install {
        if (-not (Test-Tool "npm")) {
            throw "npm is required to install Renovate."
        }
        npm install -g "renovate"
    }
}

Write-Host ""
if ($Installed.Count -gt 0) {
    Write-Host "Installed or attempted:" -ForegroundColor Cyan
    foreach ($Tool in $Installed) {
        Write-Host "- $Tool"
    }
}

if ($Missing.Count -gt 0) {
    Write-Host ""
    Write-Host "Toolchain is incomplete." -ForegroundColor Red
    foreach ($Tool in $Missing) {
        Write-Host "- $Tool"
    }
    exit 1
}

Write-Host "Runenwerk toolchain is ready." -ForegroundColor Green
