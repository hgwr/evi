# evi - vi clone editor

evi is a vi clone editor written in Rust. It is a simple and lightweight text editor that is easy to use and has a small memory footprint. It is a work in progress and is not yet feature complete.

## Features

- [ ] Basic vi commands
- [ ] Ex commands
- [ ] Search and replace
- [ ] Unicode support
- [ ] Syntax highlighting

## Installation

TBD

## Usage

TBD

## Testing

### E2E Tests

End-to-end tests are located in the `e2e/` directory and use `pytest` with `pexpect` to test the TUI application.

**Note:** On macOS, e2e tests can be unstable due to timing and terminal handling differences. For reliable test execution, use Docker:

```bash
scripts/e2e_docker.sh
```

The tests run without issues in ChatGPT Codex containers. See `e2e/README.md` for detailed testing instructions.
