#!/bin/bash
# CLUAIZ Core Infrastructure Installer (Unix/macOS)
# Standard Deployment Script - Industrial Grade

set -e

HUB_PATH="$HOME/.cluaiz"
REPO="cluaiz/cluaiz"
VERSION="${1:-latest}"

# --- UI & Personality Matrix ---
BOLD='\033[1m'
CYAN='\033[0;36m'
GRAY='\033[0;90m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m'

TAGLINES=(
    "Neural logic initialized. Preparing for ignition."
    "Bypassing hardware boundaries... Extraction in progress."
    "Establishing the Sovereign Hub. Secure partition active."
    "Claws out, logic in. Let's build something impossible."
    "Your terminal just grew a brain. Minimal fuss, maximal pinch."
    "Silicon Mastery achieved. Extracting every bit of performance."
    "Neural Core sync in progress. Stay calibrated."
)

write_step() { echo -e "  ${GRAY}[*] $1${NC}"; }
write_success() { echo -e "  ${GREEN}[OK] $1${NC}"; }
write_error() { echo -e "  ${RED}[ERROR] $1${NC}"; }

# --- Banner ---
echo -e "\n  ${CYAN}${BOLD}CLUAIZ Core Installer${NC}"
SELECTED_TAGLINE=${TAGLINES[$RANDOM % ${#TAGLINES[@]}]}
echo -e "  ${GRAY}${SELECTED_TAGLINE}${NC}\n"

# 1. Filesystem Provisioning
mkdir -p "$HUB_PATH/bin" "$HUB_PATH/apps/cli"
write_success "Workspace initialized at: $HUB_PATH"

# 2. Environment Configuration
if [[ ":$PATH:" != *":$HUB_PATH/bin:"* ]]; then
    write_step "Registering binary path..."
    SHELL_RC="$HOME/.bashrc"
    [[ "$SHELL" == *"zsh"* ]] && SHELL_RC="$HOME/.zshrc"
    
    echo "export CLUAIZ_ROOT=\"$HUB_PATH\"" >> "$SHELL_RC"
    echo "export PATH=\"\$PATH:$HUB_PATH/bin\"" >> "$SHELL_RC"
    export CLUAIZ_ROOT="$HUB_PATH"
    export PATH="$PATH:$HUB_PATH/bin"
    write_success "Shell profile updated."
fi

# 3. Binary Retrieval
APP_PATH="$HUB_PATH/apps/cli/cluaiz"
BIN_LINK="$HUB_PATH/bin/cluaiz"

if [ "$VERSION" == "latest" ]; then
    write_step "Fetching latest release manifest..."
    RELEASE_DATA=$(curl -s "https://api.github.com/repos/$REPO/releases")
    TARGET_TAG=$(echo "$RELEASE_DATA" | grep -oE '"tag_name": "cli-v[^"]+"' | head -1 | cut -d'"' -f4)
else
    [[ "$VERSION" != "cli-"* ]] && VERSION="cli-$VERSION"
    write_step "Fetching target release: $VERSION..."
    RELEASE_DATA=$(curl -s "https://api.github.com/repos/$REPO/releases/tags/$VERSION")
    TARGET_TAG=$(echo "$RELEASE_DATA" | grep -oE '"tag_name": "[^"]+"' | head -1 | cut -d'"' -f4)
fi

if [ -z "$TARGET_TAG" ]; then
    write_error "Specified release not found."
    exit 1
fi

write_success "Active Channel: $TARGET_TAG"

# Find manifest asset URL
MANIFEST_URL=$(echo "$RELEASE_DATA" | grep -oE '"browser_download_url": "[^"]+cli-manifest.json"' | head -1 | cut -d'"' -f4)

if [ -z "$MANIFEST_URL" ]; then
    write_error "Manifest asset missing."
    exit 1
fi

MANIFEST=$(curl -sL "$MANIFEST_URL")

# Architecture Mapping
OS_TYPE=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH_TYPE=$(uname -m)

case "$OS_TYPE" in
    linux) OS="linux" ;;
    darwin) OS="mac" ;;
    *) write_error "Unsupported OS: $OS_TYPE"; exit 1 ;;
esac

case "$ARCH_TYPE" in
    x86_64) ARCH="x64" ;;
    aarch64|arm64) ARCH="arm64" ;;
    *) write_error "Unsupported Arch: $ARCH_TYPE"; exit 1 ;;
esac

ARCH_KEY="$OS-$ARCH"
CLI_URL=$(echo "$MANIFEST" | grep -oE "\"$ARCH_KEY\": \"[^\"]+\"" | cut -d'"' -f4)

if [ -z "$CLI_URL" ]; then
    write_error "No binary mapped for architecture: $ARCH_KEY"
    exit 1
fi

write_step "Downloading core binary ($ARCH_KEY)..."
curl -sL "$CLI_URL" -o "$APP_PATH"
chmod +x "$APP_PATH"

# Establish Link
ln -sf "$APP_PATH" "$BIN_LINK"

echo -e "\n  ${CYAN}${BOLD}[DONE] CLUAIZ Core successfully initialized.${NC}"
echo -e "  ${GRAY}Deployment sequence complete.${NC}\n"
