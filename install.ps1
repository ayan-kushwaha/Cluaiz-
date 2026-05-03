# 🏛️ Cluaiz-OS: Sovereign CLI Installer (Windows)
# This script downloads the latest Cluaiz CLI binary for Windows.

$REPO = "cluaiz/cluaiz"
$VERSION = "cli-v0.1.0"
$URL = "https://github.com/$REPO/releases/download/$VERSION/cluaiz-win-x64.exe"

Write-Host "🛰️ Downloading Cluaiz CLI ($VERSION) for Windows..." -ForegroundColor Cyan

try {
    Invoke-RestMethod -Uri $URL -OutFile cluaiz.exe
    Write-Host "✅ Installation complete." -ForegroundColor Green
    Write-Host "👉 Add the directory containing 'cluaiz.exe' to your Environment PATH." -ForegroundColor Yellow
} catch {
    Write-Host "❌ Failed to download Cluaiz CLI. Please check your internet connection or the release URL." -ForegroundColor Red
}
