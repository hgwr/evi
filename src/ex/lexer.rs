#[derive(Debug, PartialEq)]
pub enum TokenType {
    Colon,
    Command,
    Option,
    Number,
    Pattern,
    AddressPattern,
    Replacement,
    Filename,
    Separator,
    EndOfInput,
    Illegal,
}

#[derive(Debug)]
pub struct Token {
    token_type: TokenType,
    lexeme: String,
}

#[derive(Debug, PartialEq)]
enum SubstitutionCommandState {
    None,
    Command,
    Pattern,
    Replace,
    Options,
    End,
}

struct Lexer {
    input: String,
    position: usize,
    current_char: Option<char>,
    substitution_command_status: SubstitutionCommandState,
}

impl Lexer {
    fn new(input: String) -> Self {
        let mut lexer = Lexer {
            input,
            position: 0,
            current_char: None,
            substitution_command_status: SubstitutionCommandState::None,
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
            self.current_char = Some(self.input.chars().nth(self.position - 1).unwrap());
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
                    token_type: TokenType::Command,
                    lexeme: ch.to_string(),
                }],
                '0'..='9' => vec![self.read_number()],
                's' => self.read_substitution_command(),
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
        println!("token: {:?}", tokens);
        println!("current_char: {:?}", self.current_char);
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
        if lexeme == "s" {
            self.substitution_command_status = SubstitutionCommandState::Command;
        }
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
                        state = SubstitutionCommandState::Pattern;
                    } else {
                        break;
                    }
                }
                SubstitutionCommandState::Pattern => {
                    if c == '\\' {
                        escaped = !escaped;
                    } else if c == '/' && !escaped {
                        lexeme.push(c);
                        tokens.push(Token {
                            token_type: TokenType::Pattern,
                            lexeme,
                        });
                        lexeme = String::new();
                        state = SubstitutionCommandState::Replace;
                    } else {
                        escaped = false;
                        lexeme.push(c);
                    }
                }
                SubstitutionCommandState::Replace => {
                    if c == '\\' {
                        escaped = !escaped;
                    } else if c == '/' && !escaped {
                        lexeme.push(c);
                        tokens.push(Token {
                            token_type: TokenType::Replacement,
                            lexeme,
                        });
                        lexeme = String::new();
                        state = SubstitutionCommandState::Options;
                    } else {
                        escaped = false;
                        lexeme.push(c);
                    }
                }
                SubstitutionCommandState::Options => {
                    if c == 'g' {
                        lexeme.push(c);
                        tokens.push(Token {
                            token_type: TokenType::Option,
                            lexeme,
                        });
                        lexeme = String::new();
                        state = SubstitutionCommandState::End;
                    } else {
                        break;
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
        self.rewind_char();
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
        println!("tokens: {:?}", tokens); // Add this line to print the value of tokens
        assert_eq!(tokens.len(), 2, "tokens: {:?}", tokens);
        assert_eq!(tokens[0].token_type, TokenType::Colon);
        assert_eq!(tokens[0].lexeme, ":");
        assert_eq!(tokens[1].token_type, TokenType::Command);
        assert_eq!(tokens[1].lexeme, "q");
    }

    #[test]
    fn test_tokenize_number_separator_command() {
        let input = ":1,23p";
        let tokens = tokenize(input);
        println!("tokens: {:?}", tokens); // Add this line to print the value of tokens
        assert_eq!(tokens.len(), 5, "tokens: {:?}", tokens);
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
    }

    #[test]
    fn test_tokenize_colon_number_separator_command_pattern_command() {
        let input = ":1,23s/screen\\/slash/line/g";
        let tokens = tokenize(input);
        assert_eq!(tokens.len(), 8, "tokens: {:?}", tokens);
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
        assert_eq!(tokens[5].lexeme, "screen\\/slash");
        assert_eq!(tokens[6].token_type, TokenType::Replacement);
        assert_eq!(tokens[6].lexeme, "line");
        assert_eq!(tokens[7].token_type, TokenType::Option);
        assert_eq!(tokens[7].lexeme, "g");
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
        assert_eq!(tokens[3].token_type, TokenType::Command);
        assert_eq!(tokens[3].lexeme, "$");
        assert_eq!(tokens[4].token_type, TokenType::Command);
        assert_eq!(tokens[4].lexeme, "m");
        assert_eq!(tokens[5].token_type, TokenType::Command);
        assert_eq!(tokens[5].lexeme, ".");
        assert_eq!(tokens[6].token_type, TokenType::Command);
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
        assert_eq!(tokens[1].token_type, TokenType::Command);
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
        assert_eq!(tokens.len(), 7);
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
        assert_eq!(tokens[5].token_type, TokenType::Pattern);
        assert_eq!(tokens[5].lexeme, "pattern");
        assert_eq!(tokens[6].token_type, TokenType::Command);
        assert_eq!(tokens[6].lexeme, "p");
    }
}
