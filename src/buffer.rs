use std::{io::Write, path::PathBuf};

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

    pub fn to_file(&self, file_path: &PathBuf) {
        let file = std::fs::File::create(file_path).expect("Failed to create file");
        let mut writer = std::io::BufWriter::new(file);
        for line in &self.lines {
            writer
                .write_all(line.as_bytes())
                .expect("Failed to write file");
            writer.write_all(b"\n").expect("Failed to write file");
        }
    }

    pub fn insert_char(&mut self, row: usize, col: usize, c: char) -> GenericResult<()>{
        let new_line = self.lines[row].chars().take(col).chain(std::iter::once(c)).chain(self.lines[row].chars().skip(col)).collect();
        self.lines[row] = new_line;
        Ok(())
    }
}
