"""用法：python tools/make-logo.py，生成 docs/logo-dark.svg、docs/logo-light.svg 与预览页 zam/target/logo-preview.html。

Zamak 动态 logo：7×11 像素点阵拼出 `> ZAMAK`，其中的 Ƶ 就是吉祥物小 Ƶ。入场与宣传片的登场镜头相同：
小 Ƶ 先落地，A、M、A、K 每半秒（120 BPM 的一拍）落下一个，提示符 > 从左边滑进来；接着小光滴从小 Ƶ 身上
滴下去，大光滴落进最后一格并呼吸发光；之后一道光时不时扫过（构建完成），小 Ƶ 时不时眨眼。
呼应 VeyraLux 微睿霖光：霖 = 连绵的雨（光滴），光 = 光点。"""
import base64
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DOCS = ROOT / 'docs'

# 字形：7×11 点阵，笔画 2 格粗。Ƶ 就是吉祥物小 Ƶ 本人：与方形图标、宣传片结尾相同的 9 列身体
# （中间一横贯穿全宽），带眼睛和两只小脚。最后一格是光滴。
MASCOT = ['111111111', '111111111', '000000111', '000001110', '000011100', '111111111',
          '001110000', '011100000', '111000000', '111111111', '111111111']
GLYPHS = {
    '>': ['1100000', '0110000', '0011000', '0001100', '0000110', '0000011', '0000110', '0001100', '0011000', '0110000', '1100000'],
    'Z': MASCOT,
    'A': ['0011100', '0111110', '1100011', '1100011', '1100011', '1111111', '1111111', '1100011', '1100011', '1100011', '1100011'],
    'M': ['1100011', '1110111', '1111111', '1101011', '1101011', '1100011', '1100011', '1100011', '1100011', '1100011', '1100011'],
    'K': ['1100011', '1100111', '1101110', '1111100', '1111000', '1110000', '1111000', '1111100', '1101110', '1100111', '1100011'],
    'drop': ['0001000', '0001000', '0011100', '0011100', '0111110', '0111110', '1111111', '1111111', '1111111', '0111110', '0011100'],
}
TEXT = ['>', 'Z', 'A', 'M', 'A', 'K', 'drop']
EYE = '#0b1020'
PITCH = 8.5                  # 一格 8.5（7×11 点阵）
CELL = 0.84 * PITCH          # 字母与光滴的像素点边长，与宣传片结尾相同
BODY = 0.90 * PITCH          # 小 Ƶ 身体的像素点边长，与吉祥物相同
GAP = PITCH - CELL
PAD_X, PAD_Y = 14, 30        # 上方留出弹跳的空间
# 每个字符占的列数：小 Ƶ 9 列，其余 7 列；字符之间空 1 列。
SLOT_X = []
_x = 0.0
for _name in TEXT:
    SLOT_X.append(_x)
    _x += (9 if _name == 'Z' else 7) * PITCH + PITCH
WIDTH = round(PAD_X * 2 + _x - PITCH + 24, 1)      # 右边给光滴的光晕与闪烁留空
HEIGHT = round(PAD_Y + 12 * PITCH + 6, 1)          # 底下给小 Ƶ 的脚留空
ZX = PAD_X + SLOT_X[1]                             # 小 Ƶ 的左边


def pixels():
    """(字符序号, 列, 行, x, y)。"""
    out = []
    for i, name in enumerate(TEXT):
        for row, bits in enumerate(GLYPHS[name]):
            for col, bit in enumerate(bits):
                if bit == '1':
                    out.append((i, col, row, PAD_X + SLOT_X[i] + col * PITCH, PAD_Y + row * PITCH))
    return out


