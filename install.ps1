# Cluaiz-OS: Sovereign Hub Installer (Windows)
# 🏛️ Architecture: Sovereign Kernel Partitioning

param (
    [string]$Version = "latest"
)

$ErrorActionPreference = "Stop"
$HubPath = Join-Path $HOME ".cluaiz"
$Repo = "cluaiz/cluaiz"

Write-Host "🏛️ CLUAIZ-OS: SOVEREIGN NEURAL KERNEL INSTALLER" -ForegroundColor Cyan
Write-Host "--------------------------------------------------" -ForegroundColor Gray

# 1. 🛡️ Check Privileges
$IsAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $IsAdmin) {
    Write-Warning "⚠️ Running without Admin privileges. Global PATH updates may require a terminal restart."
}

Write-Host "`n🚀 Establishing Sovereign Hub at: $HubPath" -ForegroundColor Cyan

# 2. 📂 Create Sovereign Entry Points
$Folders = @("bin", "apps/cli")
foreach ($f in $Folders) {
    $path = Join-Path $HubPath $f
    if (-not (Test-Path $path)) {
        New-Item -ItemType Directory -Path $path -Force | Out-Null
        Write-Host "✅ Created folder: $f" -ForegroundColor DarkGray
    }
}

# 3. 🛰️ Set Environment Variables (The Source of Truth)
Write-Host "📡 Setting CLUAIZ_ROOT environment variable..." -ForegroundColor Yellow
[System.Environment]::SetEnvironmentVariable("CLUAIZ_ROOT", $HubPath, "User")
$env:CLUAIZ_ROOT = $HubPath

# Add bin to PATH
$BinPath = Join-Path $HubPath "bin"
$OldPath = [System.Environment]::GetEnvironmentVariable("Path", "User")
if ($OldPath -notlike "*$BinPath*") {
    Write-Host "🔗 Adding Hub to System PATH..." -ForegroundColor Yellow
    [System.Environment]::SetEnvironmentVariable("Path", "$OldPath;$BinPath", "User")
    $env:Path = "$env:Path;$BinPath"
}

# 4. 📥 Download CLI (The Sovereign Orchestrator)
$AppPath = Join-Path $HubPath "apps/cli/cluaiz.exe"
$BinLink = Join-Path $HubPath "bin/cluaiz.exe"

try {
    if ($Version -eq "latest") {
        Write-Host "📡 Fetching Latest Sovereign Release..." -ForegroundColor Yellow
        $Releases = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases"
        $CliRelease = $Releases | Where-Object { $_.tag_name -like "cli-v*" } | Select-Object -First 1
    } else {
        # Ensure version has 'cli-' prefix if it's just vX.Y.Z
        $TargetTag = if ($Version -notlike "cli-*") { "cli-$Version" } else { $Version }
        Write-Host "📡 Fetching Specific Sovereign Release: $TargetTag" -ForegroundColor Yellow
        $CliRelease = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/tags/$TargetTag"
    }
    
    if ($null -eq $CliRelease) {
        throw "Could not find target CLI release."
    }

    $Tag = $CliRelease.tag_name
    Write-Host "✨ Targeted Release: $Tag" -ForegroundColor Green

    # 🔍 Find the manifest asset
    $ManifestAsset = $CliRelease.assets | Where-Object { $_.name -eq "cli-manifest.json" }
    if ($null -eq $ManifestAsset) {
        throw "cli-manifest.json not found in release $Tag."
    }

    $ManifestUrl = $ManifestAsset.browser_download_url
    Write-Host "📡 Downloading Manifest from Release Assets..." -ForegroundColor Yellow
    $Manifest = Invoke-RestMethod -Uri $ManifestUrl
    
    # Determine Arch
    $Arch = if ([System.Environment]::Is64BitOperatingSystem) { "win-x64" } else { "win-arm64" }
    $CliUrl = $Manifest.binaries.$Arch

    if ($null -eq $CliUrl) {
        throw "Could not find binary for $Arch in manifest."
    }

    Write-Host "📥 Downloading Cluaiz CLI ($Arch)..." -ForegroundColor Yellow
    Invoke-WebRequest -Uri $CliUrl -OutFile $AppPath

    # 🔗 Create Hard Link for Global Access
    if (Test-Path $BinLink) { Remove-Item $BinLink -Force }
    Write-Host "🔗 Establishing Global Hard Link..." -ForegroundColor Green
    cmd /c mklink /H "$BinLink" "$AppPath" | Out-Null

    Write-Host "`n✅ Cluaiz-OS Sovereign Hub Initialized Successfully!" -ForegroundColor Cyan
    Write-Host "🚀 Restart your terminal and type 'cluaiz' to ignite." -ForegroundColor Green
}
catch {
    Write-Host "`n⚠️ Sovereign Hub initialization encountered a pulse error." -ForegroundColor Yellow
    Write-Host "Error: $($_.Exception.Message)" -ForegroundColor DarkGray
    Write-Host "You can build the CLI locally using: 'cargo build --release -p cli'" -ForegroundColor White
}

Write-Host "`n🎉 Installation sequence complete." -ForegroundColor Cyan
