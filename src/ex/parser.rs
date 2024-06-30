use log::info;
use std::ops::BitOr;

use crate::command::base::Command;
use crate::ex::lexer;
use crate::generic_error::GenericError;

use crate::command::commands::exit;

use super::lexer::TokenType;

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

pub enum SimpleLineAddressType {
    LineNumber(usize),
    CurrentLine,
    LastLine,
    AllLines,
}

pub enum LineAddressType {
    Absolute(SimpleLineAddressType),
    Relative(SimpleLineAddressType, isize),
}

pub struct LineRange {
    start: LineAddressType,
    end: LineAddressType,
}

pub struct Parser {
    tokens: Vec<lexer::Token>,
    token: MyOption<lexer::Token>,
    command: MyOption<Box<dyn Command>>,
    line_range: MyOption<LineRange>,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        let tokens = lexer::tokenize(input);
        info!("tokens {:?}", tokens);
        Parser {
            tokens,
            token: MyOption::None,
            command: MyOption::None,
            line_range: MyOption::None,
        }
    }

    pub fn parse(&mut self) -> Result<Box<dyn Command>, GenericError> {
        self.command()
    }

    fn command(&mut self) -> Result<Box<dyn Command>, GenericError> {
        let command_opt = self.simple_command()? | self.complex_command()?;
        match command_opt {
            MyOption::Some(command) => Ok(command),
            MyOption::None => Err("Invalid command".to_string().into()),
        }
    }

    fn get_symbol(&mut self) -> MyOption<lexer::Token> {
        if self.tokens.len() > 0 {
            return MyOption::Some(self.tokens.remove(0));
        } else {
            return MyOption::Some(lexer::Token {
                token_type: TokenType::EndOfInput,
                lexeme: "".to_string(),
            });
        }
    }

    fn error(&self, message: &str) -> GenericError {
        format!("Error: {}", message).into()
    }

    fn accept(&mut self, token_type: lexer::TokenType) -> bool {
        if let MyOption::Some(token) = &self.token {
            if token.token_type == token_type {
                self.token = self.get_symbol();
                return true;
            }
        }
        false
    }

    fn expect(&mut self, token_type: lexer::TokenType) -> Result<(), GenericError> {
        if self.accept(token_type) {
            return Ok(());
        }
        Err(self.error("Unexpected token"))
    }

    fn simple_command(&mut self) -> Result<MyOption<Box<dyn Command>>, GenericError> {
        let command_opt = self.q_command()? | self.wq_command()?;
        if let MyOption::Some(command) = command_opt {
            return Ok(MyOption::Some(command));
        }
        Ok(MyOption::None)
    }

    fn q_command(&mut self) -> Result<MyOption<Box<dyn Command>>, GenericError> {
        // tokens が "q" は Ok(Some(Box::new(ExitCommand {}))) を返す。
        if self.tokens.len() == 2 {
            if self.tokens[0].token_type == lexer::TokenType::Command {
                if self.tokens[0].lexeme == "q" {
                    return Ok(MyOption::Some(Box::new(exit::ExitCommand {})));
                }
            }
        }
        Ok(MyOption::None)
    }

    fn wq_command(&mut self) -> Result<MyOption<Box<dyn Command>>, GenericError> {
        // tokens が "w", "q" は Ok(Some(Box::new(ExitWithSaveCommand {}))) を返す。
        if self.tokens.len() == 3 {
            if self.tokens[0].token_type == lexer::TokenType::Command
                && self.tokens[1].token_type == lexer::TokenType::Command
            {
                if self.tokens[0].lexeme == "w" && self.tokens[1].lexeme == "q" {
                    return Ok(MyOption::Some(Box::new(exit::ExitWithSaveCommand {})));
                }
            }
        }
        Ok(MyOption::None)
    }

    fn complex_command(&mut self) -> Result<MyOption<Box<dyn Command>>, GenericError> {
        Ok(MyOption::None)
    }

    // fn complex_command(tokens: &Vec<lexer::Token>) -> Result<MyOption<Box<dyn Command>>, GenericError> {
    //     let command_opt =
    //         display_command(tokens)? | substitute_command(tokens)?;
    //     if let MyOption::Some(command) = command_opt {
    //         return Ok(MyOption::Some(command));
    //     }
    //     Ok(MyOption::None)
    // }

    // fn display_command(tokens: &Vec<lexer::Token>) -> Result<MyOption<Box<dyn Command>>, GenericError> {
    //     let command_opt =
    //         line_range(tokens)? & print_command(tokens)?;
    //     if let MyOption::Some(command) = command_opt {
    //         return Ok(MyOption::Some(command));
    //     }
    //     Ok(MyOption::None)
    // }

    // fn line_range(tokens: &Vec<lexer::Token>) -> Result<MyOption<LineRange>, GenericError> {
    //     let result =
    //         line_address(tokens)? & comma(tokens)? & line_address(tokens)?
    //         | pattern(tokens)? & comma(tokens)? & line_address(tokens)?
    //         | line_address(tokens)? & comma(tokens)? & pattern(tokens)?
    //         | pattern(tokens)? & comma(tokens)? & pattern(tokens)?;
    //         | line_address(tokens)?;
    //     if let MyOption::Some(line_range) = result {
    //         return Ok(MyOption::Some(line_range));
    //     }
    //     Ok(MyOption::None)
    // }

    // fn line_address(tokens: &Vec<lexer::Token>) -> Result<MyOption<LineAddressType>, GenericError> {
    //     if tokens.len() == 1 {
    //         if tokens[0].token_type == lexer::TokenType::Number {
    //             let line_number = tokens[0].lexeme.parse::<usize>().unwrap();
    //             return Ok(MyOption::Some(LineAddressType::Absolute(SimpleLineAddressType::LineNumber(line_number))));
    //         }
    //     }
    //     Ok(MyOption::None)
    // }

    // fn substitute_command(tokens: &Vec<lexer::Token>) -> Result<MyOption<Box<dyn Command>>, GenericError> {
    //     Ok(MyOption::None)
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_command() {
        let input = "wq";
        let mut parser = Parser::new(input);
        let command = parser.parse().unwrap();
        assert!(command.is::<exit::ExitWithSaveCommand>());
    }
}
