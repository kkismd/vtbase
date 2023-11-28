use super::expression::Operator;
use crate::assembler::{Address, LabelEntry};
use crate::error::AssemblyError;
use crate::opcode::{
    AddressingMode, AssemblyInstruction, Mnemonic, Mode, OpcodeTable, OperandValue,
};
use crate::parser::expression::matcher::*;
use crate::parser::expression::Expr;
use std::collections::HashMap;
mod decoder;
use decoder::*;
use nom::Err;

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
                Expr::ByteNum(_) => Ok(()),
                _ => Err(AssemblyError::syntax("operand must be address")),
            }
        } else if command == "?" {
            match self.expression {
                Expr::StringLiteral(_) => Ok(()),
                Expr::BinOp(_, Operator::Comma, _) => Ok(()),
                _ => Err(AssemblyError::syntax("operand must be string")),
            }
        } else {
            let details = format!("unknown command: <{}>", command);
            Err(AssemblyError::syntax(&details))
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
        match command.as_str() {
            "X" => self.decode_x(labels),
            "Y" => self.decode_y(labels),
            "A" => self.decode_a(labels),
            "T" => self.decode_t(labels),
            "C" => self.decode_c(labels),
            "!" => self.decode_call(labels),
            "#" => self.decode_goto(labels),
            ";" => self.decode_if(labels),
            _ => self.decode_other(labels),
        }
    }

    /**
     * A=1 or A=$10 or A=label
     */
    fn immediate(&self, mnemonic: Mnemonic) -> Result<AssemblyInstruction, AssemblyError> {
        num8bit(&self.expression)
            .and_then(|num| self.ok_byte(&mnemonic, Mode::Immediate, num))
            .map_err(|_| AssemblyError::syntax("invalid immediate"))
    }

    fn lookup(
        name: &str,
        labels: &HashMap<String, LabelEntry>,
    ) -> Result<LabelEntry, AssemblyError> {
        labels
            .get(name)
            .cloned()
            .ok_or(AssemblyError::syntax("unknown label"))
    }

    /**
     * A=($1F) or A=(31) or A=(label)
     */
    fn zeropage(
        &self,
        mnemonic: Mnemonic,
        labels: &HashMap<String, LabelEntry>,
    ) -> Result<AssemblyInstruction, AssemblyError> {
        let expr = &self.expression;
        parenthes_within::<u8>(expr, num8bit)
            // A=($1F) or A=(31)
            .and_then(|num| self.ok_byte(&mnemonic, Mode::ZeroPage, num))
            .or_else(|_| {
                parenthes_within::<String>(expr, identifier)
                    // A=(label)
                    .and_then(|name| {
                        Self::lookup(&name, labels).and_then(|entry| match entry.address {
                            Address::ZeroPage(addr) => {
                                self.ok_byte(&mnemonic, Mode::ZeroPage, addr)
                            }
                            Address::Full(addr) => self.ok_word(&mnemonic, Mode::Absolute, addr),
                            _ => Err(AssemblyError::syntax("invalid zeropage")),
                        })
                    })
            })
    }
    fn label_zeropage(&self, labels: &HashMap<String, LabelEntry>) -> Result<u8, AssemblyError> {
        let expr = &self.expression;
        identifier(expr).and_then(|name| {
            Self::lookup(&name, labels).and_then(|entry| match entry.address {
                Address::ZeroPage(addr) => Ok(addr),
                _ => Err(AssemblyError::syntax("invalid zeropage")),
            })
        })
    }

    /**
     * X=($1F+Y) or X=(31+Y) or X=(label+Y)
     */
    fn zeropage_y(&self, mnemonic: Mnemonic) -> Result<AssemblyInstruction, AssemblyError> {
        let expr = &self.expression;
        parenthes_within(expr, plus).and_then(|(left, right)| {
            register_y(&right)
                .and_then(|_| {
                    // X=($1F+Y) or X=(31+Y)
                    num8bit(&left).and_then(|num| self.ok_byte(&mnemonic, Mode::ZeroPageY, num))
                })
                .or_else(|_|
                    // X=(label+Y)
                    todo!())
                .or(self.decode_error())
        })
    }

    /**
     * X=X+1 or X=+
     */
    fn incr_decrement(
        &self,
        mnemonic_plus: Mnemonic,
        mnemonic_minus: Mnemonic,
        register_left: &str,
    ) -> Result<AssemblyInstruction, AssemblyError> {
        self.incr_decr_long(&mnemonic_plus, &mnemonic_minus, register_left)
            .or_else(|_| self.incr_decr_short(&mnemonic_plus, &mnemonic_minus))
    }

    /**
     * X=X+1
     */
    fn incr_decr_long(
        &self,
        mnemonic_plus: &Mnemonic,
        mnemonic_minus: &Mnemonic,
        register_left: &str,
    ) -> Result<AssemblyInstruction, AssemblyError> {
        let expr = &self.expression;
        binop(expr).and_then(|(left, operator, right)| {
            // X=X+1 or Y=Y+1
            identifier(&left).and_then(|register_right| {
                decimal(&right).and_then(|num| {
                    if num == 1 && register_left == register_right {
                        match operator {
                            Operator::Add => self.ok_none(mnemonic_plus.clone(), Mode::Implied),
                            Operator::Sub => self.ok_none(mnemonic_minus.clone(), Mode::Implied),
                            _ => self.decode_error(),
                        }
                    } else {
                        self.decode_error()
                    }
                })
            })
        })
    }

    /**
     * X=+
     */
    fn incr_decr_short(
        &self,
        mnemonic_plus: &Mnemonic,
        mnemonic_minus: &Mnemonic,
    ) -> Result<AssemblyInstruction, AssemblyError> {
        let expr = &self.expression;
        sysop(expr).and_then(|symbol| {
            if symbol == '+' {
                self.ok_none(mnemonic_plus.clone(), Mode::Implied)
            } else if symbol == '-' {
                self.ok_none(mnemonic_minus.clone(), Mode::Implied)
            } else {
                self.decode_error()
            }
        })
    }

    fn decode_x(
        &self,
        labels: &HashMap<String, LabelEntry>,
    ) -> Result<AssemblyInstruction, AssemblyError> {
        self.immediate(Mnemonic::LDX)
            .or_else(|_| self.zeropage(Mnemonic::LDX, labels))
            .or_else(|_| self.incr_decrement(Mnemonic::INX, Mnemonic::DEX, "X"))
            .or_else(|_| self.decode_error())
    }

    fn decode_y(
        &self,
        labels: &HashMap<String, LabelEntry>,
    ) -> Result<AssemblyInstruction, AssemblyError> {
        self.zeropage(Mnemonic::LDY, labels).or_else(|_| {
            let expr = &self.expression;
            if let Expr::ByteNum(num) = *expr {
                return self.ok_byte(&Mnemonic::LDY, Mode::Immediate, num);
            }
            if let Expr::DecimalNum(num) = *expr {
                if num > 255 {
                    return Err(AssemblyError::syntax("operand must be 8bit"));
                }
                return self.ok_byte(&Mnemonic::LDY, Mode::Immediate, num as u8);
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
        })
    }

    fn decode_a(
        &self,
        labels: &HashMap<String, LabelEntry>,
    ) -> Result<AssemblyInstruction, AssemblyError> {
        let expr = &self.expression;
        if let Expr::DecimalNum(num) = *expr {
            if num > 255 {
                return Err(AssemblyError::syntax("operand must be 8bit"));
            } else {
                return self.ok_byte(&Mnemonic::LDA, Mode::Immediate, num as u8);
            }
        }
        if let Expr::ByteNum(num) = *expr {
            return self.ok_byte(&Mnemonic::LDA, Mode::Immediate, num);
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
                            return self.ok_byte(&Mnemonic::SBC, Mode::Immediate, num as u8);
                        }
                    }
                    if let Expr::ByteNum(num) = **expr {
                        return self.ok_byte(&Mnemonic::SBC, Mode::Immediate, num);
                    }
                }
                // A=A+$48
                if name == "A" && *operator == Operator::Add {
                    let expr = right;
                    if let Expr::DecimalNum(num) = **expr {
                        if num < 256 {
                            return self.ok_byte(&Mnemonic::ADC, Mode::Immediate, num as u8);
                        }
                    }
                    if let Expr::ByteNum(num) = **expr {
                        return self.ok_byte(&Mnemonic::ADC, Mode::Immediate, num);
                    }
                }
            }
        }
        if let Expr::Identifier(name) = &self.expression {
            if name == "X" {
                return self.ok_none(Mnemonic::TXA, Mode::Implied);
            } else if name == "Y" {
                return self.ok_none(Mnemonic::TYA, Mode::Implied);
            }
        }
        // A=(label) => LDA label
        if let Expr::Parenthesized(expr) = &self.expression {
            if let Expr::Identifier(ref name) = **expr {
                let entry = labels
                    .get(name)
                    .ok_or(AssemblyError::syntax("unknown label"))?;
                match entry.address {
                    Address::Full(addr) => {
                        return self.ok_word(&Mnemonic::LDA, Mode::Absolute, addr);
                    }
                    Address::ZeroPage(addr) => {
                        return self.ok_byte(&Mnemonic::LDA, Mode::ZeroPage, addr);
                    }
                }
            }
        }
        self.decode_error()
    }

    fn decode_t(
        &self,
        labels: &HashMap<String, LabelEntry>,
    ) -> Result<AssemblyInstruction, AssemblyError> {
        if let Expr::BinOp(left, operator, right) = &self.expression {
            if let Expr::Identifier(ref reg) = **left {
                if *operator == Operator::Sub {
                    let expr = right;
                    if let Expr::DecimalNum(num) = **expr {
                        if num < 256 {
                            if reg == "A" {
                                return self.ok_byte(&Mnemonic::CMP, Mode::Immediate, num as u8);
                            }
                            if reg == "X" {
                                return self.ok_byte(&Mnemonic::CPX, Mode::Immediate, num as u8);
                            }
                            if reg == "Y" {
                                return self.ok_byte(&Mnemonic::CPY, Mode::Immediate, num as u8);
                            }
                        }
                    }
                    if let Expr::ByteNum(num) = **expr {
                        if reg == "A" {
                            return self.ok_byte(&Mnemonic::CMP, Mode::Immediate, num);
                        }
                        if reg == "X" {
                            return self.ok_byte(&Mnemonic::CPX, Mode::Immediate, num);
                        }
                        if reg == "Y" {
                            return self.ok_byte(&Mnemonic::CPY, Mode::Immediate, num);
                        }
                    }
                }
            }
        }
        self.decode_error()
    }

    fn decode_c(
        &self,
        labels: &HashMap<String, LabelEntry>,
    ) -> Result<AssemblyInstruction, AssemblyError> {
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

    fn decode_call(
        &self,
        labels: &HashMap<String, LabelEntry>,
    ) -> Result<AssemblyInstruction, AssemblyError> {
        if let Expr::Identifier(name) = &self.expression {
            return self.ok_unresolved_label(Mnemonic::JSR, Mode::Absolute, &name);
        }
        if let Expr::WordNum(num) = self.expression {
            return self.ok_word(&Mnemonic::JSR, Mode::Absolute, num);
        }
        self.decode_error()
    }

    fn decode_goto(
        &self,
        labels: &HashMap<String, LabelEntry>,
    ) -> Result<AssemblyInstruction, AssemblyError> {
        match &self.expression {
            Expr::WordNum(num) => {
                return self.ok_word(&Mnemonic::JMP, Mode::Absolute, *num);
            }
            Expr::Identifier(name) => {
                return self.ok_unresolved_label(Mnemonic::JMP, Mode::Absolute, &name);
            }
            Expr::SystemOperator('!') => return self.ok_none(Mnemonic::RTS, Mode::Implied),
            _ => return self.decode_error(),
        }
    }

    fn decode_if(
        &self,
        labels: &HashMap<String, LabelEntry>,
    ) -> Result<AssemblyInstruction, AssemblyError> {
        // ;=/,$12fd (IF NOT EQUAL THEN GOTO $12FD)
        if let Expr::BinOp(expr_left, Operator::Comma, expr_right) = &self.expression {
            if let Expr::SystemOperator(symbol) = **expr_left {
                match **expr_right {
                    Expr::WordNum(addr) => match symbol {
                        '\\' => {
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

    fn decode_other(
        &self,
        labels: &HashMap<String, LabelEntry>,
    ) -> Result<AssemblyInstruction, AssemblyError> {
        if let Expr::Parenthesized(expr) = &self.command {
            match **expr {
                Expr::Identifier(ref name) => {
                    let entry = labels
                        .get(name)
                        .ok_or(AssemblyError::syntax("unknown label"))?;
                    match entry.address {
                        Address::Full(addr) => {
                            return self.ok_word(&Mnemonic::STA, Mode::Absolute, addr);
                        }
                        Address::ZeroPage(addr) => {
                            return self.ok_byte(&Mnemonic::STA, Mode::ZeroPage, addr);
                        }
                    }
                }
                Expr::WordNum(num) => {
                    return self.ok_word(&Mnemonic::STA, Mode::Absolute, num);
                }
                Expr::ByteNum(num) => {
                    return self.ok_byte(&Mnemonic::STA, Mode::ZeroPage, num);
                }
                _ => return self.decode_error(),
            }
        }
        dbg!(self);
        todo!()
    }

    fn ok_byte(
        &self,
        mnemonic: &Mnemonic,
        mode: Mode,
        num: u8,
    ) -> Result<AssemblyInstruction, AssemblyError> {
        Ok(AssemblyInstruction::new(
            mnemonic.clone(),
            mode,
            OperandValue::Byte(num),
        ))
    }

    fn ok_word(
        &self,
        mnemonic: &Mnemonic,
        mode: Mode,
        num: u16,
    ) -> Result<AssemblyInstruction, AssemblyError> {
        Ok(AssemblyInstruction::new(
            mnemonic.clone(),
            mode,
            OperandValue::Word(num),
        ))
    }

    fn instruction(
        &self,
        mnemonic: Mnemonic,
        mode: Mode,
        value: OperandValue,
    ) -> AssemblyInstruction {
        AssemblyInstruction::new(mnemonic, mode, value)
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
