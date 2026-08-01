#!/usr/bin/env bash
# Yubi Installer Script
# Detects OS and Architecture, installs the `yubi` TUI binary, and sets up PATH.

set -e

# Styling colors
BOLD="\033[1m"
GREEN="\033[32m"
BLUE="\033[34m"
YELLOW="\033[33m"
RED="\033[31m"
RESET="\033[0m"

log_info() {
    printf "${BLUE}${BOLD}[INFO]${RESET} %s\n" "$1"
}

log_success() {
    printf "${GREEN}${BOLD}[SUCCESS]${RESET} %s\n" "$1"
}

log_warn() {
    printf "${YELLOW}${BOLD}[WARNING]${RESET} %s\n" "$1"
}

log_error() {
    printf "${RED}${BOLD}[ERROR]${RESET} %s\n" "$1"
}

printf "${BOLD}=====================================================${RESET}\n"
printf "${BOLD}          Yubi TUI Typing Trainer Installer          ${RESET}\n"
printf "${BOLD}=====================================================${RESET}\n\n"

# 1. Detect Operating System
OS="$(uname -s)"
case "${OS}" in
    Linux*)     TARGET_OS="linux";;
    Darwin*)    TARGET_OS="macos";;
    MINGW*|MSYS*|CYGWIN*) TARGET_OS="windows";;
    *)          log_error "Unsupported operating system: ${OS}"; exit 1;;
esac

# 2. Detect Architecture
ARCH="$(uname -m)"
case "${ARCH}" in
    x86_64|amd64)  TARGET_ARCH="x86_64";;
    aarch64|arm64) TARGET_ARCH="aarch64";;
    i386|i686)     TARGET_ARCH="i686";;
    *)             log_error "Unsupported architecture: ${ARCH}"; exit 1;;
esac

log_info "Detected OS: ${BOLD}${TARGET_OS}${RESET} (${OS})"
log_info "Detected Architecture: ${BOLD}${TARGET_ARCH}${RESET} (${ARCH})"

# 3. Determine Installation Directory
INSTALL_DIR="${HOME}/.local/bin"
mkdir -p "${INSTALL_DIR}"

BINARY_NAME="yubi"
if [ "${TARGET_OS}" = "windows" ]; then
    BINARY_NAME="yubi.exe"
fi

DEST_PATH="${INSTALL_DIR}/${BINARY_NAME}"

# 4. Download or Build Binary
REPO_OWNER="temesgensida-code"
REPO_NAME="yubi"
RELEASE_URL="https://github.com/${REPO_OWNER}/${REPO_NAME}/releases/latest/download/yubi-${TARGET_OS}-${TARGET_ARCH}"

DOWNLOAD_SUCCESS=false

# Try downloading pre-built binary
if command -v curl >/dev/null 2>&1; then
    log_info "Attempting to download binary from GitHub releases..."
    if curl -fsSL "${RELEASE_URL}" -o "${DEST_PATH}" 2>/dev/null; then
        DOWNLOAD_SUCCESS=true
    fi
elif command -v wget >/dev/null 2>&1; then
    log_info "Attempting to download binary using wget..."
    if wget -qO "${DEST_PATH}" "${RELEASE_URL}" 2>/dev/null; then
        DOWNLOAD_SUCCESS=true
    fi
fi

