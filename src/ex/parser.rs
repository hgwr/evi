use std::ops::BitOr;

use crate::command::base::Command;
use crate::command::commands::go_to_line;
use crate::data::LineAddressType;
use crate::data::LineRange;
use crate::data::Pattern;
use crate::data::SimpleLineAddressType;
use crate::data::Token;
use crate::data::TokenType;
use crate::ex::lexer;
use crate::generic_error::GenericError;

use crate::command::commands::exit;
use crate::command::commands::print;

enum MyOption<T> {
    Some(T),
    None,
}

impl<T> MyOption<T> {
    fn is_some(&self) -> bool {
        match self {
            MyOption::Some(_) => true,
            MyOption::None => false,
        }
    }
}

impl BitOr for MyOption<Box<dyn Command>> {
    type Output = MyOption<Box<dyn Command>>;

    fn bitor(self, rhs: Self) -> Self::Output {
        if self.is_some() {
            self
        } else {
            rhs
        }
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    original_tokens: Vec<Token>,
    token_opt: MyOption<Token>,
    stack: Vec<Token>,
    command_opt: MyOption<Box<dyn Command>>,
    line_range_opt: MyOption<LineRange>,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        let tokens = lexer::tokenize(input);
        println!("tokens {:?}", tokens);
        Parser {
            original_tokens: tokens.clone(),
            tokens,
            token_opt: MyOption::None,
            stack: Vec::new(),
            command_opt: MyOption::None,
            line_range_opt: MyOption::None,
        }
    }

    pub fn parse(&mut self) -> Result<Box<dyn Command>, GenericError> {
        self.get_symbol();
        self.command()
    }

    fn command(&mut self) -> Result<Box<dyn Command>, GenericError> {
        let command_opt = self.simple_command()? | self.complex_command()?;
        match command_opt {
            MyOption::Some(command) => Ok(command),
            MyOption::None => Err("Invalid command".to_string().into()),
        }
    }

    fn push(&mut self, token: Token) {
        self.stack.push(token);
    }

    fn pop(&mut self) -> MyOption<Token> {
        if self.stack.len() > 0 {
            return MyOption::Some(self.stack.pop().unwrap());
        }
        MyOption::None
    }

    fn get_symbol(&mut self) {
        if self.tokens.len() > 0 {
            self.token_opt = MyOption::Some(self.tokens.remove(0));
        } else {
            self.token_opt = MyOption::Some(Token {
                token_type: TokenType::EndOfInput,
                lexeme: "".to_string(),
            });
        }
    }

    fn undo_parse(&mut self) {
        self.tokens = self.original_tokens.clone();
        self.stack.clear();
        self.command_opt = MyOption::None;
        self.line_range_opt = MyOption::None;
        self.get_symbol();
    }

    fn error(&self, message: &str) -> GenericError {
        format!("Error: {}", message).into()
    }

    fn accept(&mut self, token_type: TokenType, lexeme: &str) -> bool {
        let expected_token = Token {
            token_type,
            lexeme: lexeme.to_string(),
        };
        if let MyOption::Some(token) = &self.token_opt {
            if token.token_type == expected_token.token_type
                && token.lexeme == expected_token.lexeme
            {
                self.push(token.clone());
                self.get_symbol();
                return true;
            }
        }
        false
    }

    fn accept_type(&mut self, token_type: TokenType) -> bool {
        if let MyOption::Some(token) = &self.token_opt {
            if token.token_type == token_type {
                self.push(token.clone());
                self.get_symbol();
                return true;
            }
        }
        false
    }

    fn complex_command(&mut self) -> Result<MyOption<Box<dyn Command>>, GenericError> {
        let line_range = if let MyOption::Some(range) = self.line_range()? {
            range
        } else {
            LineRange {
                start: LineAddressType::Absolute(SimpleLineAddressType::CurrentLine),
                end: LineAddressType::Absolute(SimpleLineAddressType::CurrentLine),
            }
        };
        let command_opt = self.display_command(line_range)? | self.substitute_command()?;
        if let MyOption::Some(command) = command_opt {
            return Ok(MyOption::Some(command));
        }
        Ok(MyOption::None)
    }

