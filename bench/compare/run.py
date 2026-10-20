"""Zamak 与其他语言的对比测试：同一台机器、同样的程序，测编译时间、可执行文件体积、运行时间与峰值内存。

用法：python bench/compare/run.py [--zam zam/target/release/zam.exe] [--languages zamak,c,...]
结果写到 results/compare-<时间>/：results.json 与 summary.md。各语言的输出必须一致才算有效。
目前只支持 Windows（峰值内存用 GetProcessMemoryInfo 读取）。
"""
import argparse
import ctypes
import datetime
import json
import os
import random
import shutil
import statistics
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
PROGRAMS = Path(__file__).resolve().parent / 'programs'
# 任务与命令行参数。hello 测体积、启动与最小内存；其余四个来自 The Computer Language Benchmarks Game。
TASKS = {'hello': [], 'primes': [], 'wordfreq': [], 'nbody': ['2000000'], 'spectralnorm': ['2000'],
         'fannkuch': ['10'], 'binarytrees': ['16']}


class Counters(ctypes.Structure):
    _fields_ = [('cb', ctypes.c_ulong), ('PageFaultCount', ctypes.c_ulong),
                ('PeakWorkingSetSize', ctypes.c_size_t), ('WorkingSetSize', ctypes.c_size_t),
                ('QuotaPeakPagedPoolUsage', ctypes.c_size_t), ('QuotaPagedPoolUsage', ctypes.c_size_t),
                ('QuotaPeakNonPagedPoolUsage', ctypes.c_size_t), ('QuotaNonPagedPoolUsage', ctypes.c_size_t),
                ('PagefileUsage', ctypes.c_size_t), ('PeakPagefileUsage', ctypes.c_size_t)]


