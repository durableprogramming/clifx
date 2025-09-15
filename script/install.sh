#!/bin/bash

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
REPO_OWNER="durableprogramming"
REPO_NAME="clifx"
INSTALL_DIR=$(readlink -f ~/.local/bin/)
TEMP_DIR=$(mktemp -d)

# Cleanup on exit
trap 'rm -rf "$TEMP_DIR"' EXIT

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

detect_architecture() {
    local arch=$(uname -m)
    case $arch in
        x86_64)
            echo "x86_64"
            ;;
        aarch64|arm64)
            echo "aarch64"
            ;;
        *)
            log_error "Unsupported architecture: $arch"
            exit 1
            ;;
    esac
}

detect_os() {
    local os=$(uname -s | tr '[:upper:]' '[:lower:]')
    case $os in
        linux)
            echo "linux"
            ;;
        darwin)
            echo "darwin"
            ;;
        *)
            log_error "Unsupported operating system: $os"
            exit 1
            ;;
    esac
}

detect_libc() {
    if [[ "$1" == "linux" ]]; then
        if ldd --version 2>&1 | grep -q musl; then
            echo "musl"
        else
            echo "gnu"
        fi
    else
        echo ""
    fi
}

get_latest_version() {
    local api_url="https://api.github.com/repos/$REPO_OWNER/$REPO_NAME/releases/latest"
    local version=$(curl -s "$api_url" | grep '"tag_name":' | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/')
    
    if [[ -z "$version" ]]; then
        log_error "Failed to fetch latest version"
        exit 1
    fi
    
    echo "$version"
}

download_and_install() {
    local version="$1"
    local arch="$2"
    local os="$3"
    local libc="$4"
    
    # Construct the target triple
    local target=""
    if [[ "$os" == "linux" ]]; then
        target="${arch}-unknown-linux-${libc}"
    else
        target="${arch}-apple-darwin"
    fi
    
    # Construct download URL
    local filename="clifx-${version}-${target}.tar.gz"
    local download_url="https://github.com/$REPO_OWNER/$REPO_NAME/releases/download/$version/$filename"
    
    log_info "Downloading $filename..."
    
    cd "$TEMP_DIR"
    if ! curl -L -o "$filename" "$download_url"; then
        log_error "Failed to download $download_url"
        exit 1
    fi
    
    log_info "Extracting archive..."
    if ! tar -xzf "$filename"; then
        log_error "Failed to extract $filename"
        exit 1
    fi
    
    log_info "Installing clifx to $INSTALL_DIR..."
    cp clifx "$INSTALL_DIR/"
    chmod +x "$INSTALL_DIR/clifx"
    
    log_success "clifx $version installed successfully!"
}

install_from_deb() {
    local version="$1"
    local arch="$2"
    
    # Map architecture for deb package
    local deb_arch=""
    case $arch in
        x86_64)
            deb_arch="amd64"
            ;;
        aarch64)
            deb_arch="arm64"
            ;;
        *)
            log_error "Unsupported architecture for .deb package: $arch"
            exit 1
            ;;
    esac
    
    local filename="clifx-${version}-${deb_arch}.deb"
    local download_url="https://github.com/$REPO_OWNER/$REPO_NAME/releases/download/$version/$filename"
    
    log_info "Downloading $filename..."
    
    cd "$TEMP_DIR"
    if ! curl -L -o "$filename" "$download_url"; then
        log_error "Failed to download $download_url"
        exit 1
    fi
    
    log_info "Installing .deb package..."
    if ! sudo dpkg -i "$filename"; then
        log_error "Failed to install .deb package"
        log_info "Attempting to fix dependencies..."
        sudo apt-get install -f -y
    fi
    
    log_success "clifx $version installed successfully via .deb package!"
}

check_prerequisites() {
    local required_commands=("curl" "tar")
    
    for cmd in "${required_commands[@]}"; do
        if ! command -v "$cmd" &> /dev/null; then
            log_error "Required command '$cmd' not found"
            exit 1
        fi
    done
}

usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -v, --version VERSION    Install specific version (default: latest)"
    echo "  -d, --dir DIRECTORY      Install directory (default: /usr/local/bin)"
    echo "  --deb                    Use .deb package installation (Debian/Ubuntu only)"
    echo "  -h, --help               Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                       # Install latest version"
    echo "  $0 -v v0.1.0            # Install specific version"
    echo "  $0 --deb                # Install using .deb package"
    echo "  $0 -d ~/bin             # Install to custom directory"
}

main() {
    local version=""
    local use_deb=false
    
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -v|--version)
                version="$2"
                shift 2
                ;;
            -d|--dir)
                INSTALL_DIR="$2"
                shift 2
                ;;
            --deb)
                use_deb=true
                shift
                ;;
            -h|--help)
                usage
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                usage
                exit 1
                ;;
        esac
    done
    
    log_info "clifx installer"
    
    check_prerequisites
    
    # Detect system information
    local arch=$(detect_architecture)
    local os=$(detect_os)
    local libc=$(detect_libc "$os")
    
    log_info "Detected system: $os/$arch$([ -n "$libc" ] && echo "/$libc")"
    
    # Get version if not specified
    if [[ -z "$version" ]]; then
        version=$(get_latest_version)
        log_info "Latest version: $version"
    fi
    
    # Install based on method
    if [[ "$use_deb" == true ]]; then
        if [[ "$os" != "linux" ]]; then
            log_error ".deb installation is only available for Linux"
            exit 1
        fi
        
        if ! command -v dpkg &> /dev/null; then
            log_error "dpkg not found. .deb installation requires a Debian-based system"
            exit 1
        fi
        
        install_from_deb "$version" "$arch"
    else
        download_and_install "$version" "$arch" "$os" "$libc"
    fi
    
    # Verify installation
    if command -v clifx &> /dev/null; then
        log_success "Installation complete! Run 'clifx --help' to get started."
        log_info "Installed version: $(clifx --version 2>/dev/null || echo 'unknown')"
    else
        log_warning "clifx was installed but not found in PATH"
        log_info "You may need to add $INSTALL_DIR to your PATH"
    fi
}

main "$@"
