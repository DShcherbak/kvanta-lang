use std::collections::HashMap;

use pest::iterators::{Pairs, Pair};
use crate::{ast::{keys::key_to_number, AstFunction, AstProgram, AstStatement, BaseValueType, Coords, ExpressionType, FunctionsAndGlobals, HalfParsedAstFunction, SimpleExpression, SimpleExpressionType, SimpleValue, SimpleValueType, Type, TypeName, VariableCall}, error::Error, Rule};


use super::{AstBlock, AstNode, Expression, Operator,  BaseType, BaseValue, goes_before, UnaryOperator };

macro_rules! coords {
    ($val:expr) => {
        { let st = $val.as_span().start_pos().line_col();
         let fin =  $val.as_span().end_pos().line_col();
         (st.0, st.1, fin.0, fin.1)
        }
    };
}

pub struct AstBuilder {
    pub function_signatures : HashMap<String, (Vec<Type>, Option<Type>)>
}

impl AstBuilder {

pub fn new() -> AstBuilder
{
    AstBuilder{ function_signatures: HashMap::new() }
}

pub fn build_ast_from_doc(&mut self, docs: Pairs<Rule>) -> Result<AstProgram, Error> {
    assert!(docs.len() == 1);
    let doc = docs.into_iter().next().unwrap();
    assert!(doc.as_rule() == Rule::document);

    let mut doc_iter = doc.into_inner().into_iter();
    assert!(doc_iter.len() == 2);
    let block_rule = doc_iter.next().unwrap();
    let eof_rule = doc_iter.next().unwrap();

    assert!(eof_rule.as_rule() == Rule::EOI);
    if block_rule.as_rule() == Rule::block {
        Ok(AstProgram::Block(self.build_ast_from_block(block_rule.into_inner())?))
    } else {
        Ok(AstProgram::Forest(self.build_ast_from_forest(block_rule.into_inner())?))
    }    
}

fn build_ast_from_forest(&mut self, statements: Pairs<Rule>) -> Result<FunctionsAndGlobals, Error> {
    let mut half_functions = vec![];
    let mut init_statements :HashMap<String, ((usize, usize, usize, usize), Type, Expression)> = HashMap::new();
    let mut blocks : Vec<AstFunction> = vec![];
    for pair in statements.clone() {
        match pair.as_rule() {
            Rule::function => {
                let res = self.get_function_signature(pair.into_inner())?;
                self.function_signatures.insert(res.name.clone(), (res.args.iter().map(|(_, t)| t.clone()).collect(), res.return_type.clone()));
                half_functions.push(res);
            }
            Rule::global_block => {
                let mut iter = pair.into_inner().into_iter();
                
                while let Some(init) = iter.next() {
                    if init.as_rule() == Rule::strong_init {
                        let coords = coords!(init);
                        let mut init_iter = init.into_inner().into_iter();
                        let type_name = self.build_ast_from_type(init_iter.next().unwrap())?;
                        let mut init_iter2 = init_iter.next().unwrap().into_inner().into_iter();
                        let name = self.build_ast_from_ident(init_iter2.next().unwrap())?;
                        let expr = self.build_ast_from_expression(init_iter2.next().unwrap())?;
                        init_statements.insert(name, (coords, type_name, expr));
                    } else {
                        return Err(Error::parse(format!("Expected global variable initialization, found: {:?}", init.as_rule()), coords!(init)));
                    }
                }
            },
            _ => return Err(Error::parse(format!("Expected a function at: {:?}", pair.as_rule()), coords!(pair)))
        }
    }
    for func in half_functions {
        blocks.push(self.build_ast_from_function(func)?);
    }
    Ok((blocks, init_statements))
}

fn build_ast_from_function(&self, function: HalfParsedAstFunction) -> Result<AstFunction, Error> {
    let body = self.build_ast_from_block(function.statements)?;
    Ok(AstFunction{name: function.name, args: function.args, return_type: function.return_type, block: body, header: function.coords})
}


fn get_function_signature<'a>(&self, statement: Pairs<'a, Rule>) -> Result<HalfParsedAstFunction<'a>, Error> {
    let mut iter = statement.into_iter();
    let header = iter.next().unwrap();
    let header_coords = coords!(header);
    assert!(header.as_rule() == Rule::fn_header);
    let body = iter.next().unwrap();
    assert!(body.as_rule() == Rule::block);

    let mut header_iter = header.into_inner().into_iter();
    let name = self.build_ast_from_ident(header_iter.next().unwrap())?;
    let mut args = vec![];
    let args_iter = header_iter.next().unwrap().into_inner().into_iter();
    for arg in args_iter {
        let mut arg_iter = arg.into_inner().into_iter();
        let arg_type = self.build_ast_from_type(arg_iter.next().unwrap())?;
        let arg_name = self.build_ast_from_ident(arg_iter.next().unwrap())?;
        args.push((arg_name, arg_type));
    }
    let typer = {
        if let Some(x) = header_iter.next() {
            assert!(x.as_rule() == Rule::type_name);
            let typed = self.build_ast_from_type(x)?;
            Some(typed)
        } else {
            None
        }
    };
    
    Ok(HalfParsedAstFunction { 
        name, 
        args, 
        return_type: typer, 
        statements: body.into_inner(),
        coords: header_coords
    })

}

