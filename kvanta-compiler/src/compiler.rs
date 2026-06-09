// use std::rc::Rc;

// use kvanta_compiler::chunk::*;
// //use kvanta_compiler::debug::*;
// use kvanta_compiler::value::*;
// use kvanta_compiler::vm::*;

// fn main() {
//     let mut chunk = Chunk::new();
//     chunk.push_code(OpCode::Constant, 0);
//     chunk.push(constant as u8, 0);
//     chunk.push_code(OpCode::Negate, 0);
//     chunk.push_code(OpCode::Return, 123);
//     let _ = interpret(Rc::new(chunk));
// }


use crate::{chunk::{Chunk, OpCode}, value::Value, vm::CommonMemory};

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
    prefix: Option<fn(&mut Compiler, bool)>,
    infix: Option<fn(&mut Compiler, bool)>,
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

    fn match_token(&mut self, token_type: TokenType) -> bool {
        if !self.check(token_type) {
            return false;
        }
        self.advance();
        true
    }

    fn check(&self, token_type: TokenType) -> bool {
        self.current.token_type == token_type
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
        }
    }

    fn synchronize(&mut self) {
        self.panic_mode = false;

        while self.current.token_type != TokenType::Eof {
            if self.previous.token_type == TokenType::Semicolon {
                return;
            }
            match self.current.token_type {
                TokenType::Class | TokenType::Fun | TokenType::Var | TokenType::For | TokenType::If | TokenType::While | TokenType::Print | TokenType::Return => return,
                _ => self.advance(),
            }
        }
    }   

    

    
}

#[derive(Debug)]
struct LocalVariable {
    name: String,
    depth: Option<usize>,
}

struct Compiler<'comp> {
    parser: Parser<'comp>,
    locals: Vec<LocalVariable>,
    scope_depth: usize,
    pub current_chunk: Chunk,
    common: &'comp mut CommonMemory,
}

impl <'comp> Compiler<'comp> {
    fn new(parser: Parser<'comp>, common: &'comp mut CommonMemory) -> Self {
        Compiler {
            parser,
            locals: vec![],
            scope_depth: 0,
            current_chunk: Chunk::new(),
            common,
        }
    }

    fn error_at_current(&mut self, message: &str) {
        if self.parser.panic_mode {
            return;
        }
        self.parser.panic_mode = true;
        let cur = self.parser.current.clone();
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
        self.parser.had_error = true;
    }

    fn define_variable(&mut self, global: usize) {
        if self.scope_depth > 0 {
            self.mark_initialized();
            return;
        }
        self.emit_bytes(OpCode::DefineGlobal as u8, global as u8);
    }

    fn declare_variable(&mut self) {
        if self.scope_depth == 0 {
            return;
        }

        let mut duplicate_found = false;
        for local in self.locals.iter().rev() {
            if let Some(d) = local.depth && d < self.scope_depth {
                break;
            }
            if local.name == self.parser.previous.lexeme {
                duplicate_found = true;
                break;
            }
        }
        
        if duplicate_found {
            self.error_at_current("Already a variable with this name in this scope.");
        }
        self.add_local(self.parser.previous.clone());
    }

    fn mark_initialized(&mut self) {
        match self.locals.last_mut() {
            None => (),
            Some(local) => local.depth = Some(self.scope_depth),
        }
    }

    fn add_local(&mut self, name: Token) {
        if self.locals.len() >= u8::MAX as usize {
            self.error_at_current("Too many local variables in function.");
            return;
        }
        self.locals.push(LocalVariable { name: name.lexeme.to_string(), depth: None });
    }

    fn emit_byte(&mut self, byte: u8) {
        self.current_chunk.push(byte, self.parser.previous.line);
    }

    fn emit_bytes(&mut self, byte1: u8, byte2: u8) {
        self.emit_byte(byte1);
        self.emit_byte(byte2);
    }

    pub fn make_constant(&mut self, value: Value) -> usize {
        self.common.constants.push(value);
        self.common.constants.len() - 1
    }

