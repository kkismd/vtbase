#[derive(Debug, Clone)]
pub enum Mnemonic {
    LDA,
    // 他の二モニックをここに追加します
}

#[derive(Debug)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPageX,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    IndirectX,
    IndirectY,
}
pub type Mode = AddressingMode;

impl AddressingMode {
    pub fn length(&self) -> u8 {
        match self {
            AddressingMode::Immediate => 2,
            AddressingMode::ZeroPage => 2,
            AddressingMode::ZeroPageX => 2,
            AddressingMode::Absolute => 3,
            AddressingMode::AbsoluteX => 3,
            AddressingMode::AbsoluteY => 3,
            AddressingMode::IndirectX => 2,
            AddressingMode::IndirectY => 2,
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
        // 他の二モニックをここに追加します
    ];

    let mut opcode_table = Vec::new();

    for (mnemonic, modes) in mnemonics {
        for (addressing_mode, opcode) in modes {
            opcode_table.push(Opcode::new(mnemonic.clone(), addressing_mode, opcode));
        }
    }

    opcode_table
}
