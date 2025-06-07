use crate::data::TokenType;
use crate::data::Token;

#[derive(Debug, PartialEq)]
enum SubstitutionCommandState {
    #[allow(dead_code)]
    None,
    Command,
    FirstSeparator,
    Pattern,
    SecondSeparator,
    Replace,
    ThirdSeparator,
    Options,
    End,
}

#[derive(Debug, PartialEq)]
enum FileCommandState {
    None,
    Command,
    Filename,
    #[allow(dead_code)]
    End,
}

struct Lexer {
    input: String,
    position: usize,
    current_char: Option<char>,
}

impl Lexer {
    fn new(input: String) -> Self {
        let mut lexer = Lexer {
            input,
            position: 0,
            current_char: None,
        };
        lexer.read_char();
        lexer
    }

    fn read_char(&mut self) {
        if self.position >= self.input.len() {
            self.current_char = None;
        } else {
            self.current_char = Some(self.input.chars().nth(self.position).unwrap());
        }
        self.position += 1;
    }

    #[allow(dead_code)]
    fn peek_char(&self) -> Option<char> {
        if self.position >= self.input.len() {
            None
        } else {
            Some(self.input.chars().nth(self.position).unwrap())
        }
    }

    fn rewind_char(&mut self) {
        if self.position > 0 {
            self.position -= 1;
            if self.position < self.input.chars().count() {
                self.current_char = Some(self.input.chars().nth(self.position).unwrap());
            } else {
                self.current_char = None;
            }
        }
    }

    fn next_tokens(&mut self) -> Vec<Token> {
        self.skip_whitespace();
        let tokens: Vec<Token> = match self.current_char {
            Some(ch) => match ch {
                ':' => vec![Token {
                    token_type: TokenType::Colon,
                    lexeme: ch.to_string(),
                }],
                ',' => vec![Token {
                    token_type: TokenType::Separator,
                    lexeme: ch.to_string(),
                }],
                '/' => vec![self.read_pattern()],
                '!' | '#' | '=' | '.' | '-' | '+' | '*' | '%' | '$' | '^' => vec![Token {
                    token_type: TokenType::Symbol,
                    lexeme: ch.to_string(),
                }],
                '0'..='9' => vec![self.read_number()],
                's' => self.read_substitution_command(),
                'r' | 'w' => self.file_command(),
                _ if ch.is_alphabetic() => vec![self.read_command()],
                _ => vec![Token {
                    token_type: TokenType::Illegal,
                    lexeme: ch.to_string(),
                }],
            },
            None => vec![Token {
                token_type: TokenType::EndOfInput,
                lexeme: "".to_string(),
            }],
        };
        self.read_char();
        tokens
    }

    fn read_number(&mut self) -> Token {
        let mut number = String::new();
        while let Some(ch) = self.current_char {
            match ch {
                '0'..='9' => {
                    number.push(ch);
                    self.read_char();
                }
                _ => {
                    self.rewind_char();
                    break;
                }
            }
        }
        Token {
            token_type: TokenType::Number,
            lexeme: number,
        }
    }

    fn read_command(&mut self) -> Token {
        let start = self.position - 1;
        while let Some(c) = self.current_char {
            if c.is_alphabetic() {
                self.read_char();
            } else {
                break;
            }
        }
        let lexeme: String = self.input[start..self.position - 1].to_string();
        self.rewind_char();
        Token {
            token_type: TokenType::Command,
            lexeme,
        }
    }

    fn read_pattern(&mut self) -> Token {
        self.read_char(); // skip initial '/'
        let start = self.position - 1;
        let mut escaped = false;
        while let Some(c) = self.current_char {
            if c == '\\' {
                escaped = !escaped;
            } else if c == '/' && !escaped {
                break;
            } else {
                escaped = false;
            }
            self.read_char();
        }
        let lexeme: String = self.input[start..self.position - 1].to_string();

        Token {
            token_type: TokenType::AddressPattern,
            lexeme,
        }
    }

