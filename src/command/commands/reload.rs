use std::any::Any;

use crate::command::base::Command;
use crate::buffer::Buffer;
use crate::editor::Editor;
use crate::generic_error::GenericResult;

#[derive(Clone)]
pub struct ReloadFileCommand;

impl Command for ReloadFileCommand {
    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        if let Some(name) = editor.current_file_name() {
            let path = std::path::PathBuf::from(name);
            match Buffer::from_file(&path) {
                Ok(buffer) => {
                    editor.buffer = buffer;
                    editor.is_dirty = false;
                }
                Err(e) => {
                    editor.status_line = format!("Failed to open file: {}", e);
                }
            }
        }
        Ok(())
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
    fn reload_replaces_buffer() {
        let tmp = NamedTempFile::new().unwrap();
        write(tmp.path(), "one\n").unwrap();
        let mut editor = Editor::new();
        editor.open_file(&tmp.path().to_path_buf());
        editor.buffer.lines[0] = "changed".to_string();
        let mut cmd = ReloadFileCommand;
        cmd.execute(&mut editor).unwrap();
        assert_eq!(editor.buffer.lines[0], "one");
    }

    #[test]
    fn reload_missing_file_sets_status() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_path_buf();
        let mut editor = Editor::new();
        editor.open_file(&path);
        std::fs::remove_file(&path).unwrap();
        let mut cmd = ReloadFileCommand;
        cmd.execute(&mut editor).unwrap();
        assert!(editor.status_line.starts_with("Failed to open file"));
    }
}
