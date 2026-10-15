#!/bin/sh
# hanfix installer for Linux and macOS:
#   curl -fsSL https://github.com/NaNIsaac/hanfix/releases/latest/download/install.sh | sh
# Puts the binary in $HANFIX_INSTALL_DIR (default ~/.local/bin). Pin a release with HANFIX_VERSION=v0.2.2.
# Run it again (or hanfix --update) to update.
set -eu

repo=NaNIsaac/hanfix
dir=${HANFIX_INSTALL_DIR:-$HOME/.local/bin}
version=${HANFIX_VERSION:-latest}

die() { echo "hanfix install: $*" >&2; exit 1; }

case $(uname -s) in
    Linux) os=unknown-linux-musl ;;
    Darwin) os=apple-darwin ;;
    *) die "unsupported system $(uname -s); on Windows run install.ps1 in PowerShell" ;;
esac
case $(uname -m) in
    x86_64 | amd64) arch=x86_64 ;;
    arm64 | aarch64) arch=aarch64 ;;
    *) die "unsupported CPU $(uname -m)" ;;
esac

asset=hanfix-$arch-$os
if [ "$version" = latest ]; then
    base=https://github.com/$repo/releases/latest/download
else
    base=https://github.com/$repo/releases/download/$version
fi

fetch() {
    if command -v curl >/dev/null 2>&1; then curl -fsSL "$1" -o "$2"
    elif command -v wget >/dev/null 2>&1; then wget -q "$1" -O "$2"
    else die "needs curl or wget"; fi
}

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
echo "hanfix: downloading $asset ($version)"
fetch "$base/$asset" "$tmp/hanfix" || die "download failed: $base/$asset"
fetch "$base/$asset.sha256" "$tmp/sum" || die "checksum download failed"

want=$(cut -d' ' -f1 "$tmp/sum")
if command -v sha256sum >/dev/null 2>&1; then got=$(sha256sum "$tmp/hanfix" | cut -d' ' -f1)
else got=$(shasum -a 256 "$tmp/hanfix" | cut -d' ' -f1); fi
[ "$want" = "$got" ] || die "checksum mismatch (expected $want, got $got)"

mkdir -p "$dir"
chmod +x "$tmp/hanfix"
mv "$tmp/hanfix" "$dir/hanfix"
echo "hanfix: installed $("$dir/hanfix" --version) to $dir/hanfix"

[ -n "${HANFIX_UPDATE:-}" ] && exit 0

case ":$PATH:" in
    *":$dir:"*) ;;
    *) echo "hanfix: $dir is not on PATH; add  export PATH=\"$dir:\$PATH\"  to your shell's start-up file" ;;
esac

case $(basename "${SHELL:-sh}") in
    bash) line='eval "$(hanfix --init bash)"'; rc='~/.bashrc' ;;
    zsh) line='eval "$(hanfix --init zsh)"'; rc='~/.zshrc' ;;
    fish) line='hanfix --init fish | source'; rc='~/.config/fish/config.fish' ;;
    *) line=; rc= ;;
esac
if [ -n "$line" ]; then
    echo "hanfix: to fix the previous command with hf, add this line to $rc and open a new shell:"
    echo "    $line"
fi
