import os, re
from e2e.helpers import expect_cursor, spawn_evi
from .test_motion_commands import get_screen_and_cursor

def _parse_screen(screen: str):
    lines = {}
    for idx, line in enumerate(screen.splitlines(), start=1):
        lines[idx] = line.rstrip('\n')
    return lines

ANSI_CSI = re.compile(r"\x1b\[[0-9;]*[A-Za-z]")
CUR_MOVE = re.compile(r"\x1b\[(\d+);(\d+)H")

def _capture_screen(child):
    screen, _ = get_screen_and_cursor(child, rows=10)
    return screen

def test_readme_initial_scrolling():
    readme_path = "README.md"; assert os.path.exists(readme_path)
    child = spawn_evi(readme_path, rows=10, cols=60)
    # 初期描画確定
    _capture_screen(child)
    # 3回 j
    for _ in range(3):
        child.send('j'); expect_cursor(child)
    screen = _capture_screen(child)
    # 直近のフレームのみ抽出 (最後のクリア以降)
    if '\x1b[2J' in screen:
        screen = screen.split('\x1b[2J')[-1]
    # 余分なカーソル移動シーケンスを処理しつつ単純に最終グリッドを復元
    # クリア後は (1,1) から始まるので移動シーケンスだけを反映すれば良い
    rows, cols = 10, 60
    grid = [[' ']*cols for _ in range(rows)]
    cur_r = 1; cur_c = 1
    pos_iter = list(CUR_MOVE.finditer(screen))
    idx = 0
    while idx < len(screen):
        # カーソル移動開始なら座標更新してシーケンスをスキップ
        if screen[idx] == '\x1b':
            m = CUR_MOVE.match(screen, idx)
            if m:
                cur_r = int(m.group(1)); cur_c = int(m.group(2))
                idx = m.end()
                continue
            # 他CSIはスキップ
            other = ANSI_CSI.match(screen, idx)
            if other:
                idx = other.end(); continue
        ch = screen[idx]
        if ch == '\n':
            cur_r += 1; cur_c = 1
        else:
            if 1 <= cur_r <= rows and 1 <= cur_c <= cols:
                grid[cur_r-1][cur_c-1] = ch
            cur_c += 1
        idx += 1
    top_line = ''.join(grid[0]).rstrip()
    top_line_clean = top_line.strip()
    assert top_line_clean.startswith('# evi - vi clone editor'), f"Top line changed: {top_line_clean!r}\nRawTop:{top_line!r}\nScreenRaw:\n{screen}"
    # カーソルは段落内へ進んでいるはず (status line で検証)
    # 簡易: 行2またはそれ以降
    cur = expect_cursor(child)
    assert cur[0] >= 2, f"Cursor not moved down as expected: {cur}"
