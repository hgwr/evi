# Implementation Tasks

The following tasks outline key feature work required to progress toward a POSIX-compliant vi clone. Each item references missing functionality listed in [doc/todo.md](todo.md).

## 1. Implement open-line commands (`o`, `O`)
- Allow inserting a new line below (`o`) or above (`O`) the current line while in command mode.
- Update editing state so that the cursor moves to the new line and enters insert mode.
- Ensure undo history and repeat (`.`) work correctly with these commands.

## 2. Implement search motions (`/`, `?`, `n`, `N`)
- Support forward (`/`) and backward (`?`) pattern searches.
- Implement repeat search commands `n` (same direction) and `N` (opposite direction).
- Integrate search highlights and navigation with existing movement commands.

## 3. Implement write command (`:w` and `:w!`)
- Add ability to save the current buffer to a file with `:w`.
- Implement `:w!` to force overwriting read-only or existing files when permitted.
- Provide error handling and messages consistent with classic vi behavior.
