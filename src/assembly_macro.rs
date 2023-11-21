use crate::{
    error::AssemblyError,
    parser::{
        expression::{Expr, Operator},
        statement::{self, Statement},
        Instruction,
    },
};

pub fn expand(instructions: &Vec<Instruction>) -> Vec<Instruction> {
    instructions
        .into_iter()
        .flat_map(transform_instruction)
        .collect()
}

fn transform_instruction(instruction: &Instruction) -> Vec<Instruction> {
    if instruction.statements.len() == 0 {
        return vec![instruction.clone()];
    }
    let statement = &instruction.statements[0];
    if !statement.is_macro() {
        return vec![instruction.clone()];
    }
    let command = &statement.command;
    match command.as_str() {
        ";" => transform_if_statement(instruction),
        "@" => transform_do_statement(instruction),
        _ => vec![instruction.clone()],
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
fn transform_if_statement(instruction: &Instruction) -> Vec<Instruction> {
    let mut result = vec![];
    let label = generate_macro_identifier();
    let header = instruction.new_label(&label);
    let trailer_label = format!("{}.1", label);
    let tlaier = instruction.new_label(&trailer_label);
    result.push(header);
    result.extend(expand_if_statement(instruction, &trailer_label));
    result.push(tlaier);
    dbg!(&result);
    result
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
fn expand_if_statement(instruction: &Instruction, macro_label: &str) -> Vec<Instruction> {
    let mut result = vec![];
    let if_stmt = &instruction.statements[0];
    if let Statement {
        command: cmd,
        expression: expr,
    } = if_stmt
    {
        if cmd == ";" {
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
                    _ => panic!("invalid operator"),
                };
                let lhs = Box::new(Expr::SystemOperator(sysop));
                let rhs = Box::new(Expr::Identifier(macro_label.to_string()));
                let expr2 = Expr::BinOp(lhs, Operator::Comma, rhs);
                let stmt2 = Statement::new(";".to_string(), expr2);
                let mut inst2 = Instruction::new(
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
        }
    }

    result
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
fn transform_do_statement(instruction: &Instruction) -> Vec<Instruction> {
    let mut result = vec![];
    let label = generate_macro_identifier();
    let header = instruction.new_label(&label);
    result.push(header);

    dbg!(&result);
    result
}

use std::{
    f32::consts::E,
    sync::atomic::{AtomicUsize, Ordering},
};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn generate_macro_identifier() -> String {
    let count = COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("#macro_{}", count)
}