    fn emit_constant(&mut self, value: Value) {
        let constant = self.make_constant(value);
        if constant > u8::MAX as usize {
            self.error_at_current("Too many constants in one chunk.");
            return;
        }
        self.emit_bytes(OpCode::Constant as u8, constant as u8);
    }

    fn end_compile(&mut self) {
        self.parser.consume(TokenType::Eof, "Expect end of expression.");
        self.emit_byte(OpCode::Return as u8); // Return
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.parser.advance();
        let can_assign = precedence <= Precedence::Assignment;
        let prefix_rule = get_rule(self.parser.previous.token_type.clone()).prefix;
        if let Some(prefix_rule) = prefix_rule {
            prefix_rule(self, can_assign);
        } else {
            self.error_at_current("Expect expression.");
            return;
        }

        while precedence <= get_rule(self.parser.current.token_type.clone()).precedence {
            self.parser.advance();
            let infix_rule = get_rule(self.parser.previous.token_type.clone()).infix;
            if let Some(infix_rule) = infix_rule {
                infix_rule(self, can_assign);
            }
        }

        if !can_assign && self.parser.match_token(TokenType::Equal) {
            self.error_at_current("Invalid assignment target.");
        }
    }

    fn declaration(&mut self) {
        if self.parser.match_token(TokenType::Var) {
            self.var_declaration();
        } else {
            self.statement();
        }

        if self.parser.panic_mode {
            self.parser.synchronize();
        }
    }

    fn var_declaration(&mut self) {
        let global_var_id = self.parse_variable("Expect variable name.");

        if self.parser.match_token(TokenType::Equal) {
            self.expression();
        } else {
            self.emit_byte(OpCode::Nil as u8);
        }

        self.parser.consume(TokenType::Semicolon, "Expect ';' after variable declaration.");
        self.define_variable(global_var_id);
    }

    fn parse_variable(&mut self, error_message: &str) -> usize {
        self.parser.consume(TokenType::Identifier, error_message);

        self.declare_variable();
        if self.scope_depth > 0 {
            return 0;
        }

        self.identifier_constant(self.parser.previous.lexeme.to_string())
    }

    fn identifier_constant(&mut self, name: String) -> usize {
        let string_id = self.take_string(name);
        let id = self.make_constant(Value::String(string_id));
        if id > u8::MAX as usize {
            self.error_at_current("Too many constants in one chunk.");
            return 0;
        }
        id
    }

    fn compile(&mut self) -> Result<Chunk, String> {
        self.parser.advance();
        while self.parser.current.token_type != TokenType::Eof {
            self.declaration();
        }
        self.end_compile();

        if self.parser.had_error {
            Err("Compile error".to_string())
        } else {
            Ok(self.current_chunk.clone())
        }
    }


    fn statement(&mut self) {
        if self.parser.match_token(TokenType::Print) {
            self.print_statement();
        } else if self.parser.match_token(TokenType::For) {
            self.for_statement();
        } else if self.parser.match_token(TokenType::If) {
            self.if_statement();
        }else if self.parser.match_token(TokenType::While) {
            self.while_statement();
        } else if self.parser.match_token(TokenType::LeftBrace) {
            self.begin_scope();
            self.block();
            self.end_scope();
        } else {
            self.expression_statement();
        }
    }

