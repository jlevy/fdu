#!/bin/bash
# Optional, checksum-verified GitHub CLI bootstrap for agent sessions.
#
# Session startup must never install software or alter network routing by default. Set
# FDU_BOOTSTRAP_GH_CLI=1 explicitly when a disposable environment needs this bootstrap.

set -euo pipefail

if [ "${FDU_BOOTSTRAP_GH_CLI:-}" != "1" ]; then
    echo "[gh] Optional bootstrap skipped (set FDU_BOOTSTRAP_GH_CLI=1 to enable)."
    exit 0
fi

export PATH="$HOME/.local/bin:$HOME/bin:/usr/local/bin:$PATH"

# Pinned release older than the repository's 14-day supply-chain cool-off. Checksums
# come from gh_2.92.0_checksums.txt in the corresponding GitHub release.
GH_VERSION="2.92.0"

checksum_for() {
    case "$1" in
        linux_amd64.tar.gz) echo "b57848131bdf0c229cd35e1f2a51aa718199858b2e728410b37e89a428943ec4" ;;
        linux_arm64.tar.gz) echo "c2248526dd0160c08d3fccca2332c3c1a07c15a78b23978e77735f1b5a18cfee" ;;
        macOS_amd64.zip) echo "ae9bb327ab0d91071bdada79f8f14034a2a0f19b0e001835a782eafa519d2af0" ;;
        macOS_arm64.zip) echo "b11c54f6bd7d15ed6590475079e5b2fcf36f45d3991a80041b29c9d0cc1f1d07" ;;
        *) echo "" ;;
    esac
}

if ! command -v gh >/dev/null 2>&1; then
    operating_system=$(uname -s | tr '[:upper:]' '[:lower:]')
    architecture=$(uname -m)
    [ "$architecture" = "x86_64" ] && architecture="amd64"
    [ "$architecture" = "aarch64" ] && architecture="arm64"

    if [ "$operating_system" = "darwin" ]; then
        platform="macOS_${architecture}.zip"
        archive_kind="zip"
        extracted_name="gh_${GH_VERSION}_macOS_${architecture}"
    else
        platform="${operating_system}_${architecture}.tar.gz"
        archive_kind="tar"
        extracted_name="gh_${GH_VERSION}_${operating_system}_${architecture}"
    fi

    expected_checksum=$(checksum_for "$platform")
    if [ -z "$expected_checksum" ]; then
        echo "[gh] ERROR: no pinned checksum for ${platform}; refusing to install"
        exit 1
    fi

    asset="gh_${GH_VERSION}_${platform}"
    download_url="https://github.com/cli/cli/releases/download/v${GH_VERSION}/${asset}"
    temp_dir=$(mktemp -d "${TMPDIR:-/tmp}/fdu-gh.XXXXXX")
    cleanup() {
        rm -r -- "$temp_dir"
    }
    trap cleanup EXIT

    echo "[gh] Installing opt-in pinned gh v${GH_VERSION}..."
    curl --proto '=https' --tlsv1.2 -fsSL --connect-timeout 15 \
        -o "$temp_dir/$asset" "$download_url"

    if command -v sha256sum >/dev/null 2>&1; then
        actual_checksum=$(sha256sum "$temp_dir/$asset" | awk '{print $1}')
    else
        actual_checksum=$(shasum -a 256 "$temp_dir/$asset" | awk '{print $1}')
    fi
    if [ "$actual_checksum" != "$expected_checksum" ]; then
        echo "[gh] ERROR: checksum mismatch for ${asset}"
        exit 1
    fi

    if [ "$archive_kind" = "zip" ]; then
        unzip -q "$temp_dir/$asset" -d "$temp_dir"
    else
        tar -xzf "$temp_dir/$asset" -C "$temp_dir"
    fi

    mkdir -p "$HOME/.local/bin"
    install -m 0755 "$temp_dir/$extracted_name/bin/gh" "$HOME/.local/bin/gh"
    echo "[gh] Installed checksum-verified gh at $HOME/.local/bin/gh"
fi

if gh auth status >/dev/null 2>&1; then
    echo "[gh] Authenticated successfully"
else
    echo "[gh] CLI is installed but not authenticated; run 'gh auth login' explicitly."
fi