fn build_ast_from_block(&self, statements: Pairs<Rule>) -> Result<AstBlock, Error> {
    let mut block = AstBlock{ nodes: vec![], coords: (0, 0, 0, 0) };
    let (mut l1, mut c1, mut l2, mut c2) = (0, 0, 0, 0) ;
    for pair in statements {
        match pair.as_rule() {
            Rule::statement => {
                let (a,b,c,d) = coords!(pair);
                l1 = std::cmp::min(l1, a);
                c1 = std::cmp::min(c1, b);
                l2 = std::cmp::max(l2, c);
                c2 = std::cmp::max(c2, d);
                block.nodes.push(self.build_ast_from_statement(pair.into_inner())?);
            }
            _ => return Err(Error::parse(String::from("Expected a statement!"),coords!(pair)))
        }
    }
    block.coords = (l1, c1, l2, c2);
    Ok(block)
}

fn build_ast_from_statement(&self, statement: Pairs<Rule>) -> Result<AstNode, Error> {
    let mut iter = statement.into_iter();
    let state = iter.next().unwrap();
    let coords = coords!(state);
    match state.as_rule() {
        Rule::command => self.build_ast_from_command(state.into_inner(), coords),
        Rule::init_statement => self.build_ast_from_init(state.into_inner(), coords),
        Rule::if_statement => self.build_ast_from_if(state.into_inner(), coords),
        Rule::for_statement => self.build_ast_from_for(state.into_inner(), coords),
        Rule::while_statement => self.build_ast_from_while(state.into_inner(), coords),
        Rule::return_statement => {
            let expr = self.build_ast_from_expression(state.into_inner().into_iter().next().unwrap())?;
            Ok(AstNode{statement: AstStatement::Return { expr: expr }, coords: coords})
        }
        _ => return Err(Error::parse(String::from("Expected a statement!"), coords!(state)))
    }
}

fn build_ast_from_command(&self, command: Pairs<Rule>, coords: Coords) -> Result<AstNode, Error> {
    let mut iter = command.into_iter().next().unwrap().into_inner().into_iter();
    let name = self.build_ast_from_ident(iter.next().unwrap())?;
    let args = self.build_ast_from_arglist(iter)?;
    return Ok(AstNode{statement: AstStatement::Command { 
        name: name,
        args: args
    }, coords});
}

fn build_ast_from_ident(&self, ident: Pair<Rule>) -> Result<String, Error> {
    Ok(String::from(ident.as_str().trim()))
}

