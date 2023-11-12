use std::error::Error;
use std::fmt;

// ParseErrorを定義
#[derive(Debug)]
pub struct ParseError {
    pub details: String,
}

impl ParseError {
    pub fn new(details: &str) -> ParseError {
        ParseError {
            details: details.to_string(),
        }
    }
    pub fn line(line_num: usize, line: &str) -> ParseError {
        ParseError {
            details: format!("line: {line_num} at: {line}"),
        }
    }
    pub fn token(token: &str) -> ParseError {
        ParseError {
            details: format!("invaled token: {}", token),
        }
    }
    pub fn operand(operand: &str) -> ParseError {
        ParseError {
            details: format!("invalid operand: {}", operand),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for ParseError {}
