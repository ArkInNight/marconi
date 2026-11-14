#!/bin/sh
# Marconi one-liner for Linux.
#
#   curl -fsSL https://raw.githubusercontent.com/ArkInNight/marconi/main/scripts/get.sh | sh
#
# Arguments are forwarded to marconi, e.g.:
#
#   curl -fsSL .../get.sh | sh -s -- --json
#
# Downloads the latest release binary, verifies its SHA-256 against the
# release's SHA256SUMS, runs it, then removes the temporary files.
#
# Reading another process's memory needs ptrace permission. On systems with
# kernel.yama.ptrace_scope=1 (Arch/Fedora/Ubuntu default) this script asks
# for sudo automatically; set MARCONI_NO_SUDO=1 to skip that and get the
# manual hint instead.
#
# marconi only reads the game's memory locally. Nothing is uploaded.

set -eu

REPO="ArkInNight/marconi"
BASE="https://github.com/$REPO/releases/latest/download"

die() {
    echo "marconi: $*" >&2
    exit 1
}

case "$(uname -s)" in
    Linux) ;;
    *) die "this one-liner only supports Linux; download a build manually from https://github.com/$REPO/releases" ;;
esac

case "$(uname -m)" in
    x86_64 | amd64) ASSET="marconi-linux-x86_64.tar.gz" ;;
    *) die "no prebuilt binary for $(uname -m); build from source instead" ;;
esac

TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT INT TERM

fetch() {
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$1" -o "$2"
    elif command -v wget >/dev/null 2>&1; then
        wget -qO "$2" "$1"
    else
        die "curl or wget is required"
    fi
}

echo "Downloading marconi..."
fetch "$BASE/$ASSET" "$TMP/$ASSET"
fetch "$BASE/SHA256SUMS" "$TMP/SHA256SUMS"

EXPECTED=$(awk -v a="$ASSET" '$2 == a { print $1; exit }' "$TMP/SHA256SUMS")
[ -n "$EXPECTED" ] || die "SHA256SUMS does not list $ASSET"

if command -v sha256sum >/dev/null 2>&1; then
    ACTUAL=$(sha256sum "$TMP/$ASSET" | awk '{ print $1 }')
elif command -v shasum >/dev/null 2>&1; then
    ACTUAL=$(shasum -a 256 "$TMP/$ASSET" | awk '{ print $1 }')
else
    die "sha256sum or shasum is required to verify the download"
fi

[ "$EXPECTED" = "$ACTUAL" ] || die "SHA-256 mismatch for $ASSET"

tar -xzf "$TMP/$ASSET" -C "$TMP"
[ -x "$TMP/marconi" ] || die "marconi is missing from the release archive"

SCOPE=""
if [ -r /proc/sys/kernel/yama/ptrace_scope ]; then
    SCOPE=$(cat /proc/sys/kernel/yama/ptrace_scope)
fi

if [ "$(id -u)" -ne 0 ] && [ -n "$SCOPE" ] && [ "$SCOPE" != "0" ] && [ "${MARCONI_NO_SUDO:-0}" != "1" ]; then
    if ! command -v sudo >/dev/null 2>&1; then
        echo "marconi: ptrace is restricted (kernel.yama.ptrace_scope=$SCOPE) and sudo is not installed." >&2
        die "run: sudo sysctl -w kernel.yama.ptrace_scope=0   (temporary, until reboot)"
    fi
    echo "ptrace is restricted (kernel.yama.ptrace_scope=$SCOPE); re-running with sudo..."
    echo "Set MARCONI_NO_SUDO=1 to see the manual instructions instead."
    sudo -E "$TMP/marconi" "$@"
    exit $?
fi

"$TMP/marconi" "$@"
