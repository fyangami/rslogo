use crate::logo_runner::LogoRunner;

pub struct LogoInterpreter {
    source_code: Vec<String>,
    interpreter: LogoRunner,
    cursor: usize,
}

impl LogoInterpreter {
    pub fn new(source_code: Vec<String>, interpreter: LogoRunner) -> Self {
        Self {
            source_code,
            interpreter,
            cursor: 0,
        }
    }

    pub fn startup(&self) {}

    pub fn eval(&mut self) -> Result<(), String> {
        let line: &str = self.source_code[self.cursor].trim();
        self.cursor += 1;
        if line.is_empty() {
            return Ok(());
        }
        if let Some((t, val)) = line.split_once("\\s") {
            match t {
                "\\\\" => {
                    // ignore the comments
                }
                "PENUP" => {
                    self.interpreter.pen_up();
                }
                "PENDOWN" => {
                    self.interpreter.pen_down();
                }
                "FORWARD" => {
                    self.interpreter
                        .draw_forward(val.parse::<i32>().map_err(|e| e.to_string())?)?;
                }
                "BACKWARD" => {
                    self.interpreter
                        .draw_backward(val.parse::<i32>().map_err(|e| e.to_string())?)?;
                }
                "LEFT" => {
                    self.interpreter
                        .draw_left(val.parse::<i32>().map_err(|e| e.to_string())?)?;
                }
                "RIGHT" => {
                    self.interpreter
                        .draw_right(val.parse::<i32>().map_err(|e| e.to_string())?)?;
                }
                "SETX" => {
                    self.interpreter.set_pos(
                        val.parse::<i32>().map_err(|e| e.to_string())?,
                        self.interpreter.get_pos_y(),
                    );
                }
                "SETY" => {
                    self.interpreter.set_pos(
                        self.interpreter.get_pos_x(),
                        val.parse::<i32>().map_err(|e| e.to_string())?,
                    );
                }
                "TURN" | "HEADING" => {
                    self.interpreter
                        .turn_degree(val.parse::<i32>().map_err(|e| e.to_string())?);
                }
                _ => {
                    // TODO rewrite to result
                    panic!("Unknown token: {}", t)
                }
            }
        }
        todo!("unimplemented")
    }
}
