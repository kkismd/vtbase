use crate::error::AssemblyError;

use super::{Expr, Operator};

pub fn byte(expr: &Expr) -> Result<u8, AssemblyError> {
    match expr {
        Expr::ByteNum(num) => Ok(*num),
        _ => syntax_error("invalid byte"),
    }
}

pub fn word(expr: &Expr) -> Result<u16, AssemblyError> {
    match expr {
        Expr::WordNum(num) => Ok(*num),
        _ => syntax_error("invalid word"),
    }
}

pub fn decimal(expr: &Expr) -> Result<u16, AssemblyError> {
    match expr {
        Expr::DecimalNum(num) => Ok(*num),
        _ => syntax_error("invalid decimal"),
    }
}

pub fn num8bit(expr: &Expr) -> Result<u8, AssemblyError> {
    byte(expr).or_else(|_| decimal8bit(expr))
}

pub fn num16bit(expr: &Expr) -> Result<u16, AssemblyError> {
    word(expr).or_else(|_| decimal(expr))
}

fn decimal8bit(expr: &Expr) -> Result<u8, AssemblyError> {
    decimal(&expr).and_then(|num| {
        if num > 255 {
            Err(AssemblyError::syntax("operand must be 8bit"))
        } else {
            Ok(num as u8)
        }
    })
}

pub fn string(expr: &Expr) -> Result<String, AssemblyError> {
    match expr {
        Expr::StringLiteral(s) => Ok(s.to_string()),
        _ => syntax_error("invalid string"),
    }
}

pub fn identifier(expr: &Expr) -> Result<String, AssemblyError> {
    match expr {
        Expr::Identifier(s) => Ok(s.to_string()),
        _ => syntax_error("invalid identifier"),
    }
}

pub fn register_y(expr: &Expr) -> Result<(), AssemblyError> {
    match expr {
        Expr::Identifier(s) if s == "Y" => Ok(()),
        _ => syntax_error("invalid register Y"),
    }
}

pub fn register_x(expr: &Expr) -> Result<(), AssemblyError> {
    match expr {
        Expr::Identifier(s) if s == "X" => Ok(()),
        _ => syntax_error("invalid register X"),
    }
}

pub fn register_a(expr: &Expr) -> Result<(), AssemblyError> {
    match expr {
        Expr::Identifier(s) if s == "A" => Ok(()),
        _ => syntax_error("invalid register A"),
    }
}

pub fn binop(expr: &Expr) -> Result<(Expr, Operator, Expr), AssemblyError> {
    match expr {
        Expr::BinOp(left, op, right) => ok3(left, op, right),
        _ => syntax_error("invalid binop"),
    }
}

pub fn hi(expr: &Expr) -> Result<Expr, AssemblyError> {
    match expr {
        Expr::HiByte(expr) => Ok(*expr.clone()),
        _ => syntax_error("invalid hi"),
    }
}

pub fn lo(expr: &Expr) -> Result<Expr, AssemblyError> {
    match expr {
        Expr::LoByte(expr) => Ok(*expr.clone()),
        _ => syntax_error("invalid lo"),
    }
}

fn ok2(a: &Box<Expr>, c: &Box<Expr>) -> Result<(Expr, Expr), AssemblyError> {
    Ok((*a.clone(), *c.clone()))
}
fn ok3(
    a: &Box<Expr>,
    b: &Operator,
    c: &Box<Expr>,
) -> Result<(Expr, Operator, Expr), AssemblyError> {
    Ok((*a.clone(), b.clone(), *c.clone()))
}

pub fn comma(expr: &Expr) -> Result<(Expr, Expr), AssemblyError> {
    match expr {
        Expr::BinOp(left, Operator::Comma, right) => ok2(left, right),
        _ => syntax_error("invalid comma"),
    }
}

pub fn plus(expr: &Expr) -> Result<(Expr, Expr), AssemblyError> {
    match expr {
        Expr::BinOp(left, Operator::Add, right) => ok2(left, right),
        _ => syntax_error("invalid plus"),
    }
}

pub fn minus(expr: &Expr) -> Result<(Expr, Expr), AssemblyError> {
    match expr {
        Expr::BinOp(left, Operator::Sub, right) => ok2(left, right),
        _ => syntax_error("invalid minus"),
    }
}

pub fn multiply(expr: &Expr) -> Result<(Expr, Expr), AssemblyError> {
    match expr {
        Expr::BinOp(left, Operator::Mul, right) => ok2(left, right),
        _ => syntax_error("invalid multiply"),
    }
}

pub fn divide(expr: &Expr) -> Result<(Expr, Expr), AssemblyError> {
    match expr {
        Expr::BinOp(left, Operator::Div, right) => ok2(left, right),
        _ => syntax_error("invalid divide"),
    }
}

pub fn equal(expr: &Expr) -> Result<(Expr, Expr), AssemblyError> {
    match expr {
        Expr::BinOp(left, Operator::Equal, right) => ok2(left, right),
        _ => syntax_error("invalid equal"),
    }
}

pub fn not_equal(expr: &Expr) -> Result<(Expr, Expr), AssemblyError> {
    match expr {
        Expr::BinOp(left, Operator::NotEqual, right) => ok2(left, right),
        _ => syntax_error("invalid not equal"),
    }
}

pub fn greater(expr: &Expr) -> Result<(Expr, Expr), AssemblyError> {
    match expr {
        Expr::BinOp(left, Operator::Greater, right) => ok2(left, right),
        _ => syntax_error("invalid greater"),
    }
}

pub fn less(expr: &Expr) -> Result<(Expr, Expr), AssemblyError> {
    match expr {
        Expr::BinOp(left, Operator::Less, right) => ok2(left, right),
        _ => syntax_error("invalid less"),
    }
}

pub fn parenthesized(expr: &Expr) -> Result<Expr, AssemblyError> {
    match expr {
        Expr::Parenthesized(expr) => Ok(*expr.clone()),
        _ => syntax_error("invalid parenthesized"),
    }
}

pub fn bracketed(expr: &Expr) -> Result<Expr, AssemblyError> {
    match expr {
        Expr::Bracketed(expr) => Ok(*expr.clone()),
        _ => syntax_error("invalid bracketed"),
    }
}

pub fn sysop(expr: &Expr) -> Result<char, AssemblyError> {
    match expr {
        Expr::SystemOperator(c) => Ok(*c),
        _ => syntax_error("invalid system operator"),
    }
}

pub fn empty(expr: &Expr) -> Result<(), AssemblyError> {
    match expr {
        Expr::Empty => Ok(()),
        _ => syntax_error("invalid empty"),
    }
}

fn syntax_error<T>(msg: &str) -> Result<T, AssemblyError> {
    Err(AssemblyError::syntax(msg))
}
