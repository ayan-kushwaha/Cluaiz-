#!/bin/bash
# 🏛️ Cluaiz Sovereign Genesis: Unix Installer (Linux/macOS)
# Role: Establishes the Sovereign Hub, Sets Environment Variables, and Provisions the CLI.

set -e

# 1. 📍 Configuration
DEFAULT_HUB="$HOME/.cluaiz"
read -p "Enter Cluaiz Hub Directory [Default: $DEFAULT_HUB]: " HUB_PATH
HUB_PATH=${HUB_PATH:-$DEFAULT_HUB}

echo -e "\n🚀 Establishing Sovereign Hub at: $HUB_PATH"

# 2. 📂 Create Structure
mkdir -p "$HUB_PATH/bin" "$HUB_PATH/engine" "$HUB_PATH/workspace" "$HUB_PATH/models"

# 3. 🛰️ Set Environment Variables
echo "📡 Updating shell profile..."
SHELL_PROFILE=""
case $SHELL in
    */zsh) SHELL_PROFILE="$HOME/.zshrc" ;;
    */bash) SHELL_PROFILE="$HOME/.bashrc" ;;
    *) SHELL_PROFILE="$HOME/.profile" ;;
esac

# Function to add to profile if not present
add_to_profile() {
    if ! grep -q "$1" "$SHELL_PROFILE"; then
        echo "$1" >> "$SHELL_PROFILE"
    fi
}

add_to_profile "export CLUAIZ_ROOT=\"$HUB_PATH\""
add_to_profile "export PATH=\"\$PATH:\$CLUAIZ_ROOT/bin\""

export CLUAIZ_ROOT="$HUB_PATH"
export PATH="$PATH:$HUB_PATH/bin"

# 4. 📥 Download CLI
VERSION="v0.1.0"
OS_TYPE=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH_TYPE=$(uname -m)

if [[ "$ARCH_TYPE" == "x86_64" ]]; then
    ARCH="x64"
elif [[ "$ARCH_TYPE" == "arm64" || "$ARCH_TYPE" == "aarch64" ]]; then
    ARCH="arm64"
else
    echo "❌ Unsupported Architecture: $ARCH_TYPE"
    exit 1
fi

CLI_URL="https://github.com/cluaiz/cluaiz/releases/download/$VERSION/cluaiz-$OS_TYPE-$ARCH"
CLI_PATH="$HUB_PATH/bin/cluaiz"

echo "📥 Fetching Cluaiz CLI ($VERSION) for $OS_TYPE-$ARCH..."
curl -L "$CLI_URL" -o "$CLI_PATH"
chmod +x "$CLI_PATH"

echo -e "\n🎉 Sovereign Hub Established!"
echo -e "Restart your terminal or run 'source $SHELL_PROFILE' to ignite the Neural Engine.\n"