# Fallback: Local compile via cargo if downloading fails or local source is present
if [ "${DOWNLOAD_SUCCESS}" = false ]; then
    log_warn "Pre-built release asset not found online or download failed."
    
    # Check if we are inside the source directory or cargo is available
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}" 2>/dev/null || echo ".")" && pwd)"
    if [ -f "${SCRIPT_DIR}/Cargo.toml" ] && command -v cargo >/dev/null 2>&1; then
        log_info "Building Yubi TUI locally using Rust (cargo)..."
        (cd "${SCRIPT_DIR}" && cargo build --release)
        cp "${SCRIPT_DIR}/target/release/${BINARY_NAME}" "${DEST_PATH}"
        DOWNLOAD_SUCCESS=true
    elif command -v cargo >/dev/null 2>&1; then
        log_info "Cargo detected! Fetching source tarball and building latest release..."
        TMP_DIR="$(mktemp -d 2>/dev/null || echo "/tmp/yubi_build")"
        mkdir -p "${TMP_DIR}"
        
        TARBALL_URL="https://github.com/${REPO_OWNER}/${REPO_NAME}/archive/refs/heads/main.tar.gz"
        BUILD_SUCCESS=false
        
        if curl -fsSL "${TARBALL_URL}" | tar -xz -C "${TMP_DIR}" 2>/dev/null; then
            SOURCE_DIR="$(find "${TMP_DIR}" -maxdepth 1 -type d -name "${REPO_NAME}-*" | head -n 1)"
            if [ -n "${SOURCE_DIR}" ] && [ -f "${SOURCE_DIR}/Cargo.toml" ]; then
                log_info "Compiling Yubi binary in temporary workspace..."
                (cd "${SOURCE_DIR}" && cargo build --release)
                cp "${SOURCE_DIR}/target/release/${BINARY_NAME}" "${DEST_PATH}"
                BUILD_SUCCESS=true
                DOWNLOAD_SUCCESS=true
            fi
        fi
        
        # Cleanup temp directory
        rm -rf "${TMP_DIR}"
        
        if [ "${BUILD_SUCCESS}" = false ]; then
            log_info "Fallback to cargo install --git..."
            cargo install --git "https://github.com/${REPO_OWNER}/${REPO_NAME}.git" --root "${HOME}/.local"
            DOWNLOAD_SUCCESS=true
        fi
    else
        log_error "Could not download pre-built binary and Rust (cargo) is not installed on this system."
        log_error "Please install Rust (https://rustup.rs) or build Yubi manually."
        exit 1
    fi
fi

# Ensure executable permissions
chmod +x "${DEST_PATH}"
log_success "Binary installed to: ${DEST_PATH}"

# 5. Shell PATH Configuration
SHELL_NAME="$(basename "${SHELL:-bash}")"
PATH_CONFIGURED=false

case "${PATH}" in
    *"${INSTALL_DIR}"*)
        PATH_CONFIGURED=true
        ;;
esac

if [ "${PATH_CONFIGURED}" = false ]; then
    log_info "Adding ${INSTALL_DIR} to your shell PATH..."
    
    EXPORT_LINE="export PATH=\"${INSTALL_DIR}:\$PATH\""
    
    if [ "${SHELL_NAME}" = "zsh" ] && [ -f "${HOME}/.zshrc" ]; then
        echo "" >> "${HOME}/.zshrc"
        echo "# Yubi TUI" >> "${HOME}/.zshrc"
        echo "${EXPORT_LINE}" >> "${HOME}/.zshrc"
        log_success "Updated ~/.zshrc"
    elif [ "${SHELL_NAME}" = "fish" ]; then
        FISH_CONFIG="${HOME}/.config/fish/config.fish"
        mkdir -p "$(dirname "${FISH_CONFIG}")"
        echo "set -gx PATH ${INSTALL_DIR} \$PATH" >> "${FISH_CONFIG}"
        log_success "Updated ${FISH_CONFIG}"
    else
        BASH_RC="${HOME}/.bashrc"
        [ -f "${HOME}/.bash_profile" ] && BASH_RC="${HOME}/.bash_profile"
        echo "" >> "${BASH_RC}"
        echo "# Yubi TUI" >> "${BASH_RC}"
        echo "${EXPORT_LINE}" >> "${BASH_RC}"
        log_success "Updated ${BASH_RC}"
    fi
fi

printf "\n${GREEN}${BOLD}🎉 Installation Complete!${RESET}\n"
printf "You can now launch the typing trainer anytime by typing:\n"
printf "  ${BOLD}yubi${RESET}\n\n"

if [ "${PATH_CONFIGURED}" = false ]; then
    printf "${YELLOW}Note: Please restart your terminal or run:${RESET}\n"
    printf "  ${BOLD}export PATH=\"${INSTALL_DIR}:\$PATH\"${RESET}\n"
fi
