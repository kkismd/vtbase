use regex::Regex;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufReader, BufWriter, Write};
use std::process;
mod opcode;

// line of source code
#[derive(Debug)]
pub struct Instruction {
    line_number: usize,
    address: u16,
    label: Option<String>,
    statements: Vec<Statement>,
    object_codes: Vec<u8>,
}

// instruction in a line of source code
#[derive(Debug)]
pub struct Statement {
    instruction: String,
    operand: Option<String>,
}

pub struct LabelTableEntry {
    name: String,
    address: Option<u16>,
    local_labels: Vec<LabelTableEntry>,
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("usage: {} <source_name> <object_file>", args[0]);
        process::exit(1);
    }

    let opcode_table = opcode::initialize_opcode_table();

    for opcode in opcode_table {
        println!(
            "Mnemonic: {:?}, Mode: {:?}, Opcode: {:X}",
            opcode.mnemonic, opcode.addressing_mode, opcode.opcode
        );
    }

    // open source file
    let file = File::open(&args[1])?;
    let instructions = parse_from_file(file);
    let pass2_instructions = assemble(instructions);

    // output bin file
    let output_file = File::create(&args[2])?;
    output_bin(output_file, pass2_instructions)
}

fn output_bin(output_file: File, instructions: Vec<Instruction>) -> io::Result<()> {
    let mut writer = BufWriter::new(output_file);
    for instruction in instructions {
        for object_code in instruction.object_codes {
            writer.write(&[object_code])?;
        }
    }
    Ok(())
}

fn assemble(instructions: Vec<Instruction>) -> Vec<Instruction> {
    let pass1_instructions = assemble_pass1(instructions);
    let pass2_instructions = assemble_pass2(pass1_instructions);
    pass2_instructions
}

fn assemble_pass1(instructions: Vec<Instruction>) -> Vec<Instruction> {
    // TODO: implement
    // ラベルテーブルを作成する
    let mut label_table: Vec<LabelTableEntry> = Vec::new();

    // ラベルをラベルテーブルに登録する
    for instruction in instructions {
        if let Some(label) = instruction.label {
            let entry = LabelTableEntry {
                name: label,
                address: None,
                local_labels: Vec::new(),
            };
            label_table.push(entry);
        }
    }

    //instructions
    Vec::new()
}

fn assemble_pass2(instructions: Vec<Instruction>) -> Vec<Instruction> {
    // TODO: implement
    instructions
}

// make abstract syntax tree from input file
fn parse_from_file(file: File) -> Vec<Instruction> {
    let reader = BufReader::new(file);
    let mut instructions = Vec::new();

    for (num, line) in reader.lines().enumerate() {
        let res = line;
        if let Ok(line) = res {
            let instruction_opt = parse_line(line, num + 1);
            if let Some(instruction) = instruction_opt {
                instructions.push(instruction);
            }
        }
    }
    instructions
}

fn parse_statements(tokens: Vec<String>) -> Vec<Statement> {
    let mut statements = Vec::new();
    for token in tokens {
        let re = Regex::new(r"^(?P<instruction>\S)=(?P<operand>.+)$").unwrap();
        match re.captures(&token) {
            Some(captures) => {
                let instruction = captures.name("instruction").map_or("", |m| m.as_str());
                let operand = captures.name("operand").map(|m| m.as_str());
                statements.push(Statement {
                    instruction: instruction.to_string(),
                    operand: operand.map(String::from),
                });
            }
            None => eprintln!("no match"),
        }
    }
    return statements;
}

// make abstract syntax tree
fn parse_line(line: String, line_num: usize) -> Option<Instruction> {
    // source line format
    let re = Regex::new(r"^(?<label>[.a-z][a-z0-9_]*)?(?<body>\s+.*)?").unwrap();
    return re.captures(&line).map(|captures| {
        let label = captures.name("label").map(|m| m.as_str());
        let body = captures.name("body").map_or("", |m| m.as_str());
        let tokens = tokenize(body);
        let statements = parse_statements(tokens);
        Instruction {
            line_number: line_num,
            address: 0,
            label: label.map(String::from),
            statements: statements,
            object_codes: Vec::new(),
        }
    });
}

fn tokenize(text: &str) -> Vec<String> {
    let re = Regex::new(r#"\S=("[^"]*"|\S+)"#).unwrap();
    let mut tokens = Vec::new();

    for cap in re.captures_iter(text) {
        tokens.push(cap[0].to_string());
    }

    tokens
}
