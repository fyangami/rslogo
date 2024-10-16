use crate::logo_interpreter::LogoInterpreter;

pub struct LogoVirtualMachine {
    source_code: Vec<String>,
    interpreter: LogoInterpreter,
    cursor: usize,
}

impl LogoVirtualMachine {
    pub fn new(source_code: Vec<String>, interpreter: LogoInterpreter) -> Self {
        Self {
            source_code,
            interpreter,
            cursor: 0,
        }
    }

    pub fn startup(&self) {}

    pub fn next(&mut self) {
        let line = &self.source_code[self.cursor];
    }

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

enum LogoToken {
    PenUp,
    PenDown,
    Forward,
    Backward,
    Left,
    Right,
    Turn,
    Heading,
    SetX,
    SetY,
    Make,
    Comment,
}

impl LogoToken {
    pub fn as_str(&self) -> &str {
        match self {
            LogoToken::PenUp => "PENUP",
            LogoToken::PenDown => "PENDOWN",
            LogoToken::Forward => "FORWARD",
            LogoToken::Backward => "BACKWARD",
            LogoToken::Left => "LEFT",
            LogoToken::Right => "RIGHT",
            LogoToken::Turn => "TURN",
            LogoToken::Heading => "HEADING",
            LogoToken::SetX => "SETX",
            LogoToken::SetY => "SETY",
            LogoToken::Make => "MAKE",
            LogoToken::Comment => "//",
        }
    }
}
