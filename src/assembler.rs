use crate::parser::Instruction;
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
    pub fn assemble(&mut self, instructions: Vec<Instruction>) -> Vec<Instruction> {
        self.create_label_entries(&instructions);
        let pass2_instructions = self.assemble_pass2(instructions);
        pass2_instructions
    }

    // ラベルの名前をラベルテーブルに登録する
    fn create_label_entries(&mut self, instructions: &Vec<Instruction>) {
        for instruction in instructions {
            if let Some(label) = &instruction.label {
                self.entry_label(&label);
            }
        }
    }

    fn assemble_pass2(&self, instructions: Vec<Instruction>) -> Vec<Instruction> {
        // TODO: implement
        instructions
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
