# CLUAIZ Core Infrastructure Installer (Windows)
# Standard Deployment Script - Industrial Grade

param (
    [string]$Version = "latest"
)

$ErrorActionPreference = "Stop"

# --- UI & Personality Matrix ---
$BOLD = "$([char]27)[1m"
$CYAN = "$([char]27)[36m"
$GRAY = "$([char]27)[90m"
$GREEN = "$([char]27)[32m"
$YELLOW = "$([char]27)[33m"
$RED = "$([char]27)[31m"
$NC = "$([char]27)[0m"

$Taglines = @(
    "Neural logic initialized. Preparing for ignition.",
    "Bypassing hardware boundaries... Extraction in progress.",
    "Establishing the Sovereign Hub. Secure partition active.",
    "Claws out, logic in. Let's build something impossible.",
    "Your terminal just grew a brain. Minimal fuss, maximal pinch.",
    "Silicon Mastery achieved. Extracting every bit of performance.",
    "Neural Core sync in progress. Stay calibrated."
)

function Write-Step ([string]$msg) { Write-Host "  [*] $msg" -ForegroundColor Gray }
function Write-Success ([string]$msg) { Write-Host "  [OK] $msg" -ForegroundColor Green }
function Write-Warn ([string]$msg) { Write-Host "  [!] $msg" -ForegroundColor Yellow }
function Write-Error ([string]$msg) { Write-Host "  [ERR] $msg" -ForegroundColor Red }

# --- Core Robustness Engine ---

# 1. Force TLS 1.2/1.3 for secure communication
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12 -bor [Net.SecurityProtocolType]::Tls13

# 2. Path Refresh Function
function Refresh-Env {
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "User")
}

# --- Banner ---
Write-Host ""
Write-Host "  $CYAN$BOLD CLUAIZ Core Installer$NC"
$SelectedTagline = Get-Random -InputObject $Taglines
Write-Host "  $GRAY $SelectedTagline $NC"
Write-Host ""

try {
    # 3. Environment Verification
    $IsAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
    if (-not $IsAdmin) {
        Write-Warn "User context: Non-Admin. Path updates localized to User environment."
    }

    $HubPath = Join-Path $HOME ".cluaiz"
    $Repo = "cluaiz/cluaiz"
    Write-Step "Initializing workspace at: $HubPath"

    # 4. Filesystem Provisioning
    $Folders = @("bin", "apps/cli")
    foreach ($f in $Folders) {
        $path = Join-Path $HubPath $f
        if (-not (Test-Path $path)) {
            New-Item -ItemType Directory -Path $path -Force | Out-Null
            Write-Step "Created: $f"
        }
    }

    # 5. Dependency Validation (Git is critical for Skills)
    if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
        Write-Warn "Git not detected. Skills integration may require manual Git setup."
    }

    # 6. Environment Variable Registration
    Write-Step "Configuring CLUAIZ_ROOT..."
    [System.Environment]::SetEnvironmentVariable("CLUAIZ_ROOT", $HubPath, "User")
    $env:CLUAIZ_ROOT = $HubPath

    $BinPath = Join-Path $HubPath "bin"
    $OldPath = [System.Environment]::GetEnvironmentVariable("Path", "User")
    if ($OldPath -notlike "*$BinPath*") {
        Write-Step "Registering binary path..."
        [System.Environment]::SetEnvironmentVariable("Path", "$OldPath;$BinPath", "User")
        Refresh-Env
        Write-Success "PATH synchronized."
    }

    # 7. Core Binary Retrieval
    $AppPath = Join-Path $HubPath "apps/cli/cluaiz.exe"
    $BinLink = Join-Path $HubPath "bin/cluaiz.exe"

    if ($Version -eq "latest") {
        Write-Step "Fetching latest release manifest..."
        $Releases = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases"
        $CliRelease = $Releases | Where-Object { $_.tag_name -like "cli-v*" } | Select-Object -First 1
    } else {
        $TargetTag = if ($Version -notlike "cli-*") { "cli-$Version" } else { $Version }
        Write-Step "Fetching target release: $TargetTag"
        $CliRelease = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/tags/$TargetTag"
    }
    
    if ($null -eq $CliRelease) { throw "Could not resolve release manifest." }

    $Tag = $CliRelease.tag_name
    Write-Success "Active Channel: $Tag"

    $ManifestAsset = $CliRelease.assets | Where-Object { $_.name -eq "cli-manifest.json" }
    if ($null -eq $ManifestAsset) { throw "Manifest asset missing in $Tag." }

    $ManifestUrl = $ManifestAsset.browser_download_url
    $Manifest = Invoke-RestMethod -Uri $ManifestUrl
    
    # Precision Architecture Detection
    $RawArch = $env:PROCESSOR_ARCHITECTURE
    $Arch = if ($RawArch -eq "ARM64") { "win-arm64" } else { "win-x64" }
    
    $CliUrl = $Manifest.binaries.$Arch
    if ($null -eq $CliUrl) { throw "No binary mapped for architecture: $Arch" }

    Write-Step "Downloading Cluaiz CLI ($Arch)..."
    Invoke-WebRequest -Uri $CliUrl -OutFile $AppPath

    if (Test-Path $BinLink) { Remove-Item $BinLink -Force }
    Write-Step "Establishing binary links..."
    cmd /c mklink /H "$BinLink" "$AppPath" | Out-Null

    Write-Host ""
    Write-Host "  $CYAN$BOLD [DONE] CLUAIZ Core successfully initialized. $NC"
}
catch {
    Write-Host ""
    Write-Error "Installation aborted."
    Write-Host "  Reason: $($_.Exception.Message)" -ForegroundColor Gray
    exit 1
}

Write-Host "  $GRAY Deployment sequence complete. $NC"
Write-Host ""