fn build_ast_from_noun(&self, ident: Pair<Rule>) -> Result<VariableCall, Error> {
    if ident.as_rule() == Rule::noun {
        let mut ident = ident.into_inner().into_iter();
        if let Some(name) = ident.next() {
            if ident.clone().count() > 0 {
                let mut args = vec![];
                for arg in ident {
                    args.push(self.build_ast_for_simple_expression(arg.into_inner().into_iter().next().unwrap())?);
                }
                return Ok(VariableCall::ArrayCall(String::from(name.as_str()), args));
            }
            return Ok(VariableCall::Name(String::from(name.as_str())));
        }
        return Ok(VariableCall::Name(String::from(ident.as_str())));
    }
    Err(Error::parse(format!("Expected identifier, found: {}", ident.as_str()), coords!(ident)))
}

fn build_ast_from_arglist(&self, args: Pairs<Rule>) -> Result<Vec<Expression>, Error> {
    let mut expressions = vec![];
    for pair in args {
        expressions.push(self.build_ast_from_expression(pair)?);
    }
    Ok(expressions)
}

fn improve_expr(&self, expr : Expression) -> (Expression, bool) {
    let bin = |op, l, r, c| {
        Expression{expr_type: ExpressionType::Binary(op, l, r), coords: c}
    };
    match expr.expr_type {
        ExpressionType::Value(_) => (Expression{expr_type: expr.expr_type, coords: expr.coords}, false),
        ExpressionType::Unary(op, inner) => {
            let (new_inner, did) = self.improve_expr(*inner);
            (Expression{expr_type: ExpressionType::Unary(op, new_inner.into()), coords: expr.coords}, did)
        },
        ExpressionType::Binary(op, left, right) => {
            let mut new_left : Expression = *left;
            if let Expression{expr_type: ExpressionType::Unary(UnaryOperator::Parentheses, _), coords: _} = new_left {
                (new_left, _) = self.improve_expr(new_left);
            }
            if let Expression{expr_type: ExpressionType::Binary(r_op, r_left, r_right), coords} = *right.clone() {
                if goes_before(op, r_op) {
                    return self.improve_expr(bin(r_op, 
                            bin(op, new_left.into(), r_left, coords).into(), r_right, expr.coords))
                } else {
                    let (new_right, redo) = self.improve_expr(*right);
                    if redo {
                        return self.improve_expr(bin(op, new_left.into(), new_right.into(), expr.coords));
                    }
                    return (bin(op, new_left.into(), new_right.into(), expr.coords), false)
                }
            }
            (bin(op, new_left.into(), right, expr.coords), false)
        },
    }
}

// todo remove separation for simple expressions
fn improve_simple_expr(&self, expr : SimpleExpression) -> (SimpleExpression, bool) {
    let sim = |op, l, r, c| {
        SimpleExpression{expr: SimpleExpressionType::Binary(op, l, r), coords: c}
    };
    match expr.expr {
        SimpleExpressionType::Value(_) => (expr, false),
        SimpleExpressionType::Unary(_, _) => (expr, false),
        SimpleExpressionType::Binary(op, left, right) => {
            let mut new_left : SimpleExpression = *left;
            if let SimpleExpression{ expr: SimpleExpressionType::Unary(UnaryOperator::Parentheses, _), coords: _} = new_left {
                (new_left, _) = self.improve_simple_expr(new_left);
            }
            if let SimpleExpression{expr: SimpleExpressionType::Binary(r_op, r_left, r_right), coords} = *right.clone() {
                if goes_before(op, r_op) {
                    return self.improve_simple_expr(
                        sim(r_op, 
                            sim(op, new_left.into(), r_left, coords).into(), r_right, expr.coords))
                } else {
                    let (new_right, redo) = self.improve_simple_expr(*right);
                    if redo {
                        return self.improve_simple_expr(SimpleExpression{expr: SimpleExpressionType::Binary(op, new_left.into(), new_right.into()), coords: expr.coords});
                    }
                    return (sim(op, new_left.into(), new_right.into(), expr.coords), false)
                }
            }
            (sim(op, new_left.into(), right, expr.coords), false)
        },
    }
}

