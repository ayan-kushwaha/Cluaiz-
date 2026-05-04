#!/bin/bash
# CLUAIZ Core Infrastructure Installer (Unix/macOS)
# Standard Deployment Script

set -e
HUB_PATH="$HOME/.cluaiz"
REPO="cluaiz/cluaiz"
VERSION="${1:-latest}"

echo -e "\033[0;36mCLUAIZ - Core Infrastructure Installer\033[0m"
echo -e "\033[0;90m--------------------------------------------------\033[0m"

# 1. Filesystem Provisioning
mkdir -p "$HUB_PATH/bin" "$HUB_PATH/apps/cli"
echo -e "\033[0;90mWorkspace initialized at: $HUB_PATH\033[0m"

# 2. Environment Configuration
if [[ ":$PATH:" != *":$HUB_PATH/bin:"* ]]; then
    echo -e "\033[0;90mRegistering binary path...\033[0m"
    SHELL_RC="$HOME/.bashrc"
    [[ "$SHELL" == *"zsh"* ]] && SHELL_RC="$HOME/.zshrc"
    
    echo "export CLUAIZ_ROOT=\"$HUB_PATH\"" >> "$SHELL_RC"
    echo "export PATH=\"\$PATH:$HUB_PATH/bin\"" >> "$SHELL_RC"
    export CLUAIZ_ROOT="$HUB_PATH"
    export PATH="$PATH:$HUB_PATH/bin"
fi

# 3. Binary Retrieval
APP_PATH="$HUB_PATH/apps/cli/cluaiz"
BIN_LINK="$HUB_PATH/bin/cluaiz"

if [ "$VERSION" == "latest" ]; then
    echo -e "\033[0;90mFetching latest release manifest...\033[0m"
    RELEASE_DATA=$(curl -s "https://api.github.com/repos/$REPO/releases")
    TARGET_TAG=$(echo "$RELEASE_DATA" | grep -oE '"tag_name": "cli-v[^"]+"' | head -1 | cut -d'"' -f4)
else
    [[ "$VERSION" != "cli-"* ]] && VERSION="cli-$VERSION"
    echo -e "\033[0;90mFetching target release: $VERSION...\033[0m"
    RELEASE_DATA=$(curl -s "https://api.github.com/repos/$REPO/releases/tags/$VERSION")
    TARGET_TAG=$(echo "$RELEASE_DATA" | grep -oE '"tag_name": "[^"]+"' | head -1 | cut -d'"' -f4)
fi

if [ -z "$TARGET_TAG" ]; then
    echo -e "\033[0;31mError: Specified release not found.\033[0m"
    exit 1
fi

echo -e "\033[0;32mActive version: $TARGET_TAG\033[0m"

# Find manifest asset URL
if [ "$VERSION" == "latest" ]; then
    MANIFEST_URL=$(echo "$RELEASE_DATA" | grep -A 20 "\"tag_name\": \"$TARGET_TAG\"" | grep -oE '"browser_download_url": "[^"]+cli-manifest.json"' | head -1 | cut -d'"' -f4)
else
    MANIFEST_URL=$(echo "$RELEASE_DATA" | grep -oE '"browser_download_url": "[^"]+cli-manifest.json"' | head -1 | cut -d'"' -f4)
fi

if [ -z "$MANIFEST_URL" ]; then
    echo -e "\033[0;31mError: Manifest asset missing.\033[0m"
    exit 1
fi

MANIFEST=$(curl -sL "$MANIFEST_URL")

# Architecture Mapping
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
    echo -e "\033[0;31mError: No binary mapped for $ARCH_KEY\033[0m"
    exit 1
fi

echo -e "\033[0;90mDownloading core binary ($ARCH_KEY)...\033[0m"
curl -sL "$CLI_URL" -o "$APP_PATH"
chmod +x "$APP_PATH"

# Establish Link
ln -sf "$APP_PATH" "$BIN_LINK"

echo -e "\n\033[0;36mCLUAIZ Core successfully initialized.\033[0m"
echo -e "\033[0;90mDeployment process complete.\033[0m"
