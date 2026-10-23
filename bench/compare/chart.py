"""把 bench/compare/run.py 的结果画成 README 用的对比图（深浅两套 SVG）。

用法：python bench/compare/chart.py results/compare-<时间>/results.json
输出：docs/compare-speed-{dark,light}.svg（六个程序的运行时间）与
      docs/compare-footprint-{dark,light}.svg（可执行文件体积、启动时间与峰值内存）。

都是越小越好。每个面板是一组横向柱子：
- 只有 Zamak 用品牌渐变，其余语言是同一种中性色（只有一个系列需要辨认，不需要图例）；
- 一条贯穿面板的虚线标出 Zamak 的位置，一眼看出谁比它快、谁比它慢；
- 数值紧跟在柱子末端；比其他语言慢一个数量级的柱子（通常是 Python）画到边界并断开，
  标出实际值，不用对数刻度把差距压扁；
- 对照组（如 binary-trees 的 C arena 版）画成虚线框，和正式结果区分开。
"""
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
NAMES = {'zamak': 'Zamak', 'c': 'C', 'rust': 'Rust', 'go': 'Go', 'java': 'Java', 'node': 'Node.js',
         'python': 'Python', 'c-arena': 'C, arena'}
ORDER = ['zamak', 'c', 'rust', 'go', 'java', 'node', 'python']
COMPILED = {'zamak', 'c', 'rust', 'go'}

# GitHub 的页面底色；文字、柱子与参考线都按它校过对比度。
THEMES = {
    'dark': dict(surface='#0d1117', text='#e6edf3', muted='#9198a1', faint='#6e7681', bar='#4a525c', rule='#30363d'),
    'light': dict(surface='#ffffff', text='#1f2328', muted='#59636e', faint='#818b98', bar='#c3cad3', rule='#d1d9e0'),
}
BRAND = [('0', '#f59e0b'), ('0.4', '#f97362'), ('0.75', '#6366f1'), ('1', '#06b6d4')]
FONT = "-apple-system, BlinkMacSystemFont, 'Segoe UI', 'Noto Sans', Helvetica, Arial, sans-serif"

ROW = 22          # 每行的高度
BAR = 12          # 柱子的粗细
LABEL_W = 72      # 左侧语言名的宽度
VALUE_W = 74      # 右侧留给数值的宽度


def human_time(s):
    # 四舍五入取整（不用 Python 的银行家舍入），与 bench/compare/README.md 的表格一致。
    return f'{int(s * 1000 + 0.5)} ms' if s < 1 else f'{int(s * 100 + 0.5) / 100:.2f} s'


def human_bytes(b):
    return f'{b / (1 << 20):.1f} MB' if b >= 1 << 20 else f'{b / 1024:.0f} KB'


def bar_path(x, y, length, height, radius):
    """只有数据端（右端）是圆角，起点贴着基线是直角。"""
    r = min(radius, length / 2, height / 2)
    return (f'M{x:.1f} {y:.1f} h{length - r:.1f} a{r} {r} 0 0 1 {r} {r} v{height - 2 * r:.1f} '
            f'a{r} {r} 0 0 1 {-r} {r} h{-(length - r):.1f} z')


def panel(spec, x, y, width, theme):
    """一个面板。spec：title、note、rows=[(语言, 数值或 None, 没有数值时的说明)]、fmt、controls（对照组行）。"""
    t = THEMES[theme]
    title, note, rows, fmt = spec['title'], spec['note'], spec['rows'], spec['fmt']
    controls = spec.get('controls', [])
    out = [f'<text x="{x}" y="{y}" font-size="15" font-weight="600" fill="{t["text"]}">{title}</text>',
           f'<text x="{x}" y="{y + 18}" font-size="12" fill="{t["muted"]}">{note}</text>']
    bar_x, bar_w = x + LABEL_W, width - LABEL_W - VALUE_W
    values = sorted([v for _, v, _ in rows if v] + [v for _, v in controls if v])
    # 量程：比第二慢的还慢 3 倍以上的值算越界，画到边界并断开。
    scale = values[-1]
    if len(values) > 2 and values[-1] > 3 * values[-2]:
        scale = values[-2] * 1.08
    top = y + 34
    all_rows = [(l, v, r, False) for l, v, r in rows] + [(l, v, '', True) for l, v in controls]
    bottom = top + ROW * len(all_rows)
    mine = dict((l, v) for l, v, _ in rows).get('zamak')
    # Zamak 的参考线：画在柱子下面，从第一行到最后一行。
    if mine:
        rx = bar_x + bar_w * min(mine / scale, 1)
        out.append(f'<line x1="{rx:.1f}" y1="{top - 2}" x2="{rx:.1f}" y2="{bottom - 4}" stroke="{t["faint"]}" '
                   f'stroke-width="1" stroke-dasharray="3 3"/>')
    row_y = top
    for language, value, remark, control in all_rows:
        is_zamak = language == 'zamak'
        ink = t['text'] if is_zamak else t['muted']
        weight = 600 if is_zamak else 400
        cy = row_y + ROW / 2
        style = ' font-style="italic"' if control else ''
        out.append(f'<text x="{bar_x - 10}" y="{cy + 4:.1f}" font-size="12.5" text-anchor="end" '
                   f'font-weight="{weight}" fill="{ink}"{style}>{NAMES[language]}</text>')
        if value:
            over = value > scale
            length = bar_w if over else max(bar_w * value / scale, 3)
            by = cy - BAR / 2
            if control:
                # 对照组：虚线框，不填色，和正式结果区分开。
                out.append(f'<rect x="{bar_x + 0.5}" y="{by + 0.5:.1f}" width="{length - 1:.1f}" height="{BAR - 1}" rx="3" '
                           f'fill="none" stroke="{t["muted"]}" stroke-width="1" stroke-dasharray="3 2"/>')
            else:
                fill = 'url(#brand)' if is_zamak else t['bar']
                out.append(f'<path d="{bar_path(bar_x, by, length, BAR, 4)}" fill="{fill}"/>')
            if over:
                # 断开标记：在柱子末端前切出两道斜缝（用底色画），表示没有按比例画完。
                for k in (0, 6):
                    sx = bar_x + length - 26 + k
                    out.append(f'<path d="M{sx:.1f} {by + BAR + 2:.1f} l5 {-(BAR + 4)}" stroke="{t["surface"]}" stroke-width="2.6"/>')
            # 数值紧跟在柱子末端；越界的柱子顶满宽度，数值照样写在它后面。
            # 文字描一圈底色，参考虚线从后面穿过时不会压住数字。
            out.append(f'<text x="{bar_x + length + 7:.1f}" y="{cy + 4:.1f}" font-size="12" font-weight="{weight}" '
                       f'fill="{ink}" stroke="{t["surface"]}" stroke-width="4" paint-order="stroke"{style}>{fmt(value)}</text>')
        else:
            out.append(f'<text x="{bar_x}" y="{cy + 4:.1f}" font-size="11.5" fill="{t["faint"]}" '
                       f'stroke="{t["surface"]}" stroke-width="4" paint-order="stroke">{remark}</text>')
        row_y += ROW
    return out, bottom


