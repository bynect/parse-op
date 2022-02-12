use crate::{Expr, OpName};
use std::iter::Peekable;

#[derive(Debug, Eq, PartialEq, Clone)]
enum Token {
    Eof,
    LParen,
    RParen,
    Op(OpName),
    Var(String),
}

macro_rules! symbols {
    () => { '@' | '$' | '+' | '-' | '|' | '~' | '*' | '%' | '\\' | '=' | '#' | '>' | '<' | '!' | '?' | ':' | '^' | '&' | '.' };
}

type Lexer<'a, I> = (Peekable<I>, Option<Token>);

fn backtrack<I: Iterator<Item = char>>(lex: &mut Lexer<I>, t: Token) {
    debug_assert!(lex.1.is_none());
    lex.1 = Some(t);
}

//fn peek_eof<I: Iterator<Item = char>>(lex: &mut Lexer<I>) -> bool {
//    let it = &mut lex.0;
//    it.peek().is_none() && lex.1.is_none()
//}
//
//fn peek_char_or_token<I: Iterator<Item = char>>(lex: &mut Lexer<I>, c: char, t: Token) -> bool {
//    let it = &mut lex.0;
//    if let Some(&c_) = it.peek() {
//        c_ == c
//    } else if let Some(t_) = &lex.1 {
//        *t_ == t
//    } else {
//        false
//    }
//}

fn token<I: Iterator<Item = char>>(lex: &mut Lexer<I>) -> Result<Token, String> {
    if let Some(t) = lex.1.take() {
        return Ok(t);
    }

    let it = &mut lex.0;
    while let Some(&c) = it.peek() {
        match c {
            '(' => {
                it.next();
                return Ok(Token::LParen);
            }
            ')' => {
                it.next();
                return Ok(Token::RParen);
            }
            c if c.is_alphabetic() => {
                let mut buf = String::new();
                while let Some(&c) = it.peek() {
                    if c.is_alphanumeric() {
                        buf.push(c);
                        it.next();
                    } else {
                        break;
                    }
                }

                return Ok(Token::Var(buf));
            }
            //'a'..='z' | 'A'..='Z' => {
            //    let mut buf = String::new();
            //    while let Some(&c) = it.peek() {
            //        match c {
            //            'a'..='z' | 'A'..='Z' | '_' | '0'..='9' => {
            //                buf.push(c);
            //                it.next();
            //            }
            //            _ => break,
            //        }
            //    }

            //    while let Some(&c) = it.peek() {
            //        if c == '\'' {
            //            buf.push(c);
            //            it.next();
            //        } else {
            //            break;
            //        }
            //    }

            //    return Ok(if buf == "let" {
            //        Token::Let
            //    } else if buf == "in" {
            //        Token::In
            //    } else {
            //        Token::Var(buf)
            //    });
            //}
            '/' => {
                if let Some('/') = it.next() {
                    while let Some(&c) = it.peek() {
                        if c == '\n' {
                            break;
                        } else {
                            it.next();
                        }
                    }
                }
            }
            symbols!() => return Ok(token_op(lex)),
            ' ' | '\t' | '\n' => {
                it.next();
            }
            c => Err(format!("Unexpected char {:?}", c))?,
        };
    }
    Ok(Token::Eof)
}

fn token_op<I: Iterator<Item = char>>(lex: &mut Lexer<I>) -> Token {
    let it = &mut lex.0;
    let mut buf = String::new();
    while let Some(&c) = it.peek() {
        match c {
            symbols!() => {
                buf.push(c);
                it.next();
            }
            _ => break,
        }
    }

    debug_assert!(buf.len() > 0);
    Token::Op(buf)
}

fn expect<I: Iterator<Item = char>>(lex: &mut Lexer<I>, e: Token) -> Result<Token, String> {
    let t = token(lex)?;
    if t != e {
        Err(format!("Expected {:?} but got {:?}", e, t))
    } else {
        Ok(t)
    }
}

fn peek<I: Iterator<Item = char>>(lex: &mut Lexer<I>) -> Result<Token, String> {
    Ok(if let Some(t) = &lex.1 {
        t.clone()
    } else {
        let t = token(lex)?;
        backtrack(lex, t.clone());
        t
    })
}

fn parse_expr<I: Iterator<Item = char>>(lex: &mut Lexer<I>) -> Result<Expr, String> {
    let mut e1 = match token(lex)? {
        Token::LParen => {
            let e = parse_expr(lex)?;
            expect(lex, Token::RParen)?;
            e
        }
        Token::Op(op) => {
            let e = parse_expr(lex)?;
            Expr::PrefixOp(op, Box::new(e))
        }
        Token::Var(x) => Expr::Var(x),
        t => return Err(format!("Unexpected token {:?}", t)),
    };

    loop {
        match token(lex)? {
            Token::Op(op) => match peek(lex)? {
                Token::Eof | Token::RParen => {
                    e1 = Expr::PostfixOp(op, Box::new(e1));
                }
                _ => {
                    let e2 = parse_expr(lex)?;
                    e1 = Expr::InfixOp(op, Box::new(e1), Box::new(e2))
                }
            },
            t => {
                backtrack(lex, t);
                break Ok(e1);
            }
        }
    }
}

// Result<T, E> -> T | Option<Result<T, E>>
macro_rules! trans {
    ($e:expr) => {
        match $e {
            Ok(e) => e,
            Err(e) => return Some(Err(e)),
        }
    };
}

pub fn parse(s: &str) -> Option<Result<Expr, String>> {
    let mut lex = (s.chars().peekable(), None);
    match trans!(token(&mut lex)) {
        Token::Eof => None,
        t => {
            backtrack(&mut lex, t);
            let e = parse_expr(&mut lex);
            match trans!(token(&mut lex)) {
                Token::Eof => Some(e),
                t => Some(Err(format!("Input not consumed: {:?}", t))),
            }
        }
    }
}
