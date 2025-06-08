import os
import pexpect
import tempfile
import time
from typing import cast
from .conftest import EVI_BIN


def run_commands(commands, initial_content="", exit_cmd=":wq\r"):
    fd, path = tempfile.mkstemp()
    try:
        if initial_content == "":
            # The reason for inserting "\n" is that the editor’s buffer assumes at least one line exists. When a file is truly empty (0 bytes), the buffer would load an empty vector of lines, and operations like insertion expect self.lines[0] to be valid. Starting with one empty line matches how vi behaves when opening an empty file and prevents index errors during tests.
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
        # `pexpect` treats an ESC byte that is immediately followed by another
        # character as an "Alt" modified key.  When the delay between sending
        # commands is too small some operating systems (notably macOS) merge
        # ``ESC`` and the following character into a single event.  This causes
        # the editor to stay in insert mode and the tests to time out waiting
        # for the process to exit.  A slightly longer delay is therefore used
        # to ensure that ``ESC`` is recognized correctly across platforms.
        #
        # The value can be overridden using ``EVI_DELAY_BEFORE_SEND`` for
        # experimentation, but defaults to 0.1 seconds. ``EVI_DELAY_AFTER_ESC``
        # can be used to delay after sending ESC and defaults to 0.05 seconds.
        child.delaybeforesend = float(os.getenv("EVI_DELAY_BEFORE_SEND", "0.1"))
        delay_after_esc = float(os.getenv("EVI_DELAY_AFTER_ESC", "0.05"))

        for c in commands:
            child.send(c)
            if c == "\x1b" and delay_after_esc > 0:
                time.sleep(delay_after_esc)

        if exit_cmd is not None:
            child.send(exit_cmd)

        child.expect(pexpect.EOF)

        with open(path) as f:
            result = f.read()
    finally:
        os.unlink(path)
    return result
