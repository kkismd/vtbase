use nom::{
    branch::alt,
    bytes::complete::escaped,
    bytes::complete::is_not,
    bytes::complete::tag,
    character::complete::hex_digit1,
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

use crate::{
    assembler::Assembler,
    error::AssemblyError::{self, ParseError},
};

#[derive(Debug)]
enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    Comma,
}

#[derive(Debug)]
pub enum Expr {
    DecimalNum(u16),
    ByteNum(u8),
    WordNum(u16),
    Immediate(Box<Expr>),
    StringLiteral(String),
    Identifier(String),
    BinOp(Box<Expr>, Operator, Box<Expr>),
    Parenthesized(Box<Expr>),
    SystemOperator(char),
}

impl Expr {
    pub fn parse(input: &str) -> Result<Expr, AssemblyError> {
        let result = parse_expr(input);
        match result {
            Ok((_, expr)) => Ok(expr),
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                Err(AssemblyError::expression(&e.to_string()))
            }
            Err(nom::Err::Incomplete(_)) => Err(AssemblyError::expression("imcomplete imput")),
        }
    }
}

fn parse_expr(input: &str) -> IResult<&str, Expr> {
    alt((
        parse_bin_op,
        parse_decimal,
        parse_word,
        parse_byte,
        parse_immediate,
        parse_sysop,
        parse_identifier,
        parse_parenthesized,
        parse_string_literal,
    ))(input)
}

fn parse_term(input: &str) -> IResult<&str, Expr> {
    alt((
        parse_decimal,
        parse_word,
        parse_byte,
        parse_immediate,
        parse_sysop,
        parse_identifier,
        parse_parenthesized,
        parse_string_literal,
    ))(input)
}

fn parse_operator(input: &str) -> IResult<&str, Operator> {
    alt((
        map(tag("+"), |_| Operator::Add),
        map(tag("-"), |_| Operator::Sub),
        map(tag("*"), |_| Operator::Mul),
        map(tag("/"), |_| Operator::Div),
        map(tag(","), |_| Operator::Comma),
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

fn parse_hex(input: &str) -> IResult<&str, u32> {
    map_res(hex_digit1, |hex_str: &str| u32::from_str_radix(hex_str, 16))(input)
}

fn parse_byte(input: &str) -> IResult<&str, Expr> {
    map_res(
        preceded(
            tag("$"),
            verify(parse_hex, |num: &u32| *num <= u8::MAX as u32),
        ),
        |num: u32| -> Result<Expr, ParseIntError> { Ok(Expr::ByteNum(num as u8)) },
    )(input)
}

fn parse_word(input: &str) -> IResult<&str, Expr> {
    map_res(
        preceded(
            tag("$"),
            verify(parse_hex, |num: &u32| *num <= u16::MAX as u32),
        ),
        |num: u32| -> Result<Expr, ParseIntError> { Ok(Expr::WordNum(num as u16)) },
    )(input)
}

fn parse_immediate(input: &str) -> IResult<&str, Expr> {
    map_res(
        preceded(
            tag("#"),
            nom::branch::alt((parse_word, parse_byte, parse_decimal)),
        ),
        |expr: Expr| -> Result<Expr, ParseIntError> { Ok(Expr::Immediate(Box::new(expr))) },
    )(input)
}

fn parse_identifier(input: &str) -> IResult<&str, Expr> {
    map(alphanumeric1, |id_str: &str| {
        Expr::Identifier(id_str.to_string())
    })(input)
}

fn parse_bin_op(input: &str) -> IResult<&str, Expr> {
    let (input, (left, op, right)) = tuple((parse_term, parse_operator, parse_expr))(input)?;

    Ok((input, Expr::BinOp(Box::new(left), op, Box::new(right))))
}

fn parse_parenthesized(input: &str) -> IResult<&str, Expr> {
    delimited(tag("("), parse_expr, tag(")"))(input)
        .map(|(remaining_input, expr)| (remaining_input, Expr::Parenthesized(Box::new(expr))))
}

fn parse_sysop(input: &str) -> IResult<&str, Expr> {
    map_res(
        one_of("-<>=/+_#!^"),
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
