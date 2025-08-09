import os
import pexpect
import re
from .conftest import EVI_BIN
from .test_motion_commands import get_screen_and_cursor

def _parse_screen(screen: str) -> dict[int, str]:
    rows: dict[int, str] = {}
    for row, _col, text in re.findall(r"\x1b\[(\d+);(\d+)H([^\x1b]*)", screen):
        if text:
            rows[int(row)] = text
    return rows

def test_readme_header_visible():
    # Open README.md and assert that the very first visible line starts with '# evi'
    path = 'README.md'
    assert os.path.exists(path), 'README.md must exist in repo root'
    env = os.environ.copy()
    env.setdefault('TERM', 'xterm')
    child = pexpect.spawn(EVI_BIN, [path], env=env, encoding='utf-8')
    child.delaybeforesend = 0.0
    child.setwinsize(24, 80)

    screen, _ = get_screen_and_cursor(child)
    lines = _parse_screen(screen)
    top = lines.get(1, '')
    # Expect markdown header on first line
    assert top.startswith('# evi'), f'Top line missing header: {top!r}'

    child.send(':q!\r')
    child.expect(pexpect.EOF)
