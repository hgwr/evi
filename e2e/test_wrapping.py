import os
import re
import tempfile
import pexpect

from .conftest import EVI_BIN
from .test_motion_commands import get_screen_and_cursor, goto


def _parse_screen(screen: str) -> dict[int, str]:
    """Return a mapping from row number to text content."""
    matches = re.findall(r"\x1b\[(\d+);(\d+)H([^\x1b]*)", screen)
    return {int(row): text for row, _col, text in matches}


def test_long_line_wrapping():
    file_content = "{}\n{}\n{}\n".format("a" * 60, "b" * 120, "c" * 60)
    fd, path = tempfile.mkstemp()
    try:
        with os.fdopen(fd, "w") as f:
            f.write(file_content)

        env = os.environ.copy()
        env.setdefault("TERM", "xterm")
        child = pexpect.spawn(EVI_BIN, [path], env=env, encoding="utf-8")
        child.delaybeforesend = float(os.getenv("EVI_DELAY_BEFORE_SEND", "0.1"))
        child.setwinsize(24, 80)

        screen, _ = get_screen_and_cursor(child)
        lines = _parse_screen(screen)

        assert lines[1] == "a" * 60
        assert lines[2] == "b" * 80
        assert lines[3] == "b" * 40
        assert lines[4] == "c" * 60

        child.send(":q!\r")
        child.expect(pexpect.EOF)
    finally:
        os.unlink(path)


def test_cursor_j_k_on_wrapped_line():
    file_content = "{}\n{}\n{}\n".format("a" * 60, "b" * 120, "c" * 60)
    fd, path = tempfile.mkstemp()
    try:
        with os.fdopen(fd, "w") as f:
            f.write(file_content)

        env = os.environ.copy()
        env.setdefault("TERM", "xterm")
        child = pexpect.spawn(EVI_BIN, [path], env=env, encoding="utf-8")
        child.delaybeforesend = float(os.getenv("EVI_DELAY_BEFORE_SEND", "0.1"))
        child.setwinsize(24, 80)

        # Ensure the editor has finished drawing
        get_screen_and_cursor(child)
        goto(child, 1, 1)

        child.send("j")
        _, pos = get_screen_and_cursor(child)
        assert pos == (2, 1)

        child.send("j")
        _, pos = get_screen_and_cursor(child)
        assert pos == (4, 1)

        child.send("k")
        _, pos = get_screen_and_cursor(child)
        assert pos == (2, 1)

        child.send("k")
        _, pos = get_screen_and_cursor(child)
        assert pos == (1, 1)

        child.send(":q!\r")
        child.expect(pexpect.EOF)
    finally:
        os.unlink(path)


def test_full_width_lines_no_extra_blank_lines():
    file_content = "{}\n{}\n{}\n".format("A" * 80, "B" * 80, "C" * 80)
    fd, path = tempfile.mkstemp()
    try:
        with os.fdopen(fd, "w") as f:
            f.write(file_content)

        env = os.environ.copy()
        env.setdefault("TERM", "xterm")
        child = pexpect.spawn(EVI_BIN, [path], env=env, encoding="utf-8")
        child.delaybeforesend = float(os.getenv("EVI_DELAY_BEFORE_SEND", "0.1"))
        child.setwinsize(24, 80)

        screen, _ = get_screen_and_cursor(child)
        lines = _parse_screen(screen)

        assert lines[1] == "A" * 80
        assert lines[2] == "B" * 80
        assert lines[3] == "C" * 80

        child.send(":q!\r")
        child.expect(pexpect.EOF)
    finally:
        os.unlink(path)


# TODO: Add tests for '^', 'G', and other motion commands once implemented in evi


def test_G_on_long_wrapped_file_no_extra_blank_lines():
    """Cursor should land on the last line without leaving a blank line."""
    lines = [(chr(ord('a') + i)) * 90 for i in range(13)]
    file_content = "\n".join(lines) + "\n"
    fd, path = tempfile.mkstemp()
    try:
        with os.fdopen(fd, "w") as f:
            f.write(file_content)

        env = os.environ.copy()
        env.setdefault("TERM", "xterm")
        child = pexpect.spawn(EVI_BIN, [path], env=env, encoding="utf-8")
        child.delaybeforesend = float(os.getenv("EVI_DELAY_BEFORE_SEND", "0.1"))
        child.setwinsize(24, 80)

        # Let the initial render finish and then jump to the last line
        get_screen_and_cursor(child)
        child.send("G")

        screen, pos = get_screen_and_cursor(child)
        lines_map = _parse_screen(screen)

        # Expect the last line to occupy rows 22 and 23 without a blank line
        assert lines_map[22] == lines[-1][:80]
        assert lines_map[23] == lines[-1][80:]
        assert pos == (22, 1)

        child.send(":q!\r")
        child.expect(pexpect.EOF)
    finally:
        os.unlink(path)

