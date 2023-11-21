use crate::error::AssemblyError;
use regex::Captures;
use regex::Regex;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::vec;
pub mod expression;
use expression::Expr;

pub mod statement;
use statement::Statement;

// line of source code
#[derive(Debug, Clone)]
pub struct Instruction {
    pub line_number: usize,
    pub address: u16,
    pub label: Option<String>,
    pub statements: Vec<Statement>,
    pub object_codes: Vec<u8>,
}

impl Instruction {
    pub fn new(
        line_number: usize,
        address: u16,
        label: Option<String>,
        statements: Vec<Statement>,
        object_codes: Vec<u8>,
    ) -> Self {
        Self {
            line_number,
            address,
            label,
            statements,
            object_codes,
        }
    }

    pub fn new_label(&self, label: &str) -> Self {
        Self {
            line_number: self.line_number,
            address: self.address,
            label: Some(label.to_string()),
            statements: vec![],
            object_codes: vec![],
        }
    }
}

// make abstract syntax tree from input file
pub fn parse_from_file(file: &File) -> Result<Vec<Instruction>, AssemblyError> {
    let reader = BufReader::new(file);
    let mut instructions = Vec::new();

    for (num, line) in reader.lines().enumerate() {
        let res = line;
        if let Ok(line) = res {
            let instruction = parse_line(line, num + 1)?;
            instructions.push(instruction);
        }
    }
    Ok(instructions)
}
// make abstract syntax tree
fn parse_line(line: String, line_num: usize) -> Result<Instruction, AssemblyError> {
    let line = remove_after_quote(&line);
    let cap = match_line(&line, line_num)?;

    let body = cap.name("body").map_or("", |m| m.as_str());
    let tokens = tokenize(body);
    Ok(Instruction::new(
        line_num,
        0,
        cap.name("label").map(|m| m.as_str()).map(String::from),
        parse_statements(tokens).map_err(|e| AssemblyError::line(line_num, &e.message()))?,
        Vec::new(),
    ))
}

// source line format
fn match_line(line: &str, line_num: usize) -> Result<Captures, AssemblyError> {
    let re = Regex::new(r"^(?<label>[.a-zA-Z][a-zA-Z0-9_]*)?(?<body>\s+.*)?").unwrap();
    re.captures(&line)
        .ok_or(AssemblyError::line(line_num, &line))
}

fn parse_statements(tokens: Vec<String>) -> Result<Vec<Statement>, AssemblyError> {
    let mut statements = Vec::new();
    for token in tokens {
        let statement = parse_token(&token)?;
        statements.push(statement);
    }
    Ok(statements)
}

fn parse_token(token: &str) -> Result<Statement, AssemblyError> {
    let re = Regex::new(r"^(?P<command>\S)=(?P<operand>.+)$")
        .map_err(|_| AssemblyError::token(token))?;
    let cap = re.captures(&token).ok_or(AssemblyError::token(token))?;
    let command = cap.name("command").map_or("", |m| m.as_str());
    let op_str = cap
        .name("operand")
        .map(|m| m.as_str())
        .ok_or(AssemblyError::token(token))?;
    let expression = Expr::parse(op_str)?;

    let statement = Statement::new(command.to_string(), expression);
    Ok(statement)
}

fn tokenize(text: &str) -> Vec<String> {
    let re = Regex::new(r#"\S=("[^"]*"|\S)+"#).unwrap();
    let mut tokens = Vec::new();

    for cap in re.captures_iter(text) {
        tokens.push(cap[0].to_string());
    }

    tokens
}

fn remove_after_quote(s: &str) -> String {
    let mut result = String::new();
    let mut in_quotes = false;
    for c in s.chars() {
        if c == '"' {
            in_quotes = !in_quotes;
        }
        if c == '\'' && !in_quotes {
            break;
        }
        result.push(c);
    }
    result
}
