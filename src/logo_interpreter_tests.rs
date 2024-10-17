use std::path::PathBuf;

use crate::{logo_interpreter, logo_runner};

#[test]
fn test_file() {
    let file_path = "logo_examples/1_10_too_many_args_2_err.lg";
    let content = std::fs::read_to_string(file_path).expect("Unable to read logo file");

    let mut interpreter = logo_interpreter::LogoInterpreter::new(content);
    let mut runner = logo_runner::LogoRunner::new(300, 300);
    println!("result: {:#?}", interpreter.interpret(&mut runner));
    runner
        .save(&PathBuf::from("test.png"))
        .expect("Unable to save image");
}
