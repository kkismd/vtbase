use nom::Err;

use crate::error::AssemblyError;
use crate::opcode::{Mnemonic, Mode, Opcode, OpcodeTable};
use crate::parser::expression::Expr;

use super::expression::Operator;

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
            let details = format!("unknown command: <{}>", self.command);
            Err(AssemblyError::syntax(&details))
        }
    }

    /**
     * decode command and expression into mnemonic and addressing mode
     * @return (Mnemonic, Mode)
     */
    pub fn decode(&self) -> Result<(Mnemonic, Mode), AssemblyError> {
        match self.command.as_str() {
            "X" => self.decode_x(),
            "A" => self.decode_a(),
            "T" => self.decode_t(),
            "!" => self.decode_call(),
            "#" => self.decode_goto(),
            ";" => self.decode_if(),
            _ => todo!(),
        }
    }

    fn decode_x(&self) -> Result<(Mnemonic, Mode), AssemblyError> {
        if let Expr::Immediate(expr) = &self.expression {
            if let Expr::ByteNum(_) = **expr {
                return Ok((Mnemonic::LDX, Mode::Immediate));
            }
            if let Expr::DecimalNum(num) = **expr {
                if num > 255 {
                    return Err(AssemblyError::syntax("operand must be 8bit"));
                }
                return Ok((Mnemonic::LDX, Mode::Immediate));
            }
            if let Expr::ByteNum(_) = self.expression {
                return Ok((Mnemonic::LDX, Mode::ZeroPage));
            }
        }
        if let Expr::BinOp(left, operator, right) = &self.expression {
            if let Expr::Identifier(ref s) = **left {
                if *operator == Operator::Add {
                    if let Expr::DecimalNum(n) = **right {
                        if s == "X" && n == 1 {
                            return Ok((Mnemonic::INX, Mode::Implied));
                        }
                    }
                }
            }
        }
        self.decode_error()
    }
    fn decode_a(&self) -> Result<(Mnemonic, Mode), AssemblyError> {
        if let Expr::Immediate(expr) = &self.expression {
            if let Expr::ByteNum(_) = **expr {
                return Ok((Mnemonic::LDA, Mode::Immediate));
            }
            if let Expr::DecimalNum(num) = **expr {
                if num > 255 {
                    return Err(AssemblyError::syntax("operand must be 8bit"));
                }
                return Ok((Mnemonic::LDA, Mode::Immediate));
            }
            if let Expr::ByteNum(_) = self.expression {
                return Ok((Mnemonic::LDA, Mode::ZeroPage));
            }
        }
        if let Expr::BinOp(left, operator, right) = &self.expression {
            if let Expr::Identifier(_) = **left {
                if *operator == Operator::Add {
                    if let Expr::Identifier(ref reg) = **right {
                        if reg == "X" || reg == "Y" {
                            return Ok((Mnemonic::LDA, Mode::AbsoluteX));
                        }
                    }
                }
            }
        }
        self.decode_error()
    }

    fn decode_t(&self) -> Result<(Mnemonic, Mode), AssemblyError> {
        if let Expr::BinOp(left, operator, right) = &self.expression {
            if let Expr::Identifier(ref reg) = **left {
                if *operator == Operator::Sub {
                    if let Expr::Immediate(_) = **right {
                        if reg == "X" {
                            return Ok((Mnemonic::CPX, Mode::Immediate));
                        }
                    }
                }
            }
        }
        self.decode_error()
    }

    fn decode_call(&self) -> Result<(Mnemonic, Mode), AssemblyError> {
        if let Expr::Identifier(_) = self.expression {
            return Ok((Mnemonic::JSR, Mode::Absolute));
        }
        if let Expr::WordNum(_) = self.expression {
            return Ok((Mnemonic::JSR, Mode::Absolute));
        }
        self.decode_error()
    }

    fn decode_goto(&self) -> Result<(Mnemonic, Mode), AssemblyError> {
        match self.expression {
            Expr::Identifier(_) | Expr::WordNum(_) | Expr::Identifier(_) => {
                return Ok((Mnemonic::JMP, Mode::Absolute))
            }
            Expr::SystemOperator('!') => return Ok((Mnemonic::RTS, Mode::Implied)),
            _ => return self.decode_error(),
        }
    }

    fn decode_if(&self) -> Result<(Mnemonic, Mode), AssemblyError> {
        if let Expr::SystemOperator(ref symbol) = self.expression {
            match symbol {
                '/' => return Ok((Mnemonic::BNE, Mode::Relative)),
                '=' => return Ok((Mnemonic::BEQ, Mode::Relative)),
                '>' => return Ok((Mnemonic::BCS, Mode::Relative)),
                '<' => return Ok((Mnemonic::BCC, Mode::Relative)),
                _ => (),
            }
        }
        self.decode_error()
    }

    fn decode_error(&self) -> Result<(Mnemonic, Mode), AssemblyError> {
        Err(AssemblyError::syntax(&format!(
            "bad expression: {:?}",
            self
        )))
    }

    /**
     * compile pseudo command
     * @return Vec<u8> assembled code
     */
    pub fn compile(&self, opcode_table: &OpcodeTable) -> Result<Vec<u8>, AssemblyError> {
        let (mnemonic, mode) = self.decode()?;
        // find opcode from mnemonic and mode
        let opcode = opcode_table.find(mnemonic, &mode)?;
        let operand = mode.bytes(&self.expression);
        let mut bytes = vec![];
        bytes.push(opcode.opcode);
        bytes.extend(&operand);
        Ok(bytes)
    }
}