def figure(panels, theme, label):
    width, col_w, gap = 920, 430, 40
    body, bottom = [], 0
    heights = [34 + ROW * (len(p['rows']) + len(p.get('controls', []))) for p in panels]
    y = 26
    for k in range(0, len(panels), 2):
        pair = panels[k:k + 2]
        row_h = max(heights[k:k + 2])
        for j, spec in enumerate(pair):
            part, end = panel(spec, 20 + j * (col_w + gap), y, col_w, theme)
            body += part
            bottom = max(bottom, end)
        y += row_h + 36
    stops = ''.join(f'<stop offset="{o}" stop-color="{c}"/>' for o, c in BRAND)
    t = THEMES[theme]
    legend_y = bottom + 22
    legend = (f'<line x1="20" y1="{legend_y - 4}" x2="34" y2="{legend_y - 4}" stroke="{t["faint"]}" stroke-dasharray="3 3"/>'
              f'<text x="40" y="{legend_y}" font-size="11.5" fill="{t["muted"]}">Zamak\'s value, for reference. Lower is better.</text>')
    height = legend_y + 10
    return f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {height}" width="{width}" height="{height}" role="img" aria-label="{label}" font-family="{FONT}">
  <title>{label}</title>
  <defs><linearGradient id="brand" x1="0" x2="1" y1="0" y2="0">{stops}</linearGradient></defs>
  {''.join(body)}
  {legend}
</svg>
'''


def main():
    rows = json.loads(Path(sys.argv[1]).read_text(encoding='utf-8'))['rows']
    by = {(r['task'], r['language']): r for r in rows}
    languages = [l for l in ORDER if ('hello', l) in by]

    def series(task, key, compiled_only=False, remark=''):
        return [(l, by[(task, l)][key] if not compiled_only or l in COMPILED else None, remark) for l in languages]

    def control(task, language, key='run_s'):
        row = by.get((task, language))
        return [(language, row[key])] if row and row.get(key) else []

    speed = [
        dict(title='n-body', note='Floating point and struct fields: 2,000,000 steps.',
             rows=series('nbody', 'run_s'), fmt=human_time),
        dict(title='spectral-norm', note='Function calls; Zamak marks one hot function @wrapping.',
             rows=series('spectralnorm', 'run_s'), fmt=human_time),
        dict(title='fannkuch-redux', note='Integer arrays and permutations: n = 10.',
             rows=series('fannkuch', 'run_s'), fmt=human_time),
        dict(title='binary-trees', note='Measures the allocator: C with an arena is the dashed row.',
             rows=series('binarytrees', 'run_s'), controls=control('binarytrees', 'c-arena'), fmt=human_time),
        dict(title='primes', note='Zamak and Rust divide in 32 bits; the C reference uses 64.',
             rows=series('primes', 'run_s'), fmt=human_time),
        dict(title='word frequency', note='Strings and hash maps: 2,000,000 words.',
             rows=series('wordfreq', 'run_s'), fmt=human_time),
    ]
    footprint = [
        dict(title='Executable size', note='Hello world, built on Windows with GCC. Linux: 16 KB; MSVC: 143 KB.',
             rows=series('hello', 'size_bytes', True, 'needs a runtime installed'), fmt=human_bytes),
        dict(title='Startup time', note='Hello world, from launch to exit.',
             rows=series('hello', 'run_s'), fmt=human_time),
        dict(title='Peak memory: n-body', note='Peak working set of the whole process.',
             rows=series('nbody', 'peak_bytes'), fmt=human_bytes),
        dict(title='Peak memory: binary-trees', note='Peak working set of the whole process.',
             rows=series('binarytrees', 'peak_bytes'), fmt=human_bytes),
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
