# `evi` Build and Test Instructions

This document describes the commands required to build the `evi` Rust project, run unit tests, and run end-to-end (e2e) tests.

## Rust Project (`evi`)

### Build

To compile the `evi` application:

```sh
cargo build --verbose
```

### Unit Testing

To run unit tests for the `evi` application:

```sh
cargo test --verbose
```

## End-to-end (e2e) Testing

The e2e tests are written in Python and use `pytest`.

### Prerequisites

Make sure you have Python and pip installed.

### Install dependencies

Install the Python dependencies required for e2e testing:

```sh
pip install -r e2e/requirements.txt
```

### Run e2e tests

Run e2e tests:

```sh
pytest e2e --verbose
```
