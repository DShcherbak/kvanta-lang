use std::{collections::HashMap, sync::{Arc, Mutex}};

use quanta_parser::{ast::{AstBlock, AstNode, AstProgram, BaseValue, Expression, Operator, Type, UnaryOperator, VariableCall}, error::Error};
use quanta_parser::ast::BaseType;
use crate::utils::canvas::Canvas;
use js_sys::Math;


#[derive(Debug, Clone)]
pub struct Scope {
    pub variables: HashMap<String, BaseValue>,
    pub outer_scope: Option<Arc<Mutex<Scope>>>,
}

impl Scope {

    pub fn contains_key(&self, name: &str) -> bool {
        if self.variables.contains_key(name) {
            return true;
        }
        if let Some(outer) = self.outer_scope.as_ref() {
            return outer.lock().unwrap().contains_key(name);
        }
        false
    }

    pub fn set(&mut self, name: String, val: BaseValue) -> bool {
        if self.variables.contains_key(&name) {
            self.variables.insert(name, val);// = val;
            return true;
        }
        if let Some(outer) = &mut self.outer_scope {
            return outer.lock().unwrap().set(name, val);
        }
        return false;
    }

    pub fn get(&self, name: &str) -> Option<BaseValue> {
        if let Some(var) = self.variables.get(name) {
            return Some(var.clone());
        }
        if let Some(outer) = &self.outer_scope {
            return outer.lock().unwrap().get(name);
        }
        None
    }

    fn clear(&mut self) {
        self.variables = HashMap::new();
        self.outer_scope = None;
    }
}

#[derive(Debug, Clone)]
pub struct Execution {
    pub lines: AstProgram, 
    pub scope : Arc<Mutex<Scope>>,
    pub global_vars : Arc<Mutex<HashMap<String, BaseValue>>>,
    pub functions : HashMap<String, (Vec<(String, Type)>, Option<Type>, AstBlock)>,
    pub canvas    : Canvas,
    pub figure_color : Arc<Mutex<String>>,
    pub line_color : Arc<Mutex<String>>,
    pub line_width : Arc<Mutex<i32>>,
}

fn color_to_str(r: &u8, g : &u8, b: &u8) -> String {
    let s = format!("#{:02x}{:02x}{:02x}", r, g, b).to_lowercase();
    s
}

macro_rules! expect_arg {
    // Варіант із полями: BaseValue::Variant(pats...)
    ($fname:expr, $vals:expr, $idx:expr, $Variant:ident ( $($pat:pat),* ) => $build:expr) => {{
        let __arg_index = $idx; // збережемо, щоб не обчислювати двічі
        match &$vals[__arg_index] {
            BaseValue::$Variant($($pat),*) => { $build }
            other => {
                return Err(Error::RuntimeError {
                    message: format!(
                        "{}: arg #{}: Expected argument type {} but got {}",
                        $fname,
                        __arg_index,            
                        stringify!($Variant),
                        other.get_type(&|_| Some(Type::typ(BaseType::Int)))?.to_string()
                    ).into(),
                });
            }
        }
    }};
}

fn update_array(name: String, array: &mut BaseValue, mut integer_indices: Vec<i32>, val: BaseValue) -> Result<(), Error> {
        if let BaseValue::Array(elems) = array {
            let index = integer_indices.remove(0);
            if index < 0 || index as usize >= elems.len() {
                return Err(Error::RuntimeError { message: format!("Index out of bounds for array {}: {}", name, index).into() });
            }
            if integer_indices.len() == 0 {
                elems[index as usize] = val;
                return Ok(());
            }
            return update_array(format!("{}[{}]", name, index), elems.get_mut(index as usize).unwrap(), integer_indices, val);
        } else {
            return Err(Error::RuntimeError { message: format!("Variable {} is not an array", name).into() });
        }
    }

impl Execution {

