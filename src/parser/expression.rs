use nom::{
    branch::alt,
    bytes::complete::escaped,
    bytes::complete::is_not,
    bytes::complete::tag,
    bytes::complete::take_while_m_n,
    character::complete::{alpha1, alphanumeric1, digit1, none_of, one_of},
    combinator::{map, verify},
    combinator::{map_res, recognize},
    multi::many0,
    sequence::delimited,
    sequence::{preceded, tuple},
    IResult,
};
use std::str::FromStr;
use std::{collections::HashMap, num::ParseIntError};

use crate::{
    assembler::{Address, LabelEntry},
    error::AssemblyError::{self},
};

pub mod matcher;

#[derive(Debug, PartialEq, Clone)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    And,
    Or,
    Xor,
    Comma,
    Greater,
    Less,
    Equal,
    NotEqual,
}

impl Operator {
    pub fn to_string(&self) -> String {
        match self {
            Operator::Add => "+".to_string(),
            Operator::Sub => "-".to_string(),
            Operator::Mul => "*".to_string(),
            Operator::Div => "/".to_string(),
            Operator::And => "&".to_string(),
            Operator::Or => "|".to_string(),
            Operator::Xor => "^".to_string(),
            Operator::Comma => ",".to_string(),
            Operator::Greater => ">".to_string(), // it means '>='
            Operator::Less => "<".to_string(),
            Operator::Equal => "=".to_string(),
            Operator::NotEqual => "\\".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    DecimalNum(u16),
    ByteNum(u8),
    WordNum(u16),
    HiByte(Box<Expr>),
    LoByte(Box<Expr>),
    StringLiteral(String),
    Identifier(String),
    BinOp(Box<Expr>, Operator, Box<Expr>),
    Parenthesized(Box<Expr>),
    Bracketed(Box<Expr>),
    SystemOperator(char),
    Empty,
}

impl Expr {
    pub fn parse(input: &str) -> Result<Expr, AssemblyError> {
        if input.is_empty() {
            return Ok(Expr::Empty);
        }
        let result = parse_expr(input);
        match result {
            Ok((_, expr)) => Ok(expr),
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                Err(AssemblyError::expression(&e.to_string()))
            }
            Err(nom::Err::Incomplete(_)) => Err(AssemblyError::expression("incomplete input")),
        }
    }

    pub fn traverse_comma(self: &Expr) -> Vec<Expr> {
        match self {
            Expr::BinOp(left, Operator::Comma, right) => {
                let mut result = Vec::new();
                result.extend(left.traverse_comma());
                result.extend(right.traverse_comma());
                result
            }
            _ => vec![self.clone()],
        }
    }

    pub fn calculate_address(
        self: &Expr,
        labels: &HashMap<String, LabelEntry>,
    ) -> Result<Address, AssemblyError> {
        match self {
            Expr::Parenthesized(expr) => expr.calculate_address(labels),
            Expr::BinOp(left, op, right) => {
                let left = left.calculate_address(labels)?;
                let right = right.calculate_address(labels)?;
                left.calculate_with(&right, op)
            }
            Expr::Identifier(name) => {
                let label_entry = labels
                    .get(name)
                    .ok_or(AssemblyError::program("label not found"))?;
                Ok(label_entry.address.clone())
            }
            Expr::DecimalNum(n) => Ok(Address::ZeroPage(*n as u8)),
            Expr::ByteNum(n) => Ok(Address::ZeroPage(*n)),
            Expr::WordNum(n) => Ok(Address::Full(*n)),
            _ => Err(AssemblyError::program("invalid label address")),
        }
    }
}

fn parse_expr(input: &str) -> IResult<&str, Expr> {
    alt((
        parse_bin_op,
        parse_decimal,
        parse_hex,
        parse_bin,
        parse_char,
        parse_hibyte,
        parse_lobyte,
        parse_sysop,
        parse_identifier,
        parse_parenthesized,
        parse_bracketed,
        parse_string_literal,
    ))(input)
}

fn parse_term(input: &str) -> IResult<&str, Expr> {
    alt((
        parse_decimal,
        parse_hex,
        parse_bin,
        parse_char,
        parse_hibyte,
        parse_lobyte,
        parse_sysop,
        parse_identifier,
        parse_parenthesized,
        parse_bracketed,
        parse_string_literal,
    ))(input)
}

fn parse_bin_op(input: &str) -> IResult<&str, Expr> {
    let (input, (left, op, right)) = tuple((parse_term, parse_operator, parse_expr))(input)?;

    Ok((input, Expr::BinOp(Box::new(left), op, Box::new(right))))
}

fn parse_operator(input: &str) -> IResult<&str, Operator> {
    alt((
        map(tag("+"), |_| Operator::Add),
        map(tag("-"), |_| Operator::Sub),
        map(tag("*"), |_| Operator::Mul),
        map(tag("/"), |_| Operator::Div),
        map(tag("&"), |_| Operator::And),
        map(tag("|"), |_| Operator::Or),
        map(tag("^"), |_| Operator::Xor),
        map(tag(","), |_| Operator::Comma),
        map(tag(">"), |_| Operator::Greater),
        map(tag("<"), |_| Operator::Less),
        map(tag("="), |_| Operator::Equal),
        map(tag("\\"), |_| Operator::NotEqual),
    ))(input)
}

fn parse_decimal(input: &str) -> IResult<&str, Expr> {
    map_res(
        verify(
            map_res(digit1, |digit_str: &str| u32::from_str(digit_str)),
            |num: &u32| *num <= u16::MAX as u32,
        ),
        |num: u32| -> Result<Expr, ParseIntError> { Ok(Expr::DecimalNum(num as u16)) },
    )(input)
}

fn is_hex_digit(c: char) -> bool {
    c.is_digit(16)
}

fn parse_hex(input: &str) -> IResult<&str, Expr> {
    let (input, hex_str) = preceded(tag("$"), take_while_m_n(1, 4, is_hex_digit))(input)?;
    let num = u32::from_str_radix(hex_str, 16).map_err(|_| {
        nom::Err::Failure(nom::error::Error::new(input, nom::error::ErrorKind::Char))
    })?;
    if hex_str.len() <= 2 {
        Ok((input, Expr::ByteNum(num as u8)))
    } else {
        Ok((input, Expr::WordNum(num as u16)))
    }
}

fn is_bin_digit(c: char) -> bool {
    c.is_digit(2)
}

fn parse_bin(input: &str) -> IResult<&str, Expr> {
    let (input, bin_str) = preceded(tag("%"), take_while_m_n(1, 16, is_bin_digit))(input)?;
    let num = u32::from_str_radix(bin_str, 2).map_err(|_| {
        nom::Err::Failure(nom::error::Error::new(input, nom::error::ErrorKind::Char))
    })?;
    if bin_str.len() <= 8 {
        Ok((input, Expr::ByteNum(num as u8)))
    } else {
        Ok((input, Expr::WordNum(num as u16)))
    }
}

// 'c' => 0x63 (ASCII code)
fn parse_char(input: &str) -> IResult<&str, Expr> {
    let (input, c) = preceded(tag("'"), none_of("'"))(input)?;
    let (input, _) = tag("'")(input)?;
    Ok((input, Expr::ByteNum(c as u8)))
}

// #>label  == MSB == Hi-Byte
fn parse_hibyte(input: &str) -> IResult<&str, Expr> {
    map_res(
        preceded(tag(">"), parse_identifier),
        |expr: Expr| -> Result<Expr, ParseIntError> { Ok(Expr::HiByte(Box::new(expr))) },
    )(input)
}

// #<label  == LSB == Lo-Byte
pub fn parse_lobyte(input: &str) -> IResult<&str, Expr> {
    map_res(
        preceded(tag("<"), parse_identifier),
        |expr: Expr| -> Result<Expr, ParseIntError> { Ok(Expr::LoByte(Box::new(expr))) },
    )(input)
}

fn parse_identifier(input: &str) -> IResult<&str, Expr> {
    map(
        recognize(tuple((
            alt((alpha1, tag("_"), tag("."))),
            many0(alt((alphanumeric1, tag("_")))),
        ))),
        |id_str: &str| Expr::Identifier(id_str.to_string()),
    )(input)
}

fn parse_parenthesized(input: &str) -> IResult<&str, Expr> {
    delimited(tag("("), parse_expr, tag(")"))(input)
        .map(|(remaining_input, expr)| (remaining_input, Expr::Parenthesized(Box::new(expr))))
}

fn parse_bracketed(input: &str) -> IResult<&str, Expr> {
    delimited(tag("["), parse_expr, tag("]"))(input)
        .map(|(remaining_input, expr)| (remaining_input, Expr::Bracketed(Box::new(expr))))
}

fn parse_sysop(input: &str) -> IResult<&str, Expr> {
    map_res(
        one_of("-<>=/+_#\\!^:;*@?"),
        |c: char| -> Result<Expr, ParseIntError> { Ok(Expr::SystemOperator(c)) },
    )(input)
}

fn parse_escaped_string(input: &str) -> IResult<&str, &str> {
    recognize(many0(alt((
        alphanumeric1,
        is_not("\\\""),
        escaped(none_of("\\\""), '\\', one_of("\\\"")),
    ))))(input)
}

fn parse_string_literal(input: &str) -> IResult<&str, Expr> {
    map_res(
        delimited(
            tag("\""),
            map(parse_escaped_string, |s| s.to_string()),
            tag("\""),
        ),
        |s: String| -> Result<Expr, ParseIntError> { Ok(Expr::StringLiteral(s)) },
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_identifier_with_single_letter() {
        assert_eq!(
            parse_identifier("A"),
            Ok(("", Expr::Identifier("A".to_string())))
        );
    }

    #[test]
    fn test_parse_identifier_with_number() {
        assert_eq!(
            parse_identifier("A1"),
            Ok(("", Expr::Identifier("A1".to_string())))
        );
    }

    #[test]
    fn test_parse_identifier_with_underscore() {
        assert_eq!(
            parse_identifier("A_1"),
            Ok(("", Expr::Identifier("A_1".to_string())))
        );
    }

    #[test]
    fn test_parse_identifier_with_dot_and_num() {
        assert_eq!(
            parse_identifier(".1"),
            Ok(("", Expr::Identifier(".1".to_string())))
        );
    }

    #[test]
    fn test_parse_identifier_with_dot() {
        assert_eq!(
            parse_identifier(".A1"),
            Ok(("", Expr::Identifier(".A1".to_string())))
        );
    }

    #[test]
    fn test_parse_identifier_with_unexpected_character() {
        assert_eq!(
            parse_identifier("A-1"),
            Ok(("-1", Expr::Identifier("A".to_string())))
        );
    }

    #[test]
    fn test_parse_binop() {
        assert_eq!(
            parse_bin_op("=,.skip"),
            Ok((
                "",
                Expr::BinOp(
                    Box::new(Expr::SystemOperator('=')),
                    Operator::Comma,
                    Box::new(Expr::Identifier(".skip".to_string()))
                )
            ))
        );
        let expression = "1,2,3,4";
        assert_eq!(
            parse_bin_op(expression),
            Ok((
                "",
                Expr::BinOp(
                    Box::new(Expr::DecimalNum(1)),
                    Operator::Comma,
                    Box::new(Expr::BinOp(
                        Box::new(Expr::DecimalNum(2)),
                        Operator::Comma,
                        Box::new(Expr::BinOp(
                            Box::new(Expr::DecimalNum(3)),
                            Operator::Comma,
                            Box::new(Expr::DecimalNum(4))
                        ))
                    ))
                )
            ))
        );
    }

    #[test]
    fn test_parse_expr() {
        assert_eq!(
            parse_expr("A>$A"),
            Ok((
                "",
                Expr::BinOp(
                    Box::new(Expr::Identifier("A".to_string())),
                    Operator::Greater,
                    Box::new(Expr::ByteNum(10))
                )
            ))
        );
        assert_eq!(
            parse_expr(".skip"),
            Ok(("", Expr::Identifier(".skip".to_string())))
        );
    }
}
