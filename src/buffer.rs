use std::{fs, io::Write, path::PathBuf};

use tempfile::NamedTempFile;

use crate::generic_error::GenericResult;

pub struct Buffer {
    pub lines: Vec<String>,
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer { lines: Vec::new() }
    }

    pub fn from_file(file_path: &PathBuf) -> Buffer {
        let lines = std::fs::read_to_string(file_path)
            .expect("Failed to read file")
            .lines()
            .map(|s| s.to_string())
            .collect();
        Buffer { lines }
    }

    pub fn to_file(&self, file_path: &PathBuf) -> GenericResult<()> {
        let mut temp_file = NamedTempFile::new()?;
        for line in &self.lines {
            temp_file.write_all(line.as_bytes())?;
            temp_file.write_all(b"\n")?;
        }
        temp_file.flush()?;

        if file_path.exists() {
            let permissions = fs::metadata(file_path)?.permissions();
            fs::set_permissions(temp_file.path(), permissions)?;
        }

        std::fs::rename(temp_file.path(), file_path)?;
        Ok(())
    }

    pub fn insert_char(&mut self, row: usize, col: usize, c: char) -> GenericResult<()> {
        let new_line = self.lines[row]
            .chars()
            .take(col)
            .chain(std::iter::once(c))
            .chain(self.lines[row].chars().skip(col))
            .collect();
        self.lines[row] = new_line;
        Ok(())
    }
}
