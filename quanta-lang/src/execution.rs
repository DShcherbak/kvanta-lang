use std::{collections::HashMap, sync::{Arc, Mutex}};

use gloo_timers::future::TimeoutFuture;
use quanta_parser::{ast::{AstBlock, AstNode, AstProgram, AstStatement, BaseValue, BaseValueType, Coords, Expression, ExpressionType, Operator, Type, UnaryOperator, VariableCall}, error::Error};
use quanta_parser::ast::BaseType;
use crate::utils::canvas::Canvas;
use js_sys::Math;
use std::pin::Pin;
use std::future::Future;

//use std::{thread, time::Duration};

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
    // Варіант із полями: BaseValueType::Variant(pats...)
    ($fname:expr, $vals:expr, $idx:expr, $Variant:ident ( $($pat:pat),* ) => $build:expr) => {{
        let __arg_index = $idx; // збережемо, щоб не обчислювати двічі
        match &$vals[__arg_index] {
            BaseValue{val: BaseValueType::$Variant($($pat),*), coords: _} => { $build }
            other => {
                return Err(Error::runtime(
                        format!(
                        "{}: arg #{}: Expected argument type {} but got {}",
                        $fname,
                        __arg_index,            
                        stringify!($Variant),
                        other.get_type(&|_| Some(Type::typ(BaseType::Int)))?.to_string()
                    ), other.coords));
            }
        }
    }};
}

fn update_array(name: String, array: &mut BaseValue, mut integer_indices: Vec<i32>, val: BaseValue) -> Result<(), Error> {
        if let BaseValueType::Array(elems) = &mut array.val {
            let index = integer_indices.remove(0);
            if index < 0 || index as usize >= elems.len() {
                return Err(Error::runtime(format!("Index out of bounds for array {}: {}", name, index), array.coords));
            }
            if integer_indices.len() == 0 {
                elems[index as usize] = val;
                return Ok(());
            }
            return update_array(format!("{}[{}]", name, index), elems.get_mut(index as usize).unwrap(), integer_indices, val);
        } else {
            return Err(Error::runtime(format!("Variable {} is not an array", name), array.coords));
        }
    }

fn int(i: i32, coords:Coords) -> BaseValue {
    BaseValue{ val:BaseValueType::Int(i), coords} 
}

fn flt(i: f32, coords:Coords) -> BaseValue {
    BaseValue{ val:BaseValueType::Float(i), coords} 
}

