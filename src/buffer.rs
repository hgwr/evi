use std::{fs, io::Write, path::PathBuf};

use tempfile::NamedTempFile;

use crate::{generic_error::GenericResult, util::split_line};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CursorPositionInBuffer {
    pub row: usize,
    pub col: usize,
}

impl CursorPositionInBuffer {
    pub fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.row < other.row {
            std::cmp::Ordering::Less
        } else if self.row > other.row {
            std::cmp::Ordering::Greater
        } else {
            self.col.cmp(&other.col)
        }
    }
}

impl PartialOrd for CursorPositionInBuffer {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

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

    pub fn insert(&mut self, row: usize, col: usize, s: &str) -> GenericResult<()> {
        let lines_to_be_inserted = split_line(s);
        if lines_to_be_inserted.len() == 0 {
            panic!("lines.len() == 0, s: '{:?}'", s);
        }
        if lines_to_be_inserted.len() == 1 {
            let new_line = self.lines[row]
                .chars()
                .take(col)
                .chain(lines_to_be_inserted[0].chars())
                .chain(self.lines[row].chars().skip(col))
                .collect();
            self.lines[row] = new_line;
        } else if lines_to_be_inserted.len() >= 2 {
            let new_first_line: String = self.lines[row].chars().take(col).collect();
            let input_last_line: String = lines_to_be_inserted[lines_to_be_inserted.len() - 1].to_string();
            let new_last_line: String = input_last_line + &self.lines[row].chars().skip(col).collect::<String>();
            self.lines[row] = new_first_line + lines_to_be_inserted[0];
            for i in 1..lines_to_be_inserted.len() - 1 {
                self.lines.insert(row + i, lines_to_be_inserted[i].to_string());
            }
            self.lines.insert(row + lines_to_be_inserted.len() - 1, new_last_line);
        }
        Ok(())
    }

    pub fn delete_char(&mut self, row: usize, col: usize) -> GenericResult<()> {
        let new_line = self.lines[row]
            .chars()
            .take(col)
            .chain(self.lines[row].chars().skip(col + 1))
            .collect();
        self.lines[row] = new_line;
        Ok(())
    }

    pub fn get_char(&self, row: usize, col: usize) -> Option<char> {
        self.lines.get(row)?.chars().nth(col)
    }