    fn display_command(&mut self, line_range: LineRange) -> Result<MyOption<Box<dyn Command>>, GenericError> {
        if self.accept(TokenType::Command, "p") {
            let print_command = print::PrintCommand {
                line_range
            };
            return Ok(MyOption::Some(Box::new(print_command)));
        }
        Ok(MyOption::None)
    }

    fn line_range(&mut self) -> Result<MyOption<LineRange>, GenericError> {
        // line_address "," line_address
        // | line_address "," pattern
        // | pattern "," line_address
        // | pattern "," pattern
        // | line_address
        // | pattern
        if let MyOption::Some(start_line_address) = self.line_address()? {
            if self.accept(TokenType::Separator, ",") {
                self.pop();
                if let MyOption::Some(end_line_address) = self.line_address()? {
                    return Ok(MyOption::Some(LineRange {
                        start: start_line_address,
                        end: end_line_address,
                    }));
                } else if let MyOption::Some(pattern) = self.pattern()? {
                    let end_line_address =
                        LineAddressType::Absolute(SimpleLineAddressType::Pattern(pattern));
                    return Ok(MyOption::Some(LineRange {
                        start: start_line_address,
                        end: end_line_address,
                    }));
                }
            } else {
                return Ok(MyOption::Some(LineRange {
                    start: start_line_address.clone(),
                    end: start_line_address.clone(),
                }));
            }
        } else if let MyOption::Some(pattern1) = self.pattern()? {
            if self.accept(TokenType::Separator, ",") {
                self.pop();
                if let MyOption::Some(end_line_address) = self.line_address()? {
                    return Ok(MyOption::Some(LineRange {
                        start: LineAddressType::Absolute(SimpleLineAddressType::Pattern(pattern1)),
                        end: end_line_address,
                    }));
                } else if let MyOption::Some(pattern2) = self.pattern()? {
                    return Ok(MyOption::Some(LineRange {
                        start: LineAddressType::Absolute(SimpleLineAddressType::Pattern(pattern1)),
                        end: LineAddressType::Absolute(SimpleLineAddressType::Pattern(pattern2)),
                    }));
                }
            } else {
                return Ok(MyOption::Some(LineRange {
                    start: LineAddressType::Absolute(SimpleLineAddressType::Pattern(
                        pattern1.clone(),
                    )),
                    end: LineAddressType::Absolute(SimpleLineAddressType::Pattern(
                        pattern1.clone(),
                    )),
                }));
            }
        }
        Ok(MyOption::None)
    }

    fn line_address(&mut self) -> Result<MyOption<LineAddressType>, GenericError> {
        // number, "$", "^", ".", "%"
        if self.accept_type(TokenType::Number) {
            if let MyOption::Some(token) = self.pop() {
                let number = token.lexeme.clone();
                return Ok(MyOption::Some(LineAddressType::Absolute(
                    SimpleLineAddressType::LineNumber(number.parse().unwrap()),
                )));
            }
        } else if self.accept(TokenType::Symbol, "$") {
            self.pop();
            return Ok(MyOption::Some(LineAddressType::Absolute(
                SimpleLineAddressType::LastLine,
            )));
        } else if self.accept(TokenType::Symbol, "^") {
            self.pop();
            return Ok(MyOption::Some(LineAddressType::Absolute(
                SimpleLineAddressType::FirstLine,
            )));
        } else if self.accept(TokenType::Symbol, ".") {
            self.pop();
            return Ok(MyOption::Some(LineAddressType::Absolute(
                SimpleLineAddressType::CurrentLine,
            )));
        } else if self.accept(TokenType::Symbol, "%") {
            self.pop();
            return Ok(MyOption::Some(LineAddressType::Absolute(
                SimpleLineAddressType::AllLines,
            )));
        }
        Ok(MyOption::None)
    }