fn bol(i: bool, coords:Coords) -> BaseValue {
    BaseValue{ val:BaseValueType::Bool(i), coords} 
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

    async fn get_variable(&mut self, var: &VariableCall, coords: Coords) -> Result<BaseValue, Error> {
        match var {
            VariableCall::Name(name) => self.get(name).ok_or(Error::runtime(format!("Unknown variable: {}", name), coords)),
            VariableCall::ArrayCall(name, indices) => {
                if !self.contains_key(name) {
                    return Err(Error::runtime(format!("Unknown array 1: {}, variables: {:?}", name, self.scope), coords));
                }
                if indices.is_empty() {
                    return Err(Error::runtime(String::from("Empty index"), coords));
                }
                let mut integer_indices: Vec<i32> = vec![];
                for index in indices {
                    match self.calculate_expression(index.clone().to_expr()).await?.val {
                        BaseValueType::Int(i) => {
                            if i < 0 {
                                return Err(Error::runtime(format!("Negative index for array {}: {}", name, i), coords));
                            }
                            integer_indices.push(i);
                        },
                        _ => return Err(Error::runtime(String::from("Array indices must be integers"), coords)),
                    }
                }
                let maybe_array = self.get(name);
                if maybe_array.is_none() {
                    return Err(Error::runtime(format!("Unknown array: {} ", name), coords));
                }
                let mut array = maybe_array.unwrap();
                while integer_indices.len() > 0 {
                    if let BaseValueType::Array(elems) = array.val {
                        let index = integer_indices.remove(0);
                        if index < 0 || index as usize >= elems.len() {
                            return Err(Error::runtime(format!("Index out of bounds for array {}: {}", name, index), coords));
                        }
                        array = elems.get(index as usize).unwrap().clone();
                    } else {
                        return Err(Error::runtime(format!("Variable {} is not an array", name), coords));
                    }
                }
                Ok(array)
            }
        }
    }

    

    async fn set_variable(&mut self, var: &VariableCall, val: BaseValue, coords: Coords) -> Result<(), Error> {
        match var {
            VariableCall::Name(name) => if self.set(name.clone(), val) { return Ok(()); } else { return Err(Error::runtime(format!("Unknown variable: {}", name), coords));},
            VariableCall::ArrayCall(name, indices) => {
                if !self.contains_key(name) {
                    return Err(Error::runtime(format!("Unknown array 1: {}, variables: {:?}", name, self.scope), coords));
                }
                if indices.is_empty() {
                    return Err(Error::runtime(String::from("Empty index"), coords));
                }
                let mut integer_indices: Vec<i32> = vec![];
                for index in indices {
                    match self.calculate_expression(index.clone().to_expr()).await?.val {
                        BaseValueType::Int(i) => {
                            if i < 0 {
                                return Err(Error::runtime(format!("Negative index for array {}: {}", name, i), coords));
                            }
                            integer_indices.push(i);
                        },
                        _ => return Err(Error::runtime(String::from("Array indices must be integers"), coords)),
                    }
                }
                let maybe_array = self.get(name);
                if maybe_array.is_none() {
                    return Err(Error::runtime(format!("Unknown array: {} ", name), coords));
                }
                let mut array = maybe_array.unwrap();
                update_array(name.clone(), &mut array, integer_indices, val)?;
                if self.set(name.clone(), array) { 
                    Ok(()) 
                } else { 
                    Err(Error::runtime(format!("Unknown variable: {}", name), coords))
                }
            }
        }
    }

    async fn execute_function(&mut self, function_name: &str, args: Vec<Expression>, coords: Coords) -> Result<Option<BaseValue>, Error>{
        let mut vals : Vec<BaseValue> = vec![];
        for arg in args {
            let val = self.calculate_expression(arg).await?;
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
                    if let BaseValueType::Int(num) = val.val {
                        nums.push_str(&format!("{} ", num));
                    } else {
                        return Err(Error::runtime(String::from("Incorrect arguments for polygon function!"), val.coords));
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
                if let BaseValueType::Color(r,g,b) = &vals[0].val {
                    let mut inner  = self.line_color.lock().unwrap();
                    *inner = color_to_str(r, g, b);
                    Ok(None)
                }
                else if let BaseValueType::RandomColor = &vals[0].val {
                    let r = (255.0 * Math::random()) as u8;
                    let g = (255.0 * Math::random()) as u8;
                    let b = (255.0 * Math::random()) as u8;
                    let mut inner  = self.line_color.lock().unwrap();
                    *inner = color_to_str(&r, &g, &b);
                    Ok(None)
                }
                else {
                    Err(Error::runtime(format!("Incorrect arguments for setLineColor function: expected a color, got {:?}!", &vals[0]), coords))
                }
            },
            "setFigureColor" => {
                if let BaseValueType::Color(r,g,b) = &vals[0].val {
                    let mut inner  = self.figure_color.lock().unwrap();
                    *inner = color_to_str(r, g, b);
                    Ok(None)
                }
                else if let BaseValueType::RandomColor = &vals[0].val {
                    let r = (255.0 * Math::random()) as u8;
                    let g = (255.0 * Math::random()) as u8;
                    let b = (255.0 * Math::random()) as u8;
                    let mut inner  = self.figure_color.lock().unwrap();
                    *inner =  color_to_str(&r, &g, &b);
                    Ok(None)
                }
                else {
                    Err(Error::runtime(format!("Incorrect arguments for setFigureColor function: expected a color, got {:?}!", &vals[0]), coords))
                }
            },
            "setLineWidth" => {
                let width = expect_arg!("setLineWidth", vals, 0, Int(width) => *width);
                if width >= 0 {
                    let mut inner  = self.line_width.lock().unwrap();
                    *inner = width;
                    Ok(None)
                } else {
                    Err(Error::runtime(String::from("Line width can't be negative!"), coords))
                }
            },
            "sleep" => {
                let sleep_time = expect_arg!("sleep", vals, 0, Int(time) => *time);
                if sleep_time >= 0 {
                    //thread::sleep(Duration::from_millis(1000));
                    self.canvas.add_command(format!("sleep {}", sleep_time));

                    Ok(None)
                } else {
                    Err(Error::runtime(String::from("Sleep time can't be negative!"), coords))
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
            }
            name => {
                if self.functions.contains_key(name) {
                    let (params, _, body) = self.functions.get(name).unwrap();
                    if params.len() != vals.len() {
                        return Err(Error::runtime(format!("Function {} expects {} arguments, but got {}", name, params.len(), vals.len()), coords));
                    }
                    let mut new_exec = self.create_subfunction();
                    for (i, param) in params.iter().enumerate() {
                        new_exec.scope.lock().unwrap().variables.insert(param.0.clone(), vals[i].clone());
                    }
                    let ret_val_wrap = new_exec.execute_commands(body.nodes.clone()).await?;

                    if let Some(return_value) = ret_val_wrap {
                        return Ok(Some(return_value));
                    }
                    return Ok(None);
                }
                Err(Error::runtime(format!("Unknown function: {}", function_name), coords))
            }
        }
    }

    async fn execute_init(&mut self, var: String, expr: Expression, coords: Coords) -> Result<(), Error>{
        let value = self.calculate_expression(expr).await?;

        if let Some(_) = self.get(&var) {
            return Err(Error::runtime(format!("Variable {} is already defined!", &var), coords));
        }
        self.scope.lock().unwrap().variables.insert(var, value);
        Ok(())
    }

    async fn execute_set(&mut self, var: &VariableCall, expr: Expression, coords: Coords) -> Result<(), Error> {
        let value = self.calculate_expression(expr).await?;
        if self.get_variable(var, coords).await.is_ok() {
            self.set_variable(var, value, coords).await?;
            return Ok(())
        }
        Err(Error::runtime(String::from("Couldn't set new value"), coords))
    }

    pub async fn execute(&mut self) -> Result<(), Error> {
        match self.lines {
            AstProgram::Block(ref block) => {
                self.execute_commands(block.nodes.clone()).await?;
                self.canvas.add_command("end".into());
            },
            AstProgram::Forest(ref funcs) => {
                for func in &funcs.0 {
                    if func.name == "main" {
                        let mut new_exec = self.create_subscope();
                        new_exec.execute_commands(func.block.nodes.clone()).await?;
                        self.canvas.add_command("end".into());
                    }
                }
                return Err(Error::runtime(String::from("No main function found"), (0,0,0,0)));
            },
        }
        Ok(())
    }

    pub async fn execute_key(&mut self, key: i32) -> Result<(), Error> {
        match self.lines {
            AstProgram::Block(_) => { Ok(())},
            AstProgram::Forest(ref funcs) => {
                for func in &funcs.0 {
                    if func.name == "keyboard" {
                        let mut new_exec = self.create_subscope();
                        new_exec.execute_init(func.args.get(0).unwrap().0.clone(), 
                        Expression{expr_type: ExpressionType::Value(
                                    BaseValue{val: BaseValueType::Int(key), coords: func.header}), coords:func.header}, func.header).await?;
                        new_exec.execute_commands(func.block.nodes.clone()).await?;
                    }
                }
                Ok(())
            },
        }
    }

    pub async fn execute_mouse(&mut self, x: i32, y:i32) -> Result<(), Error> {
        match self.lines {
            AstProgram::Block(_) => { Ok(())},
            AstProgram::Forest(ref funcs) => {
                for func in &funcs.0 {
                    if func.name == "mouse" {
                        let mut new_exec = self.create_subscope();
                        new_exec.execute_init(func.args.get(0).unwrap().0.clone(), 
                        Expression{expr_type: ExpressionType::Value(
                                    BaseValue{val: BaseValueType::Int(x), coords: func.header}), coords:func.header}, func.header).await?;
                        new_exec.execute_init(func.args.get(1).unwrap().0.clone(), 
                        Expression{expr_type: ExpressionType::Value(
                                    BaseValue{val: BaseValueType::Int(y), coords: func.header}), coords:func.header}, func.header).await?;
                        new_exec.execute_commands(func.block.nodes.clone()).await?;
                    }
                }
                Ok(())
            },
        }
    }

    pub fn execute_commands<'a>(&'a mut self, nodes : Vec<AstNode>) -> Pin<Box<dyn Future<Output = Result<Option<BaseValue>, Error>> + 'a>> {
        Box::pin(async move {
            TimeoutFuture::new(1).await;
            for line in nodes {
                match line.statement {
                    AstStatement::Command { name, args } => {
                        self.execute_function(&name, args, line.coords).await?;
                    },
                    AstStatement::Init { typ : _, val, expr } => {
                        self.execute_init(val, expr, line.coords).await?;
                    }
                    AstStatement::SetVal { val, expr } => {
                        self.execute_set(&val, expr, line.coords).await?;
                    }
                    
                    AstStatement::If { clause, block, else_block } => {
                        if let BaseValueType::Bool(val) = self.calculate_expression(clause).await?.val {
                            let mut new_exec = self.create_subscope();
                            if val {
                                if let Some(return_value) = new_exec.execute_commands(block.nodes).await? {
                                    return Ok(Some(return_value));
                                }
                            } else if let Some(else_block) = else_block {
                                if let Some(return_value) = new_exec.execute_commands(else_block.nodes).await? {
                                    return Ok(Some(return_value));
                                }
                            }
                        } else {
                            return Err(Error::runtime(String::from("If clause must be a boolean expression"), line.coords));
                        }
                    },
                    AstStatement::While { clause, block } => {
                        loop {
                            match self.calculate_expression(clause.clone()).await?.val {
                                BaseValueType::Bool(while_clause) => {
                                    if while_clause {
                                        let mut new_exec = self.create_subscope();
                                        let result = new_exec.execute_commands(block.nodes.clone()).await?;
                                        if let Some(return_value) = result {
                                            return Ok(Some(return_value));
                                        }
                                    } else {
                                        break;
                                    }
                                },
                                v => return Err(Error::runtime(format!("Expected bool value but got: {:?}", v), line.coords))
                            }
                        }
                    },
                    AstStatement::For { val, from, to, block } => {
                        if let BaseValueType::Int(f_) = from.val {
                            if let BaseValueType::Int(t_) = to.val {
                                let (f,t) = {
                                    if f_ <= t_ {
                                        (f_, t_)
                                    } else {
                                        (t_, f_)
                                    }
                                };
                                for cycle in f..=t {
                                    if let Some(return_value) = self.execute_for(val.clone(), cycle, block.clone(), line.coords).await?{
                                        return Ok(Some(return_value));
                                    }
                                }                    
                            }
                        }
                    },
                    AstStatement::Return { expr } => {
                        let val = self.calculate_expression(expr).await?;
                        return Ok(Some(val.clone()))
                    },
                }
            }
            Ok(None)
        })
    }

    async fn execute_for(&mut self, val: String, cycle : i32, block : AstBlock, coords: Coords) -> Result<Option<BaseValue>, Error> {
        let mut new_exec = self.create_subscope();
        new_exec.execute_init(val, 
                        Expression{expr_type: ExpressionType::Value(
                                    BaseValue{val: BaseValueType::Int(cycle), coords}), coords}, coords).await?;
        let result = new_exec.execute_commands(block.nodes.clone()).await;
        result
    }



    pub fn calculate_expression<'a>(
        &'a mut self,
        expr: Expression,
    ) -> Pin<Box<dyn Future<Output = Result<BaseValue, Error>> + 'a>> {
        
        Box::pin(async move {
            match expr.expr_type {
                ExpressionType::Value(base_value) => {
                    match base_value.val {
                        BaseValueType::Id(var) => {
                            self.get_variable(&var, expr.coords).await
                        },
                        BaseValueType::FunctionCall(name, exprs, _ ) => {
                            let mut vals = vec![];
                            for expr in exprs {
                                let val = self.calculate_expression(expr).await?;
                                vals.push(val);
                            }
                            if let Some((params, _, body)) = self.functions.get(&name) {
                                let mut new_exec = self.create_subfunction();
                                for (i, (name, _)) in params.iter().enumerate() {
                                    if i < vals.len() {
                                        new_exec.scope.lock().unwrap().variables.insert(name.clone(), vals[i].clone());
                                    } else {
                                        return Err(Error::runtime(format!("Function {} expects {} arguments, but got {}", name, params.len(), vals.len()), expr.coords));
                                    }
                                }
                                let result = new_exec.execute_commands(body.nodes.clone()).await?;
                                if let Some(return_value) = result {
                                    return Ok(return_value);
                                }
                                return Err(Error::runtime(format!("Function {} didn't return a value", name), expr.coords));
                            }
                            Err(Error::runtime(format!("Unknown function: {}", name), expr.coords))
                        }
                        x => Ok(BaseValue { val: x, coords: base_value.coords })
                    }
                },
                ExpressionType::Unary(op, inner) => {
                    let inner_val = self.calculate_expression(*inner).await?;
                    match op {
                        UnaryOperator::UnaryMinus => {
                            match inner_val.val {
                                BaseValueType::Int(num) => Ok(int((-1) * num, inner_val.coords)),
                                BaseValueType::Float(num) => Ok(flt((-1.0) * num, inner_val.coords)),
                                v => Err(Error::runtime(format!("Cannot apply unary minus to: {:?}", v), inner_val.coords))
                            }
                        },
                        UnaryOperator::NOT => {
                            match inner_val.val {
                                BaseValueType::Bool(val) => Ok(bol(!val, inner_val.coords)),
                                _ => Err(Error::runtime(String::from("Unary not only allowed on bool: {}"), inner_val.coords))
                            }
                        }
                        UnaryOperator::Parentheses => Ok(inner_val)
                    }
                },
                ExpressionType::Binary(op, lhs, rhs) => {
                    let left_val = self.calculate_expression(*lhs).await?;
                    let right_val = self.calculate_expression(*rhs).await?;

                    if let BaseValueType::Int(x) = left_val.val {
                        if let BaseValueType::Int(y) = right_val.val {
                            return compare_ints(x, y, op, expr.coords);
                        }
                        if let BaseValueType::Float(y) = right_val.val {
                            let t = x as f32;
                            return compare_floats(t, y, op, expr.coords);
                        }
                    }

                    if let BaseValueType::Float(y) = left_val.val {
                        if let BaseValueType::Int(x) = right_val.val {
                            let t = x as f32;
                            return compare_floats(y, t, op, expr.coords);
                        }
                        if let BaseValueType::Float(x) = right_val.val {
                            return compare_floats(y, x, op, expr.coords);
                        }
                    }

                    if let BaseValueType::Bool(a) = left_val.val {
                        if let BaseValueType::Bool(b) = right_val.val {
                            return compare_bools(a, b, op, expr.coords);
                        }
                    }

                    Err(Error::runtime(String::from("Unsolvable expression!"), expr.coords))
                },
            }
        })
    }
}


