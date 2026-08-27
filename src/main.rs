#![forbid(unsafe_code)]

use std::process::ExitCode;

use tidyid::{DEFAULT_LENGTH, MAX_LENGTH, MIN_LENGTH, tidyid};

const HELP: &str = "Usage
  tidyid [options]

Options
  -s, --size <SIZE>       Generated ID size (3-256)
  -u, --allow-uppercase   Allow uppercase letters
  -v, --version           Show version number
  -h, --help              Show this help";

fn run() -> Result<(), String> {
    let mut length = DEFAULT_LENGTH;
    let mut allow_uppercase = false;
    let arguments: Vec<String> = std::env::args().skip(1).collect();

    if arguments
        .iter()
        .any(|argument| argument == "-v" || argument == "--version")
    {
        println!("{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    if arguments
        .iter()
        .any(|argument| argument == "-h" || argument == "--help")
    {
        println!("{HELP}");
        return Ok(());
    }

    let mut arguments = arguments.into_iter();
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "-u" | "--allow-uppercase" => allow_uppercase = true,
            "-s" | "--size" => {
                let value = arguments.next().ok_or_else(|| {
                    format!("Size must be an integer between {MIN_LENGTH} and {MAX_LENGTH}")
                })?;
                length = value.parse::<usize>().map_err(|_| {
                    format!("Size must be an integer between {MIN_LENGTH} and {MAX_LENGTH}")
                })?;
                if !(MIN_LENGTH..=MAX_LENGTH).contains(&length) {
                    return Err(format!(
                        "Size must be an integer between {MIN_LENGTH} and {MAX_LENGTH}"
                    ));
                }
            }
            _ => return Err(format!("Unknown argument {argument}")),
        }
    }

    let id = tidyid(length, allow_uppercase).map_err(|error| error.to_string())?;
    println!("{id}");
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
