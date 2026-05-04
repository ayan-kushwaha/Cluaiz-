# CLUAIZ Core Infrastructure Installer (Windows)
# Industrial Standard Deployment Script

param (
    [string]$Version = "latest"
)

$ErrorActionPreference = "Stop"
$ProgressPreference = 'SilentlyContinue'

# --- UI Matrix (Minimalist Industrial) ---
$BOLD = "$([char]27)[1m"; $CYAN = "$([char]27)[36m"; $GRAY = "$([char]27)[90m"; $GREEN = "$([char]27)[32m"; $YELLOW = "$([char]27)[33m"; $RED = "$([char]27)[31m"; $NC = "$([char]27)[0m"

function Write-Step ([string]$msg) { Write-Host "  $GRAY[*] $msg$NC" }
function Write-Success ([string]$msg) { Write-Host "  $GREEN[OK] $msg$NC" }
function Write-Warn ([string]$msg) { Write-Host "  $YELLOW[!] $msg$NC" }
function Write-Fail ([string]$msg) { Write-Host "  $RED[ERR] $msg$NC" -ForegroundColor Red }

[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12 -bor [Net.SecurityProtocolType]::Tls13

# --- Header ---
Clear-Host
Write-Host ""
Write-Host "  $BOLD CLUAIZ CORE INFRASTRUCTURE $NC"
Write-Host "  $GRAY Standard Deployment Sequence $NC"
Write-Host ""

try {
    $HubPath = Join-Path $HOME ".cluaiz"
    $Repo = "cluaiz/cluaiz"

    # 1. Environment Provisioning
    Write-Step "Provisioning environment..."
    $Folders = @("bin", "apps/cli", "interface-engines", "interface-engines/kernels", "interface-engines/drivers")
    foreach ($f in $Folders) {
        $path = Join-Path $HubPath $f
        if (-not (Test-Path $path)) { New-Item -ItemType Directory -Path $path -Force | Out-Null }
    }

    # 2. System Integration
    [System.Environment]::SetEnvironmentVariable("CLUAIZ_ROOT", $HubPath, "User")
    $BinPath = Join-Path $HubPath "bin"
    $OldPath = [System.Environment]::GetEnvironmentVariable("Path", "User")
    if ($OldPath -notlike "*$BinPath*") {
        Write-Step "Registering system path..."
        [System.Environment]::SetEnvironmentVariable("Path", "$OldPath;$BinPath", "User")
    }

    # 3. Artifact Retrieval
    Write-Step "Resolving artifacts from registry..."
    $AllReleases = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases"
    
    $RawArch = $env:PROCESSOR_ARCHITECTURE
    $Arch = if ($RawArch -eq "ARM64") { "win-arm64" } else { "win-x64" }

    $CliRelease = $AllReleases | Where-Object { $_.tag_name -like "cli-v*" } | Select-Object -First 1
    if ($null -eq $CliRelease) { throw "CLI release tag not found." }
    $CliTag = $CliRelease.tag_name
    $CliManifestUrl = "https://github.com/$Repo/releases/download/$CliTag/cli-manifest.json"
    
    $CliManifest = Invoke-WebRequest -Uri $CliManifestUrl -UseBasicParsing | Select-Object -ExpandProperty Content | ConvertFrom-Json
    
    if ($null -eq $CliManifest.binaries) { throw "CLI manifest is structurally invalid." }
    
    Write-Step "Retrieving Cluaiz CLI ($Arch)..."
    $CliUrl = $CliManifest.binaries.($Arch)
    if ($null -eq $CliUrl -or $CliUrl -eq "") { throw "CLI binary URL not found for $Arch in manifest." }
    Invoke-WebRequest -Uri $CliUrl -OutFile (Join-Path $HubPath "apps/cli/cluaiz.exe")
    $BinLink = Join-Path $BinPath 'cluaiz.exe'
    if (Test-Path $BinLink) { Remove-Item $BinLink -Force }
    cmd /c mklink /H "$BinLink" "$(Join-Path $HubPath 'apps/cli/cluaiz.exe')" | Out-Null

    # --- Engine ---
    $EngineRelease = $AllReleases | Where-Object { $_.tag_name -like "engine-v*" } | Select-Object -First 1
    if ($null -eq $EngineRelease) { throw "Engine release tag not found." }
    $EngineTag = $EngineRelease.tag_name
    $EngineManifestUrl = "https://github.com/$Repo/releases/download/$EngineTag/engine-manifest.json"
    
    $EngineManifest = Invoke-WebRequest -Uri $EngineManifestUrl -UseBasicParsing | Select-Object -ExpandProperty Content | ConvertFrom-Json
    if ($null -eq $EngineManifest.binaries) { throw "Engine manifest is structurally invalid." }
    
    Write-Step "Retrieving Neural Engine..."
    $EngineUrl = $EngineManifest.binaries.($Arch)
    if ($null -eq $EngineUrl -or $EngineUrl -eq "") { throw "Engine binary URL not found for $Arch in manifest." }
    Invoke-WebRequest -Uri $EngineUrl -OutFile (Join-Path $HubPath "interface-engines/cluaiz-engine.dll")

    # --- Default Kernel ---
    $KernelRelease = $AllReleases | Where-Object { $_.tag_name -like "kernel-v*" } | Select-Object -First 1
    if ($null -eq $KernelRelease) { throw "Kernel release tag not found." }
    $KernelTag = $KernelRelease.tag_name
    $KernelManifestUrl = "https://github.com/$Repo/releases/download/$KernelTag/kernel-manifest.json"
    
    $KernelManifest = Invoke-WebRequest -Uri $KernelManifestUrl -UseBasicParsing | Select-Object -ExpandProperty Content | ConvertFrom-Json
    if ($null -eq $KernelManifest.kernels) { throw "Kernel manifest is structurally invalid." }
    
    $Key = "$($Arch)-cuda"
    $KernelUrl = $KernelManifest.kernels.llama.($Key)
    if ($null -eq $KernelUrl -or $KernelUrl -eq "") { 
        $Key = "$($Arch)-cpu"
        $KernelUrl = $KernelManifest.kernels.llama.($Key)
    }
    
    if ($null -ne $KernelUrl -and $KernelUrl -ne "") {
        Write-Step "Retrieving Neural Kernel..."
        Invoke-WebRequest -Uri $KernelUrl -OutFile (Join-Path $HubPath "interface-engines/kernels/archer_llama.dll")
    }

    Write-Host ""
    Write-Success "Deployment successful."
    Write-Host "  Path: $HubPath $NC"
    Write-Host "  Launching Cluaiz CLI...$NC"
    Write-Host ""
    
    # Launch CLI
    & "$BinLink"
}
catch {
    Write-Host ""
    Write-Fail "Deployment failed: $($_.Exception.Message)"
    Write-Host "  Press any key to exit..." -ForegroundColor Gray
    $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
}