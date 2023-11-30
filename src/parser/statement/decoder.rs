use std::collections::HashMap;

use crate::opcode::{AssemblyInstruction, Mnemonic, Mode, OperandValue};
use crate::{
    assembler::{Address, LabelEntry},
    error::AssemblyError,
    parser::expression::matcher::*,
    parser::expression::{matcher::parenthesized, Expr, Operator},
};

type Decoder<T> = fn(&Expr) -> Result<T, AssemblyError>;

pub fn decode_x(
    expr: &Expr,
    labels: &HashMap<String, LabelEntry>,
) -> Result<AssemblyInstruction, AssemblyError> {
    immediate(expr, labels)
        // X=$12
        .and_then(|num| ok_byte(&Mnemonic::LDX, Mode::Immediate, num))
        .or_else(|_| {
            // X=(label), X=(31), X=($1F)
            zeropage(expr, labels).and_then(|num| ok_byte(&Mnemonic::LDX, Mode::ZeroPage, num))
        })
        .or_else(|_| {
            // X=(label+Y), X=(31+Y), X=($1F+Y)
            zeropage_y(expr, labels).and_then(|num| ok_byte(&Mnemonic::LDX, Mode::ZeroPageY, num))
        })
        .or_else(|_| {
            // X=(label), X=(4863), X=($12FF)
            absolute(expr, labels).and_then(|num| ok_word(&Mnemonic::LDX, Mode::Absolute, num))
        })
        .or_else(|_| {
            // X=(label+Y), X=(4863+Y), X=($12FF+Y)
            absolute_y(expr, labels).and_then(|num| ok_word(&Mnemonic::LDX, Mode::AbsoluteY, num))
        })
        .or_else(|_| {
            // X=X+1, X=+
            increment(expr, "X").and_then(|_| ok_none(&Mnemonic::INX, Mode::Implied))
        })
        .or_else(|_| {
            // X=X-1, X=-
            decrement(expr, "X").and_then(|_| ok_none(&Mnemonic::DEX, Mode::Implied))
        })
        .or_else(|_| decode_error(expr))
}

pub fn decode_y(
    expr: &Expr,
    labels: &HashMap<String, LabelEntry>,
) -> Result<AssemblyInstruction, AssemblyError> {
    immediate(expr, labels)
        .and_then(|num| ok_byte(&Mnemonic::LDY, Mode::Immediate, num))
        .or_else(|_| {
            zeropage(expr, labels).and_then(|num| ok_byte(&Mnemonic::LDY, Mode::ZeroPage, num))
        })
        .or_else(|_| {
            zeropage_x(expr, labels).and_then(|num| ok_byte(&Mnemonic::LDY, Mode::ZeroPageX, num))
        })
        .or_else(|_| {
            absolute(expr, labels).and_then(|num| ok_word(&Mnemonic::LDY, Mode::Absolute, num))
        })
        .or_else(|_| {
            absolute_x(expr, labels).and_then(|num| ok_word(&Mnemonic::LDY, Mode::AbsoluteX, num))
        })
        .or_else(|_| increment(expr, "Y").and_then(|_| ok_none(&Mnemonic::INY, Mode::Implied)))
        .or_else(|_| decrement(expr, "Y").and_then(|_| ok_none(&Mnemonic::DEY, Mode::Implied)))
        .or_else(|_| decode_error(expr))
}

pub fn decode_a(
    expr: &Expr,
    labels: &HashMap<String, LabelEntry>,
) -> Result<AssemblyInstruction, AssemblyError> {
    decode_lda(expr, labels)
        .or_else(|_| decode_adc(expr, labels))
        .or_else(|_| decode_error(expr))
}

/**
 * Immediate     LDA #$44      A=$44
 * Zero Page     LDA $44       A=($44)
 * Zero Page,X   LDA $44,X     A=($44+X)
 * Absolute      LDA $4400     A=($4400)
 * Absolute,X    LDA $4400,X   A=($4400+X)
 * Absolute,Y    LDA $4400,Y   A=($4400+Y)
 * Indirect,X    LDA ($44,X)   A=[$44+X]
 * Indirect,Y    LDA ($44),Y   A=[$44]+Y
 */
