#!/bin/bash
set -e

# 🏛️ Cluaiz-OS: Sovereign CLI Installer
# This script detects your OS and architecture to download the correct binary.

REPO="cluaiz/cluaiz"
VERSION="cli-v0.1.0"

OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS" in
  linux) SUFFIX="linux" ;;
  darwin) SUFFIX="mac" ;;
  *) echo "❌ Unsupported OS: $OS"; exit 1 ;;
esac

case "$ARCH" in
  x86_64) SUFFIX="${SUFFIX}-x64" ;;
  arm64|aarch64) SUFFIX="${SUFFIX}-arm64" ;;
  *) echo "❌ Unsupported Architecture: $ARCH"; exit 1 ;;
esac

URL="https://github.com/$REPO/releases/download/$VERSION/cluaiz-$SUFFIX"

echo "🛰️ Downloading Cluaiz CLI ($VERSION) for $OS-$ARCH..."
curl -L "$URL" -o cluaiz
chmod +x cluaiz

echo "✅ Installation complete."
# 2. 📂 Create Structure
mkdir -p "$HUB_PATH/bin" "$HUB_PATH/interface-engines" "$HUB_PATH/booster" "$HUB_PATH/vault" "$HUB_PATH/skills" "$HUB_PATH/logs"
echo "👉 Move 'cluaiz' to your PATH: 'sudo mv cluaiz /usr/local/bin/'"
