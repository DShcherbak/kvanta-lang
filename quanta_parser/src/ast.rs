use std::{collections::HashMap, fmt};

use pest::iterators::Pairs;

use crate::{error::Error, Rule};

pub mod builder;
pub mod keys;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BaseType {
    Int,
    Bool,
    Color,
    Float
}

impl BaseType {
    pub fn to_string(&self) -> String {
        match self {
            BaseType::Int => "int".to_string(),
            BaseType::Bool => "bool".to_string(),
            BaseType::Color => "color".to_string(),
            BaseType::Float => "float".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum VariableCall {
    Name(String),
    ArrayCall(String, Vec<SimpleExpression>)
}

impl fmt::Display for VariableCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VariableCall::Name(name) => write!(f, "{}", name),
            VariableCall::ArrayCall(name, indices) => {
                write!(f, "{}[", name)?;
                for (i, index) in indices.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", index)?;
                }
                write!(f, "]")
            }
        }
    }
}

impl fmt::Display for SimpleExpression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.expr {
            SimpleExpressionType::Value(value) => write!(f, "{}", value),
            SimpleExpressionType::Unary(op, expr) => write!(f, "{:?}({})", op, expr),
            SimpleExpressionType::Binary(op, left, right) => write!(f, "({} {:?} {})", left, op, right),
        }
    }
}

