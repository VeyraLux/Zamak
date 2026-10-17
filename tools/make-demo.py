"""用法：python tools/make-demo.py，生成 docs/demo.svg。

README 里的语法短片（10 秒一轮，循环播放）：小 Ƶ 亲手把一段 Python 改成 Zamak，然后运行出结果。
与宣传片「好读」镜头的剧情相同：一脚踢飞 def、fn 从天而降、类型一个个落进来、一屁股坐扁 return，
print 长出 ln，文件名从 area.py 变成 area.zm，最后 `zam run` 打印 13.5。

GitHub 以 <img> 显示 SVG：只有 CSS 动画，不能跑脚本，也加载不了网络字体。所以每个会动的元素都把
「时间 → 位移、旋转、缩放、透明度」在 Python 里算好，按 0.04 秒一帧采样写成 @keyframes；
代码用系统等宽字体，并用 textLength 把每段文字的宽度钉死在「字符数 × 字宽」上，换哪种等宽字体都对得齐。"""
import math
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LOOP = 10.0                  # 一轮的秒数
STEP = 0.04                  # 关键帧采样间隔
FONT = 20                    # 代码字号
CH = 12.0                    # 等宽字宽 = 0.6 × 字号
X0, LINE = 96, 32            # 代码左边距、行高
WIN_X, WIN_Y, WIN_W, WIN_H = 20, 70, 680, 300
W, H = 720, 390
MONO = 'ui-monospace, SFMono-Regular, Menlo, Consolas, &quot;DejaVu Sans Mono&quot;, monospace'
Y1, Y2, Y4, Y5 = WIN_Y + 70, WIN_Y + 70 + LINE, WIN_Y + 70 + 3 * LINE, WIN_Y + 70 + 4 * LINE
TERM_Y = WIN_Y + WIN_H - 34

# 颜色与宣传片一致。
KW, FN, TYPE, TEXT, DIM, OK = '#60a5fa', '#f472b6', '#34d399', '#e2e8f0', '#94a3b8', '#a7f3d0'
MASCOT = ['111111111', '111111111', '000000111', '000001110', '000011100', '111111111',
          '001110000', '011100000', '111000000', '111111111', '111111111']
U = 4.0                      # 小 Ƶ 一格的边长


def clamp(v, lo=0.0, hi=1.0):
    return max(lo, min(hi, v))


def seg(t, a, b):
    return clamp((t - a) / (b - a))


def ease_out(p):
    return 1 - (1 - p) ** 3


def ease_in(p):
    return p ** 3


def ease_io(p):
    return 4 * p ** 3 if p < 0.5 else 1 - (-2 * p + 2) ** 3 / 2


def back(p):
    c = 1.70158
    return 1 + (c + 1) * (p - 1) ** 3 + c * (p - 1) ** 2


def lerp(a, b, p):
    return a + (b - a) * p


def squash(t, at, amount=0.3, dur=0.4):
    """落地挤压：at 时刻压扁，随后弹回。返回 (sx, sy)。"""
    p = seg(t, at, at + dur)
    if t < at or p >= 1:
        return 1.0, 1.0
    k = math.sin(p * math.pi * 2.5) * (1 - p) * amount
    return 1 + k, 1 - k


# 整体：0.0～0.4 淡入，9.3～9.9 淡出，下一轮从头开始。
def scene_alpha(t):
    return seg(t, 0, 0.4) * (1 - seg(t, 9.3, 9.9))


# ---- 第一行：def area(w, h): → fn area(w: f64, h: f64) -> f64: ----
T_FN, T_T1, T_T2, T_RET = 1.8, 2.5, 3.0, 3.5     # 各次变化的时刻（都在拍点附近）


def shift(t, at):
    return ease_out(seg(t, at, at + 0.15))


def col_area(t):        # " area(w" 的起始列：def 3 列 → fn 2 列
    return 3 - shift(t, T_FN)