    pub fn create_subscope(&self) -> Execution {
        Execution {
            lines: self.lines.clone(),
            scope: Arc::new(Mutex::new(Scope { variables: HashMap::new(), outer_scope: Some(Arc::clone(&self.scope)) })),
            canvas: self.canvas.clone(),
            global_vars: self.global_vars.clone(),
            functions: self.functions.clone(),
            figure_color: Arc::clone(&self.figure_color),
            line_color: self.line_color.clone(),
            line_width: self.line_width.clone(),
        }
    }

    fn create_subfunction(&self) -> Execution {
        let e = self.create_subscope();
        e.scope.lock().unwrap().clear();
        e
    }

    pub fn contains_key(&self, name: &str) -> bool {
        if self.scope.lock().unwrap().contains_key(name) {
            return true;
        }
        self.global_vars.lock().unwrap().contains_key(name)
    }

    fn set(&mut self, name: String, val: BaseValue) -> bool {
        if self.scope.lock().unwrap().set(name.clone(), val.clone()) {
            return true;
        }
        let mut globs = self.global_vars.lock().unwrap();
        if globs.contains_key(&name) {
            globs.insert(name, val);// = val;
            return true;
        }
        return false;
    }

    pub fn get(&mut self, name: &str) -> Option<BaseValue> {
        if let Some(var) = self.scope.lock().unwrap().get(name) {
            return Some(var.clone());
        }
        self.global_vars.lock().unwrap().get(name).map(|x| x.clone())
    }

    fn get_variable(&mut self, var: &VariableCall) -> Result<BaseValue, Error> {
        match var {
            VariableCall::Name(name) => self.get(name).ok_or(Error::RuntimeError { message: format!("Unknown variable: {}", name).into() }),
            VariableCall::ArrayCall(name, indices) => {
                if !self.contains_key(name) {
                    return Err(Error::RuntimeError { message: format!("Unknown array 1: {}, variables: {:?}", name, self.scope).into() });
                }
                if indices.is_empty() {
                    return Err(Error::RuntimeError { message: "Empty index".into() });
                }
                let mut integer_indices: Vec<i32> = vec![];
                for index in indices {
                    match self.calculate_expression(index.clone().to_expr()) {
                        Ok(BaseValue::Int(i)) => {
                            if i < 0 {
                                return Err(Error::RuntimeError { message: format!("Negative index for array {}: {}", name, i).into() });
                            }
                            integer_indices.push(i);
                        },
                        _ => return Err(Error::RuntimeError { message: "Array indices must be integers".into() }),
                    }
                }
                let maybe_array = self.get(name);
                if maybe_array.is_none() {
                    return Err(Error::RuntimeError { message: format!("Unknown array: {} ", name).into() });
                }
                let mut array = maybe_array.unwrap();
                while integer_indices.len() > 0 {
                    if let BaseValue::Array(elems) = array {
                        let index = integer_indices.remove(0);
                        if index < 0 || index as usize >= elems.len() {
                            return Err(Error::RuntimeError { message: format!("Index out of bounds for array {}: {}", name, index).into() });
                        }
                        array = elems.get(index as usize).unwrap().clone();
                    } else {
                        return Err(Error::RuntimeError { message: format!("Variable {} is not an array", name).into() });
                    }
                }
                Ok(array)
            }
        }
    }

    

