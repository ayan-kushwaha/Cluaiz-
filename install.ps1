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
    "Silicon Mastery achieved. Extracting every bit of performance."
)

function Write-Step ([string]$msg) { Write-Host "  $GRAY[*] $msg$NC" }
function Write-Success ([string]$msg) { Write-Host "  $GREEN[OK] $msg$NC" }
function Write-Warn ([string]$msg) { Write-Host "  $YELLOW[!] $msg$NC" }
function Write-Error ([string]$msg) { Write-Host "  $RED[ERR] $msg$NC" }

# --- Core Robustness Engine ---
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12 -bor [Net.SecurityProtocolType]::Tls13

function Refresh-Env {
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "User")
}

# --- Premium UI Header ---
Clear-Host
Write-Host ""
Write-Host "  $CYAN$BOLD ──────────────────────────────────────────$NC"
Write-Host "  $CYAN$BOLD    CLUAIZ CORE INFRASTRUCTURE $NC"
Write-Host "  $CYAN$BOLD ──────────────────────────────────────────$NC"
$SelectedTagline = Get-Random -InputObject $Taglines
Write-Host "  $GRAY  $SelectedTagline $NC"
Write-Host ""

try {
    # 1. Environment Verification
    $IsAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
    if ($IsAdmin) {
        Write-Success "Elevated privileges detected."
    } else {
        Write-Warn "Non-Admin context. Path updates localized to User."
    }

    $HubPath = Join-Path $HOME ".cluaiz"
    $Repo = "cluaiz/cluaiz"
    Write-Step "Initializing workspace at: $HubPath"

    # 2. Filesystem Provisioning
    $Folders = @("bin", "apps/cli", "interface-engines")
    foreach ($f in $Folders) {
        $path = Join-Path $HubPath $f
        if (-not (Test-Path $path)) {
            New-Item -ItemType Directory -Path $path -Force | Out-Null
        }
    }

    # 3. Environment Variable Registration
    [System.Environment]::SetEnvironmentVariable("CLUAIZ_ROOT", $HubPath, "User")
    $env:CLUAIZ_ROOT = $HubPath

    $BinPath = Join-Path $HubPath "bin"
    $OldPath = [System.Environment]::GetEnvironmentVariable("Path", "User")
    if ($OldPath -notlike "*$BinPath*") {
        Write-Step "Registering binary path..."
        [System.Environment]::SetEnvironmentVariable("Path", "$OldPath;$BinPath", "User")
        Refresh-Env
    }

    # 4. Core Binary Retrieval
    $AppPath = Join-Path $HubPath "apps/cli/cluaiz.exe"
    $BinLink = Join-Path $HubPath "bin/cluaiz.exe"

    Write-Step "Syncing with Sovereign Registry..."
    if ($Version -eq "latest") {
        $Releases = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases"
        $CliRelease = $Releases | Where-Object { $_.tag_name -like "cli-v*" } | Select-Object -First 1
    } else {
        $TargetTag = if ($Version -notlike "cli-*") { "cli-$Version" } else { $Version }
        $CliRelease = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/tags/$TargetTag"
    }
    
    if ($null -eq $CliRelease) { throw "Could not resolve release manifest." }
    Write-Success "Active Channel: $($CliRelease.tag_name)"

    $ManifestAsset = $CliRelease.assets | Where-Object { $_.name -eq "cli-manifest.json" }
    if ($null -eq $ManifestAsset) { throw "Manifest asset missing." }

    $Manifest = Invoke-RestMethod -Uri $ManifestAsset.browser_download_url
    
    $RawArch = $env:PROCESSOR_ARCHITECTURE
    $Arch = if ($RawArch -eq "ARM64") { "win-arm64" } else { "win-x64" }
    
    $CliUrl = $Manifest.binaries.$Arch
    if ($null -eq $CliUrl) { throw "No binary mapped for architecture: $Arch" }

    Write-Step "Downloading Cluaiz CLI ($Arch)..."
    Invoke-WebRequest -Uri $CliUrl -OutFile $AppPath -ProgressAction SilentlyContinue

    if (Test-Path $BinLink) { Remove-Item $BinLink -Force }
    cmd /c mklink /H "$BinLink" "$AppPath" | Out-Null

    Write-Host ""
    Write-Host "  $CYAN$BOLD ──────────────────────────────────────────$NC"
    Write-Host "  $CYAN$BOLD    DEPLOYMENT COMPLETE $NC"
    Write-Host "  $CYAN$BOLD ──────────────────────────────────────────$NC"
    Write-Host "  $GRAY  Run 'cluaiz' to ignite your neural engine. $NC"
    Write-Host ""
}
catch {
    Write-Host ""
    Write-Error "Installation aborted."
    Write-Host "  Reason: $($_.Exception.Message)" -ForegroundColor Gray
    Write-Host ""
    exit 1
}