def col_after_w(t):     # ", h" 之前：" area(w" 占 7 列，再加上落进来的 ": f64"（5 列）
    return col_area(t) + 7 + 5 * shift(t, T_T1)


def col_close(t):
    return col_after_w(t) + 3 + 5 * shift(t, T_T2)


def col_colon(t):
    return col_close(t) + 1 + 7 * shift(t, T_RET)


def dropping(t, at):
    """新出现的一段：从上方落下，at 时刻落定。"""
    p = seg(t, at - 0.25, at)
    return (lerp(-90, 0, ease_in(p)), 1.0 if t >= at - 0.25 else 0.0)


def def_kick(t):
    """被小 Ƶ 一脚踢飞：转着往右上方飞出窗口。"""
    p = seg(t, 1.5, 2.2)
    return dict(dx=p * 520, dy=-math.sin(p * math.pi) * 160 - p * 120, rot=p * 540,
                op=1 - seg(t, 2.0, 2.2))


def def_pop(t):
    """第四行的 def 同时缩小消失。"""
    p = seg(t, 1.6, 1.8)
    return dict(sx=1 - p, sy=1 - p, op=1 - p)


def fn_drop(t):
    p = seg(t, T_FN - 0.3, T_FN)
    sx, sy = squash(t, T_FN, 0.25, 0.4)
    return dict(dy=lerp(-110, 0, ease_in(p)), sx=sx, sy=sy, op=1 if t >= T_FN - 0.3 else 0)


# ---- 第二行：return 被坐扁，w * h 往左挪 ----
def return_kw(t):
    p = seg(t, 4.05, 4.3)
    return dict(sx=lerp(1, 1.3, p), sy=lerp(1, 0.05, p), op=1 - seg(t, 4.25, 4.45))


def body_col(t):
    return 11 - 7 * ease_out(seg(t, 4.45, 4.75))


# ---- 第五行：print 长出 ln ----
T_LN = 5.0


def ln_pop(t):
    p = back(seg(t, T_LN, T_LN + 0.25))
    return dict(sx=p, sy=p, op=1 if t >= T_LN else 0)


def call_col(t):
    return 9 + 2 * ease_out(seg(t, T_LN, T_LN + 0.2))


# ---- 小 Ƶ 的路线 ----
def mascot(t):
    top = WIN_Y                      # 窗口上沿
    x, y, rot, sx, sy, happy = 640.0, float(top), 0.0, 1.0, 1.0, 0.0
    if t >= 0.6:                     # 跳到第一行 def 左边
        p = seg(t, 0.6, 1.2)
        x = lerp(640, 70, ease_io(p))
        y = lerp(top, Y1 + 6, ease_in(p)) - math.sin(p * math.pi) * 70
        sx, sy = squash(t, 1.2, 0.3, 0.35)
    if t >= 1.35:                    # 往后蓄力，1.5 秒一脚踢出去，再回正
        rot = (15 * ease_out(seg(t, 1.35, 1.5)) - 40 * ease_out(seg(t, 1.5, 1.6))) * (1 - ease_io(seg(t, 1.6, 1.9)))
    if t >= 2.0:                     # 跳回窗口上沿看热闹，踩着拍子点头
        p = seg(t, 2.0, 2.4)
        x = lerp(70, 330, ease_io(p))
        y = lerp(Y1 + 6, top, ease_out(p)) - math.sin(p * math.pi) * 60
        sx, sy = squash(t, 2.4, 0.25, 0.35)
        if t >= 2.4:
            b = (t - 2.4) % 0.5 / 0.5
            sy *= 1 - 0.08 * math.sin(b * math.pi)
            sx *= 1 + 0.08 * math.sin(b * math.pi)
    if t >= 3.7:                     # 跳到 return 上，一屁股坐下
        p = seg(t, 3.7, 4.05)
        x = lerp(330, X0 + 4 * CH + 36, ease_io(p))
        y = lerp(top, Y2 + 6, ease_in(p)) - math.sin(p * math.pi) * 80
        sx, sy = squash(t, 4.05, 0.35, 0.45)
    if t >= 4.8:                     # 跳到窗口右下角，等结果
        p = seg(t, 4.8, 5.3)
        x = lerp(X0 + 4 * CH + 36, 640, ease_io(p))
        y = lerp(Y2 + 6, WIN_Y + WIN_H, ease_in(p)) - math.sin(p * math.pi) * 110
        sx, sy = squash(t, 5.3, 0.3, 0.4)
    if t >= 6.2:                     # 13.5 出来了：开心地连跳两下
        happy = 1.0
        for a in (6.2, 6.7):
            p = seg(t, a, a + 0.45)
            if 0 < p < 1:
                y = WIN_Y + WIN_H - math.sin(p * math.pi) * 40
        sx, sy = squash(t, 6.65, 0.25, 0.3) if t < 7.15 else squash(t, 7.15, 0.25, 0.3)
    return dict(dx=x, dy=y, rot=rot, sx=sx, sy=sy, happy=happy)


