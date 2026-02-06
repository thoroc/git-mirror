#!/usr/bin/env bash
# Installation script for git-mirror
# Usage: curl -fsSL https://raw.githubusercontent.com/thoroc/git-mirror/main/install.sh | bash

set -e

# Configuration
REPO="thoroc/git-mirror"
BINARY_NAME="git-mirror"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Detect OS
detect_os() {
    case "$(uname -s)" in
        Linux*)     echo "linux";;
        Darwin*)    echo "macos";;
        CYGWIN*|MINGW*|MSYS*) echo "windows";;
        *)          echo "unknown";;
    esac
}

# Detect architecture
detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64)   echo "x86_64";;
        aarch64|arm64)  echo "aarch64";;
        *)              echo "unknown";;
    esac
}

# Get latest release version from GitHub
get_latest_version() {
    curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/'
}

# Main installation logic
main() {
    log_info "Starting git-mirror installation..."

    # Detect system
    OS=$(detect_os)
    ARCH=$(detect_arch)

    if [ "$OS" = "unknown" ]; then
        log_error "Unsupported operating system: $(uname -s)"
        exit 1
    fi

    if [ "$ARCH" = "unknown" ]; then
        log_error "Unsupported architecture: $(uname -m)"
        exit 1
    fi

    log_info "Detected OS: $OS"
    log_info "Detected architecture: $ARCH"

    # Determine file extension and archive name
    if [ "$OS" = "windows" ]; then
        EXT="zip"
        BINARY_NAME="git-mirror.exe"
    else
        EXT="tar.gz"
    fi

    ARCHIVE_NAME="${BINARY_NAME%.*}-${OS}-${ARCH}.${EXT}"

    # Get latest version
    log_info "Fetching latest release version..."
    VERSION=$(get_latest_version)

    if [ -z "$VERSION" ]; then
        log_error "Failed to fetch latest version. Please check your internet connection or try again later."
        exit 1
    fi

    log_info "Latest version: $VERSION"

    # Download URL
    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${VERSION}/${ARCHIVE_NAME}"
    CHECKSUM_URL="${DOWNLOAD_URL}.sha256"

    log_info "Downloading $ARCHIVE_NAME..."

    # Create temporary directory
    TMP_DIR=$(mktemp -d)
    trap 'rm -rf "$TMP_DIR"' EXIT

    # Download archive
    if ! curl -fsSL -o "${TMP_DIR}/${ARCHIVE_NAME}" "$DOWNLOAD_URL"; then
        log_error "Failed to download $ARCHIVE_NAME from $DOWNLOAD_URL"
        exit 1
    fi

    # Download and verify checksum
    log_info "Verifying checksum..."
    if curl -fsSL -o "${TMP_DIR}/${ARCHIVE_NAME}.sha256" "$CHECKSUM_URL"; then
        cd "$TMP_DIR"
        if command -v sha256sum >/dev/null 2>&1; then
            sha256sum -c "${ARCHIVE_NAME}.sha256" || {
                log_error "Checksum verification failed!"
                exit 1
            }
        elif command -v shasum >/dev/null 2>&1; then
            shasum -a 256 -c "${ARCHIVE_NAME}.sha256" || {
                log_error "Checksum verification failed!"
                exit 1
            }
        else
            log_warn "sha256sum/shasum not found, skipping checksum verification"
        fi
        cd - >/dev/null
    else
        log_warn "Could not download checksum file, skipping verification"
    fi

    # Extract archive
    log_info "Extracting archive..."
    cd "$TMP_DIR"
    if [ "$EXT" = "zip" ]; then
        if command -v unzip >/dev/null 2>&1; then
            unzip -q "$ARCHIVE_NAME"
        else
            log_error "unzip command not found. Please install unzip and try again."
            exit 1
        fi
    else
        tar -xzf "$ARCHIVE_NAME"
    fi
    cd - >/dev/null

    # Create install directory if it doesn't exist
    mkdir -p "$INSTALL_DIR"

    # Install binary
    log_info "Installing to $INSTALL_DIR/$BINARY_NAME..."
    mv "${TMP_DIR}/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"
    chmod +x "${INSTALL_DIR}/${BINARY_NAME}"

    log_info "✓ Installation complete!"
    log_info ""
    log_info "git-mirror has been installed to: ${INSTALL_DIR}/${BINARY_NAME}"
    log_info ""

    # Check if install directory is in PATH
    if [[ ":$PATH:" != *":${INSTALL_DIR}:"* ]]; then
        log_warn "Warning: ${INSTALL_DIR} is not in your PATH"
        log_info "Add it to your PATH by adding this line to your shell profile:"
        log_info "  export PATH=\"\$PATH:${INSTALL_DIR}\""
        log_info ""
        case "$SHELL" in
            */zsh)  log_info "For zsh, add to: ~/.zshrc";;
            */bash) log_info "For bash, add to: ~/.bashrc or ~/.bash_profile";;
            */fish) log_info "For fish, run: fish_add_path ${INSTALL_DIR}";;
        esac
    else
        log_info "Run 'git-mirror --help' to get started!"
    fi
}

main "$@"
