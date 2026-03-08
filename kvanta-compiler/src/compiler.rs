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

use crate::{chunk::{Chunk, OpCode}, value::Value};

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

    fn new(source: &'comp str) -> Self {
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
    current_chunk: Chunk,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Precedence {
    None,
    Assignment,  // =
    Or,          // or
    And,         // and
    Equality,    // == !=
    Comparison,   // < > <= >=
    Term,         // + -
    Factor,       // * /
    Unary,        // ! -
    Call,         // . ()
    Primary,
}

impl Precedence {
    fn next(self) -> Option<Precedence> {
        match self {
            Precedence::None => Some(Precedence::Assignment),
            Precedence::Assignment => Some(Precedence::Or),
            Precedence::Or => Some(Precedence::And),
            Precedence::And => Some(Precedence::Equality),
            Precedence::Equality => Some(Precedence::Comparison),
            Precedence::Comparison => Some(Precedence::Term),
            Precedence::Term => Some(Precedence::Factor),
            Precedence::Factor => Some(Precedence::Unary),
            Precedence::Unary => Some(Precedence::Call),
            Precedence::Call => Some(Precedence::Primary),
            Precedence::Primary => None,
        }
    }
}

struct ParseRule {
    prefix: Option<fn(&mut Parser)>,
    infix: Option<fn(&mut Parser)>,
    precedence: Precedence,
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

    fn advance(&mut self) {
        self.previous = self.current.clone();
        loop {
            self.current = self.scanner.scan_token();
            if self.current.token_type != TokenType::Error {
                break;
            }
            self.error_at_current(self.current.lexeme);
        }
    }

    fn consume(&mut self, token_type: TokenType, message: &str) {
        if self.current.token_type == token_type {
            self.advance();
            return;
        }
        self.error_at_current(message);
    }

    fn emit_byte(&mut self, byte: u8) {
        self.current_chunk.push(byte, self.previous.line);
    }

    fn emit_bytes(&mut self, byte1: u8, byte2: u8) {
        self.emit_byte(byte1);
        self.emit_byte(byte2);
    }

    fn emit_constant(&mut self, value: f32) {
        let constant = self.current_chunk.add_constant(Value::Float(value));
        if constant > u8::MAX as usize {
            self.error_at_current("Too many constants in one chunk.");
            return;
        }
        self.emit_bytes(OpCode::OpConstant as u8, constant as u8);
    }


    fn new(scanner: Scanner<'comp>) -> Self {
        let dummy_token = Token {
            token_type: TokenType::Eof,
            lexeme: "",
            line: 0,
        };

        Parser {
            current: dummy_token.clone(),
            previous: dummy_token,
            had_error: false,
            panic_mode: false,
            scanner,
            current_chunk: Chunk::new(),
        }
    }

    fn end_compile(&mut self) {
        self.consume(TokenType::Eof, "Expect end of expression.");
        self.emit_byte(OpCode::OpReturn as u8); // OpReturn
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn number(&mut self) {
        let value = self.previous.lexeme.parse::<f32>().unwrap();
        self.emit_constant(value);
    }

    fn literal(&mut self) {
        match self.previous.token_type {
            TokenType::False => self.emit_byte(OpCode::OpFalse as u8),
            TokenType::True => self.emit_byte(OpCode::OpTrue as u8),
            TokenType::Nil => self.emit_byte(OpCode::OpNil as u8),
            _ => (),
        }
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression.");
    }

    fn unary(&mut self) {
        let operator_type = self.previous.token_type.clone();
        self.parse_precedence(Precedence::Unary);
        if operator_type == TokenType::Minus {
            self.emit_byte(OpCode::OpNegate as u8);
        }
        // match operator_type {
        //     TokenType::Minus => self.emit_byte(OpCode::OpNegate as u8),
        //     _ => (),
        // }
    }
   

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        let prefix_rule = get_rule(self.previous.token_type.clone()).prefix;
        if let Some(prefix_rule) = prefix_rule {
            prefix_rule(self);
        } else {
            self.error_at_current("Expect expression.");
            return;
        }

        while precedence <= get_rule(self.current.token_type.clone()).precedence {
            self.advance();
            let infix_rule = get_rule(self.previous.token_type.clone()).infix;
            if let Some(infix_rule) = infix_rule {
                infix_rule(self);
            }
        }
    }

    fn binary(&mut self) {
        let operator_type = self.previous.token_type.clone();
        let rule = get_rule(operator_type.clone());
        self.parse_precedence(rule.precedence.next().unwrap());

        match operator_type {
            TokenType::Plus => self.emit_byte(OpCode::OpAdd as u8),
            TokenType::Minus => self.emit_byte(OpCode::OpSubtract as u8),
            TokenType::Star => self.emit_byte(OpCode::OpMultiply as u8),
            TokenType::Slash => self.emit_byte(OpCode::OpDivide as u8),
            _ => (),
        }
    }
}

fn get_rule(token_type: TokenType) -> ParseRule {
    match token_type {
        TokenType::LeftParen => ParseRule { prefix: Some(|p| p.grouping()), infix: None, precedence: Precedence::None },
        TokenType::RightParen => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
        TokenType::LeftBrace => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
        TokenType::RightBrace => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
        TokenType::Comma => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
        TokenType::Dot => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
        TokenType::Minus => ParseRule { prefix: Some(|p| p.unary()), infix: Some(|p| p.binary()), precedence: Precedence::Term },
        TokenType::Plus => ParseRule { prefix: None, infix: Some(|p| p.binary()), precedence: Precedence::Term },
        TokenType::Semicolon => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
        TokenType::Slash => ParseRule { prefix: None, infix: Some(|p| p.binary()), precedence: Precedence::Factor },
        TokenType::Star => ParseRule { prefix: None, infix: Some(|p| p.binary()), precedence: Precedence::Factor },
        TokenType::Bang => ParseRule { prefix: Some(|p| p.unary()), infix: None, precedence: Precedence::None },
        TokenType::BangEqual => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
        TokenType::Equal => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
        TokenType::EqualEqual => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
        TokenType::Greater => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
        TokenType::GreaterEqual => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
        TokenType::Less => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
        TokenType::LessEqual => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
        TokenType::Identifier => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
        TokenType::String => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
        TokenType::Number => ParseRule { prefix: Some(|p| p.number()), infix: None, precedence: Precedence::None },
        TokenType::And => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
        TokenType::Class => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
        TokenType::Else => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
        TokenType::False => ParseRule { prefix: Some(|p| p.literal()), infix: None, precedence: Precedence::None },
        TokenType::For => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
        TokenType::Fun => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
        TokenType::If => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
        TokenType::Nil => ParseRule { prefix: Some(|p| p.literal()), infix: None, precedence: Precedence::None },
        TokenType::Or => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
        TokenType::Print => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
        TokenType::Return => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
        TokenType::Super => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
        TokenType::This => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
        TokenType::True => ParseRule { prefix: Some(|p| p.literal()), infix: None, precedence: Precedence::None },
        TokenType::Var => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
        TokenType::While => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
        TokenType::Eof => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
        TokenType::Error => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
    }
}

pub fn compile(source: String) -> Result<Chunk, String> {
    let scanner = Scanner::new(&source);
    let mut parser = Parser::new(scanner);

    parser.advance();
    parser.expression();
    parser.end_compile();
    

    if parser.had_error {
        Err("Compilation failed".to_string())
    } else {
        Ok(parser.current_chunk)
    }
}