    fn for_statement(&mut self) {
        self.begin_scope();
        self.parser.consume(TokenType::LeftParen, "Expect '(' after 'for'.");

        if self.parser.match_token(TokenType::Semicolon) {
            // No initializer.
        } else if self.parser.match_token(TokenType::Var) {
            self.var_declaration();
        } else {
            self.expression_statement();
        }

        let mut loop_start = self.current_chunk.len();

        let exit_jump = if !self.parser.match_token(TokenType::Semicolon) {
            self.expression();
            self.parser.consume(TokenType::Semicolon, "Expect ';' after loop condition.");
            let r = self.emit_jump(OpCode::JumpIfFalse);
            self.emit_byte(OpCode::Pop as u8);
            r
        } else {
            0
        };

        if !self.parser.match_token(TokenType::RightParen) {
            let body_jump = self.emit_jump(OpCode::Jump);
            let increment_start = self.current_chunk.len();
            self.expression();
            self.emit_byte(OpCode::Pop as u8);
            self.parser.consume(TokenType::RightParen, "Expect ')' after for clauses.");

            self.emit_loop(loop_start);
            loop_start = increment_start;
            self.patch_jump(body_jump);
        }

        self.statement();
        self.emit_loop(loop_start);

        if exit_jump != 0 {
            self.patch_jump(exit_jump);
            self.emit_byte(OpCode::Pop as u8);
        }

        self.end_scope();
    }

    fn while_statement(&mut self) {
        let loop_start = self.current_chunk.len();
        self.parser.consume(TokenType::LeftParen, "Expect '(' after 'while'.");
        self.expression();
        self.parser.consume(TokenType::RightParen, "Expect ')' after condition.");

        let exit_jump = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_byte(OpCode::Pop as u8);
        self.statement();
        self.emit_loop(loop_start);

        self.patch_jump(exit_jump);
        self.emit_byte(OpCode::Pop as u8);
    }

    fn emit_loop(&mut self, loop_start: usize) {
        let offset = self.current_chunk.len() - loop_start + 3;
        if offset > u16::MAX as usize {
            self.error_at_current("Loop body too large.");
        }
        self.emit_byte(OpCode::Loop as u8);
        self.emit_byte(((offset >> 8) & 0xff) as u8);
        self.emit_byte((offset & 0xff) as u8);
    }

    fn if_statement(&mut self) {
        self.parser.consume(TokenType::LeftParen, "Expect '(' after 'if'.");
        self.expression();
        self.parser.consume(TokenType::RightParen, "Expect ')' after condition.");
        let then_jump = self.emit_jump(OpCode::JumpIfFalse);

        self.emit_byte(OpCode::Pop as u8);
        self.statement();

        let else_jump = self.emit_jump(OpCode::Jump);
        self.patch_jump(then_jump);
        self.emit_byte(OpCode::Pop as u8);

        if self.parser.match_token(TokenType::Else) {
            self.statement();
        }
        self.patch_jump(else_jump);
    }

    fn emit_jump(&mut self, instruction: OpCode) -> usize {
        self.emit_byte(instruction as u8);
        self.emit_byte(0xff);
        self.emit_byte(0xff);
        self.current_chunk.len() - 2
    }

    fn patch_jump(&mut self, offset: usize) {
        let jump = self.current_chunk.len() - offset - 2;
        if jump > u16::MAX as usize {
            self.error_at_current("Too much code to jump over.");
        }

        self.current_chunk.chunk[offset] = ((jump >> 8) & 0xff) as u8;
        self.current_chunk.chunk[offset + 1] = (jump & 0xff) as u8;
    }