fn build_ast_for_simple_expression(&self, expression : Pair<Rule>) -> Result<SimpleExpression, Error> {
    let expr = self.build_ast_from_simple_expression_inner(expression)?;
    let (res, _) = self.improve_simple_expr(expr);
    Ok(res)
}

fn build_ast_from_simple_expression_inner(&self, expression: Pair<Rule>) -> Result<SimpleExpression, Error> {
    let coords = coords!(expression);
    match expression.as_rule() {
        Rule::monadicExpr => {
            let coords = coords!(expression);
            let mut iter = expression.into_inner().into_iter();
            let operator = iter.next().unwrap();
            let right = self.build_ast_from_simple_expression_inner(iter.next().unwrap())?;
            if operator.as_str().trim() == "-" {
                Ok(SimpleExpression{expr: SimpleExpressionType::Unary(super::UnaryOperator::UnaryMinus, right.into()), coords: coords})
            } else if operator.as_str().trim() == "!" {
                Ok(SimpleExpression{expr: SimpleExpressionType::Unary(super::UnaryOperator::NOT, right.into()), coords: coords})
            } else {
                Err(Error::parse(format!("Unknown unary operator '{}'", operator.as_str()), coords))
            }
        },
        Rule::dyadicExpr => {
            let coords = coords!(expression);
            let mut iter = expression.into_inner().into_iter();
            let left = self.build_ast_from_simple_expression_inner(iter.next().unwrap())?;
            let operator = iter.next().unwrap();
            let right = self.build_ast_from_simple_expression_inner(iter.next().unwrap())?;
            let v = match operator.as_str() {
                "+" => Ok(SimpleExpressionType::Binary(Operator::Plus, left.into(), right.into())),
                "-" => Ok(SimpleExpressionType::Binary(Operator::Minus, left.into(), right.into())),
                "*" => Ok(SimpleExpressionType::Binary(Operator::Mult, left.into(), right.into())),
                "/" => Ok(SimpleExpressionType::Binary(Operator::Div, left.into(), right.into())),
                "%" => Ok(SimpleExpressionType::Binary(Operator::Mod, left.into(), right.into())),

                ">"   => Ok(SimpleExpressionType::Binary(Operator::GT, left.into(), right.into())),
                "<"   => Ok(SimpleExpressionType::Binary(Operator::LT, left.into(), right.into())),
                ">="  => Ok(SimpleExpressionType::Binary(Operator::GQ, left.into(), right.into())),
                "<="  => Ok(SimpleExpressionType::Binary(Operator::LQ, left.into(), right.into())),
                "=="  => Ok(SimpleExpressionType::Binary(Operator::EQ, left.into(), right.into())),
                "!="  => Ok(SimpleExpressionType::Binary(Operator::NQ, left.into(), right.into())),

                "&&"  => Ok(SimpleExpressionType::Binary(Operator::AND, left.into(), right.into())),
                "||"  => Ok(SimpleExpressionType::Binary(Operator::OR, left.into(), right.into())),

                op => Err(Error::parse(format!("Unknown operator {}", op), coords))
            }?;
            Ok(SimpleExpression { expr: v, coords: coords })
        },
        Rule::expression => {
            return self.build_ast_from_simple_expression_inner(expression.into_inner().into_iter().next().unwrap())
        },
        Rule::parenth_expr => {
            let inner_expr = self.build_ast_from_simple_expression_inner(expression.into_inner().into_iter().next().unwrap().into_inner().into_iter().next().unwrap())?;
            Ok(SimpleExpression{expr: SimpleExpressionType::Unary(super::UnaryOperator::Parentheses, inner_expr.into()), coords:coords})
        },
        _ => {
            return Ok(SimpleExpression{expr: SimpleExpressionType::Value(self.build_ast_from_simple_value(expression)?), coords: coords})
        }
    }


}

