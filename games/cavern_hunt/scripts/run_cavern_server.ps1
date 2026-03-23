param(
    [string]$ConfigPath,
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$PassThroughArgs
)

$ErrorActionPreference = "Stop"

function Get-RepoRoot {
    param([string]$ScriptDir)

    try {
        $root = git -C $ScriptDir rev-parse --show-toplevel 2>$null
        if ($LASTEXITCODE -eq 0 -and -not [string]::IsNullOrWhiteSpace($root)) {
            return $root.Trim()
        }
    } catch {
    }

    return (Resolve-Path (Join-Path $ScriptDir "..\..\..")).Path
}

function Test-EnvFlag {
    param(
        [string]$Value,
        [bool]$Default
    )

    if ([string]::IsNullOrWhiteSpace($Value)) {
        return $Default
    }

    switch -Regex ($Value.Trim().ToLowerInvariant()) {
        "^(1|true|yes|on)$" { return $true }
        "^(0|false|no|off)$" { return $false }
        default { return $Default }
    }
}

function Resolve-BinaryPath {
    param(
        [string]$RootDir,
        [string]$ProfileDir,
        [string]$Name
    )

    $exePath = Join-Path $RootDir "target\$ProfileDir\$Name.exe"
    if (Test-Path $exePath) {
        return $exePath
    }
    $rawPath = Join-Path $RootDir "target\$ProfileDir\$Name"
    if (Test-Path $rawPath) {
        return $rawPath
    }
    return $exePath
}

$scriptDir = Split-Path -Parent $PSCommandPath
$rootDir = Get-RepoRoot -ScriptDir $scriptDir
Set-Location $rootDir

$netProfile = if ([string]::IsNullOrWhiteSpace($env:CAVERN_NET_PROFILE)) { "local_dev" } else { $env:CAVERN_NET_PROFILE }
if ([string]::IsNullOrWhiteSpace($ConfigPath)) {
    if (-not [string]::IsNullOrWhiteSpace($env:CAVERN_SERVER_CONFIG_PATH)) {
        $ConfigPath = $env:CAVERN_SERVER_CONFIG_PATH
    } else {
        $ConfigPath = "games/cavern_hunt/assets/networking/server/$netProfile.ron"
    }
}

if (-not (Test-Path $ConfigPath)) {
    throw "Missing server config at $ConfigPath. Set CAVERN_SERVER_CONFIG_PATH or CAVERN_NET_PROFILE."
}

Write-Host "Starting Cavern Hunt dedicated server with config $ConfigPath"

$useRelease = Test-EnvFlag -Value $env:CAVERN_RELEASE -Default $true
$usePrebuilt = Test-EnvFlag -Value $env:CAVERN_USE_PREBUILT -Default $false
$cargoProfileArgs = @()
if ($useRelease) {
    $cargoProfileArgs += "--release"
}

if ($usePrebuilt) {
    $binProfileDir = if ($useRelease) { "release" } else { "debug" }
    $binPath = Resolve-BinaryPath -RootDir $rootDir -ProfileDir $binProfileDir -Name "grotto_server"
    if (-not (Test-Path $binPath)) {
        throw "Missing prebuilt server binary at $binPath. Build first with: cargo build $($cargoProfileArgs -join ' ') -p grotto_server"
    }
    & $binPath --config $ConfigPath @PassThroughArgs
    exit $LASTEXITCODE
}

$cargoArgs = @("run") + $cargoProfileArgs + @("-p", "grotto_server", "--", "--config", $ConfigPath) + $PassThroughArgs
& cargo @cargoArgs
exit $LASTEXITCODE
