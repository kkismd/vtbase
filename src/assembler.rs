use crate::opcode;
use crate::parser::expression::matcher::{identifier, num16bit, num8bit, plus};
use crate::parser::expression::Expr;
use crate::parser::statement::{decoder, Statement};
use crate::{error::AssemblyError, parser::Instruction};
use std::collections::HashMap;

pub struct Assembler {
    pub origin: u16,
    pub pc: u16,
    pub labels: HashMap<String, LabelEntry>,
    pub opcode_table: opcode::OpcodeTable,
    pub current_label: String,
}

#[derive(Debug, Clone)]
pub struct LabelEntry {
    pub name: String,
    pub line: usize,
    pub address: Address,
}

#[derive(Debug, Clone)]
pub enum Address {
    Full(u16),
    ZeroPage(u8),
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
        self.assemble_pass1(instructions)?;
        let obj_size = self.assemble_pass2(instructions)?;
        Ok(obj_size)
    }

    fn assemble_pass1(&mut self, instructions: &mut Vec<Instruction>) -> Result<(), AssemblyError> {
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
            let assymbly_instruction = statement.decode(&self.labels)?;
            let len = assymbly_instruction.addressing_mode.length();
            self.pc += len as u16;
        })
    }

    fn assemble_pass2(
        &mut self,
        instructions: &mut Vec<Instruction>,
    ) -> Result<u16, AssemblyError> {
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

    fn dump_objects(objects: &Vec<u8>) -> String {
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
        let command = statement.command()?;
        if command == "*" {
            self.pass1_command_start_address(statement)
        } else if command == ":" {
            self.pass1_command_label_def(instruction, statement)
        } else if command == "?" {
            self.pass1_command_data_def(statement)
        } else {
            Ok(())
        }
    }

    fn pass1_command_label_def(
        &mut self,
        instruction: &Instruction,
        statement: &Statement,
    ) -> Result<(), AssemblyError> {
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
            label_entry.address = Address::Full(address);
        } else if let Expr::ByteNum(address) = statement.expression {
            label_entry.address = Address::ZeroPage(address as u8);
        }
        Ok(())
    }

    fn pass1_command_data_def(&mut self, statement: &Statement) -> Result<(), AssemblyError> {
        let values = statement.expression.traverse_comma();
        for value in values {
            self.pc += match value {
                Expr::ByteNum(_) => 1,
                Expr::WordNum(_) => 2,
                Expr::DecimalNum(_) => 1,
                Expr::StringLiteral(ref s) => s.len() as u16,
                _ => 0,
            }
        }
        Ok(())
    }

    fn pass1_command_start_address(&mut self, statement: &Statement) -> Result<(), AssemblyError> {
        if let Expr::WordNum(address) = statement.expression {
            self.origin = address;
            self.pc = address;
        }
        Ok(())
    }
    fn pseudo_command_pass2(&mut self, statement: &Statement) -> Result<Vec<u8>, AssemblyError> {
        let command = statement.command()?;
        let expression = &statement.expression;
        if command == "?" {
            let mut objects = Vec::new();
            let values = expression.traverse_comma();
            for value in values {
                match value {
                    Expr::DecimalNum(num) => {
                        objects.push(num as u8);
                        self.pc += 1;
                    }
                    Expr::ByteNum(num) => {
                        objects.push(num);
                        self.pc += 1;
                    }
                    Expr::WordNum(num) => {
                        objects.push((num & 0xff) as u8);
                        objects.push((num >> 8) as u8);
                        self.pc += 2;
                    }
                    Expr::StringLiteral(ref s) => {
                        for c in s.chars() {
                            objects.push(c as u8);
                        }
                        self.pc += s.len() as u16;
                    }
                    _ => {
                        dbg!(statement);
                        return Err(AssemblyError::program("invalid pseudo command"));
                    }
                }
            }
            return Ok(objects);
        }
        return Ok(Vec::new());
    }

    fn add_label(&mut self, name: &str, line: usize, address: u16) {
        let entry = LabelEntry {
            name: name.to_string(),
            line: line,
            address: Address::Full(address),
        };
        self.labels.insert(name.to_string(), entry);
    }

    fn lookup_label_mut(&mut self, name: &str) -> Result<&mut LabelEntry, AssemblyError> {
        self.labels
            .get_mut(name)
            .ok_or(AssemblyError::label_not_found(name))
    }

    fn lookup_label(&self, name: &str) -> Result<&LabelEntry, AssemblyError> {
        self.labels
            .get(name)
            .ok_or(AssemblyError::label_not_found(name))
    }
}