# ---- 关键帧 ----
def keyframes(name, fn):
    """fn(t) → dict(dx, dy, rot, sx, sy, op)。只写出与上一帧不同的帧（加上前一帧，保证线性插值正确）。"""
    frames, last, last_t, prev_t = [], None, None, None
    n = int(round(LOOP / STEP))
    for k in range(n + 1):
        t = k * STEP
        v = fn(t)
        tr = (f'translate({v.get("dx", 0):.1f}px,{v.get("dy", 0):.1f}px) rotate({v.get("rot", 0):.1f}deg) '
              f'scale({v.get("sx", 1):.3f},{v.get("sy", 1):.3f})')
        css = f'transform:{tr};opacity:{clamp(v.get("op", 1)) * scene_alpha(t):.2f}'
        if css != last:
            # 停了一段之后才变：先在变之前那一帧把旧值再写一遍，否则会从很早以前慢慢插值过来。
            if prev_t is not None and prev_t != last_t:
                frames.append(f'{prev_t / LOOP * 100:.2f}%{{{last}}}')
            frames.append(f'{t / LOOP * 100:.2f}%{{{css}}}')
            last, last_t = css, t
        prev_t = t
    if not frames[-1].startswith('100.00%'):
        frames.append(f'100.00%{{{last}}}')
    return f'@keyframes {name}{{{"".join(frames)}}}'


ANIMS = []


def anim(name, fn, origin='50% 50%'):
    ANIMS.append((name, fn, origin))
    return f'class="{name}"'


def text(name, fn, x, y, s, fill, origin='50% 50%'):
    return (f'<text {anim(name, fn, origin)} x="{x:.1f}" y="{y}" fill="{fill}" textLength="{len(s) * CH:.1f}" '
            f'lengthAdjust="spacingAndGlyphs" xml:space="preserve">{s.replace("<", "&lt;").replace(">", "&gt;")}</text>')


def at_col(col_fn, extra=None):
    """位置随时间变化的一段：col_fn(t) 是起始列。"""
    def fn(t):
        v = dict(dx=col_fn(t) * CH)
        if extra:
            v.update(extra(t))
        return v
    return fn


def drop_at(col_fn, at):
    def fn(t):
        dy, op = dropping(t, at)
        return dict(dx=col_fn(t) * CH, dy=dy, op=op)
    return fn


