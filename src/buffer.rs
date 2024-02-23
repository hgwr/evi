use std::path::PathBuf;

pub struct Buffer {
  lines: Vec<String>,
}

impl Buffer {
  pub fn new() -> Buffer {
    Buffer {
      lines: Vec::new(),
    }
  }

  pub fn from_file(file_path: &PathBuf) -> Buffer {
    let filename = file_path.clone();
    let lines = std::fs::read_to_string(file_path)
      .expect("Failed to read file")
      .lines()
      .map(|s| s.to_string())
      .collect();
    Buffer { lines }
  }

  pub fn to_string(&self) -> String {
    self.lines.join("\n")
  }
}
