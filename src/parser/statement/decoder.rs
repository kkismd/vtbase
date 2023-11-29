use std::collections::HashMap;

use crate::{
    assembler::{Address, LabelEntry},
    error::AssemblyError,
    parser::expression::matcher::*,
    parser::expression::{matcher::parenthesized, Expr, Operator},
};

type Decoder<T> = fn(&Expr) -> Result<T, AssemblyError>;

pub fn parenthesized_within<T>(expr: &Expr, decoder: Decoder<T>) -> Result<T, AssemblyError> {
    parenthesized(&expr).and_then(|expr| decoder(&expr))
}

pub fn bracketed_within<T>(expr: &Expr, decoder: Decoder<T>) -> Result<T, AssemblyError> {
    bracketed(&expr).and_then(|expr| decoder(&expr))
}

fn error<T>() -> Result<T, AssemblyError> {
    Err(AssemblyError::DecodeError)
}

/**
 * A=1 or A=$10 or A=label
 */
pub fn immediate(expr: &Expr, labels: &HashMap<String, LabelEntry>) -> Result<u8, AssemblyError> {
    num8bit(expr)
        .and_then(|num| Ok(num))
        .or_else(|_| zeropage_label(expr, labels))
}

fn lookup(name: &str, labels: &HashMap<String, LabelEntry>) -> Result<LabelEntry, AssemblyError> {
    labels
        .get(name)
        .cloned()
        .ok_or(AssemblyError::label_not_found(name))
}

/**
 * A=($1F) or A=(31) or A=(label)
 */
pub fn zeropage(expr: &Expr, labels: &HashMap<String, LabelEntry>) -> Result<u8, AssemblyError> {
    parenthesized_within::<u8>(expr, num8bit)
        // A=($1F) or A=(31)
        .and_then(|num| Ok(num))
        .or_else(|_| parenthesized(expr).and_then(|expr| zeropage_label(&expr, labels)))
}

fn zeropage_label(expr: &Expr, labels: &HashMap<String, LabelEntry>) -> Result<u8, AssemblyError> {
    identifier(expr).and_then(|name| {
        lookup(&name, labels).and_then(|entry| match entry.address {
            Address::ZeroPage(addr) => Ok(addr),
            _ => error(),
        })
    })
}

pub fn absolute(expr: &Expr, labels: &HashMap<String, LabelEntry>) -> Result<u16, AssemblyError> {
    parenthesized_within::<u16>(expr, num16bit)
        // A=($1F) or A=(31)
        .and_then(|num| Ok(num))
        .or_else(|_| parenthesized(expr).and_then(|expr| absolute_label(&expr, labels)))
}

fn absolute_label(expr: &Expr, labels: &HashMap<String, LabelEntry>) -> Result<u16, AssemblyError> {
    identifier(expr).and_then(|name| {
        lookup(&name, labels)
            .and_then(|entry| match entry.address {
                Address::Full(addr) => Ok(addr),
                _ => error(),
            })
            .or_else(|_| Ok(0 as u16))
    })
}

/**
 * X=($1F+Y) or X=(31+Y) or X=(label+Y)
 */
pub fn zeropage_y(expr: &Expr, labels: &HashMap<String, LabelEntry>) -> Result<u8, AssemblyError> {
    parenthesized_within(expr, plus).and_then(|(left, right)| {
        register_y(&right)
            .and_then(|_| {
                // X=($1F+Y) or X=(31+Y)
                num8bit(&left).and_then(|num| Ok(num))
            })
            .or_else(|_|
                    // X=(label+Y)
                    zeropage_label(&left, labels).and_then(|addr| Ok(addr)))
    })
}

pub fn zeropage_x(expr: &Expr, labels: &HashMap<String, LabelEntry>) -> Result<u8, AssemblyError> {
    parenthesized_within(expr, plus).and_then(|(left, right)| {
        register_x(&right)
            .and_then(|_| {
                // X=($1F+X) or X=(31+X)
                num8bit(&left).and_then(|num| Ok(num))
            })
            .or_else(|_|
                    // X=(label+X)
                    zeropage_label(&left, labels).and_then(|addr| Ok(addr)))
    })
}

pub fn absolute_y(expr: &Expr, labels: &HashMap<String, LabelEntry>) -> Result<u16, AssemblyError> {
    parenthesized_within(expr, plus).and_then(|(left, right)| {
        register_y(&right)
            .and_then(|_| {
                // X=($12FF+Y) or X=(311+Y)
                num16bit(&left).and_then(|num| Ok(num))
            })
            .or_else(|_|
                    // X=(label+Y)
                    absolute_label(&left, labels).and_then(|addr| Ok(addr)))
    })
}

#[test]
fn test_absolute_y() {
    let mut labels = HashMap::new();
    labels.insert(
        "label".to_string(),
        LabelEntry {
            name: "label".to_string(),
            address: Address::Full(0x1234),
            line: 0,
        },
    );
    let expr = Expr::Parenthesized(Box::new(Expr::BinOp(
        Box::new(Expr::Identifier("label".to_string())),
        Operator::Add,
        Box::new(Expr::Identifier("Y".to_string())),
    )));
    assert_eq!(absolute_y(&expr, &labels), Ok(0x1234));
}

