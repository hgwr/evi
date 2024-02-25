use crate::command::base::Command;
use crate::editor::Editor;

pub struct DownOneLine;
impl Command for DownOneLine {
    fn execute(&mut self, editor: &mut Editor) {
        editor.cursor_position_on_screen.row += 1;
    }
}

pub struct UpOneLine;
impl Command for UpOneLine {
    fn execute(&mut self, editor: &mut Editor) {
        editor.cursor_position_on_screen.row -= 1;
    }
}

pub struct RightOneChar;
impl Command for RightOneChar {
    fn execute(&mut self, editor: &mut Editor) {
        editor.cursor_position_on_screen.col += 1;
    }
}

pub struct LeftOneChar;
impl Command for LeftOneChar {
    fn execute(&mut self, editor: &mut Editor) {
        editor.cursor_position_on_screen.col -= 1;
    }
}
