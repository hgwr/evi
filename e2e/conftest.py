import os
import subprocess
import pytest

ROOT_DIR = os.path.dirname(os.path.dirname(__file__))
EVI_BIN = os.path.join(ROOT_DIR, 'target', 'debug', 'evi')

# Increase the delay between keystrokes to avoid timing issues in slow
# environments such as CI containers.  The tests run faster with smaller
# defaults but callers can override these values if needed.
os.environ.setdefault('EVI_DELAY_BEFORE_SEND', '0.01')
os.environ.setdefault('EVI_DELAY_AFTER_ESC', '0.005')
# Slow execution environments (like the Codex workspace container) may take
# longer to output screen updates. ``EVI_PEXPECT_TIMEOUT`` controls how long the
# helper functions wait when reading from the spawned ``evi`` process.  A lower
# timeout speeds up the tests on faster machines, but may need to be increased
# if spurious ``pexpect.TIMEOUT`` errors occur.  We therefore use a relatively
# small default of ``0.2`` seconds and allow callers to override it via the
# ``EVI_PEXPECT_TIMEOUT`` environment variable if required.
os.environ.setdefault('EVI_PEXPECT_TIMEOUT', '0.2')

@pytest.fixture(scope='session', autouse=True)
def build_evi():
    subprocess.run(['cargo', 'build'], cwd=ROOT_DIR, check=True)