# 打开页面时只演一次：字母踩着拍子落下就位，之后保持不动；扫光每隔几秒扫一次，光滴持续呼吸。
BEAT = 0.5                   # 宣传片的节拍（120 BPM）
FALL = 1.0                   # 一个字母从落下到停稳的秒数（0.3 秒时落地）
# 每个字符开始落下的时刻（秒），落地正好在拍点上：小 Ƶ、A、M、A、K 依次各隔一拍，> 再隔一拍滑进来。
START = {1: 0.0, 2: BEAT, 3: 2 * BEAT, 4: 3 * BEAT, 5: 4 * BEAT}
PROMPT_AT = 5 * BEAT
HOP_T = 1.3                  # 光滴的弹跳更高、更久
DRIP_T = 0.75                # 小光滴从小 Ƶ 右上角滴到右下角的秒数
DRIP_FALL = 9.6 * 8.5        # 小光滴落下的距离：从顶上一横到底下一横
SWEEP = 5.0                  # 扫光的间隔


def pct(seconds: float, total: float) -> str:
    return f'{seconds / total * 100:.2f}%'


# 弹跳：加速落下 → 落地压扁 → 弹起拉长 → 再落 → 小弹一下 → 停稳。
BOUNCE = f'''@keyframes bounce {{
      0% {{ transform: translateY(-120px) scale(1, 1); opacity: 0; animation-timing-function: cubic-bezier(.55, 0, 1, .45); }}
      {pct(0.30, FALL)} {{ transform: translateY(0) scale(1.28, .7); opacity: 1; animation-timing-function: cubic-bezier(0, .55, .45, 1); }}
      {pct(0.46, FALL)} {{ transform: translateY(-16px) scale(.9, 1.12); animation-timing-function: cubic-bezier(.55, 0, 1, .45); }}
      {pct(0.60, FALL)} {{ transform: translateY(0) scale(1.14, .84); animation-timing-function: cubic-bezier(0, .55, .45, 1); }}
      {pct(0.70, FALL)} {{ transform: translateY(-5px) scale(.97, 1.04); animation-timing-function: cubic-bezier(.55, 0, 1, .45); }}
      {pct(0.78, FALL)} {{ transform: translateY(0) scale(1.04, .96); }}
      {pct(0.86, FALL)}, 100% {{ transform: translateY(0) scale(1, 1); opacity: 1; }}
    }}'''
# 光滴整颗落下，弹得更高。
HOP = f'''@keyframes hop {{
      0% {{ transform: translateY(-96px) scale(1, 1); opacity: 0; animation-timing-function: cubic-bezier(.55, 0, 1, .45); }}
      {pct(0.36, HOP_T)} {{ transform: translateY(0) scale(1.3, .68); opacity: 1; animation-timing-function: cubic-bezier(0, .55, .45, 1); }}
      {pct(0.56, HOP_T)} {{ transform: translateY(-30px) scale(.88, 1.16); animation-timing-function: cubic-bezier(.55, 0, 1, .45); }}
      {pct(0.74, HOP_T)} {{ transform: translateY(0) scale(1.16, .82); animation-timing-function: cubic-bezier(0, .55, .45, 1); }}
      {pct(0.86, HOP_T)} {{ transform: translateY(-9px) scale(.96, 1.05); animation-timing-function: cubic-bezier(.55, 0, 1, .45); }}
      {pct(0.96, HOP_T)}, 100% {{ transform: translateY(0) scale(1, 1); opacity: 1; }}
    }}'''


