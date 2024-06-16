use log::info;

use crate::command::base::Command;
use crate::ex::lexer;
use crate::generic_error::GenericError;

use crate::command::commands::exit;

pub fn parse(input: &str) -> Result<Box<dyn Command>, GenericError> {
    let tokens = lexer::tokenize(input);
    info!("tokens {:?}", tokens);
    let command_opt = simple_command(tokens)?;
    match command_opt {
        Some(command) => Ok(command),
        None => Err("Invalid command".to_string().into()),
    }
}

fn simple_command(tokens: Vec<lexer::Token>) -> Result<Option<Box<dyn Command>>, GenericError> {
    // tokens が "w", "q" は Ok(Some(Box::new(ExitCommand {}))) を返す。
    if tokens.len() == 3 {
        if tokens[0].token_type == lexer::TokenType::Command
            && tokens[1].token_type == lexer::TokenType::Command
        {
            if tokens[0].lexeme == "w" && tokens[1].lexeme == "q" {
                return Ok(Some(Box::new(exit::ExitWithSaveCommand {})));
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
        let input = "wq";
        let command = parse(input).unwrap();
        assert!(command.is::<exit::ExitWithSaveCommand>());
    }
}