fn compare_ints(x: i32, y : i32, op: Operator, coords: Coords) -> Result<BaseValue, Error> {
    match op {

        Operator::EQ => return Ok(bol(x == y, coords)),
        Operator::NQ => return Ok(bol(x != y, coords)),
        Operator::GT => return Ok(bol(x > y, coords)),
        Operator::LT => return Ok(bol(x < y, coords)),
        Operator::GQ => return Ok(bol(x >= y, coords)),
        Operator::LQ => return Ok(bol(x <= y, coords)),
        
        Operator::Plus => Ok(int(x + y, coords)),
        Operator::Minus => Ok(int(x - y, coords)),
        Operator::Mult => Ok(int(x * y, coords)),
        Operator::Div => Ok(int(x / y, coords)),
        Operator::Mod => Ok(int(x % y, coords)),
        v => Err(Error::runtime(format!("Cannot apply operator {:?} to values of type int!",v), coords))   
    }
}

fn compare_floats(x: f32, y : f32, op: Operator, coords: Coords) -> Result<BaseValue, Error> {
    match op {

        Operator::EQ => Ok(bol(x == y, coords)),
        Operator::NQ => Ok(bol(x != y, coords)),
        Operator::GT => Ok(bol(x > y, coords)),
        Operator::LT => Ok(bol(x < y, coords)),
        Operator::GQ => Ok(bol(x >= y, coords)),
        Operator::LQ => Ok(bol(x <= y, coords)),
        
        Operator::Plus => Ok(flt(x + y, coords)),
        Operator::Minus => Ok(flt(x - y, coords)),
        Operator::Mult => Ok(flt(x * y, coords)),
        Operator::Div => Ok(flt(x / y, coords)),
        Operator::Mod => Ok(flt(x % y, coords)),

        v => Err(Error::runtime(format!("Cannot apply operator {:?} to values of type float!",v), coords))

    }
}

fn compare_bools(a: bool, b : bool, op: Operator, coords: Coords) -> Result<BaseValue, Error> {
    match op {

        Operator::EQ => Ok(bol(a == b, coords)),
        Operator::NQ => Ok(bol(a != b, coords)),
        
        Operator::AND => Ok(bol(a && b, coords)),
        Operator::OR => Ok(bol(a || b, coords)),

        o => Err(Error::runtime(format!("Cannot apply operator '{:?}' to values of type bool!", o), coords))
    }
}