def svg(colors, prompt_alpha, halo, halo_alpha, glow, core, feet):
    """colors：提示符与 A、M、A、K 的颜色（字符序号 → 颜色），与宣传片结尾的 logo 相同；
    小 Ƶ 用吉祥物的对角渐变。halo：光滴光晕的颜色。core：光滴从中心到边缘的颜色（中心最亮）。"""
    groups = {i: [] for i in range(len(TEXT) - 1)}
    drop, shines, eyes, lids = [], [], [], []
    for i, col, row, px, py in pixels():
        name = TEXT[i]
        if name == 'Z':
            size, x, y, rx, fill = BODY, px + 0.05 * PITCH, py + 0.05 * PITCH, 0.18 * PITCH, 'url(#zink)'
        else:
            size, x, y, rx, fill = CELL, px, py, 0.2 * PITCH, colors.get(i)
        rect = f'<rect x="{x:.2f}" y="{y:.2f}" width="{size:.2f}" height="{size:.2f}" rx="{rx:.2f}"'
        if name == 'drop':
            drop.append(rect + ' fill="url(#core)"/>')
            continue
        shines.append(rect + '/>')
        opacity = f' fill-opacity="{prompt_alpha}"' if i == 0 else ''
        groups[i].append(f'{rect} fill="{fill}"{opacity}/>')
    # 眼睛：小 Ƶ 顶上那一横的第 3、7 列（从 0 数是 2、6），连起来的两格高、小一号的深色圆角长方形，
    # 与吉祥物相同；跟着小 Ƶ 一起落下。眼皮是同色同形的一层，落定后时不时眨眼。
    for c in (2, 6):
        eye = (f'<rect x="{ZX + (c + 0.14) * PITCH:.2f}" y="{PAD_Y + 0.16 * PITCH:.2f}" width="{0.72 * PITCH:.2f}" '
               f'height="{1.68 * PITCH:.2f}" rx="{0.22 * PITCH:.2f}"')
        eyes.append(f'{eye} fill="{EYE}"/>')
        lids.append(f'{eye} fill="url(#zink)" class="lid"/>')
    # 两只小脚：与吉祥物相同的位置和大小。
    legs = ''.join(
        f'<rect x="{ZX + fx * PITCH:.2f}" y="{PAD_Y + 11.05 * PITCH:.2f}" width="{1.6 * PITCH:.2f}" height="{0.9 * PITCH:.2f}" '
        f'rx="{0.3 * PITCH:.2f}" fill="{feet}"/>'
        for fx in (1.2, 6.2))
    groups[1] += eyes + [legs] + lids
    # 每个字符一个组，整个字母一起落下；提示符 > 从左边滑进来。
    letters = ''.join(
        f'<g class="{"slide" if i == 0 else "px"}" style="animation-delay:{PROMPT_AT if i == 0 else START[i]:.2f}s">{"".join(g)}</g>'
        for i, g in groups.items())
    drip_delay = PROMPT_AT + 0.6            # > 滑到位后，小光滴从小 Ƶ 身上滴下来
    drop_delay = drip_delay + DRIP_T - 0.05  # 小光滴一消失，大光滴就从最右边落下
    # 小光滴：与大光滴同形的 5×6 小点阵，从小 Ƶ 右上角（身体右边外侧）落到右下角。
    drip_x = ZX + 9.15 * PITCH
    drip_unit = 0.46 * PITCH
    drip = ''.join(
        f'<rect x="{drip_x + c * drip_unit:.2f}" y="{PAD_Y - 0.6 * PITCH + r * drip_unit:.2f}" width="{drip_unit * 0.9:.2f}" '
        f'height="{drip_unit * 0.9:.2f}" rx="{drip_unit * 0.25:.2f}"/>'
        for r, row in enumerate(['00100', '01110', '01110', '11111', '11111', '01110'])
        for c, bit in enumerate(row) if bit == '1')
    landed = drop_delay + HOP_T
    blink = landed + 0.35
    # 光滴的位置：最后一格，光晕的中心落在水滴的腹部。
    x0 = PAD_X + SLOT_X[6]
    cx = x0 + 3.5 * PITCH - GAP / 2
    cy = PAD_Y + 7.4 * PITCH
    # 高光：左上方两颗白色像素，像玻璃珠上的反光点。
    highlight = ''.join(
        f'<rect x="{x0 + c * PITCH:.2f}" y="{PAD_Y + r * PITCH:.2f}" width="{CELL:.2f}" height="{CELL:.2f}" rx="{0.2 * PITCH:.2f}" fill="#fff" fill-opacity="{o}"/>'
        for r, c, o in [(5, 2, 0.9), (6, 1, 0.6)])
    # 闪烁：周围散落的小光点，依次一闪一闪，像光往外溅。
    sparkles = ''.join(
        f'<rect class="spark" x="{cx + dx - 2:g}" y="{cy + dy - 2:g}" width="4" height="4" rx="1" fill="{halo}" style="animation-delay:{landed + k * 0.7:.2f}s"/>'
        for k, (dx, dy) in enumerate([(24, -48), (40, -16), (34, 22), (-6, -62)]))
    zink = (f'<linearGradient id="zink" gradientUnits="userSpaceOnUse" x1="{ZX:.2f}" y1="{PAD_Y}" x2="{ZX + 9 * PITCH:.2f}" y2="{PAD_Y + 11 * PITCH:.2f}">'
            '<stop offset="0" stop-color="#f59e0b"/><stop offset="0.4" stop-color="#f97362"/>'
            '<stop offset="0.75" stop-color="#6366f1"/><stop offset="1" stop-color="#06b6d4"/></linearGradient>')
    glow_filter = ('<filter id="glow" x="-10%" y="-30%" width="120%" height="160%">'
                   f'<feGaussianBlur stdDeviation="{glow}" result="b"/>'
                   '<feMerge><feMergeNode in="b"/><feMergeNode in="SourceGraphic"/></feMerge></filter>') if glow else ''
    group_filter = ' filter="url(#glow)"' if glow else ''
    return f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {WIDTH} {HEIGHT}" width="{WIDTH}" height="{HEIGHT}" role="img" aria-label="Zamak">
  <title>Zamak</title>
  <style>
    .px, .drop {{ transform-box: fill-box; transform-origin: 50% 100%; }}
    .px {{ animation: bounce {FALL}s both; }}
    .slide {{ animation: slide .45s cubic-bezier(.34, 1.56, .64, 1) both; }}
    @keyframes slide {{ 0% {{ transform: translateX(-70px); opacity: 0; }} 100% {{ transform: translateX(0); opacity: 1; }} }}
    .drop {{ animation: hop {HOP_T}s {drop_delay:.2f}s both; }}
    .drip {{ transform-box: fill-box; transform-origin: 50% 0; animation: drip {DRIP_T}s {drip_delay:.2f}s both; }}
    .breath {{ transform-box: fill-box; transform-origin: 50% 60%; animation: breath 2.4s {landed:.2f}s ease-in-out infinite; }}
    .halo {{ transform-box: fill-box; transform-origin: center; opacity: 0; animation: halo 2.4s {landed:.2f}s ease-in-out infinite; }}
    .spark {{ transform-box: fill-box; transform-origin: center; opacity: 0; animation: spark 2.8s ease-in-out infinite; }}
    .band {{ animation: sweep {SWEEP}s {landed - 0.6:.2f}s ease-in-out infinite both; }}
    .lid {{ transform-box: fill-box; transform-origin: 50% 0; transform: scaleY(0); animation: blink 6.4s {blink:.2f}s ease-in-out infinite both; }}
    {BOUNCE}
    {HOP}
    /* 光滴落定后呼吸：吸气时光滴微微胀大、光晕向外扩散，呼气时收回；亮度不降。 */
    /* 小光滴：冒出来（变大）→ 加速落下 → 到小 Ƶ 右下角缩小消失。 */
    @keyframes drip {{
      0% {{ transform: translateY(0) scale(.2); opacity: 0; }}
      18% {{ transform: translateY(0) scale(1); opacity: 1; animation-timing-function: cubic-bezier(.55, 0, 1, .45); }}
      82% {{ transform: translateY({DRIP_FALL:.1f}px) scale(.95, 1.1); opacity: 1; }}
      100% {{ transform: translateY({DRIP_FALL + 4:.1f}px) scale(.1); opacity: 0; }}
    }}
    @keyframes breath {{ 0%, 100% {{ transform: scale(1); }} 50% {{ transform: scale(1.06); }} }}
    @keyframes halo {{ 0%, 100% {{ opacity: .25; transform: scale(.8); }} 50% {{ opacity: .75; transform: scale(1.15); }} }}
    /* 都落定后时不时眨眼：眼皮从上往下合上再睁开。每 6.4 秒一轮：先眨一下，过一会儿连眨两下，像活的。 */
    @keyframes blink {{ 0%, 5%, 55%, 60%, 66%, 100% {{ transform: scaleY(0); }} 2.5%, 57.5%, 63% {{ transform: scaleY(1); }} }}
    @keyframes spark {{ 0%, 60%, 100% {{ opacity: 0; transform: scale(.4) rotate(0deg); }} 72% {{ opacity: 1; transform: scale(1.3) rotate(45deg); }} 84% {{ opacity: 0; transform: scale(.6) rotate(90deg); }} }}
    /* 都落定后一道光扫过（构建完成），之后每隔几秒再扫一次。 */
    @keyframes sweep {{ 0% {{ transform: translateX(-140px); }} 24%, 100% {{ transform: translateX({WIDTH + 60}px); }} }}
    @media (prefers-reduced-motion: reduce) {{
      .px, .slide, .drop, .breath, .band, .spark, .lid {{ animation: none; }}
      .drip {{ animation: none; opacity: 0; }}
      .halo {{ animation: none; opacity: 1; }}
      .band {{ transform: translateX(-140px); }}
    }}
  </style>
  <defs>
    {zink}
    <radialGradient id="core" gradientUnits="userSpaceOnUse" cx="{cx - 6:g}" cy="{cy - 4:g}" r="36">
      <stop offset="0" stop-color="{core[0]}"/>
      <stop offset="0.45" stop-color="{core[1]}"/>
      <stop offset="1" stop-color="{core[2]}"/>
    </radialGradient>
    <filter id="dropglow" x="-70%" y="-70%" width="240%" height="240%">
      <feGaussianBlur in="SourceGraphic" stdDeviation="3" result="b"/>
      <feMerge><feMergeNode in="b"/><feMergeNode in="SourceGraphic"/></feMerge>
    </filter>
    <radialGradient id="halo-fill">
      <stop offset="0" stop-color="{halo}" stop-opacity="{halo_alpha}"/>
      <stop offset="0.45" stop-color="{halo}" stop-opacity="{halo_alpha * 0.45:.3f}"/>
      <stop offset="1" stop-color="{halo}" stop-opacity="0"/>
    </radialGradient>
    <linearGradient id="beam" x1="0" x2="1" y1="0" y2="0">
      <stop offset="0" stop-color="#fff" stop-opacity="0"/>
      <stop offset="0.5" stop-color="#fff" stop-opacity="1"/>
      <stop offset="1" stop-color="#fff" stop-opacity="0"/>
    </linearGradient>
    <mask id="pass" maskUnits="userSpaceOnUse" x="0" y="0" width="{WIDTH}" height="{HEIGHT}">
      <rect class="band" x="0" y="0" width="90" height="{HEIGHT}" fill="url(#beam)" transform="skewX(-14)"/>
    </mask>
    {glow_filter}
  </defs>
  <circle class="halo" cx="{cx:g}" cy="{cy:g}" r="56" fill="url(#halo-fill)"/>
  <g{group_filter}>
    {letters}
    <g class="drip" fill="{core[0] if glow else core[1]}" filter="url(#dropglow)">{drip}</g>
    <g class="breath" filter="url(#dropglow)"><g class="drop">{''.join(drop)}{highlight}</g></g>
  </g>
  {sparkles}
  <g fill="#fff" mask="url(#pass)" opacity=".85">{''.join(shines)}</g>
