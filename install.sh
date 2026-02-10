#!/bin/bash

set -e

# Gazette CLI Installer
# Usage: curl -fsSL https://raw.githubusercontent.com/your-username/gazette/main/install.sh | bash

REPO="your-username/gazette"
BINARY_NAME="gazette"
INSTALL_DIR="/usr/local/bin"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

print_banner() {
    echo -e "${GREEN}"
    echo "     ▗▄▄▖ ▗▄▖ ▗▄▄▄▄▖▗▄▄▄▖▗▄▄▄▖▗▄▄▄▖▗▄▄▄▖"
    echo "    ▐▌   ▐▌ ▐▌   ▗▞▘▐▌     █    █  ▐▌   "
    echo "    ▐▌▝▜▌▐▛▀▜▌ ▗▞▘  ▐▛▀▀▘  █    █  ▐▛▀▀▘"
    echo "    ▝▚▄▞▘▐▌ ▐▌▐▙▄▄▄▖▐▙▄▄▖  █    █  ▐▙▄▄▖"
    echo -e "${NC}"
    echo -e "${CYAN}--- Gazette CLI Installer ---${NC}"
    echo ""
}

detect_os() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "$OS" in
        Linux*)
            OS="linux"
            ;;
        Darwin*)
            OS="darwin"
            ;;
        *)
            echo -e "${RED}Error: Unsupported operating system: $OS${NC}"
            exit 1
            ;;
    esac

    case "$ARCH" in
        x86_64|amd64)
            ARCH="x86_64"
            ;;
        arm64|aarch64)
            ARCH="aarch64"
            ;;
        *)
            echo -e "${RED}Error: Unsupported architecture: $ARCH${NC}"
            exit 1
            ;;
    esac

    echo -e "${CYAN}Detected: ${OS}-${ARCH}${NC}"
}

get_latest_version() {
    echo -e "${CYAN}Fetching latest version...${NC}"
    
    VERSION=$(curl -sL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    
    if [ -z "$VERSION" ]; then
        echo -e "${YELLOW}Warning: Could not fetch latest version, using 'latest'${NC}"
        VERSION="latest"
    else
        echo -e "${GREEN}Latest version: ${VERSION}${NC}"
    fi
}

download_binary() {
    local url
    local tmp_dir
    
    tmp_dir=$(mktemp -d)
    trap 'rm -rf "$tmp_dir"' EXIT

    if [ "$VERSION" = "latest" ]; then
        url="https://github.com/${REPO}/releases/latest/download/${BINARY_NAME}-${OS}-${ARCH}.tar.gz"
    else
        url="https://github.com/${REPO}/releases/download/${VERSION}/${BINARY_NAME}-${OS}-${ARCH}.tar.gz"
    fi

    echo -e "${CYAN}Downloading from: ${url}${NC}"
    
    if ! curl -fsSL "$url" -o "$tmp_dir/${BINARY_NAME}.tar.gz"; then
        echo -e "${RED}Error: Failed to download binary${NC}"
        echo -e "${YELLOW}The release might not exist yet. Try building from source:${NC}"
        echo ""
        echo "  git clone https://github.com/${REPO}.git"
        echo "  cd gazette"
        echo "  cargo build --release"
        echo "  sudo cp target/release/gazette /usr/local/bin/"
        echo ""
        exit 1
    fi

    echo -e "${CYAN}Extracting...${NC}"
    tar -xzf "$tmp_dir/${BINARY_NAME}.tar.gz" -C "$tmp_dir"

    echo -e "${CYAN}Installing to ${INSTALL_DIR}...${NC}"
    
    if [ -w "$INSTALL_DIR" ]; then
        mv "$tmp_dir/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"
    else
        echo -e "${YELLOW}Requesting sudo permissions to install to ${INSTALL_DIR}${NC}"
        sudo mv "$tmp_dir/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"
    fi

    chmod +x "${INSTALL_DIR}/${BINARY_NAME}"
}

verify_installation() {
    if command -v "$BINARY_NAME" &> /dev/null; then
        echo ""
        echo -e "${GREEN}✔ Gazette installed successfully!${NC}"
        echo ""
        echo -e "Run ${CYAN}gazette${NC} to get started."
        echo ""
    else
        echo -e "${YELLOW}Warning: Installation completed but 'gazette' not found in PATH${NC}"
        echo -e "You may need to add ${INSTALL_DIR} to your PATH:"
        echo ""
        echo "  export PATH=\"\$PATH:${INSTALL_DIR}\""
        echo ""
    fi
}

build_from_source() {
    echo -e "${CYAN}Building from source...${NC}"
    
    if ! command -v cargo &> /dev/null; then
        echo -e "${RED}Error: Rust/Cargo not found${NC}"
        echo -e "Install Rust from: ${CYAN}https://rustup.rs/${NC}"
        exit 1
    fi

    tmp_dir=$(mktemp -d)
    trap 'rm -rf "$tmp_dir"' EXIT

    git clone "https://github.com/${REPO}.git" "$tmp_dir/gazette"
    cd "$tmp_dir/gazette"
    
    cargo build --release
    
    if [ -w "$INSTALL_DIR" ]; then
        cp target/release/gazette "${INSTALL_DIR}/"
    else
        sudo cp target/release/gazette "${INSTALL_DIR}/"
    fi
    
    chmod +x "${INSTALL_DIR}/${BINARY_NAME}"
}

main() {
    print_banner
    detect_os
    get_latest_version
    
    echo ""
    echo -e "${CYAN}Installing Gazette CLI...${NC}"
    echo ""

    # Try to download pre-built binary, fall back to source
    if ! download_binary 2>/dev/null; then
        echo -e "${YELLOW}Pre-built binary not available, building from source...${NC}"
        build_from_source
    fi

    verify_installation
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --from-source)
            FROM_SOURCE=true
            shift
            ;;
        --install-dir)
            INSTALL_DIR="$2"
            shift 2
            ;;
        --version)
            VERSION="$2"
            shift 2
            ;;
        -h|--help)
            echo "Gazette CLI Installer"
            echo ""
            echo "Usage: install.sh [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --from-source     Build from source instead of downloading binary"
            echo "  --install-dir     Installation directory (default: /usr/local/bin)"
            echo "  --version         Specific version to install (default: latest)"
            echo "  -h, --help        Show this help message"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            exit 1
            ;;
    esac
done

if [ "$FROM_SOURCE" = true ]; then
    print_banner
    build_from_source
    verify_installation
else
    main
fi