    fn set_variable(&mut self, var: &VariableCall, val: BaseValue) -> Result<(), Error> {
        match var {
            VariableCall::Name(name) => if self.set(name.clone(), val) { return Ok(()); } else { return Err(Error::RuntimeError { message: format!("Unknown variable: {}", name).into() });},
            VariableCall::ArrayCall(name, indices) => {
                if !self.contains_key(name) {
                    return Err(Error::RuntimeError { message: format!("Unknown array 1: {}, variables: {:?}", name, self.scope).into() });
                }
                if indices.is_empty() {
                    return Err(Error::RuntimeError { message: "Empty index".into() });
                }
                let mut integer_indices: Vec<i32> = vec![];
                for index in indices {
                    match self.calculate_expression(index.clone().to_expr()) {
                        Ok(BaseValue::Int(i)) => {
                            if i < 0 {
                                return Err(Error::RuntimeError { message: format!("Negative index for array {}: {}", name, i).into() });
                            }
                            integer_indices.push(i);
                        },
                        _ => return Err(Error::RuntimeError { message: "Array indices must be integers".into() }),
                    }
                }
                let maybe_array = self.get(name);
                if maybe_array.is_none() {
                    return Err(Error::RuntimeError { message: format!("Unknown array: {} ", name).into() });
                }
                let mut array = maybe_array.unwrap();
                update_array(name.clone(), &mut array, integer_indices, val)?;
                if self.set(name.clone(), array) { 
                    Ok(()) 
                } else { 
                    Err(Error::RuntimeError { message: format!("Unknown variable: {}", name).into() })
                }
            }
        }
    }

