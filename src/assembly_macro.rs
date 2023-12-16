use regex::Regex;

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
            Operator::Equal => "/",
            Operator::NotEqual => "=",
            Operator::Less => ">",
            Operator::Greater => "<",
            _ => {
                return Err(AssemblyError::MacroError(format!(
                    "{:?} expand_if_statement() invalid operator {:?}",
                    line, op
                )))
            }
        };
        let lhs = Box::new(Expr::SystemOperator(sysop.to_string()));
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
            .ok_or(AssemblyError::MacroError(format!("mismatch do loop")))?;
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
            Operator::Equal => "/",
            Operator::NotEqual => "=",
            Operator::Less => ">",
            Operator::Greater => "<",
            _ => {
                return Err(AssemblyError::MacroError(format!(
                    "expand_do_statement() invalid operator {:?}",
                    op
                )))
            }
        };
        let lhs = Box::new(Expr::SystemOperator(sysop.to_string()));
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
        let transformed_statements = transform_statement(statement)?;
        result.extend(transformed_statements);
    }

    let mut inst = line.clone();
    inst.statements = result;
    Ok(vec![inst])
}

fn transform_statement(statement: &Statement) -> Result<Vec<Statement>, AssemblyError> {
    a_plus_n(&statement.expression)
        .map(|expr| transform_adc_statement(&expr))
        .or_else(|_| inxx(statement).map(|n| transform_xx_statement(n, "X", "+")))
        .or_else(|_| dexx(statement).map(|n| transform_xx_statement(n, "X", "-")))
        .or_else(|_| inyy(statement).map(|n| transform_xx_statement(n, "Y", "+")))
        .or_else(|_| deyy(statement).map(|n| transform_xx_statement(n, "Y", "-")))
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

fn process_xx(
    statement: &Statement,
    cmd: &str,
    op_pattern: &str,
    error_msg: &str,
) -> Result<usize, AssemblyError> {
    let re = Regex::new(op_pattern).unwrap();
    match statement.command.clone() {
        Expr::Identifier(command) if command == cmd => match &statement.expression {
            Expr::SystemOperator(op) if re.is_match(&op) => Ok(op.len()),
            _ => Err(AssemblyError::MacroError(error_msg.to_string())),
        },
        _ => Err(AssemblyError::MacroError(error_msg.to_string())),
    }
}

fn inxx(statement: &Statement) -> Result<usize, AssemblyError> {
    process_xx(statement, "X", r"^[+]+$", "inxx()")
}

fn dexx(statement: &Statement) -> Result<usize, AssemblyError> {
    process_xx(statement, "X", r"^[-]+$", "dexx()")
}

fn inyy(statement: &Statement) -> Result<usize, AssemblyError> {
    process_xx(statement, "Y", r"^[+]+$", "inyy()")
}

fn deyy(statement: &Statement) -> Result<usize, AssemblyError> {
    process_xx(statement, "Y", r"^[-]+$", "deyy()")
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

fn transform_xx_statement(n: usize, cmd: &str, op: &str) -> Vec<Statement> {
    let mut result = vec![];
    let stmt = Statement::new(cmd, Expr::SystemOperator(op.to_string()));
    for _ in 0..n {
        result.push(stmt.clone());
    }
    result
}

use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn generate_macro_identifier() -> String {
    let count = COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("#macro_{}", count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_add_statement() {
        let statement = Statement::new(
            "A",
            Expr::BinOp(
                Box::new(Expr::Identifier("A".to_string())),
                Operator::Add,
                Box::new(Expr::DecimalNum(1)),
            ),
        );
        let result = transform_statement(&statement).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].command, Expr::Identifier("C".to_string()));
        assert_eq!(result[0].expression, Expr::DecimalNum(0));
        assert_eq!(result[1].command, Expr::Identifier("A".to_string()));
        assert_eq!(
            result[1].expression,
            Expr::BinOp(
                Box::new(Expr::Identifier("AC".to_string())),
                Operator::Add,
                Box::new(Expr::DecimalNum(1))
            )
        );
    }

    #[test]
    fn test_transform_inxx_statement() {
        let statement = Statement::new("X", Expr::SystemOperator("++".to_string()));
        let result = transform_statement(&statement).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].command, Expr::Identifier("X".to_string()));
        assert_eq!(result[0].expression, Expr::SystemOperator("+".to_string()));
        assert_eq!(result[1].command, Expr::Identifier("X".to_string()));
        assert_eq!(result[1].expression, Expr::SystemOperator("+".to_string()));
    }

    #[test]
    fn test_transform_inxxxx_statement() {
        let statement = Statement::new("X", Expr::SystemOperator("++++".to_string()));
        let result = transform_statement(&statement).unwrap();
        assert_eq!(result.len(), 4);
        assert_eq!(result[0].command, Expr::Identifier("X".to_string()));
        assert_eq!(result[0].expression, Expr::SystemOperator("+".to_string()));
        assert_eq!(result[1].command, Expr::Identifier("X".to_string()));
        assert_eq!(result[1].expression, Expr::SystemOperator("+".to_string()));
        assert_eq!(result[2].command, Expr::Identifier("X".to_string()));
        assert_eq!(result[2].expression, Expr::SystemOperator("+".to_string()));
        assert_eq!(result[3].command, Expr::Identifier("X".to_string()));
        assert_eq!(result[3].expression, Expr::SystemOperator("+".to_string()));
    }

    #[test]
    fn test_transform_dexx_statement() {
        let statement = Statement::new("X", Expr::SystemOperator("--".to_string()));
        let result = transform_statement(&statement).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].command, Expr::Identifier("X".to_string()));
        assert_eq!(result[0].expression, Expr::SystemOperator("-".to_string()));
        assert_eq!(result[1].command, Expr::Identifier("X".to_string()));
        assert_eq!(result[1].expression, Expr::SystemOperator("-".to_string()));
    }

    #[test]
    fn test_transform_dexxxx_statement() {
        let statement = Statement::new("X", Expr::SystemOperator("----".to_string()));
        let result = transform_statement(&statement).unwrap();
        assert_eq!(result.len(), 4);
        assert_eq!(result[0].command, Expr::Identifier("X".to_string()));
        assert_eq!(result[0].expression, Expr::SystemOperator("-".to_string()));
        assert_eq!(result[1].command, Expr::Identifier("X".to_string()));
        assert_eq!(result[1].expression, Expr::SystemOperator("-".to_string()));
        assert_eq!(result[2].command, Expr::Identifier("X".to_string()));
        assert_eq!(result[2].expression, Expr::SystemOperator("-".to_string()));
        assert_eq!(result[3].command, Expr::Identifier("X".to_string()));
        assert_eq!(result[3].expression, Expr::SystemOperator("-".to_string()));
    }

    #[test]
    fn test_transform_inyy_statement() {
        let statement = Statement::new("Y", Expr::SystemOperator("++".to_string()));
        let result = transform_statement(&statement).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].command, Expr::Identifier("Y".to_string()));
        assert_eq!(result[0].expression, Expr::SystemOperator("+".to_string()));
        assert_eq!(result[1].command, Expr::Identifier("Y".to_string()));
        assert_eq!(result[1].expression, Expr::SystemOperator("+".to_string()));
    }

    #[test]
    fn test_transform_inyyyy_statement() {
        let statement = Statement::new("Y", Expr::SystemOperator("++++".to_string()));
        let result = transform_statement(&statement).unwrap();
        assert_eq!(result.len(), 4);
        assert_eq!(result[0].command, Expr::Identifier("Y".to_string()));
        assert_eq!(result[0].expression, Expr::SystemOperator("+".to_string()));
        assert_eq!(result[1].command, Expr::Identifier("Y".to_string()));
        assert_eq!(result[1].expression, Expr::SystemOperator("+".to_string()));
        assert_eq!(result[2].command, Expr::Identifier("Y".to_string()));
        assert_eq!(result[2].expression, Expr::SystemOperator("+".to_string()));
        assert_eq!(result[3].command, Expr::Identifier("Y".to_string()));
        assert_eq!(result[3].expression, Expr::SystemOperator("+".to_string()));
    }

    #[test]
    fn test_transform_deyy_statement() {
        let statement = Statement::new("Y", Expr::SystemOperator("--".to_string()));
        let result = transform_statement(&statement).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].command, Expr::Identifier("Y".to_string()));
        assert_eq!(result[0].expression, Expr::SystemOperator("-".to_string()));
        assert_eq!(result[1].command, Expr::Identifier("Y".to_string()));
        assert_eq!(result[1].expression, Expr::SystemOperator("-".to_string()));
    }

    #[test]
    fn test_transform_deyyyy_statement() {
        let statement = Statement::new("Y", Expr::SystemOperator("----".to_string()));
        let result = transform_statement(&statement).unwrap();
        assert_eq!(result.len(), 4);
        assert_eq!(result[0].command, Expr::Identifier("Y".to_string()));
        assert_eq!(result[0].expression, Expr::SystemOperator("-".to_string()));
        assert_eq!(result[1].command, Expr::Identifier("Y".to_string()));
        assert_eq!(result[1].expression, Expr::SystemOperator("-".to_string()));
        assert_eq!(result[2].command, Expr::Identifier("Y".to_string()));
        assert_eq!(result[2].expression, Expr::SystemOperator("-".to_string()));
        assert_eq!(result[3].command, Expr::Identifier("Y".to_string()));
        assert_eq!(result[3].expression, Expr::SystemOperator("-".to_string()));
    }
}
