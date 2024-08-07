use crate::assembler::LabelTable;
use crate::opcode::{AssemblyInstruction, Mnemonic, Mode, OperandValue};
use crate::{
    assembler::{Address, LabelEntry},
    error::AssemblyError,
    parser::expression::matcher::*,
    parser::expression::{matcher::parenthesized, Expr, Operator},
};

use crate::opcode::AddressingMode::*;
use crate::opcode::Mnemonic::*;

type Decoder<T> = fn(&Expr) -> Result<T, AssemblyError>;

pub fn decode_x(expr: &Expr, labels: &LabelTable) -> Result<AssemblyInstruction, AssemblyError> {
    immediate(expr, labels)
        .and_then(|num| ok_byte(&LDX, Immediate, num))
        .or_else(|_| zeropage(expr, labels).and_then(|num| ok_byte(&LDX, ZeroPage, num)))
        .or_else(|_| zeropage_y(expr, labels).and_then(|num| ok_byte(&LDX, ZeroPageY, num)))
        .or_else(|_| absolute(expr, labels).and_then(|num| ok_word(&LDX, Absolute, num)))
        .or_else(|_| absolute_y(expr, labels).and_then(|num| ok_word(&LDX, AbsoluteY, num)))
        .or_else(|_| increment(expr, "X").and_then(|_| ok_none(&INX, Implied)))
        .or_else(|_| decrement(expr, "X").and_then(|_| ok_none(&DEX, Implied)))
        .or_else(|_| register_a(expr).and_then(|_| ok_none(&TAX, Implied)))
        .or_else(|_| decode_error(expr))
}

pub fn decode_y(expr: &Expr, labels: &LabelTable) -> Result<AssemblyInstruction, AssemblyError> {
    immediate(expr, labels)
        .and_then(|num| ok_byte(&LDY, Immediate, num))
        .or_else(|_| zeropage(expr, labels).and_then(|num| ok_byte(&LDY, ZeroPage, num)))
        .or_else(|_| zeropage_x(expr, labels).and_then(|num| ok_byte(&LDY, ZeroPageX, num)))
        .or_else(|_| absolute(expr, labels).and_then(|num| ok_word(&LDY, Absolute, num)))
        .or_else(|_| absolute_x(expr, labels).and_then(|num| ok_word(&LDY, AbsoluteX, num)))
        .or_else(|_| increment(expr, "Y").and_then(|_| ok_none(&INY, Implied)))
        .or_else(|_| decrement(expr, "Y").and_then(|_| ok_none(&DEY, Implied)))
        .or_else(|_| register_a(expr).and_then(|_| ok_none(&TAY, Implied)))
        .or_else(|_| decode_error(expr))
}

