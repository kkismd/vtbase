use crate::{
    error::AssemblyError,
    parser::{
        expression::{Expr, Operator},
        statement::Statement,
        Instruction,
    },
};

pub fn expand(instructions: &Vec<Instruction>) -> Result<Vec<Instruction>, AssemblyError> {
    let mut result = Vec::new();
    for instruction in instructions {
        let instructions = transform_instruction(instruction)?;
        result.extend(instructions);
    }
    Ok(result)
}

fn transform_instruction(instruction: &Instruction) -> Result<Vec<Instruction>, AssemblyError> {
    if instruction.statements.len() == 0 {
        return Ok(vec![instruction.clone()]);
    }
    let statement = &instruction.statements[0];
    if !statement.is_macro() {
        return Ok(vec![instruction.clone()]);
    }
    let command = &statement.command;
    match command.as_str() {
        ";" => transform_if_statement(instruction),
        "@" => transform_do_statement(instruction),
        _ => Ok(vec![instruction.clone()]),
    }
}

/**
 * 元のステートメント
 *  ;=X>10 A=A+1 Y=Y+1
 *    ^^(1) ^^^^^^^^^(2)
 *   1 = 判定式 / 2 = 実行文
 *
 * 展開形
 *  #macro_0
 *      T=X-10
 *      ;=>,#macro_0.1
 *      A=A+1 Y=Y+1
 *  #macro_0.1
 */
fn transform_if_statement(instruction: &Instruction) -> Result<Vec<Instruction>, AssemblyError> {
    let mut result = vec![];
    let label = generate_macro_identifier();
    let header = instruction.new_label(&label);
    let trailer_label = format!("{}.1", label);
    let tlaier = instruction.new_label(&trailer_label);
    result.push(header);
    let instructions = expand_if_statement(instruction, &trailer_label)?;
    result.extend(instructions);
    result.push(tlaier);
    dbg!(&result);
    Ok(result)
}

/**
 * 元コード
 *  ;=X>10 A=A+1 Y=Y+1
 *
 * 展開系
 * #macro_1
 *   T=X-10
 *   ;=>,#macro_1.1
 *   A=A+1 Y=Y+1
 * #macro_1.1
 *
 */
fn expand_if_statement(
    instruction: &Instruction,
    macro_label: &str,
) -> Result<Vec<Instruction>, AssemblyError> {
    let mut result = vec![];
    let if_stmt = &instruction.statements[0];
    let Statement {
        command: cmd,
        expression: expr,
    } = if_stmt;
    if cmd != ";" {
        return Err(AssemblyError::MacroError(format!("invalid command")));
    }
    if let Expr::BinOp(lhs, op, rhs) = expr {
        // 1st line
        // T=X-10
        let stmt1 = Statement::new(
            "T".to_string(),
            Expr::BinOp(lhs.clone(), Operator::Sub, rhs.clone()),
        );
        let inst1 = Instruction::new(
            instruction.line_number,
            instruction.address,
            None,
            vec![stmt1],
            vec![],
        );
        result.push(inst1);

        // 2nd line
        // ;=>,#macro_1.1
        let sysop = match op {
            Operator::Equal => '=',
            Operator::Less => '<',
            Operator::Greater => '>',
            _ => return Err(AssemblyError::MacroError(format!("invalid operator"))),
        };
        let lhs = Box::new(Expr::SystemOperator(sysop));
        let rhs = Box::new(Expr::Identifier(macro_label.to_string()));
        let expr2 = Expr::BinOp(lhs, Operator::Comma, rhs);
        let stmt2 = Statement::new(";".to_string(), expr2);
        let inst2 = Instruction::new(
            instruction.line_number,
            instruction.address,
            None,
            vec![stmt2],
            vec![],
        );
        result.push(inst2);

        // 3rd line
        // A=A+1 Y=Y+1
        let stmt3 = instruction.statements[1..].to_vec();
        let inst3 = Instruction::new(
            instruction.line_number,
            instruction.address,
            None,
            stmt3,
            vec![],
        );
        result.push(inst3);
    }

    Ok(result)
}

/**
 * 元のステートメント
 *   @
 *   A=A+1 Y=Y+1
 *   X=-
 *   @=X>10
 *
 * 展開形
 * #macro_1
 *     A=A+1 Y=Y+1
 *     X=-
 *     T=X-10
 *     ;=>,#macro_1
 */
fn transform_do_statement(instruction: &Instruction) -> Result<Vec<Instruction>, AssemblyError> {
    let mut result = vec![];
    let label = generate_macro_identifier();
    let header = instruction.new_label(&label);
    result.push(header);

    dbg!(&result);
    Ok(result)
}
use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn generate_macro_identifier() -> String {
    let count = COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("#macro_{}", count)
}
