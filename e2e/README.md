# e2e Tests

This directory contains end-to-end tests for **evi** using `pytest` and the
`pexpect` library. The tests drive the TUI application through a pseudo
terminal and verify behaviour described in `doc/spec.md`.

## Setup

Install the required Python packages:

```bash
pip install -r requirements.txt
```

The tests automatically build the `evi` binary using Cargo before running.

## Running the tests

Execute all e2e tests with:

```bash
pytest
```

Specific tests can be selected in the usual `pytest` ways, e.g.:

```bash
pytest e2e/test_vi_commands.py::test_delete_word
```

## Running the tests in Docker

The repository provides a Docker setup for running the e2e tests in an isolated environment. Build the image and execute the tests using:

```bash
scripts/e2e_docker.sh
```

This script builds the Docker image based on the official `rust` image, mounts the project directory into the container, and runs `cargo build --verbose` followed by `pytest e2e --verbose`. The Docker image installs the Python dependencies in a virtual environment to avoid system package conflicts.

## Handling slow environments

When running in slower containers, `pexpect` may time out before `evi` responds.
To increase reliability, you can adjust the following environment variables:

- `EVI_DELAY_BEFORE_SEND` – Delay (in seconds) before sending each keystroke to `evi` (default: 0.1s).
- `EVI_DELAY_AFTER_ESC` – Delay (in seconds) after sending an Escape (ESC) key (default: 0.05s).
- `EVI_PEXPECT_TIMEOUT` – Timeout (in seconds) for `pexpect` when waiting for `evi`'s output (default: 0.2s).

Example command:

```bash
EVI_PEXPECT_TIMEOUT=2 EVI_DELAY_BEFORE_SEND=0.2 pytest e2e --verbose
```

Higher values may be required when running inside the Codex workspace.