impl fmt::Display for SimpleValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.val {
            SimpleValueType::Id(var) => write!(f, "{}", var),
            SimpleValueType::Int(value) => write!(f, "{}", value),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SimpleValueType {
    Id(VariableCall),
    Int(i32)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SimpleValue {
    val: SimpleValueType,
    coords: (usize, usize, usize, usize)
}

#[derive(Debug, Clone, PartialEq)]
pub enum BaseValueType {
    Id(VariableCall),
    Int(i32),
    Bool(bool),
    Color(u8, u8, u8),
    RandomColor,
    Float(f32),
    Array(Vec<BaseValue>), // Array of BaseValues
    FunctionCall(String, Vec<Expression>, Type), // Function call with name and arguments
}

#[derive(Debug, Clone, PartialEq)]
pub struct BaseValue {
    pub val: BaseValueType,
    pub coords: Coords
}


#[derive(Debug, Clone, PartialEq)]
pub struct Type {
    pub type_name: TypeName,
    pub is_const: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeName {
    Primitive(BaseType),
    Array(Box<Option<Type>>, usize), // Array of a certain type with a fixed size
}

impl Type {
    pub fn to_string(&self) -> String {
        format!("{} {}", if self.is_const { "const" } else { "" }, 
            self.type_name.to_string())
    }

    pub fn typ(t: BaseType) -> Type {
        Type{type_name: TypeName::Primitive(t), is_const: false}
    }

    pub fn can_assign(&self, t: &Type) -> bool {
        if let TypeName::Array(t1, size1) = &self.type_name {
            if let TypeName::Array(t2, size2) = &t.type_name {
                if size1 != size2 { return false; }
                if t2.is_none() { return true; }
                if t1.is_none() { return false; }
                return size1 == size2 && t1.clone().unwrap().can_assign(&t2.clone().unwrap());
            } else {
                return false;
            }
        }  
        return true;
    }
}

impl TypeName {
    pub fn to_string(&self) -> String {
        match &self {
            TypeName::Primitive(type_name) => type_name.to_string(),
            TypeName::Array(inner_type, size) => {
                let inner = match inner_type.as_ref() {
                    Some(t) => t.to_string(),
                    None => "()".to_string(),
                };
                format!("array<{},{}>", inner, size)
            },
        }
    }
}

impl BaseValue {
    pub fn get_type<F>(&self, get_var_type: &F) -> Result<TypeName, Error> 
        where F: Fn(&VariableCall) -> Option<Type>  
    {
        use TypeName::*;
        match &self.val {
            BaseValueType::Id(name) => {
                if let Some(type_) = get_var_type(&name) {
                    Ok(type_.type_name)
                } else {
                    Err(Error::typeEr(format!("Variable type unknown: {}", name.to_string()), self.coords))
                }
            }, 
            BaseValueType::Int(_) => Ok(Primitive(BaseType::Int)),
            BaseValueType::Bool(_) => Ok(Primitive(BaseType::Bool)),
            BaseValueType::Color(_, _, _) => Ok(Primitive(BaseType::Color)),
            BaseValueType::RandomColor => Ok(Primitive(BaseType::Color)),
            BaseValueType::Float(_) => Ok(Primitive(BaseType::Float)),
            BaseValueType::Array(elems) => {
                if elems.is_empty() {
                    return Ok(Array(Box::new(None), 0));
                }
                let type_ = elems.first().unwrap().clone().get_type(get_var_type)?;
                for elem in elems {
                    if elem.get_type(get_var_type)? != type_ {
                        return Err(Error::typeEr(format!("Array type unknown"), self.coords));
                    }
                }
                Ok(Array(Box::new(Some(Type{type_name:type_, is_const:false})), elems.len()))
            }
            BaseValueType::FunctionCall(_, _, t) => {
                Ok(t.type_name.clone())
            }
        }
    }
}

impl Eq for BaseValue {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnaryOperator {
    UnaryMinus,
    NOT,
    Parentheses
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Operator {
    EQ,
    NQ,
    GT,
    LT,
    GQ,
    LQ,

    AND,
    OR,

    Plus,
    Minus,
    Mult,
    Div, 
    Mod
}

pub fn is_arith(op : Operator) -> bool {
    op == Operator::Plus  ||
    op == Operator::Minus || 
    op == Operator::Mult  ||
    op == Operator::Div   ||
    op == Operator::Mod
}

fn prec(op : Operator) -> i32 {
    if op == Operator::OR {
        return 10;
    }else if op == Operator::AND {
        return 9;
    } else if op == Operator::Mult || op == Operator::Div || op == Operator::Mod {
        return 4;
    } else if op == Operator::Plus || op == Operator::Minus {
        return 6;
    } else {
        return 8;
    }
}
 
pub fn goes_before(op1 : Operator,  op2: Operator) -> bool {
    return prec(op1) < prec(op2)
}


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SimpleExpressionType {
    Value(SimpleValue),
    Unary(UnaryOperator, Box<SimpleExpression>),
    Binary(Operator, Box<SimpleExpression>, Box<SimpleExpression>)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SimpleExpression {
    expr: SimpleExpressionType,
    coords: (usize, usize, usize, usize)
}

impl SimpleExpression {
    pub fn to_expr(self) -> Expression {
        match self.expr {
            SimpleExpressionType::Value(value) => {
                Expression{expr_type: ExpressionType::Value(match value.val {
                    SimpleValueType::Id(var) => BaseValue{val: BaseValueType::Id(var), coords: value.coords},
                    SimpleValueType::Int(i) => BaseValue{val: BaseValueType::Int(i), coords: value.coords},
                }), coords: self.coords}
            }
            SimpleExpressionType::Unary(op, expr) => 
                Expression{ expr_type: ExpressionType::Unary(op, Box::new(expr.to_expr())), coords: self.coords},
            SimpleExpressionType::Binary(op, left, right) => 
                Expression{ expr_type: ExpressionType::Binary(op, Box::new(left.to_expr()), Box::new(right.to_expr()))
                    , coords: self.coords}
        }
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpressionType {
    Value(BaseValue),
    Unary(UnaryOperator, Box<Expression>),
    Binary(Operator, Box<Expression>, Box<Expression>)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expression {
    pub expr_type: ExpressionType,
    pub coords: (usize, usize, usize, usize)
}


#[derive(Debug, Clone)]
pub enum AstStatement {
    Command { name: String, args: Vec<Expression> },
    Init    { typ: Type, val : String, expr: Expression },
    SetVal { val: VariableCall, expr: Expression },
    For     { val: String, from: BaseValue, to: BaseValue, block: AstBlock },
    While   { clause: Expression, block: AstBlock},
    If      { clause: Expression, block: AstBlock, else_block: Option<AstBlock>},
    Return  { expr: Expression },
}

#[derive(Debug, Clone)]
pub struct AstNode {
    pub statement: AstStatement,
    pub coords: (usize, usize, usize, usize)
}

impl Expression {
    pub fn get_type<F>(&self, get_var_type: &F) -> Result<TypeName, Error>
        where F: Fn(&VariableCall) -> Option<Type>
    {
        let type_mismatch = Err(Error::typeEr(format!("Type mismatch error"), self.coords));
        match &self.expr_type {
            ExpressionType::Value(base_value) => base_value.get_type(get_var_type),
            ExpressionType::Unary(_, expr) => expr.get_type(get_var_type),
            ExpressionType::Binary(_, e1, e2) => {
                let t1 = e1.get_type(get_var_type)?;
                let t2 = e2.get_type(get_var_type)?;
                if let TypeName::Array(ar_typ_1, ar_sz_1) = &t1 {
                    if let TypeName::Array(ar_typ_2, ar_sz_2) = &t2 {
                        if ar_sz_1 != ar_sz_2 {
                            return type_mismatch;
                        }
                        if ar_typ_1.is_none() && ar_typ_2.is_none() {
                            return Ok(t1);
                        }
                        if ar_typ_1.is_none() || ar_typ_2.is_none() {
                            return type_mismatch;
                        }
                        let ar_typ_1 = ar_typ_1.clone().unwrap();
                        let ar_typ_2 = ar_typ_2.clone().unwrap();
                        if ar_typ_1.can_assign(&ar_typ_2) {
                            return Ok(t1);
                        } else {
                            return type_mismatch;
                        }
                    }
                    return type_mismatch;
                }
                return if t1 == t2 { Ok(t1) } else { type_mismatch }
            },
        }
    }
}

pub type Coords = (usize, usize, usize, usize);

#[derive(Debug, Clone)]
pub struct AstBlock {
    pub nodes : Vec<AstNode>,
    pub coords: Coords,
}


#[derive(Debug, Clone)]
pub struct HalfParsedAstFunction<'a> {
    pub name: String,
    pub args: Vec<(String, Type)>,
    pub return_type: Option<Type>,
    pub statements: Pairs<'a, Rule>,
    pub coords: Coords
}

#[derive(Debug, Clone)]
pub struct AstFunction {
    pub name: String,
    pub args: Vec<(String, Type)>,
    pub return_type: Option<Type>,
    pub block: AstBlock,
    pub header: Coords,
}

pub type FunctionsAndGlobals = (Vec<AstFunction>, HashMap<String, (Coords, Type, Expression)>);

#[derive(Debug, Clone)]
pub enum AstProgram {
    Block(AstBlock),
    Forest(FunctionsAndGlobals)
}