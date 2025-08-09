use evi::editor::Editor;
use evi::wrapping::WrappingCalculator;
use evi::buffer::Buffer;

fn make_editor(lines: &[&str], width: u16, height: u16) -> Editor {
    let mut ed = Editor::new();
    ed.resize_terminal(width, height);
    let mut buf = Buffer::new();
    for l in lines { buf.lines.push((*l).to_string()); }
    ed.buffer = buf;
    ed
}

#[test]
fn long_line_wrapping_height() {
    let width = 10u16;
    let ed = make_editor(&["abcdefghijklmnopqrstuvwxyz"], width, 20);
    let calc = WrappingCalculator::new(width);
    let h = calc.line_height(&ed.buffer.lines[0]);
    assert!(h >= 3, "expected at least 3 wrapped rows, got {}", h);
}

#[test]
fn fullwidth_chars_alignment() {
    let width = 8u16;
    let ed = make_editor(&["あいうえおかき"], width, 20); // full-width chars (2 cells)
    let calc = WrappingCalculator::new(width);
    let h = calc.line_height(&ed.buffer.lines[0]);
    assert!(h >= 2, "full-width wrapping height should be >=2, got {}", h);
}

#[test]
fn screen_position_moves_with_window() {
    let width = 10u16; let height = 5u16;
    let mut ed = make_editor(&["1234567890abcdefghij" , "zzz"], width, height); // first line wraps twice
    // place cursor at end of first long line
    ed.cursor.row = 0; ed.cursor.col = ed.buffer.lines[0].chars().count();
    ed.ensure_cursor_visible();
    let _ = ed.calculate_screen_position();
    // move cursor to second line; ensure we get a valid screen position (non-panicking)
    ed.cursor.row = 1; ed.cursor.col = 0; ed.ensure_cursor_visible();
    let sp2 = ed.calculate_screen_position();
    assert!(sp2.col <= width, "cursor column should be within width");
}

#[test]
fn upward_scroll_visibility() {
    let width = 8u16; let height = 4u16;
    let mut ed = make_editor(&["aaaaaaaaaaaaaaaa", "bbbb", "cccc"], width, height); // first wraps
    // move cursor to last line to force window down
    ed.cursor.row = 2; ed.cursor.col = 0; ed.ensure_cursor_visible();
    let window_after_down = ed.window_top_row;
    // move cursor back to first line; ensure window_top_row adjusts upward
    ed.cursor.row = 0; ed.cursor.col = 0; ed.ensure_cursor_visible();
    assert!(ed.window_top_row <= window_after_down, "window should scroll back up");
    assert_eq!(ed.cursor.row, 0);
}
