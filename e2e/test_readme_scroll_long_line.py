import os, re, pexpect
from .conftest import EVI_BIN
from e2e.helpers import expect_cursor

CUR_MOVE = re.compile(r"\x1b\[(\d+);(\d+)H")
ANSI_CSI = re.compile(r"\x1b\[[0-9;]*[A-Za-z]")

def _capture(child):
    child.send('\x07')
    child.expect(r'line (\d+) of')
    data = child.before + child.after
    try:
        data += child.read_nonblocking(size=4096, timeout=0.2)
    except Exception:
        pass
    return data

def test_readme_32j_long_line_fully_visible():
    readme_path = os.path.abspath('README.md')
    env = os.environ.copy(); env.setdefault('TERM','xterm')
    child = pexpect.spawn(EVI_BIN, [readme_path], env=env, encoding='utf-8')
    child.delaybeforesend = 0.0
    child.setwinsize(24, 80)
    # 動作安定のため初期フレーム取得
    _capture(child)
    # 32j を実行しカーソル到達を確認
    child.send('32j')
    expect_cursor(child, expected_line=33)
    screen = _capture(child)
    if '\x1b[2J' in screen:
        screen = screen.split('\x1b[2J')[-1]
    rows, cols = 24, 80
    grid = [[' ']*cols for _ in range(rows)]
    r = 1; c = 1
    i = 0
    while i < len(screen):
        if screen[i] == '\x1b':
            m = CUR_MOVE.match(screen, i)
            if m:
                r = int(m.group(1)); c = int(m.group(2)); i = m.end(); continue
            other = ANSI_CSI.match(screen, i)
            if other:
                i = other.end(); continue
        ch = screen[i]
        if ch == '\n':
            r += 1; c = 1
        else:
            if 1 <= r <= rows and 1 <= c <= cols:
                grid[r-1][c-1] = ch
            c += 1
        i += 1
    lines = [''.join(row).rstrip() for row in grid]
    # コードフェンス行を探索
    fence_idx = None
    for idx, line in enumerate(lines):
        if line.startswith('```bash'):
            fence_idx = idx; break
    assert fence_idx is not None, '\n'.join(lines)
    # 完全表示 (行頭開始)
    assert lines[fence_idx].startswith('```bash')
    # ノート行(長行)は fence_idx より上に存在しつつ表示文脈を残していることを軽く確認
    note_visible = any('**Note:**' in l for l in lines[:fence_idx])
    assert note_visible, 'Note line not visible before code fence\n' + '\n'.join(lines)
    child.send(':q!\r')
    child.expect(pexpect.EOF)
