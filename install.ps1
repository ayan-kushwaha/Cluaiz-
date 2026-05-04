# CLUAIZ CORE INFRASTRUCTURE - VERSION 1.0.5
# Industrial Standard Deployment Script

param ([string]$Version = "latest")

$ErrorActionPreference = "Stop"
$ProgressPreference = 'SilentlyContinue'

# --- UI Matrix ---
$BOLD = "$([char]27)[1m"; $CYAN = "$([char]27)[36m"; $GRAY = "$([char]27)[90m"; $GREEN = "$([char]27)[32m"; $YELLOW = "$([char]27)[33m"; $RED = "$([char]27)[31m"; $NC = "$([char]27)[0m"

function Write-Step ([string]$msg) { Write-Host "  $GRAY[*] $msg$NC" }
function Write-Success ([string]$msg) { Write-Host "  $GREEN[OK] $msg$NC" }
function Write-Fail ([string]$msg) { Write-Host "  $RED[ERR] $msg$NC" -ForegroundColor Red }

# --- Security & Protocol ---
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12 -bor [Net.SecurityProtocolType]::Tls13

# --- Header ---
Clear-Host
Write-Host "`n  $BOLD CLUAIZ CORE INFRASTRUCTURE (V1.0.5) $NC"
Write-Host "  $GRAY Standard Deployment Sequence $NC`n"

try {
    $HubPath = Join-Path $HOME ".cluaiz"
    $Repo = "cluaiz/cluaiz"

    # 1. Workspace
    Write-Step "Provisioning environment..."
    $Folders = @("bin", "apps/cli", "interface-engines", "interface-engines/kernels", "interface-engines/drivers")
    foreach ($f in $Folders) {
        $path = Join-Path $HubPath $f
        if (-not (Test-Path $path)) { New-Item -ItemType Directory -Path $path -Force | Out-Null }
    }

    # 2. Path
    $BinPath = Join-Path $HubPath "bin"
    [System.Environment]::SetEnvironmentVariable("CLUAIZ_ROOT", $HubPath, "User")
    $OldPath = [System.Environment]::GetEnvironmentVariable("Path", "User")
    if ($OldPath -notlike "*$BinPath*") {
        [System.Environment]::SetEnvironmentVariable("Path", "$OldPath;$BinPath", "User")
    }

    # 3. Registry
    Write-Step "Resolving artifacts from registry..."
    $Releases = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases"
    $Arch = if ($env:PROCESSOR_ARCHITECTURE -eq "ARM64") { "win-arm64" } else { "win-x64" }

    # --- CLI Deployment ---
    $CliRel = $Releases | Where-Object { $_.tag_name -like "cli-v*" } | Select-Object -First 1
    $CliUrl = "https://github.com/$Repo/releases/download/$($CliRel.tag_name)/cli-manifest.json"
    $CliManifest = Invoke-RestMethod -Uri $CliUrl
    
    $CliBins = if ($CliManifest.binaries) { $CliManifest.binaries } else { $CliManifest.assets }
    if (-not $CliBins) { throw "X01: Artifact map missing in CLI manifest." }
    
    Write-Step "Retrieving CLI ($Arch)..."
    $CliDownloadUrl = $CliBins.($Arch)
    if (-not $CliDownloadUrl) { throw "CLI binary for $Arch missing in manifest." }
    Invoke-WebRequest -Uri $CliDownloadUrl -OutFile (Join-Path $HubPath "apps/cli/cluaiz.exe")
    
    $BinLink = Join-Path $BinPath 'cluaiz.exe'
    if (Test-Path $BinLink) { Remove-Item $BinLink -Force }
    cmd /c mklink /H "$BinLink" "$(Join-Path $HubPath 'apps/cli/cluaiz.exe')" | Out-Null

    # --- Engine Deployment ---
    $EngRel = $Releases | Where-Object { $_.tag_name -like "engine-v*" } | Select-Object -First 1
    $EngUrl = "https://github.com/$Repo/releases/download/$($EngRel.tag_name)/engine-manifest.json"
    $EngManifest = Invoke-RestMethod -Uri $EngUrl
    
    $EngBins = if ($EngManifest.binaries) { $EngManifest.binaries } else { $EngManifest.assets }
    if (-not $EngBins) { throw "X02: Artifact map missing in Engine manifest." }
    
    Write-Step "Retrieving Neural Engine..."
    $EngDownloadUrl = $EngBins.($Arch)
    if (-not $EngDownloadUrl) { throw "Engine binary for $Arch missing in manifest." }
    Invoke-WebRequest -Uri $EngDownloadUrl -OutFile (Join-Path $HubPath "interface-engines/cluaiz-engine.dll")

    # --- Kernel Deployment ---
    $KerRel = $Releases | Where-Object { $_.tag_name -like "kernel-v*" } | Select-Object -First 1
    $KerUrl = "https://github.com/$Repo/releases/download/$($KerRel.tag_name)/kernel-manifest.json"
    $KerManifest = Invoke-RestMethod -Uri $KerUrl
    
    if (-not $KerManifest.kernels) { throw "X03: Kernel map missing in manifest." }
    $KUrl = $KerManifest.kernels.llama.("$Arch-cuda")
    if (-not $KUrl) { $KUrl = $KerManifest.kernels.llama.("$Arch-cpu") }
    
    if ($KUrl) {
        Write-Step "Retrieving Neural Kernel..."
        Invoke-WebRequest -Uri $KUrl -OutFile (Join-Path $HubPath "interface-engines/kernels/archer_llama.dll")
    }

    Write-Host ""
    Write-Success "Deployment successful."
    Write-Host "  Launching Cluaiz CLI..."
    & "$BinLink"
}
catch {
    Write-Host ""
    Write-Fail "Deployment failed: $($_.Exception.Message)"
    Write-Host "`n  [Troubleshoot] Check your connection or GitHub release state." -ForegroundColor Gray
    Write-Host "  Press any key to exit..." -ForegroundColor Gray
    $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
}