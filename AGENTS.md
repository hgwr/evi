# `evi` Project Overview and Instructions

This document provides an overview of the `evi` project and the commands required to build and test it.

## Project Goal

`evi` is a clone of the POSIX-compatible `vi` editor, developed in Rust. The primary objective of this project is to faithfully replicate the behavior of the standard `vi` editor as specified by the POSIX standard.

To achieve this, development is heavily driven by end-to-end (e2e) tests. These tests ensure that `evi`'s functionality precisely matches that of a reference `vi` implementation, providing a robust way to verify compatibility as the project evolves.

## Build and Test Instructions

This section describes the commands required to build the `evi` Rust project, run unit tests, and run end-to-end (e2e) tests.

### Rust Project (`evi`)

#### Build

To compile the `evi` application:

```sh
cargo build --verbose
```

#### Unit Testing

To run unit tests for the `evi` application:

```sh
cargo test --verbose
```

### End-to-end (e2e) Testing

The e2e tests are crucial for ensuring that `evi` behaves identically to a POSIX-compatible `vi` editor. The tests are located in the `e2e/` directory, written in Python, and utilize `pytest` for execution and `pexpect` for interaction.

Using `pexpect`, each test programmatically controls the `evi` process, sending keystrokes and asserting the editor's state against the expected behavior of a standard `vi`. This rigorous testing methodology is central to the development process.

#### Prerequisites

Make sure you have Python and `pip` installed.

#### Install dependencies

Install the Python dependencies required for e2e testing:

```sh
pip install -r e2e/requirements.txt
```

#### Run e2e tests

To run the full suite of e2e tests using `pytest-xdist` for parallel execution:

```sh
pytest -n auto e2e --verbose
```