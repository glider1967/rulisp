use core::fmt;
use std::collections::VecDeque;

use crate::lexer::{lex, Token};

#[derive(Debug, Clone, PartialEq)]
pub enum Object {
    Nil,
    Integer(i64),
    Bool(bool),
    Symbol(String),
    Lambda(Vec<String>, VecDeque<Object>),
    List(VecDeque<Object>),
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Object::Nil => write!(f, "NIL"),
            Object::Integer(n) => write!(f, "{}", n),
            Object::Bool(b) => write!(f, "{}", b),
            Object::Symbol(s) => write!(f, "{}", s),
            Object::Lambda(params, body) => {
                write!(f, "Lambda(")?;
                for param in params {
                    write!(f, "{} ", param)?;
                }
                write!(f, ")")?;
                for expr in body {
                    write!(f, " {}", expr)?;
                }
                Ok(())
            }
            Object::List(list) => {
                write!(f, "(")?;
                for (i, obj) in list.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", obj)?;
                }
                write!(f, ")")
            }
        }
    }
}

fn parse(tokens: &mut Vec<Token>) -> Result<Object, String> {
    let token = tokens.pop();
    if Some(Token::LParen) != token {
        return Err(format!("Expected LParen, found {:?}", token));
    }

    let mut list = VecDeque::new();

    while let Some(token) = tokens.pop() {
        match token {
            Token::Integer(n) => list.push_back(Object::Integer(n)),
            Token::Symbol(s) => list.push_back(Object::Symbol(s)),
            Token::LParen => {
                tokens.push(Token::LParen);
                let sub = parse(tokens)?;
                list.push_back(sub);
            }
            Token::RParen => {
                return Ok(Object::List(list));
            }
            Token::Quote => {
                if let Some(next) = tokens.pop() {
                    let next_obj = match next {
                        Token::Integer(n) => Object::Integer(n),
                        Token::Symbol(s) => Object::Symbol(s),
                        Token::LParen => {
                            tokens.push(Token::LParen);
                            parse(tokens)?
                        }
                        _ => return Err("Invalid quote".to_string()),
                    };
                    let mut new_list = VecDeque::new();
                    new_list.push_back(Object::Symbol("quote".to_string()));
                    new_list.push_back(next_obj);
                    list.push_back(Object::List(new_list));
                }
            }
        }
    }

    Ok(Object::List(list))
}

pub fn parse_program(program: &str) -> Result<Object, String> {
    let tokens = lex(program);
    let mut tokens = tokens.into_iter().rev().collect();
    let parsed_list = parse(&mut tokens)?;
    Ok(parsed_list)
}
