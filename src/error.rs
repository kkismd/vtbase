use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum AssemblyError {
    SyntaxError(String),
    LabelError(String),
    ProgremError(String),
}

impl AssemblyError {
    pub fn message(&self) -> &String {
        match self {
            AssemblyError::SyntaxError(details) => details,
            AssemblyError::LabelError(details) => details,
            AssemblyError::ProgremError(details) => details,
        }
    }

    pub fn new_parse_error(details: &str) -> Self {
        Self::SyntaxError(details.to_string())
    }

    pub fn line(line_num: usize, line: &str) -> Self {
        Self::SyntaxError(format!("line: {line_num} at: {line}"))
    }

    pub fn token(token: &str) -> Self {
        Self::SyntaxError(format!("invaled token: {}", token))
    }

    pub fn expression(expr: &str) -> Self {
        Self::SyntaxError(format!("invalid expression: {}", expr))
    }

    pub fn label_not_found(name: &str) -> Self {
        Self::LabelError(format!("label not found: {}", name))
    }

    pub fn label_used(line_num: usize, name: &str) -> Self {
        Self::LabelError(format!("line: {line_num} label <{name}> alreadsy used"))
    }

    pub fn program(details: &str) -> Self {
        Self::ProgremError(format!("program error: {details}"))
    }
}

impl fmt::Display for AssemblyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AssemblyError::SyntaxError(details) => write!(f, "parse error: {}", details),
            AssemblyError::LabelError(details) => write!(f, "label error: {}", details),
            AssemblyError::ProgremError(details) => write!(f, "syntax error: {}", details),
        }
    }
}

impl Error for AssemblyError {}
