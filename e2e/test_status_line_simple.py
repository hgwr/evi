#!/usr/bin/env python3

import os
import sys
import tempfile
import time
import pexpect

from .helpers import spawn_evi, expect_cursor

# Test file to verify that j key doesn't move cursor to status line
# Timing robustified: remove large delaybeforesend and add retry wait after sending keys.

def _expect_cursor(child, expected_line=None, max_wait=0.3):  # backward compat wrapper
    return expect_cursor(child, expected_line, timeout=max_wait)

def test_j_doesnt_go_to_status_line():
    file_content = "line1\nline2\nline3\nline4\nline5\n"
    fd, path = tempfile.mkstemp()
    try:
        with os.fdopen(fd, "w") as f:
            f.write(file_content)

        child = spawn_evi(path, rows=24, cols=80)

        time.sleep(0.1)

        # Go to first line
        child.send("1G0")
        # Wait for line 1 to be recognized
        line, col = _expect_cursor(child, expected_line=1)
        assert line == 1

        # Move down until reaching line 5 (buffer-wise). Implementation may skip intermediate visual rows.
        target_final = 5
        safety = 10
        while line < target_final and safety > 0:
            child.send("j")
            new_line, col = _expect_cursor(child)
            assert new_line <= 23, f"Cursor in status line! Position: line={new_line}"
            # Should not move backwards
            assert new_line >= line, f"Line moved backwards from {line} to {new_line}"
            line = new_line
            safety -= 1
        assert line == target_final, f"Did not reach line {target_final}, got {line}"

        # Extra j at EOF should stay at 5
        child.send("j")
        line, col = _expect_cursor(child, expected_line=5)
        assert line == 5, f"Should stay at line 5, got {line}"
        assert line <= 23, f"Cursor in status line! Position: line={line}"

        child.send(":q!\r")
        child.expect(pexpect.EOF)
        print("\u2713 Test passed: j movement does not go to status line")
        
    finally:
        os.unlink(path)

if __name__ == "__main__":
    test_j_doesnt_go_to_status_line()
