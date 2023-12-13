use crate::parser::expression::{Expr, Operator};
use crate::Line;

use super::*;

pub fn pass1(
    line: &Line,
    statement: &Statement,
    labels: &mut LabelTable,
    pc: &mut usize,
    is_address_set: &mut bool,
) -> Result<(), AssemblyError> {
    let command = statement.command()?;
    if command == "*" {
        let address = pass1_command_start_address(statement)?;
        *pc = address as usize;
        *is_address_set = true;
        Ok(())
    } else if command == ":" {
        pass1_command_label_def(line, statement, labels)
    } else if command == "?" {
        let bytes = pass1_command_data_def(statement)?;
        if *is_address_set {
            *pc += bytes as usize;
        }
        Ok(())
    } else if command == "$" {
        let pc_u16 = *pc as u16;
        let bytes = pass1_command_data_fill(statement, &labels, &pc_u16)?;
        if *is_address_set {
            *pc += bytes as usize;
        }
        Ok(())
    } else {
        Ok(())
    }
}

// label def
fn pass1_command_label_def(
    line: &Line,
    statement: &Statement,
    labels: &mut LabelTable,
) -> Result<(), AssemblyError> {
    let address = statement.expression.calculate_address(&labels)?;
    let label_name = line
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
            Expr::Identifier(_) => 2,
            _ => return Err(AssemblyError::program("invalid data command")),
        }
    }
    Ok(pc)
}

// %=$FF,12 -> fill 12 bytes with data $FF
fn pass1_command_data_fill(
    statement: &Statement,
    labels: &LabelTable,
    current_address: &u16,
) -> Result<u16, AssemblyError> {
    let expr = &statement.expression;
    match expr {
        Expr::BinOp(left, Operator::Comma, right) => {
            if let Expr::ByteNum(_) = **left {
                let fill_count = right.evaluate(labels, current_address)?;
                return Ok(fill_count);
            }
            if let Expr::WordNum(_) = **left {
                let fill_count = right.evaluate(labels, current_address)?;
                return Ok(fill_count * 2);
            }
        }
        _ => (),
    }
    Err(AssemblyError::program("invalid fill command"))
}

fn pass1_command_start_address(statement: &Statement) -> Result<u16, AssemblyError> {
    if let Expr::WordNum(address) = statement.expression {
        return Ok(address);
    }
    Err(AssemblyError::program("invalid start address"))
}
pub fn pass2(
    statement: &Statement,
    labels: &LabelTable,
    current_address: &u16,
) -> Result<Vec<u8>, AssemblyError> {
    let command = statement.command()?;
    let expression = &statement.expression;
    if command == "?" {
        return pass2_command_data_def(expression, statement, labels);
    } else if command == "$" {
        return pass2_command_data_fill(statement, labels, current_address);
    }
    return Ok(Vec::new());
}

fn pass2_command_data_def(
    expression: &Expr,
    statement: &Statement,
    labels: &LabelTable,
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
            Expr::Identifier(ref s) => {
                let label = labels
                    .get(s)
                    .ok_or(AssemblyError::program("label not found"))?;
                if let Address::Full(address) = label.address {
                    objects.push((address & 0xff) as u8);
                    objects.push((address >> 8) as u8);
                } else {
                    dbg!(statement);
                    return Err(AssemblyError::program("invalid data command"));
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

// %=$FF,12 -> fill 12 bytes with data $FF
fn pass2_command_data_fill(
    statement: &Statement,
    labels: &LabelTable,
    current_address: &u16,
) -> Result<Vec<u8>, AssemblyError> {
    let expr = &statement.expression;
    let mut objects = Vec::new();
    match expr {
        Expr::BinOp(left, Operator::Comma, right) => {
            let fill_count = right.evaluate(labels, current_address)?;
            if let Expr::ByteNum(fill_value) = **left {
                for _ in 0..fill_count {
                    objects.push(fill_value);
                }
                return Ok(objects);
            }
            if let Expr::WordNum(fill_value) = **left {
                for _ in 0..fill_count {
                    objects.push((fill_value & 0xff) as u8);
                    objects.push((fill_value >> 8) as u8);
                }
                return Ok(objects);
            }
        }
        _ => (),
    }
    dbg!(statement);
    return Err(AssemblyError::program("invalid fill command"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pass1_command_data_fill() {
        let statement = Statement::new(
            "$",
            Expr::BinOp(
                Box::new(Expr::ByteNum(0xff)),
                Operator::Comma,
                Box::new(Expr::DecimalNum(12)),
            ),
        );
        let mut labels = HashMap::new();
        let mut pc = 0;
        let mut is_address_set = false;
        let statement_clone = statement.clone();
        let result = pass1(
            &Line::new(0, 0, None, vec![statement], vec![]),
            &statement_clone,
            &mut labels,
            &mut pc,
            &mut is_address_set,
        );
        assert!(result.is_ok());
        assert_eq!(pc, 12);
    }

    #[test]
    fn test_pass2_command_data_fill() {
        let statement = Statement::new(
            "$",
            Expr::BinOp(
                Box::new(Expr::ByteNum(0xff)),
                Operator::Comma,
                Box::new(Expr::DecimalNum(12)),
            ),
        );
        let labels = HashMap::new();
        let pc = 0;
        let result = pass2(&statement, &labels, &pc);
        assert!(result.is_ok());
        let objects = result.unwrap();
        assert_eq!(objects.len(), 12);
        assert_eq!(objects[0], 0xff);
    }

    #[test]
    fn test_pass1_command_data_fill_word() {
        let statement = Statement::new(
            "$",
            Expr::BinOp(
                Box::new(Expr::WordNum(0xff00)),
                Operator::Comma,
                Box::new(Expr::DecimalNum(12)),
            ),
        );
        let mut labels = HashMap::new();
        let mut pc = 0;
        let mut is_address_set = false;
        let statement_clone = statement.clone();
        let result = pass1(
            &Line::new(0, 0, None, vec![statement], vec![]),
            &statement_clone,
            &mut labels,
            &mut pc,
            &mut is_address_set,
        );
        assert!(result.is_ok());
        assert_eq!(pc, 12 * 2);
    }

    #[test]
    fn test_pass2_command_data_fill_word() {
        let statement = Statement::new(
            "$",
            Expr::BinOp(
                Box::new(Expr::WordNum(0x1234)),
                Operator::Comma,
                Box::new(Expr::DecimalNum(12)),
            ),
        );
        let labels = HashMap::new();
        let pc = 0;
        let result = pass2(&statement, &labels, &pc);
        assert!(result.is_ok());
        let objects = result.unwrap();
        assert_eq!(objects.len(), 12 * 2);

        assert_eq!(objects[0], 0x34);
        assert_eq!(objects[1], 0x12);

        for i in 0..12 {
            assert_eq!(objects[i * 2], 0x34);
            assert_eq!(objects[i * 2 + 1], 0x12);
        }
    }
}
