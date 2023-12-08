use crate::parser::expression::{Expr, Operator};
use crate::Instruction;

use super::*;

pub fn pass1(
    instruction: &Instruction,
    statement: &Statement,
    labels: &mut HashMap<String, LabelEntry>,
    pc: &mut u16,
    origin: &mut u16,
) -> Result<(), AssemblyError> {
    let command = statement.command()?;
    if command == "*" {
        let address = pass1_command_start_address(statement)?;
        *origin = address;
        *pc = address;
        Ok(())
    } else if command == ":" {
        pass1_command_label_def(instruction, statement, labels)
    } else if command == "?" {
        *pc = pass1_command_data_def(statement)?;
        Ok(())
    } else {
        Ok(())
    }
}

fn pass1_command_label_def(
    instruction: &Instruction,
    statement: &Statement,
    labels: &mut HashMap<String, LabelEntry>,
) -> Result<(), AssemblyError> {
    let address = statement.expression.calculate_address(&labels)?;
    let label_name = instruction
        .label
        .clone()
        .ok_or(AssemblyError::program("label def need label"))?;
    // fetch label entry
    let label_entry = labels
        .get_mut(&label_name)
        .ok_or(AssemblyError::program("label not found"))?;
    // set address to entry
    label_entry.address = address;
    Ok(())
}

fn pass1_command_data_def(statement: &Statement) -> Result<u16, AssemblyError> {
    let mut pc = 0;
    let values = statement.expression.traverse_comma();
    for value in values {
        pc += match value {
            Expr::ByteNum(_) => 1,
            Expr::WordNum(_) => 2,
            Expr::DecimalNum(_) => 1,
            Expr::StringLiteral(ref s) => s.len() as u16,
            _ => return Err(AssemblyError::program("invalid data command")),
        }
    }
    Ok(pc)
}

fn pass1_command_start_address(statement: &Statement) -> Result<u16, AssemblyError> {
    if let Expr::WordNum(address) = statement.expression {
        return Ok(address);
    }
    Err(AssemblyError::program("invalid start address"))
}
pub fn pass2(statement: &Statement) -> Result<Vec<u8>, AssemblyError> {
    let command = statement.command()?;
    let expression = &statement.expression;
    if command == "?" {
        return pass2_command_data_def(expression, statement);
    }
    return Ok(Vec::new());
}

fn pass2_command_data_def(
    expression: &Expr,
    statement: &Statement,
) -> Result<Vec<u8>, AssemblyError> {
    let mut objects = Vec::new();
    let values = expression.traverse_comma();
    for value in values {
        match value {
            Expr::DecimalNum(num) => {
                objects.push(num as u8);
            }
            Expr::ByteNum(num) => {
                objects.push(num);
            }
            Expr::WordNum(num) => {
                objects.push((num & 0xff) as u8);
                objects.push((num >> 8) as u8);
            }
            Expr::StringLiteral(ref s) => {
                for c in s.chars() {
                    objects.push(c as u8);
                }
            }
            _ => {
                dbg!(statement);
                return Err(AssemblyError::program("invalid data command"));
            }
        }
    }
    Ok(objects)
}
