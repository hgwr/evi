import os
import subprocess
import pytest

ROOT_DIR = os.path.dirname(os.path.dirname(__file__))
EVI_BIN = os.path.join(ROOT_DIR, 'target', 'debug', 'evi')

@pytest.fixture(scope='session', autouse=True)
def build_evi():
    subprocess.run(['cargo', 'build'], cwd=ROOT_DIR, check=True)
