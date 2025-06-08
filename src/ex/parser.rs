use std::ops::BitOr;

use crate::command::base::Command;
use crate::command::commands::copy_lines;
use crate::command::commands::delete;
use crate::command::commands::global;
use crate::command::commands::go_to_line;
use crate::command::commands::move_lines;
use crate::command::commands::substitute;
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
use crate::command::commands::write;

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

    pub fn from_tokens(tokens: Vec<Token>) -> Self {
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
        let command_opt = self.display_command(&line_range)?
            | self.substitute_command(&line_range)?
            | self.delete_command(&line_range)?
            | self.move_command(&line_range)?
            | self.copy_command(&line_range)?
            | self.global_command(&line_range)?;
        if let MyOption::Some(command) = command_opt {
            return Ok(MyOption::Some(command));
        }
        Ok(MyOption::None)
    }

    fn delete_command(
        &mut self,
        line_range: &LineRange,
    ) -> Result<MyOption<Box<dyn Command>>, GenericError> {
        if self.accept(TokenType::Command, "d") {
            let delete_command = delete::DeleteLines {
                line_range: line_range.clone(),
                editor_cursor_data: None,
                text: None,
            };
            return Ok(MyOption::Some(Box::new(delete_command)));
        }
        Ok(MyOption::None)
    }

    fn destination_address(&mut self) -> Result<LineAddressType, GenericError> {
        if let MyOption::Some(address) = self.line_address()? {
            Ok(address)
        } else {
            Err(self.error("line address expected"))
        }
    }

    fn move_command(
        &mut self,
        line_range: &LineRange,
    ) -> Result<MyOption<Box<dyn Command>>, GenericError> {
        if self.accept(TokenType::Command, "m") {
            let address = self.destination_address()?;
            let mv = move_lines::MoveLines {
                line_range: line_range.clone(),
                address,
                original_start_idx: None,
                inserted_base: None,
                drained_lines: Vec::new(),
            };
            return Ok(MyOption::Some(Box::new(mv)));
        }
        Ok(MyOption::None)
    }

    fn copy_command(
        &mut self,
        line_range: &LineRange,
    ) -> Result<MyOption<Box<dyn Command>>, GenericError> {
        if self.accept(TokenType::Command, "co") {
            // accepted 'co'
        } else if self.accept(TokenType::Command, "t") {
            // accepted 't'
        } else {
            return Ok(MyOption::None);
        }
        let address = self.destination_address()?;
        let cp = copy_lines::CopyLines {
            line_range: line_range.clone(),
            address,
            editor_cursor_data: None,
            insertion_idx: None,
            copied_len: 0,
        };
        return Ok(MyOption::Some(Box::new(cp)));
    }

    fn global_command(
        &mut self,
        line_range: &LineRange,
    ) -> Result<MyOption<Box<dyn Command>>, GenericError> {
        if self.accept(TokenType::Command, "g") {
            self.pop();
            let mut invert = false;
            if self.accept(TokenType::Symbol, "!") {
                self.pop();
                invert = true;
            }

            let pattern = if let MyOption::Some(tok) = &self.token_opt {
                if tok.token_type == TokenType::Pattern
                    || tok.token_type == TokenType::AddressPattern
                {
                    let t_type = tok.token_type.clone();
                    self.accept_type(t_type);
                    if let MyOption::Some(token) = self.pop() {
                        token.lexeme
                    } else {
                        return Err(self.error("pattern expected"));
                    }
                } else {
                    return Err(self.error("pattern expected"));
                }
            } else {
                return Err(self.error("pattern expected"));
            };

            let mut remaining_tokens = Vec::new();
            if let MyOption::Some(tok) = &self.token_opt {
                remaining_tokens.push(tok.clone());
            }
            remaining_tokens.append(&mut self.tokens.clone());

            self.tokens.clear();
            self.token_opt = MyOption::Some(Token {
                token_type: TokenType::EndOfInput,
                lexeme: String::new(),
            });

            let command_tokens = if remaining_tokens.len() == 1
                && remaining_tokens[0].token_type == TokenType::EndOfInput
            {
                None
            } else {
                Some(remaining_tokens)
            };

            let command = global::GlobalCommand {
                line_range: line_range.clone(),
                pattern,
                command_tokens,
                invert,
            };
            return Ok(MyOption::Some(Box::new(command)));
        }
        Ok(MyOption::None)
    }

    fn display_command(
        &mut self,
        line_range: &LineRange,
    ) -> Result<MyOption<Box<dyn Command>>, GenericError> {
        if self.accept(TokenType::Command, "p") {
            let print_command = print::PrintCommand {
                line_range: line_range.clone(),
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
                if start_line_address == LineAddressType::Absolute(SimpleLineAddressType::AllLines)
                {
                    return Ok(MyOption::Some(LineRange {
                        start: LineAddressType::Absolute(SimpleLineAddressType::FirstLine),
                        end: LineAddressType::Absolute(SimpleLineAddressType::LastLine),
                    }));
                }
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

    fn substitute_command(
        &mut self,
        line_range: &LineRange,
    ) -> Result<MyOption<Box<dyn Command>>, GenericError> {
        if self.accept(TokenType::Command, "s") {
            self.pop();

            let pattern = if self.accept_type(TokenType::Pattern) {
                if let MyOption::Some(token) = self.pop() {
                    token.lexeme
                } else {
                    String::new()
                }
            } else {
                return Err(self.error("pattern expected"));
            };

            let replacement = if self.accept_type(TokenType::Replacement) {
                if let MyOption::Some(token) = self.pop() {
                    token.lexeme
                } else {
                    String::new()
                }
            } else {
                return Err(self.error("replacement expected"));
            };

            let mut options = String::new();
            if self.accept_type(TokenType::Option) {
                if let MyOption::Some(token) = self.pop() {
                    options = token.lexeme;
                }
            }

            let mut global = false;
            let mut ignore_case = false;
            for ch in options.chars() {
                match ch {
                    'g' => global = true,
                    'i' => ignore_case = true,
                    _ => {}
                }
            }

            let command = substitute::SubstituteCommand {
                line_range: line_range.clone(),
                pattern,
                replacement,
                global,
                ignore_case,
                original_lines: Vec::new(),
            };
            return Ok(MyOption::Some(Box::new(command)));
        }

        Ok(MyOption::None)
    }

    fn simple_command(&mut self) -> Result<MyOption<Box<dyn Command>>, GenericError> {
        let command_opt = self.q_command()?
            | self.wq_command()?
            | self.q_exclamation_command()?
            | self.w_exclamation_command()?
            | self.w_command()?
            | self.x_command()?
            | self.go_to_line_command()?;
        if let MyOption::Some(command) = command_opt {
            return Ok(MyOption::Some(command));
        }
        Ok(MyOption::None)
    }

    fn go_to_line_command(&mut self) -> Result<MyOption<Box<dyn Command>>, GenericError> {
        let line_address = self.line_address()?;
        let end_of_input = self.accept_type(TokenType::EndOfInput);

        if let MyOption::Some(line_address) = line_address {
            if end_of_input {
                return Ok(MyOption::Some(Box::new(go_to_line::GoToLineCommand {
                    line_address,
                })));
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
            } else {
                self.undo_parse();
            }
        }
        Ok(MyOption::None)
    }

    fn w_exclamation_command(&mut self) -> Result<MyOption<Box<dyn Command>>, GenericError> {
        if self.accept(TokenType::Command, "w") {
            self.pop();
            if self.accept(TokenType::Command, "!") || self.accept(TokenType::Symbol, "!") {
                self.pop();
                if self.accept_type(TokenType::EndOfInput) {
                    return Ok(MyOption::Some(Box::new(write::WriteCommand {
                        force: true,
                    })));
                } else {
                    self.undo_parse();
                }
            } else {
                self.undo_parse();
            }
        }
        Ok(MyOption::None)
    }

    fn w_command(&mut self) -> Result<MyOption<Box<dyn Command>>, GenericError> {
        if self.accept(TokenType::Command, "w") {
            self.pop();
            if self.accept_type(TokenType::EndOfInput) {
                return Ok(MyOption::Some(Box::new(write::WriteCommand {
                    force: false,
                })));
            } else {
                self.undo_parse();
            }
        }
        Ok(MyOption::None)
    }

    fn x_command(&mut self) -> Result<MyOption<Box<dyn Command>>, GenericError> {
        if self.accept(TokenType::Command, "x") {
            self.pop();
            if self.accept_type(TokenType::EndOfInput) {
                return Ok(MyOption::Some(Box::new(exit::ExitCommand {})));
            } else {
                self.undo_parse();
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
    fn test_parse_x_command() {
        let input = "x";
        let mut parser = Parser::new(input);
        let command = parser.parse().unwrap();
        assert!(command.is::<exit::ExitCommand>());
    }

    #[test]
    fn test_parse_write_command() {
        let input = "w";
        let mut parser = Parser::new(input);
        let command = parser.parse().unwrap();
        assert!(command.is::<write::WriteCommand>());
        let cmd = command.downcast_ref::<write::WriteCommand>().unwrap();
        assert!(!cmd.force);
    }

    #[test]
    fn test_parse_write_force_command() {
        let input = "w!";
        let mut parser = Parser::new(input);
        let command = parser.parse().unwrap();
        assert!(command.is::<write::WriteCommand>());
        let cmd = command.downcast_ref::<write::WriteCommand>().unwrap();
        assert!(cmd.force);
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

    #[test]
    fn test_parse_delete_line_with_address() {
        let input = "1,3d";
        let mut parser = Parser::new(input);
        let command = parser.parse().unwrap();
        assert!(command.is::<delete::DeleteLines>());
        let delete_command = command.downcast_ref::<delete::DeleteLines>().unwrap();
        assert_eq!(
            delete_command.line_range,
            LineRange {
                start: LineAddressType::Absolute(SimpleLineAddressType::LineNumber(1)),
                end: LineAddressType::Absolute(SimpleLineAddressType::LineNumber(3)),
            }
        );
    }

    #[test]
    fn test_parse_substitute_ignore_case() {
        let input = "1,5s/^abc/cba/i";
        let mut parser = Parser::new(input);
        let command = parser.parse().unwrap();
        assert!(command.is::<substitute::SubstituteCommand>());
        let sub = command
            .downcast_ref::<substitute::SubstituteCommand>()
            .unwrap();
        assert_eq!(
            sub.line_range,
            LineRange {
                start: LineAddressType::Absolute(SimpleLineAddressType::LineNumber(1)),
                end: LineAddressType::Absolute(SimpleLineAddressType::LineNumber(5)),
            }
        );
        assert_eq!(sub.pattern, "^abc");
        assert_eq!(sub.replacement, "cba");
        assert!(sub.ignore_case);
        assert!(!sub.global);
    }

    #[test]
    fn test_parse_substitute_global_all_lines() {
        let input = "%s/^abc/cba/g";
        let mut parser = Parser::new(input);
        let command = parser.parse().unwrap();
        assert!(command.is::<substitute::SubstituteCommand>());
        let sub = command
            .downcast_ref::<substitute::SubstituteCommand>()
            .unwrap();
        assert_eq!(
            sub.line_range,
            LineRange {
                start: LineAddressType::Absolute(SimpleLineAddressType::FirstLine),
                end: LineAddressType::Absolute(SimpleLineAddressType::LastLine),
            }
        );
        assert_eq!(sub.pattern, "^abc");
        assert_eq!(sub.replacement, "cba");
        assert!(sub.global);
        assert!(!sub.ignore_case);
    }

    #[test]
    fn test_parse_substitute_line_range_last() {
        let input = "1,$s/cde$/CDE/";
        let mut parser = Parser::new(input);
        let command = parser.parse().unwrap();
        assert!(command.is::<substitute::SubstituteCommand>());
        let sub = command
            .downcast_ref::<substitute::SubstituteCommand>()
            .unwrap();
        assert_eq!(
            sub.line_range,
            LineRange {
                start: LineAddressType::Absolute(SimpleLineAddressType::LineNumber(1)),
                end: LineAddressType::Absolute(SimpleLineAddressType::LastLine),
            }
        );
        assert_eq!(sub.pattern, "cde$");
        assert_eq!(sub.replacement, "CDE");
        assert!(!sub.global);
        assert!(!sub.ignore_case);
    }

    #[test]
    fn test_parse_global_delete() {
        let input = "g/foo/d";
        let mut parser = Parser::new(input);
        let command = parser.parse().unwrap();
        assert!(command.is::<global::GlobalCommand>());
        let g = command.downcast_ref::<global::GlobalCommand>().unwrap();
        assert_eq!(g.pattern, "foo");
        assert!(!g.invert);
        assert!(g.command_tokens.is_some());
    }

    #[test]
    fn test_parse_global_invert() {
        let input = "g!/bar/p";
        let mut parser = Parser::new(input);
        let command = parser.parse().unwrap();
        assert!(command.is::<global::GlobalCommand>());
        let g = command.downcast_ref::<global::GlobalCommand>().unwrap();
        assert_eq!(g.pattern, "bar");
        assert!(g.invert);
    }
}
