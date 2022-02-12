use crate::{Expr, OpName};
use std::collections::HashMap;

pub type OpMap = HashMap<OpName, OpInfo>;

pub enum Order {
    Above(OpName),
    Below(OpName),
    Equal(OpName),
}

#[derive(Eq, PartialEq)]
pub enum OpAssoc {
    Left,
    Right,
    Non,
}

#[derive(Eq, PartialEq)]
pub enum OpKind {
    Prefix,
    Postfix,
    Infix(OpAssoc),
}

pub struct OpInfo {
    name: OpName,
    kind: OpKind,
    relation: Vec<Order>,
    prec: Option<usize>,
}

impl OpInfo {
    pub fn new(name: &str, kind: OpKind, relation: Vec<Order>) -> Self {
        Self {
            name: name.to_owned(),
            kind,
            relation,
            prec: None,
        }
    }
}

const DEFAULT_PREC: usize = 1;
const MAX_PREC: usize = 999999;

// names -> abs_precs
// abs_precs -> tree
// tree -> topsort
// topsort -> precs

fn calculate_order(op: &OpInfo, map: &HashMap<OpName, usize>) {}

pub fn calculate_poset(opers: &[OpInfo]) -> Result<OpMap, String> {
    let mut map = HashMap::new();
    for op in opers {
        match op.kind {
            OpKind::Infix(_) => {
                calculate_order(&op, &map);
            }
            _ => {}
        }
    }

    Err(String::from("arg"))
}

pub fn fixup(e: Expr, map: &OpMap) -> Result<Expr, String> {
    Ok(match e {
        Expr::InfixOp(op, e1, e2) => {
            let e1 = fixup(*e1, map)?;
            let e2 = fixup(*e2, map)?;
            match e1 {
                Expr::InfixOp(op2, e3, e4) => {
                    let info1 = map.get(&op).ok_or(format!("Unbound operator {}", op))?;
                    let info2 = map.get(&op2).ok_or(format!("Unbound operator {}", op2))?;

                    let assoc1 = match &info1.kind {
                        OpKind::Infix(assoc) => assoc,
                        _ => return Err(format!("Operator {} used in infix position", op)),
                    };

                    let assoc2 = match &info2.kind {
                        OpKind::Infix(assoc) => assoc,
                        _ => return Err(format!("Operator {} used in infix position", op2)),
                    };

                    let prec1 = info1.prec.unwrap();
                    let prec2 = info2.prec.unwrap();

                    if prec1 == prec2 && assoc1 != assoc2 {
                        return Err(format!("Infix is not resolvable {} {}", op, op2));
                    } else if prec1 < prec2 || (prec1 == prec2 && *assoc1 == OpAssoc::Right) {
                        Expr::InfixOp(
                            op2,
                            e3,
                            Box::new(fixup(Expr::InfixOp(op, e4, Box::new(e2)), map)?),
                        )
                    } else {
                        Expr::InfixOp(
                            op.to_string(),
                            Box::new(Expr::InfixOp(op2, e3, e4)),
                            Box::new(e2),
                        )
                    }
                }
                _ => Expr::InfixOp(op.to_string(), Box::new(e1), Box::new(e2)),
            }
        }
        Expr::PrefixOp(op, e) => {
            let e = fixup(*e, map)?;

            let info = map.get(&op).ok_or(format!("Unbound operator {}", op))?;

            match &info.kind {
                OpKind::Prefix => {}
                _ => return Err(format!("Operator {} used in prefix position", op)),
            };

            Expr::PrefixOp(op, Box::new(e))
        }
        Expr::PostfixOp(op, e) => {
            let e = fixup(*e, map)?;

            let info = map.get(&op).ok_or(format!("Unbound operator {}", op))?;

            match &info.kind {
                OpKind::Postfix => {}
                _ => return Err(format!("Operator {} used in postfix position", op)),
            };

            Expr::PostfixOp(op, Box::new(e))
        }
        _ => e,
    })
}
