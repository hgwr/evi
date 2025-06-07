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
