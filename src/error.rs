use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum AssemblyError {
    ParseError(String),
    LabelError(String),
    SyntaxError(String),
}

impl AssemblyError {
    pub fn message(&self) -> &String {
        match self {
            AssemblyError::ParseError(details) => details,
            AssemblyError::LabelError(details) => details,
            AssemblyError::SyntaxError(details) => details,
        }
    }

    pub fn new_parse_error(details: &str) -> Self {
        Self::ParseError(details.to_string())
    }

    pub fn line(line_num: usize, line: &str) -> Self {
        Self::ParseError(format!("line: {line_num} at: {line}"))
    }

    pub fn token(token: &str) -> Self {
        Self::ParseError(format!("invaled token: {}", token))
    }

    pub fn expression(expr: &str) -> Self {
        Self::ParseError(format!("invalid expression: {}", expr))
    }

    pub fn label_not_found(name: &str) -> Self {
        Self::LabelError(format!("label not found: {}", name))
    }

    pub fn label_used(line_num: usize, name: &str) -> Self {
        Self::LabelError(format!("line: {line_num} label <{name}> alreadsy used"))
    }
}

impl fmt::Display for AssemblyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AssemblyError::ParseError(details) => write!(f, "parse error: {}", details),
            AssemblyError::LabelError(details) => write!(f, "label error: {}", details),
            AssemblyError::SyntaxError(details) => write!(f, "syntax error: {}", details),
        }
    }
}

impl Error for AssemblyError {}
