use log::info;
use std::ops::BitOr;

use crate::command::base::Command;
use crate::ex::lexer;
use crate::generic_error::GenericError;

use crate::command::commands::exit;

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

pub fn parse(input: &str) -> Result<Box<dyn Command>, GenericError> {
    let tokens = lexer::tokenize(input);
    info!("tokens {:?}", tokens);
    let command_opt = simple_command(&tokens)?;
    match command_opt {
        MyOption::Some(command) => Ok(command),
        MyOption::None => Err("Invalid command".to_string().into()),
    }
}

fn simple_command(tokens: &Vec<lexer::Token>) -> Result<MyOption<Box<dyn Command>>, GenericError> {
    let command_opt =
        q_command(tokens)? | wq_command(tokens)?;
    if let MyOption::Some(command) = command_opt {
        return Ok(MyOption::Some(command));
    }
    Ok(MyOption::None)
}

fn q_command(tokens: &Vec<lexer::Token>) -> Result<MyOption<Box<dyn Command>>, GenericError> {
    // tokens が "q" は Ok(Some(Box::new(ExitCommand {}))) を返す。
    if tokens.len() == 2 {
        if tokens[0].token_type == lexer::TokenType::Command {
            if tokens[0].lexeme == "q" {
                return Ok(MyOption::Some(Box::new(exit::ExitCommand {})));
            }
        }
    }
    Ok(MyOption::None)
}

fn wq_command(tokens: &Vec<lexer::Token>) -> Result<MyOption<Box<dyn Command>>, GenericError> {
    // tokens が "w", "q" は Ok(Some(Box::new(ExitCommand {}))) を返す。
    if tokens.len() == 3 {
        if tokens[0].token_type == lexer::TokenType::Command
            && tokens[1].token_type == lexer::TokenType::Command
        {
            if tokens[0].lexeme == "w" && tokens[1].lexeme == "q" {
                return Ok(MyOption::Some(Box::new(exit::ExitWithSaveCommand {})));
            }
        }
    }
    Ok(MyOption::None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_command() {
        let input = "wq";
        let command = parse(input).unwrap();
        assert!(command.is::<exit::ExitWithSaveCommand>());
    }
}
