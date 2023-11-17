use nom::Err;

use crate::error::AssemblyError;
use crate::opcode::{Mnemonic, Mode, Opcode};
use crate::parser::expression::Expr;

// instruction in a line of source code
#[derive(Debug)]
pub struct Statement {
    pub(crate) command: String,
    pub(crate) expression: Expr,
}

impl Statement {
    pub fn new(command: String, expression: Expr) -> Self {
        Self {
            command,
            expression,
        }
    }

    pub fn is_pseudo(&self) -> bool {
        let pseudo_commands = vec!["*", ":", "?"];
        pseudo_commands.iter().any(|&pc| pc == self.command)
    }

    pub fn validate_pseudo_command(&self) -> Result<(), AssemblyError> {
        if self.command == "*" || self.command == ":" {
            match self.expression {
                Expr::WordNum(_) => Ok(()),
                _ => Err(AssemblyError::syntax("operand must be 16bit address")),
            }
        } else if self.command == "?" {
            match self.expression {
                Expr::StringLiteral(_) => Ok(()),
                _ => Err(AssemblyError::syntax("operand must be string")),
            }
        } else {
            Ok(())
            // TODO: need more command check
            // let details = format!("unknown command: <{}>", self.command);
            // Err(AssemblyError::syntax(&details))
        }
    }

    /**
     * decode command and expression into mnemonic and addressing mode
     * @return (Mnemonic, Mode)
     */
    pub fn decode(&self) -> Result<(Mnemonic, Mode), AssemblyError> {
        match self.command.as_str() {
            "X" => self.decode_x(),
            _ => todo!(),
        }
    }

    fn decode_x(&self) -> Result<(Mnemonic, Mode), AssemblyError> {
        if let Expr::Immediate(expr) = self.expression {
            if let Expr::ByteNum(_) = *expr {
                return Ok((Mnemonic::LDX, Mode::Immediate));
            }
            if let Expr::DecimalNum(num) = *expr {
                if num > 255 {
                    return Err(AssemblyError::syntax("operand must be 8bit"));
                }
                return Ok((Mnemonic::LDX, Mode::Immediate));
            }
            if let Expr::ByteNum(_) = self.expression {
                return Ok((Mnemonic::LDX, Mode::ZeroPage));
            }
        }
        Err(AssemblyError::syntax(
            "nomatch command and expression: {:?}",
        ))
    }

    /**
     * compile pseudo command
     * @return Vec<u8> assembled code
     */
    pub fn compile(&self, opcode_table: Vec<Opcode>) -> Result<Vec<u8>, AssemblyError> {
        let (mnemonic, mode) = self.decode()?;
        // find opcode from mnemonic and mode
        let opcode = Opcode::find(opcode_table, mnemonic, mode)?;
        opcode.Ok(vec![])
    }
}
