import os
import pexpect
import tempfile
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
        child.delaybeforesend = 0.05

        for c in commands:
            child.send(c)

        if exit_cmd is not None:
            child.send(exit_cmd)

        child.expect(pexpect.EOF)

        with open(path) as f:
            result = f.read()
    finally:
        os.unlink(path)
    return result
