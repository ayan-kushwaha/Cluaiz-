# CLUAIZ Core Infrastructure Installer (Windows)
# Standard Deployment Script - Industrial Grade

param (
    [string]$Version = "latest"
)

$ErrorActionPreference = "Stop"

# --- UI Matrix ---
$BOLD = "$([char]27)[1m"; $CYAN = "$([char]27)[36m"; $GRAY = "$([char]27)[90m"; $GREEN = "$([char]27)[32m"; $YELLOW = "$([char]27)[33m"; $RED = "$([char]27)[31m"; $NC = "$([char]27)[0m"

function Write-Step ([string]$msg) { Write-Host "  $GRAY[*] $msg$NC" }
function Write-Success ([string]$msg) { Write-Host "  $GREEN[OK] $msg$NC" }
function Write-Warn ([string]$msg) { Write-Host "  $YELLOW[!] $msg$NC" }
function Write-Error ([string]$msg) { Write-Host "  $RED[ERR] $msg$NC" }

[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12 -bor [Net.SecurityProtocolType]::Tls13

# --- Premium Header ---
Clear-Host
Write-Host "`n  $CYAN$BOLD CLUAIZ CORE: SOVEREIGN NEURAL KERNEL$NC"
Write-Host "  $GRAY Establishing silicon-to-registry handshake...$NC`n"

try {
    $HubPath = Join-Path $HOME ".cluaiz"
    $Repo = "cluaiz/cluaiz"

    # 1. Workspace Provisioning
    $Folders = @("bin", "apps/cli", "interface-engines", "interface-engines/kernels", "interface-engines/drivers")
    foreach ($f in $Folders) {
        $path = Join-Path $HubPath $f
        if (-not (Test-Path $path)) { New-Item -ItemType Directory -Path $path -Force | Out-Null }
    }

    # 2. Path Registration
    [System.Environment]::SetEnvironmentVariable("CLUAIZ_ROOT", $HubPath, "User")
    $BinPath = Join-Path $HubPath "bin"
    $OldPath = [System.Environment]::GetEnvironmentVariable("Path", "User")
    if ($OldPath -notlike "*$BinPath*") {
        [System.Environment]::SetEnvironmentVariable("Path", "$OldPath;$BinPath", "User")
    }

    # 3. Registry Discovery
    Write-Step "Discovering latest neural artifacts..."
    $AllReleases = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases"
    
    $RawArch = $env:PROCESSOR_ARCHITECTURE
    $Arch = if ($RawArch -eq "ARM64") { "win-arm64" } else { "win-x64" }

    # --- A. CLI Download ---
    $CliRelease = $AllReleases | Where-Object { $_.tag_name -like "cli-v*" } | Select-Object -First 1
    $CliManifest = Invoke-RestMethod -Uri ($CliRelease.assets | Where-Object { $_.name -eq "cli-manifest.json" }).browser_download_url
    Write-Step "Downloading CLI ($Arch)..."
    Invoke-WebRequest -Uri $CliManifest.binaries.$Arch -OutFile (Join-Path $HubPath "apps/cli/cluaiz.exe") -ProgressAction SilentlyContinue
    cmd /c mklink /H "$(Join-Path $BinPath 'cluaiz.exe')" "$(Join-Path $HubPath 'apps/cli/cluaiz.exe')" | Out-Null

    # --- B. Engine Download ---
    $EngineRelease = $AllReleases | Where-Object { $_.tag_name -like "engine-v*" } | Select-Object -First 1
    $EngineManifest = Invoke-RestMethod -Uri ($EngineRelease.assets | Where-Object { $_.name -eq "engine-manifest.json" }).browser_download_url
    Write-Step "Downloading Neural Engine..."
    Invoke-WebRequest -Uri $EngineManifest.binaries.$Arch -OutFile (Join-Path $HubPath "interface-engines/cluaiz-engine.dll") -ProgressAction SilentlyContinue

    # --- C. Kernel Sync (Default Llama) ---
    $KernelRelease = $AllReleases | Where-Object { $_.tag_name -like "kernel-v*" } | Select-Object -First 1
    $KernelManifest = Invoke-RestMethod -Uri ($KernelRelease.assets | Where-Object { $_.name -eq "kernel-manifest.json" }).browser_download_url
    Write-Step "Provisioning Neural Kernels..."
    # Defaulting to CPU/CUDA based on architecture for first-run
    $KernelUrl = $KernelManifest.kernels.llama."$Arch-cuda" 
    if ($null -eq $KernelUrl) { $KernelUrl = $KernelManifest.kernels.llama."$Arch-cpu" }
    Invoke-WebRequest -Uri $KernelUrl -OutFile (Join-Path $HubPath "interface-engines/kernels/archer_llama.dll") -ProgressAction SilentlyContinue

    Write-Host "`n  $GREEN$BOLD [COMPLETE] Sovereign stack initialized.$NC"
    Write-Host "  $GRAY Path: $HubPath $NC`n"
}
catch {
    Write-Error "Deployment failed: $($_.Exception.Message)"
    exit 1
}
