use std::vec;

#[derive(Debug, Clone)]
pub enum Mnemonic {
    ADC,
    AND,
    ASL,
    BCC,
    BCS,
    BEQ,
    BIT,
    BMI,
    BNE,
    BPL,
    BRK,
    BVC,
    BVS,
    CLC,
    CLD,
    CLI,
    CLV,
    CMP,
    CPX,
    CPY,
    DEC,
    DEX,
    DEY,
    EOR,
    INC,
    INX,
    INY,
    JMP,
    JSR,
    LDA,
    LDX,
    LDY,
    LSR,
    NOP,
    ORA,
    PHA,
    PHP,
    PLA,
    PLP,
    ROL,
    ROR,
    RTI,
    RTS,
    SBC,
    SEC,
    SED,
    SEI,
    STA,
    STX,
    STY,
    TAX,
    TAY,
    TSX,
    TXA,
    TXS,
    TYA,
}

#[derive(Debug)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Relative,
    Indirect,
    IndirectX,
    IndirectY,
    Implied,
    Accumulator,
}
pub type Mode = AddressingMode;

impl AddressingMode {
    pub fn length(&self) -> u8 {
        match self {
            Self::Immediate => 2,
            Self::ZeroPage => 2,
            Self::ZeroPageX => 2,
            Self::ZeroPageY => 2,
            Self::Absolute => 3,
            Self::AbsoluteX => 3,
            Self::AbsoluteY => 3,
            Self::Relative => 2,
            Self::Indirect => 3,
            Self::IndirectX => 2,
            Self::IndirectY => 2,
            Self::Implied => 1,
            Self::Accumulator => 1,
        }
    }
}

pub struct Opcode {
    pub mnemonic: Mnemonic,
    pub addressing_mode: AddressingMode,
    pub opcode: u8,
}

impl Opcode {
    pub fn new(mnemonic: Mnemonic, addressing_mode: AddressingMode, opcode: u8) -> Self {
        Self {
            mnemonic,
            addressing_mode,
            opcode,
        }
    }
}