fn build_ast_from_expression(&self, expression: Pair<Rule>) -> Result<Expression, Error> {
    let expr = self.build_ast_from_expression_inner(expression)?;
    let (res, _) = self.improve_expr(expr);
    Ok(res)
}

fn build_ast_from_expression_inner(&self, expression: Pair<Rule>) -> Result<Expression, Error> {
    let coords = coords!(expression);
    match expression.as_rule() {
        Rule::monadicExpr => {
            let mut iter = expression.into_inner().into_iter();
            let operator = iter.next().unwrap();
            let right = self.build_ast_from_expression_inner(iter.next().unwrap())?;
            if operator.as_str() == "-" {
                Ok(Expression{expr_type: ExpressionType::Unary(super::UnaryOperator::UnaryMinus, right.into()), coords})
            } else if operator.as_str() == "!" {
                Ok(Expression{expr_type: ExpressionType::Unary(super::UnaryOperator::NOT, right.into()), coords: coords})
            } else {
                Err(Error::parse(format!("Unknown unary operator {}", operator.as_str()), coords))
            }
        },
        Rule::dyadicExpr => {
            let coords = coords!(expression);
            let mut iter = expression.into_inner().into_iter();
            let left = self.build_ast_from_expression_inner(iter.next().unwrap())?;
            let operator = iter.next().unwrap();
            let right = self.build_ast_from_expression_inner(iter.next().unwrap())?;
            let expr = match operator.as_str() {
                "+" => Ok(ExpressionType::Binary(Operator::Plus, left.into(), right.into())),
                "-" => Ok(ExpressionType::Binary(Operator::Minus, left.into(), right.into())),
                "*" => Ok(ExpressionType::Binary(Operator::Mult, left.into(), right.into())),
                "/" => Ok(ExpressionType::Binary(Operator::Div, left.into(), right.into())),
                "%" => Ok(ExpressionType::Binary(Operator::Mod, left.into(), right.into())),

                ">"   => Ok(ExpressionType::Binary(Operator::GT, left.into(), right.into())),
                "<"   => Ok(ExpressionType::Binary(Operator::LT, left.into(), right.into())),
                ">="  => Ok(ExpressionType::Binary(Operator::GQ, left.into(), right.into())),
                "<="  => Ok(ExpressionType::Binary(Operator::LQ, left.into(), right.into())),
                "=="  => Ok(ExpressionType::Binary(Operator::EQ, left.into(), right.into())),
                "!="  => Ok(ExpressionType::Binary(Operator::NQ, left.into(), right.into())),

                "&&"  => Ok(ExpressionType::Binary(Operator::AND, left.into(), right.into())),
                "||"  => Ok(ExpressionType::Binary(Operator::OR, left.into(), right.into())),

                op => Err(Error::parse(format!("Unknown operator {}", op), coords))
            }?;
            Ok(Expression{expr_type: expr, coords: coords})
        },
        Rule::expression => {
            return self.build_ast_from_expression_inner(expression.into_inner().into_iter().next().unwrap())
        },
        Rule::parenth_expr => {
            let inner_expr = self.build_ast_from_expression_inner(expression.into_inner().into_iter().next().unwrap().into_inner().into_iter().next().unwrap())?;
            Ok(Expression{expr_type: ExpressionType::Unary(super::UnaryOperator::Parentheses, inner_expr.into()), coords: coords})
        },
        _ => {
            return Ok(Expression{expr_type: ExpressionType::Value(self.build_ast_from_value(expression)?), coords: coords});
        }
    }


}

fn build_ast_from_init(&self, command: Pairs<Rule>, coords: Coords) -> Result<AstNode, Error> {
    let mut iter = command.into_iter();
    let mut first = iter.next().unwrap();
    if let Rule::type_name = first.as_rule() {
        let type_val = self.build_ast_from_type(first)?;
        first = iter.next().unwrap();
        let mut assign = first.into_inner().into_iter();
        let name = self.build_ast_from_ident(assign.next().unwrap())?;
        let expr = self.build_ast_from_expression(assign.next().unwrap())?;
        return Ok(AstNode{statement: AstStatement::Init { typ: type_val, val: name, expr }, coords});
    } 
    let mut assign = first.into_inner().into_iter();
    let name = self.build_ast_from_noun(assign.next().unwrap())?;
    let expr = self.build_ast_from_expression(assign.next().unwrap())?;

    Ok(AstNode{statement: AstStatement::SetVal { val: name, expr },coords})
}