    fn pattern(&mut self) -> Result<MyOption<Pattern>, GenericError> {
        if self.accept_type(TokenType::Pattern) {
            if let MyOption::Some(token) = &self.token_opt {
                let pattern = token.lexeme.clone();
                return Ok(MyOption::Some(Pattern { pattern }));
            }
        }
        Ok(MyOption::None)
    }

    fn substitute_command(&mut self) -> Result<MyOption<Box<dyn Command>>, GenericError> {
        Ok(MyOption::None)
    }

    fn simple_command(&mut self) -> Result<MyOption<Box<dyn Command>>, GenericError> {
        let command_opt =
            self.q_command()? | self.wq_command()? | self.q_exclamation_command()?
            | self.go_to_line_command()?;
        if let MyOption::Some(command) = command_opt {
            return Ok(MyOption::Some(command));
        }
        Ok(MyOption::None)
    }

    fn go_to_line_command(&mut self)  -> Result<MyOption<Box<dyn Command>>, GenericError> {
        let line_address = self.line_address()?;
        let end_of_input = self.accept_type(TokenType::EndOfInput);

        if let MyOption::Some(line_address) = line_address {
            if end_of_input {
                return Ok(MyOption::Some(Box::new(go_to_line::GoToLineCommand { line_address })));
            } else {
                self.undo_parse();
            }
        }

        Ok(MyOption::None)
    }

    fn q_command(&mut self) -> Result<MyOption<Box<dyn Command>>, GenericError> {
        if self.accept(TokenType::Command, "q") {
            self.pop();
            return Ok(MyOption::Some(Box::new(exit::ExitCommand {})));
        }

        Ok(MyOption::None)
    }

    fn q_exclamation_command(&mut self) -> Result<MyOption<Box<dyn Command>>, GenericError> {
        if self.accept(TokenType::Command, "q") {
            self.pop();
            if self.accept(TokenType::Command, "!") {
                self.pop();
                return Ok(MyOption::Some(Box::new(exit::ExitWithoutSaveCommand {})));
            }
        }
        Ok(MyOption::None)
    }

    fn wq_command(&mut self) -> Result<MyOption<Box<dyn Command>>, GenericError> {
        if self.accept(TokenType::Command, "w") {
            self.pop();
            if self.accept(TokenType::Command, "q") {
                self.pop();
                return Ok(MyOption::Some(Box::new(exit::ExitWithSaveCommand {})));
            }
        }
        Ok(MyOption::None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_q_command() {
        let input = "q";
        let mut parser = Parser::new(input);
        let command = parser.parse().unwrap();
        assert!(command.is::<exit::ExitCommand>());
    }

    #[test]
    fn test_parse_wq_command() {
        let input = "wq";
        let mut parser = Parser::new(input);
        let command = parser.parse().unwrap();
        assert!(command.is::<exit::ExitWithSaveCommand>());
    }

    #[test]
    fn test_parse_go_to_line_command() {
        let input = "1";
        let mut parser = Parser::new(input);
        let command = parser.parse().unwrap();
        assert!(command.is::<go_to_line::GoToLineCommand>());
    }

    #[test]
    fn test_parse_print_command() {
        let input = "p";
        let mut parser = Parser::new(input);
        let command = parser.parse().unwrap();
        assert!(command.is::<print::PrintCommand>());
    }

    #[test]
    fn test_parse_print_with_line_address() {
        let input = "1,2p";
        let mut parser = Parser::new(input);
        let command = parser.parse().unwrap();
        assert!(command.is::<print::PrintCommand>());
        let print_command = command.downcast_ref::<print::PrintCommand>().unwrap();
        assert_eq!(
            print_command.line_range,
            LineRange {
                start: LineAddressType::Absolute(SimpleLineAddressType::LineNumber(1)),
                end: LineAddressType::Absolute(SimpleLineAddressType::LineNumber(2)),
            }
        );
    }
}
