"""把 bench/compare/run.py 的结果画成 README 用的对比图（深浅两套 SVG）。

用法：python bench/compare/chart.py results/compare-<时间>/results.json
输出：docs/compare-dark.svg、docs/compare-light.svg。数值越小越好的四项：运行时间、可执行文件体积、
峰值内存、启动时间。Zamak 用品牌渐变色，其他语言用中性色；每根柱子旁标出实测值。
"""
import json
import math
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
NAMES = {'zamak': 'Zamak', 'c': 'C', 'rust': 'Rust', 'go': 'Go', 'java': 'Java', 'node': 'Node.js', 'python': 'Python'}

THEMES = {
    'dark': dict(text='#e6edf3', muted='#9198a1', grid='#30363d', bar='#3d444d', track='#151b23'),
    'light': dict(text='#1f2328', muted='#59636e', grid='#d1d9e0', bar='#c8d1da', track='#f6f8fa'),
}
BRAND = [('0', '#f59e0b'), ('0.4', '#f97362'), ('0.75', '#6366f1'), ('1', '#06b6d4')]


def human_time(s):
    return f'{s * 1000:.0f} ms' if s < 1 else f'{s:.2f} s'


def human_bytes(b):
    if b >= 1 << 20:
        return f'{b / (1 << 20):.1f} MB'
    return f'{b / 1024:.0f} KB'


def panel(title, note, items, fmt, log, x, y, width, theme, gradient_id):
    """一个面板：标题、说明、横向柱子。items 是 [(语言, 数值或 None, 备注)]。"""
    t = THEMES[theme]
    out = [f'<text x="{x}" y="{y}" font-size="15" font-weight="600" fill="{t["text"]}">{title}</text>',
           f'<text x="{x}" y="{y + 18}" font-size="11.5" fill="{t["muted"]}">{note}</text>']
    label_w, value_w = 70, 74
    bar_x, bar_w = x + label_w, width - label_w - value_w
    values = [v for _, v, _ in items if v]
    top = max(values)
    low = min(values)

    def length(v):
        if not log:
            return bar_w * v / top
        # 对数刻度：最小值也留出一截，最大值占满。
        span = math.log10(top) - math.log10(low * 0.5)
        return bar_w * (math.log10(v) - math.log10(low * 0.5)) / span

    row_y = y + 34
    for language, value, remark in items:
        mine = language == 'zamak'
        out.append(f'<text x="{bar_x - 10}" y="{row_y + 13}" font-size="12.5" text-anchor="end" '
                   f'font-weight="{600 if mine else 400}" fill="{t["text"] if mine else t["muted"]}">{NAMES[language]}</text>')
        out.append(f'<rect x="{bar_x}" y="{row_y + 2}" width="{bar_w}" height="15" rx="3" fill="{t["track"]}"/>')
        if value:
            fill = f'url(#{gradient_id})' if mine else t['bar']
            out.append(f'<rect x="{bar_x}" y="{row_y + 2}" width="{max(length(value), 3):.1f}" height="15" rx="3" fill="{fill}"/>')
            out.append(f'<text x="{bar_x + bar_w + 8}" y="{row_y + 13}" font-size="12" '
                       f'font-weight="{600 if mine else 400}" fill="{t["text"] if mine else t["muted"]}">{fmt(value)}</text>')
        else:
            out.append(f'<text x="{bar_x + 8}" y="{row_y + 13}" font-size="11.5" fill="{t["muted"]}">{remark}</text>')
        row_y += 24
    return out, row_y


def chart(rows, theme):
    t = THEMES[theme]
    by = {(r['task'], r['language']): r for r in rows}
    order = ['zamak', 'c', 'rust', 'go', 'java', 'node', 'python']
    languages = [l for l in order if ('hello', l) in by]
    compiled = {'zamak', 'c', 'rust', 'go'}
    panels = [
        ('Run time · count primes below 1,000,000', 'Trial division, integer only. Lower is better (log scale).',
         [(l, by[('primes', l)]['run_s'], '') for l in languages], human_time, True),
        ('Executable size · hello world', 'A file you can copy to another machine and run. Lower is better.',
         [(l, by[('hello', l)]['size_bytes'] if l in compiled else None, 'needs a runtime installed') for l in languages],
         human_bytes, True),
        ('Peak memory · hello world', 'Peak working set of the whole process. Lower is better.',
         [(l, by[('hello', l)]['peak_bytes'], '') for l in languages], human_bytes, False),
        ('Startup · hello world', 'From launch to exit, including the runtime. Lower is better.',
         [(l, by[('hello', l)]['run_s'], '') for l in languages], human_time, False),
    ]
    width, col_w, gap = 920, 440, 40
    body, y_bottom = [], 0
    for k, (title, note, items, fmt, log) in enumerate(panels):
        col, row = k % 2, k // 2
        x = 20 + col * (col_w + gap)
        y = 30 + row * (34 + 24 * len(languages) + 46)
        part, end = panel(title, note, items, fmt, log, x, y, col_w, theme, 'brand')
        body += part
        y_bottom = max(y_bottom, end)
    height = y_bottom + 10
    stops = ''.join(f'<stop offset="{o}" stop-color="{c}"/>' for o, c in BRAND)
    return f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {height}" width="{width}" height="{height}" role="img" aria-label="Zamak compared with C, Rust, Go, Java, Node.js and Python" font-family="-apple-system, 'Segoe UI', Helvetica, Arial, sans-serif">
  <title>Zamak compared with other languages</title>
  <defs><linearGradient id="brand" x1="0" x2="1" y1="0" y2="0">{stops}</linearGradient></defs>
  {''.join(body)}
</svg>
'''


def main():
    data = json.loads(Path(sys.argv[1]).read_text(encoding='utf-8'))
    for theme in THEMES:
        (ROOT / 'docs' / f'compare-{theme}.svg').write_text(chart(data['rows'], theme), encoding='utf-8', newline='\n')
    print('ok')


if __name__ == '__main__':
    main()
