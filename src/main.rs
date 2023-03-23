use std::io::{self, BufRead, Write};
use std::process;

use crate::row::Row;
use crate::table::Table;

pub mod row;
pub mod table;

const INSERT_CMD: &str = "insert";
const SELECT_CMD: &str = "select";

#[derive(Debug)]
enum Statement {
    Insert(Row),
    Select(String),
}

fn main() {
    let mut table = Table::new();

    loop {
        print_prompt();

        match read_line() {
            Ok(input) => handle_input(input, &mut table),
            Err(error) => {
                eprintln!("Error reading input: {:?}. Please try again.", error);
            }
        }
    }
}

fn handle_input(input: String, table: &mut Table) {
    if input.starts_with(".") {
        do_meta_command(input.as_str())
    } else {
        match parse_statement(input.as_str()) {
            Ok(statement) => do_process_statement(statement, table),
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

fn do_process_statement(statement: Statement, table: &mut Table) {
    match statement {
        Statement::Insert(row) => {
            match table.insert_row(&row) {
                Ok(_) => println!("Row inserted successfully"),
                Err(cause) => println!("Error inserting row: {}", cause)
            }
        }
        Statement::Select(args) => {
            if args.trim().is_empty() {
                for i in 0..table.num_rows() {
                    let row = table.select_row(i).unwrap();
                    println!("{:?}", row);
                }
            } else {
                match args.trim().parse::<usize>() {
                    Ok(row_idx) => print_table_row(&table, row_idx),
                    Err(err) => eprintln!("Error printing row for input '{}': {}", args, err)
                }
            }
        }
    }
}

fn print_table_row(table: &Table, row_idx: usize) {
    let num_rows = table.num_rows();

    if num_rows == 0 {
        println!("Table is empty, nothing to print for index {}", row_idx);
    } else if row_idx >= table.num_rows() {
        println!("Row index out of bounds: {} is not in [0, {}]", row_idx, num_rows)
    } else {
        println!("{:?}", table.select_row(row_idx).unwrap())
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

fn parse_statement(s: &str) -> Result<Statement, String> {
    match s.trim().to_lowercase().split_ascii_whitespace().next().unwrap_or("") {
        INSERT_CMD => {
            match Row::from_string(s[INSERT_CMD.len()..].trim()) {
                Ok(row) => Ok(Statement::Insert(row)),
                Err(e) => Err(format!("Illegal insert statement: {}", e))
            }
        }
        SELECT_CMD => {
            let args = String::from(s[SELECT_CMD.len()..].trim());
            Ok(Statement::Select(args))
        }
        _ => Err("Unknown statement".to_string()),
    }
}
