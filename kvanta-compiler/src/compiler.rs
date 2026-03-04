// use std::rc::Rc;

// use kvanta_compiler::chunk::*;
// //use kvanta_compiler::debug::*;
// use kvanta_compiler::value::*;
// use kvanta_compiler::vm::*;

// fn main() {
//     let mut chunk = Chunk::new();
//     let constant = chunk.add_constant(Value::Float(1.2));
//     chunk.push_code(OpCode::OpConstant, 0);
//     chunk.push(constant as u8, 0);
//     chunk.push_code(OpCode::OpNegate, 0);
//     chunk.push_code(OpCode::OpReturn, 123);
//     let _ = interpret(Rc::new(chunk));
// }

use crate::chunk::Chunk;

struct Scanner<'comp> {
    start: usize,
    current: usize,
    line: u32,
    source: &'comp str,
    line_size: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    String,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    For,
    Fun,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
    Error
}

#[derive(Debug, Clone)]
struct Token<'a> {
    token_type: TokenType,
    lexeme: &'a str,
    line: u32,
}



impl<'comp> Scanner<'comp> {
    fn make_token(&self, token_type: TokenType) -> Token<'comp> {
        Token {
            token_type,
            lexeme: &self.source[self.start..self.current],
            line: self.line,
        }
    }

    fn error_token(&self, message: &'static str) -> Token<'static> {
        Token {
            token_type: TokenType::Error,
            lexeme: message,
            line: self.line,
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.line_size
    }

    fn peek(&self) -> char {
        self.source.chars().nth(self.current).unwrap_or('\0')
    }

    fn peek_next(&self) -> char {
        self.source.chars().nth(self.current + 1).unwrap_or('\0')
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.peek() != expected {
            return false;
        }
        self.current += 1;
        true
    }

    fn skip_whitespace(&mut self) {
        loop {
            let c = self.peek();
            match c {
                ' ' | '\r' | '\t' => self.current += 1,
                '\n' => {
                    self.line += 1;
                    self.current += 1;
                }
                '/' => {
                    if self.peek_next() == '/' {
                        while self.peek() != '\n' {
                            self.current += 1;
                        }
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }
    }

    fn check_keyword(&self, start: usize, rest: &str, token_type: TokenType) -> TokenType {
        let end = self.start + start + rest.len();
        if self.current == end && self.source[self.start + start..end] == *rest {
            token_type
        } else {
            TokenType::Identifier
        }
    }

    fn identifier_type(&self) -> TokenType {
        match self.source.chars().nth(self.start).unwrap_or('\0') {
            'a' => self.check_keyword(1, "nd", TokenType::And),
            'c' => self.check_keyword(1, "lass", TokenType::Class),
            'e' => self.check_keyword(1, "lse", TokenType::Else),
            'f' => {
                if self.current - self.start > 1 {
                    match self.source.chars().nth(self.start + 1).unwrap_or('\0') {
                        'a' => self.check_keyword(2, "lse", TokenType::False),
                        'o' => self.check_keyword(2, "r", TokenType::For),
                        'u' => self.check_keyword(2, "n", TokenType::Fun),
                        _ => TokenType::Identifier,
                    }
                } else {
                    TokenType::Identifier
                }
            }
            'i' => self.check_keyword(1, "f", TokenType::If),
            'n' => self.check_keyword(1, "il", TokenType::Nil),
            'o' => self.check_keyword(1, "r", TokenType::Or),
            'p' => self.check_keyword(1, "rint", TokenType::Print),
            'r' => self.check_keyword(1, "eturn", TokenType::Return),
            's' => self.check_keyword(1, "uper", TokenType::Super),
            't' => {
                if self.current - self.start > 1 {
                    match self.source.chars().nth(self.start + 1).unwrap_or('\0') {
                        'h' => self.check_keyword(2, "is", TokenType::This),
                        'r' => self.check_keyword(2, "ue", TokenType::True),
                        _ => TokenType::Identifier,
                    }
                } else {
                    TokenType::Identifier
                }
            }
            'v' => self.check_keyword(1, "ar", TokenType::Var),
            'w' => self.check_keyword(1, "hile", TokenType::While),
            _ => TokenType::Identifier,
        }
    }

    fn scan_token(&mut self) -> Token<'comp> {
        self.skip_whitespace();
        self.start = self.current;

        if self.is_at_end() {
            return self.make_token(TokenType::Eof);
        }

        let c = self.advance();
        match c {
            '(' => self.make_token(TokenType::LeftParen),
            ')' => self.make_token(TokenType::RightParen),
            '{' => self.make_token(TokenType::LeftBrace),
            '}' => self.make_token(TokenType::RightBrace),
            ';' => self.make_token(TokenType::Semicolon),
            ',' => self.make_token(TokenType::Comma),
            '.' => self.make_token(TokenType::Dot),
            '-' => self.make_token(TokenType::Minus),
            '+' => self.make_token(TokenType::Plus),
            '/' => self.make_token(TokenType::Slash),
            '*' => self.make_token(TokenType::Star),
            '!' => {
                if self.match_char('=') {
                    self.make_token(TokenType::BangEqual)
                } else {
                    self.make_token(TokenType::Bang)
                }
            },
            '=' => {
                if self.match_char('=') {
                    self.make_token(TokenType::EqualEqual)
                } else {
                    self.make_token(TokenType::Equal)
                }
            },
            '<' => {
                if self.match_char('=') {
                    self.make_token(TokenType::LessEqual)
                } else {                    
                    self.make_token(TokenType::Less)
                }
            },
            '>' => {
                if self.match_char('=') {
                    self.make_token(TokenType::GreaterEqual)
                } else {
                    self.make_token(TokenType::Greater)
                }
            },
            '"' => {
                while self.current < self.line_size && self.peek() != '"' {
                    if self.peek() == '\n' {
                        self.line += 1;
                    }
                    self.current += 1;
                }
                if self.is_at_end() {
                    return self.error_token("Unterminated string.");
                }
                self.current += 1;
                self.make_token(TokenType::String)
            },
            '0'..='9' => {
                while self.peek().is_ascii_digit() {
                    self.current += 1;
                }
                if self.peek() == '.' && self.peek_next().is_ascii_digit() {
                    self.current += 1;
                    while self.peek().is_ascii_digit() {
                        self.current += 1;
                    }
                }
                self.make_token(TokenType::Number)
            },
            x => {
                if x.is_alphabetic() || x == '_' {
                    while self.peek().is_alphanumeric() || self.peek() == '_' {
                        self.current += 1;
                    }
                    return self.make_token(self.identifier_type());
                }
                self.error_token("Unknown token")
            }
        }
    }

    fn advance(&mut self) -> char {
        let c = self.peek();
        self.current += 1;
        c
    }

    fn new(source: &str) -> Self {
        Scanner {
            start: 0,
            current: 0,
            line: 1,
            source,
            line_size: source.len(),
        }
    }
}

struct Parser<'comp> {
    current: Token<'comp>,
    previous: Token<'comp>,
    scanner: Scanner<'comp>,
    had_error: bool,
    panic_mode: bool,
}

impl<'comp> Parser<'comp> {
    fn error_at_current(&mut self, message: &str) {
        if self.panic_mode {
            return;
        }
        self.panic_mode = true;
        let cur = self.current.clone();
        self.error_at(&cur, message);
    }

    fn error_at(&mut self, token: &Token, message: &str) {
        eprint!("[line {}] Error", token.line);
        if token.token_type == TokenType::Eof {
            eprint!(" at end");
        } else if token.token_type == TokenType::Error {
            // Nothing.
        } else {
            eprint!(" at '{}'", token.lexeme);
        }
        eprintln!(": {}", message);
        self.had_error = true;
    }

    fn advance(&mut self, scanner: &'comp mut Scanner<'comp>) {
        self.previous = self.current.clone();
        loop {
            self.current = scanner.scan_token();
            if self.current.token_type != TokenType::Error {
                break;
            }
            self.error_at_current(&self.current.lexeme);
        }
    }

    fn consume(&mut self, token_type: TokenType, message: &str, scanner: &'comp mut Scanner<'comp>) {
        if self.current.token_type == token_type {
            self.advance(scanner);
            return;
        }
        self.error_at_current(message);
    }
}

pub fn compile(source: String) -> Result<Chunk, String> {
    let mut scanner = Scanner::new(&source);

    let dummy_token = Token {
        token_type: TokenType::Eof,
        lexeme: "",
        line: 0,
    };

    let mut parser = Parser {
        current: dummy_token.clone(),
        previous: dummy_token,
        had_error: false,
        panic_mode: false,
    };

    parser.advance(&mut scanner);
    
    let mut line : u32 = 0;

    loop {
        let token = scanner.scan_token();
        let token_type = token.token_type.clone();
        let token_line = token.line;
        if token_line != line {
            line = token_line;
            println!("LINE: {}", line);
        } else {
            println!("     |");
        }
        println!("{:?}", token);
        if token_type == TokenType::Eof {
            break;
        } 
    }

    if parser.had_error {
        Err("Compilation failed".to_string())
    } else {
        Ok(Chunk::new())
    }
}
