# CLUAIZ Core Infrastructure Installer (Windows)
# Standard Deployment Script

param (
    [string]$Version = "latest"
)

$ErrorActionPreference = "Stop"
# Force TLS 1.2 for secure GitHub communication
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
$HubPath = Join-Path $HOME ".cluaiz"
$Repo = "cluaiz/cluaiz"

Write-Host "CLUAIZ - Core Infrastructure Installer" -ForegroundColor Cyan
Write-Host "--------------------------------------------------" -ForegroundColor Gray

# 1. Environment Verification
$IsAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $IsAdmin) {
    Write-Warning "User context: Non-Admin. Path updates localized to User environment."
}

Write-Host "Initializing CLUAIZ workspace at: $HubPath" -ForegroundColor Gray

# 2. Filesystem Provisioning
$Folders = @("bin", "apps/cli")
foreach ($f in $Folders) {
    $path = Join-Path $HubPath $f
    if (-not (Test-Path $path)) {
        New-Item -ItemType Directory -Path $path -Force | Out-Null
        Write-Host "Directory created: $f" -ForegroundColor DarkGray
    }
}

# 3. Environment Variables
Write-Host "Configuring CLUAIZ_ROOT..." -ForegroundColor Gray
[System.Environment]::SetEnvironmentVariable("CLUAIZ_ROOT", $HubPath, "User")
$env:CLUAIZ_ROOT = $HubPath

# Path registration
$BinPath = Join-Path $HubPath "bin"
$OldPath = [System.Environment]::GetEnvironmentVariable("Path", "User")
if ($OldPath -notlike "*$BinPath*") {
    Write-Host "Registering binary path..." -ForegroundColor Gray
    [System.Environment]::SetEnvironmentVariable("Path", "$OldPath;$BinPath", "User")
    $env:Path = "$env:Path;$BinPath"
}

# 4. Binary Retrieval
$AppPath = Join-Path $HubPath "apps/cli/cluaiz.exe"
$BinLink = Join-Path $HubPath "bin/cluaiz.exe"

try {
    if ($Version -eq "latest") {
        Write-Host "Fetching latest release manifest..." -ForegroundColor Gray
        $Releases = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases"
        $CliRelease = $Releases | Where-Object { $_.tag_name -like "cli-v*" } | Select-Object -First 1
    } else {
        $TargetTag = if ($Version -notlike "cli-*") { "cli-$Version" } else { $Version }
        Write-Host "Fetching target release: $TargetTag" -ForegroundColor Gray
        $CliRelease = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/tags/$TargetTag"
    }
    
    if ($null -eq $CliRelease) {
        throw "Specified release not found."
    }

    $Tag = $CliRelease.tag_name
    Write-Host "Active version: $Tag" -ForegroundColor Green

    $ManifestAsset = $CliRelease.assets | Where-Object { $_.name -eq "cli-manifest.json" }
    if ($null -eq $ManifestAsset) {
        throw "Manifest asset missing in $Tag."
    }

    $ManifestUrl = $ManifestAsset.browser_download_url
    $Manifest = Invoke-RestMethod -Uri $ManifestUrl
    
    # Architecture Mapping
    $RawArch = $env:PROCESSOR_ARCHITECTURE
    $Arch = if ($RawArch -eq "ARM64") { "win-arm64" } else { "win-x64" }
    
    $CliUrl = $Manifest.binaries.$Arch
    if ($null -eq $CliUrl) { throw "No binary mapped for architecture: $Arch" }

    Write-Host "Downloading core binary ($Arch)..." -ForegroundColor Gray
    Invoke-WebRequest -Uri $CliUrl -OutFile $AppPath

    if (Test-Path $BinLink) { Remove-Item $BinLink -Force }
    Write-Host "Establishing binary link..." -ForegroundColor Gray
    cmd /c mklink /H "$BinLink" "$AppPath" | Out-Null

    Write-Host "CLUAIZ Core successfully initialized." -ForegroundColor Cyan
}
catch {
    Write-Host "Installation failed." -ForegroundColor Red
    Write-Host "Error: $($_.Exception.Message)" -ForegroundColor Gray
}

Write-Host "Deployment process complete." -ForegroundColor Gray
