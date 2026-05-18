#!/usr/bin/env bash
# RepoLens Installer (macOS/Linux)
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/xinzhuang/repolens/main/scripts/install.sh | bash

set -euo pipefail

REPO="xinzhuang/repolens"
INSTALL_DIR="${REPOLENS_INSTALL_DIR:-$HOME/.local/bin}"
DRY_RUN=false

if [ -t 1 ]; then
  RED='\033[0;31m'
  GREEN='\033[0;32m'
  CYAN='\033[0;36m'
  YELLOW='\033[0;33m'
  NC='\033[0m'
else
  RED='' GREEN='' CYAN='' YELLOW='' NC=''
fi

log_info()  { echo -e "${CYAN}→${NC} $*"; }
log_ok()    { echo -e "${GREEN}✓${NC} $*"; }
log_warn()  { echo -e "${YELLOW}!${NC} $*"; }
log_err()   { echo -e "${RED}x${NC} $*" >&2; }

usage() {
  cat <<'EOF'
RepoLens Installer

Usage: install.sh [OPTIONS]

Options:
  --help            Show help
  --dir PATH        Install directory (default: ~/.local/bin)
  --dry-run         Print actions without installing
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --help|-h) usage; exit 0 ;;
    --dir)
      if [[ $# -lt 2 || "${2:-}" == -* ]]; then
        log_err "Missing value for --dir"; exit 1
      fi
      INSTALL_DIR="$2"; shift 2 ;;
    --dry-run) DRY_RUN=true; shift ;;
    *) log_err "Unknown option: $1"; exit 1 ;;
  esac
done

# ── Dependency check ──────────────────────────────────
for cmd in curl tar; do
  if ! command -v "$cmd" >/dev/null 2>&1; then
    log_err "Missing required command: $cmd"
    exit 1
  fi
done

# ── Detect platform ───────────────────────────────────
OS_RAW="$(uname -s)"
ARCH_RAW="$(uname -m)"

case "${OS_RAW}" in
  Darwin) OS="macos" ;;
  Linux)  OS="linux" ;;
  *) log_err "Unsupported OS: ${OS_RAW}"; exit 1 ;;
esac

case "${ARCH_RAW}" in
  x86_64|amd64) ARCH="x86_64" ;;
  arm64|aarch64) ARCH="aarch64" ;;
  *) log_err "Unsupported architecture: ${ARCH_RAW}"; exit 1 ;;
esac

log_ok "Detected platform: ${OS}/${ARCH}"

# ── Resolve latest release ────────────────────────────
log_info "Checking latest release..."

RELEASE_JSON="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest")"
VERSION="$(echo "$RELEASE_JSON" | grep '"tag_name"' | head -1 | sed -E 's/.*"v?([^"]+)".*/\1/')"

if [ -z "$VERSION" ]; then
  log_err "Could not determine latest version"
  exit 1
fi

log_ok "Latest version: ${VERSION}"

# ── Find matching asset ───────────────────────────────
ASSET_NAME="repolens-${ARCH}-${OS}.tar.gz"
ASSET_URL="$(echo "$RELEASE_JSON" | grep "\"browser_download_url\"" | grep "$ASSET_NAME" | head -1 | sed -E 's/.*"([^"]+)".*/\1/')"

if [ -z "$ASSET_URL" ]; then
  log_err "No binary found for ${OS}/${ARCH}"
  log_err "Check https://github.com/${REPO}/releases/latest for available assets."
  exit 1
fi

# ── Download ──────────────────────────────────────────
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

DOWNLOAD_PATH="${TMP_DIR}/${ASSET_NAME}"

if [ "${DRY_RUN}" = true ]; then
  echo "DRY RUN: curl -fsSL ${ASSET_URL} -o ${DOWNLOAD_PATH}"
  echo "DRY RUN: tar -xzf ${DOWNLOAD_PATH} -C ${TMP_DIR}"
  echo "DRY RUN: cp repolens -> ${INSTALL_DIR}/repolens"
  exit 0
fi

log_info "Downloading ${ASSET_NAME}..."
curl -fsSL "$ASSET_URL" -o "$DOWNLOAD_PATH"

# ── Verify checksum ───────────────────────────────────
CHECKSUMS_URL="$(echo "$RELEASE_JSON" | grep "\"browser_download_url\"" | grep "checksums.txt" | head -1 | sed -E 's/.*"([^"]+)".*/\1/')"

if [ -n "$CHECKSUMS_URL" ]; then
  EXPECTED_SHA="$(curl -fsSL "$CHECKSUMS_URL" | grep "$ASSET_NAME" | awk '{print $1}')"
  if [ -n "$EXPECTED_SHA" ]; then
    ACTUAL_SHA="$(sha256sum "$DOWNLOAD_PATH" 2>/dev/null | awk '{print $1}' || shasum -a 256 "$DOWNLOAD_PATH" 2>/dev/null | awk '{print $1}' || true)"
    if [ -n "$ACTUAL_SHA" ]; then
      if [ "$ACTUAL_SHA" != "$EXPECTED_SHA" ]; then
        log_err "SHA256 mismatch!"
        log_err "Expected: ${EXPECTED_SHA}"
        log_err "Actual:   ${ACTUAL_SHA}"
        exit 1
      fi
      log_ok "Integrity verified (sha256)"
    else
      log_warn "No checksum tool available; skipping verification"
    fi
  fi
fi

# ── Install ───────────────────────────────────────────
tar -xzf "$DOWNLOAD_PATH" -C "$TMP_DIR"

mkdir -p "$INSTALL_DIR"
cp "${TMP_DIR}/repolens" "${INSTALL_DIR}/repolens"
chmod +x "${INSTALL_DIR}/repolens"

# ── Ensure PATH ───────────────────────────────────────
if ! echo ":${PATH}:" | grep -q ":${INSTALL_DIR}:"; then
  SHELL_NAME="$(basename "${SHELL:-/bin/bash}")"
  case "${SHELL_NAME}" in
    zsh)  RC_FILE="${HOME}/.zshrc" ;;
    bash) RC_FILE="${HOME}/.bashrc" ;;
    *)    RC_FILE="${HOME}/.profile" ;;
  esac
  if ! grep -q 'repolens' "$RC_FILE" 2>/dev/null; then
    echo "" >> "$RC_FILE"
    echo "# Added by repolens installer" >> "$RC_FILE"
    echo "export PATH=\"${INSTALL_DIR}:\$PATH\"" >> "$RC_FILE"
    log_ok "Added ${INSTALL_DIR} to PATH in ${RC_FILE}"
    log_warn "Run 'source ${RC_FILE}' or start a new shell to update PATH"
  fi
fi

log_ok "Installed repolens v${VERSION} to ${INSTALL_DIR}/repolens"
echo ""
echo "  repolens --version"
echo "  repolens config --add-path ~/projects"
echo "  repolens scan"
