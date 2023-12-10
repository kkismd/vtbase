use crate::{
    error::AssemblyError,
    parser::{
        expression::{Expr, Operator},
        statement::Statement,
        Line,
    },
};

pub fn expand(lines: &Vec<Line>) -> Result<Vec<Line>, AssemblyError> {
    let mut result = Vec::new();
    let mut stack = Vec::new();
    for line in lines {
        let lines = transform_line(line, &mut stack)?;
        result.extend(lines);
    }
    Ok(result)
}

fn transform_line(line: &Line, stack: &mut Vec<String>) -> Result<Vec<Line>, AssemblyError> {
    if line.statements.len() == 0 {
        return Ok(vec![line.clone()]);
    }
    let statement = &line.statements[0];
    let command = statement.command()?;
    match command.as_str() {
        ";" => transform_if_statement(line),
        "@" => transform_do_statement(line, stack),
        _ => transform_statements(&line),
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
fn transform_if_statement(line: &Line) -> Result<Vec<Line>, AssemblyError> {
    if !(&line.statements[0]).check_macro_if_statement() {
        return Ok(vec![line.clone()]);
    }

    let mut result = vec![];
    let label = generate_macro_identifier();
    let header = line.new_label(&label);
    let trailer_label = format!("{}.1", label);
    let trailer = line.new_label(&trailer_label);
    result.push(header);
    let lines = expand_if_statement(line, &trailer_label)?;
    result.extend(lines);
    result.push(trailer);
    // dbg!(&result);
    Ok(result)
}

fn expand_if_statement(line: &Line, macro_label: &str) -> Result<Vec<Line>, AssemblyError> {
    let mut result = vec![];
    let if_stmt = &line.statements[0];
    let cmd = if_stmt.command()?;
    let expr = &if_stmt.expression;
    if cmd != ";" {
        return Err(AssemblyError::MacroError(format!("invalid command")));
    }
    if let Expr::BinOp(lhs, op, rhs) = expr {
        // 1st line
        // T=X-10
        let stmt1 = Statement::new("T", Expr::BinOp(lhs.clone(), Operator::Sub, rhs.clone()));
        let inst1 = Line::new(line.line_number, line.address, None, vec![stmt1], vec![]);
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
                    line, op
                )))
            }
        };
        let lhs = Box::new(Expr::SystemOperator(sysop));
        let rhs = Box::new(Expr::Identifier(macro_label.to_string()));
        let expr2 = Expr::BinOp(lhs, Operator::Comma, rhs);
        let stmt2 = Statement::new(";", expr2);
        let inst2 = Line::new(line.line_number, line.address, None, vec![stmt2], vec![]);
        result.push(inst2);

        // 3rd line
        // A=A+1 Y=Y+1
        let then_statements = line.statements[1..].to_vec();
        let mut expanded_statements = vec![];
        for statement in then_statements {
            let stmt = transform_statement(&statement)?;
            expanded_statements.extend(stmt);
        }

        let inst3 = Line::new(
            line.line_number,
            line.address,
            None,
            expanded_statements,
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
    line: &Line,
    stack: &mut Vec<String>,
) -> Result<Vec<Line>, AssemblyError> {
    let mut result = vec![];
    let statement = &line.statements[0];
    if statement.expression == Expr::Empty {
        // ループ開始行 @ の処理
        let label = generate_macro_identifier();
        stack.push(label.clone());
        let label_line = line.new_label(&label);
        result.push(label_line);
        let rest = line.statements[1..].to_vec();
        let rest_line = Line::new(line.line_number, line.address, None, rest, vec![]);
        result.push(rest_line);
    } else {
        // ループ終了行 @=X>10 の処理
        let label = stack
            .pop()
            .ok_or(AssemblyError::MacroError(format!("unmatch do loop")))?;
        let lines = expand_do_statement(line, &label)?;
        result.extend(lines);
        let stmts = &line.statements[1..];
        let rest_line = Line::new(line.line_number, line.address, None, stmts.to_vec(), vec![]);
        result.push(rest_line);
    }
    // dbg!(&result);
    Ok(result)
}

fn expand_do_statement(line: &Line, label: &str) -> Result<Vec<Line>, AssemblyError> {
    let mut result = vec![];
    let Statement {
        command: _,
        expression: expr,
    } = &line.statements[0];
    if let Expr::BinOp(lhs, op, rhs) = expr {
        // 1st line
        // T=X-10
        let stmt1 = Statement::new("T", Expr::BinOp(lhs.clone(), Operator::Sub, rhs.clone()));
        let inst1 = Line::new(line.line_number, line.address, None, vec![stmt1], vec![]);
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
        let inst2 = Line::new(line.line_number, line.address, None, vec![stmt2], vec![]);
        result.push(inst2);
        // 3rd line
        // #=#macro_1
        let stmt3 = Statement::new("#", Expr::Identifier(label.to_string()));
        let inst3 = Line::new(line.line_number, line.address, None, vec![stmt3], vec![]);
        result.push(inst3);
        // 4th line
        // #macro_1.1
        let inst4 = line.new_label(next_label.as_str());
        result.push(inst4);
    }

    Ok(result)
}

fn transform_statements(line: &Line) -> Result<Vec<Line>, AssemblyError> {
    let statements = &line.statements;
    let mut result = vec![];
    for statement in statements {
        let transformd_statements = transform_statement(statement)?;
        result.extend(transformd_statements);
    }

    let mut inst = line.clone();
    inst.statements = result;
    Ok(vec![inst])
}

fn transform_statement(statement: &Statement) -> Result<Vec<Statement>, AssemblyError> {
    a_plus_n(&statement.expression)
        .map(|expr| transform_adc_statement(&expr))
        .or_else(|_| Ok(vec![statement.clone()]))
}

fn a_plus_n(expr: &Expr) -> Result<Expr, AssemblyError> {
    match expr {
        Expr::BinOp(lhs, Operator::Add, rhs) => match **lhs {
            Expr::Identifier(ref id) if id == "A" => Ok((**rhs).clone()),
            _ => Err(AssemblyError::MacroError("a_plus_n() ".to_string())),
        },
        _ => Err(AssemblyError::MacroError("a_plus_n()".to_string())),
    }
}

// A=A+n -> C=0 A=AC+n
fn transform_adc_statement(expr: &Expr) -> Vec<Statement> {
    let mut result = vec![];
    let stmt1 = Statement::new("C", Expr::DecimalNum(0));
    let ac_plus_n = Expr::BinOp(
        Box::new(Expr::Identifier("AC".to_string())),
        Operator::Add,
        Box::new(expr.clone()),
    );
    let stmt2 = Statement::new("A", ac_plus_n);
    result.push(stmt1);
    result.push(stmt2);
    result
}

use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn generate_macro_identifier() -> String {
    let count = COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("#macro_{}", count)
}