pub fn initialize_opcode_table() -> Vec<Opcode> {
    let mnemonics = vec![
        (
            Mnemonic::ADC,
            vec![
                (Mode::Immediate, 0x69),
                (Mode::ZeroPage, 0x65),
                (Mode::ZeroPageX, 0x75),
                (Mode::Absolute, 0x6D),
                (Mode::AbsoluteX, 0x7D),
                (Mode::AbsoluteY, 0x79),
                (Mode::IndirectX, 0x61),
                (Mode::IndirectY, 0x71),
            ],
        ),
        (
            Mnemonic::AND,
            vec![
                (Mode::Immediate, 0x29),
                (Mode::ZeroPage, 0x25),
                (Mode::ZeroPageX, 0x35),
                (Mode::Absolute, 0x2D),
                (Mode::AbsoluteX, 0x3D),
                (Mode::AbsoluteY, 0x39),
                (Mode::IndirectX, 0x21),
                (Mode::IndirectY, 0x31),
            ],
        ),
        (
            Mnemonic::ASL,
            vec![
                (Mode::Accumulator, 0x0A),
                (Mode::ZeroPage, 0x06),
                (Mode::ZeroPageX, 0x16),
                (Mode::Absolute, 0x0E),
                (Mode::AbsoluteX, 0x1E),
            ],
        ),
        (Mnemonic::BCC, vec![(Mode::Relative, 0x90)]),
        (Mnemonic::BCS, vec![(Mode::Relative, 0xB0)]),
        (Mnemonic::BEQ, vec![(Mode::Relative, 0xF0)]),
        (
            Mnemonic::BIT,
            vec![(Mode::ZeroPage, 0x24), (Mode::Absolute, 0x2C)],
        ),
        (Mnemonic::BMI, vec![(Mode::Relative, 0x30)]),
        (Mnemonic::BNE, vec![(Mode::Relative, 0xD0)]),
        (Mnemonic::BPL, vec![(Mode::Relative, 0x10)]),
        (Mnemonic::BRK, vec![(Mode::Implied, 0x00)]),
        (Mnemonic::BVC, vec![(Mode::Relative, 0x50)]),
        (Mnemonic::BVS, vec![(Mode::Relative, 0x70)]),
        (Mnemonic::CLC, vec![(Mode::Implied, 0x18)]),
        (Mnemonic::CLD, vec![(Mode::Implied, 0xD8)]),
        (Mnemonic::CLI, vec![(Mode::Implied, 0x58)]),
        (Mnemonic::CLV, vec![(Mode::Implied, 0xB8)]),
        (
            Mnemonic::CMP,
            vec![
                (Mode::Immediate, 0xC9),
                (Mode::ZeroPage, 0xC5),
                (Mode::ZeroPageX, 0xD5),
                (Mode::Absolute, 0xCD),
                (Mode::AbsoluteX, 0xDD),
                (Mode::AbsoluteY, 0xD9),
                (Mode::IndirectX, 0xC1),
                (Mode::IndirectY, 0xD1),
            ],
        ),
        (
            Mnemonic::CPX,
            vec![
                (Mode::Immediate, 0xE0),
                (Mode::ZeroPage, 0xE4),
                (Mode::Absolute, 0xEC),
            ],
        ),
        (
            Mnemonic::CPY,
            vec![
                (Mode::Immediate, 0xC0),
                (Mode::ZeroPage, 0xC4),
                (Mode::Absolute, 0xCC),
            ],
        ),
        (
            Mnemonic::DEC,
            vec![
                (Mode::ZeroPage, 0xC6),
                (Mode::ZeroPageX, 0xD6),
                (Mode::Absolute, 0xCE),
                (Mode::AbsoluteX, 0xDE),
            ],
        ),
        (Mnemonic::DEX, vec![(Mode::Implied, 0xCA)]),
        (Mnemonic::DEY, vec![(Mode::Implied, 0x88)]),
        (
            Mnemonic::EOR,
            vec![
                (Mode::Immediate, 0x49),
                (Mode::ZeroPage, 0x45),
                (Mode::ZeroPageX, 0x55),
                (Mode::Absolute, 0x4D),
                (Mode::AbsoluteX, 0x5D),
                (Mode::AbsoluteY, 0x59),
                (Mode::IndirectX, 0x41),
                (Mode::IndirectY, 0x51),
            ],
        ),
        (
            Mnemonic::INC,
            vec![
                (Mode::ZeroPage, 0xE6),
                (Mode::ZeroPageX, 0xF6),
                (Mode::Absolute, 0xEE),
                (Mode::AbsoluteX, 0xFE),
            ],
        ),
        (Mnemonic::INX, vec![(Mode::Implied, 0xE8)]),
        (Mnemonic::INY, vec![(Mode::Implied, 0xC8)]),
        (
            Mnemonic::JMP,
            vec![(Mode::Absolute, 0x4C), (Mode::Indirect, 0x6C)],
        ),
        (Mnemonic::JSR, vec![(Mode::Absolute, 0x20)]),
        (
            Mnemonic::LDA,
            vec![
                (Mode::Immediate, 0xA9),
                (Mode::ZeroPage, 0xA5),
                (Mode::ZeroPageX, 0xB5),
                (Mode::Absolute, 0xAD),
                (Mode::AbsoluteX, 0xBD),
                (Mode::AbsoluteY, 0xB9),
                (Mode::IndirectX, 0xA1),
                (Mode::IndirectY, 0xB1),
            ],
        ),
        (
            Mnemonic::LDX,
            vec![
                (Mode::Immediate, 0xA2),
                (Mode::ZeroPage, 0xA6),
                (Mode::ZeroPageY, 0xB6),
                (Mode::Absolute, 0xAE),
                (Mode::AbsoluteY, 0xBE),
            ],
        ),
        (
            Mnemonic::LDY,
            vec![
                (Mode::Immediate, 0xA0),
                (Mode::ZeroPage, 0xA4),
                (Mode::ZeroPageX, 0xB4),
                (Mode::Absolute, 0xAC),
                (Mode::AbsoluteX, 0xBC),
            ],
        ),
        (
            Mnemonic::LSR,
            vec![
                (Mode::Accumulator, 0x4A),
                (Mode::ZeroPage, 0x46),
                (Mode::ZeroPageX, 0x56),
                (Mode::Absolute, 0x4E),
                (Mode::AbsoluteX, 0x5E),
            ],
        ),
        (Mnemonic::NOP, vec![(Mode::Implied, 0xEA)]),
        (
            Mnemonic::ORA,
            vec![
                (Mode::Immediate, 0x09),
                (Mode::ZeroPage, 0x05),
                (Mode::ZeroPageX, 0x15),
                (Mode::Absolute, 0x0D),
                (Mode::AbsoluteX, 0x1D),
                (Mode::AbsoluteY, 0x19),
                (Mode::IndirectX, 0x01),
                (Mode::IndirectY, 0x11),
            ],
        ),
        (Mnemonic::PHA, vec![(Mode::Implied, 0x48)]),
        (Mnemonic::PHP, vec![(Mode::Implied, 0x08)]),
        (Mnemonic::PLA, vec![(Mode::Implied, 0x68)]),
        (Mnemonic::PLP, vec![(Mode::Implied, 0x28)]),
        (
            Mnemonic::ROL,
            vec![
                (Mode::Accumulator, 0x2A),
                (Mode::ZeroPage, 0x26),
                (Mode::ZeroPageX, 0x36),
                (Mode::Absolute, 0x2E),
                (Mode::AbsoluteX, 0x3E),
            ],
        ),
        (
            Mnemonic::ROR,
            vec![
                (Mode::Accumulator, 0x6A),
                (Mode::ZeroPage, 0x66),
                (Mode::ZeroPageX, 0x76),
                (Mode::Absolute, 0x6E),
                (Mode::AbsoluteX, 0x7E),
            ],
        ),
        (Mnemonic::RTI, vec![(Mode::Implied, 0x40)]),
        (Mnemonic::RTS, vec![(Mode::Implied, 0x60)]),
        (
            Mnemonic::SBC,
            vec![
                (Mode::Immediate, 0xE9),
                (Mode::ZeroPage, 0xE5),
                (Mode::ZeroPageX, 0xF5),
                (Mode::Absolute, 0xED),
                (Mode::AbsoluteX, 0xFD),
                (Mode::AbsoluteY, 0xF9),
                (Mode::IndirectX, 0xE1),
                (Mode::IndirectY, 0xF1),
            ],
        ),
        (Mnemonic::SEC, vec![(Mode::Implied, 0x38)]),
        (Mnemonic::SED, vec![(Mode::Implied, 0xF8)]),
        (Mnemonic::SEI, vec![(Mode::Implied, 0x78)]),
        (
            Mnemonic::STA,
            vec![
                (Mode::ZeroPage, 0x85),
                (Mode::ZeroPageX, 0x95),
                (Mode::Absolute, 0x8D),
                (Mode::AbsoluteX, 0x9D),
                (Mode::AbsoluteY, 0x99),
                (Mode::IndirectX, 0x81),
                (Mode::IndirectY, 0x91),
            ],
        ),
        (
            Mnemonic::STX,
            vec![
                (Mode::ZeroPage, 0x86),
                (Mode::ZeroPageY, 0x96),
                (Mode::Absolute, 0x8E),
            ],
        ),
        (
            Mnemonic::STY,
            vec![
                (Mode::ZeroPage, 0x84),
                (Mode::ZeroPageX, 0x94),
                (Mode::Absolute, 0x8C),
            ],
        ),
        (Mnemonic::TAX, vec![(Mode::Implied, 0xAA)]),
        (Mnemonic::TAY, vec![(Mode::Implied, 0xA8)]),
        (Mnemonic::TSX, vec![(Mode::Implied, 0xBA)]),
        (Mnemonic::TXA, vec![(Mode::Implied, 0x8A)]),
        (Mnemonic::TXS, vec![(Mode::Implied, 0x9A)]),
        (Mnemonic::TYA, vec![(Mode::Implied, 0x98)]),
    ];

    let mut opcode_table = Vec::new();

    for (mnemonic, modes) in mnemonics {
        for (addressing_mode, opcode) in modes {
            opcode_table.push(Opcode::new(mnemonic.clone(), addressing_mode, opcode));
        }
    }

    opcode_table
}
