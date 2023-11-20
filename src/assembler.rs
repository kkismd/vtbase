use crate::opcode;
use crate::parser::expression::Expr;
use crate::parser::statement::{self, Statement};
use crate::{error::AssemblyError, parser::Instruction};
use std::collections::HashMap;

// assembler.rs
pub struct Assembler {
    pub origin: u16,
    pub pc: u16,
    pub labels: HashMap<String, LabelEntry>,
    pub opcode_table: opcode::OpcodeTable,
    pub current_label: String,
}

#[derive(Debug)]
pub struct LabelEntry {
    pub name: String,
    pub line: usize,
    pub address: u16,
}

impl Assembler {
    pub fn new() -> Self {
        Self {
            origin: 0,
            pc: 0,
            labels: HashMap::new(),
            opcode_table: opcode::OpcodeTable::new(),
            current_label: String::new(),
        }
    }

    pub fn assemble(&mut self, instructions: &mut Vec<Instruction>) -> Result<(), AssemblyError> {
        self.assemble_pass1(instructions)?;
        self.assemble_pass2(instructions)?;
        Ok(())
    }

    fn assemble_pass1(&mut self, instructions: &mut Vec<Instruction>) -> Result<(), AssemblyError> {
        for instruction in instructions {
            eprintln!("pc = {:04x}, statement = {:?}", self.pc, instruction);
            instruction.address = self.pc;
            self.entry_label(instruction)?;
            for statement in &instruction.statements {
                if statement.is_pseudo() {
                    statement.validate_pseudo_command()?;
                    self.do_pseudo_command(instruction, statement)?;
                    continue;
                }
                let assymbly_instruction = statement.decode()?;
                let len = assymbly_instruction.addressing_mode.length();
                self.pc += len as u16;
            }
        }
        Ok(())
    }

    /**
     * entry label to label table
     *  - if start with ".", treat as local label
     *  - if label is global, set current_label
     */
    fn entry_label(&mut self, instruction: &mut Instruction) -> Result<(), AssemblyError> {
        if let Some(label) = &instruction.label {
            if label.starts_with(".") {
                // local label
                if self.current_label == "" {
                    return Err(AssemblyError::program("global label not found"));
                }
                let label = format!("{}{}", self.current_label, label);
                self.add_entry(&label, instruction)?;
            } else {
                // global label
                self.add_entry(label, instruction)?;
                self.current_label = label.to_string();
            }
            if let Some(entry) = self.labels.get_mut(label) {
                entry.address = self.pc;
                eprintln!("label_entry = {:?}", entry);
            }
        }
        Ok(())
    }

    fn add_entry(&mut self, label: &str, instruction: &Instruction) -> Result<(), AssemblyError> {
        if !self.labels.contains_key(label) {
            self.add_label(&label, instruction.line_number, self.pc);
            Ok(())
        } else {
            return Err(AssemblyError::label_used(instruction.line_number, &label));
        }
    }

    fn assemble_pass2(&mut self, instructions: &mut Vec<Instruction>) -> Result<(), AssemblyError> {
        for mut instruction in instructions {
            let mut pc = instruction.address;
            for mut statement in &instruction.statements {
                if !statement.is_pseudo() {
                    let objects = statement.compile(&self.opcode_table, &self.labels, pc)?;
                    let dump = Self::dump_objects(&objects);
                    eprintln!("{}\t{:?}", dump, statement);
                    pc += objects.len() as u16;
                    instruction.object_codes.extend(objects);
                }
            }
        }

        Ok(())
    }

    fn dump_objects(objects: &Vec<u8>) -> String {
        objects
            .iter()
            .map(|n| format!("{:02X}", n))
            .collect::<Vec<String>>()
            .join(",")
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
                self.pc = address;
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
                label_entry.address = address;
                eprintln!("label def: {:?}", label_entry);
            }
            Ok(())
        } else if statement.command == "?" {
            if let Expr::StringLiteral(ref s) = statement.expression {
                let len = s.len();
                self.pc += len as u16;
            }
            Ok(())
        } else {
            Ok(())
        }
    }

    fn add_label(&mut self, name: &str, line: usize, address: u16) {
        let entry = LabelEntry {
            name: name.to_string(),
            line: line,
            address: address,
        };
        self.labels.insert(name.to_string(), entry);
    }
}