pub fn absolute_x(expr: &Expr, labels: &HashMap<String, LabelEntry>) -> Result<u16, AssemblyError> {
    parenthesized_within(expr, plus).and_then(|(left, right)| {
        register_x(&right)
            .and_then(|_| {
                // X=($12FF+X) or X=(311+X)
                num16bit(&left).and_then(|num| Ok(num))
            })
            .or_else(|_|
                    // X=(label+X)
                    absolute_label(&left, labels).and_then(|addr| Ok(addr)))
    })
}

#[test]
fn test_absolute_x() {
    let mut labels = HashMap::new();
    labels.insert(
        "label".to_string(),
        LabelEntry {
            name: "label".to_string(),
            address: Address::Full(0x1234),
            line: 0,
        },
    );
    // Parenthesized(BinOp(Identifier(\"hello\"), Add, Identifier(\"X\")))
    let expr = Expr::Parenthesized(Box::new(Expr::BinOp(
        Box::new(Expr::Identifier("label".to_string())),
        Operator::Add,
        Box::new(Expr::Identifier("X".to_string())),
    )));
    assert_eq!(absolute_x(&expr, &labels), Ok(0x1234));
}

// Indirect,X    LDA ($44,X)   A=[$44+X]
pub fn indirect_x(expr: &Expr, labels: &HashMap<String, LabelEntry>) -> Result<u8, AssemblyError> {
    bracketed_within(expr, plus).and_then(|(left, right)| {
        register_x(&right)
            .and_then(|_| {
                // A=[$1F+X] or A=[31+X]
                num8bit(&left).and_then(|num| Ok(num))
            })
            .or_else(|_|
                    // A=[label+X]
                    zeropage_label(&left, labels).and_then(|addr| Ok(addr)))
    })
}

// Indirect,Y    LDA ($44),Y   A=[$44]+Y, A=[44]+Y, A=[label]+Y
pub fn indirect_y(expr: &Expr, labels: &HashMap<String, LabelEntry>) -> Result<u8, AssemblyError> {
    plus(expr).and_then(|(ref left, ref right)| {
        // A=[$1F]+Y or A=[31]+Y
        register_y(right).and_then(|_| {
            bracketed(left)
                .and_then(|num| {
                    // A=[$1F]+Y
                    num8bit(&num).and_then(|addr| Ok(addr))
                })
                .or_else(|_| {
                    // A=[label]+Y
                    bracketed(left).and_then(|label| zeropage_label(&label, labels))
                })
        })
    })
}
#[test]
fn test_indirect_y() {
    let mut labels = HashMap::new();
    labels.insert(
        "label".to_string(),
        LabelEntry {
            name: "label".to_string(),
            address: Address::ZeroPage(0x12),
            line: 0,
        },
    );
    let expr = Expr::BinOp(
        Box::new(Expr::Bracketed(Box::new(Expr::Identifier(
            "label".to_string(),
        )))),
        Operator::Add,
        Box::new(Expr::Identifier("Y".to_string())),
    );
    assert_eq!(indirect_y(&expr, &labels), Ok(0x12));
}

/**
 * X=X+1 or X=+
 */
pub fn incr_decrement(expr: &Expr, register_left: &str) -> Result<Operator, AssemblyError> {
    incr_decr_long(expr, register_left).or_else(|_| incr_decr_short(expr))
}

pub fn increment(expr: &Expr, register_left: &str) -> Result<(), AssemblyError> {
    incr_decrement(expr, register_left).and_then(|operator| match operator {
        Operator::Add => Ok(()),
        _ => error(),
    })
}

pub fn decrement(expr: &Expr, register_left: &str) -> Result<(), AssemblyError> {
    incr_decrement(expr, register_left).and_then(|operator| match operator {
        Operator::Sub => Ok(()),
        _ => error(),
    })
}

/**
 * X=X+1
 */
fn incr_decr_long(expr: &Expr, register_left: &str) -> Result<Operator, AssemblyError> {
    binop(expr).and_then(|(left, operator, right)| {
        // X=X+1 or Y=Y+1
        identifier(&left).and_then(|register_right| {
            decimal(&right).and_then(|num| {
                if num == 1 && register_left == register_right {
                    match operator {
                        Operator::Add | Operator::Sub => Ok(operator),
                        _ => error(),
                    }
                } else {
                    error()
                }
            })
        })
    })
}

/**
 * X=+
 */
fn incr_decr_short(expr: &Expr) -> Result<Operator, AssemblyError> {
    sysop(expr).and_then(|symbol| {
        if symbol == '+' {
            Ok(Operator::Add)
        } else if symbol == '-' {
            Ok(Operator::Sub)
        } else {
            error()
        }
    })
}