fn decode_lda(
    expr: &Expr,

    labels: &HashMap<String, LabelEntry>,
) -> Result<AssemblyInstruction, AssemblyError> {
    immediate(expr, labels)
        .and_then(|num| ok_byte(&Mnemonic::LDA, Mode::Immediate, num))
        .or_else(|_| {
            zeropage(expr, labels).and_then(|num| ok_byte(&Mnemonic::LDA, Mode::ZeroPage, num))
        })
        .or_else(|_| {
            zeropage_x(expr, labels).and_then(|num| ok_byte(&Mnemonic::LDA, Mode::ZeroPageX, num))
        })
        .or_else(|_| {
            absolute(expr, labels).and_then(|num| ok_word(&Mnemonic::LDA, Mode::Absolute, num))
        })
        .or_else(|_| {
            absolute_x(expr, labels).and_then(|num| ok_word(&Mnemonic::LDA, Mode::AbsoluteX, num))
        })
        .or_else(|_| {
            absolute_y(expr, labels).and_then(|num| ok_word(&Mnemonic::LDA, Mode::AbsoluteX, num))
        })
        .or_else(|_| {
            indirect_x(expr, labels).and_then(|num| ok_byte(&Mnemonic::LDA, Mode::IndirectX, num))
        })
        .or_else(|_| {
            indirect_y(expr, labels).and_then(|num| ok_byte(&Mnemonic::LDA, Mode::IndirectY, num))
        })
        .or_else(|_| register_x(expr).and_then(|_| ok_none(&Mnemonic::TXA, Mode::Implied)))
        .or_else(|_| register_y(expr).and_then(|_| ok_none(&Mnemonic::TYA, Mode::Implied)))
}

/**
 * Immediate     ADC #$44
 * Zero Page     ADC $44
 * Zero Page,X   ADC $44,X
 * Absolute      ADC $4400
 * Absolute,X    ADC $4400,X
 * Absolute,Y    ADC $4400,Y
 * Indirect,X    ADC ($44,X)
 * Indirect,Y    ADC ($44),Y
 */
fn decode_adc(
    expr: &Expr,

    labels: &HashMap<String, LabelEntry>,
) -> Result<AssemblyInstruction, AssemblyError> {
    plus(expr)
        .and_then(|(left, right)| {
            register_a(&left).and_then(|_| {
                immediate(&right, labels)
                    .and_then(|num| ok_byte(&Mnemonic::ADC, Mode::Immediate, num))
            })
        })
        .or_else(|_| todo!())
}

/**
 * T=A-??? -> CMP ???
 * T=X-??? -> CPX ???
 * T=Y-??? -> CPY ???
 */
pub fn decode_t(
    expr: &Expr,
    labels: &HashMap<String, LabelEntry>,
) -> Result<AssemblyInstruction, AssemblyError> {
    minus(expr)
        .and_then(|(left, right)| {
            register_a(&left)
                .and_then(|_| decode_cmp(&right, labels))
                .or_else(|_| register_x(&left).and_then(|_| decode_cpx(&right, labels)))
                .or_else(|_| register_y(&left).and_then(|_| decode_cpy(&right, labels)))
                .or_else(|_| decode_error(expr))
        })
        .or_else(|_| decode_error(expr))
}

/**
 * CMP (T=A-???)
 * Immediate     CMP #$44
 * Zero Page     CMP $44
 * Zero Page,X   CMP $44,X
 * Absolute      CMP $4400
 * Absolute,X    CMP $4400,X
 * Absolute,Y    CMP $4400,Y
 * Indirect,X    CMP ($44,X)
 * Indirect,Y    CMP ($44),Y
 */
fn decode_cmp(
    expr: &Expr,
    labels: &HashMap<String, LabelEntry>,
) -> Result<AssemblyInstruction, AssemblyError> {
    immediate(expr, labels)
        .and_then(|num| ok_byte(&Mnemonic::CMP, Mode::Immediate, num))
        .or_else(|_| {
            zeropage(expr, labels).and_then(|num| ok_byte(&Mnemonic::CMP, Mode::ZeroPage, num))
        })
        .or_else(|_| {
            zeropage_x(expr, labels).and_then(|num| ok_byte(&Mnemonic::CMP, Mode::ZeroPageX, num))
        })
        .or_else(|_| {
            absolute(expr, labels).and_then(|num| ok_word(&Mnemonic::CMP, Mode::Absolute, num))
        })
        .or_else(|_| {
            absolute_x(expr, labels).and_then(|num| ok_word(&Mnemonic::CMP, Mode::AbsoluteX, num))
        })
        .or_else(|_| {
            absolute_y(expr, labels).and_then(|num| ok_word(&Mnemonic::CMP, Mode::AbsoluteX, num))
        })
        .or_else(|_| {
            indirect_x(expr, labels).and_then(|num| ok_byte(&Mnemonic::CMP, Mode::IndirectX, num))
        })
        .or_else(|_| {
            indirect_y(expr, labels).and_then(|num| ok_byte(&Mnemonic::CMP, Mode::IndirectY, num))
        })
}

