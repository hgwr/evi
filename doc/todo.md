# TODO

This document lists vim commands that are not yet implemented in evi.

## Implemented vim commands

- [x] `h`, `j`, `k`, `l` — move the cursor left, down, up and right
- [x] `0`, `$` — jump to the beginning or end of the current line
- [x] `w`, `b` — move forward or backward by word
- [x] `i` — insert before the cursor
- [x] `a` — append after the cursor
- [x] `x` — delete the character under the cursor
- [x] `d{motion}` — delete text specified by a motion
- [x] `u` — undo the last change
- [x] `Ctrl-g` — display file information
- [x] `ZZ` — write the file if modified and exit

## Unimplemented vim commands

- [ ] `o`, `O` — open a new line below/above the current line
- [ ] `I`, `A` — insert/append at the beginning/end of line
- [ ] `c`, `cc`, `cw`, `C` — change commands
- [ ] `y`, `yy`, `yw` — yank operations
- [ ] `p`, `P` — paste text from the unnamed buffer
- [ ] `r`, `R` — replace character or enter replace mode
- [ ] `J` — join lines
- [ ] `/`, `?`, `n`, `N` — search motions
- [ ] `f`, `F`, `t`, `T` — find character on the current line
- [ ] Visual mode commands such as `v`, `V`
- [ ] Marks (`m`{char}) and jumps (`'{char}`)
- [ ] Macros (`@`{register})

## Implemented ex commands

- [x] `:q`, `:q!` — quit (with force)
- [x] `:wq` — write and quit
- [x] `:s/pattern/replacement/[g]` — substitution
- [x] `:p` — display current line
- [x] `:{line}` — go to line number
- [x] `:{range}d` — delete lines in range
- [x] `:{range}p` — print lines in range

## Unimplemented ex commands

The ex commands described in `doc/spec.md` but not yet implemented include:

- [ ] `:w` and `:w!` — write buffer to file (with or without force)
- [ ] `:e!` — reload file discarding changes
- [ ] `:x` — write if modified and exit
- [ ] `:r {file}` — read another file into the buffer
- [ ] `:m` and `:co` — move or copy lines
- [ ] `:set number`, `:set nonumber`, `:set nu`, `:set nonu`
- [ ] `:#`, `:=`, `:.=` and `:/pattern/=` — line number related commands
- [ ] Global search commands `:g` and `:g!`
- [ ] Line range addresses using patterns or relative offsets (`+`, `-`) are not handled
- [ ] Printing with `:p` and related range forms (implementation pending)

## Key unimplemented features

- [ ] Unicode support (full handling of multibyte characters)
- [ ] Syntax highlighting
- [ ] Configuration file customization

These items are targets for future development in order to be closer to a full vim clone.
