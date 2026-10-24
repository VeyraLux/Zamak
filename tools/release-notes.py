"""从 CHANGELOG.md 摘出某个版本的发布说明（英文一节，后面附折叠的中文一节），给 GitHub Release 用。

用法：python3 tools/release-notes.py 0.2.0 [上一个标签，如 v0.1.1] > notes.md

英文一节的标题是 `## 0.2.0` 或 `## 0.2.0 (2026-10-25)`，中文一节是 `## 0.2.0（中文）`。
找不到英文一节时（例如预发布 0.2.0-rc.1）给出一句指向 CHANGELOG 的话，不让发布失败。
"""
import re
import sys
from pathlib import Path

sys.stdout.reconfigure(encoding="utf-8")  # Windows 的控制台默认不是 UTF-8
version = sys.argv[1]
text = Path(__file__).resolve().parent.parent.joinpath('CHANGELOG.md').read_text(encoding='utf-8')


def section(pattern):
    match = re.search(pattern, text, re.MULTILINE)
    if not match:
        return None
    rest = text[match.end():]
    end = re.search(r'^## ', rest, re.MULTILINE)
    return (rest[:end.start()] if end else rest).strip()


escaped = re.escape(version)
english = section(rf'^## {escaped}(?: \(.*\))?[ \t]*$')
chinese = section(rf'^## {escaped}（中文）[ \t]*$')
url = 'https://github.com/VeyraLux/Zamak/blob/main/CHANGELOG.md'
if not english:
    print(f'See [CHANGELOG.md]({url}) for the changes in this version.')
    sys.exit(0)
print(english)
print()
print('### Install')
print()
print('```sh')
print('npm install -g @veyralux/zamak')
print('```')
print()
print('Without Node.js, use the install script. On Linux and macOS:')
print()
print('```sh')
print('curl -fsSL https://raw.githubusercontent.com/VeyraLux/Zamak/main/install.sh | sh')
print('```')
print()
print('On Windows (PowerShell):')
print()
print('```powershell')
print('irm https://raw.githubusercontent.com/VeyraLux/Zamak/main/install.ps1 | iex')
print('```')
print()
print('If you already have Zamak, run `zam upgrade`. Each archive below has a `.sha256` file next to it.')
if len(sys.argv) > 2:
    print()
    print(f'**Full changelog**: https://github.com/VeyraLux/Zamak/compare/{sys.argv[2]}...v{version}')
if chinese:
    print()
    print('<details>')
    print('<summary>中文</summary>')
    print()
    print(chinese)
    print()
    print('</details>')
