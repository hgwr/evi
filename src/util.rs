use unicode_width::UnicodeWidthChar;

pub fn get_char_width(c: char) -> u16 {
    UnicodeWidthChar::width(c).unwrap_or(0) as u16
}

pub fn split_line(input: &str) -> Vec<&str> {
    input.split('\n').collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_char_width() {
        assert_eq!(get_char_width('a'), 1);
        assert_eq!(get_char_width('あ'), 2);
        assert_eq!(get_char_width('🍣'), 2);
    }

    #[test]
    fn test_split_line() {
        assert_eq!(split_line("\n"), vec!["", ""]);
        assert_eq!(split_line("\n\n"), vec!["", "", ""]);
        assert_eq!(split_line("a\nb\nc"), vec!["a", "b", "c"]);
        assert_eq!(split_line("a\nb\nc\n"), vec!["a", "b", "c", ""]);
        assert_eq!(split_line("a\nb\nc\n\n"), vec!["a", "b", "c", "", ""]);
    }
}
