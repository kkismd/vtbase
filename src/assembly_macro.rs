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
    let mut stack = Vec::new();
    for instruction in instructions {
        let instructions = transform_instruction(instruction, &mut stack)?;
        result.extend(instructions);
    }
    Ok(result)
}

fn transform_instruction(
    instruction: &Instruction,
    stack: &mut Vec<String>,
) -> Result<Vec<Instruction>, AssemblyError> {
    if instruction.statements.len() == 0 {
        return Ok(vec![instruction.clone()]);
    }
    let statement = &instruction.statements[0];
    let command = statement.command()?;
    match command.as_str() {
        ";" => transform_if_statement(instruction),
        "@" => transform_do_statement(instruction, stack),
        _ => transform_statements(&instruction),
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
 *      ;=<,#macro_0.1
 *      A=A+1 Y=Y+1
 *  #macro_0.1
 */
fn transform_if_statement(instruction: &Instruction) -> Result<Vec<Instruction>, AssemblyError> {
    if !(&instruction.statements[0]).check_macro_if_statement() {
        return Ok(vec![instruction.clone()]);
    }

    let mut result = vec![];
    let label = generate_macro_identifier();
    let header = instruction.new_label(&label);
    let trailer_label = format!("{}.1", label);
    let tlaier = instruction.new_label(&trailer_label);
    result.push(header);
    let instructions = expand_if_statement(instruction, &trailer_label)?;
    result.extend(instructions);
    result.push(tlaier);
    // dbg!(&result);
    Ok(result)
}

fn expand_if_statement(
    instruction: &Instruction,
    macro_label: &str,
) -> Result<Vec<Instruction>, AssemblyError> {
    let mut result = vec![];
    let if_stmt = &instruction.statements[0];
    let cmd = if_stmt.command()?;
    let expr = &if_stmt.expression;
    if cmd != ";" {
        return Err(AssemblyError::MacroError(format!("invalid command")));
    }
    if let Expr::BinOp(lhs, op, rhs) = expr {
        // 1st line
        // T=X-10
        let stmt1 = Statement::new("T", Expr::BinOp(lhs.clone(), Operator::Sub, rhs.clone()));
        let inst1 = Instruction::new(
            instruction.line_number,
            instruction.address,
            None,
            vec![stmt1],
            vec![],
        );
        result.push(inst1);

        // 2nd line
        // ;=<,#macro_1.1
        let sysop = match op {
            // 条件を逆にしてTHEN節をスキップする判定を行う
            Operator::Equal => '/',
            Operator::NotEqual => '=',
            Operator::Less => '>',
            Operator::Greater => '<',
            _ => {
                return Err(AssemblyError::MacroError(format!(
                    "{:?} expand_if_statement() invalid operator {:?}",
                    instruction, op
                )))
            }
        };
        let lhs = Box::new(Expr::SystemOperator(sysop));
        let rhs = Box::new(Expr::Identifier(macro_label.to_string()));
        let expr2 = Expr::BinOp(lhs, Operator::Comma, rhs);
        let stmt2 = Statement::new(";", expr2);
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
 *     ;=<,#macro_1.1
 *     #=#macro_1
 * #macro_1.1
 */
fn transform_do_statement(
    instruction: &Instruction,
    stack: &mut Vec<String>,
) -> Result<Vec<Instruction>, AssemblyError> {
    let mut result = vec![];
    let statement = &instruction.statements[0];
    if statement.expression == Expr::Empty {
        // ループ開始行 @ の処理
        let label = generate_macro_identifier();
        stack.push(label.clone());
        let line = instruction.new_label(&label);
        result.push(line);
        let rest = instruction.statements[1..].to_vec();
        let rest_instruction = Instruction::new(
            instruction.line_number,
            instruction.address,
            None,
            rest,
            vec![],
        );
        result.push(rest_instruction);
    } else {
        // ループ終了行 @=X>10 の処理
        let label = stack
            .pop()
            .ok_or(AssemblyError::MacroError(format!("unmatch do loop")))?;
        let instructions = expand_do_statement(instruction, &label)?;
        result.extend(instructions);
        let stmts = &instruction.statements[1..];
        let rest_instruction = Instruction::new(
            instruction.line_number,
            instruction.address,
            None,
            stmts.to_vec(),
            vec![],
        );
        result.push(rest_instruction);
    }
    // dbg!(&result);
    Ok(result)
}

fn expand_do_statement(
    instruction: &Instruction,
    label: &str,
) -> Result<Vec<Instruction>, AssemblyError> {
    let mut result = vec![];
    let Statement {
        command: _,
        expression: expr,
    } = &instruction.statements[0];
    if let Expr::BinOp(lhs, op, rhs) = expr {
        // 1st line
        // T=X-10
        let stmt1 = Statement::new("T", Expr::BinOp(lhs.clone(), Operator::Sub, rhs.clone()));
        let inst1 = Instruction::new(
            instruction.line_number,
            instruction.address,
            None,
            vec![stmt1],
            vec![],
        );
        result.push(inst1);

        // 2nd line
        // ;=<,#macro_1.1
        let sysop = match op {
            // 条件を逆にしてループを抜ける判定を行う
            Operator::Equal => '/',
            Operator::NotEqual => '=',
            Operator::Less => '>',
            Operator::Greater => '<',
            _ => {
                return Err(AssemblyError::MacroError(format!(
                    "expand_do_statement() invalid operator {:?}",
                    op
                )))
            }
        };
        let lhs = Box::new(Expr::SystemOperator(sysop));
        let next_label = format!("{}.1", label);
        let rhs = Box::new(Expr::Identifier(next_label.clone()));
        let expr2 = Expr::BinOp(lhs, Operator::Comma, rhs);
        let stmt2 = Statement::new(";", expr2);
        let inst2 = Instruction::new(
            instruction.line_number,
            instruction.address,
            None,
            vec![stmt2],
            vec![],
        );
        result.push(inst2);
        // 3rd line
        // #=#macro_1
        let stmt3 = Statement::new("#", Expr::Identifier(label.to_string()));
        let inst3 = Instruction::new(
            instruction.line_number,
            instruction.address,
            None,
            vec![stmt3],
            vec![],
        );
        result.push(inst3);
        // 4th line
        // #macro_1.1
        let inst4 = instruction.new_label(next_label.as_str());
        result.push(inst4);
    }

    Ok(result)
}

fn transform_statements(instruction: &Instruction) -> Result<Vec<Instruction>, AssemblyError> {
    let statements = &instruction.statements;
    let mut result = vec![];
    for statement in statements {
        let stmt = transform_statement(statement)?;
        result.push(stmt);
    }

    let mut inst = instruction.clone();
    inst.statements = result;
    Ok(vec![inst])
}

fn transform_statement(statement: &Statement) -> Result<Statement, AssemblyError> {
    Ok(statement.clone())
}

use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn generate_macro_identifier() -> String {
    let count = COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("#macro_{}", count)
}
