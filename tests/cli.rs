use std::error::Error;
use std::io::Write;
use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::{Command, Stdio};

// #[test]
// fn start_db() -> Result<(), Box<dyn Error>> {
//     let mut cmd = Command::cargo_bin("dbrs")?.spawn()?;
//
//     // Wait for the child process to finish
//     let output = cmd.wait_with_output().unwrap();
//
//     println!("{:?}", output);
//
//
//     Ok(())
// }

#[test]
fn test_dbrs() -> Result<(), Box<dyn Error>> {
    let mut child = Command::new("cargo")
        .args(&["run", "--bin", "dbrs"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let input = ".exit";
    let expected_output = String::from("db> Exiting...\n");

    // Write the input to the stdin of the child process
    child.stdin.as_mut().unwrap().write_all(input.as_bytes()).unwrap();

    // Wait for the child process to finish
    let output = child.wait_with_output().unwrap();
    let str_output = String::from_utf8(output.stdout)?;

    // Check that the output of the child process matches the expected output
    assert_eq!(str_output, expected_output);

    Ok(())
}