/**
 * CPX (T=X=???)
 * Immediate     CPX #$44
 * Zero Page     CPX $44
 * Absolute      CPX $4400
 */
fn decode_cpx(
    expr: &Expr,
    labels: &HashMap<String, LabelEntry>,
) -> Result<AssemblyInstruction, AssemblyError> {
    immediate(expr, labels)
        .and_then(|num| ok_byte(&Mnemonic::CPX, Mode::Immediate, num))
        .or_else(|_| {
            zeropage(expr, labels).and_then(|num| ok_byte(&Mnemonic::CPX, Mode::ZeroPage, num))
        })
        .or_else(|_| {
            absolute(expr, labels).and_then(|num| ok_word(&Mnemonic::CPX, Mode::Absolute, num))
        })
}

/**
 * CPY (T=Y-???)
 * Immediate     CPY #$44
 * Zero Page     CPY $44
 * Absolute      CPY $4400
 */
fn decode_cpy(
    expr: &Expr,
    labels: &HashMap<String, LabelEntry>,
) -> Result<AssemblyInstruction, AssemblyError> {
    immediate(expr, labels)
        .and_then(|num| ok_byte(&Mnemonic::CPY, Mode::Immediate, num))
        .or_else(|_| {
            zeropage(expr, labels).and_then(|num| ok_byte(&Mnemonic::CPY, Mode::ZeroPage, num))
        })
        .or_else(|_| {
            absolute(expr, labels).and_then(|num| ok_word(&Mnemonic::CPY, Mode::Absolute, num))
        })
}

pub fn decode_c(
    expr: &Expr,
    _labels: &HashMap<String, LabelEntry>,
) -> Result<AssemblyInstruction, AssemblyError> {
    decimal(expr).and_then(|num| match num {
        0 => ok_none(&Mnemonic::CLC, Mode::Implied),
        1 => ok_none(&Mnemonic::SEC, Mode::Implied),
        _ => decode_error(expr),
    })
}

pub fn decode_call(
    expr: &Expr,
    _labels: &HashMap<String, LabelEntry>,
) -> Result<AssemblyInstruction, AssemblyError> {
    if let Expr::Identifier(name) = expr {
        return ok_unresolved_label(Mnemonic::JSR, Mode::Absolute, &name);
    }
    if let Expr::WordNum(num) = expr {
        return ok_word(&Mnemonic::JSR, Mode::Absolute, *num);
    }
    decode_error(expr)
}

pub fn decode_goto(
    expr: &Expr,
    _labels: &HashMap<String, LabelEntry>,
) -> Result<AssemblyInstruction, AssemblyError> {
    match expr {
        Expr::WordNum(num) => {
            return ok_word(&Mnemonic::JMP, Mode::Absolute, *num);
        }
        Expr::Identifier(name) => {
            return ok_unresolved_label(Mnemonic::JMP, Mode::Absolute, &name);
        }
        Expr::SystemOperator('!') => return ok_none(&Mnemonic::RTS, Mode::Implied),
        _ => return decode_error(expr),
    }
}