pub fn decode_a(expr: &Expr, labels: &LabelTable) -> Result<AssemblyInstruction, AssemblyError> {
    decode_lda(expr, labels)
        .or_else(|_| decode_adc(expr, labels))
        .or_else(|_| decode_sbc(expr, labels))
        .or_else(|_| decode_ora(expr, labels))
        .or_else(|_| decode_and(expr, labels))
        .or_else(|_| decode_eor(expr, labels))
        .or_else(|_| decode_pop(expr, labels))
        .or_else(|_| decode_shift_a(expr))
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
fn decode_lda(expr: &Expr, labels: &LabelTable) -> Result<AssemblyInstruction, AssemblyError> {
    immediate(expr, labels)
        .and_then(|num| ok_byte(&LDA, Immediate, num))
        .or_else(|_| zeropage(expr, labels).and_then(|num| ok_byte(&LDA, ZeroPage, num)))
        .or_else(|_| zeropage_x(expr, labels).and_then(|num| ok_byte(&LDA, ZeroPageX, num)))
        .or_else(|_| absolute(expr, labels).and_then(|num| ok_word(&LDA, Absolute, num)))
        .or_else(|_| absolute_x(expr, labels).and_then(|num| ok_word(&LDA, AbsoluteX, num)))
        .or_else(|_| absolute_y(expr, labels).and_then(|num| ok_word(&LDA, AbsoluteY, num)))
        .or_else(|_| indirect_x(expr, labels).and_then(|num| ok_byte(&LDA, IndirectX, num)))
        .or_else(|_| indirect_y(expr, labels).and_then(|num| ok_byte(&LDA, IndirectY, num)))
        .or_else(|_| register_x(expr).and_then(|_| ok_none(&TXA, Implied)))
        .or_else(|_| register_y(expr).and_then(|_| ok_none(&TYA, Implied)))
}

/**
 * Immediate     ADC #$44       A=AC+$44
 * Zero Page     ADC $44        A=AC+($44)
 * Zero Page,X   ADC $44,X      A=AC+($44+X)
 * Absolute      ADC $4400      A=AC+($4400)
 * Absolute,X    ADC $4400,X    A=AC+($4400+X)
 * Absolute,Y    ADC $4400,Y    A=AC+($4400+Y)
 * Indirect,X    ADC ($44,X)    A=AC+[$44+X]
 * Indirect,Y    ADC ($44),Y    A=AC+[$44]+Y
 */
fn decode_adc(expr: &Expr, labels: &LabelTable) -> Result<AssemblyInstruction, AssemblyError> {
    plus(expr).and_then(|(left, right)| {
        register_ac(&left).and_then(|_| {
            immediate(&right, labels)
                .and_then(|num| ok_byte(&ADC, Immediate, num))
                .or_else(|_| zeropage(&right, labels).and_then(|num| ok_byte(&ADC, ZeroPage, num)))
                .or_else(|_| {
                    zeropage_x(&right, labels).and_then(|num| ok_byte(&ADC, ZeroPageX, num))
                })
                .or_else(|_| absolute(&right, labels).and_then(|num| ok_word(&ADC, Absolute, num)))
                .or_else(|_| {
                    absolute_x(&right, labels).and_then(|num| ok_word(&ADC, AbsoluteX, num))
                })
                .or_else(|_| {
                    absolute_y(&right, labels).and_then(|num| ok_word(&ADC, AbsoluteX, num))
                })
                .or_else(|_| {
                    indirect_x(&right, labels).and_then(|num| ok_byte(&ADC, IndirectX, num))
                })
                .or_else(|_| {
                    indirect_y(&right, labels).and_then(|num| ok_byte(&ADC, IndirectY, num))
                })
        })
    })
}

/**
 * Immediate     SBC #$44      A=AC-$44
 * Zero Page     SBC $44       A=AC-($44)
 * Zero Page,X   SBC $44,X     A=AC-($44+X)
 * Absolute      SBC $4400     A=AC-($4400)
 * Absolute,X    SBC $4400,X   A=AC-($4400+X)
 * Absolute,Y    SBC $4400,Y   A=AC-($4400+Y)
 * Indirect,X    SBC ($44,X)   A=AC-[$44+X]
 * Indirect,Y    SBC ($44),Y   A=AC-[$44]+Y
 */
fn decode_sbc(expr: &Expr, labels: &LabelTable) -> Result<AssemblyInstruction, AssemblyError> {
    minus(expr).and_then(|(left, right)| {
        register_ac(&left).and_then(|_| {
            immediate(&right, labels)
                .and_then(|num| ok_byte(&SBC, Immediate, num))
                .or_else(|_| zeropage(&right, labels).and_then(|num| ok_byte(&SBC, ZeroPage, num)))
                .or_else(|_| {
                    zeropage_x(&right, labels).and_then(|num| ok_byte(&SBC, ZeroPageX, num))
                })
                .or_else(|_| absolute(&right, labels).and_then(|num| ok_word(&SBC, Absolute, num)))
                .or_else(|_| {
                    absolute_x(&right, labels).and_then(|num| ok_word(&SBC, AbsoluteX, num))
                })
                .or_else(|_| {
                    absolute_y(&right, labels).and_then(|num| ok_word(&SBC, AbsoluteX, num))
                })
                .or_else(|_| {
                    indirect_x(&right, labels).and_then(|num| ok_byte(&SBC, IndirectX, num))
                })
                .or_else(|_| {
                    indirect_y(&right, labels).and_then(|num| ok_byte(&SBC, IndirectY, num))
                })
        })
    })
}

/**
 * Immediate     ORA #$44      $09  2   2
 * Zero Page     ORA $44       $05  2   3
 * Zero Page,X   ORA $44,X     $15  2   4
 * Absolute      ORA $4400     $0D  3   4
 * Absolute,X    ORA $4400,X   $1D  3   4+
 * Absolute,Y    ORA $4400,Y   $19  3   4+
 * Indirect,X    ORA ($44,X)   $01  2   6
 * Indirect,Y    ORA ($44),Y   $11  2   5+
 */
fn decode_ora(expr: &Expr, labels: &LabelTable) -> Result<AssemblyInstruction, AssemblyError> {
    or(expr).and_then(|(left, right)| {
        register_a(&left).and_then(|_| {
            immediate(&right, labels)
                .and_then(|num| ok_byte(&ORA, Immediate, num))
                .or_else(|_| zeropage(&right, labels).and_then(|num| ok_byte(&ORA, ZeroPage, num)))
                .or_else(|_| {
                    zeropage_x(&right, labels).and_then(|num| ok_byte(&ORA, ZeroPageX, num))
                })
                .or_else(|_| absolute(&right, labels).and_then(|num| ok_word(&ORA, Absolute, num)))
                .or_else(|_| {
                    absolute_x(&right, labels).and_then(|num| ok_word(&ORA, AbsoluteX, num))
                })
                .or_else(|_| {
                    absolute_y(&right, labels).and_then(|num| ok_word(&ORA, AbsoluteX, num))
                })
                .or_else(|_| {
                    indirect_x(&right, labels).and_then(|num| ok_byte(&ORA, IndirectX, num))
                })
                .or_else(|_| {
                    indirect_y(&right, labels).and_then(|num| ok_byte(&ORA, IndirectY, num))
                })
        })
    })
}

/**
 * Immediate     AND #$44      $29  2   2
 * Zero Page     AND $44       $25  2   3
 * Zero Page,X   AND $44,X     $35  2   4
 * Absolute      AND $4400     $2D  3   4
 * Absolute,X    AND $4400,X   $3D  3   4+
 * Absolute,Y    AND $4400,Y   $39  3   4+
 * Indirect,X    AND ($44,X)   $21  2   6
 * Indirect,Y    AND ($44),Y   $31  2   5+
 */
fn decode_and(expr: &Expr, labels: &LabelTable) -> Result<AssemblyInstruction, AssemblyError> {
    and(expr).and_then(|(left, right)| {
        register_a(&left).and_then(|_| {
            immediate(&right, labels)
                .and_then(|num| ok_byte(&AND, Immediate, num))
                .or_else(|_| zeropage(&right, labels).and_then(|num| ok_byte(&AND, ZeroPage, num)))
                .or_else(|_| {
                    zeropage_x(&right, labels).and_then(|num| ok_byte(&AND, ZeroPageX, num))
                })
                .or_else(|_| absolute(&right, labels).and_then(|num| ok_word(&AND, Absolute, num)))
                .or_else(|_| {
                    absolute_x(&right, labels).and_then(|num| ok_word(&AND, AbsoluteX, num))
                })
                .or_else(|_| {
                    absolute_y(&right, labels).and_then(|num| ok_word(&AND, AbsoluteX, num))
                })
                .or_else(|_| {
                    indirect_x(&right, labels).and_then(|num| ok_byte(&AND, IndirectX, num))
                })
                .or_else(|_| {
                    indirect_y(&right, labels).and_then(|num| ok_byte(&AND, IndirectY, num))
                })
        })
    })
}

fn decode_eor(expr: &Expr, labels: &LabelTable) -> Result<AssemblyInstruction, AssemblyError> {
    eor(expr).and_then(|(left, right)| {
        register_a(&left).and_then(|_| {
            immediate(&right, labels)
                .and_then(|num| ok_byte(&EOR, Immediate, num))
                .or_else(|_| zeropage(&right, labels).and_then(|num| ok_byte(&EOR, ZeroPage, num)))
                .or_else(|_| {
                    zeropage_x(&right, labels).and_then(|num| ok_byte(&EOR, ZeroPageX, num))
                })
                .or_else(|_| absolute(&right, labels).and_then(|num| ok_word(&EOR, Absolute, num)))
                .or_else(|_| {
                    absolute_x(&right, labels).and_then(|num| ok_word(&EOR, AbsoluteX, num))
                })
                .or_else(|_| {
                    absolute_y(&right, labels).and_then(|num| ok_word(&EOR, AbsoluteX, num))
                })
                .or_else(|_| {
                    indirect_x(&right, labels).and_then(|num| ok_byte(&EOR, IndirectX, num))
                })
                .or_else(|_| {
                    indirect_y(&right, labels).and_then(|num| ok_byte(&EOR, IndirectY, num))
                })
        })
    })
}

fn decode_pop(expr: &Expr, _labels: &LabelTable) -> Result<AssemblyInstruction, AssemblyError> {
    sysop(expr)
        .and_then(|symbol| {
            if symbol == "]" {
                ok_none(&PLA, Implied)
            } else {
                decode_error(expr)
            }
        })
        .or_else(|_| decode_error(expr))
}

/**
 */
fn decode_shift_a(expr: &Expr) -> Result<AssemblyInstruction, AssemblyError> {
    sysop(expr)
        .and_then(|symbol| {
            if symbol == "<" {
                ok_none(&ASL, Accumulator)
            } else if symbol == ">" {
                ok_none(&LSR, Accumulator)
            } else if symbol == "(" {
                ok_none(&ROL, Accumulator)
            } else if symbol == ")" {
                ok_none(&ROR, Accumulator)
            } else {
                decode_error(expr)
            }
        })
        .or_else(|_| decode_error(expr))
}

/**
 * T=A-??? -> CMP ???
 * T=X-??? -> CPX ???
 * T=Y-??? -> CPY ???
 * T=A&??? -> BIT ???
 */
pub fn decode_t(expr: &Expr, labels: &LabelTable) -> Result<AssemblyInstruction, AssemblyError> {
    minus(expr)
        .and_then(|(left, right)| {
            register_a(&left)
                .and_then(|_| decode_cmp(&right, labels))
                .or_else(|_| register_x(&left).and_then(|_| decode_cpx(&right, labels)))
                .or_else(|_| register_y(&left).and_then(|_| decode_cpy(&right, labels)))
                .or_else(|_| decode_error(expr))
        })
        .or_else(|_| {
            and(expr).and_then(|(left, right)| {
                register_a(&left).and_then(|_| decode_bit(&right, labels))
            })
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
fn decode_cmp(expr: &Expr, labels: &LabelTable) -> Result<AssemblyInstruction, AssemblyError> {
    immediate(expr, labels)
        .and_then(|num| ok_byte(&CMP, Immediate, num))
        .or_else(|_| zeropage(expr, labels).and_then(|num| ok_byte(&CMP, ZeroPage, num)))
        .or_else(|_| zeropage_x(expr, labels).and_then(|num| ok_byte(&CMP, ZeroPageX, num)))
        .or_else(|_| absolute(expr, labels).and_then(|num| ok_word(&CMP, Absolute, num)))
        .or_else(|_| absolute_x(expr, labels).and_then(|num| ok_word(&CMP, AbsoluteX, num)))
        .or_else(|_| absolute_y(expr, labels).and_then(|num| ok_word(&CMP, AbsoluteX, num)))
        .or_else(|_| indirect_x(expr, labels).and_then(|num| ok_byte(&CMP, IndirectX, num)))
        .or_else(|_| indirect_y(expr, labels).and_then(|num| ok_byte(&CMP, IndirectY, num)))
}

/**
 * CPX (T=X=???)
 * Immediate     CPX #$44
 * Zero Page     CPX $44
 * Absolute      CPX $4400
 */
fn decode_cpx(expr: &Expr, labels: &LabelTable) -> Result<AssemblyInstruction, AssemblyError> {
    immediate(expr, labels)
        .and_then(|num| ok_byte(&CPX, Immediate, num))
        .or_else(|_| zeropage(expr, labels).and_then(|num| ok_byte(&CPX, ZeroPage, num)))
        .or_else(|_| absolute(expr, labels).and_then(|num| ok_word(&CPX, Absolute, num)))
}

/**
 * CPY (T=Y-???)
 * Immediate     CPY #$44
 * Zero Page     CPY $44
 * Absolute      CPY $4400
 */
fn decode_cpy(expr: &Expr, labels: &LabelTable) -> Result<AssemblyInstruction, AssemblyError> {
    immediate(expr, labels)
        .and_then(|num| ok_byte(&CPY, Immediate, num))
        .or_else(|_| zeropage(expr, labels).and_then(|num| ok_byte(&CPY, ZeroPage, num)))
        .or_else(|_| absolute(expr, labels).and_then(|num| ok_word(&CPY, Absolute, num)))
}

/**
 * Zero Page     BIT $44       $24  2   3
 * Absolute      BIT $4400     $2C  3   4
 */
fn decode_bit(expr: &Expr, labels: &LabelTable) -> Result<AssemblyInstruction, AssemblyError> {
    zeropage(expr, labels)
        .and_then(|num| ok_byte(&BIT, ZeroPage, num))
        .or_else(|_| absolute(expr, labels).and_then(|num| ok_word(&BIT, Absolute, num)))
}

pub fn decode_flags(
    command: &Expr,
    expr: &Expr,
    _labels: &LabelTable,
) -> Result<AssemblyInstruction, AssemblyError> {
    identifier(command).and_then(|register| {
        decimal(expr).and_then(|num| match (register.as_str(), num) {
            ("C", 0) => ok_none(&CLC, Implied),
            ("C", 1) => ok_none(&SEC, Implied),
            ("I", 0) => ok_none(&CLI, Implied),
            ("I", 1) => ok_none(&SEI, Implied),
            ("V", 0) => ok_none(&CLV, Implied),
            ("D", 0) => ok_none(&CLD, Implied),
            ("D", 1) => ok_none(&SED, Implied),
            _ => decode_error(expr),
        })
    })
}

pub fn decode_stack(
    expr: &Expr,
    _labels: &LabelTable,
) -> Result<AssemblyInstruction, AssemblyError> {
    register_x(expr).and_then(|_| ok_none(&TXS, Implied))
}

pub fn decode_nop(expr: &Expr) -> Result<AssemblyInstruction, AssemblyError> {
    match expr {
        Expr::Empty => Ok(AssemblyInstruction::new(NOP, Implied, OperandValue::None)),
        _ => decode_error(expr),
    }
}

/**
 * JSR label -> !=label
 * JSR $12df -> !=$12df
 */
pub fn decode_call(
    expr: &Expr,
    _labels: &LabelTable,
) -> Result<AssemblyInstruction, AssemblyError> {
    identifier(expr)
        .and_then(|name| ok_unresolved_label(JSR, Absolute, &name))
        .or_else(|_| num16bit(expr).and_then(|num| ok_word(&JSR, Absolute, num)))
        .or_else(|_| decode_error(expr))
}

fn sysop_bang(expr: &Expr) -> Result<(), AssemblyError> {
    if let Expr::SystemOperator(c) = expr {
        if c == "!" {
            return Ok(());
        }
    }
    decode_error(expr)
}

fn sysop_tilda(expr: &Expr) -> Result<(), AssemblyError> {
    if let Expr::SystemOperator(c) = expr {
        if c == "~" {
            return Ok(());
        }
    }
    decode_error(expr)
}

pub fn decode_goto(
    expr: &Expr,
    _labels: &LabelTable,
) -> Result<AssemblyInstruction, AssemblyError> {
    identifier(expr)
        .and_then(|name| ok_unresolved_label(JMP, Absolute, &name))
        .or_else(|_| num16bit(expr).and_then(|num| ok_word(&JMP, Absolute, num)))
        .or_else(|_| sysop_bang(expr).and_then(|_| ok_none(&RTS, Implied)))
        .or_else(|_| sysop_tilda(expr).and_then(|_| ok_none(&RTI, Implied)))
        .or_else(|_| decode_error(expr))
}

pub fn decode_if(expr: &Expr, _labels: &LabelTable) -> Result<AssemblyInstruction, AssemblyError> {
    // ;=\,$12fd (IF NOT EQUAL THEN GOTO $12FD)
    comma(expr)
        .and_then(|(left, right)| {
            sysop_or_identifier(&left).and_then(|symbol| {
                if_condition_mnemonic(&symbol).and_then(|mnemonic| {
                    num16bit(&right)
                        .and_then(|addr| ok_unresolved_relative(mnemonic.clone(), Relative, addr))
                        .or_else(|_| {
                            identifier(&right)
                                .and_then(|name| ok_unresolved_label(mnemonic, Relative, &name))
                        })
                })
            })
        })
        .or_else(|_| decode_error(expr))
}

fn if_condition_mnemonic(symbol: &str) -> Result<Mnemonic, AssemblyError> {
    match symbol {
        "\\" | "/" | "!" | "NE" => Ok(BNE),
        "=" | "EQ" | "Z" => Ok(BEQ),
        ">" | "CS" | "GE" => Ok(BCS),
        "<" | "CC" | "LT" => Ok(BCC),
        "-" | "MI" => Ok(BMI),
        "+" | "PL" => Ok(BPL),
        "_" | "VC" => Ok(BVC),
        "^" | "VS" => Ok(BVS),
        _ => decode_error(&Expr::SystemOperator(symbol.to_string())),
    }
}

pub fn decode_shift(
    command: &Expr,
    expr: &Expr,
    labels: &LabelTable,
) -> Result<AssemblyInstruction, AssemblyError> {
    sysop(command)
        .and_then(|symbol| match symbol.as_str() {
            "<" => decode_asl(expr, labels),
            ">" => decode_lsr(expr, labels),
            "(" => decode_rol(expr, labels),
            ")" => decode_ror(expr, labels),
            _ => decode_error(expr),
        })
        .or_else(|_| decode_error(command))
}

pub fn decode_push(expr: &Expr) -> Result<AssemblyInstruction, AssemblyError> {
    register_a(expr)
        .and_then(|_| ok_none(&PHA, Implied))
        .or_else(|_| decode_error(expr))
}

fn decode_asl(expr: &Expr, labels: &LabelTable) -> Result<AssemblyInstruction, AssemblyError> {
    register_a(expr)
        .and_then(|_| ok_none(&ASL, Accumulator))
        .or_else(|_| zeropage(expr, labels).and_then(|num| ok_byte(&ASL, ZeroPage, num)))
        .or_else(|_| zeropage_x(expr, labels).and_then(|num| ok_byte(&ASL, ZeroPageX, num)))
        .or_else(|_| absolute(expr, labels).and_then(|num| ok_word(&ASL, Absolute, num)))
        .or_else(|_| absolute_x(expr, labels).and_then(|num| ok_word(&ASL, AbsoluteX, num)))
        .or_else(|_| decode_error(expr))
}

fn decode_lsr(expr: &Expr, labels: &LabelTable) -> Result<AssemblyInstruction, AssemblyError> {
    register_a(expr)
        .and_then(|_| ok_none(&LSR, Accumulator))
        .or_else(|_| zeropage(expr, labels).and_then(|num| ok_byte(&LSR, ZeroPage, num)))
        .or_else(|_| zeropage_x(expr, labels).and_then(|num| ok_byte(&LSR, ZeroPageX, num)))
        .or_else(|_| absolute(expr, labels).and_then(|num| ok_word(&LSR, Absolute, num)))
        .or_else(|_| absolute_x(expr, labels).and_then(|num| ok_word(&LSR, AbsoluteX, num)))
        .or_else(|_| decode_error(expr))
}

fn decode_rol(expr: &Expr, labels: &LabelTable) -> Result<AssemblyInstruction, AssemblyError> {
    register_a(expr)
        .and_then(|_| ok_none(&ROL, Accumulator))
        .or_else(|_| zeropage(expr, labels).and_then(|num| ok_byte(&ROL, ZeroPage, num)))
        .or_else(|_| zeropage_x(expr, labels).and_then(|num| ok_byte(&ROL, ZeroPageX, num)))
        .or_else(|_| absolute(expr, labels).and_then(|num| ok_word(&ROL, Absolute, num)))
        .or_else(|_| absolute_x(expr, labels).and_then(|num| ok_word(&ROL, AbsoluteX, num)))
        .or_else(|_| decode_error(expr))
}

fn decode_ror(expr: &Expr, labels: &LabelTable) -> Result<AssemblyInstruction, AssemblyError> {
    register_a(expr)
        .and_then(|_| ok_none(&ROR, Accumulator))
        .or_else(|_| zeropage(expr, labels).and_then(|num| ok_byte(&ROR, ZeroPage, num)))
        .or_else(|_| zeropage_x(expr, labels).and_then(|num| ok_byte(&ROR, ZeroPageX, num)))
        .or_else(|_| absolute(expr, labels).and_then(|num| ok_word(&ROR, Absolute, num)))
        .or_else(|_| absolute_x(expr, labels).and_then(|num| ok_word(&ROR, AbsoluteX, num)))
        .or_else(|_| decode_error(expr))
}

pub fn decode_address(
    command: &Expr,
    expr: &Expr,
    labels: &LabelTable,
) -> Result<AssemblyInstruction, AssemblyError> {
    register_a(expr)
        .and_then(|_| decode_sta(command, labels))
        .or_else(|_| register_x(expr).and_then(|_| decode_stx(command, labels)))
        .or_else(|_| register_y(expr).and_then(|_| decode_sty(command, labels)))
        .or_else(|_| {
            sysop(expr).and_then(|op| {
                if op == "+" {
                    decode_inc(command, labels)
                } else if op == "-" {
                    decode_dec(command, labels)
                } else if op == ">" {
                    decode_lsr(command, labels)
                } else if op == "<" {
                    decode_asl(command, labels)
                } else if op == "(" {
                    decode_rol(command, labels)
                } else if op == ")" {
                    decode_ror(command, labels)
                } else {
                    decode_error(expr)
                }
            })
        })
        .or_else(|_| decode_error(expr))
}

fn decode_inc(command: &Expr, labels: &LabelTable) -> Result<AssemblyInstruction, AssemblyError> {
    zeropage(command, labels)
        .and_then(|num| ok_byte(&INC, ZeroPage, num))
        .or_else(|_| zeropage_x(command, labels).and_then(|num| ok_byte(&INC, ZeroPageX, num)))
        .or_else(|_| absolute(command, labels).and_then(|num| ok_word(&INC, Absolute, num)))
        .or_else(|_| absolute_x(command, labels).and_then(|num| ok_word(&INC, AbsoluteX, num)))
        .or_else(|_| decode_error(command))
}

fn decode_dec(command: &Expr, labels: &LabelTable) -> Result<AssemblyInstruction, AssemblyError> {
    zeropage(command, labels)
        .and_then(|num| ok_byte(&DEC, ZeroPage, num))
        .or_else(|_| zeropage_x(command, labels).and_then(|num| ok_byte(&DEC, ZeroPageX, num)))
        .or_else(|_| absolute(command, labels).and_then(|num| ok_word(&DEC, Absolute, num)))
        .or_else(|_| absolute_x(command, labels).and_then(|num| ok_word(&DEC, AbsoluteX, num)))
        .or_else(|_| decode_error(command))
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
pub fn decode_sta(expr: &Expr, labels: &LabelTable) -> Result<AssemblyInstruction, AssemblyError> {
    zeropage(expr, labels)
        .and_then(|num| ok_byte(&STA, ZeroPage, num))
        .or_else(|_| zeropage_x(expr, labels).and_then(|num| ok_byte(&STA, ZeroPageX, num)))
        .or_else(|_| absolute(expr, labels).and_then(|num| ok_word(&STA, Absolute, num)))
        .or_else(|_| absolute_x(expr, labels).and_then(|num| ok_word(&STA, AbsoluteX, num)))
        .or_else(|_| absolute_y(expr, labels).and_then(|num| ok_word(&STA, AbsoluteY, num)))
        .or_else(|_| indirect_x(expr, labels).and_then(|num| ok_byte(&STA, IndirectX, num)))
        .or_else(|_| indirect_y(expr, labels).and_then(|num| ok_byte(&STA, IndirectY, num)))
}

/**
 * Zero Page     STX $44       $86  2   3
 * Zero Page,Y   STX $44,Y     $96  2   4
 * Absolute      STX $4400     $8E  3   4
 */
pub fn decode_stx(expr: &Expr, labels: &LabelTable) -> Result<AssemblyInstruction, AssemblyError> {
    zeropage(expr, labels)
        .and_then(|num| ok_byte(&STX, ZeroPage, num))
        .or_else(|_| zeropage_y(expr, labels).and_then(|num| ok_byte(&STX, ZeroPageY, num)))
        .or_else(|_| absolute(expr, labels).and_then(|num| ok_word(&STX, Absolute, num)))
}

/**
 * Zero Page     STY $44       $84  2   3
 * Zero Page,X   STY $44,X     $94  2   4
 * Absolute      STY $4400     $8C  3   4
 */
pub fn decode_sty(expr: &Expr, labels: &LabelTable) -> Result<AssemblyInstruction, AssemblyError> {
    zeropage(expr, labels)
        .and_then(|num| ok_byte(&STY, ZeroPage, num))
        .or_else(|_| zeropage_x(expr, labels).and_then(|num| ok_byte(&STY, ZeroPageX, num)))
        .or_else(|_| absolute(expr, labels).and_then(|num| ok_word(&STY, Absolute, num)))
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

fn decode_error<T>(expr: &Expr) -> Result<T, AssemblyError> {
    Err(AssemblyError::decode_failed(&format!("{:?}", expr)))
}

pub fn parenthesized_within<T>(expr: &Expr, decoder: Decoder<T>) -> Result<T, AssemblyError> {
    parenthesized(expr).and_then(|expr| decoder(&expr))
}

pub fn bracketed_within<T>(expr: &Expr, decoder: Decoder<T>) -> Result<T, AssemblyError> {
    bracketed(expr).and_then(|expr| decoder(&expr))
}

/**
 * A=1 or A=$10 or A=label or A=<label or A=>label
 */
pub fn immediate(expr: &Expr, labels: &LabelTable) -> Result<u8, AssemblyError> {
    num8bit(expr)
        .or_else(|_| zeropage_label(expr, labels))
        .or_else(|_| hi_label(expr, labels))
        .or_else(|_| lo_label(expr, labels))
        .or_else(|_| decode_error(expr))
}

fn hi_label(expr: &Expr, labels: &LabelTable) -> Result<u8, AssemblyError> {
    hi(expr).and_then(|label| {
        identifier(&label).and_then(|name| {
            lookup(&name, labels)
                .and_then(|entry| match entry.address {
                    Address::Full(addr) => Ok((addr >> 8) as u8),
                    _ => decode_error(expr),
                })
                .or(Ok(0_u8))
        })
    })
}

fn lo_label(expr: &Expr, labels: &LabelTable) -> Result<u8, AssemblyError> {
    lo(expr).and_then(|label| {
        identifier(&label).and_then(|name| {
            lookup(&name, labels)
                .and_then(|entry| match entry.address {
                    Address::Full(addr) => Ok((addr & 0xff) as u8),
                    _ => decode_error(expr),
                })
                .or(Ok(0_u8))
        })
    })
}

fn lookup(name: &str, labels: &LabelTable) -> Result<LabelEntry, AssemblyError> {
    labels
        .get(name)
        .cloned()
        .ok_or(AssemblyError::label_not_found(name))
}

/**
 * A=($1F) or A=(31) or A=(label)
 */
pub fn zeropage(expr: &Expr, labels: &LabelTable) -> Result<u8, AssemblyError> {
    parenthesized_within::<u8>(expr, num8bit)
        // A=($1F) or A=(31)
        .or_else(|_| parenthesized(expr).and_then(|expr| zeropage_label(&expr, labels)))
}

fn zeropage_label(expr: &Expr, labels: &LabelTable) -> Result<u8, AssemblyError> {
    offset_zeropage_label(expr, labels).or_else(|_| normal_zeropage_label(expr, labels))
}

fn normal_zeropage_label(expr: &Expr, labels: &LabelTable) -> Result<u8, AssemblyError> {
    identifier(expr).and_then(|name| {
        lookup(&name, labels).and_then(|entry| match entry.address {
            Address::ZeroPage(addr) => Ok(addr),
            _ => decode_error(expr),
        })
    })
}

fn offset_zeropage_label(expr: &Expr, labels: &LabelTable) -> Result<u8, AssemblyError> {
    plus(expr).and_then(|(left, right)| {
        normal_zeropage_label(&left, labels)
            .and_then(|addr| num8bit(&right).map(|offset| addr + offset))
    })
}

pub fn absolute(expr: &Expr, labels: &LabelTable) -> Result<u16, AssemblyError> {
    parenthesized_within::<u16>(expr, num16bit)
        .or_else(|_| parenthesized(expr).and_then(|expr| absolute_label(&expr, labels)))
}

fn absolute_label(expr: &Expr, labels: &LabelTable) -> Result<u16, AssemblyError> {
    offset_label(expr, labels).or_else(|_| full_label(expr, labels))
}

fn full_label(expr: &Expr, labels: &LabelTable) -> Result<u16, AssemblyError> {
    identifier(expr).and_then(|name| {
        lookup(&name, labels)
            .and_then(|entry| match entry.address {
                Address::Full(addr) => Ok(addr),
                _ => decode_error(expr),
            })
            .or(Ok(0_u16))
    })
}

// label+123
fn offset_label(expr: &Expr, labels: &LabelTable) -> Result<u16, AssemblyError> {
    expr.calculate_address(labels).and_then(|addr| match addr {
        Address::Full(addr) => Ok(addr),
        _ => decode_error(expr),
    })
}

/**
 * X=($1F+Y) or X=(31+Y) or X=(label+Y)
 */
pub fn zeropage_y(expr: &Expr, labels: &LabelTable) -> Result<u8, AssemblyError> {
    parenthesized_within(expr, plus).and_then(|(left, right)| {
        register_y(&right).and_then(|_| num8bit(&left).or_else(|_| zeropage_label(&left, labels)))
    })
}

pub fn zeropage_x(expr: &Expr, labels: &LabelTable) -> Result<u8, AssemblyError> {
    parenthesized_within(expr, plus).and_then(|(left, right)| {
        register_x(&right).and_then(|_| num8bit(&left).or_else(|_| zeropage_label(&left, labels)))
    })
}

pub fn absolute_y(expr: &Expr, labels: &LabelTable) -> Result<u16, AssemblyError> {
    parenthesized_within(expr, plus).and_then(|(left, right)| {
        register_y(&right).and_then(|_| num16bit(&left).or_else(|_| absolute_label(&left, labels)))
    })
}

pub fn absolute_x(expr: &Expr, labels: &LabelTable) -> Result<u16, AssemblyError> {
    parenthesized_within(expr, plus).and_then(|(left, right)| {
        register_x(&right).and_then(|_| num16bit(&left).or_else(|_| absolute_label(&left, labels)))
    })
}

// Indirect,X    LDA ($44,X)   A=[$44+X]
pub fn indirect_x(expr: &Expr, labels: &LabelTable) -> Result<u8, AssemblyError> {
    bracketed_within(expr, plus).and_then(|(left, right)| {
        register_x(&right).and_then(|_| num8bit(&left).or_else(|_| zeropage_label(&left, labels)))
    })
}

// Indirect,Y    LDA ($44),Y   A=[$44]+Y, A=[44]+Y, A=[label]+Y
pub fn indirect_y(expr: &Expr, labels: &LabelTable) -> Result<u8, AssemblyError> {
    plus(expr).and_then(|(ref left, ref right)| {
        // A=[$1F]+Y or A=[31]+Y
        register_y(right).and_then(|_| {
            bracketed(left).and_then(|num| num8bit(&num).or_else(|_| zeropage_label(&num, labels)))
        })
    })
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
        _ => decode_error(expr),
    })
}

pub fn decrement(expr: &Expr, register_left: &str) -> Result<(), AssemblyError> {
    incr_decrement(expr, register_left).and_then(|operator| match operator {
        Operator::Sub => Ok(()),
        _ => decode_error(expr),
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
                        _ => decode_error(expr),
                    }
                } else {
                    decode_error(expr)
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
        if symbol == "+" {
            Ok(Operator::Add)
        } else if symbol == "-" {
            Ok(Operator::Sub)
        } else {
            decode_error(expr)
        }
    })
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_absolute_y() {
        let mut labels = LabelTable::new();
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

    #[test]
    fn test_absolute_x() {
        let mut labels = LabelTable::new();
        labels.insert(
            "label".to_string(),
            LabelEntry {
                name: "label".to_string(),
                address: Address::Full(0x1234),
                line: 0,
            },
        );
        // Parenthesized(BinOp(Identifier(\"hello\"), Add, Identifier(\"X\")))
        let expr = Expr::parse("(label+X)").unwrap();
        assert_eq!(absolute_x(&expr, &labels), Ok(0x1234));
    }

    #[test]
    fn test_address_absolute_x_0x0000() {
        let labels = LabelTable::new();
        let expr = Expr::Parenthesized(Box::new(Expr::BinOp(
            Box::new(Expr::WordNum(0x0000)),
            Operator::Add,
            Box::new(Expr::Identifier("X".to_string())),
        )));
        assert_eq!(absolute_x(&expr, &labels), Ok(0x0000));
    }

    #[test]
    fn test_indirect_y() {
        let mut labels = LabelTable::new();
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

    #[test]
    fn test_hi_label() {
        let mut labels = LabelTable::new();
        labels.insert(
            "label".to_string(),
            LabelEntry {
                name: "label".to_string(),
                address: Address::Full(0x1234),
                line: 0,
            },
        );
        let expr = Expr::HiByte(Box::new(Expr::Identifier("label".to_string())));
        assert_eq!(hi_label(&expr, &labels), Ok(0x12));
    }

    #[test]
    fn test_lo_label() {
        let mut labels = LabelTable::new();
        labels.insert(
            "label".to_string(),
            LabelEntry {
                name: "label".to_string(),
                address: Address::Full(0x1234),
                line: 0,
            },
        );
        let expr = Expr::LoByte(Box::new(Expr::Identifier("label".to_string())));
        assert_eq!(lo_label(&expr, &labels), Ok(0x34));
    }

    #[test]
    fn test_lda_lo_label() {
        let mut labels = LabelTable::new();
        labels.insert(
            "label".to_string(),
            LabelEntry {
                name: "label".to_string(),
                address: Address::Full(0x1234),
                line: 0,
            },
        );
        let (rest, expr) = crate::parser::expression::parse_lobyte("<label").unwrap();
        assert_eq!(
            decode_a(&expr, &labels).unwrap(),
            AssemblyInstruction::new(LDA, Immediate, OperandValue::Byte(0x34))
        );
        assert_eq!(rest, "");
    }
}
