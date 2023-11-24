use std::collections::HashMap;

use crate::assembler::{Address, LabelEntry};
use crate::error::AssemblyError;
use crate::opcode::{
    AddressingMode, AssemblyInstruction, Mnemonic, Mode, OpcodeTable, OperandValue,
};
use crate::parser::expression::Expr;

use super::expression::Operator;

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

    pub fn validate_pseudo_command(&self) -> Result<(), AssemblyError> {
        let command = self.command()?;
        if command == "*" || command == ":" {
            match self.expression {
                Expr::WordNum(_) => Ok(()),
                _ => Err(AssemblyError::syntax("operand must be 16bit address")),
            }
        } else if command == "?" {
            match self.expression {
                Expr::StringLiteral(_) => Ok(()),
                _ => Err(AssemblyError::syntax("operand must be string")),
            }
        } else {
            let details = format!("unknown command: <{}>", command);
            Err(AssemblyError::syntax(&details))
        }
    }

    pub fn is_macro(&self) -> bool {
        if let Expr::Identifier(command) = &self.command {
            match command.as_str() {
                ";" => self.check_macro_if_statement(),
                "@" => true,
                _ => false,
            }
        } else {
            false
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

    /**
     * decode command and expression into mnemonic and addressing mode
     * @return (Mnemonic, Mode)
     */
    pub fn decode(&self) -> Result<AssemblyInstruction, AssemblyError> {
        let command = self.command()?;
        match command.as_str() {
            "X" => self.decode_x(),
            "Y" => self.decode_y(),
            "A" => self.decode_a(),
            "T" => self.decode_t(),
            "C" => self.decode_c(),
            "!" => self.decode_call(),
            "#" => self.decode_goto(),
            ";" => self.decode_if(),
            _ => {
                dbg!(self);
                todo!()
            }
        }
    }

    fn decode_x(&self) -> Result<AssemblyInstruction, AssemblyError> {
        let expr = &self.expression;
        if let Expr::ByteNum(num) = *expr {
            return Ok(AssemblyInstruction::new(
                Mnemonic::LDX,
                Mode::Immediate,
                OperandValue::byte(num),
            ));
        }
        if let Expr::DecimalNum(num) = *expr {
            if num > 255 {
                return Err(AssemblyError::syntax("operand must be 8bit"));
            }
            return Ok(AssemblyInstruction::new(
                Mnemonic::LDX,
                Mode::Immediate,
                OperandValue::Byte(num as u8),
            ));
        }
        if let Expr::ByteNum(num) = self.expression {
            return self.ok_byte(Mnemonic::LDX, Mode::ZeroPage, num);
        }
        if let Expr::BinOp(left, operator, right) = &self.expression {
            if let Expr::Identifier(ref s) = **left {
                if let Expr::DecimalNum(n) = **right {
                    if s == "X" && n == 1 {
                        if *operator == Operator::Add {
                            return self.ok_none(Mnemonic::INX, Mode::Implied);
                        }
                        if *operator == Operator::Sub {
                            return self.ok_none(Mnemonic::DEX, Mode::Implied);
                        }
                    }
                }
            }
        }
        if let Expr::SystemOperator(symbol) = self.expression {
            if symbol == '+' {
                return self.ok_none(Mnemonic::INX, Mode::Implied);
            }
            if symbol == '-' {
                return self.ok_none(Mnemonic::DEX, Mode::Implied);
            }
        }
        self.decode_error()
    }

    fn decode_y(&self) -> Result<AssemblyInstruction, AssemblyError> {
        let expr = &self.expression;
        if let Expr::ByteNum(num) = *expr {
            return self.ok_byte(Mnemonic::LDY, Mode::Immediate, num);
        }
        if let Expr::DecimalNum(num) = *expr {
            if num > 255 {
                return Err(AssemblyError::syntax("operand must be 8bit"));
            }
            return self.ok_byte(Mnemonic::LDY, Mode::Immediate, num as u8);
        }
        if let Expr::ByteNum(num) = self.expression {
            return self.ok_byte(Mnemonic::LDY, Mode::ZeroPage, num);
        }
        if let Expr::BinOp(left, operator, right) = &self.expression {
            if let Expr::Identifier(ref s) = **left {
                if let Expr::DecimalNum(n) = **right {
                    if s == "Y" && n == 1 {
                        if *operator == Operator::Add {
                            return self.ok_none(Mnemonic::INY, Mode::Implied);
                        }
                        if *operator == Operator::Sub {
                            return self.ok_none(Mnemonic::DEY, Mode::Implied);
                        }
                    }
                }
            }
        }
        if let Expr::SystemOperator(symbol) = self.expression {
            if symbol == '+' {
                return self.ok_none(Mnemonic::INY, Mode::Implied);
            }
            if symbol == '-' {
                return self.ok_none(Mnemonic::DEY, Mode::Implied);
            }
        }
        self.decode_error()
    }

    fn decode_a(&self) -> Result<AssemblyInstruction, AssemblyError> {
        let expr = &self.expression;
        if let Expr::DecimalNum(num) = *expr {
            if num > 255 {
                return Err(AssemblyError::syntax("operand must be 8bit"));
            } else {
                return self.ok_byte(Mnemonic::LDA, Mode::Immediate, num as u8);
            }
        }
        if let Expr::ByteNum(num) = *expr {
            return self.ok_byte(Mnemonic::LDA, Mode::Immediate, num);
        }
        if let Expr::BinOp(left, operator, right) = &self.expression {
            if let Expr::Identifier(ref name) = **left {
                if *operator == Operator::Add {
                    if let Expr::Identifier(ref reg) = **right {
                        if reg == "X" {
                            return self.ok_unresolved_label(Mnemonic::LDA, Mode::AbsoluteX, &name);
                        } else if reg == "Y" {
                            return self.ok_unresolved_label(Mnemonic::LDA, Mode::AbsoluteY, &name);
                        }
                    }
                }
                // A=A-$48
                if name == "A" && *operator == Operator::Sub {
                    let expr = right;
                    if let Expr::DecimalNum(num) = **expr {
                        if num < 256 {
                            return self.ok_byte(Mnemonic::SBC, Mode::Immediate, num as u8);
                        }
                    }
                    if let Expr::ByteNum(num) = **expr {
                        return self.ok_byte(Mnemonic::SBC, Mode::Immediate, num);
                    }
                }
                // A=A+$48
                if name == "A" && *operator == Operator::Add {
                    let expr = right;
                    if let Expr::DecimalNum(num) = **expr {
                        if num < 256 {
                            return self.ok_byte(Mnemonic::ADC, Mode::Immediate, num as u8);
                        }
                    }
                    if let Expr::ByteNum(num) = **expr {
                        return self.ok_byte(Mnemonic::ADC, Mode::Immediate, num);
                    }
                }
            }
        }
        if let Expr::Identifier(name) = &self.expression {
            if name == "X" {
                return self.ok_none(Mnemonic::TXA, Mode::Implied);
            } else if name == "Y" {
                return self.ok_none(Mnemonic::TYA, Mode::Implied);
            } else {
                return self.ok_unresolved_label(Mnemonic::LDA, Mode::Absolute, &name);
            }
        }
        self.decode_error()
    }

    fn decode_t(&self) -> Result<AssemblyInstruction, AssemblyError> {
        if let Expr::BinOp(left, operator, right) = &self.expression {
            if let Expr::Identifier(ref reg) = **left {
                if *operator == Operator::Sub {
                    let expr = right;
                    if let Expr::DecimalNum(num) = **expr {
                        if num < 256 {
                            if reg == "A" {
                                return self.ok_byte(Mnemonic::CMP, Mode::Immediate, num as u8);
                            }
                            if reg == "X" {
                                return self.ok_byte(Mnemonic::CPX, Mode::Immediate, num as u8);
                            }
                            if reg == "Y" {
                                return self.ok_byte(Mnemonic::CPY, Mode::Immediate, num as u8);
                            }
                        }
                    }
                    if let Expr::ByteNum(num) = **expr {
                        if reg == "A" {
                            return self.ok_byte(Mnemonic::CMP, Mode::Immediate, num);
                        }
                        if reg == "X" {
                            return self.ok_byte(Mnemonic::CPX, Mode::Immediate, num);
                        }
                        if reg == "Y" {
                            return self.ok_byte(Mnemonic::CPY, Mode::Immediate, num);
                        }
                    }
                }
            }
        }
        self.decode_error()
    }

    fn decode_c(&self) -> Result<AssemblyInstruction, AssemblyError> {
        if let Expr::DecimalNum(num) = self.expression {
            if num == 1 {
                return self.ok_none(Mnemonic::SEC, Mode::Implied);
            }
            if num == 0 {
                return self.ok_none(Mnemonic::CLC, Mode::Implied);
            }
        }
        self.decode_error()
    }

    fn decode_call(&self) -> Result<AssemblyInstruction, AssemblyError> {
        if let Expr::Identifier(name) = &self.expression {
            return self.ok_unresolved_label(Mnemonic::JSR, Mode::Absolute, &name);
        }
        if let Expr::WordNum(num) = self.expression {
            return self.ok_word(Mnemonic::JSR, Mode::Absolute, num);
        }
        self.decode_error()
    }

    fn decode_goto(&self) -> Result<AssemblyInstruction, AssemblyError> {
        match &self.expression {
            Expr::WordNum(num) => {
                return self.ok_word(Mnemonic::JMP, Mode::Absolute, *num);
            }
            Expr::Identifier(name) => {
                return self.ok_unresolved_label(Mnemonic::JMP, Mode::Absolute, &name);
            }
            Expr::SystemOperator('!') => return self.ok_none(Mnemonic::RTS, Mode::Implied),
            _ => return self.decode_error(),
        }
    }

    fn decode_if(&self) -> Result<AssemblyInstruction, AssemblyError> {
        // ;=/,$12fd (IF NOT EQUAL THEN GOTO $12FD)
        if let Expr::BinOp(expr_left, Operator::Comma, expr_right) = &self.expression {
            if let Expr::SystemOperator(symbol) = **expr_left {
                match **expr_right {
                    Expr::WordNum(addr) => match symbol {
                        '/' | '~' | '!' => {
                            return self.ok_unresolved_relative(Mnemonic::BNE, Mode::Relative, addr)
                        }
                        '=' => {
                            return self.ok_unresolved_relative(Mnemonic::BEQ, Mode::Relative, addr)
                        }
                        '>' => {
                            return self.ok_unresolved_relative(Mnemonic::BCS, Mode::Relative, addr)
                        }
                        '<' => {
                            return self.ok_unresolved_relative(Mnemonic::BCC, Mode::Relative, addr)
                        }
                        _ => (),
                    },
                    Expr::Identifier(ref name) => match symbol {
                        '/' => {
                            return self.ok_unresolved_label(Mnemonic::BNE, Mode::Relative, name)
                        }
                        '=' => {
                            return self.ok_unresolved_label(Mnemonic::BEQ, Mode::Relative, name)
                        }
                        '>' => {
                            return self.ok_unresolved_label(Mnemonic::BCS, Mode::Relative, name)
                        }
                        '<' => {
                            return self.ok_unresolved_label(Mnemonic::BCC, Mode::Relative, name)
                        }

                        _ => (),
                    },
                    _ => (),
                }
            }
        }
        self.decode_error()
    }

    fn ok_byte(
        &self,
        mnemonic: Mnemonic,
        mode: Mode,
        num: u8,
    ) -> Result<AssemblyInstruction, AssemblyError> {
        Ok(AssemblyInstruction::new(
            mnemonic,
            mode,
            OperandValue::Byte(num),
        ))
    }

    fn ok_word(
        &self,
        mnemonic: Mnemonic,
        mode: Mode,
        num: u16,
    ) -> Result<AssemblyInstruction, AssemblyError> {
        Ok(AssemblyInstruction::new(
            mnemonic,
            mode,
            OperandValue::Word(num),
        ))
    }

    fn ok_none(
        &self,
        mnemonic: Mnemonic,
        mode: Mode,
    ) -> Result<AssemblyInstruction, AssemblyError> {
        Ok(AssemblyInstruction::new(mnemonic, mode, OperandValue::None))
    }

    fn ok_unresolved_label(
        &self,
        mnemonic: Mnemonic,
        mode: Mode,
        name: &str,
    ) -> Result<AssemblyInstruction, AssemblyError> {
        Ok(AssemblyInstruction::new(
            mnemonic,
            mode,
            OperandValue::unresolved_label(name),
        ))
    }

    fn ok_unresolved_relative(
        &self,
        mnemonic: Mnemonic,
        mode: Mode,
        addr: u16,
    ) -> Result<AssemblyInstruction, AssemblyError> {
        Ok(AssemblyInstruction::new(
            mnemonic,
            mode,
            OperandValue::UnresolvedRelative(addr),
        ))
    }

    fn decode_error(&self) -> Result<AssemblyInstruction, AssemblyError> {
        Err(AssemblyError::syntax(&format!(
            "bad expression: {:?}",
            self
        )))
    }

    /**
     * compile statement to object codes
     * @return Vec<u8> assembled code
     */
    pub fn compile(
        &self,
        opcode_table: &OpcodeTable,
        labels: &HashMap<String, LabelEntry>,
        pc: u16,
    ) -> Result<Vec<u8>, AssemblyError> {
        let assembly_instruction = self.decode()?;
        // find opcode from mnemonic and mode
        let opcode = opcode_table.find(
            &assembly_instruction.mnemonic,
            &assembly_instruction.addressing_mode,
        )?;
        let operand = self.operand_bytes(&assembly_instruction, labels, pc)?;

        let mut bytes = vec![];
        bytes.push(opcode.opcode);
        bytes.extend(&operand);
        Ok(bytes)
    }

    fn operand_bytes(
        &self,
        assembly_instruction: &AssemblyInstruction,
        labels: &HashMap<String, LabelEntry>,
        pc: u16,
    ) -> Result<Vec<u8>, AssemblyError> {
        let operand = match assembly_instruction.value {
            OperandValue::None => vec![],
            OperandValue::Byte(value) => vec![value],
            OperandValue::Word(value) => vec![value as u8, (value >> 8) as u8],
            OperandValue::UnresolvedLabel(ref name) => {
                self.resolve_label(&name, &assembly_instruction.addressing_mode, &labels, pc)?
            }
            OperandValue::UnresolvedRelative(addr) => Self::absolute_to_relative(addr, pc),
        };
        Ok(operand)
    }

    fn resolve_label(
        &self,
        name: &str,
        mode: &AddressingMode,
        labels: &HashMap<String, LabelEntry>,
        pc: u16,
    ) -> Result<Vec<u8>, AssemblyError> {
        if let Some(entry) = labels.get(name) {
            if let Address::Full(absolute_address) = entry.address {
                if mode == &AddressingMode::Relative {
                    return Ok(Self::absolute_to_relative(absolute_address, pc + 2));
                } else {
                    return Ok(vec![absolute_address as u8, (absolute_address >> 8) as u8]);
                }
            }
            if let Address::ZeroPage(address) = entry.address {
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

    fn absolute_to_relative(address: u16, pc: u16) -> Vec<u8> {
        let diff = address.wrapping_sub(pc) as u8;
        vec![diff as u8]
    }
}
