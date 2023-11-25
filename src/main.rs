use ihex::Record;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::process;

mod assembler;
mod assembly_macro;
mod error;
mod opcode;
mod parser;
use assembler::Assembler;
use parser::Instruction;

use structopt::StructOpt;

#[derive(StructOpt)]
struct Opt {
    /// Source file
    src_file: String,
    /// Object file
    obj_file: String,
    /// Use Intel HEX format
    #[structopt(long)]
    ihex: bool,
    #[structopt(long)]
    c64: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();

    // open source file
    let file_open_result = File::open(&opt.src_file);
    if let Err(e) = file_open_result {
        eprintln!("can't open source file: {}", e);
        process::exit(1);
    }
    // output bin file
    let create_file_result = File::create(&opt.obj_file);
    if let Err(e) = create_file_result {
        eprintln!("can't create object file: {}", e);
        process::exit(1);
    }

    let source_file = file_open_result.unwrap();
    let object_file = create_file_result.unwrap();
    run(&source_file, object_file, opt)
}

fn run(sorce_file: &File, output_file: File, opt: Opt) -> Result<(), Box<dyn std::error::Error>> {
    let instructions = parser::parse_from_file(sorce_file)?;
    // print insts for debug
    // for inst in &instructions {
    // eprintln!(
    //     "line number: {}, address: {}, label: {:?}, statements: {:?}",
    //     inst.line_number, inst.address, inst.label, inst.statements,
    // );
    // }
    let mut instructions = assembly_macro::expand(&instructions)?;
    let mut assembler = Assembler::new();
    let obj_size = assembler.assemble(&mut instructions)?;
    eprintln!("assemble done. objext size = {} bytes", obj_size);

    if opt.ihex {
        output_ihex(output_file, instructions, assembler.origin)
    } else {
        output_bin(output_file, instructions, opt.c64)
    }
}

fn output_bin(
    output_file: File,
    instructions: Vec<Instruction>,
    c64: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = BufWriter::new(output_file);
    if c64 {
        writer.write(&vec![0x01, 0x08])?;
    }
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

fn output_ihex(
    output_file: File,
    instructions: Vec<Instruction>,
    start_address: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut objects = vec![];
    for instruction in instructions {
        objects.extend(instruction.object_codes);
    }
    let mut writer = BufWriter::new(output_file);
    let result = render_ihex(objects, start_address)?;
    writer.write_all(result.as_bytes())?;
    Ok(())
}

fn render_ihex(objects: Vec<u8>, start_address: u16) -> std::io::Result<String> {
    let mut records = Vec::new();
    let chunk_size = 40;

    for (index, chunk) in objects.chunks(chunk_size).enumerate() {
        records.push(Record::Data {
            offset: start_address + (index * chunk_size) as u16,
            value: chunk.to_vec(),
        });
    }

    records.push(Record::EndOfFile);

    let object = ihex::create_object_file_representation(&records)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    Ok(object)
}
