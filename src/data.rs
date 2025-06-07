#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    Colon,
    Command,
    Option,
    Number,
    Symbol,
    Pattern,
    AddressPattern,
    Replacement,
    Filename,
    Separator,
    EndOfInput,
    Illegal,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Pattern {
    pub pattern: String,
}

#[derive(Debug, PartialEq, Clone)]
pub enum SimpleLineAddressType {
    LineNumber(usize),
    CurrentLine,
    FirstLine,
    LastLine,
    AllLines,
    Pattern(Pattern)
}

#[derive(Debug, PartialEq, Clone)]
pub enum LineAddressType {
    Absolute(SimpleLineAddressType),
    #[allow(dead_code)]
    Relative(SimpleLineAddressType, isize),
}

#[derive(Debug, PartialEq, Clone)]
pub struct LineRange {
    pub start: LineAddressType,
    pub end: LineAddressType,
}
