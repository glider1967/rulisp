use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    rc::Rc,
};

use crate::parser::{parse_program, Object};

pub struct Env {
    parent: Option<Rc<RefCell<Env>>>,
    vars: HashMap<String, Object>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            parent: None,
            vars: HashMap::new(),
        }
    }

    fn get_object(&self, name: &str) -> Option<Object> {
        match self.vars.get(name) {
            Some(val) => Some(val.clone()),
            None => match &self.parent {
                Some(env) => env.borrow().get_object(name),
                None => None,
            },
        }
    }

    fn set_object(&mut self, name: &str, val: Object) {
        self.vars.insert(name.to_string(), val);
    }

    fn extend(parent: Rc<RefCell<Self>>) -> Self {
        Self {
            parent: Some(parent),
            vars: HashMap::new(),
        }
    }
}

fn eval_binary_op(list: &VecDeque<Object>, env: &mut Rc<RefCell<Env>>) -> Result<Object, String> {
    if list.len() != 3 {
        return Err("Invalid number of arguments for binary operator".to_string());
    };

    let op = list[0].clone();
    let left = {
        let obj = eval_object(&list[1], env)?;
        match obj {
            Object::Integer(n) => n,
            _ => return Err(format!("Left operand must be an integer {:?}", obj)),
        }
    };
    let right = {
        let obj = eval_object(&list[2], env)?;
        match obj {
            Object::Integer(n) => n,
            _ => return Err(format!("Right operand must be an integer {:?}", obj)),
        }
    };

    match op {
        Object::Symbol(s) => match s.as_str() {
            "+" => Ok(Object::Integer(left + right)),
            "-" => Ok(Object::Integer(left - right)),
            "*" => Ok(Object::Integer(left * right)),
            "/" => {
                if right != 0 {
                    Ok(Object::Integer(left / right))
                } else {
                    Err("Connot divide by 0".to_string())
                }
            }
            "<" => Ok(Object::Bool(left < right)),
            ">" => Ok(Object::Bool(left > right)),
            "==" => Ok(Object::Bool(left == right)),
            "!=" => Ok(Object::Bool(left != right)),
            _ => Err("Operator must be symbol".to_string()),
        },
        _ => Err("Operator must be a symbol".to_string()),
    }
}

fn eval_define(list: &VecDeque<Object>, env: &mut Rc<RefCell<Env>>) -> Result<Object, String> {
    if list.len() != 3 {
        return Err("Invalid number of arguments for define".to_string());
    };

    let sym = match &list[1] {
        Object::Symbol(name) => name,
        _ => return Err("Invalid identifier for define".to_string()),
    };
    let val = eval_object(&list[2], env)?;
    env.borrow_mut().set_object(&sym, val);
    Ok(Object::Nil)
}

fn eval_if(list: &VecDeque<Object>, env: &mut Rc<RefCell<Env>>) -> Result<Object, String> {
    if list.len() != 4 {
        return Err("Invalid number of arguments for if statement".to_string());
    };

    let cond = {
        let obj = eval_object(&list[1], env)?;
        match obj {
            Object::Bool(b) => b,
            _ => return Err("Condition must be a boolean".to_string()),
        }
    };

    if cond {
        eval_object(&list[2], env)
    } else {
        eval_object(&list[3], env)
    }
}

fn eval_lambda(list: &VecDeque<Object>) -> Result<Object, String> {
    if list.len() != 3 {
        return Err("Invalid number of arguments for lambda statement".to_string());
    };

    let params = match &list[1] {
        Object::List(list) => {
            let mut params = vec![];
            for param in list {
                match param {
                    Object::Symbol(s) => params.push(s.clone()),
                    _ => return Err("Invalid lambda parameter: not symbol".to_string()),
                }
            }
            params
        }
        _ => return Err("Invalid lambda: first argument is not list".to_string()),
    };

    let body = match &list[2] {
        Object::List(list) => list.clone(),
        _ => return Err("Invalid lambda: body is not list".to_string()),
    };

    Ok(Object::Lambda(params, body))
}

fn eval_func_call(
    name: &str,
    list: &VecDeque<Object>,
    env: &mut Rc<RefCell<Env>>,
) -> Result<Object, String> {
    let func = {
        let lambda = env.borrow().get_object(name);
        if lambda.is_none() {
            return Err(format!("Unbound func: {}", name));
        };
        lambda.unwrap()
    };

    match func {
        Object::Lambda(params, body) => {
            if params.len() != list.len() - 1 {
                return Err(format!(
                    "Invalid call of function `{}`: number of arguments is not correct",
                    name
                ));
            }

            let mut new_env = Rc::new(RefCell::new(Env::extend(env.clone())));
            for (i, param) in params.iter().enumerate() {
                let val = eval_object(&list[i + 1], env)?;
                new_env.borrow_mut().set_object(param, val);
            }
            eval_object(&Object::List(body), &mut new_env)
        }
        _ => Err(format!("Not a lambda: {}", name)),
    }
}

