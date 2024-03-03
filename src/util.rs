use unicode_width::UnicodeWidthChar;

pub fn get_char_width(c: char) -> u16 {
    UnicodeWidthChar::width(c).unwrap_or(0) as u16
}