    fn block(&mut self) {
        while !self.parser.check(TokenType::RightBrace) && self.parser.current.token_type != TokenType::Eof {
            self.declaration();
        }

        self.parser.consume(TokenType::RightBrace, "Expect '}' after block.");
    }

    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        while !self.locals.is_empty() && self.locals.last().unwrap().depth == Some(self.scope_depth) {
            self.emit_byte(OpCode::Pop as u8);
            self.locals.pop();
        }
        self.scope_depth -= 1;
    }

    fn print_statement(&mut self) {
        self.expression();
        self.parser.consume(TokenType::Semicolon, "Expect ';' after value.");
        self.emit_byte(OpCode::Print as u8);
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.parser.consume(TokenType::Semicolon, "Expect ';' after expression.");
        self.emit_byte(OpCode::Pop as u8);
    }
    
    fn number(&mut self) {
        let value = self.parser.previous.lexeme.parse::<f32>().unwrap();
        self.emit_constant(Value::Float(value));
    }

    fn copy_string(&mut self, s: &str) -> i32 {
        self.common.heap.push(s.to_string());
        (self.common.heap.len() - 1) as i32
    }

    fn take_string(&mut self, s: String) -> i32 {
        self.common.heap.push(s);
        (self.common.heap.len() - 1) as i32
    }

    fn string(&mut self) {
        let value = self.copy_string(self.parser.previous.lexeme);
        self.emit_constant(Value::String(value));
    }

    fn literal(&mut self) {
        match self.parser.previous.token_type {
            TokenType::False => self.emit_byte(OpCode::False as u8),
            TokenType::True => self.emit_byte(OpCode::True as u8),
            TokenType::Nil => self.emit_byte(OpCode::Nil as u8),
            _ => (),
        }
    }

    fn grouping(&mut self) {
        self.expression();
        self.parser.consume(TokenType::RightParen, "Expect ')' after expression.");
    }

    fn unary(&mut self) {
        let operator_type = self.parser.previous.token_type.clone();
        self.parse_precedence(Precedence::Unary);
        match operator_type {
            TokenType::Minus => self.emit_byte(OpCode::Negate as u8),
            TokenType::Bang => self.emit_byte(OpCode::Not as u8),
            _ => (),
        }
    }

    fn binary(&mut self) {
        let operator_type = self.parser.previous.token_type.clone();
        let rule = get_rule(operator_type.clone());
        self.parse_precedence(rule.precedence.next().unwrap());

        match operator_type {
            TokenType::Plus => self.emit_byte(OpCode::Add as u8),
            TokenType::Minus => self.emit_byte(OpCode::Subtract as u8),
            TokenType::Star => self.emit_byte(OpCode::Multiply as u8),
            TokenType::Slash => self.emit_byte(OpCode::Divide as u8),
            TokenType::BangEqual => self.emit_bytes(OpCode::Equal as u8, OpCode::Not as u8), // TODO: Implement OpNotEqual
            TokenType::EqualEqual => self.emit_byte(OpCode::Equal as u8),
            TokenType::Greater => self.emit_byte(OpCode::Greater as u8),
            TokenType::GreaterEqual => self.emit_bytes(OpCode::Less as u8, OpCode::Not as u8),
            TokenType::Less => self.emit_byte(OpCode::Less as u8),
            TokenType::LessEqual => self.emit_bytes(OpCode::Greater as u8, OpCode::Not as u8),
            _ => (),
        }
    }

    fn variable(&mut self, can_assign: bool) {
        self.named_variable(self.parser.previous.lexeme.to_string(), can_assign);
    }

    fn named_variable(&mut self, name: String, can_assign: bool) {
        let mut set = OpCode::SetGlobal as u8;
        let mut get = OpCode::GetGlobal as u8;
        let id : u8 = {
            if let Some(idx) = self.resolve_local(&name) {
                set = OpCode::SetLocal as u8;
                get = OpCode::GetLocal as u8;
                idx
            } else {
                self.identifier_constant(name) as u8
            }
        };
        
        if can_assign && self.parser.match_token(TokenType::Equal) {
           self.expression();
           self.emit_bytes(set, id);
       } else {
            self.emit_bytes(get, id); 
       }
    }

    fn resolve_local(&mut self, name: &str) -> Option<u8> {
        let mut res : Option<(u8, Option<usize>)> = None;
        for (i, local) in self.locals.iter().enumerate().rev() {
            if local.name == name {
                res = Some((i as u8, local.depth));
            }
        }
        
        match res {
            None => None,
            Some((idx, depth)) => {
                if depth.is_none() {
                    self.error_at_current("Can't read local variable in its own initializer.");
                    None
                } else {
                    Some(idx)
                }
            }
        }
    }

    fn and(&mut self) {
        let end_jump = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_byte(OpCode::Pop as u8);
        self.parse_precedence(Precedence::And);
        self.patch_jump(end_jump);
    }

    fn or(&mut self) {
        let else_jump = self.emit_jump(OpCode::JumpIfFalse);
        let end_jump = self.emit_jump(OpCode::Jump);

        self.patch_jump(else_jump);
        self.emit_byte(OpCode::Pop as u8);

        self.parse_precedence(Precedence::Or);
        self.patch_jump(end_jump);
    }

}