pub fn decode_if(
    expr: &Expr,
    labels: &HashMap<String, LabelEntry>,
) -> Result<AssemblyInstruction, AssemblyError> {
    // ;=/,$12fd (IF NOT EQUAL THEN GOTO $12FD)
    if let Expr::BinOp(expr_left, Operator::Comma, expr_right) = expr {
        if let Expr::SystemOperator(symbol) = **expr_left {
            match **expr_right {
                Expr::WordNum(addr) => match symbol {
                    '\\' => return ok_unresolved_relative(Mnemonic::BNE, Mode::Relative, addr),
                    '=' => return ok_unresolved_relative(Mnemonic::BEQ, Mode::Relative, addr),
                    '>' => return ok_unresolved_relative(Mnemonic::BCS, Mode::Relative, addr),
                    '<' => return ok_unresolved_relative(Mnemonic::BCC, Mode::Relative, addr),
                    _ => (),
                },
                Expr::Identifier(ref name) => match symbol {
                    '/' => return ok_unresolved_label(Mnemonic::BNE, Mode::Relative, name),
                    '=' => return ok_unresolved_label(Mnemonic::BEQ, Mode::Relative, name),
                    '>' => return ok_unresolved_label(Mnemonic::BCS, Mode::Relative, name),
                    '<' => return ok_unresolved_label(Mnemonic::BCC, Mode::Relative, name),

                    _ => (),
                },
                _ => (),
            }
        }
    }
    decode_error(expr)
}

pub fn decode_address(
    command: &Expr,
    expr: &Expr,
    labels: &HashMap<String, LabelEntry>,
) -> Result<AssemblyInstruction, AssemblyError> {
    register_a(expr)
        .and_then(|_| decode_sta(command, labels))
        .or_else(|_| register_x(expr).and_then(|_| decode_stx(command, labels)))
        .or_else(|_| register_y(expr).and_then(|_| decode_sty(command, labels)))
        .or_else(|_| decode_error(expr))
}

/**
 * Zero Page     STA $44    
 * Zero Page,X   STA $44,X  
 * Absolute      STA $4400  
 * Absolute,X    STA $4400,X
 * Absolute,Y    STA $4400,Y
 * Indirect,X    STA ($44,X)
 * Indirect,Y    STA ($44),Y
 */
pub fn decode_sta(
    expr: &Expr,
    labels: &HashMap<String, LabelEntry>,
) -> Result<AssemblyInstruction, AssemblyError> {
    zeropage(expr, labels)
        .and_then(|num| ok_byte(&Mnemonic::STA, Mode::ZeroPage, num))
        .or_else(|_| {
            zeropage_x(expr, labels).and_then(|num| ok_byte(&Mnemonic::STA, Mode::ZeroPageX, num))
        })
        .or_else(|_| {
            absolute(expr, labels).and_then(|num| ok_word(&Mnemonic::STA, Mode::Absolute, num))
        })
        .or_else(|_| {
            absolute_x(expr, labels).and_then(|num| ok_word(&Mnemonic::STA, Mode::AbsoluteX, num))
        })
        .or_else(|_| {
            absolute_y(expr, labels).and_then(|num| ok_word(&Mnemonic::STA, Mode::AbsoluteY, num))
        })
        .or_else(|_| {
            indirect_x(expr, labels).and_then(|num| ok_byte(&Mnemonic::STA, Mode::IndirectX, num))
        })
        .or_else(|_| {
            indirect_y(expr, labels).and_then(|num| ok_byte(&Mnemonic::STA, Mode::IndirectY, num))
        })
}

/**
 * Zero Page     STX $44       $86  2   3
 * Zero Page,Y   STX $44,Y     $96  2   4
 * Absolute      STX $4400     $8E  3   4
 */
pub fn decode_stx(
    expr: &Expr,
    labels: &HashMap<String, LabelEntry>,
) -> Result<AssemblyInstruction, AssemblyError> {
    zeropage(expr, labels)
        .and_then(|num| ok_byte(&Mnemonic::STX, Mode::ZeroPage, num))
        .or_else(|_| {
            zeropage_y(expr, labels).and_then(|num| ok_byte(&Mnemonic::STX, Mode::ZeroPageY, num))
        })
        .or_else(|_| {
            absolute(expr, labels).and_then(|num| ok_word(&Mnemonic::STX, Mode::Absolute, num))
        })
}

/**
 * Zero Page     STY $44       $84  2   3
 * Zero Page,X   STY $44,X     $94  2   4
 * Absolute      STY $4400     $8C  3   4
 */
pub fn decode_sty(
    expr: &Expr,
    labels: &HashMap<String, LabelEntry>,
) -> Result<AssemblyInstruction, AssemblyError> {
    zeropage(expr, labels)
        .and_then(|num| ok_byte(&Mnemonic::STY, Mode::ZeroPage, num))
        .or_else(|_| {
            zeropage_x(expr, labels).and_then(|num| ok_byte(&Mnemonic::STY, Mode::ZeroPageX, num))
        })
        .or_else(|_| {
            absolute(expr, labels).and_then(|num| ok_word(&Mnemonic::STY, Mode::Absolute, num))
        })
}

