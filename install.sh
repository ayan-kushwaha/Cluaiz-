#!/bin/bash
# Cluaiz-OS: Sovereign Hub Installer (Unix/macOS)
# 🏛️ Architecture: Sovereign Kernel Partitioning

set -e
HUB_PATH="$HOME/.cluaiz"
REPO="cluaiz/cluaiz"
VERSION="${1:-latest}"

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

if [ "$VERSION" == "latest" ]; then
    echo -e "\033[0;33m📡 Fetching Latest Sovereign Release...\033[0m"
    RELEASE_DATA=$(curl -s "https://api.github.com/repos/$REPO/releases")
    TARGET_TAG=$(echo "$RELEASE_DATA" | grep -oE '"tag_name": "cli-v[^"]+"' | head -1 | cut -d'"' -f4)
else
    # Support both v0.1.0 and cli-v0.1.0
    [[ "$VERSION" != "cli-"* ]] && VERSION="cli-$VERSION"
    echo -e "\033[0;33m📡 Fetching Specific Sovereign Release: $VERSION...\033[0m"
    RELEASE_DATA=$(curl -s "https://api.github.com/repos/$REPO/releases/tags/$VERSION")
    TARGET_TAG=$(echo "$RELEASE_DATA" | grep -oE '"tag_name": "[^"]+"' | head -1 | cut -d'"' -f4)
fi

if [ -z "$TARGET_TAG" ]; then
    echo -e "\033[0;31m❌ Targeted release not found.\033[0m"
    exit 1
fi

echo -e "\033[0;32m✨ Targeted Release: $TARGET_TAG\033[0m"

# 🔍 Find manifest asset URL from release data
if [ "$VERSION" == "latest" ]; then
    # For latest, we already have the full list in RELEASE_DATA, need to find the specific release's asset
    MANIFEST_URL=$(echo "$RELEASE_DATA" | grep -A 20 "\"tag_name\": \"$TARGET_TAG\"" | grep -oE '"browser_download_url": "[^"]+cli-manifest.json"' | head -1 | cut -d'"' -f4)
else
    MANIFEST_URL=$(echo "$RELEASE_DATA" | grep -oE '"browser_download_url": "[^"]+cli-manifest.json"' | head -1 | cut -d'"' -f4)
fi

if [ -z "$MANIFEST_URL" ]; then
    echo -e "\033[0;31m❌ cli-manifest.json not found for $TARGET_TAG.\033[0m"
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