    fn read_substitution_command(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut lexeme = String::new();
        let mut state = SubstitutionCommandState::Command;
        let mut escaped = false;
        while let Some(c) = self.current_char {
            match state {
                SubstitutionCommandState::Command => {
                    if c == 's' {
                        lexeme.push(c);
                        tokens.push(Token {
                            token_type: TokenType::Command,
                            lexeme,
                        });
                        lexeme = String::new();
                        state = SubstitutionCommandState::FirstSeparator;
                    } else {
                        break;
                    }
                }
                SubstitutionCommandState::FirstSeparator => {
                    if c == '/' {
                        state = SubstitutionCommandState::Pattern;
                    } else {
                        break;
                    }
                }
                SubstitutionCommandState::Pattern => {
                    if c == '\\' {
                        escaped = !escaped;
                    } else if c == '/' && !escaped {
                        self.rewind_char();
                        tokens.push(Token {
                            token_type: TokenType::Pattern,
                            lexeme,
                        });
                        lexeme = String::new();
                        state = SubstitutionCommandState::SecondSeparator;
                    } else {
                        escaped = false;
                        lexeme.push(c);
                    }
                }
                SubstitutionCommandState::SecondSeparator => {
                    if c == '/' {
                        state = SubstitutionCommandState::Replace;
                    } else {
                        break;
                    }
                }
                SubstitutionCommandState::Replace => {
                    if c == '\\' {
                        escaped = !escaped;
                    } else if c == '/' && !escaped {
                        self.rewind_char();
                        tokens.push(Token {
                            token_type: TokenType::Replacement,
                            lexeme,
                        });
                        lexeme = String::new();
                        state = SubstitutionCommandState::ThirdSeparator;
                    } else {
                        escaped = false;
                        lexeme.push(c);
                    }
                }
                SubstitutionCommandState::ThirdSeparator => {
                    if c == '/' {
                        state = SubstitutionCommandState::Options;
                    } else {
                        break;
                    }
                }
                SubstitutionCommandState::Options => {
                    if c == 'g' || c == 'i' {
                        lexeme.push(c);
                    } else {
                        if !lexeme.is_empty() {
                            tokens.push(Token {
                                token_type: TokenType::Option,
                                lexeme,
                            });
                            lexeme = String::new();
                        }
                        self.rewind_char();
                        state = SubstitutionCommandState::End;
                    }
                }
                SubstitutionCommandState::End => {
                    break;
                }
                SubstitutionCommandState::None => {
                    break;
                }
            }
            self.read_char();
        }
        if state == SubstitutionCommandState::Options && !lexeme.is_empty() {
            tokens.push(Token {
                token_type: TokenType::Option,
                lexeme,
            });
        }
        self.rewind_char();
        tokens
    }

    fn file_command(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut lexeme = String::new();
        let mut file_command_status = FileCommandState::None;
        while let Some(c) = self.current_char {
            match file_command_status {
                FileCommandState::None => {
                    if c == 'r' || c == 'w' {
                        lexeme.push(c);
                        tokens.push(Token {
                            token_type: TokenType::Command,
                            lexeme,
                        });
                        lexeme = String::new();
                    } else {
                        break;
                    }
                    file_command_status = FileCommandState::Command;
                }
                FileCommandState::Command => {
                    if c.is_whitespace() {
                        file_command_status = FileCommandState::Filename;
                    } else {
                        self.rewind_char();
                        break;
                    }
                }
                FileCommandState::Filename => {
                    lexeme.push(c);
                }
                FileCommandState::End => {
                    break;
                }
            }
            self.read_char();
        }
        if file_command_status == FileCommandState::Filename {
            tokens.push(Token {
                token_type: TokenType::Filename,
                lexeme,
            });
        }
        tokens
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.current_char {
            if c.is_whitespace() {
                self.read_char();
            } else {
                break;
            }
        }
    }
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut lexer = Lexer::new(input.to_string());
    let mut tokens = Vec::new();