fn ok_byte(mnemonic: &Mnemonic, mode: Mode, num: u8) -> Result<AssemblyInstruction, AssemblyError> {
    Ok(AssemblyInstruction::new(
        mnemonic.clone(),
        mode,
        OperandValue::Byte(num),
    ))
}

fn ok_word(
    mnemonic: &Mnemonic,
    mode: Mode,
    num: u16,
) -> Result<AssemblyInstruction, AssemblyError> {
    Ok(AssemblyInstruction::new(
        mnemonic.clone(),
        mode,
        OperandValue::Word(num),
    ))
}

fn instruction(mnemonic: Mnemonic, mode: Mode, value: OperandValue) -> AssemblyInstruction {
    AssemblyInstruction::new(mnemonic, mode, value)
}

fn ok_none(mnemonic: &Mnemonic, mode: Mode) -> Result<AssemblyInstruction, AssemblyError> {
    Ok(AssemblyInstruction::new(
        mnemonic.clone(),
        mode,
        OperandValue::None,
    ))
}

fn ok_unresolved_label(
    mnemonic: Mnemonic,
    mode: Mode,
    name: &str,
) -> Result<AssemblyInstruction, AssemblyError> {
    Ok(AssemblyInstruction::new(
        mnemonic,
        mode,
        OperandValue::unresolved_label(name),
    ))
}

fn ok_unresolved_relative(
    mnemonic: Mnemonic,
    mode: Mode,
    addr: u16,
) -> Result<AssemblyInstruction, AssemblyError> {
    Ok(AssemblyInstruction::new(
        mnemonic,
        mode,
        OperandValue::UnresolvedRelative(addr),
    ))
}

fn decode_error(expr: &Expr) -> Result<AssemblyInstruction, AssemblyError> {
    Err(AssemblyError::syntax(&format!(
        "bad expression: {:?}",
        expr
    )))
}

pub fn parenthesized_within<T>(expr: &Expr, decoder: Decoder<T>) -> Result<T, AssemblyError> {
    parenthesized(&expr).and_then(|expr| decoder(&expr))
}

pub fn bracketed_within<T>(expr: &Expr, decoder: Decoder<T>) -> Result<T, AssemblyError> {
    bracketed(&expr).and_then(|expr| decoder(&expr))
}

fn error<T>() -> Result<T, AssemblyError> {
    Err(AssemblyError::DecodeError)
}

/**
 * A=1 or A=$10 or A=label
 */
pub fn immediate(expr: &Expr, labels: &HashMap<String, LabelEntry>) -> Result<u8, AssemblyError> {
    num8bit(expr)
        .and_then(|num| Ok(num))
        .or_else(|_| zeropage_label(expr, labels))
}

fn lookup(name: &str, labels: &HashMap<String, LabelEntry>) -> Result<LabelEntry, AssemblyError> {
    labels
        .get(name)
        .cloned()
        .ok_or(AssemblyError::label_not_found(name))
}

/**
 * A=($1F) or A=(31) or A=(label)
 */
pub fn zeropage(expr: &Expr, labels: &HashMap<String, LabelEntry>) -> Result<u8, AssemblyError> {
    parenthesized_within::<u8>(expr, num8bit)
        // A=($1F) or A=(31)
        .and_then(|num| Ok(num))
        .or_else(|_| parenthesized(expr).and_then(|expr| zeropage_label(&expr, labels)))
}

fn zeropage_label(expr: &Expr, labels: &HashMap<String, LabelEntry>) -> Result<u8, AssemblyError> {
    offset_zeropage_label(expr, labels).or_else(|_| normal_zeropage_label(expr, labels))
}

fn normal_zeropage_label(
    expr: &Expr,
    labels: &HashMap<String, LabelEntry>,
) -> Result<u8, AssemblyError> {
    identifier(expr).and_then(|name| {
        lookup(&name, labels).and_then(|entry| match entry.address {
            Address::ZeroPage(addr) => Ok(addr),
            _ => error(),
        })
    })
}