    fn execute_function(&mut self, function_name: &str, args: Vec<Expression>) -> Result<Option<BaseValue>, Error>{
        let mut vals : Vec<BaseValue> = vec![];
        for arg in args {
            let val = self.calculate_expression(arg)?;
            vals.push(val);
        }
        match function_name {
            "circle" => {
                let x1 = expect_arg!("circle", vals, 0, Int(v) => *v);
                let y1 = expect_arg!("circle", vals, 1, Int(v) => *v);
                let r = expect_arg!("circle", vals, 2, Int(v) => *v);

                self.canvas.add_command(format!("circle {} {} {} fill={} stroke={} width={}", x1, y1, r, self.figure_color.lock().unwrap(), self.line_color.lock().unwrap(), self.line_width.lock().unwrap()));
                Ok(None)
            },
            "line" => {
                let x1 = expect_arg!("line", vals, 0, Int(v) => *v);
                let y1 = expect_arg!("line", vals, 1, Int(v) => *v);
                let x2 = expect_arg!("line", vals, 2, Int(v) => *v);
                let y2 = expect_arg!("line", vals, 3, Int(v) => *v);

                self.canvas.add_command(format!("line {} {} {} {} stroke={} width={}", x1, y1, x2, y2, self.line_color.lock().unwrap(), self.line_width.lock().unwrap()));
                Ok(None)
            },
            "rectangle" => {
                let x1 = expect_arg!("rectangle", vals, 0, Int(v) => *v);
                let y1 = expect_arg!("rectangle", vals, 1, Int(v) => *v);
                let x2 = expect_arg!("rectangle", vals, 2, Int(v) => *v);
                let y2 = expect_arg!("rectangle", vals, 3, Int(v) => *v);
                
                self.canvas.add_command(format!("rectangle {} {} {} {} fill={} stroke={} width={}", x1, y1, x2, y2, self.figure_color.lock().unwrap(), self.line_color.lock().unwrap(), self.line_width.lock().unwrap()));
                Ok(None)
            },
            "polygon" => {
                let mut nums = String::new();
                for val in &vals {
                    if let BaseValue::Int(num) = val {
                        nums.push_str(&format!("{} ", num));
                    } else {
                        return Err(Error::RuntimeError { message: "Incorrect arguments for polygon function!".into() });
                    }
                }
                self.canvas.add_command(format!("polygon {} fill={} stroke={} width={}", nums.trim(), self.figure_color.lock().unwrap(), self.line_color.lock().unwrap(), self.line_width.lock().unwrap()));
                Ok(None)
            },
            "arc" => {
                let x = expect_arg!("arc", vals, 0, Int(v) => *v);
                let y = expect_arg!("arc", vals, 1, Int(v) => *v);
                let r = expect_arg!("arc", vals, 2, Int(v) => *v);
                let start = expect_arg!("arc", vals, 3, Int(v) => *v);
                let end = expect_arg!("arc", vals, 4, Int(v) => *v);

                self.canvas.add_command(format!("arc {} {} {} {} {} fill={} stroke={} width={}", x, y, r, start, end, self.figure_color.lock().unwrap(), self.line_color.lock().unwrap(), self.line_width.lock().unwrap()));
                Ok(None)
            },
            "setLineColor" => {
                if let BaseValue::Color(r,g,b) = &vals[0] {
                    let mut inner  = self.line_color.lock().unwrap();
                    *inner = color_to_str(r, g, b);
                    Ok(None)
                }
                else if let BaseValue::RandomColor = &vals[0] {
                    let r = (255.0 * Math::random()) as u8;
                    let g = (255.0 * Math::random()) as u8;
                    let b = (255.0 * Math::random()) as u8;
                    let mut inner  = self.line_color.lock().unwrap();
                    *inner = color_to_str(&r, &g, &b);
                    Ok(None)
                }
                else {
                    Err(Error::RuntimeError { message: format!("Incorrect arguments for setLineColor function: expected a color, got {:?}!", &vals[0]).into() })
                }
            },
            "setFigureColor" => {
                if let BaseValue::Color(r,g,b) = &vals[0] {
                    let mut inner  = self.figure_color.lock().unwrap();
                    *inner = color_to_str(r, g, b);
                    Ok(None)
                }
                else if let BaseValue::RandomColor = &vals[0] {
                    let r = (255.0 * Math::random()) as u8;
                    let g = (255.0 * Math::random()) as u8;
                    let b = (255.0 * Math::random()) as u8;
                    let mut inner  = self.figure_color.lock().unwrap();
                    *inner =  color_to_str(&r, &g, &b);
                    Ok(None)
                }
                else {
                    Err(Error::RuntimeError { message: format!("Incorrect arguments for setFigureColor function: expected a color, got {:?}!", &vals[0]).into() })
                }
            },
            "setLineWidth" => {
                let width = expect_arg!("setLineWidth", vals, 0, Int(width) => *width);
                if width >= 0 {
                    let mut inner  = self.line_width.lock().unwrap();
                    *inner = width;
                    Ok(None)
                } else {
                    Err(Error::RuntimeError { message: "Line width can't be negative!".into() })
                }
            },
            "sleep" => {
                let sleep_time = expect_arg!("sleep", vals, 0, Int(time) => *time);
                if sleep_time >= 0 {
                    //thread::sleep(Duration::from_millis(1000));
                    self.canvas.add_command(format!("sleep {}", sleep_time));

                    Ok(None)
                } else {
                    Err(Error::RuntimeError { message: "Sleep time can't be negative!".into() })
                }
            },
            "animate" => {
                self.canvas.add_command(format!("animate"));
                Ok(None)
            },
            "frame" => {
                self.canvas.add_command(format!("frame"));
                Ok(None)
            },
            "clear" => {
                self.canvas.add_command(format!("clear"));
                Ok(None)
            },
            "rgb" => {
                let r = expect_arg!("rgb", vals, 0, Int(v) => *v);
                let g = expect_arg!("rgb", vals, 1, Int(v) => *v);
                let b = expect_arg!("rgb", vals, 2, Int(v) => *v);
                if r < 0 || r > 255 || g < 0 || g > 255 || b < 0 || b > 255 {
                    return Err(Error::runtime(String::from("RGB values must be between 0 and 255"), coords));
                }
                Ok(Some(BaseValue{val: BaseValueType::Color(r as u8, g as u8, b as u8), coords}))
            }
            name => {
                if self.functions.contains_key(name) {
                    let (params, _, body) = self.functions.get(name).unwrap();
                    if params.len() != vals.len() {
                        return Err(Error::RuntimeError { message: format!("Function {} expects {} arguments, but got {}", name, params.len(), vals.len()).into() });
                    }
                    let new_exec = self.create_subfunction();
                    for (i, param) in params.iter().enumerate() {
                        new_exec.scope.lock().unwrap().variables.insert(param.0.clone(), vals[i].clone());
                    }
                    let ret_val_wrap = new_exec.execute_commands(body.nodes.clone())?;

                    if let Some(return_value) = ret_val_wrap {
                        return Ok(Some(return_value));
                    }
                    return Ok(None);
                }
                Err(Error::RuntimeError { message: format!("Unknown function: {}", function_name).into() })
            }
        }
    }

