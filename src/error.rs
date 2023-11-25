use std::error::Error;
use std::fmt;

use crate::opcode::{AddressingMode, Mnemonic};

#[derive(Debug)]
pub enum AssemblyError {
    SyntaxError(String),
    LabelError(String),
    ProgramError(String),
    MacroError(String),
}

impl AssemblyError {
    pub fn message(&self) -> &String {
        match self {
            AssemblyError::SyntaxError(details) => details,
            AssemblyError::LabelError(details) => details,
            AssemblyError::ProgramError(details) => details,
            AssemblyError::MacroError(details) => details,
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
}

impl fmt::Display for AssemblyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AssemblyError::SyntaxError(details) => write!(f, "parse error: {}", details),
            AssemblyError::LabelError(details) => write!(f, "label error: {}", details),
            AssemblyError::ProgramError(details) => write!(f, "syntax error: {}", details),
            AssemblyError::MacroError(details) => write!(f, "syntax error: {}", details),
        }
    }
}

impl Error for AssemblyError {}
