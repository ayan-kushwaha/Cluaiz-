#!/bin/bash
# CLUAIZ Core Infrastructure Installer (Unix/macOS)
# Industrial Standard Deployment Script

set -euo pipefail

HUB_PATH="${HOME}/.cluaiz"
REPO="cluaiz/cluaiz"

# --- UI Matrix (Minimalist Industrial) ---
BOLD='\033[1m'; CYAN='\033[0;36m'; GRAY='\033[0;90m'; GREEN='\033[0;32m'; YELLOW='\033[0;33m'; RED='\033[0;31m'; NC='\033[0m'

write_step() { echo -e "  ${GRAY}[*] $1${NC}"; }
write_success() { echo -e "  ${GREEN}[OK] $1${NC}"; }
write_error() { echo -e "  ${RED}[ERR] $1${NC}"; }

# --- Header ---
clear
echo -e "\n  ${BOLD}CLUAIZ CORE INFRASTRUCTURE${NC}"
echo -e "  ${GRAY}Standard Deployment Sequence${NC}\n"

# 1. Environment Provisioning
write_step "Provisioning environment..."
mkdir -p "$HUB_PATH/bin" "$HUB_PATH/apps/cli" "$HUB_PATH/interface-engines/kernels" "$HUB_PATH/interface-engines/drivers"

# 2. System Integration
if [[ ":$PATH:" != *":$HUB_PATH/bin:"* ]]; then
    SHELL_RC="$HOME/.bashrc"
    [[ "$SHELL" == *"zsh"* ]] && SHELL_RC="$HOME/.zshrc"
    if ! grep -q "CLUAIZ_ROOT" "$SHELL_RC" 2>/dev/null; then
        echo -e "\n# CLUAIZ Environment\nexport CLUAIZ_ROOT=\"$HUB_PATH\"\nexport PATH=\"\$PATH:$HUB_PATH/bin\"" >> "$SHELL_RC"
    fi
    export CLUAIZ_ROOT="$HUB_PATH"
    export PATH="$PATH:$HUB_PATH/bin"
fi

# 3. Artifact Retrieval
write_step "Resolving artifacts from registry..."
ALL_RELEASES=$(curl -s "https://api.github.com/repos/$REPO/releases")

OS_TYPE=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH_TYPE=$(uname -m)
case "$OS_TYPE" in
    linux) OS="linux"; EXT="so" ;;
    darwin) OS="mac"; EXT="dylib" ;;
    *) write_error "Unsupported OS"; exit 1 ;;
esac
case "$ARCH_TYPE" in
    x86_64) ARCH="x64" ;;
    aarch64|arm64) ARCH="arm64" ;;
    *) write_error "Unsupported Arch"; exit 1 ;;
esac
PLATFORM="$OS-$ARCH"

# --- CLI ---
CLI_MANIFEST_URL=$(echo "$ALL_RELEASES" | grep -oE '"browser_download_url": "[^"]+cli-manifest.json"' | head -1 | cut -d'"' -f4)
CLI_URL=$(curl -sL "$CLI_MANIFEST_URL" | grep -oE "\"$PLATFORM\": \"[^\"]+\"" | cut -d'"' -f4)
write_step "Retrieving Cluaiz CLI ($PLATFORM)..."
curl -sL "$CLI_URL" -o "$HUB_PATH/apps/cli/cluaiz"
chmod +x "$HUB_PATH/apps/cli/cluaiz"
ln -sf "$HUB_PATH/apps/cli/cluaiz" "$HUB_PATH/bin/cluaiz"

# --- Engine ---
ENGINE_MANIFEST_URL=$(echo "$ALL_RELEASES" | grep -oE '"browser_download_url": "[^"]+engine-manifest.json"' | head -1 | cut -d'"' -f4)
ENGINE_URL=$(curl -sL "$ENGINE_MANIFEST_URL" | grep -oE "\"$PLATFORM\": \"[^\"]+\"" | cut -d'"' -f4)
write_step "Retrieving Neural Engine..."
curl -sL "$ENGINE_URL" -o "$HUB_PATH/interface-engines/cluaiz-engine.$EXT"

# --- Default Kernel ---
KERNEL_MANIFEST_URL=$(echo "$ALL_RELEASES" | grep -oE '"browser_download_url": "[^"]+kernel-manifest.json"' | head -1 | cut -d'"' -f4)
MANIFEST_CONTENT=$(curl -sL "$KERNEL_MANIFEST_URL")

# Smart detection
if [[ "$OS" == "mac" ]]; then BACKEND="metal"; else BACKEND="cuda"; fi
KERNEL_URL=$(echo "$MANIFEST_CONTENT" | grep -oE "\"$PLATFORM-$BACKEND\": \"[^\"]+\"" | cut -d'"' -f4 || echo "")

if [ -n "$KERNEL_URL" ]; then
    write_step "Retrieving Neural Kernel ($BACKEND)..."
    curl -sL "$KERNEL_URL" -o "$HUB_PATH/interface-engines/kernels/libarcher_llama.$EXT"
fi

echo -e "\n  ${GREEN}${BOLD}[OK] Deployment successful.${NC}"
echo -e "  ${GRAY}Path: $HUB_PATH${NC}"
echo -e "  Launching Cluaiz CLI...\n"

# Launch CLI
"$HUB_PATH/bin/cluaiz"
