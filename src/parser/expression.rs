use nom::{
    branch::alt,
    bytes::complete::escaped,
    bytes::complete::is_not,
    bytes::complete::tag,
    bytes::complete::take_while_m_n,
    character::complete::{alphanumeric1, digit1, none_of, one_of},
    combinator::{map, verify},
    combinator::{map_res, recognize},
    multi::many0,
    sequence::delimited,
    sequence::{preceded, tuple},
    IResult,
};
use std::num::ParseIntError;
use std::str::FromStr;

use crate::error::AssemblyError::{self};

pub mod matcher;

#[derive(Debug, PartialEq, Clone)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    Comma,
    Greater,
    Less,
    Equal,
    NotEqual,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    DecimalNum(u16),
    ByteNum(u8),
    WordNum(u16),
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
            Err(nom::Err::Incomplete(_)) => Err(AssemblyError::expression("imcomplete imput")),
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
}

fn parse_expr(input: &str) -> IResult<&str, Expr> {
    alt((
        parse_bin_op,
        parse_decimal,
        parse_hex,
        parse_sysop,
        parse_identifier,
        parse_parenthesized,
        parse_string_literal,
    ))(input)
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

fn parse_term(input: &str) -> IResult<&str, Expr> {
    alt((
        parse_decimal,
        parse_hex,
        parse_sysop,
        parse_identifier,
        parse_parenthesized,
        parse_string_literal,
    ))(input)
}

fn parse_bin_op(input: &str) -> IResult<&str, Expr> {
    let (input, (left, op, right)) = tuple((parse_term, parse_operator, parse_expr))(input)?;

    Ok((input, Expr::BinOp(Box::new(left), op, Box::new(right))))
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

fn parse_operator(input: &str) -> IResult<&str, Operator> {
    alt((
        map(tag("+"), |_| Operator::Add),
        map(tag("-"), |_| Operator::Sub),
        map(tag("*"), |_| Operator::Mul),
        map(tag("/"), |_| Operator::Div),
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

fn parse_identifier(input: &str) -> IResult<&str, Expr> {
    map(
        recognize(tuple((
            alt((preceded(tag("."), alphanumeric1), alphanumeric1)),
            many0(alphanumeric1),
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
        one_of("-<>=/+_#!^:;*@?"),
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