def run_once(command, stdin_path):
    """运行一次：返回（秒, 峰值工作集字节数, 标准输出, 程序内部计时秒数或 None）。

    外部计时（`seconds`）含进程启动、动态链接和退出，是用户实际感受到的耗时。
    程序自己在计算前后打一圈时钟，把 `TIME <毫秒>` 写到 stderr，得到纯计算耗时；
    两个数都记下来，才分得清「算得快」和「启动快」。
    """
    with open(stdin_path, 'rb') if stdin_path else open(os.devnull, 'rb') as stdin:
        start = time.perf_counter()
        proc = subprocess.Popen(command, stdin=stdin, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        out, err = proc.communicate()
        seconds = time.perf_counter() - start
    if proc.returncode != 0:
        raise RuntimeError(f'{command} failed: {err.decode(errors="replace")}')
    counters = Counters()
    counters.cb = ctypes.sizeof(Counters)
    ctypes.windll.psapi.GetProcessMemoryInfo(int(proc._handle), ctypes.byref(counters), counters.cb)
    inner = None
    for line in err.decode(errors='replace').splitlines():
        if line.startswith('TIME '):
            inner = float(line[5:]) / 1000.0
    return seconds, counters.PeakWorkingSetSize, out.decode().replace('\r\n', '\n'), inner


def timed(command, env=None, cwd=None):
    start = time.perf_counter()
    result = subprocess.run(command, capture_output=True, text=True, env=env, cwd=cwd)
    seconds = time.perf_counter() - start
    if result.returncode != 0:
        raise RuntimeError(f'{command} failed:\n{result.stdout}\n{result.stderr}')
    return seconds, result


class Skip(Exception):
    """这台机器上量不了的组合（例如没有 MSVC）。只跳过这一行，不中断整轮。"""


def vswhere_path():
    """MSVC 的安装路径；没有 Visual Studio 时返回 None。"""
    vswhere = Path(os.environ.get('ProgramFiles(x86)', 'C:/Program Files (x86)')) / \
        'Microsoft Visual Studio/Installer/vswhere.exe'
    if not vswhere.exists():
        return None
    r = subprocess.run([str(vswhere), '-latest', '-products', '*', '-property', 'installationPath'],
                       capture_output=True, text=True)
    path = r.stdout.strip()
    return path or None


def zam_command(zam):
    """`--zam` 可以给编译器的可执行文件，也可以给 npm 包里那个 .js 启动器。
    Windows 无法直接执行 .js，此时用当前解释器之外的 node 调起来。"""
    if str(zam).endswith('.js'):
        return ['node', str(zam)]
    return [str(zam)]


def git_commit():
    """当前检出的短 commit；不是 git 仓库（例如下载的源码包）时返回 unknown。"""
    try:
        r = subprocess.run(['git', 'rev-parse', '--short', 'HEAD'], cwd=ROOT,
                           capture_output=True, text=True)
        return r.stdout.strip() or 'unknown'
    except OSError:
        return 'unknown'


def build(language, task, work, zam):
    """编译（如需要）：返回（编译秒数或 None, 可执行文件或 None, 运行命令）。"""
    out = work / 'build' / language
    out.mkdir(parents=True, exist_ok=True)
    exe = out / f'{task}.exe'
    if language == 'zamak':
        # 与 C 基线用同一个 C 编译器（gcc -O2），比较的才是语言本身；冷编译：每个任务一个新缓存。
        env = dict(os.environ, ZAMAK_CACHE_DIR=str(work / 'zam-cache' / task), CC='gcc')
        seconds, result = timed(zam_command(zam) + ['build', str(PROGRAMS / 'zamak' / f'{task}.zm')], env=env)
        artifact = next(line[7:] for line in result.stderr.splitlines() if line.startswith('OUTPUT '))
        shutil.copy(artifact, exe)
        return seconds, exe, [str(exe)]
    if language == 'zamak-msvc':
        # Windows 用户装完 npm 包直接 `zam build` 走的是 MSVC，而 MSVC 静态链接运行库，
        # 体积与 gcc 口径差一倍以上。单独量一遍，免得对外只用 gcc 的数字。
        # 没有 MSVC 的机器上 skip()，不把整轮跑废。
        if sys.platform != 'win32' or shutil.which('cl') is None and not vswhere_path():
            raise Skip(f'{language}: no MSVC on this machine')
        env = dict(os.environ, ZAMAK_CACHE_DIR=str(work / 'zam-cache-msvc' / task))
        env.pop('CC', None)
        seconds, result = timed(zam_command(zam) + ['build', str(PROGRAMS / 'zamak' / f'{task}.zm')], env=env)
        artifact = next(line[7:] for line in result.stderr.splitlines() if line.startswith('OUTPUT '))
        shutil.copy(artifact, exe)
        return seconds, exe, [str(exe)]
    if language == 'c':
        seconds, _ = timed(['gcc', '-O2', '-o', str(exe), str(PROGRAMS / 'c' / f'{task}.c')])
        return seconds, exe, [str(exe)]
    if language == 'rust':
        seconds, _ = timed(['rustc', '-C', 'opt-level=3', '-o', str(exe), str(PROGRAMS / 'rust' / f'{task}.rs')])
        return seconds, exe, [str(exe)]
    # 「最小体积」口径：strip + 单代码单元 + LTO + panic=abort，是 Rust 用户压体积的常用组合。
    # 计时仍用默认口径（上一条），这里只额外量体积，免得把优化选项的代价算进运行时间。
    if language == 'rust-min':
        seconds, _ = timed(['rustc', '-C', 'opt-level=z', '-C', 'strip=symbols', '-C', 'lto=fat',
                            '-C', 'codegen-units=1', '-C', 'panic=abort', '-o', str(exe),
                            str(PROGRAMS / 'rust' / f'{task}.rs')])
        return seconds, exe, [str(exe)]
    if language == 'go':
        seconds, _ = timed(['go', 'build', '-o', str(exe), str(PROGRAMS / 'go' / f'{task}.go')])
        return seconds, exe, [str(exe)]
    if language == 'java':
        name = task.capitalize()
        seconds, _ = timed(['javac', '-d', str(out), str(PROGRAMS / 'java' / f'{name}.java')])
        return seconds, out / f'{name}.class', ['java', '-cp', str(out), name]
    if language == 'node':
        return None, None, ['node', str(PROGRAMS / 'node' / f'{task}.js')]
    if language == 'python':
        return None, None, [sys.executable, str(PROGRAMS / 'python' / f'{task}.py')]
    raise ValueError(language)


def make_words(path: Path, count=2_000_000, vocabulary=60_000, seed=7):
    """固定种子生成的文本：词长 2–10，出现频率近似齐夫分布。"""
    rng = random.Random(seed)
    letters = 'abcdefghijklmnopqrstuvwxyz'
    words = [''.join(rng.choice(letters) for _ in range(rng.randint(2, 10))) for _ in range(vocabulary)]
    weights = [1 / (k + 1) for k in range(vocabulary)]
    picked = rng.choices(words, weights=weights, k=count)
    lines = [' '.join(picked[i:i + 12]) for i in range(0, count, 12)]
    path.write_text('\n'.join(lines) + '\n', encoding='utf-8')


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--zam', default=str(ROOT / 'zam/target/release/zam.exe'))
    parser.add_argument('--languages', default='zamak,c,rust,go,java,node,python')
    # 只跑一部分程序（调试或复核某个口径时用）；不给就跑全部。
    parser.add_argument('--tasks', default='')
    args = parser.parse_args()
    work = ROOT / 'results' / ('compare-' + datetime.datetime.now().strftime('%Y%m%d-%H%M%S'))
    work.mkdir(parents=True)
    words = work / 'words.txt'
    make_words(words)
    languages = args.languages.split(',')
    tasks = [t for t in args.tasks.split(',') if t] or list(TASKS)
    unknown = [t for t in tasks if t not in TASKS]
    if unknown:
        raise SystemExit(f'unknown tasks: {unknown}; pick from {list(TASKS)}')
    repeats = dict.fromkeys(TASKS, 5) | {'hello': 10}
    slow = {'python': dict.fromkeys(TASKS, 1) | {'hello': 10, 'wordfreq': 3}}
    rows, outputs = [], {}
    for task in tasks:
        for language in languages:
            try:
                build_s, artifact, command = build(language, task, work, args.zam)
            except Skip as reason:
                print(f'skip {task}/{language}: {reason}', file=sys.stderr, flush=True)
                continue
            n = slow.get(language, {}).get(task, repeats[task])
            samples = [run_once(command + TASKS[task], words if task == 'wordfreq' else None) for _ in range(n)]
            # `rust-min` 与 `zamak-msvc` 只用于比体积：它们的运行时间不算进成绩（优化选项、
            # 另一个 C 编译器都会影响速度），输出仍需与其他语言一致，所以照常跑。
            outputs.setdefault(task, {})[language] = samples[0][2]
            row = {
                'task': task, 'language': language, 'runs': n,
                'build_s': build_s,
                'size_bytes': artifact.stat().st_size if artifact else None,
                'run_s': statistics.median(s[0] for s in samples),
                'peak_bytes': max(s[1] for s in samples),
            }
            # 程序内部计时：把「算得快」和「启动快」分开。只有写了 TIME 的程序才有。
            inners = [s[3] for s in samples if s[3] is not None]
            row['inner_s'] = statistics.median(inners) if inners else None
            rows.append(row)
            print(json.dumps(row), flush=True)
    # 各语言输出必须一致。
    for task, by_language in outputs.items():
        if len(set(by_language.values())) != 1:
            raise SystemExit(f'{task}: outputs differ: {by_language}')
    versions = {}
    for name, command in [('zam', zam_command(args.zam) + ['--version']), ('gcc', ['gcc', '--version']), ('rustc', ['rustc', '--version']),
                          ('go', ['go', 'version']), ('java', ['java', '-version']), ('node', ['node', '--version']),
                          ('python', [sys.executable, '--version'])]:
        try:
            r = subprocess.run(command, capture_output=True, text=True)
            versions[name] = (r.stdout or r.stderr).splitlines()[0]
        except OSError:
            pass
    # 记下跑这一轮的代码版本，否则同一个版本号的不同 commit 分不清。
    versions['zam_commit'] = git_commit()
    (work / 'results.json').write_text(json.dumps({'versions': versions, 'rows': rows}, indent=2), encoding='utf-8')
    print(f'results: {work}')


if __name__ == '__main__':
    main()