</svg>
'''


DOCS.mkdir(exist_ok=True)
DARK = {0: '#b45309', 2: '#fdba74', 3: '#fb8a6b', 4: '#818cf8', 5: '#60a5fa'}
LIGHT = {0: '#b45309', 2: '#c2410c', 3: '#dc2626', 4: '#4338ca', 5: '#1d4ed8'}
(DOCS / 'logo-dark.svg').write_text(
    svg(DARK, 0.8, '#67e8f9', 0.45, 2.2, ['#cffafe', '#67e8f9', '#22d3ee'], feet='#f1f5f9'),
    encoding='utf-8', newline='\n')
(DOCS / 'logo-light.svg').write_text(
    svg(LIGHT, 0.8, '#22d3ee', 0.5, 0, ['#a5f3fc', '#22d3ee', '#0891b2'], feet='#94a3b8'),
    encoding='utf-8', newline='\n')


def uri(path: Path) -> str:
    return 'data:image/svg+xml;base64,' + base64.b64encode(path.read_bytes()).decode()


(ROOT / 'zam/target').mkdir(parents=True, exist_ok=True)
(ROOT / 'zam/target/logo-preview.html').write_text(f'''<!doctype html><meta charset="utf-8"><title>Zamak logo</title>
<style>body{{margin:0;font-family:system-ui,sans-serif}}section{{padding:36px;display:flex;flex-direction:column;align-items:center;gap:10px}}
.dark{{background:#0d1117;color:#c9d1d9}}.light{{background:#fff;color:#1f2328}}img{{width:{WIDTH}px;max-width:92vw}}</style>
<section class="dark"><img src="{uri(DOCS / 'logo-dark.svg')}"><small>GitHub 深色主题</small></section>
<section class="light"><img src="{uri(DOCS / 'logo-light.svg')}"><small>GitHub 浅色主题</small></section>''', encoding='utf-8')
print('ok', WIDTH, HEIGHT, (DOCS / 'logo-dark.svg').stat().st_size)


# ---- 方形图标：与宣传片里的小 Ƶ 相同——紫色夜空底、渐变身体、深色眼睛、白色小脚、发光的光滴。----
# 16×16 网格。身体每格按宣传片吉祥物的对角渐变取色（x 0→9 格、y 0→11 格），光滴中心亮、边缘青，
# 左上两颗白色高光像素，与标题 logo 的光滴一致。
ICON_Z = MASCOT
ICON_DROP = ['00100', '01110', '01110', '11111', '11111', '01110']
ICON_LEFT, ICON_TOP = 1.6, 3.0
ICON_DROP_AT, ICON_DROP_UNIT = (11.0, 1.2), 0.62
ICON_HIGHLIGHT = [(2, 1, 0.96), (3, 1, 0.67)]        # (行, 列, 不透明度)
ICON_STARS = [(2.0, 1.6, 0.13, '#67e8f9'), (6.4, 1.2, 0.1, '#fde68a'), (14.2, 8.5, 0.12, '#f97362'),
              (13.4, 14.2, 0.1, '#67e8f9'), (1.0, 13.0, 0.1, '#fde68a')]
SKY = ('#5b46a8', '#26235c')                          # 夜空：中心偏上的紫色 → 四周
BODY_STOPS = [(0, '#f59e0b'), (0.4, '#f97362'), (0.75, '#6366f1'), (1, '#06b6d4')]
DROP_CORE = ['#cffafe', '#67e8f9', '#22d3ee']


def rgb(hex_color):
    return tuple(int(hex_color[i:i + 2], 16) for i in (1, 3, 5))


def mix(stops, t):
    t = min(1, max(0, t))
    for (a, ca), (b, cb) in zip(stops, stops[1:]):
        if t <= b:
            f = (t - a) / (b - a)
            return tuple(round(x + (y - x) * f) for x, y in zip(rgb(ca), rgb(cb)))
    return rgb(stops[-1][1])


def body_color(r, c):
    return mix(BODY_STOPS, ((c + 0.5) * 9 + (r + 0.5) * 11) / (9 * 9 + 11 * 11))


def drop_color(r, c):
    import math
    dist = min(1, math.hypot(c - 2.2, r - 3.6) / 2.6)
    return mix([(0, DROP_CORE[0]), (0.45, DROP_CORE[1]), (1, DROP_CORE[2])], dist)


def hexed(color):
    return '#%02x%02x%02x' % color


def icon() -> str:
    body = ''.join(
        f'<rect x="{ICON_LEFT + c + 0.05:.2f}" y="{ICON_TOP + r + 0.05:.2f}" width="0.9" height="0.9" rx="0.18" fill="{hexed(body_color(r, c))}"/>'
        for r, row in enumerate(ICON_Z) for c, bit in enumerate(row) if bit == '1')
    eyes = ''.join(f'<rect x="{ICON_LEFT + c + 0.14:.2f}" y="{ICON_TOP + 0.16:.2f}" width="0.72" height="1.68" rx="0.22"/>' for c in (2, 6))
    feet = ''.join(f'<rect x="{ICON_LEFT + x:.2f}" y="{ICON_TOP + 11.05:.2f}" width="1.6" height="0.9" rx="0.3"/>' for x in (1.2, 6.2))
    dx, dy = ICON_DROP_AT
    u = ICON_DROP_UNIT
    drop = ''.join(
        f'<rect x="{dx + c * u + 0.03:.3f}" y="{dy + r * u + 0.03:.3f}" width="{u - 0.06:.2f}" height="{u - 0.06:.2f}" rx="0.12" fill="{hexed(drop_color(r, c))}"/>'
        for r, row in enumerate(ICON_DROP) for c, bit in enumerate(row) if bit == '1')
    shine = ''.join(
        f'<rect x="{dx + c * u + 0.03:.3f}" y="{dy + r * u + 0.03:.3f}" width="{u - 0.06:.2f}" height="{u - 0.06:.2f}" rx="0.12" fill-opacity="{o}"/>'
        for r, c, o in ICON_HIGHLIGHT)
    stars = ''.join(f'<circle cx="{x}" cy="{y}" r="{r}" fill="{c}" fill-opacity="0.8"/>' for x, y, r, c in ICON_STARS)
    hx, hy = dx + 2.5 * u, dy + 3.4 * u
    return f"""<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" width="256" height="256" role="img" aria-label="Zamak">
  <title>Zamak</title>
  <defs>
    <radialGradient id="sky" gradientUnits="userSpaceOnUse" cx="7.2" cy="6.7" r="12">
      <stop offset="0" stop-color="{SKY[0]}"/><stop offset="1" stop-color="{SKY[1]}"/>
    </radialGradient>
    <radialGradient id="halo"><stop offset="0" stop-color="#67e8f9" stop-opacity="0.45"/><stop offset="1" stop-color="#67e8f9" stop-opacity="0"/></radialGradient>
    <filter id="soft" x="-60%" y="-60%" width="220%" height="220%">
      <feGaussianBlur stdDeviation="0.35" result="b"/>
      <feMerge><feMergeNode in="b"/><feMergeNode in="SourceGraphic"/></feMerge>
    </filter>
  </defs>
  <rect width="16" height="16" rx="3.4" fill="url(#sky)"/>
  {stars}
  {body}
  <g fill="#0b1020">{eyes}</g>
  <g fill="#f1f5f9">{feet}</g>
  <circle cx="{hx:.2f}" cy="{hy:.2f}" r="3.2" fill="url(#halo)"/>
  <g filter="url(#soft)">{drop}</g>
  <g fill="#fff">{shine}</g>
</svg>
"""


(DOCS / 'icon.svg').write_text(icon(), encoding='utf-8', newline='\n')
print('icon ok')


# ---- 图标的 PNG 与 ICO：按 8 倍超采样画，再缩到目标尺寸（需要 Pillow；没有时跳过）。----
def icon_png(size: int):
    import math
    from PIL import Image, ImageDraw, ImageFilter
    big = size * 8
    k = big / 16
    sky = Image.new('RGB', (big, big))
    px = sky.load()
    cx, cy, a, b = 7.2 * k, 6.7 * k, rgb(SKY[0]), rgb(SKY[1])
    for y in range(big):
        for x in range(big):
            d = min(1, math.hypot(x - cx, y - cy) / (12 * k))
            px[x, y] = tuple(round(a[i] + (b[i] - a[i]) * d) for i in range(3))
    mask = Image.new('L', (big, big), 0)
    ImageDraw.Draw(mask).rounded_rectangle([0, 0, big - 1, big - 1], radius=round(3.4 * k), fill=255)
    out = Image.new('RGBA', (big, big), (0, 0, 0, 0))
    out.paste(sky, (0, 0), mask)
    draw = ImageDraw.Draw(out)
    for x, y, r, c in ICON_STARS:
        draw.ellipse([(x - r) * k, (y - r) * k, (x + r) * k, (y + r) * k], fill=rgb(c) + (204,))
    for r, row in enumerate(ICON_Z):
        for c, bit in enumerate(row):
            if bit == '1':
                x0, y0 = (ICON_LEFT + c + 0.05) * k, (ICON_TOP + r + 0.05) * k
                draw.rounded_rectangle([x0, y0, x0 + 0.9 * k, y0 + 0.9 * k], radius=0.18 * k, fill=body_color(r, c) + (255,))
    for c in (2, 6):
        x, y = ICON_LEFT + c + 0.14, ICON_TOP + 0.16
        draw.rounded_rectangle([x * k, y * k, (x + 0.72) * k, (y + 1.68) * k], radius=0.22 * k, fill=(0x0b, 0x10, 0x20, 255))
    for fx in (1.2, 6.2):
        x, y = ICON_LEFT + fx, ICON_TOP + 11.05
        draw.rounded_rectangle([x * k, y * k, (x + 1.6) * k, (y + 0.9) * k], radius=0.3 * k, fill=(0xf1, 0xf5, 0xf9, 255))
    dx, dy = ICON_DROP_AT
    u = ICON_DROP_UNIT
    halo = Image.new('RGBA', (big, big), (0, 0, 0, 0))
    hx, hy = (dx + 2.5 * u) * k, (dy + 3.4 * u) * k
    ImageDraw.Draw(halo).ellipse([hx - 2.4 * k, hy - 2.4 * k, hx + 2.4 * k, hy + 2.4 * k], fill=(0x67, 0xe8, 0xf9, 110))
    out.alpha_composite(halo.filter(ImageFilter.GaussianBlur(1.0 * k)))
    drop = Image.new('RGBA', (big, big), (0, 0, 0, 0))
    dd = ImageDraw.Draw(drop)
    cell = lambda r, c: [(dx + c * u + 0.03) * k, (dy + r * u + 0.03) * k, (dx + c * u + u - 0.03) * k, (dy + r * u + u - 0.03) * k]
    for r, row in enumerate(ICON_DROP):
        for c, bit in enumerate(row):
            if bit == '1':
                dd.rounded_rectangle(cell(r, c), radius=0.12 * k, fill=drop_color(r, c) + (255,))
    out.alpha_composite(drop.filter(ImageFilter.GaussianBlur(0.35 * k)))
    out.alpha_composite(drop)
    shine = ImageDraw.Draw(out)
    for r, c, o in ICON_HIGHLIGHT:
        shine.rounded_rectangle(cell(r, c), radius=0.12 * k, fill=(255, 255, 255, round(255 * o)))
    return out.resize((size, size), Image.LANCZOS)


try:
    icon_png(128).save(ROOT / 'editors/vscode/icon.png')
    icon_png(32).save(DOCS / 'icon-32.png')
    icon_png(512).save(DOCS / 'icon-512.png')
    # Windows 图标（zam.exe 与安装包用）：各尺寸单独绘制，小尺寸也清楚。
    sizes = [16, 24, 32, 48, 64, 128, 256]
    images = {n: icon_png(n) for n in sizes}
    images[256].save(ROOT / 'ico/Zamak_256x256.png')
    images[256].save(ROOT / 'ico/Zamak_256x256.ico', sizes=[(n, n) for n in sizes],
                     append_images=[images[n] for n in sizes[:-1]])
    print('png ok')
except ImportError:
    print('Pillow 未安装：跳过 PNG 导出（pip install pillow）')
