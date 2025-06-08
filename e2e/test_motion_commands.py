import os
import re
import pexpect
import tempfile
from typing import Tuple

from .conftest import EVI_BIN


def get_cursor_position(child: pexpect.spawn) -> Tuple[int, int]:
    """Return the current (row, col) using the `Ctrl-G` file information."""
    child.send("\x07")
    child.expect(r"line (\d+) of \d+ --\d+%-- col (\d+)")
    row = int(child.match.group(1))
    col = int(child.match.group(2))
    return row, col


# The `goto` function is currently unused in tests but is intended for future use
# in testing or implementing motion commands that require moving the cursor to
# a specific (row, col) position in the editor.
def goto(child: pexpect.spawn, row: int, col: int) -> None:
    """Move the cursor to (row, col) using vi commands."""
    child.send(f"{row}G")
    child.send("0")
    if col > 1:
        child.send(f"{col - 1}l")


def get_screen_and_cursor(child: pexpect.spawn, rows: int = 24) -> Tuple[str, Tuple[int, int]]:
    """Return screen contents and final cursor position using `Ctrl-G`."""
    child.send("\x07")
    child.expect(r"line (\d+) of \d+ --\d+%-- col (\d+)")
    screen = child.before + child.after
    try:
        screen += child.read_nonblocking(size=4096, timeout=0.1)
    except (pexpect.exceptions.TIMEOUT, pexpect.exceptions.EOF):
        pass
    matches = list(re.finditer(r"\x1b\[(\d+);(\d+)H", screen))
    for m in reversed(matches):
        if int(m.group(1)) < rows:
            return screen, (int(m.group(1)), int(m.group(2)))
    if matches:
        m = matches[-1]
        return screen, (int(m.group(1)), int(m.group(2)))
    return screen, (1, 1)


def run_motion_test(
    file_content: str,
    terminal_size: Tuple[int, int],
    initial_cursor_pos: Tuple[int, int],
    command_to_test: str,
    expected_cursor_pos: Tuple[int, int],
) -> str:
    """Generic helper to test motion commands."""
    fd, path = tempfile.mkstemp()
    try:
        if file_content == "":
            file_content = "\n"
        with os.fdopen(fd, "w") as f:
            f.write(file_content)

        env = os.environ.copy()
        env.setdefault("TERM", "xterm")

        child = pexpect.spawn(EVI_BIN, [path], env=env, encoding="utf-8")
        child.delaybeforesend = float(os.getenv("EVI_DELAY_BEFORE_SEND", "0.1"))
        child.setwinsize(*terminal_size)

        # Ensure the editor has finished drawing before running commands
        get_screen_and_cursor(child)

        # Move to the requested starting position
        goto(child, initial_cursor_pos[0], initial_cursor_pos[1])

        # Send the command to test
        child.send(command_to_test)
        screen, pos = get_screen_and_cursor(child)
        assert pos == expected_cursor_pos
        assert file_content.splitlines()[0] in screen

        child.send(":q!\r")
        child.expect(pexpect.EOF)
        return screen
    finally:
        os.unlink(path)


def test_motion_w():
    run_motion_test(
        file_content="word1 word2",
        terminal_size=(24, 80),
        initial_cursor_pos=(1, 1),
        command_to_test="w",
        expected_cursor_pos=(1, 7),
    )


def test_motion_dollar():
    run_motion_test(
        file_content="hello world\n",
        terminal_size=(24, 80),
        initial_cursor_pos=(1, 1),
        command_to_test="$",
        expected_cursor_pos=(1, 11),
    )

def test_motion_l():
    run_motion_test(
        file_content="abc\n",
        terminal_size=(24, 80),
        initial_cursor_pos=(1, 1),
        command_to_test="l",
        expected_cursor_pos=(1, 2),
    )


def test_motion_caret():
    # TODO: '^' command is not yet implemented in evi
    # run_motion_test(
    #     file_content="  indented text\n",
    #     terminal_size=(24, 80),
    #     initial_cursor_pos=(1, 5),
    #     command_to_test="^",
    #     expected_cursor_pos=(1, 3),
    # )
    pass


def test_motion_G():
    run_motion_test(
        file_content="line1\nline2\nline3\n",
        terminal_size=(24, 80),
        initial_cursor_pos=(1, 1),
        command_to_test="G",
        expected_cursor_pos=(4, 1),
    )


def test_motion_gg():
    run_motion_test(
        file_content="line1\nline2\nline3\n",
        terminal_size=(24, 80),
        initial_cursor_pos=(3, 1),
        command_to_test="gg",
        expected_cursor_pos=(1, 1),
    )

# def test_motion_ctrl_b():
#     run_motion_test(
#         file_content="one\ntwo\nthree\nfour\n",
#         terminal_size=(24, 80),
#         initial_cursor_pos=(4, 1),
#         command_to_test="\x02",  # Ctrl-B page up
#         expected_cursor_pos=(1, 1),
#     )


def test_motion_ctrl_f():
    run_motion_test(
        file_content="line1\nline2\nline3\nline4\nline5\nline6\n",
        terminal_size=(4, 20),
        initial_cursor_pos=(1, 1),
        command_to_test="\x1b[6~",
        expected_cursor_pos=(1, 1),
    )


def test_motion_ctrl_b():
    run_motion_test(
        file_content="one\ntwo\nthree\nfour\nfive\nsix\n",
        terminal_size=(4, 20),
        initial_cursor_pos=(4, 1),
        command_to_test="\x1b[5~",
        expected_cursor_pos=(1, 1),
    )
