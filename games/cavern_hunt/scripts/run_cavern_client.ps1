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
    if (-not [string]::IsNullOrWhiteSpace($env:CAVERN_CLIENT_CONFIG_PATH)) {
        $ConfigPath = $env:CAVERN_CLIENT_CONFIG_PATH
    } else {
        $ConfigPath = "games/cavern_hunt/assets/networking/client/$netProfile.ron"
    }
}

if (-not (Test-Path $ConfigPath)) {
    throw "Missing client config at $ConfigPath. Set CAVERN_CLIENT_CONFIG_PATH or CAVERN_NET_PROFILE."
}

$configText = Get-Content $ConfigPath -Raw
$certPathFromConfig = $null
if ($configText -match 'cert_path:\s*"([^"]+)"') {
    $certPathFromConfig = $Matches[1]
}
$certPath = if (-not [string]::IsNullOrWhiteSpace($env:GROTTO_SERVER_CERT_PATH)) {
    $env:GROTTO_SERVER_CERT_PATH
} elseif (-not [string]::IsNullOrWhiteSpace($certPathFromConfig)) {
    $certPathFromConfig
} else {
    "var/dev/server-cert.der"
}
$waitSeconds = 10
if (-not [string]::IsNullOrWhiteSpace($env:CAVERN_WAIT_FOR_CERT_SECONDS)) {
    $waitSeconds = [int]$env:CAVERN_WAIT_FOR_CERT_SECONDS
}

if (-not (Test-Path $certPath)) {
    Write-Host "Waiting for server certificate at $certPath"
    for ($i = 0; $i -lt $waitSeconds; $i++) {
        if (Test-Path $certPath) {
            break
        }
        Start-Sleep -Seconds 1
    }
}

if (-not (Test-Path $certPath)) {
    throw "Server certificate not found at $certPath. Start the server first with games/cavern_hunt/scripts/run_cavern_server.ps1."
}

$useRelease = Test-EnvFlag -Value $env:CAVERN_RELEASE -Default $true
$usePrebuilt = Test-EnvFlag -Value $env:CAVERN_USE_PREBUILT -Default $false
$cargoProfileArgs = @()
if ($useRelease) {
    $cargoProfileArgs += "--release"
}

if ($usePrebuilt) {
    $binProfileDir = if ($useRelease) { "release" } else { "debug" }
    $binPath = Resolve-BinaryPath -RootDir $rootDir -ProfileDir $binProfileDir -Name "grotto_client"
    if (-not (Test-Path $binPath)) {
        throw "Missing prebuilt client binary at $binPath. Build first with: cargo build $($cargoProfileArgs -join ' ') -p grotto_client"
    }
    & $binPath --config $ConfigPath @PassThroughArgs
    exit $LASTEXITCODE
}

$cargoArgs = @("run") + $cargoProfileArgs + @("-p", "grotto_client", "--", "--config", $ConfigPath) + $PassThroughArgs
& cargo @cargoArgs
exit $LASTEXITCODE
