# Marks are not implemented yet
# import os
# import tempfile
# import pexpect

# from .conftest import EVI_BIN
# from .test_motion_commands import get_screen_and_cursor, goto


# def test_set_and_jump_mark():
#     fd, path = tempfile.mkstemp()
#     try:
#         with os.fdopen(fd, "w") as f:
#             f.write("line1\nline2\nline3\n")

#         env = os.environ.copy()
#         env.setdefault("TERM", "xterm")
#         child = pexpect.spawn(EVI_BIN, [path], env=env, encoding="utf-8")
#         child.delaybeforesend = float(os.getenv("EVI_DELAY_BEFORE_SEND", "0.1"))
#         child.setwinsize(24, 80)

#         get_screen_and_cursor(child)
#         goto(child, 2, 1)
#         child.send("ma")
#         goto(child, 1, 1)
#         child.send("'a")
#         _, pos = get_screen_and_cursor(child)
#         assert pos in [(2, 1), (3, 1)]

#         child.send(":q!\r")
#         child.expect(pexpect.EOF)
#     finally:
#         os.unlink(path)

