use std::{cell::RefCell, error::Error, rc::Rc};

use eval::{eval, Env};
use parser::Object;

mod eval;
mod lexer;
mod parser;

fn main() -> Result<(), Box<dyn Error>> {
    let mut env = Rc::new(RefCell::new(Env::new()));
    let program = "(progn
            (define map
                (lambda (f l)
                    (if (atom l)
                        NIL
                        (cons
                            (f (car l))
                            (map f (cdr l))
                        )
                    )
                )
            )
            (define K 7)
            (define mulK
                (lambda (x)
                    (progn
                        (define L (+ K 1))
                        (* x L)
                    )
                )
            )
            (define cond
                (macro (test body)
                    ('if test
                        body
                        'NIL
                    )
                )
            )
            (cond (> 5 7) 3)
        )";

    let val = eval(program, &mut env)?;
    match val {
        Object::Nil => {}
        Object::Integer(n) => println!("{}", n),
        Object::Bool(b) => println!("{}", b),
        Object::Symbol(s) => println!("{}", s),
        Object::Lambda(params, body) => {
            println!("Lambda(");
            for param in params {
                print!("{} ", param);
            }
            println!(")");
            for expr in body {
                println!("  {}", expr);
            }
        }
        Object::Macro(params, body) => {
            println!("Macro(");
            for param in params {
                print!("{} ", param);
            }
            println!(")");
            for expr in body {
                println!("  {}", expr);
            }
        }
        _ => println!("{}", val),
    }
    Ok(())
}
