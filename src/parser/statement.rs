use super::expression::Operator;
use crate::assembler::{Address, LabelEntry};
use crate::error::AssemblyError;
use crate::opcode::{AddressingMode, AssemblyInstruction, OpcodeTable, OperandValue};
use crate::parser::expression::Expr;
use std::collections::HashMap;
pub mod decoder;
use decoder::*;

// instruction in a line of source code
#[derive(Debug, Clone)]
pub struct Statement {
    pub command: Expr,
    pub expression: Expr,
}

impl Statement {
    pub fn new(command: &str, expression: Expr) -> Self {
        let command = Expr::Identifier(command.to_string());
        Self {
            command,
            expression,
        }
    }

    pub fn command(&self) -> Result<String, AssemblyError> {
        match &self.command {
            Expr::Identifier(command) => Ok(command.clone()),
            Expr::SystemOperator(symbol) => Ok(symbol.to_string()),
            Expr::Parenthesized(_) => Ok("(#<Expr>)".to_string()),
            Expr::Bracketed(_) => Ok("[#<Expr>]".to_string()),
            Expr::BinOp(_, Operator::Add, _) => Ok("#<Expr>+#<Expr>".to_string()),
            _ => Err(AssemblyError::syntax("must be identifier")),
        }
    }

    pub fn is_pseudo(&self) -> bool {
        if let Ok(command) = self.command() {
            return command == "*" || command == ":" || command == "?";
        } else {
            false
        }
    }

    pub fn is_macro(&self) -> bool {
        match self.command {
            Expr::SystemOperator('@') => true,
            Expr::SystemOperator(';') => self.check_macro_if_statement(),
            _ => false,
        }
    }

    // ;=記号,式 の場合はマクロではない
    // それ以外の二項演算子の場合はマクロ
    pub fn check_macro_if_statement(&self) -> bool {
        match &self.expression {
            Expr::BinOp(left, operator, _) => {
                if let Expr::SystemOperator(_) = **left {
                    if *operator == Operator::Comma {
                        return false;
                    }
                }
                return true;
            }
            _ => (),
        }
        false
    }

    pub fn decode(
        &self,
        labels: &HashMap<String, LabelEntry>,
    ) -> Result<AssemblyInstruction, AssemblyError> {
        let command = self.command()?;
        let expr = &self.expression;
        match command.as_str() {
            "X" => decode_x(&expr, labels),
            "Y" => decode_y(&expr, labels),
            "A" => decode_a(&expr, labels),
            "T" => decode_t(&expr, labels),
            "C" => decode_c(&expr, labels),
            "!" => decode_call(&expr, labels),
            "#" => decode_goto(&expr, labels),
            ";" => decode_if(&expr, labels),
            _ => decode_address(&self.command, &expr, labels),
        }
    }

    /**
     * compile statement to object codes
     * @return Vec<u8> assembled code
     */
    pub fn compile(
        &self,
        opcode_table: &OpcodeTable,
        labels: &HashMap<String, LabelEntry>,
        current_label: &str,
        pc: u16,
    ) -> Result<Vec<u8>, AssemblyError> {
        let assembly_instruction = self.decode(labels)?;
        // find opcode from mnemonic and mode
        let opcode = opcode_table.find(
            &assembly_instruction.mnemonic,
            &assembly_instruction.addressing_mode,
        )?;
        let operand = self.operand_bytes(&assembly_instruction, labels, current_label, pc)?;

        let mut bytes = vec![];
        bytes.push(opcode.opcode);
        bytes.extend(&operand);
        Ok(bytes)
    }

    fn operand_bytes(
        &self,
        assembly_instruction: &AssemblyInstruction,
        labels: &HashMap<String, LabelEntry>,
        current_label: &str,
        pc: u16,
    ) -> Result<Vec<u8>, AssemblyError> {
        let operand = match assembly_instruction.value {
            OperandValue::None => vec![],
            OperandValue::Byte(value) => vec![value],
            OperandValue::Word(value) => vec![value as u8, (value >> 8) as u8],
            OperandValue::UnresolvedLabel(ref name) => self.resolve_label(
                &name,
                &assembly_instruction.addressing_mode,
                &labels,
                current_label,
                pc,
            )?,
            OperandValue::UnresolvedRelative(addr) => Self::absolute_to_relative(addr, pc),
        };
        Ok(operand)
    }

    fn resolve_label(
        &self,
        name: &str,
        mode: &AddressingMode,
        labels: &HashMap<String, LabelEntry>,
        current_label: &str,
        pc: u16,
    ) -> Result<Vec<u8>, AssemblyError> {
        let name = Self::full_qualify_name(name, current_label);
        if let Some(entry) = labels.get(&name) {
            if let Address::Full(absolute_address) = entry.address {
                if mode == &AddressingMode::Relative {
                    return Ok(Self::absolute_to_relative(absolute_address, pc + 2));
                } else {
                    return Ok(vec![absolute_address as u8, (absolute_address >> 8) as u8]);
                }
            }
            if let Address::ZeroPage(address) = entry.address {
                dbg!(address);
                dbg!(mode);
                match mode {
                    AddressingMode::ZeroPage
                    | AddressingMode::ZeroPageX
                    | AddressingMode::ZeroPageY => {
                        return Ok(vec![address as u8]);
                    }
                    _ => (),
                }
            }
        }
        Err(AssemblyError::syntax(&format!("unknown label: {}", name)))
    }

    fn full_qualify_name(name: &str, current_label: &str) -> String {
        if name.starts_with('.') {
            format!("{}{}", current_label, &name)
        } else {
            name.to_string()
        }
    }

    fn absolute_to_relative(address: u16, pc: u16) -> Vec<u8> {
        let diff = address.wrapping_sub(pc) as u8;
        vec![diff as u8]
    }
}
