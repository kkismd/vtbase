use super::expression::Operator;
use crate::assembler::{Address, LabelTable};
use crate::error::AssemblyError;
use crate::opcode::{AddressingMode, AssemblyInstruction, OpcodeTable, OperandValue};
use crate::parser::expression::Expr;
pub mod decoder;
use decoder::*;

// statement in a line of source code
#[derive(Debug, Clone)]
pub struct Statement {
    pub command: Expr,
    pub expression: Expr,
}
impl Statement {
    pub fn new(command: &str, expression: Expr) -> Self {
        let command = if command.chars().all(|c| c.is_alphanumeric()) {
            Expr::Identifier(command.to_string())
        } else {
            Expr::SystemOperator(command.to_string())
        };
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
            return ["*", ":", "?", "$", "&"].contains(&command.as_str());
        }
        false
    }

    // ;=記号,式 の場合はマクロではない
    // それ以外の二項演算子の場合はマクロ
    pub fn check_macro_if_statement(&self) -> bool {
        match &self.expression {
            Expr::BinOp(_, Operator::Comma, _) => false,
            _ => true,
        }
    }

    pub fn decode(&self, labels: &LabelTable) -> Result<AssemblyInstruction, AssemblyError> {
        let expr = &self.expression;
        match &self.command {
            Expr::Identifier(sym) if sym == "X" => decode_x(&expr, labels),
            Expr::Identifier(sym) if sym == "Y" => decode_y(&expr, labels),
            Expr::Identifier(sym) if sym == "A" => decode_a(&expr, labels),
            Expr::Identifier(sym) if sym == "T" => decode_t(&expr, labels),
            Expr::Identifier(sym) if "CIVD".contains(sym) => {
                decode_flags(&self.command, &expr, labels)
            }
            Expr::Identifier(sym) if sym == "S" => decode_stack(&expr, labels),
            Expr::Identifier(sym) if sym == "_" => decode_nop(&expr),
            Expr::SystemOperator(sym) if sym == "!" => decode_call(&expr, labels),
            Expr::SystemOperator(sym) if sym == "#" => decode_goto(&expr, labels),
            Expr::SystemOperator(sym) if sym == ";" => decode_if(&expr, labels),
            Expr::SystemOperator(sym) if "<>()".contains(sym) => {
                decode_shift(&self.command, &expr, labels)
            }
            Expr::SystemOperator(sym) if sym == "[" => decode_push(&expr),
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
        labels: &LabelTable,
        current_label: &str,
        pc: usize,
    ) -> Result<Vec<u8>, AssemblyError> {
        let assembly_instruction = self.decode(labels)?;
        // find opcode from mnemonic and mode
        let opcode = opcode_table.find(
            &assembly_instruction.mnemonic,
            &assembly_instruction.addressing_mode,
        )?;
        let operand =
            self.operand_bytes(&assembly_instruction, labels, current_label, pc as u16)?;

        let mut bytes = vec![];
        bytes.push(opcode.opcode);
        bytes.extend(&operand);
        Ok(bytes)
    }

    fn operand_bytes(
        &self,
        assembly_instruction: &AssemblyInstruction,
        labels: &LabelTable,
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
        labels: &LabelTable,
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

#[cfg(test)]
mod tests {

    use crate::assembler::LabelEntry;
    use crate::opcode::AssemblyInstruction;
    use crate::opcode::Mnemonic;
    use crate::parser::parse_token;

    use super::*;

    #[test]
    fn test_address_absolute_x_0x0000() {
        let labels = LabelTable::new();
        let expr = Expr::Parenthesized(Box::new(Expr::BinOp(
            Box::new(Expr::WordNum(0x0000)),
            Operator::Add,
            Box::new(Expr::Identifier("X".to_string())),
        )));
        let statement = Statement {
            command: expr,
            expression: Expr::Identifier("A".to_string()),
        };
        let instruction = statement.decode(&labels).unwrap();
        assert_eq!(
            instruction,
            AssemblyInstruction {
                mnemonic: Mnemonic::STA,
                addressing_mode: AddressingMode::AbsoluteX,
                value: OperandValue::Word(0x0000),
            }
        );
    }

    #[test]
    fn test_cmp_immediate() {
        let statement = parse_token("T=A-2").unwrap();
        let labels = LabelTable::new();
        let instruction = statement.decode(&labels).unwrap();
        assert_eq!(
            instruction,
            AssemblyInstruction {
                mnemonic: Mnemonic::CMP,
                addressing_mode: AddressingMode::Immediate,
                value: OperandValue::Byte(0x02),
            }
        );
    }

    #[test]
    fn test_sta_absolute_x() {
        let statement = parse_token("($0000+X)=A").unwrap();
        let labels = LabelTable::new();
        let instruction = statement.decode(&labels).unwrap();
        assert_eq!(
            instruction,
            AssemblyInstruction {
                mnemonic: Mnemonic::STA,
                addressing_mode: AddressingMode::AbsoluteX,
                value: OperandValue::Word(0x0000),
            }
        );
    }

    #[test]
    fn test_lda_absolute_x_label() {
        let statement = parse_token("A=(palette+X)").unwrap();
        let mut labels = LabelTable::new();
        let label_str = "palette";
        let entry = LabelEntry {
            name: label_str.to_string(),
            line: 0,
            address: Address::Full(0x0400),
        };
        labels.insert(label_str.to_string(), entry);
        let instruction = statement.decode(&labels).unwrap();
        assert_eq!(
            instruction,
            AssemblyInstruction {
                mnemonic: Mnemonic::LDA,
                addressing_mode: AddressingMode::AbsoluteX,
                value: OperandValue::Word(0x0400),
            }
        );
    }
}
