import os
import subprocess
import pytest

ROOT_DIR = os.path.dirname(os.path.dirname(__file__))
EVI_BIN = os.path.join(ROOT_DIR, 'target', 'debug', 'evi')

# Increase the delay between keystrokes to avoid timing issues in slow
# environments such as CI containers.
os.environ.setdefault('EVI_DELAY_BEFORE_SEND', '0.1')

@pytest.fixture(scope='session', autouse=True)
def build_evi():
    subprocess.run(['cargo', 'build'], cwd=ROOT_DIR, check=True)
