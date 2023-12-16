use crate::error::AssemblyError;

use regex::Captures;
use regex::Regex;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::vec;
pub mod expression;
use expression::Expr;

mod include_reader;
use include_reader::IncludeReader;

pub mod statement;
use statement::Statement;

// line of source code
#[derive(Debug, Clone)]
pub struct Line {
    pub line_number: usize,
    pub address: u16,
    pub label: Option<String>,
    pub statements: Vec<Statement>,
    pub object_codes: Vec<u8>,
}

impl Line {
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
pub fn parse_from_file(file: &File, file_path: PathBuf) -> Result<Vec<Line>, AssemblyError> {
    let reader = BufReader::new(file);
    let include_reader = IncludeReader::new(reader, file_path);
    let mut lines = Vec::new();

    for (num, line) in include_reader.lines().enumerate() {
        let res = line;
        if let Ok(line) = res {
            let line = parse_line(line, num + 1)?;
            lines.push(line);
        }
    }
    Ok(lines)
}

// make abstract syntax tree
fn parse_line(line: String, line_num: usize) -> Result<Line, AssemblyError> {
    let line = remove_after_double_semicolon(&line);
    let cap = match_line(&line, line_num)?;

    let body = cap.name("body").map_or("", |m| m.as_str());
    let tokens = tokenize(body);
    let statements =
        parse_statements(tokens).map_err(|e| AssemblyError::line(line_num, &e.message()))?;
    Ok(Line::new(
        line_num,
        0,
        cap.name("label").map(|m| m.as_str()).map(String::from),
        statements,
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
    let assignment_pattern = Regex::new(r"^(?P<command>[^=]+)=(?P<operand>.+)$").unwrap();
    let single_pattern = Regex::new(r"(?P<command>\S)").unwrap();
    let cap = assignment_pattern
        .captures(&token)
        .or_else(|| single_pattern.captures(&token))
        .ok_or(AssemblyError::token(token))?;
    let command = cap
        .name("command")
        .map(|m| m.as_str())
        .ok_or(AssemblyError::token(token))
        .and_then(|s| Expr::parse(s))?;

    let operand = cap.name("operand").map(|m| m.as_str()).unwrap_or("");
    let expression = Expr::parse(operand)?;

    let statement = Statement {
        command,
        expression,
    };
    Ok(statement)
}

fn tokenize(text: &str) -> Vec<String> {
    let re = Regex::new(r#"("[^"]*"|\S)+"#).unwrap();
    let mut tokens = Vec::new();

    for cap in re.captures_iter(text) {
        tokens.push(cap[0].to_string());
    }

    tokens
}

fn remove_after_double_semicolon(s: &str) -> String {
    let mut result = String::new();
    let mut in_quotes = false;
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '"' {
            in_quotes = !in_quotes;
        }
        if c == ';' && !in_quotes {
            if let Some(&next) = chars.peek() {
                if next == ';' {
                    break;
                }
            }
        }
        result.push(c);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_assignemnt_decimal() {
        let statement = parse_token("A=1").unwrap();
        assert_eq!(statement.command, Expr::Identifier("A".to_string()));
        assert_eq!(statement.expression, Expr::DecimalNum(1));
    }

    #[test]
    fn test_parse_assignemnt_hex() {
        let statement = parse_token("A=$10").unwrap();
        assert_eq!(statement.command, Expr::Identifier("A".to_string()));
        assert_eq!(statement.expression, Expr::ByteNum(0x10));
    }

    #[test]
    fn test_parse_assignment_zerox() {
        let statement = parse_token("A=$10+X").unwrap();
        assert_eq!(statement.command, Expr::Identifier("A".to_string()));
        assert_eq!(
            statement.expression,
            Expr::BinOp(
                Box::new(Expr::ByteNum(0x10)),
                expression::Operator::Add,
                Box::new(Expr::Identifier("X".to_string()))
            )
        );
    }

    #[test]
    fn test_parse_label_def() {
        let statement = parse_token(":=$80FF").unwrap();
        assert_eq!(statement.command, Expr::SystemOperator(':'.to_string()));
        assert_eq!(statement.expression, Expr::WordNum(0x80FF));
    }

    #[test]
    fn test_parse_sta() {
        let statement = parse_token("($10)=A").unwrap();
        assert_eq!(
            statement.command,
            Expr::Parenthesized(Box::new(Expr::ByteNum(0x10)))
        );
        assert_eq!(statement.expression, Expr::Identifier("A".to_string()));
    }

    #[test]
    fn test_parse_token_ifeq() {
        let statement = parse_token(";==,.skip").unwrap();
        assert_eq!(statement.command, Expr::SystemOperator(';'.to_string()));
        assert_eq!(
            statement.expression,
            Expr::BinOp(
                Box::new(Expr::SystemOperator('='.to_string())),
                expression::Operator::Comma,
                Box::new(Expr::Identifier(".skip".to_string()))
            )
        );
    }

    #[test]
    fn test_tokenize() {
        let tokens = tokenize("@ A=1 B=2 C=\"hello world\",0 (0)=A");
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0], "@");
        assert_eq!(tokens[1], "A=1");
        assert_eq!(tokens[2], "B=2");
        assert_eq!(tokens[3], "C=\"hello world\",0");
        assert_eq!(tokens[4], "(0)=A");
    }

    #[test]
    fn test_remove_after_quote() {
        let s = "A=1 ;; comment";
        assert_eq!(remove_after_double_semicolon(s), "A=1 ");
    }
}
