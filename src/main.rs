use regex::{Captures, Regex};
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufReader, BufWriter, Write};
use std::process;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("usage: {} <source_name> <object_file>", args[0]);
        process::exit(1);
    }

    // open source file
    let file = File::open(&args[1])?;
    let reader = BufReader::new(file);

    // output bin file
    let output_file = File::create(&args[2])?;
    let mut writer = BufWriter::new(output_file);

    // source line format
    let re = Regex::new(r"^(?<label>[.a-z][a-z0-9_]*)?(?<body>\s+.*)?").unwrap();

    for line in reader.lines() {
        let line = line?;
        if line.len() > 0 {
            eprintln!("------------------------------");
            eprintln!("line: <{}>", line);
            match re.captures(&line) {
                Some(captures) => {
                    eprintln!("captures: {}", captures.len());
                    let label = captures.name("label").map_or("", |m| m.as_str());
                    let body = captures.name("body").map_or("", |m| m.as_str());
                    eprintln!("label: <{}>", label);
                    eprintln!("body:  <{}>", body);

                    let tokens = tokenize(body);
                    for token in tokens {
                        eprintln!("token:  <{}>", token);
                    }
                }
                None => eprintln!("no match"),
            }
        }
        // eprintln!("label is: <{}> (length: {})", cap, cap.len());
    }

    Ok(())
}

fn tokenize(text: &str) -> Vec<String> {
    let re = Regex::new(r#"\S=("[^"]*"|\S+)"#).unwrap();
    let mut tokens = Vec::new();

    for cap in re.captures_iter(text) {
        tokens.push(cap[0].to_string());
    }

    tokens
}
