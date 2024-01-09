pub mod pseudo_commands;

use crate::opcode;
use crate::parser::expression::Operator;
use crate::parser::statement::Statement;
use crate::{error::AssemblyError, parser::Line};
use std::collections::HashMap;
use std::path::PathBuf;

pub struct Assembler {
    pub pc: usize,
    pub labels: LabelTable,
    pub opcode_table: opcode::OpcodeTable,
    pub current_label: String,
    pub is_address_set: bool,
    pub current_path: PathBuf,
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
                        Operator::Eor => n ^ m,
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
                        Operator::Eor => n ^ (*m as u16),
                        _ => return self.type_error(other, &op.to_string()),
                    }))
                } else {
                    self.type_error(other, &op.to_string())
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
                        Operator::Eor => n ^ m,
                        _ => return self.type_error(other, &op.to_string()),
                    }))
                } else {
                    self.type_error(other, &op.to_string())
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
    pub fn new(current_path: PathBuf) -> Self {
        Self {
            pc: 0,
            labels: HashMap::new(),
            opcode_table: opcode::OpcodeTable::new(),
            current_label: String::new(),
            is_address_set: false,
            current_path,
        }
    }

    pub fn assemble(&mut self, lines: &mut Vec<Line>) -> Result<usize, AssemblyError> {
        self.pass1(lines)?;
        let obj_size = self.pass2(lines)?;
        Ok(obj_size)
    }

    fn pass1(&mut self, lines: &mut Vec<Line>) -> Result<(), AssemblyError> {
        for line in lines {
            self.pass1_process_line(line).map_err(|e| {
                eprintln!("[pass1] line = {:?}, error = {}", line, e.message());
                e
            })?
        }
        Ok(())
    }

    fn pass1_process_line(&mut self, line: &mut Line) -> Result<(), AssemblyError> {
        if self.pc > 0x10000 {
            return Err(AssemblyError::program("address overflow"));
        }
        line.address = self.pc as u16;
        self.entry_label(line)?;
        for statement in &line.statements {
            if statement.is_pseudo() {
                self.pseudo_command_pass1(line, statement)?;
                continue;
            }
            if !self.is_address_set {
                return Err(AssemblyError::program("address not set"));
            }
            let assembly_instruction = statement.decode(&self.labels)?;
            let len = assembly_instruction.addressing_mode.length();
            self.pc += len;
        }
        Ok(())
    }

    fn pass2(&mut self, lines: &mut Vec<Line>) -> Result<usize, AssemblyError> {
        self.current_label = String::new();
        let mut objects_size = 0;
        for line in lines {
            let size = self.pass2_process_line(line).map_err(|e| {
                eprintln!("[pass2] line = {:?}, error = {}", line, e.message());
                e
            })?;
            objects_size += size;
        }
        Ok(objects_size)
    }

    fn pass2_process_line(&mut self, line: &mut Line) -> Result<usize, AssemblyError> {
        let mut objects_size = 0;
        let mut pc: usize = line.address as usize;
        self.track_global_label(line);
        for statement in &line.statements {
            let objects = if statement.is_pseudo() {
                let pc_u16 = pc as u16;
                self.pseudo_command_pass2(statement, &pc_u16)?
            } else {
                statement.compile(&self.opcode_table, &self.labels, &self.current_label, pc)?
            };
            pc += objects.len();
            objects_size += objects.len();
            line.object_codes.extend(objects);
        }
        Ok(objects_size)
    }

    fn track_global_label(&mut self, line: &mut Line) {
        if let Some(label) = &line.label {
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
    fn entry_label(&mut self, line: &mut Line) -> Result<(), AssemblyError> {
        if let Some(label) = &line.label {
            let mut label = label.clone();
            if label.starts_with('.') {
                // local label
                if self.current_label.is_empty() {
                    return Err(AssemblyError::program("global label not found"));
                }
                label = format!("{}{}", self.current_label, label);
                self.add_entry(&label, line)?;
            } else if label.starts_with("#macro_") {
                // macro label -- don't memorize current label
                self.add_entry(&label, line)?;
            } else {
                // global label
                self.add_entry(&label, line)?;
                self.current_label = label.to_string();
            }
            if let Some(entry) = self.labels.get_mut(&label) {
                entry.address = Address::Full(self.pc as u16);
            }
        }
        Ok(())
    }

    fn add_entry(&mut self, label: &str, line: &Line) -> Result<(), AssemblyError> {
        if !self.labels.contains_key(label) {
            self.add_label(label, line.line_number, self.pc as u16);
            Ok(())
        } else {
            Err(AssemblyError::label_used(line.line_number, label))
        }
    }

    fn _dump_objects(objects: &[u8]) -> String {
        objects
            .iter()
            .map(|n| format!("{:02X}", n))
            .collect::<Vec<String>>()
            .join(",")
    }

    fn pseudo_command_pass1(
        &mut self,
        line: &Line,
        statement: &Statement,
    ) -> Result<(), AssemblyError> {
        pseudo_commands::pass1(
            line,
            statement,
            self.current_path.clone(),
            &mut self.labels,
            &mut self.pc,
            &mut self.is_address_set,
        )
    }

    fn pseudo_command_pass2(
        &mut self,
        statement: &Statement,
        current_address: &u16,
    ) -> Result<Vec<u8>, AssemblyError> {
        let labels = &self.labels;
        pseudo_commands::pass2(
            statement,
            labels,
            current_address,
            self.current_path.clone(),
        )
    }

    fn add_label(&mut self, name: &str, line: usize, address: u16) {
        let entry = LabelEntry {
            name: name.to_string(),
            line,
            address: Address::Full(address),
        };
        self.labels.insert(name.to_string(), entry);
    }
}
