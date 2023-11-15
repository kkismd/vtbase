use crate::{error::AssemblyError, parser::Instruction};
use std::collections::HashMap;

// assembler.rs
pub struct Assembler {
    pub origin: u16,
    pub pc: u16,
    pub labels: HashMap<String, Label>,
}

pub struct Label {
    pub name: String,
    pub address: Option<u16>,
    pub local_labels: HashMap<String, Label>,
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
                if !self.labels.contains_key(label) {
                    self.entry_label(&label);
                } else {
                    return Err(AssemblyError::label_used(instruction.line_number, &label));
                }
            }
        }
        Ok(())
    }

    /**
     */
    fn assemble_pass2(&self, instructions: &mut Vec<Instruction>) -> Result<(), AssemblyError> {
        /* 1. ORG命令の処理
         * ORG命令がくるまでの間にEQL命令以外が来たらエラーとする
         * ORG命令までのEQL命令のアドレスは0のままとする
         */
        for mut Instruction in instructions {
            // TODO: ここで1行ごとの処理を書く
            for statement in &Instruction.statements {
                // TODO: ここで1命令ごとの処理を書く
            }
            Instruction.object_codes.push(0);
        }

        Ok(())
    }

    pub fn entry_label(&mut self, name: &str) {
        self.labels.insert(
            name.to_string(),
            Label {
                name: name.to_string(),
                address: None,
                local_labels: HashMap::new(),
            },
        );
    }
}
