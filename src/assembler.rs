pub mod pseudo_commands;

use crate::opcode;
use crate::parser::expression::Operator;
use crate::parser::statement::Statement;
use crate::{error::AssemblyError, parser::Instruction};
use std::collections::HashMap;

pub struct Assembler {
    pub origin: u16,
    pub pc: u16,
    pub labels: LabelTable,
    pub opcode_table: opcode::OpcodeTable,
    pub current_label: String,
}

#[derive(Debug, Clone)]
pub struct LabelEntry {
    pub name: String,
    pub line: usize,
    pub address: Address,
}

pub type LabelTable = HashMap<String, LabelEntry>;

#[derive(Debug, Clone)]
pub enum Address {
    Full(u16),
    ZeroPage(u8),
}

impl Address {
    pub fn calculate_with(&self, other: &Address, op: &Operator) -> Result<Self, AssemblyError> {
        match self {
            Address::Full(n) => {
                if let Address::Full(m) = other {
                    Ok(Address::Full(match op {
                        Operator::Add => n + m,
                        Operator::Sub => n - m,
                        Operator::Mul => n * m,
                        Operator::Div => n / m,
                        Operator::And => n & m,
                        Operator::Or => n | m,
                        Operator::Xor => n ^ m,
                        _ => return self.type_error(other, &op.to_string()),
                    }))
                } else if let Address::ZeroPage(m) = other {
                    // FullとZeroPageの演算は、Fullとして計算する
                    Ok(Address::Full(match op {
                        Operator::Add => n + (*m as u16),
                        Operator::Sub => n - (*m as u16),
                        Operator::Mul => n * (*m as u16),
                        Operator::Div => n / (*m as u16),
                        Operator::And => n & (*m as u16),
                        Operator::Or => n | (*m as u16),
                        Operator::Xor => n ^ (*m as u16),
                        _ => return self.type_error(other, &op.to_string()),
                    }))
                } else {
                    self.type_error(&other, &op.to_string())
                }
            }
            Address::ZeroPage(n) => {
                if let Address::ZeroPage(m) = other {
                    Ok(Address::ZeroPage(match op {
                        Operator::Add => n + m,
                        Operator::Sub => n - m,
                        Operator::Mul => n * m,
                        Operator::Div => n / m,
                        Operator::And => n & m,
                        Operator::Or => n | m,
                        Operator::Xor => n ^ m,
                        _ => return self.type_error(other, &op.to_string()),
                    }))
                } else {
                    self.type_error(&other, &op.to_string())
                }
            }
        }
    }

    fn type_error(&self, other: &Address, op: &str) -> Result<Self, AssemblyError> {
        Err(AssemblyError::expression(&format!(
            "cannot {} {:?} and {:?}",
            op, self, other
        )))
    }
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

    pub fn assemble(&mut self, instructions: &mut Vec<Instruction>) -> Result<u16, AssemblyError> {
        self.pass1(instructions)?;
        let obj_size = self.pass2(instructions)?;
        Ok(obj_size)
    }

    fn pass1(&mut self, instructions: &mut Vec<Instruction>) -> Result<(), AssemblyError> {
        for instruction in instructions {
            // eprintln!("pc = {:04x}, statement = {:?}", self.pc, instruction);
            self.pass1_instruction(instruction).map_err(|e| {
                eprintln!("line = {:?}, error = {:?}", instruction, e.message());
                e
            })?
        }
        Ok(())
    }

    fn pass1_instruction(&mut self, instruction: &mut Instruction) -> Result<(), AssemblyError> {
        instruction.address = self.pc;
        self.entry_label(instruction)?;
        Ok(for statement in &instruction.statements {
            if statement.is_pseudo() {
                self.pseudo_command_pass1(instruction, statement)?;
                continue;
            }
            let assembly_instruction = statement.decode(&self.labels)?;
            let len = assembly_instruction.addressing_mode.length();
            self.pc += len as u16;
        })
    }

    fn pass2(&mut self, instructions: &mut Vec<Instruction>) -> Result<u16, AssemblyError> {
        self.current_label = String::new();
        let mut objects_size = 0;
        for instruction in instructions {
            let mut pc = instruction.address;
            self.track_global_label(instruction);
            for statement in &instruction.statements {
                let objects;
                if statement.is_pseudo() {
                    objects = self.pseudo_command_pass2(statement)?;
                } else {
                    objects = statement.compile(
                        &self.opcode_table,
                        &self.labels,
                        &self.current_label,
                        pc,
                    )?;
                }
                // let dump = Self::dump_objects(&objects);
                // eprintln!("{}\t{:?}", dump, statement);
                pc += objects.len() as u16;
                objects_size += objects.len() as u16;
                instruction.object_codes.extend(objects);
            }
        }
        Ok(objects_size)
    }

    fn track_global_label(&mut self, instruction: &mut Instruction) {
        if let Some(label) = &instruction.label {
            let first_char = label.chars().next().unwrap();
            if first_char != '.' && first_char != '#' {
                self.current_label = label.to_string();
            }
        }
    }

    /**
     * entry label to label table
     *  - if start with ".", treat as local label
     *  - if label is global, set current_label
     */
    fn entry_label(&mut self, instruction: &mut Instruction) -> Result<(), AssemblyError> {
        if let Some(label) = &instruction.label {
            let mut label = label.clone();
            if label.starts_with(".") {
                // local label
                if self.current_label == "" {
                    return Err(AssemblyError::program("global label not found"));
                }
                label = format!("{}{}", self.current_label, label);
                self.add_entry(&label, instruction)?;
            } else if label.starts_with("#macro_") {
                // macro label -- don't memorize current label
                self.add_entry(&label, instruction)?;
            } else {
                // global label
                self.add_entry(&label, instruction)?;
                self.current_label = label.to_string();
            }
            if let Some(entry) = self.labels.get_mut(&label) {
                entry.address = Address::Full(self.pc);
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

    fn _dump_objects(objects: &Vec<u8>) -> String {
        objects
            .iter()
            .map(|n| format!("{:02X}", n))
            .collect::<Vec<String>>()
            .join(",")
    }

    fn pseudo_command_pass1(
        &mut self,
        instruction: &Instruction,
        statement: &Statement,
    ) -> Result<(), AssemblyError> {
        pseudo_commands::pass1(
            instruction,
            statement,
            &mut self.labels,
            &mut self.pc,
            &mut self.origin,
        )
    }

    fn pseudo_command_pass2(&mut self, statement: &Statement) -> Result<Vec<u8>, AssemblyError> {
        pseudo_commands::pass2(statement)
    }

    fn add_label(&mut self, name: &str, line: usize, address: u16) {
        let entry = LabelEntry {
            name: name.to_string(),
            line: line,
            address: Address::Full(address),
        };
        self.labels.insert(name.to_string(), entry);
    }
}