    pub fn delete(
        &mut self,
        mut start: CursorPositionInBuffer,
        mut end: CursorPositionInBuffer,
    ) -> GenericResult<String> {
        if start.cmp(&end) == std::cmp::Ordering::Greater {
            let tmp = start;
            start = end;
            end = tmp;
        }
        if start.row == end.row {
            let line = &mut self.lines[start.row];
            let deleted: String = line
                .chars()
                .skip(start.col)
                .take(end.col - start.col)
                .collect();
            let new_line: String = line
                .chars()
                .take(start.col)
                .chain(line.chars().skip(end.col))
                .collect();
            *line = new_line;
            Ok(deleted)
        } else {
            let first_line = self.lines[start.row].clone();
            let last_line = self.lines[end.row].clone();
            let new_first_line: String = first_line.chars().take(start.col).collect();
            let first_line_deleted: String = first_line.chars().skip(start.col).collect();
            let new_last_line: String = last_line.chars().skip(end.col).collect();
            let last_line_deleted: String = last_line.chars().take(end.col).collect();
            self.lines[start.row] = new_first_line + new_last_line.as_str();
            let mut deleted_chars = first_line_deleted;
            for i in 0..end.row - start.row {
                deleted_chars.push('\n');
                if i < end.row - start.row - 1 {
                    deleted_chars.push_str(&self.lines[start.row + 1]);
                }
                self.lines.remove(start.row + 1);
            }
            deleted_chars.push_str(&last_line_deleted);
            Ok(deleted_chars)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_position_in_buffer_cmp() {
        let a = CursorPositionInBuffer { row: 0, col: 0 };
        let b = CursorPositionInBuffer { row: 0, col: 1 };
        let c = CursorPositionInBuffer { row: 1, col: 0 };
        let d = CursorPositionInBuffer { row: 1, col: 1 };
        assert_eq!(a.cmp(&a), std::cmp::Ordering::Equal);
        assert_eq!(a.cmp(&b), std::cmp::Ordering::Less);
        assert_eq!(a.cmp(&c), std::cmp::Ordering::Less);
        assert_eq!(a.cmp(&d), std::cmp::Ordering::Less);
        assert_eq!(b.cmp(&a), std::cmp::Ordering::Greater);
        assert_eq!(b.cmp(&b), std::cmp::Ordering::Equal);
        assert_eq!(b.cmp(&c), std::cmp::Ordering::Less);
        assert_eq!(b.cmp(&d), std::cmp::Ordering::Less);
        assert_eq!(c.cmp(&a), std::cmp::Ordering::Greater);
        assert_eq!(c.cmp(&b), std::cmp::Ordering::Greater);
        assert_eq!(c.cmp(&c), std::cmp::Ordering::Equal);
        assert_eq!(c.cmp(&d), std::cmp::Ordering::Less);
        assert_eq!(d.cmp(&a), std::cmp::Ordering::Greater);
        assert_eq!(d.cmp(&b), std::cmp::Ordering::Greater);
        assert_eq!(d.cmp(&c), std::cmp::Ordering::Greater);
        assert_eq!(d.cmp(&d), std::cmp::Ordering::Equal);
    }

    #[test]
    fn test_buffer_insert_char() {
        let mut buffer = Buffer {
            lines: vec!["abc".to_string(), "def".to_string()],
        };
        buffer.insert_char(0, 1, 'x').unwrap();
        assert_eq!(buffer.lines, vec!["axbc".to_string(), "def".to_string()]);
    }

    #[test]
    fn test_buffer_delete_char() {
        let mut buffer = Buffer {
            lines: vec!["abc".to_string(), "def".to_string()],
        };
        buffer.delete_char(0, 1).unwrap();
        assert_eq!(buffer.lines, vec!["ac".to_string(), "def".to_string()]);
    }

    #[test]
    fn test_buffer_get_char() {
        let buffer = Buffer {
            lines: vec!["abc".to_string(), "def".to_string()],
        };
        assert_eq!(buffer.get_char(0, 1), Some('b'));
        assert_eq!(buffer.get_char(0, 3), None);
    }

    #[test]
    fn test_buffer_delete() {
        let mut buffer = Buffer {
            lines: vec!["abcdef".to_string()],
        };
        let deleted = buffer
            .delete(
                CursorPositionInBuffer { row: 0, col: 1 },
                CursorPositionInBuffer { row: 0, col: 4 },
            )
            .unwrap();
        assert_eq!(buffer.lines, vec!["aef".to_string()]);
        assert_eq!(deleted, "bcd");

        buffer = Buffer {
            lines: vec!["abc".to_string(), "def".to_string(), "ghi".to_string()],
        };
        let deleted = buffer
            .delete(
                CursorPositionInBuffer { row: 0, col: 1 },
                CursorPositionInBuffer { row: 1, col: 1 },
            )
            .unwrap();
        assert_eq!(buffer.lines, vec!["aef".to_string(), "ghi".to_string()]);
        assert_eq!(deleted, "bc\nd");
    }

    #[test]
    fn test_insert() {
        let mut buffer = Buffer {
            lines: vec!["abc".to_string(), "def".to_string()],
        };
        buffer.insert(0, 1, "x").unwrap();
        assert_eq!(buffer.lines, vec!["axbc".to_string(), "def".to_string()]);

        buffer = Buffer {
            lines: vec!["abc".to_string(), "def".to_string()],
        };
        buffer.insert(0, 1, "x\ny").unwrap();
        assert_eq!(buffer.lines, vec!["ax".to_string(), "ybc".to_string(), "def".to_string()]);
    }
}