    loop {
        let mut next_tokens = lexer.next_tokens();
        tokens.append(&mut next_tokens);
        if tokens.last().unwrap().token_type == TokenType::EndOfInput {
            break;
        }
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_colon_command() {
        let input = ":q";
        let tokens = tokenize(input);
        assert_eq!(tokens.len(), 3, "tokens: {:?}", tokens);
        assert_eq!(tokens[0].token_type, TokenType::Colon);
        assert_eq!(tokens[0].lexeme, ":");
        assert_eq!(tokens[1].token_type, TokenType::Command);
        assert_eq!(tokens[1].lexeme, "q");
        assert_eq!(tokens[2].token_type, TokenType::EndOfInput);
    }

    #[test]
    fn test_tokenize_number_command() {
        let input = "1p";
        let tokens = tokenize(input);
        assert_eq!(tokens.len(), 3, "tokens: {:?}", tokens);
        assert_eq!(tokens[0].token_type, TokenType::Number);
        assert_eq!(tokens[0].lexeme, "1");
        assert_eq!(tokens[1].token_type, TokenType::Command);
        assert_eq!(tokens[1].lexeme, "p");
        assert_eq!(tokens[2].token_type, TokenType::EndOfInput);
    }

    #[test]
    fn test_tokenize_number_separator_command() {
        let input = ":1,23p";
        let tokens = tokenize(input);
        assert_eq!(tokens.len(), 6, "tokens: {:?}", tokens);
        assert_eq!(tokens[0].token_type, TokenType::Colon);
        assert_eq!(tokens[0].lexeme, ":");
        assert_eq!(tokens[1].token_type, TokenType::Number);
        assert_eq!(tokens[1].lexeme, "1");
        assert_eq!(tokens[2].token_type, TokenType::Separator);
        assert_eq!(tokens[2].lexeme, ",");
        assert_eq!(tokens[3].token_type, TokenType::Number);
        assert_eq!(tokens[3].lexeme, "23");
        assert_eq!(tokens[4].token_type, TokenType::Command);
        assert_eq!(tokens[4].lexeme, "p");
        assert_eq!(tokens[5].token_type, TokenType::EndOfInput);
    }

    #[test]
    fn test_tokenize_colon_number_separator_command_pattern_command() {
        let input = ":1,23s/screen\\/slash/line/g";
        let tokens = tokenize(input);
        assert_eq!(tokens.len(), 9, "tokens: {:?}", tokens);
        assert_eq!(tokens[0].token_type, TokenType::Colon);
        assert_eq!(tokens[0].lexeme, ":");
        assert_eq!(tokens[1].token_type, TokenType::Number);
        assert_eq!(tokens[1].lexeme, "1");
        assert_eq!(tokens[2].token_type, TokenType::Separator);
        assert_eq!(tokens[2].lexeme, ",");
        assert_eq!(tokens[3].token_type, TokenType::Number);
        assert_eq!(tokens[3].lexeme, "23");
        assert_eq!(tokens[4].token_type, TokenType::Command);
        assert_eq!(tokens[4].lexeme, "s");
        assert_eq!(tokens[5].token_type, TokenType::Pattern);
        assert_eq!(tokens[5].lexeme, "screen/slash");
        assert_eq!(tokens[6].token_type, TokenType::Replacement);
        assert_eq!(tokens[6].lexeme, "line");
        assert_eq!(tokens[7].token_type, TokenType::Option);
        assert_eq!(tokens[7].lexeme, "g");
        assert_eq!(tokens[8].token_type, TokenType::EndOfInput);
    }

    #[test]
    fn test_tokenize_colon_number_command_command_command() {
        let input = ":10,$m.-2";
        let tokens = tokenize(input);
        assert_eq!(tokens.len(), 9);
        assert_eq!(tokens[0].token_type, TokenType::Colon);
        assert_eq!(tokens[0].lexeme, ":");
        assert_eq!(tokens[1].token_type, TokenType::Number);
        assert_eq!(tokens[1].lexeme, "10");
        assert_eq!(tokens[2].token_type, TokenType::Separator);
        assert_eq!(tokens[2].lexeme, ",");
        assert_eq!(tokens[3].token_type, TokenType::Symbol);
        assert_eq!(tokens[3].lexeme, "$");
        assert_eq!(tokens[4].token_type, TokenType::Command);
        assert_eq!(tokens[4].lexeme, "m");
        assert_eq!(tokens[5].token_type, TokenType::Symbol);
        assert_eq!(tokens[5].lexeme, ".");
        assert_eq!(tokens[6].token_type, TokenType::Symbol);
        assert_eq!(tokens[6].lexeme, "-");
        assert_eq!(tokens[7].token_type, TokenType::Number);
        assert_eq!(tokens[7].lexeme, "2");
        assert_eq!(tokens[8].token_type, TokenType::EndOfInput);
    }

    #[test]
    fn test_tokenize_colon_command_separator_pattern_command() {
        let input = ":.,/while/d";
        let tokens = tokenize(input);
        assert_eq!(tokens.len(), 6, "tokens: {:?}", tokens);
        assert_eq!(tokens[0].token_type, TokenType::Colon);
        assert_eq!(tokens[0].lexeme, ":");
        assert_eq!(tokens[1].token_type, TokenType::Symbol);
        assert_eq!(tokens[1].lexeme, ".");
        assert_eq!(tokens[2].token_type, TokenType::Separator);
        assert_eq!(tokens[2].lexeme, ",");
        assert_eq!(tokens[3].token_type, TokenType::AddressPattern);
        assert_eq!(tokens[3].lexeme, "while");
        assert_eq!(tokens[4].token_type, TokenType::Command);
        assert_eq!(tokens[4].lexeme, "d");
        assert_eq!(tokens[5].token_type, TokenType::EndOfInput);
    }

    #[test]
    fn test_tokenize_colon_number_separator_command_pattern_command2() {
        let input = ":1,10g/pattern/p";
        let tokens = tokenize(input);
        assert_eq!(tokens.len(), 8);
        assert_eq!(tokens[0].token_type, TokenType::Colon);
        assert_eq!(tokens[0].lexeme, ":");
        assert_eq!(tokens[1].token_type, TokenType::Number);
        assert_eq!(tokens[1].lexeme, "1");
        assert_eq!(tokens[2].token_type, TokenType::Separator);
        assert_eq!(tokens[2].lexeme, ",");
        assert_eq!(tokens[3].token_type, TokenType::Number);
        assert_eq!(tokens[3].lexeme, "10");
        assert_eq!(tokens[4].token_type, TokenType::Command);
        assert_eq!(tokens[4].lexeme, "g");
        assert_eq!(tokens[5].token_type, TokenType::AddressPattern);
        assert_eq!(tokens[5].lexeme, "pattern");
        assert_eq!(tokens[6].token_type, TokenType::Command);
        assert_eq!(tokens[6].lexeme, "p");
        assert_eq!(tokens[7].token_type, TokenType::EndOfInput);
    }

    #[test]
    fn test_tokenize_substitute_ignore_case() {
        let input = "1,5s/^abc/cba/i";
        let tokens = tokenize(input);
        assert_eq!(tokens.len(), 8, "tokens: {:?}", tokens);
        assert_eq!(tokens[0].token_type, TokenType::Number);
        assert_eq!(tokens[0].lexeme, "1");
        assert_eq!(tokens[1].token_type, TokenType::Separator);
        assert_eq!(tokens[1].lexeme, ",");
        assert_eq!(tokens[2].token_type, TokenType::Number);
        assert_eq!(tokens[2].lexeme, "5");
        assert_eq!(tokens[3].token_type, TokenType::Command);
        assert_eq!(tokens[3].lexeme, "s");
        assert_eq!(tokens[4].token_type, TokenType::Pattern);
        assert_eq!(tokens[4].lexeme, "^abc");
        assert_eq!(tokens[5].token_type, TokenType::Replacement);
        assert_eq!(tokens[5].lexeme, "cba");
        assert_eq!(tokens[6].token_type, TokenType::Option);
        assert_eq!(tokens[6].lexeme, "i");
        assert_eq!(tokens[7].token_type, TokenType::EndOfInput);
    }

    #[test]
    fn test_tokenize_substitute_global_all_lines() {
        let input = "%s/^abc/cba/g";
        let tokens = tokenize(input);
        assert_eq!(tokens.len(), 6, "tokens: {:?}", tokens);
        assert_eq!(tokens[0].token_type, TokenType::Symbol);
        assert_eq!(tokens[0].lexeme, "%");
        assert_eq!(tokens[1].token_type, TokenType::Command);
        assert_eq!(tokens[1].lexeme, "s");
        assert_eq!(tokens[2].token_type, TokenType::Pattern);
        assert_eq!(tokens[2].lexeme, "^abc");
        assert_eq!(tokens[3].token_type, TokenType::Replacement);
        assert_eq!(tokens[3].lexeme, "cba");
        assert_eq!(tokens[4].token_type, TokenType::Option);
        assert_eq!(tokens[4].lexeme, "g");
        assert_eq!(tokens[5].token_type, TokenType::EndOfInput);
    }

    #[test]
    fn test_tokenize_substitute_line_range_last() {
        let input = "1,$s/cde$/CDE/";
        let tokens = tokenize(input);
        assert_eq!(tokens.len(), 7, "tokens: {:?}", tokens);
        assert_eq!(tokens[0].token_type, TokenType::Number);
        assert_eq!(tokens[0].lexeme, "1");
        assert_eq!(tokens[1].token_type, TokenType::Separator);
        assert_eq!(tokens[1].lexeme, ",");
        assert_eq!(tokens[2].token_type, TokenType::Symbol);
        assert_eq!(tokens[2].lexeme, "$");
        assert_eq!(tokens[3].token_type, TokenType::Command);
        assert_eq!(tokens[3].lexeme, "s");
        assert_eq!(tokens[4].token_type, TokenType::Pattern);
        assert_eq!(tokens[4].lexeme, "cde$");
        assert_eq!(tokens[5].token_type, TokenType::Replacement);
        assert_eq!(tokens[5].lexeme, "CDE");
        assert_eq!(tokens[6].token_type, TokenType::EndOfInput);
    }

    #[test]
    fn test_tokenize_read_file() {
        let input = ":r file.txt";
        let tokens = tokenize(input);
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0].token_type, TokenType::Colon);
        assert_eq!(tokens[0].lexeme, ":");
        assert_eq!(tokens[1].token_type, TokenType::Command);
        assert_eq!(tokens[1].lexeme, "r");
        assert_eq!(tokens[2].token_type, TokenType::Filename);
        assert_eq!(tokens[2].lexeme, "file.txt");
        assert_eq!(tokens[3].token_type, TokenType::EndOfInput);
    }

    #[test]
    fn test_tokenize_read_file_with_spaces() {
        let input = ":r file with spaces.txt";
        let tokens = tokenize(input);
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0].token_type, TokenType::Colon);
        assert_eq!(tokens[0].lexeme, ":");
        assert_eq!(tokens[1].token_type, TokenType::Command);
        assert_eq!(tokens[1].lexeme, "r");
        assert_eq!(tokens[2].token_type, TokenType::Filename);
        assert_eq!(tokens[2].lexeme, "file with spaces.txt");
        assert_eq!(tokens[3].token_type, TokenType::EndOfInput);
    }

    #[test]
    fn test_tokenize_q() {
        let input = ":q";
        let tokens = tokenize(input);
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].token_type, TokenType::Colon);
        assert_eq!(tokens[0].lexeme, ":");
        assert_eq!(tokens[1].token_type, TokenType::Command);
        assert_eq!(tokens[1].lexeme, "q");
        assert_eq!(tokens[2].token_type, TokenType::EndOfInput);
    }

    #[test]
    fn test_tokenize_wq() {
        let input = ":wq";
        let tokens = tokenize(input);
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0].token_type, TokenType::Colon);
        assert_eq!(tokens[0].lexeme, ":");
        assert_eq!(tokens[1].token_type, TokenType::Command);
        assert_eq!(tokens[1].lexeme, "w");
        assert_eq!(tokens[2].token_type, TokenType::Command);
        assert_eq!(tokens[2].lexeme, "q");
        assert_eq!(tokens[3].token_type, TokenType::EndOfInput);
    }

    #[test]
    fn test_tokenize_print_with_line_addresses() {
        let input = ":1,2p";
        let tokens = tokenize(input);
        assert_eq!(tokens.len(), 6);
        assert_eq!(tokens[0].token_type, TokenType::Colon);
        assert_eq!(tokens[0].lexeme, ":");
        assert_eq!(tokens[1].token_type, TokenType::Number);
        assert_eq!(tokens[1].lexeme, "1");
        assert_eq!(tokens[2].token_type, TokenType::Separator);
        assert_eq!(tokens[2].lexeme, ",");
        assert_eq!(tokens[3].token_type, TokenType::Number);
        assert_eq!(tokens[3].lexeme, "2");
        assert_eq!(tokens[4].token_type, TokenType::Command);
        assert_eq!(tokens[4].lexeme, "p");
        assert_eq!(tokens[5].token_type, TokenType::EndOfInput);
    }
}
