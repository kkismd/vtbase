use crate::error::AssemblyError;

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

    pub fn compile(&self) -> Result<Vec<u8>, AssemblyError> {
        match self.command.as_str() {
            "*" => self.compile_origin(),
            ":" => self.compile_label_def(),
            "?" => self.compile_string_def(),
            "#" => self.compile_goto(),
            "!" => self.compile_gosub(),
            ";" => self.compile_if(),
            "X" => self.compile_X(),
            "A" => self.compile_A(),
            "P" => self.compile_P(),
            &_ => todo!(),
        }
    }

    fn compile_origin(&self) -> Result<Vec<u8>, AssemblyError> {
        todo!()
    }

    fn compile_label_def(&self) -> Result<Vec<u8>, AssemblyError> {
        todo!()
    }

    fn compile_string_def(&self) -> Result<Vec<u8>, AssemblyError> {
        todo!()
    }

    fn compile_goto(&self) -> Result<Vec<u8>, AssemblyError> {
        todo!()
    }

    fn compile_gosub(&self) -> Result<Vec<u8>, AssemblyError> {
        todo!()
    }

    fn compile_if(&self) -> Result<Vec<u8>, AssemblyError> {
        todo!()
    }

    fn compile_X(&self) -> Result<Vec<u8>, AssemblyError> {
        todo!()
    }

    fn compile_A(&self) -> Result<Vec<u8>, AssemblyError> {
        todo!()
    }

    fn compile_P(&self) -> Result<Vec<u8>, AssemblyError> {
        todo!()
    }
}