fn build_ast_from_if(&self, command: Pairs<Rule>, coords: Coords) -> Result<AstNode, Error> {
    let mut iter = command.into_iter();
    return Ok(AstNode{statement: AstStatement::If { 
        clause: self.build_ast_from_expression(iter.next().unwrap())?, 
        block: self.build_ast_from_block(iter.next().unwrap().into_inner().into_iter().next().unwrap().into_inner())?,
        else_block: { 
            if let Some(rule) = iter.next() {
                let block = self.build_ast_from_block(rule.into_inner().into_iter().next().unwrap().into_inner())?;
                    Some(block)
            } else { 
                None 
            }
        }
    }, coords})
}

fn build_ast_from_for(&self, command: Pairs<Rule>, coords: Coords) -> Result<AstNode, Error> {
    let mut iter = command.into_iter();
    let name = iter.next().unwrap();
    let mut range = iter.next().unwrap().into_inner().into_iter();
    Ok(AstNode{statement: AstStatement::For { 
        val:  self.build_ast_from_ident(name).unwrap(), 
        from: self.build_ast_from_value(range.next().unwrap())?, 
        to: self.build_ast_from_value(range.next().unwrap())?,
        block: self.build_ast_from_block(iter.next().unwrap().into_inner().into_iter().next().unwrap().into_inner())?
    }, coords})
}

fn build_ast_from_value(&self, val: Pair<Rule>) -> Result<BaseValue, Error> {
    let coords = coords!(val);
    let v = match val.as_rule() {
        Rule::integer => Ok(BaseValueType::Int(val.as_str().parse::<i32>().unwrap())),
        Rule::decimal => Ok(BaseValueType::Float(val.as_str().parse::<f32>().unwrap())),
        Rule::boolean => Ok(BaseValueType::Bool(val.as_str() == "true")),
        Rule::color   => {return self.build_ast_from_color(val);},
        Rule::key     => {return self.build_ast_from_key(val);},
        Rule::noun   => Ok(BaseValueType::Id(self.build_ast_from_noun(val)?)),
        Rule::array_literal => {
            let mut elements = vec![];
            for item in val.into_inner() {
                elements.push(self.build_ast_from_value(item)?);
            }
            Ok(BaseValueType::Array(elements))
        },
        Rule::function_call => {
            let coords = coords!(val);
            let mut iter = val.into_inner().into_iter();
            let name = self.build_ast_from_ident(iter.next().unwrap())?;
            let args = self.build_ast_from_arglist(iter)?;
            if let Some((_, return_type)) = self.function_signatures.get(&name) {
                if let Some(typ) = return_type {
                    Ok(BaseValueType::FunctionCall(name, args, typ.clone()))
                } else {
                    Err(Error::typeEr(format!("Function {} has no return type", name), coords))
                }
            } else {
                Err(Error::typeEr(format!("Unknown function {}", name), coords))
            }
        }
        _ => return Err(Error::parse(String::from("Expected a value!"), coords!(val)))
    }?;
    Ok(BaseValue{val: v, coords: coords})
}

fn build_ast_from_simple_value(&self, val: Pair<Rule>) -> Result<SimpleValue, Error> {
    let coords = coords!(val);
    match val.as_rule() {
        Rule::integer => Ok(SimpleValue{val:SimpleValueType::Int(val.as_str().parse::<i32>().unwrap()), coords: coords}),
        Rule::noun   => Ok(SimpleValue{val:SimpleValueType::Id(self.build_ast_from_noun(val)?), coords: coords}),
        _ => return Err(Error::parse(String::from("Expected a simple value!"), coords!(val)))
    }
}