    fn execute_init(&mut self, var: String, expr: Expression) -> Result<(), Error>{
        let value = self.calculate_expression(expr)?;

        if let Some(_) = self.get(&var) {
            return Err(Error::RuntimeError { message: format!("Variable {} is already defined!", &var).into() });
        }
        self.scope.lock().unwrap().variables.insert(var, value);
        Ok(())
    }

    fn execute_set(&mut self, var: &VariableCall, expr: Expression) -> Result<(), Error> {
        let value = self.calculate_expression(expr)?;
        if self.get_variable(var).is_ok() {
            self.set_variable(var, value)?;
            return Ok(())
        }
        Err(Error::RuntimeError { message: "Couldn't set new value".into() })
    }

    pub fn execute(&mut self) -> Result<(), Error> {
        match self.lines {
            AstProgram::Block(ref block) => {
                self.clone().execute_commands(block.nodes.clone())?;
                self.canvas.add_command("end".into());
                Ok(())
            },
            AstProgram::Forest(ref funcs) => {
                for func in &funcs.0 {
                    if func.name == "main" {
                        let new_exec = self.create_subscope();
                        new_exec.execute_commands(func.block.nodes.clone())?;
                        self.canvas.add_command("end".into());
                        return Ok(());
                    }
                }
                Err(Error::RuntimeError { message: "No main function found".into() })
            },
        }
        
    }

    // pub fn execute_key(&mut self, key: i32) -> Result<(), Error> {
    //     match self.lines {
    //         AstProgram::Block(_) => { Ok(())},
    //         AstProgram::Forest(ref funcs) => {
    //             for func in &funcs.0 {
    //                 if func.name == "keyboard" {
    //                     let mut new_exec = self.create_subscope();
    //                     new_exec.execute_init(func.args.get(0).unwrap().0.clone(), Expression::Value(BaseValue::Int(key)))?;
    //                     new_exec.execute_commands(func.block.nodes.clone())?;
    //                 }
    //             }
    //             Ok(())
    //         },
    //     }
    // }

    // pub fn execute_mouse(&mut self, x: i32, y:i32) -> Result<(), Error> {
    //     match self.lines {
    //         AstProgram::Block(_) => { Ok(())},
    //         AstProgram::Forest(ref funcs) => {
    //             for func in &funcs.0 {
    //                 if func.name == "mouse" {
    //                     let mut new_exec = self.create_subscope();
    //                     new_exec.execute_init(func.args.get(0).unwrap().0.clone(), Expression::Value(BaseValue::Int(x)))?;
    //                     new_exec.execute_init(func.args.get(1).unwrap().0.clone(), Expression::Value(BaseValue::Int(y)))?;
    //                     new_exec.execute_commands(func.block.nodes.clone())?;
    //                 }
    //             }
    //             Ok(())
    //         },
    //     }
    // }