fn offset_zeropage_label(
    expr: &Expr,
    labels: &HashMap<String, LabelEntry>,
) -> Result<u8, AssemblyError> {
    plus(expr).and_then(|(left, right)| {
        normal_zeropage_label(&left, labels)
            .and_then(|addr| num8bit(&right).and_then(|offset| Ok(addr + offset as u8)))
    })
}

pub fn absolute(expr: &Expr, labels: &HashMap<String, LabelEntry>) -> Result<u16, AssemblyError> {
    parenthesized_within::<u16>(expr, num16bit)
        // A=($1F) or A=(31)
        .and_then(|num| Ok(num))
        .or_else(|_| parenthesized(expr).and_then(|expr| absolute_label(&expr, labels)))
}

fn absolute_label(expr: &Expr, labels: &HashMap<String, LabelEntry>) -> Result<u16, AssemblyError> {
    offset_label(expr, labels).or_else(|_| full_label(expr, labels))
}

fn full_label(expr: &Expr, labels: &HashMap<String, LabelEntry>) -> Result<u16, AssemblyError> {
    identifier(expr).and_then(|name| {
        lookup(&name, labels)
            .and_then(|entry| match entry.address {
                Address::Full(addr) => Ok(addr),
                _ => error(),
            })
            .or_else(|_| Ok(0 as u16))
    })
}

// label+123
fn offset_label(expr: &Expr, labels: &HashMap<String, LabelEntry>) -> Result<u16, AssemblyError> {
    plus(expr).and_then(|(left, right)| {
        full_label(&left, labels)
            .and_then(|addr| num8bit(&right).and_then(|offset| Ok(addr + offset as u16)))
    })
}

/**
 * X=($1F+Y) or X=(31+Y) or X=(label+Y)
 */
pub fn zeropage_y(expr: &Expr, labels: &HashMap<String, LabelEntry>) -> Result<u8, AssemblyError> {
    parenthesized_within(expr, plus).and_then(|(left, right)| {
        register_y(&right)
            .and_then(|_| {
                // X=($1F+Y) or X=(31+Y)
                num8bit(&left).and_then(|num| Ok(num))
            })
            .or_else(|_|
                    // X=(label+Y)
                    zeropage_label(&left, labels).and_then(|addr| Ok(addr)))
    })
}

pub fn zeropage_x(expr: &Expr, labels: &HashMap<String, LabelEntry>) -> Result<u8, AssemblyError> {
    parenthesized_within(expr, plus).and_then(|(left, right)| {
        register_x(&right)
            .and_then(|_| {
                // X=($1F+X) or X=(31+X)
                num8bit(&left).and_then(|num| Ok(num))
            })
            .or_else(|_|
                    // X=(label+X)
                    zeropage_label(&left, labels).and_then(|addr| Ok(addr)))
    })
}

pub fn absolute_y(expr: &Expr, labels: &HashMap<String, LabelEntry>) -> Result<u16, AssemblyError> {
    parenthesized_within(expr, plus).and_then(|(left, right)| {
        register_y(&right)
            .and_then(|_| {
                // X=($12FF+Y) or X=(311+Y)
                num16bit(&left).and_then(|num| Ok(num))
            })
            .or_else(|_|
                    // X=(label+Y)
                    absolute_label(&left, labels).and_then(|addr| Ok(addr)))
    })
}

#[test]
fn test_absolute_y() {
    let mut labels = HashMap::new();
    labels.insert(
        "label".to_string(),
        LabelEntry {
            name: "label".to_string(),
            address: Address::Full(0x1234),
            line: 0,
        },
    );
    let expr = Expr::Parenthesized(Box::new(Expr::BinOp(
        Box::new(Expr::Identifier("label".to_string())),
        Operator::Add,
        Box::new(Expr::Identifier("Y".to_string())),
    )));
    assert_eq!(absolute_y(&expr, &labels), Ok(0x1234));
}

pub fn absolute_x(expr: &Expr, labels: &HashMap<String, LabelEntry>) -> Result<u16, AssemblyError> {
    parenthesized_within(expr, plus).and_then(|(left, right)| {
        register_x(&right)
            .and_then(|_| {
                // X=($12FF+X) or X=(311+X)
                num16bit(&left).and_then(|num| Ok(num))
            })
            .or_else(|_|
                    // X=(label+X)
                    absolute_label(&left, labels).and_then(|addr| Ok(addr)))
    })
}

