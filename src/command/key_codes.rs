use crossterm::event::KeyCode;
use crossterm::event::KeyCode::Char;

pub fn is_jump_command(key: &KeyCode) -> bool {
    match key {
        KeyCode::Left | KeyCode::Right | KeyCode::Up | KeyCode::Down => true,
        Char('h') | Char('j') | Char('k') | Char('l') => true,
        Char('w') | Char('W') | Char('b') | Char('B') => true,
        Char('e') | Char('E') | Char('0') | Char('$') => true,
        Char('g') | Char('G') => true,
        Char('^') | Char('H') | Char('M') | Char('L') => true,
        Char('f') | Char('F') | Char('t') | Char('T') => true,
        Char(';') | Char(',') | Char(')') | Char('(') => true,
        Char('}') | Char('{') | Char(']') | Char('[') => true,
        Char('%') => true,
        _ => false,
    }
}

pub fn is_editing_command_without_range(key: &KeyCode) -> bool {
    match key {
        Char('i') | Char('I') | Char('a') | Char('A') => true,
        Char('o') | Char('O') | Char('s') | Char('S') => true,
        Char('x') | Char('X') | Char('r') | Char('R') => true,
        Char('D') | Char('p') | Char('P') | Char('~') => true,
        Char('u') => true,
        _ => false,
    }
}

pub fn is_editing_command_with_range(key: &KeyCode) -> bool {
    match key {
        Char('d') | Char('c') | Char('y') => true,
        Char('>') | Char('<') => true,
        Char('Z') => true,
        _ => false,
    }
}

pub fn is_ctrl_command(key: &KeyCode) -> bool {
    match key {
        Char('[') | Char('l') | Char('g') => true,
        Char('f') | Char('b') | Char('d') | Char('u') => true,
        Char('e') | Char('y') => true,
        Char('c') => true,
        Char('z') => true,
         _ => false,
    }
}
