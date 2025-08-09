import os
import tempfile
import pexpect
from .conftest import EVI_BIN


def test_ex_command_backspace():
    fd, path = tempfile.mkstemp()
    try:
        with os.fdopen(fd, 'w') as f:
            f.write('a\nb\n')

        env = os.environ.copy()
        env.setdefault('TERM', 'xterm')

        child = pexpect.spawn(EVI_BIN, [path], env=env, encoding='utf-8')
        child.delaybeforesend = float(os.getenv('EVI_DELAY_BEFORE_SEND', '0.1'))

        child.send(':qz')
        child.sendcontrol('h')
        child.send('\r')

        child.expect(pexpect.EOF, timeout=5)
    finally:
        os.unlink(path)

