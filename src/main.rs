use std::io::{self, BufRead, Write};
use std::process;

fn main() {
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
        todo!("Handle statement")
    }
}

fn do_meta_command(command: &str) {
    match command{
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