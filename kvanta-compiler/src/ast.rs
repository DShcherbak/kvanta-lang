use crate::value::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum ProgramAst {
    Forest,
    Script(Vec<StatementAst>),
}

#[derive(Debug, Clone, PartialEq)]

pub struct ExpressionAst {

}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeAst {
    Int,
    Float,
    Color,
    Bool,
    Array(Box<TypeAst>, Value),
}

#[derive(Debug, Clone, PartialEq)]
pub enum StatementAst {
    Expression(ExpressionAst),
    Print,
    Var(TypeAst, String, Value),
    Block,
    If,
    While,
    For,
    Function,
    Return
}