#[test]
fn test_absolute_x() {
    let mut labels = HashMap::new();
    labels.insert(
        "label".to_string(),
        LabelEntry {
            name: "label".to_string(),
            address: Address::Full(0x1234),
            line: 0,
        },
    );
    // Parenthesized(BinOp(Identifier(\"hello\"), Add, Identifier(\"X\")))
    let expr = Expr::Parenthesized(Box::new(Expr::BinOp(
        Box::new(Expr::Identifier("label".to_string())),
        Operator::Add,
        Box::new(Expr::Identifier("X".to_string())),
    )));
    assert_eq!(absolute_x(&expr, &labels), Ok(0x1234));
}

// Indirect,X    LDA ($44,X)   A=[$44+X]
pub fn indirect_x(expr: &Expr, labels: &HashMap<String, LabelEntry>) -> Result<u8, AssemblyError> {
    bracketed_within(expr, plus).and_then(|(left, right)| {
        register_x(&right)
            .and_then(|_| {
                // A=[$1F+X] or A=[31+X]
                num8bit(&left).and_then(|num| Ok(num))
            })
            .or_else(|_|
                    // A=[label+X]
                    zeropage_label(&left, labels).and_then(|addr| Ok(addr)))
    })
}

// Indirect,Y    LDA ($44),Y   A=[$44]+Y, A=[44]+Y, A=[label]+Y
pub fn indirect_y(expr: &Expr, labels: &HashMap<String, LabelEntry>) -> Result<u8, AssemblyError> {
    plus(expr).and_then(|(ref left, ref right)| {
        // A=[$1F]+Y or A=[31]+Y
        register_y(right).and_then(|_| {
            bracketed(left)
                .and_then(|num| {
                    // A=[$1F]+Y
                    num8bit(&num).and_then(|addr| Ok(addr))
                })
                .or_else(|_| {
                    // A=[label]+Y
                    bracketed(left).and_then(|label| zeropage_label(&label, labels))
                })
        })
    })
}
#[test]
fn test_indirect_y() {
    let mut labels = HashMap::new();
    labels.insert(
        "label".to_string(),
        LabelEntry {
            name: "label".to_string(),
            address: Address::ZeroPage(0x12),
            line: 0,
        },
    );
    let expr = Expr::BinOp(
        Box::new(Expr::Bracketed(Box::new(Expr::Identifier(
            "label".to_string(),
        )))),
        Operator::Add,
        Box::new(Expr::Identifier("Y".to_string())),
    );
    assert_eq!(indirect_y(&expr, &labels), Ok(0x12));
}

/**
 * X=X+1 or X=+
 */
pub fn incr_decrement(expr: &Expr, register_left: &str) -> Result<Operator, AssemblyError> {
    incr_decr_long(expr, register_left).or_else(|_| incr_decr_short(expr))
}

pub fn increment(expr: &Expr, register_left: &str) -> Result<(), AssemblyError> {
    incr_decrement(expr, register_left).and_then(|operator| match operator {
        Operator::Add => Ok(()),
        _ => error(),
    })
}

pub fn decrement(expr: &Expr, register_left: &str) -> Result<(), AssemblyError> {
    incr_decrement(expr, register_left).and_then(|operator| match operator {
        Operator::Sub => Ok(()),
        _ => error(),
    })
}

/**
 * X=X+1
 */
fn incr_decr_long(expr: &Expr, register_left: &str) -> Result<Operator, AssemblyError> {
    binop(expr).and_then(|(left, operator, right)| {
        // X=X+1 or Y=Y+1
        identifier(&left).and_then(|register_right| {
            decimal(&right).and_then(|num| {
                if num == 1 && register_left == register_right {
                    match operator {
                        Operator::Add | Operator::Sub => Ok(operator),
                        _ => error(),
                    }
                } else {
                    error()
                }
            })
        })
    })
}

/**
 * X=+
 */
fn incr_decr_short(expr: &Expr) -> Result<Operator, AssemblyError> {
    sysop(expr).and_then(|symbol| {
        if symbol == '+' {
            Ok(Operator::Add)
        } else if symbol == '-' {
            Ok(Operator::Sub)
        } else {
            error()
        }
    })
}
