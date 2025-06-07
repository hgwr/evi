# Specification

evi is a vi clone editor written in Rust. It is a simple and lightweight text editor that is easy to use and has a small memory footprint. It is a work in progress and is not yet feature complete.

## Features

- [x] Basic vi commands
- [x] Ex commands
- [x] Search and replace
- [ ] Unicode support
- [ ] Syntax highlighting

## Requirement

- Open a text file and display it on the screen.
- Manipulate the cursor on the screen to display any position in the opened text file on the screen.
- Modify the contents of a text file.
- Save file.
- Create a new file.
- Loading files.
- Delete files.
- Line number display.
- Cursor movement (up/down, left/right, word by word, movement to the beginning/end of the line).
- Inserting text (insert mode).
- Delete text (delete character at cursor, line).
- Copy text (selection in visual mode).
- Paste text.
- Search for text (forward, backward).
- Text replacement (within specified range, in all files).
- Undo.
- Redo.
- Automatic indentation (applies the same indentation as the previous line when starting a new line).
- Wrap display (display long lines to fit screen width).
- Syntax highlighting (optional, color-coded according to language).
- Customize behavior with configuration files.

## vi commands

The following vi commands are currently implemented:

- `h`, `j`, `k`, `l` — move the cursor left, down, up and right
- `0`, `$` — jump to the beginning or end of the current line
- `w`, `b` — move forward or backward by word
- `i`, `a` — insert before or append after the cursor
- `o`, `O` — open a new line below or above the current line
- `c{motion}` — change text specified by a motion
- `d{motion}` — delete text specified by a motion
- `y{motion}` — yank text into the unnamed register
- `p`, `P` — paste text from the unnamed register
- `r`, `R` — replace a character or enter replace mode
- `/`, `?`, `n`, `N` — search forward or backward and repeat the search
- `f`, `F`, `t`, `T`, `;` — find character on the current line
- `x` — delete the character under the cursor
- `u` — undo the last change
- `Ctrl-g` — display file information
- `ZZ` — write the file if modified and exit
- `:` — enter ex command mode

### POSIX vi compatibility

evi aims to eventually implement the full set of commands described in the
POSIX \`vi\` specification.  The current focus is on a small core that can be
tested end to end.  The following subset of standard commands is used as the
target for the E2E tests described in this repository:

- Cursor motion commands: `h`, `j`, `k`, `l`, `0`, `$`, `w`, `b`.
- Insert and append commands: `i`, `a`, `o`, `O`, `I`, `A`.
- Deletion: `x`, `dd`, `dw`.
- Searching: `/pattern`, `?pattern`, `n`, `N`.
- Undo and redo: `u`, `Ctrl-r`.
- File information: `Ctrl-g`, and `ZZ` to write and exit.

Additional commands such as marks, macros and visual mode are not yet
implemented.

## ex commands

The following ex commands are implemented:

- `:q`, `:q!`
- `:w`, `:w!`
- `:x`, `:wq`
- `:g`, `:g!`

### Exit and write, read

`:q`
`:q!`

`:w` Write buffer to file
`:w!` Force a buffer to be written to a file even if the file is open in read mode

`:e!` Reload the file (edited content is discarded).

`:x` The file is written and exits.The write only occurs if the file has been modified.
`:wq` The file is written and exits. The writing occurs even if the file has not been modified.

`:r filename` Loading another file

### Display

`:p` Display the current line
`:1,3p` Display lines 1 to 3
`:1p` Display line 1
`:1` Display line 1
`:1,3` Display lines 1 to 3

### Substitution

`:s/screen/line` Replace the first occurrence of `screen` with `line`

`:s/screen/line/g` Replace all occurrences of `screen` with `line`

`:1s/screen/line` Replace the first occurrence of `screen` with `line` in line 1

`:1,3s/screen/line` Replace the first occurrence of `screen` with `line` in lines 1 to 3

### Deletion of lines

`:1d` Delete line 1
`:1,3d` Delete lines 1 to 3

### Movement of lines

`:1m5` Move line 1 to line 5
`:1,3m5` Move lines 1 to 3 to line 5

### Copy lines

`:1co5` Copy line 1 to line 5
`:1,3co5` Copy lines 1 to 3 to line 5

### Show/hide line numbers

`:set number` Display line numbers
`:set nonumber` Hide line numbers
`:set nu` Display line numbers
`:set nonu` Hide line numbers

`:1,10#` Display line numbers from line 1 to line 10

`:=` Display the total number of lines

`:.=` Display the current line number

`:/pattern/=` Display the line number of the pattern

### Row address symbols, patterns

`:.,$d` Delete current line to end of file
`10,.m$` Move from line 10 to the current line to the end of the file
`:%d` Delete all lines in the file
`:%t$` Copy all lines and append them to the end of the file

`:.,.+2d` Delete the current line and the next 2 lines

`:10,$m.-2` Move from line 10 to the end of the file to 2 lines before the current line

`:.,+10#` Display line numbers from the current line to 20 lines ahead

`:-,+t0` Copy from the line above the cursor to the line below the cursor and insert it at the beginning of the file

`:.,/while/d` Delete from the current line to the line that contains the pattern

### Global search

`:g/pattern` Find the last occurrence of the pattern in the file and set it as the cursor line

`:g/pattern/p` Find and display all lines in the file that contain the pattern

`:g!/pattern/nu` Find and display all lines that do not contain the pattern, including line numbers

`:1,10g/pattern/p` Find and display lines that contain the pattern from line 1 to line 10

### ex commands BNF

```BNF
<command> ::= ":" <simple_command> | ":" <complex_command>

<simple_command> ::= "q" | "q!" | "w" | "w!" | "e!" | "x" | "wq" | "p" | ":=" | ".=" | "set number" | "set nonumber" | "set nu" | "set nonu"

<complex_command> ::= <write_read_command> | <display_command> | <substitution_command> | <deletion_command> | <movement_command> | <copy_command> | <line_number_command> | <global_command> | <pattern_command>

<write_read_command> ::= "r" <filename>
<display_command> ::= [<line_range>] "p"
<substitution_command> ::= <line_range> "s/" <pattern> "/" <replacement> ["/g"]
<deletion_command> ::= [<line_range>] "d"
<movement_command> ::= [<line_range>] "m" <line_address>
<copy_command> ::= [<line_range>] "co" <line_address>
<line_number_command> ::= [<line_range>] "#"
<global_command> ::= [<line_range>] "g/" <pattern> "/" <global_option>
<pattern_command> ::= [<line_range>] "t" <line_address>

<line_range> ::= <line_address> | <line_address> "," <line_address> | "%" | <line_address> "," <pattern>
<line_address> ::= <number> | "." | "$" | "-" | "+"
<pattern> ::= [a-zA-Z0-9]+
<replacement> ::= [a-zA-Z0-9]+
<filename> ::= [a-zA-Z0-9._/-]+
<global_option> ::= "p" | "nu"
```
