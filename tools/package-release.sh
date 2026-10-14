#!/bin/bash
# 把编译好的 zam 和文档、示例打成发行目录 dist/<name>/：
#   tools/package-release.sh zam/target/release/zam zamak-linux-x64
set -euo pipefail
binary="$1"
name="$2"
root="$(cd "$(dirname "$0")/.." && pwd)"
out="$root/dist/$name"
rm -rf "$out"
mkdir -p "$out/bin"
cp "$binary" "$out/bin/"
for file in README.md README.zh-CN.md TUTORIAL.md LICENSE-MIT LICENSE-APACHE; do
    cp "$root/$file" "$out/"
done
# 只带示例源码，不带本地构建产物。
(cd "$root" && find examples -type f \( -name '*.zm' -o -name '*.toml' -o -name '*.md' -o -name '*.html' \) \
    -not -path '*/build/*' -print) | while read -r file; do
    mkdir -p "$out/$(dirname "$file")"
    cp "$root/$file" "$out/$file"
done
echo "packaged $out"
