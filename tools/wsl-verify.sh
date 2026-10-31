#!/bin/bash
# 在 WSL 里验证 Linux：克隆仓库到 WSL 文件系统（LF 检出），套上 Windows 侧导出的未提交补丁，
# 跑 fmt、gcc/clang 两套全量测试、clippy，再用 ASan/UBSan/LeakSanitizer 跑示例。
# 用法（pwsh，仓库根目录）：
#   git diff HEAD --output=zam/target/wsl.patch
#   wsl -d Ubuntu -u root -- env SANITIZE_FILES="examples/io/wc.zm" SANITIZE_FAILING="examples/io/wc.zm" bash tools/wsl-verify.sh
. ~/.cargo/env
repo="$(cd "$(dirname "$0")/.." && pwd)"
set -e
export CARGO_TARGET_DIR=~/zam-target
rm -rf ~/zamak
git clone -q "$repo" ~/zamak
cd ~/zamak
if [ -s "$repo/zam/target/wsl.patch" ]; then git apply "$repo/zam/target/wsl.patch"; fi
cargo fmt --manifest-path zam/Cargo.toml -- --check && echo FMT_OK
for cc in gcc clang; do
  echo "== $cc"
  CC=$cc cargo test --manifest-path zam/Cargo.toml --locked --no-fail-fast 2>&1 \
    | grep -E "^test result|FAILED|panicked" \
    | awk '/test result/{p+=$4; f+=$6} /FAILED|panicked/{print} END {print "passed=" p, "failed=" f}'
done
cargo clippy --manifest-path zam/Cargo.toml --all-targets --locked -q -- -D warnings && echo CLIPPY_OK
cat > ~/asan-cc <<'EOF'
#!/bin/sh
exec gcc -fsanitize=address,undefined -fno-sanitize-recover=all -fno-omit-frame-pointer "$@"
EOF
chmod +x ~/asan-cc
set +e
printf '3\n 4 \n\n-10\n' > /tmp/zam-asan-in.txt
printf 'hello world\n' > /tmp/zam-asan-arg.txt
for f in ${SANITIZE_FILES:-examples/control/loops.zm}; do
  CC=~/asan-cc ASAN_OPTIONS=detect_leaks=1 ZAMAK_CACHE_DIR=/tmp/zam-asan ~/zam-target/debug/zam run $f -- /tmp/zam-asan-arg.txt < /tmp/zam-asan-in.txt > /tmp/out.txt 2>/tmp/err.txt
  code=$?
  echo "$f exit=$code sanitizer_reports=$(grep -c -E 'ERROR: (Address|Leak)Sanitizer|runtime error' /tmp/err.txt)"
done
# 未捕获错误一路冒出 main 的路径：错误字符串也要释放。
for f in ${SANITIZE_FAILING:-}; do
  CC=~/asan-cc ASAN_OPTIONS=detect_leaks=1 ZAMAK_CACHE_DIR=/tmp/zam-asan ~/zam-target/debug/zam run $f -- /nonexistent/zam-input < /dev/null > /tmp/out.txt 2>/tmp/err.txt
  code=$?
  echo "$f (failing) exit=$code sanitizer_reports=$(grep -c -E 'ERROR: (Address|Leak)Sanitizer|runtime error' /tmp/err.txt)"
done
