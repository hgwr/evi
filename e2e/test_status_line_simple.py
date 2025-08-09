#!/usr/bin/env python3

import os
import sys
import tempfile
import time
import pexpect

from .conftest import EVI_BIN

# Test file to verify that j key doesn't move cursor to status line
# Timing robustified: remove large delaybeforesend and add retry wait after sending keys.

def _expect_cursor(child, expected_line=None, max_wait=0.3):
    """Fetch current cursor (line, col) via Ctrl-G; retry until stable or timeout.
    Returns (line, col). If expected_line given, will retry until line matches or timeout.
    """
    deadline = time.time() + max_wait
    last_line = None
    last_col = None
    while True:
        child.send("\x07")  # Ctrl-G
        child.expect(r"line (\d+) of \d+ --\d+%-- col (\d+)", timeout=0.2)
        line = int(child.match.group(1))
        col = int(child.match.group(2))
        if expected_line is None:
            return line, col
        if line == expected_line:
            return line, col
        last_line, last_col = line, col
        if time.time() >= deadline:
            return last_line, last_col
        time.sleep(0.02)

def test_j_doesnt_go_to_status_line():
    file_content = "line1\nline2\nline3\nline4\nline5\n"
    fd, path = tempfile.mkstemp()
    try:
        with os.fdopen(fd, "w") as f:
            f.write(file_content)

        env = os.environ.copy()
        env.setdefault("TERM", "xterm")
        
        child = pexpect.spawn(
            EVI_BIN,
            [path],
            env=env,
            encoding="utf-8"
        )
        # Remove artificial per-character delay to avoid race (was 0.1 previously)
        child.delaybeforesend = 0
        child.setwinsize(24, 80)  # 24 rows, 80 columns

        time.sleep(0.1)

        # Go to first line
        child.send("1G0")
        # Wait for line 1 to be recognized
        line, col = _expect_cursor(child, expected_line=1)
        assert line == 1

        # Move from line 1 to line 5
        for target in range(2, 6):
            child.send("j")
            # Retry until we actually observe the new line (avoids timing races)
            line, col = _expect_cursor(child, expected_line=target)
            assert line == target, f"Expected line {target}, got {line}"
            assert line <= 23, f"Cursor in status line! Position: line={line}"

        # Extra j at EOF should stay at 5
        child.send("j")
        line, col = _expect_cursor(child, expected_line=5)
        assert line == 5, f"Should stay at line 5, got {line}"
        assert line <= 23, f"Cursor in status line! Position: line={line}"

        child.send(":q!\r")
        child.expect(pexpect.EOF)
        print("✓ Test passed: j movement does not go to status line")
        
    finally:
        os.unlink(path)

if __name__ == "__main__":
    test_j_doesnt_go_to_status_line()