    pub fn execute_commands( mut self, nodes : Vec<AstNode>) -> Result<Option<BaseValue>, Error> {
            for line in nodes {
                match line {
                    AstNode::Command { name, args } => {
                        self.execute_function(&name, args)?;
                    },
                    AstNode::Init { typ : _, val, expr } => {
                        self.execute_init(val, expr)?;
                    }
                    AstNode::SetVal { val, expr } => {
                        self.execute_set(&val, expr)?;
                    }
                    
                    AstNode::If { clause, block, else_block } => {
                        if let BaseValue::Bool(val) = self.calculate_expression(clause)? {
                            let new_exec = self.create_subscope();
                            if val {
                                if let Some(return_value) = new_exec.execute_commands(block.nodes)? {
                                    return Ok(Some(return_value));
                                }
                            } else if let Some(else_block) = else_block {
                                if let Some(return_value) = new_exec.execute_commands(else_block.nodes)? {
                                    return Ok(Some(return_value));
                                }
                            }
                        } else {
                            return Err(Error::RuntimeError { message: "If clause must be a boolean expression".into() });
                        }
                    },
                    AstNode::While { clause, block } => {
                        loop {
                            let val = self.calculate_expression(clause.clone());
                            match val {
                                Ok(BaseValue::Bool(while_clause)) => {
                                    if while_clause {
                                        let new_exec = self.create_subscope();
                                        let result = new_exec.execute_commands(block.nodes.clone())?;
                                        if let Some(return_value) = result {
                                            return Ok(Some(return_value));
                                        }
                                    } else {
                                        break;
                                    }
                                }
                                Err(err) => {
                                    return Err(err);
                                }
                                Ok(v) => return Err(Error::RuntimeError { message: format!("Expected a boolean value, but got {:?}", v).into() })
                            }
                        }
                    },
                    AstNode::For { val, from, to, block } => {
                        if let BaseValue::Int(f_) = from {
                            if let BaseValue::Int(t_) = to {
                                let (f,t) = {
                                    if f_ <= t_ {
                                        (f_, t_)
                                    } else {
                                        (t_, f_)
                                    }
                                };
                                for cycle in f..=t {
                                    if let Some(return_value) = self.execute_for(val.clone(), cycle, block.clone())?{
                                        return Ok(Some(return_value));
                                    }
                                }                    
                            }
                        }
                    },
                    AstNode::Return { expr } => {
                        let val = self.calculate_expression(expr)?;
                        return Ok(Some(val.clone()))
                    },
                }
            }
            Ok(None)
    }

    fn execute_for(&mut self, val: String, cycle : i32, block : AstBlock) -> Result<Option<BaseValue>, Error> {
        let mut new_exec = self.create_subscope();
        new_exec.execute_init(val, Expression::Value(BaseValue::Int(cycle)))?;
        let result: Result<Option<BaseValue>, Error> = new_exec.execute_commands(block.nodes.clone());
        result
    }



    pub fn calculate_expression(&mut self, expr: Expression,) -> Result<BaseValue, Error> {

            match expr {
                Expression::Value(base_value) => {
                    match base_value {
                        BaseValue::Id(var) => {
                            self.get_variable(&var)
                        },
                        BaseValue::FunctionCall(name, exprs, _ ) => {
                            let mut vals = vec![];
                            for expr in exprs {
                                let val = self.calculate_expression(expr)?;
                                vals.push(val);
                            }
                            if let Some((params, _, body)) = self.functions.get(&name) {
                                let new_exec = self.create_subfunction();
                                for (i, (name, _)) in params.iter().enumerate() {
                                    if i < vals.len() {
                                        new_exec.scope.lock().unwrap().variables.insert(name.clone(), vals[i].clone());
                                    } else {
                                        return Err(Error::RuntimeError { message: format!("Function {} expects {} arguments, but got {}", name, params.len(), vals.len()).into() });
                                    }
                                }
                                let result = new_exec.execute_commands(body.nodes.clone())?;
                                if let Some(return_value) = result {
                                    return Ok(return_value);
                                }
                                return Err(Error::RuntimeError { message: format!("Function {} didn't return a value", name).into() });
                            }
                            Err(Error::RuntimeError { message: format!("Unknown function: {}", name).into() })
                        }
                        x => Ok(x)
                    }
                },
                Expression::Unary(op, inner) => {
                    let inner_val = self.calculate_expression(*inner)?;
                    match op {
                        UnaryOperator::UnaryMinus => {
                            match inner_val {
                                BaseValue::Int(num) => Ok(BaseValue::Int((-1) * num)),
                                BaseValue::Float(num) => Ok(BaseValue::Float((-1.0) * num)),
                                v => Err(Error::RuntimeError { message: format!("Cannot apply unary minus to: {:?}", v).into() })
                            }
                        },
                        UnaryOperator::NOT => {
                            match inner_val {
                                BaseValue::Bool(val) => Ok(BaseValue::Bool(!val)),
                                _ => Err(Error::RuntimeError { message: "Unary not only allowed on bool: {}".into() })
                            }
                        }
                        UnaryOperator::Parentheses => Ok(inner_val)
                    }
                },
                Expression::Binary(op, lhs, rhs) => {
                    let left_val = self.calculate_expression(*lhs)?;
                    let right_val = self.calculate_expression(*rhs)?;

                    if let BaseValue::Int(x) = left_val {
                        if let BaseValue::Int(y) = right_val {
                            return compare_ints(x, y, op);
                        }
                        if let BaseValue::Float(y) = right_val {
                            let t = x as f32;
                            return compare_floats(t, y, op);
                        }
                    }

                    if let BaseValue::Float(y) = left_val {
                        if let BaseValue::Int(x) = right_val {
                            let t = x as f32;
                            return compare_floats(y, t, op);
                        }
                        if let BaseValue::Float(x) = right_val {
                            return compare_floats(y, x, op);
                        }
                    }

                    if let BaseValue::Bool(a) = left_val {
                        if let BaseValue::Bool(b) = right_val {
                            return compare_bools(a, b, op);
                        }
                    }

                    Err(Error::RuntimeError { message: "Unsolvable expression!".into() })
                },
            }
        
    }
}


