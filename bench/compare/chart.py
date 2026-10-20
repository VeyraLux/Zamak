"""把 bench/compare/run.py 的结果画成 README 用的对比图（深浅两套 SVG）。

用法：python bench/compare/chart.py results/compare-<时间>/results.json
输出：docs/compare-speed-{dark,light}.svg（六个程序的运行时间）与
      docs/compare-footprint-{dark,light}.svg（可执行文件体积、启动时间与峰值内存）。
都是越小越好。Zamak 用品牌渐变色，其他语言用中性色，每根柱子旁标出实测值。刻度是线性的；
比其他语言慢一个数量级的柱子（通常是 Python）画到边界截断，并标出实际值，不用对数刻度把差距压扁。
"""
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
NAMES = {'zamak': 'Zamak', 'c': 'C', 'rust': 'Rust', 'go': 'Go', 'java': 'Java', 'node': 'Node.js', 'python': 'Python'}
ORDER = ['zamak', 'c', 'rust', 'go', 'java', 'node', 'python']
COMPILED = {'zamak', 'c', 'rust', 'go'}

THEMES = {
    'dark': dict(text='#e6edf3', muted='#9198a1', bar='#3d444d', track='#151b23'),
    'light': dict(text='#1f2328', muted='#59636e', bar='#c8d1da', track='#f6f8fa'),
}
BRAND = [('0', '#f59e0b'), ('0.4', '#f97362'), ('0.75', '#6366f1'), ('1', '#06b6d4')]


def human_time(s):
    return f'{s * 1000:.0f} ms' if s < 1 else f'{s:.2f} s'


def human_bytes(b):
    return f'{b / (1 << 20):.1f} MB' if b >= 1 << 20 else f'{b / 1024:.0f} KB'


def panel(title, note, items, fmt, x, y, width, theme):
    """一个面板：标题、说明、横向柱子。items 是 [(语言, 数值或 None, 没有数值时的说明)]。"""
    t = THEMES[theme]
    out = [f'<text x="{x}" y="{y}" font-size="14.5" font-weight="600" fill="{t["text"]}">{title}</text>',
           f'<text x="{x}" y="{y + 17}" font-size="11.5" fill="{t["muted"]}">{note}</text>']
    label_w, value_w = 66, 70
    bar_x, bar_w = x + label_w, width - label_w - value_w
    values = sorted(v for _, v, _ in items if v)
    # 刻度：比第二慢的还慢 4 倍以上的柱子截断，免得把其他柱子压扁。
    scale = values[-1]
    if len(values) > 2 and values[-1] > 4 * values[-2]:
        scale = values[-2] * 1.15
    row_y = y + 30
    for language, value, remark in items:
        mine = language == 'zamak'
        color = t['text'] if mine else t['muted']
        weight = 600 if mine else 400
        out.append(f'<text x="{bar_x - 9}" y="{row_y + 12}" font-size="12" text-anchor="end" font-weight="{weight}" fill="{color}">{NAMES[language]}</text>')
        out.append(f'<rect x="{bar_x}" y="{row_y + 2}" width="{bar_w}" height="13" rx="3" fill="{t["track"]}"/>')
        if value:
            clipped = value > scale
            length = bar_w if clipped else max(bar_w * value / scale, 3)
            fill = 'url(#brand)' if mine else t['bar']
            out.append(f'<rect x="{bar_x}" y="{row_y + 2}" width="{length:.1f}" height="13" rx="3" fill="{fill}"/>')
            if clipped:
                # 截断标记：两道斜线。
                for k in (0, 5):
                    sx = bar_x + bar_w * 0.82 + k
                    out.append(f'<path d="M{sx:.1f} {row_y + 17} l5 -17" stroke="{t["track"]}" stroke-width="2.4"/>')
            out.append(f'<text x="{bar_x + bar_w + 8}" y="{row_y + 12}" font-size="11.5" font-weight="{weight}" fill="{color}">{fmt(value)}</text>')
        else:
            out.append(f'<text x="{bar_x + 8}" y="{row_y + 12}" font-size="11" fill="{t["muted"]}">{remark}</text>')
        row_y += 21
    return out, row_y


def figure(panels, theme, label):
    width, col_w, gap = 920, 440, 40
    body, bottom = [], 0
    for k, (title, note, items, fmt) in enumerate(panels):
        x = 20 + (k % 2) * (col_w + gap)
        y = 28 + (k // 2) * (30 + 21 * len(items) + 40)
        part, end = panel(title, note, items, fmt, x, y, col_w, theme)
        body += part
        bottom = max(bottom, end)
    stops = ''.join(f'<stop offset="{o}" stop-color="{c}"/>' for o, c in BRAND)
    return f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {bottom + 8}" width="{width}" height="{bottom + 8}" role="img" aria-label="{label}" font-family="-apple-system, 'Segoe UI', Helvetica, Arial, sans-serif">
  <title>{label}</title>
  <defs><linearGradient id="brand" x1="0" x2="1" y1="0" y2="0">{stops}</linearGradient></defs>
  {''.join(body)}
</svg>
'''


def main():
    rows = json.loads(Path(sys.argv[1]).read_text(encoding='utf-8'))['rows']
    by = {(r['task'], r['language']): r for r in rows}
    languages = [l for l in ORDER if ('hello', l) in by]

    def series(task, key, compiled_only=False, remark=''):
        return [(l, by[(task, l)][key] if not compiled_only or l in COMPILED else None, remark) for l in languages]

    speed = [
        ('n-body', 'Floating point and struct fields: 2,000,000 steps.', series('nbody', 'run_s'), human_time),
        ('spectral-norm', 'Floating point and function calls: n = 2,000.', series('spectralnorm', 'run_s'), human_time),
        ('fannkuch-redux', 'Integer arrays and permutations: n = 10.', series('fannkuch', 'run_s'), human_time),
        ('binary-trees', 'Allocating and freeing many small trees: depth 16.', series('binarytrees', 'run_s'), human_time),
        ('primes', 'Integer division: count primes below 1,000,000.', series('primes', 'run_s'), human_time),
        ('word frequency', 'Strings and hash maps: 2,000,000 words.', series('wordfreq', 'run_s'), human_time),
    ]
    footprint = [
        ('Executable size', 'Hello world as a file you can copy and run.', series('hello', 'size_bytes', True, 'needs a runtime installed'), human_bytes),
        ('Startup time', 'Hello world, from launch to exit.', series('hello', 'run_s'), human_time),
        ('Peak memory · n-body', 'Peak working set of the whole process.', series('nbody', 'peak_bytes'), human_bytes),
        ('Peak memory · binary-trees', 'Peak working set of the whole process.', series('binarytrees', 'peak_bytes'), human_bytes),
    ]
    for theme in THEMES:
        (ROOT / 'docs' / f'compare-speed-{theme}.svg').write_text(
            figure(speed, theme, 'Run time of six programs in Zamak, C, Rust, Go, Java, Node.js, and Python'),
            encoding='utf-8', newline='\n')
        (ROOT / 'docs' / f'compare-footprint-{theme}.svg').write_text(
            figure(footprint, theme, 'Executable size, startup time, and peak memory of Zamak, C, Rust, Go, Java, Node.js, and Python'),
            encoding='utf-8', newline='\n')
    print('ok')


if __name__ == '__main__':
    main()
