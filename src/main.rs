pub mod row;

use crate::row::{Table, Row};
use std::io::{self, BufRead, Write};
use std::process;

#[derive(Debug)]
enum Statement {
    Insert,
    Select,
}

fn main() {
    let table = Table::new();
    println!("{:?}", table.num_pages());

    loop {
        print_prompt();

        match read_line() {
            Ok(input) => handle_input(input),
            Err(error) => {
                eprintln!("Error reading input: {:?}", error);
                process::exit(1);
            }
        }
    }
}

fn handle_input(input: String) {
    if input.starts_with(".") {
        do_meta_command(input.as_str())
    } else {
        match parse_statement(input.as_str()) {
            Ok(statement) => println!("Using statement {:?}", statement),
            Err(error) => eprintln!("Error: {}", error),
        }
    }
}

fn do_meta_command(command: &str) {
    match command {
        ".exit" => {
            println!("Exiting...");
            process::exit(0)
        }
        _ => {
            println!("Unknown command: {}", command)
        }
    }
}

fn print_prompt() {
    print!("db> ");
    let _ = io::stdout().flush();
}

fn read_line() -> io::Result<String> {
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let mut input = String::new();
    handle.read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn parse_statement(s: &str) -> Result<Statement, &'static str> {
    match s.trim().to_lowercase().as_str() {
        "insert" => Ok(Statement::Insert),
        "select" => Ok(Statement::Select),
        _ => Err("Unknown statement"),
    }
}