fn compare_ints(x: i32, y : i32, op: Operator) -> Result<BaseValue, Error> {
    match op {

        Operator::EQ => return Ok(BaseValue::Bool(x == y)),
        Operator::NQ => return Ok(BaseValue::Bool(x != y)),
        Operator::GT => return Ok(BaseValue::Bool(x > y)),
        Operator::LT => return Ok(BaseValue::Bool(x < y)),
        Operator::GQ => return Ok(BaseValue::Bool(x >= y)),
        Operator::LQ => return Ok(BaseValue::Bool(x <= y)),
        
        Operator::Plus => Ok(BaseValue::Int(x + y)),
        Operator::Minus => Ok(BaseValue::Int(x - y)),
        Operator::Mult => Ok(BaseValue::Int(x * y)),
        Operator::Div => Ok(BaseValue::Int(x / y)),
        Operator::Mod => Ok(BaseValue::Int(x % y)),
        v => return Err(Error::RuntimeError { message: format!("Cannot apply operator {:?} to values of type int!",v).into()})
    }
}

fn compare_floats(x: f32, y : f32, op: Operator) -> Result<BaseValue, Error> {
    match op {

        Operator::EQ => Ok(BaseValue::Bool(x == y)),
        Operator::NQ => Ok(BaseValue::Bool(x != y)),
        Operator::GT => Ok(BaseValue::Bool(x > y)),
        Operator::LT => Ok(BaseValue::Bool(x < y)),
        Operator::GQ => Ok(BaseValue::Bool(x >= y)),
        Operator::LQ => Ok(BaseValue::Bool(x <= y)),
        
        Operator::Plus => Ok(BaseValue::Float(x + y)),
        Operator::Minus => Ok(BaseValue::Float(x - y)),
        Operator::Mult => Ok(BaseValue::Float(x * y)),
        Operator::Div => Ok(BaseValue::Float(x / y)),
        Operator::Mod => Ok(BaseValue::Float(x % y)),

        v => return Err(Error::RuntimeError { message: format!("Cannot apply operator {:?} to values of type float!",v).into()})

    }
}

fn compare_bools(a: bool, b : bool, op: Operator) -> Result<BaseValue, Error> {
    match op {

        Operator::EQ => Ok(BaseValue::Bool(a == b)),
        Operator::NQ => Ok(BaseValue::Bool(a != b)),
        
        Operator::AND => Ok(BaseValue::Bool(a && b)),
        Operator::OR => Ok(BaseValue::Bool(a || b)),

        o => Err(Error::RuntimeError { message: format!("Cannot apply operator '{:?}' to values of type bool!", o).into() })
    }
}



