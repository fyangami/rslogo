use crate::{logo_interpreter, logo_runner};



#[test]
fn test_file() {
    let content = std::fs::read_to_string("logo_examples/1_00_penup_pendown.lg").expect("Unable to read logo file");

    let mut interpreter = logo_interpreter::LogoInterpreter::new(content);
    let mut runner = logo_runner::LogoRunner::new(300, 300);
    assert!(interpreter.interpret(&mut runner) == Ok(()))
}
