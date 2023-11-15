use crate::parser::statement;
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
                        self.add_global_label(&label, instruction.line_number);
                    } else {
                        return Err(AssemblyError::label_used(instruction.line_number, &label));
                    }
                }
            }
        }
        Ok(())
    }

    /**
     */
    fn assemble_pass2(&mut self, instructions: &mut Vec<Instruction>) -> Result<(), AssemblyError> {
        for mut instruction in instructions {
            if let Some(name) = &instruction.label {
                if let Some(label_entry) = self.labels.get_mut(name) {
                    label_entry.address = Some(self.pc);
                }
            }

            for mut statement in &instruction.statements {
                let objects = self.process_statement(statement)?;
                instruction.object_codes.extend(objects);
            }
            match <usize as TryInto<u16>>::try_into(instruction.object_codes.len()) {
                Ok(v) => self.pc += v,
                Err(_) => return Err(AssemblyError::program("object code memory size overflow.")),
            }
        }

        Ok(())
    }

    fn process_statement(
        &self,
        statement: &statement::Statement,
    ) -> Result<Vec<u8>, AssemblyError> {
        // TODO: ここで1命令ごとの処理を書く
        statement.compile()
    }

    fn add_global_label(&mut self, name: &str, line: usize) {
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
