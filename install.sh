#!/bin/bash
# Cluaiz-OS: Sovereign Hub Installer (Unix/macOS)
# 🏛️ Architecture: Sovereign Kernel Partitioning

set -e
HUB_PATH="$HOME/.cluaiz"
REPO="cluaiz/cluaiz"

echo -e "\033[0;36m🏛️ CLUAIZ-OS: SOVEREIGN NEURAL KERNEL INSTALLER\033[0m"
echo -e "\033[0;90m--------------------------------------------------\033[0m"

# 1. 📂 Create Sovereign Entry Points
mkdir -p "$HUB_PATH/bin" "$HUB_PATH/apps/cli"
echo -e "\033[0;90m✅ Created Sovereign Hub at: $HUB_PATH\033[0m"

# 2. 🛰️ Set Environment Variables
if [[ ":$PATH:" != *":$HUB_PATH/bin:"* ]]; then
    echo -e "\033[0;33m🔗 Adding Hub to System PATH...\033[0m"
    SHELL_RC="$HOME/.bashrc"
    [[ "$SHELL" == *"zsh"* ]] && SHELL_RC="$HOME/.zshrc"
    
    echo "export CLUAIZ_ROOT=\"$HUB_PATH\"" >> "$SHELL_RC"
    echo "export PATH=\"\$PATH:$HUB_PATH/bin\"" >> "$SHELL_RC"
    export CLUAIZ_ROOT="$HUB_PATH"
    export PATH="$PATH:$HUB_PATH/bin"
fi

# 3. 📥 Download CLI (The Sovereign Orchestrator)
APP_PATH="$HUB_PATH/apps/cli/cluaiz"
BIN_LINK="$HUB_PATH/bin/cluaiz"

echo -e "\033[0;33m📡 Fetching Latest Sovereign Release...\033[0m"

# 🔍 Find latest release tag via GitHub API
RELEASE_JSON=$(curl -s "https://api.github.com/repos/$REPO/releases")
LATEST_TAG=$(echo "$RELEASE_JSON" | grep -oE '"tag_name": "cli-v[^"]+"' | head -1 | cut -d'"' -f4)

if [ -z "$LATEST_TAG" ]; then
    echo -e "\033[0;31m❌ No CLI releases found starting with 'cli-v*'.\033[0m"
    exit 1
fi

echo -e "\033[0;32m✨ Detected Release: $LATEST_TAG\033[0m"

# 🔍 Find manifest asset URL
MANIFEST_URL=$(echo "$RELEASE_JSON" | grep -oE '"browser_download_url": "[^"]+cli-manifest.json"' | head -1 | cut -d'"' -f4)

if [ -z "$MANIFEST_URL" ]; then
    echo -e "\033[0;31m❌ cli-manifest.json not found in release $LATEST_TAG.\033[0m"
    exit 1
fi

echo -e "\033[0;33m📡 Fetching Manifest...\033[0m"
MANIFEST=$(curl -sL "$MANIFEST_URL")

# Determine Arch
OS_TYPE=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH_TYPE=$(uname -m)

case "$OS_TYPE" in
    linux) OS="linux" ;;
    darwin) OS="mac" ;;
    *) echo "Unsupported OS: $OS_TYPE"; exit 1 ;;
esac

case "$ARCH_TYPE" in
    x86_64) ARCH="x64" ;;
    aarch64|arm64) ARCH="arm64" ;;
    *) echo "Unsupported Arch: $ARCH_TYPE"; exit 1 ;;
esac

ARCH_KEY="$OS-$ARCH"
CLI_URL=$(echo "$MANIFEST" | grep -oE "\"$ARCH_KEY\": \"[^\"]+\"" | cut -d'"' -f4)

if [ -z "$CLI_URL" ]; then
    echo -e "\033[0;31m❌ Could not find binary for $ARCH_KEY in manifest.\033[0m"
    exit 1
fi

echo -e "\033[0;33m📥 Downloading Cluaiz CLI ($ARCH_KEY)...\033[0m"
curl -sL "$CLI_URL" -o "$APP_PATH"
chmod +x "$APP_PATH"

# 🔗 Create Link
ln -sf "$APP_PATH" "$BIN_LINK"

echo -e "\n\033[0;36m✅ Cluaiz-OS Sovereign Hub Initialized Successfully!\033[0m"
echo -e "\033[0;32m🚀 Restart your terminal or run 'source $SHELL_RC' to ignite.\033[0m"
