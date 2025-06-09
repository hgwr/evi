use std::any::Any;
use std::path::PathBuf;

use crate::buffer::Buffer;
use crate::command::base::Command;
use crate::data::{LineAddressType, SimpleLineAddressType};
use crate::editor::{Editor, EditorCursorData};
use crate::generic_error::GenericResult;

#[derive(Clone)]
pub struct ReadFileCommand {
    pub address: LineAddressType,
    pub filename: String,
    pub inserted_idx: Option<usize>,
    pub inserted_len: usize,
    pub editor_cursor_data: Option<EditorCursorData>,
}

impl Command for ReadFileCommand {
    fn is_undoable(&self) -> bool {
        true
    }

    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        let path = PathBuf::from(&self.filename);
        let buffer = match Buffer::from_file(&path) {
            Ok(buf) => buf,
            Err(e) => {
                editor.status_line = format!("Failed to read file: {}", e);
                return Ok(());
            }
        };
        self.editor_cursor_data = Some(editor.snapshot_cursor_data());
        let buffer_len = editor.buffer.lines.len();
        let line_num = editor.get_line_number_from(&self.address);
        let base_idx = if buffer_len == 0 {
            0
        } else if matches!(
            self.address,
            LineAddressType::Absolute(SimpleLineAddressType::LineNumber(0))
        ) {
            0
        } else {
            std::cmp::min(line_num, buffer_len - 1) + 1
        };
        let lines = buffer.lines;
        self.inserted_len = lines.len();
        self.inserted_idx = Some(base_idx);
        editor.buffer.lines.splice(base_idx..base_idx, lines.into_iter());
        Ok(())
    }

    fn undo(&mut self, editor: &mut Editor) -> GenericResult<()> {
        if let Some(idx) = self.inserted_idx {
            let end = (idx + self.inserted_len).min(editor.buffer.lines.len());
            editor.buffer.lines.drain(idx..end);
        }
        if let Some(cursor) = self.editor_cursor_data {
            editor.restore_cursor_data(cursor);
        }
        Ok(())
    }

    fn redo(&mut self, editor: &mut Editor) -> GenericResult<Option<Box<dyn Command>>> {
        let mut new_cmd = Box::new(ReadFileCommand {
            address: self.address.clone(),
            filename: self.filename.clone(),
            inserted_idx: None,
            inserted_len: 0,
            editor_cursor_data: None,
        });
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
    use std::fs::write;
    use tempfile::NamedTempFile;

    #[test]
    fn read_file_inserts_lines() {
        let base = NamedTempFile::new().unwrap();
        write(base.path(), "one\n").unwrap();
        let extra = NamedTempFile::new().unwrap();
        write(extra.path(), "a\nb\n").unwrap();

        let mut editor = Editor::new();
        editor.open_file(&base.path().to_path_buf());
        let mut cmd = ReadFileCommand {
            address: LineAddressType::Absolute(SimpleLineAddressType::CurrentLine),
            filename: extra.path().to_string_lossy().to_string(),
            inserted_idx: None,
            inserted_len: 0,
            editor_cursor_data: None,
        };
        cmd.execute(&mut editor).unwrap();
        assert_eq!(editor.buffer.lines, vec!["one".to_string(), "a".to_string(), "b".to_string()]);
    }

    #[test]
    fn read_file_missing_sets_status() {
        let base = NamedTempFile::new().unwrap();
        write(base.path(), "one\n").unwrap();
        let mut editor = Editor::new();
        editor.open_file(&base.path().to_path_buf());
        let path = base.path().with_extension("missing");
        let mut cmd = ReadFileCommand {
            address: LineAddressType::Absolute(SimpleLineAddressType::CurrentLine),
            filename: path.to_string_lossy().to_string(),
            inserted_idx: None,
            inserted_len: 0,
            editor_cursor_data: None,
        };
        cmd.execute(&mut editor).unwrap();
        assert!(editor.status_line.starts_with("Failed to read file"));
    }
}
