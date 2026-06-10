
#![allow(dead_code)]

use crate::{ast::{ExpressionAst, ProgramAst, StatementAst, TypeAst}, chunk::{Chunk, OpCode}, scanner::{Token, TokenType}, value::{new_function, Function, Value}, vm::CommonMemory};

pub struct Tokenizer<'comp> {
    current: Token<'comp>,
    previous: Token<'comp>,
    tokens: Vec<Token<'comp>>,
    token_index: usize,
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
    prefix: Option<fn(&mut Parser<'_, '_>, bool)>,
    infix: Option<fn(&mut Parser<'_, '_>, bool)>,
    precedence: Precedence,
}



impl<'comp> Tokenizer<'comp> {
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
            self.current = self.next_token();
            if self.current.token_type != TokenType::Error {
                break;
            }
            self.error_at_current(self.current.lexeme);
        }
    }

    fn next_token(&mut self) -> Token<'comp> {
        if self.token_index >= self.tokens.len() {
            return Token {
                token_type: TokenType::Eof,
                lexeme: "",
                line: 0,
            };
        }
        let token = self.tokens[self.token_index].clone();
        self.token_index += 1;
        token
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

    fn is_type_token(&self) -> bool {
        matches!(self.current.token_type, TokenType::Int | TokenType::Float | TokenType::Color
             | TokenType::Bool | TokenType::Array)
    }

    fn check(&self, token_type: TokenType) -> bool {
        self.current.token_type == token_type
    }

    pub fn new(tokens: Vec<Token<'comp>>) -> Self {
        let dummy_token = Token {
            token_type: TokenType::Eof,
            lexeme: "",
            line: 0,
        };

        Tokenizer {
            current: dummy_token.clone(),
            previous: dummy_token,
            had_error: false,
            panic_mode: false,
            tokens,
            token_index: 0,
        }
    }

    fn synchronize(&mut self) {
        self.panic_mode = false;

        while self.current.token_type != TokenType::Eof {
            if self.previous.token_type == TokenType::Semicolon {
                return;
            }
            match self.current.token_type {
                TokenType::Class | TokenType::Fun | TokenType::For | TokenType::If | TokenType::While | TokenType::Print | TokenType::Return => return,
                _ => self.advance(),
            }
        }
    }   

    

    
}

#[derive(Debug)]
pub struct LocalVariable {
    name: String,
    depth: Option<usize>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum FunctionType {
    Function,
    Script,
}

pub struct Parser<'src, 'a> {
    function_type: FunctionType,
    function: Function,
    tokenizer: &'a mut Tokenizer<'src>,
    locals: Vec<LocalVariable>,
    scope_depth: usize,
    common: &'a mut CommonMemory,
}

impl<'src, 'a> Parser<'src, 'a> {
    pub fn new(tokenizer: &'a mut Tokenizer<'src>, common: &'a mut CommonMemory) -> Self {
        Parser {
            function_type: FunctionType::Script,
            function: new_function("MAIN_SCRIPT".to_string()),
            tokenizer,
            locals: vec![],
            scope_depth: 0,
            common,
        }
    }

    pub fn compile(&mut self) -> Result<ProgramAst, String> {

        self.tokenizer.advance();
        if self.tokenizer.current.token_type == TokenType::Fun || self.tokenizer.current.token_type == TokenType::Global {
            self.forest()
        } else {
            self.script()
        }
        // while self.tokenizer.current.token_type != TokenType::Eof {
        //     self.declaration();
        // }
        // self.end_compile();

        //TODO: Handle errors properly
        // if self.tokenizer.had_error {
        //     Err("Compile error".to_string())
        // } else {
        //     Ok(ProgramAst::Forest)
        // }
    }

    fn declaration(&mut self) -> Result<StatementAst, String> {
        let result = if self.tokenizer.is_type_token() {
            self.var_declaration()
        } else {
            self.statement()
        };

        if self.tokenizer.panic_mode {
            self.tokenizer.synchronize();
        }
        Ok(result)
    }

    fn forest(&mut self) -> Result<ProgramAst, String> {
        Ok(ProgramAst::Forest)
    }

    fn script(&mut self) -> Result<ProgramAst, String> {
        let mut result : Vec<StatementAst> = vec![];
        while self.tokenizer.current.token_type != TokenType::Eof {
            result.push(self.declaration()?);
        }
        self.end_compile();

        if self.tokenizer.had_error {
            Err("Compile error".to_string())
        } else {
            Ok(ProgramAst::Script(result))
        }
    }

    ///////////////////////////////////////////////////

    fn current_chunk(&mut self) -> &mut Chunk {
        &mut self.function.chunk
    }

    
    fn compiled_function(&self) -> Function {
        self.function.clone()
    }

