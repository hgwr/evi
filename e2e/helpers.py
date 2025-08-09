import os
import pexpect
import tempfile
import time
from typing import cast, Callable, Optional, Tuple
from .conftest import EVI_BIN


DEFAULT_DELAY = 0.0  # unified zero delay for deterministic tests

def run_commands(commands, initial_content: str = "", exit_cmd: str = ":wq\r"):
    fd, path = tempfile.mkstemp()
    try:
        # Ensure at least one line exists
        if initial_content == "":
            initial_content = "\n"
        with os.fdopen(fd, "w") as f:
            f.write(initial_content)

        env = os.environ.copy()
        env.setdefault("TERM", "xterm")

        child = pexpect.spawn(
            EVI_BIN,
            [path],
            env=cast(os._Environ[str], env),
            encoding="utf-8"
        )
        child.delaybeforesend = float(os.getenv("EVI_DELAY_BEFORE_SEND", str(DEFAULT_DELAY)))
        delay_after_esc = float(os.getenv("EVI_DELAY_AFTER_ESC", "0.0"))

        for c in commands:
            child.send(c)
            if c == "\x1b" and delay_after_esc > 0:
                time.sleep(delay_after_esc)

        if exit_cmd:
            child.send(exit_cmd)

        child.expect(pexpect.EOF)

        with open(path) as f:
            return f.read()
    finally:
        os.unlink(path)


def expect_cursor(child: pexpect.spawn, expected_line: Optional[int] = None, timeout: float = 0.3) -> Tuple[int, int]:
    """Fetch current cursor (line, col) via Ctrl-G; optionally wait until line matches.

    Returns (line, col).
    """
    deadline = time.time() + timeout
    last = (1, 1)
    while True:
        child.send("\x07")
        child.expect(r"line (\d+) of \d+ --\d+%-- col (\d+)", timeout=0.2)
        row = int(child.match.group(1))
        col = int(child.match.group(2))
        last = (row, col)
        if expected_line is None or row == expected_line:
            return last
        if time.time() >= deadline:
            return last
        time.sleep(0.02)


def wait_until_top_line(child: pexpect.spawn, predicate: Callable[[str], bool], timeout: float = 0.5) -> str:
    """Poll screen (via Ctrl-G triggered render flush) until predicate(screen) is True or timeout.

    Returns final captured screen.
    """
    end = time.time() + timeout
    last_screen = ""
    while time.time() < end:
        child.send("\x07")
        try:
            child.expect(r"line (\d+) of \d+ --\d+%-- col (\d+)", timeout=0.2)
        except pexpect.exceptions.TIMEOUT:
            pass
        last_screen = child.before + child.after
        # Try to read any remaining buffer (non-fatal)
        try:
            last_screen += child.read_nonblocking(size=4096, timeout=0.05)
        except Exception:
            pass
        if predicate(last_screen):
            return last_screen
        time.sleep(0.02)
    return last_screen
