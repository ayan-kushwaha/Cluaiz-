# CLUAIZ CORE INFRASTRUCTURE - VERSION 0.1.0
# Industrial Standard Deployment Script (CURL ENHANCED)

param ([string]$Version = "latest")

$ErrorActionPreference = "Stop"
$ProgressPreference = 'SilentlyContinue'

# --- UI Matrix ---
$BOLD = "$([char]27)[1m"; $CYAN = "$([char]27)[36m"; $GRAY = "$([char]27)[90m"; $GREEN = "$([char]27)[32m"; $YELLOW = "$([char]27)[33m"; $RED = "$([char]27)[31m"; $NC = "$([char]27)[0m"

function Write-Step ([string]$msg) { Write-Host "  $GRAY[*] $msg$NC" }
function Write-Success ([string]$msg) { Write-Host "  $GREEN[OK] $msg$NC" }
function Write-Fail ([string]$msg) { Write-Host "  $RED[ERR] $msg$NC" -ForegroundColor Red }

# --- Robust Download Engine (CURL BASED) ---
function Invoke-SovereignDownload ([string]$url, [string]$path) {
    if (-not $url) { throw "Download URL is null." }
    $dir = Split-Path $path
    if (-not (Test-Path $dir)) { New-Item -ItemType Directory -Path $dir -Force | Out-Null }
    
    $MaxRetries = 3
    $RetryCount = 0
    $Success = $false
    
    while (-not $Success -and $RetryCount -lt $MaxRetries) {
        try {
            # Use curl.exe if available (Win10+), otherwise fallback to iwr
            if (Get-Command "curl.exe" -ErrorAction SilentlyContinue) {
                curl.exe -L -o "$path" "$url" --retry 3 --retry-delay 2 --silent
                if ($LASTEXITCODE -ne 0) { throw "curl exit code: $LASTEXITCODE" }
            } else {
                Invoke-WebRequest -Uri $url -OutFile $path -ErrorAction Stop
            }
            $Success = $true
        }
        catch {
            $RetryCount++
            Write-Host "  $YELLOW[!] Download attempt $RetryCount failed. Retrying...$NC"
            Start-Sleep -Seconds 2
        }
    }
    
    if (-not $Success) { throw "Artifact retrieval failed after $MaxRetries attempts: $url" }
}

# --- Security ---
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12 -bor [Net.SecurityProtocolType]::Tls13

# --- Header ---
Clear-Host
Write-Host "`n  $BOLD CLUAIZ CORE INFRASTRUCTURE (V0.1.0) $NC"
Write-Host "  $GRAY Industrial CURL Deployment $NC`n"

try {
    $HubPath = if ($env:CLUAIZ_ROOT) { $env:CLUAIZ_ROOT } else { Join-Path $HOME ".cluaiz" }
    $Repo = "cluaiz/cluaiz"

    # 1. Provisioning
    Write-Step "Provisioning environment..."
    $Folders = @("bin", "apps/cli", "engine", "interface-engines", "interface-engines/kernels", "interface-engines/drivers")
    foreach ($f in $Folders) {
        $path = Join-Path $HubPath $f
        if (-not (Test-Path $path)) { New-Item -ItemType Directory -Path $path -Force | Out-Null }
    }

    # 2. Registry Handshake
    Write-Step "Resolving artifacts from registry..."
    $Releases = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases"
    $Arch = if ($env:PROCESSOR_ARCHITECTURE -eq "ARM64") { "win-arm64" } else { "win-x64" }

    # --- CLI Deployment ---
    $CliRel = $Releases | Where-Object { $_.tag_name -like "cli-v*" } | Select-Object -First 1
    $CliUrl = "https://github.com/$Repo/releases/download/$($CliRel.tag_name)/cli-manifest.json"
    $CliManifest = Invoke-RestMethod -Uri $CliUrl
    $CliBins = if ($CliManifest.binaries) { $CliManifest.binaries } else { $CliManifest.assets }
    
    Write-Step "Retrieving CLI ($Arch)..."
    Invoke-SovereignDownload -url $CliBins.($Arch) -path (Join-Path $HubPath "apps/cli/cluaiz.exe")
    
    $BinPath = Join-Path $HubPath "bin"
    $BinLink = Join-Path $BinPath 'cluaiz.exe'
    if (Test-Path $BinLink) { Remove-Item $BinLink -Force }
    cmd /c mklink /H "$BinLink" "$(Join-Path $HubPath 'apps/cli/cluaiz.exe')" | Out-Null

    # --- Engine Deployment ---
    $EngRel = $Releases | Where-Object { $_.tag_name -like "engine-v*" } | Select-Object -First 1
    $EngUrl = "https://github.com/$Repo/releases/download/$($EngRel.tag_name)/engine-manifest.json"
    $EngManifest = Invoke-RestMethod -Uri $EngUrl
    $EngBins = if ($EngManifest.binaries) { $EngManifest.binaries } else { $EngManifest.assets }
    
    Write-Step "Retrieving Neural Engine..."
    $EUrl = $EngBins.($Arch) -replace "latest-engine", "$($EngRel.tag_name)"
    Invoke-SovereignDownload -url $EUrl -path (Join-Path $HubPath "engine/cluaiz-engine.dll")

    # --- Kernel Deployment ---
    $KerRel = $Releases | Where-Object { $_.tag_name -like "kernel-v*" } | Select-Object -First 1
    $KerUrl = "https://github.com/$Repo/releases/download/$($KerRel.tag_name)/kernel-manifest.json"
    $KerManifest = Invoke-RestMethod -Uri $KerUrl
    $KerBins = if ($KerManifest.kernels) { $KerManifest.kernels } else { $KerManifest.assets }
    
    $KUrlRaw = $KerBins.llama.("$Arch-cuda")
    if (-not $KUrlRaw) { $KUrlRaw = $KerBins.llama.("$Arch-cpu") }
    
    if ($KUrlRaw) {
        $KUrl = $KUrlRaw -replace "latest-kernels", "$($KerRel.tag_name)"
        Write-Step "Retrieving Neural Kernel..."
        Invoke-SovereignDownload -url $KUrl -path (Join-Path $HubPath "interface-engines/kernels/archer_llama.dll")
    }

    # Environment Path Update
    [System.Environment]::SetEnvironmentVariable("CLUAIZ_ROOT", $HubPath, "User")
    $OldPath = [System.Environment]::GetEnvironmentVariable("Path", "User")
    if ($OldPath -notlike "*$BinPath*") {
        [System.Environment]::SetEnvironmentVariable("Path", "$OldPath;$BinPath", "User")
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