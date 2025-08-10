import os, re, pexpect
from .conftest import EVI_BIN

ROW_RE = re.compile(r"\x1b\[(\d+);(\d+)H([^\x1b]*)")
STATUS_RE = re.compile(r'line (\d+) of')

def parse_rows(screen: str):
    rows = {}
    for row, col, text in ROW_RE.findall(screen):
        if text:
            rows[int(row)] = text
    return rows

def dump(child):
    child.send('\x07')  # Ctrl-G status dump
    child.expect(STATUS_RE)
    raw = child.before + child.after
    try:
        raw += child.read_nonblocking(size=2048, timeout=0.05)
    except Exception:
        pass
    m = STATUS_RE.search(raw)
    line_num = int(m.group(1)) if m else -1
    rows = parse_rows(raw.split('\x1b[2J')[-1])
    top_line = rows.get(1, '')
    return line_num, top_line, raw

def test_no_scroll_at_eof_after_32j_then_j():
    readme_path = os.path.abspath('README.md')
    env = os.environ.copy(); env.setdefault('TERM','xterm')
    child = pexpect.spawn(EVI_BIN, [readme_path], env=env, encoding='utf-8')
    child.delaybeforesend = 0.0
    child.setwinsize(10, 60)

    child.send('32j')
    ln1, top1, _ = dump(child)

    child.send('j')  # should not move nor scroll
    ln2, top2, _ = dump(child)

    assert ln2 == ln1, (ln1, ln2)
    assert top2 == top1, f"Top changed: before={top1!r} after={top2!r}"

    child.send(':q!\r')
    child.expect(pexpect.EOF)
