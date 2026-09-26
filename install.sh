#!/bin/sh
# Install CHAOS RPG on Linux or macOS from the latest GitHub release.
#
#   curl -fsSL https://raw.githubusercontent.com/Mattbusel/chaos-rpg/master/install.sh | sh
#
# Installs chaos-rpg, chaos-rpg-graphical and chaos-rpg-proof into ~/.local/bin
# (override with CHAOS_RPG_INSTALL_DIR) after checking the SHA-256 of the download.
# Pin a version with CHAOS_RPG_VERSION=v2.2.1.
set -eu

REPO="Mattbusel/chaos-rpg"
BIN_DIR="${CHAOS_RPG_INSTALL_DIR:-$HOME/.local/bin}"

say() { printf '%s\n' "$*"; }
die() { printf 'chaos-rpg install: %s\n' "$*" >&2; exit 1; }
need() { command -v "$1" >/dev/null 2>&1 || die "needs '$1' but it is not installed"; }

need curl
need tar
need uname

os=$(uname -s)
arch=$(uname -m)
case "$os" in
  Linux)
    case "$arch" in
      x86_64|amd64) target="x86_64-unknown-linux-gnu" ;;
      *) die "no prebuilt Linux build for $arch yet. Build from source: cargo install chaos-rpg-graphical" ;;
    esac ;;
  Darwin)
    case "$arch" in
      arm64|aarch64) target="aarch64-apple-darwin" ;;
      x86_64) target="x86_64-apple-darwin" ;;
      *) die "unknown Mac architecture $arch" ;;
    esac ;;
  MINGW*|MSYS*|CYGWIN*)
    die "on Windows use PowerShell: irm https://raw.githubusercontent.com/$REPO/master/install.ps1 | iex" ;;
  *) die "unsupported OS $os. Build from source: cargo install chaos-rpg-graphical" ;;
esac

if [ -n "${CHAOS_RPG_VERSION:-}" ]; then
  tag="$CHAOS_RPG_VERSION"
else
  # The releases/latest page redirects to .../releases/tag/vX.Y.Z
  tag=$(curl -fsSLI -o /dev/null -w '%{url_effective}' "https://github.com/$REPO/releases/latest" | sed 's|.*/tag/||')
fi
case "$tag" in v[0-9]*) ;; *) die "could not find the latest release (got '$tag')" ;; esac

name="chaos-rpg-$tag-$target"
url="https://github.com/$REPO/releases/download/$tag/$name.tar.gz"
tmp=$(mktemp -d 2>/dev/null || mktemp -d -t chaosrpg)
trap 'rm -rf "$tmp"' EXIT INT TERM

say "Downloading CHAOS RPG $tag for $target"
curl -fsSL "$url" -o "$tmp/$name.tar.gz" || die "download failed: $url"
curl -fsSL "$url.sha256" -o "$tmp/$name.tar.gz.sha256" || die "checksum download failed: $url.sha256"

want=$(awk '{print $1}' "$tmp/$name.tar.gz.sha256")
if command -v sha256sum >/dev/null 2>&1; then
  got=$(sha256sum "$tmp/$name.tar.gz" | awk '{print $1}')
elif command -v shasum >/dev/null 2>&1; then
  got=$(shasum -a 256 "$tmp/$name.tar.gz" | awk '{print $1}')
else
  die "needs sha256sum or shasum to check the download"
fi
[ "$want" = "$got" ] || die "checksum mismatch (expected $want, got $got). Nothing was installed."
say "Checksum OK"

tar -xzf "$tmp/$name.tar.gz" -C "$tmp"
mkdir -p "$BIN_DIR"
for b in chaos-rpg chaos-rpg-graphical chaos-rpg-proof; do
  install -m 755 "$tmp/$name/$b" "$BIN_DIR/$b"
done
# Settings file sits next to the programs, where the game looks for it.
[ -f "$BIN_DIR/chaos_config.toml" ] || cp "$tmp/$name/chaos_config.toml" "$BIN_DIR/chaos_config.toml"

say ""
say "Installed to $BIN_DIR:"
say "  chaos-rpg-graphical   the game in its own window (start here)"
say "  chaos-rpg-proof       the Proof Engine version (preview)"
say "  chaos-rpg             the terminal version"
case ":$PATH:" in
  *":$BIN_DIR:"*) ;;
  *) say ""; say "$BIN_DIR is not on your PATH. Add this line to your shell profile:"; say "  export PATH=\"$BIN_DIR:\$PATH\"" ;;
esac
if [ "$os" = "Linux" ]; then
  say ""
  say "The windowed versions need OpenGL and ALSA (on Debian/Ubuntu: sudo apt install libasound2 libgl1 libxkbcommon0)."
fi
