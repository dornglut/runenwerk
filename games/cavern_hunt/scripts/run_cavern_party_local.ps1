param(
    [ValidateRange(1, 4)]
    [int]$ClientCount = 2,
    [switch]$LiveLogWindows
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

function Start-LogTailWindow {
    param(
        [string]$Title,
        [string]$LogPath
    )

    $pwshPath = (Get-Process -Id $PID).Path
    $safeTitle = $Title.Replace("'", "''")
    $safeLogPath = $LogPath.Replace("'", "''")
    $tailCommand = @"
`$Host.UI.RawUI.WindowTitle = '$safeTitle'
if (-not (Test-Path '$safeLogPath')) {
    New-Item -ItemType File -Path '$safeLogPath' -Force | Out-Null
}
Write-Host \"Tailing $safeLogPath\"
Get-Content -Path '$safeLogPath' -Wait -Tail 40
"@

    Start-Process -FilePath $pwshPath -ArgumentList @("-NoLogo", "-NoProfile", "-Command", $tailCommand) | Out-Null
}

$scriptDir = Split-Path -Parent $PSCommandPath
$rootDir = Get-RepoRoot -ScriptDir $scriptDir
Set-Location $rootDir

$netProfileDefault = if ($ClientCount -ge 3) { "four_local_conservative" } else { "two_local_balanced" }
$netProfile = if ([string]::IsNullOrWhiteSpace($env:CAVERN_NET_PROFILE)) { $netProfileDefault } else { $env:CAVERN_NET_PROFILE }
$clientConfigPath = if (-not [string]::IsNullOrWhiteSpace($env:CAVERN_CLIENT_CONFIG_PATH)) {
    $env:CAVERN_CLIENT_CONFIG_PATH
} else {
    "games/cavern_hunt/assets/networking/client/$netProfile.ron"
}
$serverConfigPath = if (-not [string]::IsNullOrWhiteSpace($env:CAVERN_SERVER_CONFIG_PATH)) {
    $env:CAVERN_SERVER_CONFIG_PATH
} else {
    "games/cavern_hunt/assets/networking/server/$netProfile.ron"
}

if (-not (Test-Path $clientConfigPath)) {
    throw "Missing client config at $clientConfigPath"
}
if (-not (Test-Path $serverConfigPath)) {
    throw "Missing server config at $serverConfigPath"
}

$logDir = if (-not [string]::IsNullOrWhiteSpace($env:CAVERN_LOG_DIR)) {
    $env:CAVERN_LOG_DIR
} elseif (-not [string]::IsNullOrWhiteSpace($env:TEMP)) {
    $env:TEMP
} else {
    Join-Path $rootDir "logs"
}
New-Item -ItemType Directory -Force -Path $logDir | Out-Null

$useRelease = Test-EnvFlag -Value $env:CAVERN_RELEASE -Default $true
$showChildWindows = Test-EnvFlag -Value $env:CAVERN_SHOW_CHILD_WINDOWS -Default $false
$liveLogWindowsEnabled = $LiveLogWindows -or (Test-EnvFlag -Value $env:CAVERN_LIVE_LOG_WINDOWS -Default $false)
$clientMaterialProfile = if ([string]::IsNullOrWhiteSpace($env:CAVERN_MATERIAL_PROFILE)) {
    "performance"
} else {
    $env:CAVERN_MATERIAL_PROFILE
}
$clientStartStaggerSeconds = if ([string]::IsNullOrWhiteSpace($env:CAVERN_CLIENT_START_STAGGER_SECONDS)) {
    0.15
} else {
    [double]$env:CAVERN_CLIENT_START_STAGGER_SECONDS
}

$buildArgs = @("build")
if ($useRelease) {
    $buildArgs += "--release"
}
$buildArgs += @("-p", "grotto_server", "-p", "grotto_client")
Write-Host "Building binaries once for local party..."
& cargo @buildArgs
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}

$profileDir = if ($useRelease) { "release" } else { "debug" }
$serverBin = Resolve-BinaryPath -RootDir $rootDir -ProfileDir $profileDir -Name "grotto_server"
$clientBin = Resolve-BinaryPath -RootDir $rootDir -ProfileDir $profileDir -Name "grotto_client"
if (-not (Test-Path $serverBin)) {
    throw "Missing server binary at $serverBin"
}
if (-not (Test-Path $clientBin)) {
    throw "Missing client binary at $clientBin"
}

$serverLog = Join-Path $logDir "cavern_hunt_server.log"
$serverErr = Join-Path $logDir "cavern_hunt_server.err.log"
if (Test-Path $serverLog) { Remove-Item $serverLog -Force }
if (Test-Path $serverErr) { Remove-Item $serverErr -Force }

$serverConfigText = Get-Content $serverConfigPath -Raw
$certPath = "var/dev/server-cert.der"
if ($serverConfigText -match 'cert_output_path:\s*"([^"]+)"') {
    $certPath = $Matches[1]
}
if (Test-Path $certPath) {
    Remove-Item $certPath -Force
}

$serverStartArgs = @{
    FilePath     = $serverBin
    ArgumentList = @("--config", $serverConfigPath)
    PassThru     = $true
}
if (-not $showChildWindows -or $liveLogWindowsEnabled) {
    $serverStartArgs["RedirectStandardOutput"] = $serverLog
    $serverStartArgs["RedirectStandardError"] = $serverErr
}
if (-not $showChildWindows -or $liveLogWindowsEnabled) {
    $serverStartArgs["WindowStyle"] = "Hidden"
}
$serverProcess = Start-Process @serverStartArgs
$clientProcesses = @()

try {
    $certReady = $false
    for ($i = 0; $i -lt 15; $i++) {
        if ($serverProcess.HasExited) {
            throw "Server process exited early. Check $serverLog and $serverErr."
        }
        if (Test-Path $certPath) {
            $certReady = $true
            break
        }
        Start-Sleep -Seconds 1
    }
    if (-not $certReady) {
        throw "Server certificate was not generated at $certPath. Check $serverLog and $serverErr."
    }

    Write-Host "Server started (pid $($serverProcess.Id)). Launching $ClientCount client(s)."
    Write-Host "Server logs: $serverLog, $serverErr"
    Write-Host "Network profile: $netProfile"
    Write-Host "Client material profile: $clientMaterialProfile"
    if ($liveLogWindowsEnabled) {
        Start-LogTailWindow -Title "Cavern Server Log" -LogPath $serverLog
    }

    for ($index = 1; $index -le $ClientCount; $index++) {
        $clientLog = Join-Path $logDir "cavern_hunt_client_$index.log"
        $clientErr = Join-Path $logDir "cavern_hunt_client_$index.err.log"
        if (Test-Path $clientLog) { Remove-Item $clientLog -Force }
        if (Test-Path $clientErr) { Remove-Item $clientErr -Force }
        $envBlock = @{
            CAVERN_MATERIAL_PROFILE = $clientMaterialProfile
        }
        $clientStartArgs = @{
            FilePath     = $clientBin
            ArgumentList = @("--config", $clientConfigPath)
            Environment  = $envBlock
            PassThru     = $true
        }
        if (-not $showChildWindows -or $liveLogWindowsEnabled) {
            $clientStartArgs["RedirectStandardOutput"] = $clientLog
            $clientStartArgs["RedirectStandardError"] = $clientErr
        }
        if (-not $showChildWindows -or $liveLogWindowsEnabled) {
            $clientStartArgs["WindowStyle"] = "Hidden"
        }
        $client = Start-Process @clientStartArgs
        $clientProcesses += $client
        if ($liveLogWindowsEnabled) {
            Start-LogTailWindow -Title "Cavern Client $index Log" -LogPath $clientLog
        }
        if ($index -lt $ClientCount) {
            Start-Sleep -Seconds $clientStartStaggerSeconds
        }
    }

    Wait-Process -Id $serverProcess.Id
} finally {
    if ($serverProcess -and -not $serverProcess.HasExited) {
        Stop-Process -Id $serverProcess.Id -Force
    }
}