fn eval_atom(list: &VecDeque<Object>, env: &mut Rc<RefCell<Env>>) -> Result<Object, String> {
    if list.len() != 2 {
        return Err("Invalid number of arguments for atom".to_string());
    }

    match eval_object(&list[1], env)? {
        Object::Nil | Object::Bool(_) | Object::Integer(_) | Object::Symbol(_) => {
            Ok(Object::Bool(true))
        }
        _ => Ok(Object::Bool(false)),
    }
}

fn eval_symbol(name: &str, env: &mut Rc<RefCell<Env>>) -> Result<Object, String> {
    match name {
        "NIL" => Ok(Object::Nil),
        "T" => Ok(Object::Bool(true)),
        "F" => Ok(Object::Bool(false)),
        _ => {
            let val = env.borrow().get_object(name);
            if val.is_none() {
                return Err(format!("Unbound sysmbol: {}", name));
            }
            Ok(val.unwrap())
        }
    }
}

fn eval_quote(list: &VecDeque<Object>) -> Result<Object, String> {
    if list.len() != 2 {
        return Err("Invalid number of arguments for quote statement".to_string());
    }

    Ok(list[1].clone())
}

fn eval_cons(list: &VecDeque<Object>, env: &mut Rc<RefCell<Env>>) -> Result<Object, String> {
    if list.len() != 3 {
        return Err("Invalid number of arguments for car".to_string());
    }

    let car = eval_object(&list[1], env)?;
    let cdr = eval_object(&list[2], env)?;
    match &cdr {
        Object::List(list) => {
            let mut new_list = list.clone();
            new_list.push_front(car);
            Ok(Object::List(new_list))
        }
        Object::Nil => {
            let new_list = VecDeque::from([car]);
            Ok(Object::List(new_list))
        }
        _ => Err(format!(
            "Second argument of cons should be list or NIL, found {}",
            cdr
        )),
    }
}

fn eval_car(list: &VecDeque<Object>, env: &mut Rc<RefCell<Env>>) -> Result<Object, String> {
    if list.len() != 2 {
        return Err("Invalid number of arguments for car".to_string());
    }

    match eval_object(&list[1], env)? {
        Object::List(list) => eval_object(&list[0], env),
        _ => Err("Invalid car: argument is not list".to_string()),
    }
}

fn eval_cdr(list: &VecDeque<Object>, env: &mut Rc<RefCell<Env>>) -> Result<Object, String> {
    if list.len() != 2 {
        return Err("Invalid number of arguments for cdr".to_string());
    }

    match eval_object(&list[1], env)? {
        Object::List(list) => {
            let mut list = list.clone();
            let _ = list.pop_front();
            if list.is_empty() {
                Ok(Object::Nil)
            } else {
                Ok(Object::List(list))
            }
        }
        _ => Err("Invalid cdr: argument is not list".to_string()),
    }
}

fn eval_progn(list: &VecDeque<Object>, env: &mut Rc<RefCell<Env>>) -> Result<Object, String> {
    let mut res = Ok(Object::Nil);
    for i in 1..list.len() {
        res = eval_object(&list[i], env);
    }
    res
}

fn eval_list(list: &VecDeque<Object>, env: &mut Rc<RefCell<Env>>) -> Result<Object, String> {
    if list.is_empty() {
        return Ok(Object::Nil);
    }

    let head = &list[0];
    match head {
        Object::Symbol(s) => match s.as_str() {
            "+" | "-" | "*" | "/" | "<" | ">" | "==" | "!=" => {
                return eval_binary_op(&list, env);
            }
            "define" => eval_define(&list, env),
            "if" => eval_if(&list, env),
            "lambda" => eval_lambda(&list),
            "atom" => eval_atom(&list, env),
            "quote" => eval_quote(&list),
            "cons" => eval_cons(&list, env),
            "car" => eval_car(&list, env),
            "cdr" => eval_cdr(&list, env),
            "progn" => eval_progn(&list, env),
            _ => eval_func_call(&s, &list, env),
        },
        _ => {
            let mut new_list = VecDeque::new();
            for obj in list {
                let res = eval_object(obj, env)?;
                match res {
                    Object::Nil => {}
                    _ => new_list.push_back(res),
                }
            }
            Ok(Object::List(new_list))
        }
    }
}

fn eval_object(obj: &Object, env: &mut Rc<RefCell<Env>>) -> Result<Object, String> {
    match obj {
        Object::Nil => Ok(Object::Nil),
        Object::Integer(n) => Ok(Object::Integer(*n)),
        Object::Bool(b) => Ok(Object::Bool(*b)),
        Object::Symbol(name) => eval_symbol(name, env),
        Object::Lambda(_, _) => Ok(Object::Nil),
        Object::List(list) => eval_list(list, env),
    }
}

pub fn eval(program: &str, env: &mut Rc<RefCell<Env>>) -> Result<Object, String> {
    let parsed_list = parse_program(program)?;
    eval_object(&parsed_list, env)
}