def build():
    parts = []
    # 第一行
    parts.append(text('d1', lambda t: def_kick(t), X0, Y1, 'def', KW))
    parts.append(text('f1', lambda t: fn_drop(t), X0, Y1, 'fn', FN, '50% 100%'))
    parts.append(text('a1', at_col(col_area), X0, Y1, ' area(w', TEXT))
    parts.append(text('t1', drop_at(lambda t: col_area(t) + 7, T_T1), X0, Y1, ': f64', TYPE))
    parts.append(text('c1', at_col(col_after_w), X0, Y1, ', h', TEXT))
    parts.append(text('t2', drop_at(lambda t: col_after_w(t) + 3, T_T2), X0, Y1, ': f64', TYPE))
    parts.append(text('p1', at_col(col_close), X0, Y1, ')', TEXT))
    parts.append(text('r1', drop_at(lambda t: col_close(t) + 1, T_RET), X0, Y1, ' -> f64', TYPE))
    parts.append(text('o1', at_col(col_colon), X0, Y1, ':', TEXT))
    # 第二行
    parts.append(text('rk', lambda t: dict(dx=4 * CH, **return_kw(t)), X0, Y2, 'return', KW, '50% 100%'))
    parts.append(text('bd', at_col(body_col), X0, Y2, 'w * h', TEXT))
    # 第四行：def main(): → fn main():
    parts.append(text('d4', def_pop, X0, Y4, 'def', KW))
    parts.append(text('f4', lambda t: fn_drop(t), X0, Y4, 'fn', FN, '50% 100%'))
    parts.append(text('m4', at_col(col_area), X0, Y4, ' main():', TEXT))
    # 第五行：print(...) → println(...)
    parts.append(text('pr', lambda t: dict(dx=4 * CH), X0, Y5, 'print', KW))
    parts.append(text('ln', lambda t: dict(dx=9 * CH, **ln_pop(t)), X0, Y5, 'ln', KW, '0% 100%'))
    parts.append(text('ar', at_col(call_col), X0, Y5, '(area(3, 4.5))', TEXT))
    # 标签页上的文件名
    tab_y = WIN_Y + 24
    parts.append(text('py', lambda t: dict(op=1 - seg(t, 5.2, 5.35)), 128, tab_y, 'area.py', DIM))
    parts.append(text('zm', lambda t: dict(op=seg(t, 5.25, 5.4)), 128, tab_y, 'area.zm', '#67e8f9'))
    # 终端：敲命令、出结果
    cmd = '$ zam run area.zm'

    def typed(t):
        return dict(sx=1, op=1 if t >= 5.6 else 0)
    parts.append(f'<clipPath id="type"><rect {anim("tc", lambda t: dict(sx=max(0.001, seg(t, 5.6, 6.0))), "0% 50%")} '
                 f'x="{X0}" y="{TERM_Y - 20}" width="{len(cmd) * CH}" height="28"/></clipPath>')
    parts.append(f'<g clip-path="url(#type)">{text("cm", typed, X0, TERM_Y, cmd, DIM)}</g>')
    parts.append(text('rs', lambda t: dict(dy=lerp(8, 0, ease_out(seg(t, 6.2, 6.4))), op=seg(t, 6.2, 6.3)),
                      X0 + (len(cmd) + 3) * CH, TERM_Y, '13.5', OK))
    # 小 Ƶ
    body = []
    for r, row in enumerate(MASCOT):
        for c, bit in enumerate(row):
            if bit == '1':
                body.append(f'<rect x="{(c - 4.5) * U + 0.2:.1f}" y="{(r - 12) * U + 0.2:.1f}" width="{U * 0.9:.1f}" height="{U * 0.9:.1f}" rx="{U * 0.18:.1f}"/>')
    eyes = ''.join(f'<rect x="{(c - 4.5 + 0.14) * U:.2f}" y="{(-12 + 0.16) * U:.2f}" width="{0.72 * U:.2f}" height="{1.68 * U:.2f}" rx="{0.22 * U:.2f}"/>' for c in (2, 6))
    happy = ''.join(f'<rect x="{(x - 4.5) * U:.2f}" y="{(y - 12) * U:.2f}" width="{0.62 * U:.2f}" height="{0.62 * U:.2f}"/>'
                    for x, y in [(1.9, 1.2), (2.5, 0.6), (3.1, 1.2), (5.9, 1.2), (6.5, 0.6), (7.1, 1.2)])
    feet = ''.join(f'<rect x="{(fx - 4.5) * U:.2f}" y="{-0.95 * U:.2f}" width="{1.6 * U:.2f}" height="{0.9 * U:.2f}" rx="{0.3 * U:.2f}"/>' for fx in (1.2, 6.2))

    def mascot_fn(t):
        m = mascot(t)
        return dict(dx=m['dx'], dy=m['dy'], rot=m['rot'], sx=m['sx'], sy=m['sy'], op=seg(t, 0.25, 0.45))
    zam = (f'<g {anim("zm0", mascot_fn, "50% 100%")}>'
           f'<g fill="#f1f5f9">{feet}</g><g fill="url(#zink)">{"".join(body)}</g>'
           f'<g fill="#0b1020" {anim("ey", lambda t: dict(op=1 - mascot(t)["happy"]))}>{eyes}</g>'
           f'<g fill="#0b1020" {anim("hp", lambda t: dict(op=mascot(t)["happy"]))}>{happy}</g></g>')
    # 结果出来时的小星星
    stars = ''.join(
        f'<rect {anim(f"s{k}", lambda t, k=k, dx=dx, dy=dy: dict(dx=dx * ease_out(seg(t, 6.2, 6.8)), dy=dy * ease_out(seg(t, 6.2, 6.8)), rot=seg(t, 6.2, 6.8) * 180, op=math.sin(seg(t, 6.2, 6.9) * math.pi)))} '
        f'x="634" y="{WIN_Y + WIN_H - 30}" width="8" height="8" rx="2" fill="{"#fde68a" if k % 2 else "#67e8f9"}"/>'
        for k, (dx, dy) in enumerate([(-50, -40), (40, -50), (-30, -70), (55, -10), (-60, 0), (10, -80)]))

    css = '\n'.join(keyframes(f'k{name}', fn) for name, fn, _ in ANIMS)
    rules = '\n'.join(f'.{name}{{transform-box:fill-box;transform-origin:{origin};animation:k{name} {LOOP}s linear infinite}}'
                      for name, _, origin in ANIMS)
    # 鼠标没法控制，但用户可以要求「减少动画」：那就停在改好、跑出结果的那一刻。
    still = f'animation-delay:-7s;animation-play-state:paused'
    dots = ''.join(f'<circle cx="{WIN_X + 22 + i * 20}" cy="{WIN_Y + 18}" r="6" fill="{c}"/>' for i, c in enumerate(['#f87171', '#fbbf24', '#34d399']))
    return f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}" width="{W}" height="{H}" role="img" aria-label="The Zamak mascot turns a Python function into Zamak, then runs it and prints 13.5">
  <title>From Python to Zamak</title>
  <style>
    text {{ font-family: {MONO}; font-size: {FONT}px; font-weight: 600; }}
{rules}
{css}
    @media (prefers-reduced-motion: reduce) {{ [class] {{ {still} }} }}
  </style>
  <defs>
    <linearGradient id="zink" gradientUnits="userSpaceOnUse" x1="{-4.5 * U}" y1="{-12 * U}" x2="{4.5 * U}" y2="{-1 * U}">
      <stop offset="0" stop-color="#f59e0b"/><stop offset="0.4" stop-color="#f97362"/><stop offset="0.75" stop-color="#6366f1"/><stop offset="1" stop-color="#06b6d4"/>
    </linearGradient>
  </defs>
  <rect x="{WIN_X}" y="{WIN_Y}" width="{WIN_W}" height="{WIN_H}" rx="18" fill="#0b1020" stroke="#334155" stroke-width="2"/>
  {dots}
  <line x1="{WIN_X}" x2="{WIN_X + WIN_W}" y1="{TERM_Y - 34}" y2="{TERM_Y - 34}" stroke="#1e293b" stroke-width="2"/>
  {''.join(parts)}
  {stars}
  {zam}
</svg>
'''


out = ROOT / 'docs' / 'demo.svg'
out.write_text(build(), encoding='utf-8', newline='\n')
print('ok', out.stat().st_size)