    fn error_at_current(&mut self, message: &str) {
        if self.tokenizer.panic_mode {
            return;
        }
        self.tokenizer.panic_mode = true;
        let cur = self.tokenizer.current.clone();
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
        self.tokenizer.had_error = true;
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
            if local.name == self.tokenizer.previous.lexeme {
                duplicate_found = true;
                break;
            }
        }
        
        if duplicate_found {
            self.error_at_current("Already a variable with this name in this scope.");
        }
        self.add_local(self.tokenizer.previous.clone());
    }

    fn mark_initialized(&mut self) {
        if self.scope_depth == 0 {
            return;
        }

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
        let line = self.tokenizer.previous.line;
        self.current_chunk().push(byte, line);
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
        self.tokenizer.consume(TokenType::Eof, "Expect end of expression.");
        self.emit_byte(OpCode::Nil as u8);
        self.emit_byte(OpCode::Return as u8);
    }

    fn expression(&mut self) -> ExpressionAst {
        self.parse_precedence(Precedence::Assignment);
        ExpressionAst {}
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.tokenizer.advance();
        let can_assign = precedence <= Precedence::Assignment;
        let prefix_rule = get_rule(&self.tokenizer.previous.token_type).prefix;
        if let Some(prefix_rule) = prefix_rule {
            prefix_rule(self, can_assign);
        } else {
            self.error_at_current("Expect expression.");
            return;
        }

        while precedence <= get_rule(&self.tokenizer.current.token_type).precedence {
            self.tokenizer.advance();
            let infix_rule = get_rule(&self.tokenizer.previous.token_type).infix;
            if let Some(infix_rule) = infix_rule {
                infix_rule(self, can_assign);
            }
        }

        if !can_assign && self.tokenizer.match_token(TokenType::Equal) {
            self.error_at_current("Invalid assignment target.");
        }
    }

    

    fn fun_declaration(&mut self) {
        let _global_var_id = self.parse_variable("Expect function name.");
        self.mark_initialized();
        self.function_type = FunctionType::Function;
        self.function();
        //self.define_variable(global_var_id);
    }

    fn var_declaration(&mut self) -> StatementAst {
        let type_token = self.variable_type();
        let global_var_id = self.parse_variable("Expect variable name.");
        self.tokenizer.consume(TokenType::Equal, "Expect '=' after variable declaration.");
        let expr = self.expression();
        self.tokenizer.consume(TokenType::Semicolon, "Expect ';' after variable declaration.");
       // self.define_variable(global_var_id);
        StatementAst::Var(type_token, global_var_id, expr)
    }

    fn variable_type(&mut self) -> TypeAst {
        if self.tokenizer.match_token(TokenType::Int) {
            TypeAst::Int
        } else if self.tokenizer.match_token(TokenType::Float) {
            TypeAst::Float
        } else if self.tokenizer.match_token(TokenType::Color) {
            TypeAst::Color
        } else if self.tokenizer.match_token(TokenType::Bool) {
            TypeAst::Bool
        } else if self.tokenizer.match_token(TokenType::Array) {
            self.tokenizer.consume(TokenType::Less, "Expect '<' after array type.");
            let inner_type = self.variable_type();
            self.tokenizer.consume(TokenType::Comma, "Expect ',' after array inner type.");
            let array_size = self.expression();
            self.tokenizer.consume(TokenType::Greater, "Expect '>' after array type.");
            TypeAst::Array(Box::new(inner_type), array_size)
        } else {
            self.error_at_current("Expect variable type.");
            TypeAst::Int
        }
    }

