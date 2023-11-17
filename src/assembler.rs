use crate::parser::expression::Expr;
use crate::parser::statement::Statement;
use crate::{error::AssemblyError, parser::Instruction};
use std::collections::HashMap;

// assembler.rs
pub struct Assembler {
    pub origin: u16,
    pub pc: u16,
    pub labels: HashMap<String, LabelEntry>,
}

pub struct LabelEntry {
    pub name: String,
    pub line: usize,
    pub address: Option<u16>,
}

impl Assembler {
    pub fn new() -> Self {
        Self {
            origin: 0,
            pc: 0,
            labels: HashMap::new(),
        }
    }
    pub fn assemble(&mut self, instructions: &mut Vec<Instruction>) -> Result<(), AssemblyError> {
        self.entry_labels(&instructions)?;
        self.assemble_pass2(instructions)?;
        Ok(())
    }

    // ラベルの名前をラベルテーブルに登録する
    fn entry_labels(&mut self, instructions: &Vec<Instruction>) -> Result<(), AssemblyError> {
        for instruction in instructions {
            if let Some(label) = &instruction.label {
                if label.starts_with(".") {
                    // local label
                    // TODO: グローバルラベル.ローカルラベル という名前で登録・参照する
                } else {
                    // global label
                    if !self.labels.contains_key(label) {
                        self.add_label(&label, instruction.line_number);
                    } else {
                        return Err(AssemblyError::label_used(instruction.line_number, &label));
                    }
                }
            }
        }
        Ok(())
    }

    fn assemble_pass2(&mut self, instructions: &mut Vec<Instruction>) -> Result<(), AssemblyError> {
        for instruction in instructions {
            if let Some(name) = &instruction.label {
                if let Some(label_entry) = self.labels.get_mut(name) {
                    label_entry.address = Some(self.pc);
                }
            }

            for statement in &instruction.statements {
                statement.validate()?;
                if statement.is_pseude() {
                    self.do_pseudo_command(instruction, statement)?;
                } else {
                    let objects = statement.compile()?;
                    instruction.object_codes.extend(objects);
                }
            }
            match <usize as TryInto<u16>>::try_into(instruction.object_codes.len()) {
                Ok(v) => self.pc += v,
                Err(_) => return Err(AssemblyError::program("object code memory size overflow.")),
            }
        }

        Ok(())
    }

    fn do_pseudo_command(
        &mut self,
        instruction: &Instruction,
        statement: &Statement,
    ) -> Result<(), AssemblyError> {
        if statement.command == "*" {
            // set stgart address
            if let Expr::WordNum(address) = statement.expression {
                self.origin = address;
            }
            Ok(())
        } else if statement.command == ":" {
            // label def
            let label_name = instruction
                .label
                .clone()
                .ok_or(AssemblyError::program("label def need label"))?;
            // fetch label entry
            let label_entry = self
                .labels
                .get_mut(&label_name)
                .ok_or(AssemblyError::program("label not found"))?;
            // set address to entry
            if let Expr::WordNum(address) = statement.expression {
                label_entry.address = Some(address);
            }
            Ok(())
        } else {
            Ok(())
        }
    }

    fn add_label(&mut self, name: &str, line: usize) {
        self.labels.insert(
            name.to_string(),
            LabelEntry {
                name: name.to_string(),
                line: line,
                address: None,
            },
        );
    }
}
