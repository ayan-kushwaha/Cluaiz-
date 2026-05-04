# 🏛️ Cluaiz Sovereign Genesis: Windows Installer
# Role: Establishes the Sovereign Hub, Sets Environment Variables, and Provisions the CLI.

$ErrorActionPreference = "Stop"

# 1. 📍 Configuration: The Sovereign Hub
$DefaultHub = "$HOME\.cluaiz"
$HubPath = Read-Host -Prompt "Enter Cluaiz Hub Directory [Default: $DefaultHub]"
if ([string]::IsNullOrWhiteSpace($HubPath)) { $HubPath = $DefaultHub }

# Expand environment variables if any
$HubPath = [System.Environment]::ExpandEnvironmentVariables($HubPath)

Write-Host "`n🚀 Establishing Sovereign Hub at: $HubPath" -ForegroundColor Cyan

# 2. 📂 Create Structure
$Folders = @("bin", "engine", "workspace", "models")
foreach ($f in $Folders) {
    $path = Join-Path $HubPath $f
    if (-not (Test-Path $path)) {
        New-Item -ItemType Directory -Path $path | Out-Null
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

# 4. 📥 Download CLI (The Orchestrator)
$Version = "v0.1.0"
$Arch = if ([System.Environment]::Is64BitOperatingSystem) { "x64" } else { "x86" }
$CliUrl = "https://github.com/cluaiz/cluaiz/releases/download/$Version/cluaiz-win-$Arch.exe"
$CliPath = Join-Path $BinPath "cluaiz.exe"

Write-Host "📥 Fetching Cluaiz CLI ($Version)..." -ForegroundColor Cyan
Invoke-WebRequest -Uri $CliUrl -OutFile $CliPath

Write-Host "`n🎉 Sovereign Hub Established!" -ForegroundColor Green
Write-Host "Type 'cluaiz' in a new terminal to ignite the Neural Engine.`n" -ForegroundColor White
