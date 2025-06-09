import os
import subprocess
import pytest

ROOT_DIR = os.path.dirname(os.path.dirname(__file__))
EVI_BIN = os.path.join(ROOT_DIR, 'target', 'debug', 'evi')

# Increase the delay between keystrokes to avoid timing issues in slow
# environments such as CI containers.
os.environ.setdefault('EVI_DELAY_BEFORE_SEND', '0.1')
# Slow execution environments (like the Codex workspace container) may take
# longer to output screen updates. ``EVI_PEXPECT_TIMEOUT`` controls how long the
# helper functions wait when reading from the spawned ``evi`` process.  The
# default of ``0.3`` seconds is sometimes too short here which leads to spurious
# timeouts.  Increase it to 2 seconds unless it has been configured explicitly.
os.environ.setdefault('EVI_PEXPECT_TIMEOUT', '2')

@pytest.fixture(scope='session', autouse=True)
def build_evi():
    subprocess.run(['cargo', 'build'], cwd=ROOT_DIR, check=True)
