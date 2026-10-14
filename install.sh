#!/bin/sh
# Install the latest Zamak compiler on Linux x64 or macOS arm64:
#   curl -fsSL https://raw.githubusercontent.com/rayL-K/Zamak/main/install.sh | sh
# Environment: ZAMAK_HOME (install directory, default ~/.zamak), ZAMAK_VERSION (e.g. 0.1.0).
set -eu

repo="rayL-K/Zamak"
case "$(uname -s)-$(uname -m)" in
    Linux-x86_64) name=linux-x64 ;;
    Darwin-arm64) name=macos-arm64 ;;
    *)
        echo "zamak: no prebuilt compiler for $(uname -s) $(uname -m)." >&2
        echo "zamak: build from source instead: cargo install --git https://github.com/$repo zam --locked" >&2
        exit 1
        ;;
esac

if [ -n "${ZAMAK_DOWNLOAD_BASE:-}" ]; then
    base="$ZAMAK_DOWNLOAD_BASE"
elif [ -n "${ZAMAK_VERSION:-}" ]; then
    base="https://github.com/$repo/releases/download/v$ZAMAK_VERSION"
else
    base="https://github.com/$repo/releases/latest/download"
fi
home="${ZAMAK_HOME:-$HOME/.zamak}"
archive="zamak-$name.tar.gz"

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
echo "zamak: downloading $base/$archive"
curl -fsSL "$base/$archive" -o "$tmp/$archive"
curl -fsSL "$base/$archive.sha256" -o "$tmp/$archive.sha256"

expected=$(cut -d ' ' -f 1 "$tmp/$archive.sha256")
if command -v sha256sum >/dev/null 2>&1; then
    actual=$(sha256sum "$tmp/$archive" | cut -d ' ' -f 1)
else
    actual=$(shasum -a 256 "$tmp/$archive" | cut -d ' ' -f 1)
fi
if [ "$expected" != "$actual" ]; then
    echo "zamak: checksum mismatch for $archive" >&2
    exit 1
fi

tar -xzf "$tmp/$archive" -C "$tmp"
rm -rf "$home"
mkdir -p "$(dirname "$home")"
mv "$tmp/zamak-$name" "$home"
echo "zamak: installed $("$home/bin/zam" --version) to $home"

case ":$PATH:" in
    *":$home/bin:"*) ;;
    *) echo "zamak: add it to PATH, for example in ~/.profile:  export PATH=\"$home/bin:\$PATH\"" ;;
esac
if ! command -v cc >/dev/null 2>&1 && ! command -v clang >/dev/null 2>&1 && ! command -v gcc >/dev/null 2>&1; then
    if [ "$name" = macos-arm64 ]; then
        echo "zamak: no C compiler found; install one with: xcode-select --install"
    else
        echo "zamak: no C compiler found; install gcc or clang (for example: sudo apt install build-essential)"
    fi
fi
echo "zamak: try it with: zam new hello && zam run hello"