fn build_ast_from_color(&self, val: Pair<Rule>) -> Result<BaseValue, Error> {
    let v = match val.as_str() {
        "Color::Red" => Ok(BaseValueType::Color(233,35,49)),
        "Color::Green" => Ok(BaseValueType::Color(126,183,134)),
        "Color::Blue" => Ok(BaseValueType::Color(46,115,230)),
        "Color::Yellow" => Ok(BaseValueType::Color(253,226,93)),
        "Color::Pink" => Ok(BaseValueType::Color(251,154,181)),
        "Color::Cyan" => Ok(BaseValueType::Color(59,168,231)),
        "Color::Black" => Ok(BaseValueType::Color(0, 0, 0)),
        "Color::White" => Ok(BaseValueType::Color(255, 255, 255)),
        "Color::Random" => Ok(BaseValueType::RandomColor),
        _ => Err(Error::parse(format!("Unknown color: {}", val.as_str()), coords!(val)))
    }?;
    Ok(BaseValue { val: v, coords: coords!(val) })
}

fn build_ast_from_key(&self, val: Pair<Rule>) -> Result<BaseValue, Error> {
    let str = val.as_str();
    if str.starts_with("Key::") {
        if let Some(num) = key_to_number(str.split_at(5).1) {
            return Ok(BaseValue{val: BaseValueType::Int(num), coords: coords!(val)});
        }
    }
    return Err(Error::parse(format!("Unknown key: {}", val.as_str()), coords!(val)));
}

fn build_ast_from_while(&self, command: Pairs<Rule>, coords: Coords) -> Result<AstNode, Error> {
    let mut iter = command.into_iter();
    Ok(AstNode{statement: AstStatement::While { 
        clause: self.build_ast_from_expression(iter.next().unwrap())?, 
        block: self.build_ast_from_block(iter.next().unwrap().into_inner().into_iter().next().unwrap().into_inner())?,
    }, coords})
}

fn build_ast_from_array_type(&self, type_val: Pairs<Rule>) -> Result<TypeName, Error> {
    
    let mut iter = type_val.into_iter().next().unwrap().into_inner().into_iter();
    let inner_type = self.build_ast_from_type(iter.next().unwrap())?;
    let val = iter.next().unwrap();
    let coords = coords!(val);
    if let BaseValue{val: BaseValueType::Int(array_size), coords: c} = self.build_ast_from_value(val)? {
        if array_size <= 0 {
            return Err(Error::parse(String::from("Array size must be greater than 0"), coords));
        }
        return Ok(TypeName::Array(Box::new(Some(inner_type)), array_size as usize));
    } else {
        return Err(Error::parse(String::from("Expected an integer for array size"), coords));
    }
}

fn build_ast_from_inner_type(&self, type_val: Pairs<Rule>) -> Result<TypeName, Error> {
    use BaseType::*;
    if let Some(i) = type_val.clone().next() {
        if i.as_rule() == Rule::array_type {
            return self.build_ast_from_array_type(type_val);
        }
    }
    match type_val.as_str() {
        "int" => Ok(TypeName::Primitive(Int)),
        "bool" => Ok(TypeName::Primitive(Bool)),
        "color" => Ok(TypeName::Primitive(Color)),
        "float" => Ok(TypeName::Primitive(Float)),
        t => Err(Error::parse(format!("Unknown type: {}", t), coords!(type_val.clone().next().unwrap())))
    }
}

fn build_ast_from_type(&self, type_val: Pair<Rule>) -> Result<Type, Error> {
    let mut whole = type_val.clone().into_inner().into_iter();
    if let Some(first) = whole.next(){
        if first.as_rule() == Rule::const_key {
            return Ok(Type{type_name: self.build_ast_from_inner_type(whole)?, is_const: true});
        }
    }
    Ok(Type{type_name: self.build_ast_from_inner_type(type_val.into_inner().into_iter())?, is_const: false})

}

}
