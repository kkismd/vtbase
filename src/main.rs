use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::process;
mod assembler;
mod error;
mod opcode;
mod parser;
use assembler::Assembler;
use parser::parse_from_file;
use parser::Instruction;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("usage: {} <source_name> <object_file>", args[0]);
        process::exit(1);
    }

    let opcode_table = opcode::initialize_opcode_table();

    // open source file
    let file_open_result = File::open(&args[1]);

    if let Err(e) = file_open_result {
        eprintln!("can't open source file: {}", e);
        process::exit(1);
    }
    // output bin file
    let create_file_result = File::create(&args[2]);
    if let Err(e) = create_file_result {
        eprintln!("can't create object file: {}", e);
        process::exit(1);
    }

    let source_file = file_open_result.unwrap();
    let object_file = create_file_result.unwrap();
    run(&source_file, object_file)
}

fn run(sorce_file: &File, output_file: File) -> Result<(), Box<dyn std::error::Error>> {
    let mut instructions = parse_from_file(sorce_file)?;
    // print insts for debug
    for inst in &instructions {
        eprintln!(
            "line number: {}, address: {}, label: {:?}, statements: {:?}",
            inst.line_number, inst.address, inst.label, inst.statements,
        );
    }
    let mut assembler = Assembler::new();
    assembler.assemble(&mut instructions)?;

    // TODO: オブジェクトサイズを受け取ってプリントする
    output_bin(output_file, instructions)
}

fn output_bin(
    output_file: File,
    instructions: Vec<Instruction>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = BufWriter::new(output_file);
    for instruction in instructions {
        for object_code in instruction.object_codes {
            writer
                .write(&[object_code])
                .map_err(|error| -> Box<dyn std::error::Error> { Box::new(error) })?;
        }
    }
    // TODO: オブジェクトサイズを返す
    Ok(())
}
