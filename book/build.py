"""构建 The Zamak Book：英文版在 site/，中文版在 site/zh-CN/。

用法：python book/build.py            # 需要 mdbook（cargo install mdbook --locked）
      python book/build.py --serve    # 构建后在 http://localhost:3000 预览

两版共用 book/theme/ 里的样式与脚本，logo 与图标取自 docs/，构建前复制进各版的 theme/。
"""
import argparse
import functools
import http.server
import shutil
import subprocess
from pathlib import Path

BOOK = Path(__file__).resolve().parent
DOCS = BOOK.parent / 'docs'
SITE = BOOK / 'site'
LANGUAGES = {'en': SITE, 'zh-CN': SITE / 'zh-CN'}


def prepare(language):
    theme = BOOK / language / 'theme'
    theme.mkdir(exist_ok=True)
    for name in ['zamak.css', 'zamak.js']:
        shutil.copy(BOOK / 'theme' / name, theme / name)
    for name in ['logo-dark.svg', 'logo-light.svg']:
        shutil.copy(DOCS / name, theme / name)
    shutil.copy(DOCS / 'icon.svg', theme / 'favicon.svg')
    shutil.copy(DOCS / 'icon-32.png', theme / 'favicon.png')


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--serve', action='store_true')
    args = parser.parse_args()
    if SITE.exists():
        shutil.rmtree(SITE)
    # 先建英文版：mdbook 会清空目标目录，中文版放在它里面，必须后建。
    for language, out in LANGUAGES.items():
        prepare(language)
        subprocess.run(['mdbook', 'build', str(BOOK / language), '--dest-dir', str(out)], check=True)
    print(f'site: {SITE}')
    if args.serve:
        handler = functools.partial(http.server.SimpleHTTPRequestHandler, directory=str(SITE))
        print('serving http://localhost:3000')
        http.server.ThreadingHTTPServer(('127.0.0.1', 3000), handler).serve_forever()


if __name__ == '__main__':
    main()
