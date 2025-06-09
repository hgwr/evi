use std::any::Any;

use crate::command::base::Command;
use crate::editor::{Editor, EditorCursorData};
use crate::generic_error::GenericResult;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct JoinLines {
    pub editor_cursor_data: Option<EditorCursorData>,
    pub first: Option<String>,
    pub second: Option<String>,
}

impl Default for JoinLines {
    fn default() -> Self {
        Self {
            editor_cursor_data: None,
            first: None,
            second: None,
        }
    }
}

impl Command for JoinLines {
    fn is_reusable(&self) -> bool {
        false
    }

    fn is_undoable(&self) -> bool {
        true
    }

    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        let row = editor.cursor_position_in_buffer.row;
        if row + 1 >= editor.buffer.lines.len() {
            return Ok(());
        }
        editor.is_dirty = true;
        self.editor_cursor_data = Some(editor.snapshot_cursor_data());
        self.first = Some(editor.buffer.lines[row].clone());
        self.second = Some(editor.buffer.lines[row + 1].clone());

        let new_line = format!("{}{}", self.first.as_ref().unwrap(), self.second.as_ref().unwrap());
        editor.buffer.lines[row] = new_line;
        editor.buffer.lines.remove(row + 1);
        Ok(())
    }

    fn undo(&mut self, editor: &mut Editor) -> GenericResult<()> {
        if let (Some(cursor), Some(first), Some(second)) = (
            self.editor_cursor_data,
            &self.first,
            &self.second,
        ) {
            let row = cursor.cursor_position_in_buffer.row;
            if row < editor.buffer.lines.len() {
                editor.buffer.lines[row] = first.clone();
                editor.buffer.lines.insert(row + 1, second.clone());
            }
            editor.restore_cursor_data(cursor);
        }
        Ok(())
    }

    fn redo(&mut self, editor: &mut Editor) -> GenericResult<Option<Box<dyn Command>>> {
        editor.is_dirty = true;
        let mut new_cmd = Box::new(JoinLines::default());
        new_cmd.execute(editor)?;
        Ok(Some(new_cmd))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::Editor;

    #[test]
    fn join_two_lines() {
        let mut editor = Editor::new();
        editor.terminal_size = crate::editor::TerminalSize { width: 80, height: 24 };
        editor.buffer.lines = vec!["foo".to_string(), "bar".to_string()];

        let mut cmd = JoinLines::default();
        cmd.execute(&mut editor).unwrap();
        assert_eq!(editor.buffer.lines, vec!["foobar".to_string()]);
        cmd.undo(&mut editor).unwrap();
        assert_eq!(editor.buffer.lines, vec!["foo".to_string(), "bar".to_string()]);
    }

    #[test]
    fn join_multiple_lines() {
        let mut editor = Editor::new();
        editor.terminal_size = crate::editor::TerminalSize { width: 80, height: 24 };
        editor.buffer.lines = vec!["1".to_string(), "2".to_string(), "3".to_string()];
        let mut cmd = JoinLines::default();
        cmd.execute(&mut editor).unwrap();
        assert_eq!(editor.buffer.lines, vec!["12".to_string(), "3".to_string()]);
        cmd.redo(&mut editor).unwrap();
        assert_eq!(editor.buffer.lines, vec!["123".to_string()]);
    }
}
