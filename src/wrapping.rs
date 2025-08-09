use crate::buffer::Buffer;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ScreenPosition {
    pub row: usize,
    pub col: u16,
}

impl ScreenPosition {
    pub fn new(row: usize, col: u16) -> Self { Self { row, col } }
}

pub struct WrappingCalculator {
    terminal_width: u16,
}

impl WrappingCalculator {
    pub fn new(terminal_width: u16) -> Self { Self { terminal_width } }

    pub fn line_height(&self, line: &str) -> usize {
        crate::util::get_line_height(line, self.terminal_width)
    }

    pub fn buffer_to_screen_position(
        &self,
        buffer: &Buffer,
        window_top_row: usize,
        target: crate::editor::BufferPosition,
    ) -> ScreenPosition {
        use crate::util::get_char_width;
        let mut screen_row = 0usize;
        for r in window_top_row..target.row {
            if let Some(line) = buffer.lines.get(r) { screen_row += self.line_height(line); } else { break; }
        }
        let mut col_width = 0usize;
        let mut wrap_row_offset = 0usize;
        if let Some(line) = buffer.lines.get(target.row) {
            for (i, c) in line.chars().enumerate() {
                if i >= target.col { break; }
                let w = get_char_width(c) as usize;
                if col_width + w >= self.terminal_width as usize {
                    wrap_row_offset += 1;
                    col_width = 0;
                }
                col_width += w;
                if col_width >= self.terminal_width as usize {
                    wrap_row_offset += 1;
                    col_width = 0;
                }
            }
        }
        ScreenPosition::new(screen_row + wrap_row_offset, col_width as u16)
    }
}