fn get_rule(token_type: TokenType) -> ParseRule {
    match token_type {
        TokenType::LeftParen => ParseRule { prefix: Some(|p, _| p.grouping()), infix: None, precedence: Precedence::None },
        TokenType::RightParen => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
        TokenType::LeftBrace => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
        TokenType::RightBrace => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
        TokenType::Comma => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
        TokenType::Dot => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
        TokenType::Minus => ParseRule { prefix: Some(|p, _| p.unary()), infix: Some(|p, _| p.binary()), precedence: Precedence::Term },
        TokenType::Plus => ParseRule { prefix: None, infix: Some(|p, _| p.binary()), precedence: Precedence::Term },
        TokenType::Semicolon => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
        TokenType::Slash => ParseRule { prefix: None, infix: Some(|p, _| p.binary()), precedence: Precedence::Factor },
        TokenType::Star => ParseRule { prefix: None, infix: Some(|p, _| p.binary()), precedence: Precedence::Factor },
        TokenType::Bang => ParseRule { prefix: Some(|p, _| p.unary()), infix: None, precedence: Precedence::None },
        TokenType::BangEqual => ParseRule { prefix: None, infix: Some(|p, _| p.binary()), precedence: Precedence::Equality },
        TokenType::Equal => ParseRule { prefix: None, infix: Some(|p, _| p.binary()), precedence: Precedence::Equality },
        TokenType::EqualEqual => ParseRule { prefix: None, infix: Some(|p, _| p.binary()), precedence: Precedence::Equality },
        TokenType::Greater => ParseRule { prefix: None, infix: Some(|p, _| p.binary()), precedence: Precedence::Comparison },
        TokenType::GreaterEqual => ParseRule { prefix: None, infix: Some(|p, _| p.binary()), precedence: Precedence::Comparison },
        TokenType::Less => ParseRule { prefix: None, infix: Some(|p, _| p.binary()), precedence: Precedence::Comparison },
        TokenType::LessEqual => ParseRule { prefix: None, infix: Some(|p, _| p.binary()), precedence: Precedence::Comparison },
        TokenType::Identifier => ParseRule { prefix: Some(|p, can_assign| p.variable(can_assign)), infix: None, precedence: Precedence::None },
        TokenType::String => ParseRule { prefix: Some(|p, _| p.string()), infix: None, precedence: Precedence::None },
        TokenType::Number => ParseRule { prefix: Some(|p, _| p.number()), infix: None, precedence: Precedence::None },
        TokenType::And => ParseRule { prefix: None, infix: Some(|p, _| p.and()), precedence: Precedence::And },
        TokenType::Class => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
        TokenType::Else => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
        TokenType::False => ParseRule { prefix: Some(|p, _| p.literal()), infix: None, precedence: Precedence::None },
        TokenType::For => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
        TokenType::Fun => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
        TokenType::If => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
        TokenType::Nil => ParseRule { prefix: Some(|p, _| p.literal()), infix: None, precedence: Precedence::None },
        TokenType::Or => ParseRule { prefix: None, infix: Some(|p, _| p.or()), precedence: Precedence::Or },
        TokenType::Print => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
        TokenType::Return => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
        TokenType::Super => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
        TokenType::This => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
        TokenType::True => ParseRule { prefix: Some(|p, _| p.literal()), infix: None, precedence: Precedence::None },
        TokenType::Var => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
        TokenType::While => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
        TokenType::Eof => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
        TokenType::Error => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
    }
}

pub fn compile(source: String, common: &mut CommonMemory) -> Result<Chunk, String> {
    let scanner = Scanner::new(&source);
    let parser = Parser::new(scanner);
    let mut compiler = Compiler::new(parser, common);

    compiler.compile()
}
