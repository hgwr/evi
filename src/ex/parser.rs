use crate::ex::lexer;
use crate::command::base::Command;
use crate::generic_error::GenericError;

use crate::command::commands::exit::ExitCommand;


pub fn parse(input: &str) -> Result<Box<dyn Command>, GenericError> {
    let tokens = lexer::tokenize(input);
    let command_opt = simple_command(tokens)?;
    match command_opt {
        Some(command) => Ok(command),
        None => Err("Invalid command".to_string().into()),
    }
}

fn simple_command(tokens: Vec<lexer::Token>) -> Result<Option<Box<dyn Command>>, GenericError> {
    println!("{:?}", tokens);
    // tokens が ":", "w", "q" は Ok(Some(Box::new(ExitCommand {}))) を返す。
    if tokens.len() == 4 {
        if tokens[0].token_type == lexer::TokenType::Colon &&
            tokens[1].token_type == lexer::TokenType::Command &&
            tokens[2].token_type == lexer::TokenType::Command {
            if tokens[1].lexeme == "w" && tokens[2].lexeme == "q" {
                return Ok(Some(Box::new(ExitCommand {})));
            }
        }
    }
    Ok(None)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_command() {
        let input = ":wq";
        let command = parse(input).unwrap();
        assert!(command.is::<ExitCommand>());
    }
}
