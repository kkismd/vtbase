use std::fmt;
use std::{error::Error, io};

use crate::opcode::{AddressingMode, Mnemonic};

#[derive(Debug, PartialEq)]
pub enum AssemblyError {
    Syntax(String),
    Label(String),
    Program(String),
    Macro(String),
    Decode(String),
    Io(String),
}

impl From<io::Error> for AssemblyError {
    fn from(error: io::Error) -> Self {
        AssemblyError::Io(error.to_string())
    }
}

impl AssemblyError {
    pub fn message(&self) -> &str {
        match self {
            AssemblyError::Syntax(details) => details,
            AssemblyError::Label(details) => details,
            AssemblyError::Program(details) => details,
            AssemblyError::Macro(details) => details,
            AssemblyError::Decode(details) => details,
            AssemblyError::Io(details) => details,
        }
    }

    pub fn syntax(details: &str) -> Self {
        Self::Syntax(format!("syntax error: {details}"))
    }

    pub fn line(line_num: usize, line: &str) -> Self {
        Self::Syntax(format!("line: {line_num} at: {line}"))
    }

    pub fn token(token: &str) -> Self {
        Self::Syntax(format!("invaled token: {}", token))
    }

    pub fn expression(expr: &str) -> Self {
        Self::Syntax(format!("invalid expression: {}", expr))
    }

    pub fn label_used(line_num: usize, name: &str) -> Self {
        Self::Label(format!("line: {line_num} label <{name}> alreadsy used"))
    }

    pub fn label_not_found(name: &str) -> Self {
        Self::Label(format!("label <{name}> not found"))
    }

    pub fn program(details: &str) -> Self {
        Self::Program(format!("program error: {details}"))
    }

    pub fn opcode_not_found(mnemonic: &Mnemonic, addressing_mode: &AddressingMode) -> Self {
        let details = format!(
            "opcode not found: {:?} with {:?}",
            mnemonic, addressing_mode
        );
        Self::Syntax(details)
    }

    pub fn decode_failed(details: &str) -> Self {
        Self::Decode(details.to_string())
    }
}

impl fmt::Display for AssemblyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AssemblyError::Syntax(details) => write!(f, "parse error: {}", details),
            AssemblyError::Label(details) => write!(f, "label error: {}", details),
            AssemblyError::Program(details) => write!(f, "syntax error: {}", details),
            AssemblyError::Macro(details) => write!(f, "syntax error: {}", details),
            AssemblyError::Decode(details) => write!(f, "decode error: {}", details),
            AssemblyError::Io(details) => write!(f, "io error: {}", details),
        }
    }
}

impl Error for AssemblyError {}
