# evi - vi clone editor

evi is a lightweight vi clone written in Rust. The project is still a work in progress, but it already provides a simple modal editor core.

## Repository Layout

- **src/main.rs** – application entry point. Each module is declared at the top of this file.
- **src/buffer.rs** – text buffer management including insertion, deletion and file I/O.
- **src/editor.rs** – tracks editor state and modes and executes Ex commands.
- **src/command/** – implementation of vi commands via the `Command` trait.
  - `compose.rs` interprets key sequences.
  - `factory.rs` creates concrete command objects.
  - `commands/` holds individual command implementations (some like `print.rs` are incomplete).
- **src/ex/** – lexer and parser for Ex commands.
- **src/main_loop.rs** – event loop using `crossterm`.
- **src/render.rs** – screen rendering logic.
- **doc/spec.md** – planned features and Ex command specification with BNF.
- **log4rs.yml** – log configuration (logs are written to `log/app.log`).

## Building and Testing

Use `cargo build` to compile the editor and `cargo test` to run the provided test suite. All tests should pass.

## Learning the Codebase

1. **Implement commands** – new vi or Ex commands can be added under `src/command/commands/`.
2. **Understand input parsing** – see `src/command/compose.rs` for how key presses are turned into commands.
3. **Ex command parser** – study `src/ex/lexer.rs` and `src/ex/parser.rs` together with `doc/spec.md`.
4. **Complete TODO items** – functions such as `PrintCommand` or `Editor::get_line_number_from` still need work and additional tests.

Contributions are welcome. Please check `doc/spec.md` and existing tests before submitting patches.
