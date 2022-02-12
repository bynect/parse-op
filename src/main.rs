use std::collections::HashMap;
use std::io::{self, Write};

mod fixup;
mod parse;

use fixup::{OpAssoc::*, OpInfo, OpKind::*};

pub type OpName = String;

#[derive(Debug, Eq, PartialEq)]
pub enum Expr {
    InfixOp(OpName, Box<Expr>, Box<Expr>),
    PrefixOp(OpName, Box<Expr>),
    PostfixOp(OpName, Box<Expr>),
    Var(String),
}

impl Expr {
    pub fn infixop(op: &str, e1: Expr, e2: Expr) -> Expr {
        Expr::InfixOp(op.to_owned(), Box::new(e1), Box::new(e2))
    }

    pub fn var(x: &str) -> Expr {
        Expr::Var(x.to_owned())
    }

    pub fn print_sexpr(&self) {
        match self {
            Expr::InfixOp(op, e1, e2) => {
                print!("({} ", op);
                e1.print_sexpr();
                print!(" ");
                e2.print_sexpr();
                print!(")");
            }
            Expr::PrefixOp(op, e) => {
                print!("({} ", op);
                e.print_sexpr();
                print!(")");
            }
            Expr::PostfixOp(op, e) => {
                print!("({} ", op);
                e.print_sexpr();
                print!(")");
            }
            Expr::Var(x) => print!("{}", x),
        }
    }

    pub fn print_dot(&self) {
        println!("digraph Expr {{");
        self.print_dot_(&mut 0);
        println!("}}");
    }

    fn print_dot_(&self, acc: &mut usize) {
        match self {
            Expr::InfixOp(op, e1, e2) => {
                let gen = *acc;
                *acc += 1;

                if gen != 0 {
                    println!("InfixOp_{};", gen);
                }

                println!("InfixOp_{} [label=\"{}\"];", gen, op);

                print!("InfixOp_{} -> ", gen);
                e1.print_dot_(acc);

                print!("InfixOp_{} -> ", gen);
                e2.print_dot_(acc);
            }
            Expr::PrefixOp(op, e) => {
                let gen = *acc;
                *acc += 1;

                if gen != 0 {
                    println!("PrefixOp_{};", gen);
                }

                println!("PrefixOp_{} [label=\"{}\"];", gen, op);

                print!("PrefixOp_{} -> ", gen);
                e.print_dot_(acc);
            }
            Expr::PostfixOp(op, e) => {
                let gen = *acc;
                *acc += 1;

                if gen != 0 {
                    println!("PostfixOp_{};", gen);
                }

                println!("PostfixOp_{} [label=\"{}\"];", gen, op);

                print!("PostfixOp_{} -> ", gen);
                e.print_dot_(acc);
            }
            Expr::Var(x) => {
                let gen = *acc;
                *acc += 1;

                if gen != 0 {
                    println!("Var_{};", gen);
                }

                println!("Var_{} [label=\"{}\"];", gen, x);
            }
        }
    }
}

#[test]
fn test() {
    let e = Expr::infixop("+", Expr::var("a"), Expr::var("b"));
    e.print_dot();

    let e = parse::parse("a + b").unwrap().unwrap();
    e.print_dot();
}

fn main() -> Result<(), io::Error> {
    let mut map = HashMap::new();
    map.insert("a".to_owned(), OpInfo::new("+", Infix(Left), vec![]));

    let mut line = String::new();
    loop {
        print!("op# ");
        io::stdout().flush()?;

        line.clear();
        io::stdin().read_line(&mut line)?;

        match parse::parse(&line) {
            None => return Ok(()),
            Some(Ok(e)) => {
                println!("// Raw");
                println!("{:?}", e);
                println!();

                println!("// Sexpr");
                e.print_sexpr();
                println!("\n");

                println!("// Dot");
                print!("// {}", line);
                e.print_dot();
                println!();

                match fixup::fixup(e, &map) {
                    Ok(e) => {
                        println!("// Resolved Raw");
                        println!("{:?}", e);

                        println!("// Resolved Dot");
                        print!("// {}", line);
                        e.print_dot();
                    }
                    Err(e) => println!("Error: {}", e),
                }
            }
            Some(Err(e)) => {
                println!("Error: {}", e);
            }
        }
    }
}
