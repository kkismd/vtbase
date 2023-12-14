use std::fmt;
use std::{error::Error, io};

use crate::opcode::{AddressingMode, Mnemonic};

#[derive(Debug, PartialEq)]
pub enum AssemblyError {
    SyntaxError(String),
    LabelError(String),
    ProgramError(String),
    MacroError(String),
    DecodeError(String),
    IoError(String),
}

impl From<io::Error> for AssemblyError {
    fn from(error: io::Error) -> Self {
        AssemblyError::IoError(error.to_string())
    }
}

impl AssemblyError {
    pub fn message(&self) -> &str {
        match self {
            AssemblyError::SyntaxError(details) => details,
            AssemblyError::LabelError(details) => details,
            AssemblyError::ProgramError(details) => details,
            AssemblyError::MacroError(details) => details,
            AssemblyError::DecodeError(details) => details,
            AssemblyError::IoError(details) => details,
        }
    }

    pub fn syntax(details: &str) -> Self {
        Self::SyntaxError(format!("syntax error: {details}"))
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

    pub fn label_used(line_num: usize, name: &str) -> Self {
        Self::LabelError(format!("line: {line_num} label <{name}> alreadsy used"))
    }

    pub fn label_not_found(name: &str) -> Self {
        Self::LabelError(format!("label <{name}> not found"))
    }

    pub fn program(details: &str) -> Self {
        Self::ProgramError(format!("program error: {details}"))
    }

    pub fn opcode_not_found(mnemonic: &Mnemonic, addressing_mode: &AddressingMode) -> Self {
        let details = format!(
            "opcode not found: {:?} with {:?}",
            mnemonic, addressing_mode
        );
        Self::SyntaxError(details)
    }

    pub fn decode_failed(details: &str) -> Self {
        Self::DecodeError(details.to_string())
    }
}

impl fmt::Display for AssemblyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AssemblyError::SyntaxError(details) => write!(f, "parse error: {}", details),
            AssemblyError::LabelError(details) => write!(f, "label error: {}", details),
            AssemblyError::ProgramError(details) => write!(f, "syntax error: {}", details),
            AssemblyError::MacroError(details) => write!(f, "syntax error: {}", details),
            AssemblyError::DecodeError(details) => write!(f, "decode error: {}", details),
            AssemblyError::IoError(details) => write!(f, "io error: {}", details),
        }
    }
}

impl Error for AssemblyError {}
