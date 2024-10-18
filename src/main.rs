use std::process::exit;
use clap::Parser;
mod logo_interpreter;
mod logo_runner;
mod logo_interpreter_tests;
mod keywords;

/// A simple program to parse four arguments using clap.
#[derive(Parser)]
struct Args {
    /// Path to a file
    file_path: std::path::PathBuf,

    /// Path to an svg or png image
    image_path: std::path::PathBuf,

    /// Height
    height: u32,

    /// Width
    width: u32,
}

fn main() -> Result<(), ()> {
    let args: Args = Args::parse();

    // Access the parsed arguments
    let file_path = args.file_path;
    let image_path = args.image_path;
    let height = args.height;
    let width = args.width;
    // read content from file_path
    let content = std::fs::read_to_string(&file_path).expect("Unable to read logo file");
    let mut runner = logo_runner::LogoRunner::new(width, height);
    let mut interpreter = logo_interpreter::LogoInterpreter::default(content);
    match interpreter.interpret(&mut runner) {
        Err(e) => {
            eprintln!("error incurred: {}", e);
            exit(1)
        }
        _ => {
            runner.save(&image_path).expect("save image failed");
        }
    }
    Ok(())
}