    fn parse_variable(&mut self, error_message: &str) -> String {
        self.tokenizer.consume(TokenType::Identifier, error_message);
        self.tokenizer.previous.lexeme.to_string()
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

    fn statement(&mut self) -> StatementAst {
        if self.tokenizer.match_token(TokenType::Print) {
            self.print_statement()
        } else if self.tokenizer.match_token(TokenType::For) {
            self.for_statement()
        } else if self.tokenizer.match_token(TokenType::If) {
            self.if_statement()
        } else if self.tokenizer.match_token(TokenType::Return) {
            self.return_statement()
        } else if self.tokenizer.match_token(TokenType::While) {
            self.while_statement()
        } else if self.tokenizer.match_token(TokenType::LeftBrace) {
            self.block()
        } else {
            self.expression_statement()
        }
    }

    fn return_statement(&mut self) -> StatementAst {
        if self.function_type == FunctionType::Script {
            self.error_at_current("Can't return from top-level code.");
        }
        if self.tokenizer.match_token(TokenType::Semicolon) {
            self.emit_byte(OpCode::Nil as u8);
        } else {
            self.expression();
            self.tokenizer.consume(TokenType::Semicolon, "Expect ';' after return value.");
        }
        self.emit_byte(OpCode::Return as u8);
        StatementAst::Return
    }

    fn for_statement(&mut self) -> StatementAst {
        self.begin_scope();
        self.tokenizer.consume(TokenType::LeftParen, "Expect '(' after 'for'.");

        if self.tokenizer.match_token(TokenType::Semicolon) {
            // No initializer.
        } else if self.tokenizer.is_type_token() {
            self.var_declaration();
        } else {
            self.expression_statement();
        }

        let mut loop_start = self.current_chunk().len();

        let exit_jump = if !self.tokenizer.match_token(TokenType::Semicolon) {
            self.expression();
            self.tokenizer.consume(TokenType::Semicolon, "Expect ';' after loop condition.");
            let r = self.emit_jump(OpCode::JumpIfFalse);
            self.emit_byte(OpCode::Pop as u8);
            r
        } else {
            0
        };

        if !self.tokenizer.match_token(TokenType::RightParen) {
            let body_jump = self.emit_jump(OpCode::Jump);
            let increment_start = self.current_chunk().len();
            self.expression();
            self.emit_byte(OpCode::Pop as u8);
            self.tokenizer.consume(TokenType::RightParen, "Expect ')' after for clauses.");

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
        StatementAst::For
    }

    fn while_statement(&mut self) -> StatementAst {
        let loop_start = self.current_chunk().len();
        self.tokenizer.consume(TokenType::LeftParen, "Expect '(' after 'while'.");
        self.expression();
        self.tokenizer.consume(TokenType::RightParen, "Expect ')' after condition.");

        let exit_jump = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_byte(OpCode::Pop as u8);
        self.statement();
        self.emit_loop(loop_start);

        self.patch_jump(exit_jump);
        self.emit_byte(OpCode::Pop as u8);
        StatementAst::While
    }

    fn emit_loop(&mut self, loop_start: usize) {
        let offset = self.current_chunk().len() - loop_start + 3;
        if offset > u16::MAX as usize {
            self.error_at_current("Loop body too large.");
        }
        self.emit_byte(OpCode::Loop as u8);
        self.emit_byte(((offset >> 8) & 0xff) as u8);
        self.emit_byte((offset & 0xff) as u8);
    }

    fn if_statement(&mut self) -> StatementAst {
        self.tokenizer.consume(TokenType::LeftParen, "Expect '(' after 'if'.");
        self.expression();
        self.tokenizer.consume(TokenType::RightParen, "Expect ')' after condition.");
        let then_jump = self.emit_jump(OpCode::JumpIfFalse);

        self.emit_byte(OpCode::Pop as u8);
        self.statement();

        let else_jump = self.emit_jump(OpCode::Jump);
        self.patch_jump(then_jump);
        self.emit_byte(OpCode::Pop as u8);

        if self.tokenizer.match_token(TokenType::Else) {
            self.statement();
        }
        self.patch_jump(else_jump);
        StatementAst::If
    }

    fn emit_jump(&mut self, instruction: OpCode) -> usize {
        self.emit_byte(instruction as u8);
        self.emit_byte(0xff);
        self.emit_byte(0xff);
        self.current_chunk().len() - 2
    }

    fn patch_jump(&mut self, offset: usize) {
        let jump = self.current_chunk().len() - offset - 2;
        if jump > u16::MAX as usize {
            self.error_at_current("Too much code to jump over.");
        }

        self.current_chunk().chunk[offset] = ((jump >> 8) & 0xff) as u8;
        self.current_chunk().chunk[offset + 1] = (jump & 0xff) as u8;
    }

    fn block(&mut self) -> StatementAst {
        while !self.tokenizer.check(TokenType::RightBrace) && self.tokenizer.current.token_type != TokenType::Eof {
            let _ = self.declaration();
        }

        self.tokenizer.consume(TokenType::RightBrace, "Expect '}' after block.");
        StatementAst::Block
    }

    fn inner_function(&mut self) {
        self.begin_scope();
        self.tokenizer.consume(TokenType::LeftParen, "Expect '(' after function name.");
        if !self.tokenizer.check(TokenType::RightParen) {
            loop {
                self.function.arity += 1;
                if self.function.arity > u8::MAX as usize {
                    self.error_at_current("Can't have more than 255 parameters.");
                }
                let _name = self.parse_variable("Expect param name");
                //self.define_variable(name);
                if !self.tokenizer.match_token(TokenType::Comma) {
                    break;
                }
            }
        }
        self.tokenizer.consume(TokenType::RightParen, "Expect ')' after parameters.");
        self.tokenizer.consume(TokenType::LeftBrace, "Expect '{' before function body.");
        self.block();
    }

    fn get_function_name(&mut self) {
        self.function.name = self.tokenizer.previous.lexeme.to_string();
    }

    fn function(&mut self) {
        let next_function = {
            let mut next_parser = Parser::new(self.tokenizer, self.common);
            next_parser.function_type = FunctionType::Function;
            next_parser.get_function_name();
            next_parser.inner_function();
            next_parser.compiled_function()
        };
        let fun_id = self.take_function(next_function);
        let function_id = self.make_constant(Value::Function(fun_id));
        self.emit_bytes(OpCode::Constant as u8, function_id as u8);
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

    fn print_statement(&mut self) -> StatementAst {
        self.expression();
        self.tokenizer.consume(TokenType::Semicolon, "Expect ';' after value.");
        StatementAst::Print
    }

    fn expression_statement(&mut self) -> StatementAst {
        self.expression();
        self.tokenizer.consume(TokenType::Semicolon, "Expect ';' after expression.");
        self.emit_byte(OpCode::Pop as u8);
        StatementAst::Expression(ExpressionAst {})
    }
    
    fn number(&mut self) {
        let value = self.tokenizer.previous.lexeme.parse::<f32>().unwrap();
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

    fn take_function(&mut self, f: Function) -> i32 {
        self.common.functions.push(f);
        (self.common.functions.len() - 1) as i32
    }

    fn string(&mut self) {
        let value = self.copy_string(self.tokenizer.previous.lexeme);
        self.emit_constant(Value::String(value));
    }

    fn literal(&mut self) {
        match self.tokenizer.previous.token_type {
            TokenType::False => self.emit_byte(OpCode::False as u8),
            TokenType::True => self.emit_byte(OpCode::True as u8),
            TokenType::Nil => self.emit_byte(OpCode::Nil as u8),
            _ => (),
        }
    }

    fn grouping(&mut self) {
        self.expression();
        self.tokenizer.consume(TokenType::RightParen, "Expect ')' after expression.");
    }

    fn unary(&mut self) {
        let operator_type = self.tokenizer.previous.token_type.clone();
        self.parse_precedence(Precedence::Unary);
        match operator_type {
            TokenType::Minus => self.emit_byte(OpCode::Negate as u8),
            TokenType::Bang => self.emit_byte(OpCode::Not as u8),
            _ => (),
        }
    }

    fn binary(&mut self) {
        let operator_type = self.tokenizer.previous.token_type.clone();
        let rule = get_rule(&operator_type);
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
        self.named_variable(self.tokenizer.previous.lexeme.to_string(), can_assign);
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
        
        if can_assign && self.tokenizer.match_token(TokenType::Equal) {
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
                break;
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

    fn argument_list(&mut self) -> usize {
        let mut arg_count = 0;
        if !self.tokenizer.check(TokenType::RightParen) {
            loop {
                self.expression();
                arg_count += 1;
                if arg_count > u8::MAX as usize {
                    self.error_at_current("Can't have more than 255 arguments.");
                }
                if !self.tokenizer.match_token(TokenType::Comma) {
                    break;
                }
            }
        }
        self.tokenizer.consume(TokenType::RightParen, "Expect ')' after arguments.");
        arg_count
    }

    fn call(&mut self) {
        let arg_count = self.argument_list();
        if arg_count > u8::MAX as usize {
            self.error_at_current("Can't have more than 255 arguments.");
        }
        self.emit_bytes(OpCode::Call as u8, arg_count as u8);
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


fn get_rule(token_type: &TokenType) -> ParseRule {
    match token_type {
        TokenType::LeftParen => ParseRule { prefix: Some(|p, _| p.grouping()), infix: Some(|p, _| p.call()), precedence: Precedence::Call },
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
        TokenType::Int => ParseRule { prefix: Some(|p, _| p.literal()), infix: None, precedence: Precedence::None },
        TokenType::Float => ParseRule { prefix: Some(|p, _| p.literal()), infix: None, precedence: Precedence::None },
        TokenType::Color => ParseRule { prefix: Some(|p, _| p.literal()), infix: None, precedence: Precedence::None },
        TokenType::Bool => ParseRule { prefix: Some(|p, _| p.literal()), infix: None, precedence: Precedence::None },
        TokenType::Array => ParseRule { prefix: Some(|p, _| p.literal()), infix: None, precedence: Precedence::None },
        TokenType::While => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
        TokenType::Global => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
        TokenType::Eof => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
        TokenType::Error => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
    }
}