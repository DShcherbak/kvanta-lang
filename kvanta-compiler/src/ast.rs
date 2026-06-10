#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProgramAst {
    Forest,
    Script(Vec<StatementAst>),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]

pub struct ExpressionAst {

}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TypeAst {
    Int,
    Float,
    Color,
    Bool,
    Array(Box<TypeAst>, ExpressionAst),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum StatementAst {
    Expression(ExpressionAst),
    Print,
    Var(TypeAst, String, ExpressionAst),
    Block,
    If,
    While,
    For,
    Function,
    Return
}