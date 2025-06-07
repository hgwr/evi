import os
import pexpect
import tempfile

from .conftest import EVI_BIN

def run_commands(commands, initial_content=""):
    fd, path = tempfile.mkstemp()
    try:
        with os.fdopen(fd, 'w') as f:
            f.write(initial_content)
        env = os.environ.copy()
        env.setdefault('TERM', 'xterm')
        child = pexpect.spawn(EVI_BIN, [path], env=env, encoding='utf-8')
        child.delaybeforesend = 0.05
        for c in commands:
            child.send(c)
        child.send(':wq\r')
        child.expect(pexpect.EOF)
        with open(path) as f:
            result = f.read()
    finally:
        os.unlink(path)
    return result
