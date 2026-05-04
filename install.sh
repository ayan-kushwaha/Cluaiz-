#!/bin/bash
# CLUAIZ Core Infrastructure Installer (Unix/macOS)
# Standard Deployment Script - Industrial Grade

set -euo pipefail

HUB_PATH="${HOME}/.cluaiz"
REPO="cluaiz/cluaiz"

# --- UI Matrix ---
BOLD='\033[1m'; CYAN='\033[0;36m'; GRAY='\033[0;90m'; GREEN='\033[0;32m'; YELLOW='\033[0;33m'; RED='\033[0;31m'; NC='\033[0m'

write_step() { echo -e "  ${GRAY}[*] $1${NC}"; }
write_success() { echo -e "  ${GREEN}[OK] $1${NC}"; }
write_error() { echo -e "  ${RED}[ERR] $1${NC}"; }

# --- Header ---
clear
echo -e "\n  ${CYAN}${BOLD}CLUAIZ CORE: SOVEREIGN NEURAL KERNEL${NC}"
echo -e "  ${GRAY}Establishing silicon-to-registry handshake...${NC}\n"

# 1. Workspace Provisioning
mkdir -p "$HUB_PATH/bin" "$HUB_PATH/apps/cli" "$HUB_PATH/interface-engines/kernels" "$HUB_PATH/interface-engines/drivers"

# 2. Environment Setup
if [[ ":$PATH:" != *":$HUB_PATH/bin:"* ]]; then
    SHELL_RC="$HOME/.bashrc"
    [[ "$SHELL" == *"zsh"* ]] && SHELL_RC="$HOME/.zshrc"
    if ! grep -q "CLUAIZ_ROOT" "$SHELL_RC" 2>/dev/null; then
        echo -e "\n# CLUAIZ Environment\nexport CLUAIZ_ROOT=\"$HUB_PATH\"\nexport PATH=\"\$PATH:$HUB_PATH/bin\"" >> "$SHELL_RC"
    fi
    export CLUAIZ_ROOT="$HUB_PATH"
    export PATH="$PATH:$HUB_PATH/bin"
fi

# 3. Registry Discovery
write_step "Discovering latest neural artifacts..."
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

# --- A. CLI Download ---
CLI_URL=$(echo "$ALL_RELEASES" | grep -oE '"browser_download_url": "[^"]+cli-manifest.json"' | head -1 | cut -d'"' -f4 | xargs curl -s | grep -oE "\"$PLATFORM\": \"[^\"]+\"" | cut -d'"' -f4)
write_step "Downloading CLI ($PLATFORM)..."
curl -sL "$CLI_URL" -o "$HUB_PATH/apps/cli/cluaiz"
chmod +x "$HUB_PATH/apps/cli/cluaiz"
ln -sf "$HUB_PATH/apps/cli/cluaiz" "$HUB_PATH/bin/cluaiz"

# --- B. Engine Download ---
ENGINE_URL=$(echo "$ALL_RELEASES" | grep -oE '"browser_download_url": "[^"]+engine-manifest.json"' | head -1 | cut -d'"' -f4 | xargs curl -s | grep -oE "\"$PLATFORM\": \"[^\"]+\"" | cut -d'"' -f4)
write_step "Downloading Neural Engine..."
curl -sL "$ENGINE_URL" -o "$HUB_PATH/interface-engines/cluaiz-engine.$EXT"

# --- C. Kernel Sync (Default Llama) ---
KERNEL_URL=$(echo "$ALL_RELEASES" | grep -oE '"browser_download_url": "[^"]+kernel-manifest.json"' | head -1 | cut -d'"' -f4 | xargs curl -s | grep -oE "\"$PLATFORM-cpu\": \"[^\"]+\"" | cut -d'"' -f4)
write_step "Provisioning Neural Kernels..."
curl -sL "$KERNEL_URL" -o "$HUB_PATH/interface-engines/kernels/libarcher_llama.$EXT"

echo -e "\n  ${GREEN}${BOLD}[COMPLETE] Sovereign stack initialized.${NC}"
echo -e "  ${GRAY}Path: $HUB_PATH${NC}\n"
