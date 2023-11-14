use crate::error::ParseError;
use regex::Regex;

#[derive(Debug)]
pub enum Operand {
    WordData(u16),          // $12fd
    ByteData(u8),           // $3f
    Identifire(String),     // CONST_LABEL global_label .local_label
    SystemOperator(String), // # > < - ! ...
    StringLiteral(String),  // "hello world!"
    Register(String),       // A X P ...
}

enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    Comma,
}

impl Operand {
    pub fn parse(s: &str) -> Result<Operand, ParseError> {
        Operand::parse_absolute(s)
            .or_else(|_| Operand::parse_absolute_x(s))
            .or_else(|_| Operand::parse_operator(s))
            .or_else(|_| Operand::parse_string_literal(s))
            .or_else(|_| Operand::parse_immediate(s))
    }

    fn parse_absolute(s: &str) -> Result<Operand, ParseError> {
        let digits_or_label = Self::match_absolute(s)?;
        if is_valid_identifier(&digits_or_label) {
            return Ok(Operand::Absolute(Address::Label(
                digits_or_label.to_string(),
            )));
        }
        let n = parse_u16(&digits_or_label)?;
        Ok(Operand::Absolute(Address::Digits(n)))
    }

    fn parse_absolute_x(s: &str) -> Result<Operand, ParseError> {
        let digits_or_label = Self::match_absolute_x(s)?;
        if is_valid_identifier(&digits_or_label) {
            return Ok(Operand::AbsoluteX(Address::Label(
                digits_or_label.to_string(),
            )));
        }
        let n = parse_u16(&digits_or_label)?;
        Ok(Operand::AbsoluteX(Address::Digits(n)))
    }

    fn parse_operator(s: &str) -> Result<Operand, ParseError> {
        if is_system_operator(s) {
            Ok(Operand::SystemOperator(s.to_string()))
        } else {
            Err(ParseError::operand(s))
        }
    }

    fn parse_string_literal(s: &str) -> Result<Operand, ParseError> {
        if is_string_literal(s) {
            Ok(Operand::StringLiteral(s[1..s.len() - 1].to_string()))
        } else {
            Err(ParseError::operand(s))
        }
    }

    fn parse_immediate(s: &str) -> Result<Operand, ParseError> {
        let digits_or_label = Self::match_immediate(s)?;
        if is_valid_identifier(&digits_or_label) {
            return Ok(Operand::Immediate(Address::Label(
                digits_or_label.to_string(),
            )));
        }
        let n = parse_u16(&digits_or_label)?;
        Ok(Operand::Immediate(Address::Digits(n)))
    }

    fn match_absolute(operand_text: &str) -> Result<String, ParseError> {
        // label or '12345' or '$ffd2'
        let re = Regex::new(r"^(\$[0-9a-fA-F]{4}|\d+|[a-zA-Z_][a-zA-Z0-9_]+)$").unwrap();
        Self::match_regexp(operand_text, re)
    }

    fn match_absolute_x(operand_text: &str) -> Result<String, ParseError> {
        // label or '12345' or '$ffd2'
        let re = Regex::new(r"^(\$[0-9a-fA-F]{4}|\d+|[a-zA-Z_][a-zA-Z0-9_]+)\+[xX]$").unwrap();
        Self::match_regexp(operand_text, re)
    }

    fn match_immediate(operand_text: &str) -> Result<String, ParseError> {
        // '#label' or '#12345' or '#$ffd2'
        let re = Regex::new(r"^#(\$[0-9a-fA-F]{4}|\d+|[a-zA-Z_][a-zA-Z0-9_]+)$").unwrap();
        Self::match_regexp(operand_text, re)
    }

    fn match_regexp(operand_text: &str, re: Regex) -> Result<String, ParseError> {
        let cap = re
            .captures(operand_text)
            .ok_or(ParseError::operand(operand_text))?;
        Ok(cap[1].to_string())
    }
}

fn is_valid_identifier(s: &str) -> bool {
    let re = Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$").unwrap();
    re.is_match(s)
}

fn is_system_operator(s: &str) -> bool {
    let re = Regex::new(r"^[-<>=/+_#!^]$").unwrap();
    re.is_match(s)
}

fn is_string_literal(s: &str) -> bool {
    let re = Regex::new(r#"^"([^"]*)"$"#).unwrap();
    re.is_match(s)
}

fn parse_u16(digits: &str) -> Result<u16, ParseError> {
    if digits.starts_with("$") {
        u16::from_str_radix(&digits[2..], 16)
    } else {
        digits.parse()
    }
    .map_err(|_| ParseError::operand(digits))
}
