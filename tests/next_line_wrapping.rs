//! Unit tests for NextLine behavior with wrapped lines.
use evi::editor::{Editor, TerminalSize};
use evi::buffer::Buffer;
use evi::command::commands::move_cursor::NextLine;
use evi::command::base::Command;

fn setup_editor(lines: &[&str], width: u16, height: u16) -> Editor {
    let mut editor = Editor::new();
    editor.resize_terminal(width, height);
    let mut buf = Buffer::new();
    for &l in lines { buf.lines.push(l.to_string()); }
    editor.buffer = buf;
    editor
}

#[test]
fn next_line_moves_to_next_buffer_line_once() {
    let width = 10u16;
    let mut ed = setup_editor(&["aaaa", &"b".repeat(25), "cccc"], width, 20);
    let mut next = NextLine;
    next.execute(&mut ed).unwrap();
    assert_eq!(ed.cursor_position_in_buffer.row, 1, "NextLine should move to following buffer line");
}

#[test]
fn visual_row_progress_can_be_simulated() {
    // Simulate moving inside a long line and calling NextLine after repositioning
    let width = 10u16;
    let long = "b".repeat(25); // 3 wrapped rows
    let mut ed = setup_editor(&[&long, &"c".repeat(5)], width, 20);
    // Manually place cursor at middle visual row by advancing columns
    // (simplified: just set state)
    ed.cursor_position_in_buffer.row = 0;
    ed.cursor_position_in_buffer.col = 11; // into second visual segment
    // approximate screen row as 1 (second visual row)
    ed.cursor_position_on_screen.row = 1;
    let mut next = NextLine;
    next.execute(&mut ed).unwrap();
    // Should move to row 1 (second buffer line) directly
    assert_eq!(ed.cursor_position_in_buffer.row, 1);
}

#[test]
fn next_line_at_last_line_only_scrolls_when_possible() {
    // Two lines that each wrap multiple times; repeatedly calling NextLine at end should not panic
    let width = 8u16;
    let mut ed = setup_editor(&[&"x".repeat(30), &"y".repeat(30)], width, 6);
    ed.cursor_position_in_buffer.row = 1; // move to last buffer line
    let mut next = NextLine;
    // executing NextLine at last line should not move row further
    let original_row = ed.cursor_position_in_buffer.row;
    next.execute(&mut ed).unwrap();
    assert_eq!(ed.cursor_position_in_buffer.row, original_row);
}
