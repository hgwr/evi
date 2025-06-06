# Implementation Tasks

The following tasks outline key feature work required to progress toward a POSIX-compliant vi clone. Each item references missing functionality listed in [doc/todo.md](todo.md).

## 1. Implement change commands (`c`, `cc`, `cw`, `C`)
- Allow replacing text using motions or linewise variants.
- Support repeating changes with `.` and include undo history.

## 2. Add yank and paste operations (`y`, `yy`, `yw`, `p`, `P`)
- Enable copying text to the unnamed buffer and pasting it elsewhere.
- Handle both linewise and characterwise cases consistent with vi.

## 3. Implement ex `:x` command
- Save the current buffer if modified and then exit.
- Provide error handling consistent with classic vi for unsaved changes.
