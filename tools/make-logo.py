"""用法：python tools/make-logo.py，生成 docs/logo-dark.svg、docs/logo-light.svg 与预览页 zam/target/logo-preview.html。

Zamak 动态 logo「光雨汇成代码」：7×11 像素点阵。光点像雨一样落下、弹跳着落进格子拼出
`> ZAMAK`，其中的 Ƶ 就是吉祥物小 Ƶ（与宣传片结尾一致）；最后一颗光滴弹进来并呼吸发光；中途一道光扫过（构建完成）。
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


# 打开页面时只演一次：光点弹跳着落下就位，之后保持不动；扫光每隔几秒扫一次，光滴持续呼吸。
FALL = 1.0                   # 一个光点从落下到停稳的秒数
HOP_T = 1.3                  # 光滴的弹跳更高、更久
DRIP_T = 0.75                # 小光滴从小 Ƶ 右上角滴到右下角的秒数
DRIP_FALL = 9.6 * 8.5        # 小光滴落下的距离：从顶上一横到底下一横
SWEEP = 5.0                  # 扫光的间隔


def pct(seconds: float, total: float) -> str:
    return f'{seconds / total * 100:.2f}%'


# 弹跳：加速落下 → 落地压扁 → 弹起拉长 → 再落 → 小弹一下 → 停稳。
BOUNCE = f'''@keyframes bounce {{
      0% {{ transform: translateY(-58px) scale(1, 1); opacity: 0; animation-timing-function: cubic-bezier(.55, 0, 1, .45); }}
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
    letters, drop, shines, eyes, lids = [], [], [], [], []
    delays = {}
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
        # 按字符、按列依次落下，再加一点错位：像雨点一样不齐整，更活泼。
        jitter = ((i * 73 + col * 31 + row * 17) % 11) / 11 * 0.14
        delay = 0.10 + i * 0.24 + col * 0.05 + jitter
        delays[(i, col, row)] = delay
        opacity = f' fill-opacity="{prompt_alpha}"' if i == 0 else ''
        letters.append(f'{rect} fill="{fill}"{opacity} class="px" style="animation-delay:{delay:.3f}s"/>')
    # 眼睛：小 Ƶ 顶上那一横的第 3、7 列（从 0 数是 2、6），连起来的两格高、小一号的深色圆角长方形，
    # 与吉祥物相同；跟着这一列最上面的像素一起落下。眼皮是同色同形的一层，落定后时不时眨眼。
    for c in (2, 6):
        eye = (f'<rect x="{ZX + (c + 0.14) * PITCH:.2f}" y="{PAD_Y + 0.16 * PITCH:.2f}" width="{0.72 * PITCH:.2f}" '
               f'height="{1.68 * PITCH:.2f}" rx="{0.22 * PITCH:.2f}"')
        eyes.append(f'{eye} fill="{EYE}" class="px" style="animation-delay:{delays[(1, c, 0)]:.3f}s"/>')
        lids.append(f'{eye} fill="url(#zink)" class="lid"/>')
    # 两只小脚：与吉祥物相同的位置和大小，跟着最下面一行像素一起落下。
    legs = ''.join(
        f'<rect x="{ZX + fx * PITCH:.2f}" y="{PAD_Y + 11.05 * PITCH:.2f}" width="{1.6 * PITCH:.2f}" height="{0.9 * PITCH:.2f}" '
        f'rx="{0.3 * PITCH:.2f}" fill="{feet}" class="px" style="animation-delay:{delays[(1, col, 10)]:.3f}s"/>'
        for fx, col in ((1.2, 1), (6.2, 6)))
    drip_delay = 1.90                       # 小 Ƶ 早已落定（约 1.6 秒）
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
      .px, .drop, .breath, .band, .spark, .lid {{ animation: none; }}
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
    {''.join(letters)}
    {''.join(eyes)}
    {legs}
    {''.join(lids)}
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


# ---- 方形图标：渐变底 + 白色像素 Ƶ + 一小滴光（16×16 网格，缩到 16px 时一格正好一个屏幕像素）。----
ICON_Z = MASCOT
ICON_DROP = ['00100', '01110', '01110', '11111', '11111', '01110']
# 眼睛占 Ƶ 顶上两行的第 3、7 列（从 0 数是 2、6），比格子略窄，四角稍圆。
ICON_TOP = 3.2   # Ƶ 的上沿；比原来高 0.6 格，给脚留出位置
ICON_EYES = [(1.2 + c + 0.14, ICON_TOP + 0.16, 0.72, 1.68) for c in (2, 6)]
# 两只小脚：与吉祥物相同，在第 1.2、6.2 格处，宽 1.6、高 0.7。
ICON_FEET = [(1.2 + x, ICON_TOP + 11 + 0.14, 1.6, 0.7) for x in (1.2, 6.2)]


def icon() -> str:
    def cells(bitmap, x0, y0, unit, inset, radius):
        return ''.join(
            f'<rect x="{x0 + c * unit + inset:.3f}" y="{y0 + r * unit + inset:.3f}" width="{unit - 2 * inset:.3f}" '
            f'height="{unit - 2 * inset:.3f}" rx="{radius}"/>'
            for r, row in enumerate(bitmap) for c, bit in enumerate(row) if bit == '1')
    # 底色与标题 logo 同一条暖→冷渐变；光滴缩成 0.65 格的细像素，带一层很轻的白色柔光。
    return f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" width="256" height="256" role="img" aria-label="Zamak">
  <title>Zamak</title>
  <defs>
    <linearGradient id="tile" gradientUnits="userSpaceOnUse" x1="0" y1="0" x2="16" y2="16">
      <stop offset="0" stop-color="#f59e0b"/><stop offset="0.4" stop-color="#f97362"/>
      <stop offset="0.75" stop-color="#6366f1"/><stop offset="1" stop-color="#06b6d4"/>
    </linearGradient>
    <filter id="soft" x="-60%" y="-60%" width="220%" height="220%">
      <feGaussianBlur stdDeviation="0.35" result="b"/>
      <feMerge><feMergeNode in="b"/><feMergeNode in="SourceGraphic"/></feMerge>
    </filter>
  </defs>
  <rect width="16" height="16" rx="3.4" fill="url(#tile)"/>
  <g fill="#fff">{cells(ICON_Z, 1.2, ICON_TOP, 1, 0.06, 0.2)}</g>
  <g fill="#0b1020">{''.join(f'<rect x="{x:.2f}" y="{y:.2f}" width="{w}" height="{h}" rx="0.22"/>' for x, y, w, h in ICON_EYES)}</g>
  <g fill="#fff">{''.join(f'<rect x="{x:.2f}" y="{y:.2f}" width="{w}" height="{h}" rx="0.25"/>' for x, y, w, h in ICON_FEET)}</g>
  <g fill="#fff" filter="url(#soft)">{cells(ICON_DROP, 10.9, 1.5, 0.65, 0.04, 0.14)}</g>
</svg>
'''


(DOCS / 'icon.svg').write_text(icon(), encoding='utf-8', newline='\n')
print('icon ok')


# ---- 图标的 PNG：按 8 倍超采样画，再缩到目标尺寸（需要 Pillow；没有时跳过）。----
def icon_png(size: int):
    from PIL import Image, ImageDraw, ImageFilter
    k = size * 8 / 16                     # 每个网格单位的像素数（超采样后）
    big = size * 8
    stops = [(0, (0xf5, 0x9e, 0x0b)), (0.4, (0xf9, 0x73, 0x62)), (0.75, (0x63, 0x66, 0xf1)), (1, (0x06, 0xb6, 0xd4))]

    def color(t):
        for (a, ca), (b, cb) in zip(stops, stops[1:]):
            if t <= b:
                f = (t - a) / (b - a)
                return tuple(round(x + (y - x) * f) for x, y in zip(ca, cb))
        return stops[-1][1]
    # 底色：沿对角线的暖→冷渐变，用圆角方块做遮罩。
    tile = Image.new('RGB', (big, big))
    px = tile.load()
    for y in range(big):
        for x in range(big):
            px[x, y] = color((x + y) / (2 * big))
    mask = Image.new('L', (big, big), 0)
    ImageDraw.Draw(mask).rounded_rectangle([0, 0, big - 1, big - 1], radius=round(3.4 * k), fill=255)
    out = Image.new('RGBA', (big, big), (0, 0, 0, 0))
    out.paste(tile, (0, 0), mask)

    def squares(bitmap, x0, y0, unit, inset, radius):
        layer = Image.new('RGBA', (big, big), (0, 0, 0, 0))
        draw = ImageDraw.Draw(layer)
        for r, row in enumerate(bitmap):
            for c, bit in enumerate(row):
                if bit == '1':
                    left, top = (x0 + c * unit + inset) * k, (y0 + r * unit + inset) * k
                    side = (unit - 2 * inset) * k
                    draw.rounded_rectangle([left, top, left + side, top + side], radius=radius * k, fill=(255, 255, 255, 255))
        return layer
    out.alpha_composite(squares(ICON_Z, 1.2, ICON_TOP, 1, 0.06, 0.2))
    eyes = ImageDraw.Draw(out)
    for x, y, w, h in ICON_EYES:
        eyes.rounded_rectangle([x * k, y * k, (x + w) * k, (y + h) * k], radius=0.22 * k, fill=(0x0b, 0x10, 0x20, 255))
    for x, y, w, h in ICON_FEET:
        eyes.rounded_rectangle([x * k, y * k, (x + w) * k, (y + h) * k], radius=0.25 * k, fill=(255, 255, 255, 255))
    drop = squares(ICON_DROP, 10.9, 1.5, 0.65, 0.04, 0.14)
    out.alpha_composite(drop.filter(ImageFilter.GaussianBlur(0.35 * k)))   # 光滴的柔光
    out.alpha_composite(drop)
    return out.resize((size, size), Image.LANCZOS)


try:
    icon_png(128).save(ROOT / 'editors/vscode/icon.png')
    icon_png(32).save(DOCS / 'icon-32.png')
    icon_png(512).save(DOCS / 'icon-512.png')
    print('png ok')
except ImportError:
    print('Pillow 未安装：跳过 PNG 导出（pip install pillow）')
