#[derive(Debug, Clone, PartialEq)]
pub enum ProgramAst {
    Forest,
    Script(Vec<StatementAst>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Bang,
    Minus
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    Plus,
    Minus,
    Mult,
    Divide,
    Assign,
    Call,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AstValue {
    Float(f32),
    Bool(bool),
    String(String),
    Variable(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionAst {
    Value(AstValue),
    Unary(UnaryOperator, Box<ExpressionAst>),
    Binary(BinaryOperator, Box<ExpressionAst>, Box<ExpressionAst>),
    FunctionCall(Box<ExpressionAst>, Vec<Box<ExpressionAst>>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeAst {
    Int,
    Float,
    Color,
    Bool,
    Array(Box<TypeAst>, AstValue),
}

#[derive(Debug, Clone, PartialEq)]
pub enum StatementAst {
    Expression(ExpressionAst),
    Print(ExpressionAst),
    Var(TypeAst, String, ExpressionAst),
    Block,
    If,
    While,
    For,
    Function,
    Return
}
