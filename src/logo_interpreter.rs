use std::{collections::HashMap, f32::consts::E, fmt::format};

use crate::logo_runner::LogoRunner;

pub struct LogoInterpreter {
    source_code: String,
    cursor: usize,
    line_number: usize,
    var_table: HashMap<String, i32>,
}

impl LogoInterpreter {
    pub fn new(source_code: String) -> Self {
        Self {
            source_code,
            cursor: 0,
            line_number: 1,
            var_table: HashMap::new(),
        }
    }

    pub fn interpret(&mut self, runner: &mut LogoRunner) -> Result<(), String> {
        loop {
            if self.cursor >= self.source_code.len() {
                return Ok(());
            }
            self.interpret_statement(runner)?;
        }
    }

    fn interpret_statement(&mut self, runner: &mut LogoRunner) -> Result<(), String> {
        let token = self.next_token();
        if token.len() == 0 {
            self.cursor += 1;
        }
        println!("token: {}", token);
        self.cursor += token.len();
        if token.trim().is_empty() {
            return Ok(());
        }
        let statement = self.collect_statement(Self::get_terminator(&token))?;
        self.cursor += statement.len() + 1;
        match token.as_str() {
            t if Self::is_builtin_fn(t) => {
                self.evaluate_builtin_fn(t, statement, runner)?;
            }
            "MAKE" => {
                self.evaluate_make_statement(statement, runner)?;
            }
            "ADDASIGN" => {
                self.evaluate_add_assign_statement(statement, runner)?;
            }
            _ => {
                // TODO find custom procedure
            }
        }
        if Self::is_builtin_fn(token.as_str()) {
            let statement = self.collect_statement(Self::get_terminator(token.as_str()))?;
            self.cursor += statement.len() + 1;
            return self.evaluate_builtin_fn(token.as_str(), statement, runner);
        } else {
            // TODO find custom procedure
        }
        todo!("unimplemented")
    }

    fn evaluate_builtin_fn(
        &self,
        token: &str,
        statement: String,
        runner: &mut LogoRunner,
    ) -> Result<(), String> {
        let args = statement
            .split_whitespace()
            .filter(|s| !s.is_empty())
            .collect::<Vec<&str>>();
        match token {
            "//" => {
                // skip all comments
                return Ok(());
            }
            "PENUP" => {
                if args.len() > 0 {
                    return Err(format!("PENUP does not take any arguments"));
                }
                runner.pen_up();
            }
            "PENDOWN" => {
                if args.len() > 0 {
                    return Err(format!("PENDOWN does not take any arguments"));
                }
                runner.pen_down();
            }
            "FORWARD" => {
                if args.len() != 1 {
                    return Err(format!("FORWARD takes exactly one argument"));
                }
                let distance = self.parse_numeric(args[0], runner)?;
                runner.draw_forward(distance)?;
            }
            "BACK" => {
                if args.len() != 1 {
                    return Err(format!("BACK takes exactly one argument"));
                }
                let distance = self.parse_numeric(args[0], runner)?;
                runner.draw_backward(distance)?;
            }
            "LEFT" => {
                if args.len() != 1 {
                    return Err(format!("LEFT takes exactly one argument"));
                }
                let distance = self.parse_numeric(args[0], runner)?;
                runner.draw_left(distance)?;
            }
            "RIGHT" => {
                if args.len() != 1 {
                    return Err(format!("RIGHT takes exactly one argument"));
                }
                let distance = self.parse_numeric(args[0], runner)?;
                runner.draw_right(distance)?;
            }
            "SETX" => {
                if args.len() != 1 {
                    return Err(format!("SETX takes exactly one argument"));
                }
                let pos_x = self.parse_numeric(args[0], runner)?;
                runner.set_pos(pos_x, runner.get_pos_y());
            }
            "SETY" => {
                if args.len() != 1 {
                    return Err(format!("SETY takes exactly one argument"));
                }
                let pos_y = self.parse_numeric(args[0], runner)?;
                runner.set_pos(runner.get_pos_x(), pos_y);
            }
            "TURN" | "SETHEADING" => {
                if args.len() != 1 {
                    return Err(format!("TURN/SETHEADING takes exactly one argument"));
                }
                let degree = self.parse_numeric(args[0], runner)?;
                runner.turn_degree(degree);
            }
            "SETPENCOLOR" => {
                if args.len() != 1 {
                    return Err(format!("SETPENCOLOR takes exactly one argument"));
                }
                let color = self.parse_numeric(args[0], runner)?;
                if color > 15 || color < 0 {
                    return Err("invalid color".to_string());
                }
                runner.set_color(color as usize);
            }
            _ => {}
        }
        Ok(())
    }

    fn parse_numeric(&self, statement: &str, runner: &LogoRunner) -> Result<i32, String> {
        let mut chs = statement.trim().chars();
        match chs.next() {
            Some(ch) if ch == '"' => {
                let i32_str = chs.as_str();
                return Ok(i32_str
                    .parse()
                    .map_err(|_| format!("invalid numeric expression: {}", i32_str))?);
            }
            Some(ch) if ch == ':' => {
                return self
                    .var_table
                    .get(chs.as_str())
                    .map(|val| *val)
                    .ok_or("undefined variable".to_string());
            }
            _ => {}
        }
        match statement {
            "XCOR" => Ok(runner.get_pos_x()),
            "YCOR" => Ok(runner.get_pos_y()),
            "HEADING" => Ok(runner.get_direction()),
            "COLOR" => Ok(runner.get_color_index()),
            _ => Err(format!("invalid numeric expression: {}", statement)),
        }
    }

    fn collect_statement(&self, terminator: &str) -> Result<String, String> {
        let mut statement = Vec::new();
        let mut cursor = self.cursor;
        let mut matcher = String::new();
        while let Some(ch) = self.source_code.chars().nth(cursor) {
            matcher.push(ch);
            cursor += 1;
            if matcher == terminator {
                return Ok(statement.iter().collect::<String>());
            }
            matcher = matcher.split_off(1);
            statement.push(ch);
        }
        Err(format!(
            "unterminated statement: {}",
            statement.iter().collect::<String>()
        ))
    }

    fn next_token(&self) -> String {
        let mut token = Vec::new();
        let mut cursor = self.cursor;
        while let Some(ch) = self.source_code.chars().nth(cursor) {
            cursor += 1;
            if ch == ' ' || ch == '\n' {
                break;
            }
            token.push(ch);
        }
        token.iter().collect::<String>()
    }

    fn is_builtin_fn(token: &str) -> bool {
        match token {
            "PENUP" | "PENDOWN" | "FORWARD" | "BACK" | "LEFT" | "RIGHT" | "SETX" | "SETY"
            | "SETPENCOLOR" | "TURN" | "SETHEADING" | "MAKE" | "TO" | "IF" | "WHILE" | "//" => true,
            _ => false,
        }
    }

    fn get_terminator(token: &str) -> &str {
        match token {
            "PENUP" | "PENDOWN" | "FORWARD" | "BACK" | "LEFT" | "RIGHT" | "SETX" | "SETY"
            | "//" | "SETPENCOLOR" | "SETHEADING" | "TURN" | "MAKE" => "\n",
            _ => "END",
        }
    }

    fn evaluate_make_statement(
        &mut self,
        statement: String,
        runner: &LogoRunner,
    ) -> Result<(), String> {
        let args = statement.split_whitespace().collect::<Vec<&str>>();
        if args.len() != 2 {
            return Err("invalid make statement".to_string());
        }
        let var_name = args[0];
        if !var_name.starts_with("\"") {
            return Err("invalid variable name".to_string());
        }
        let val = self.parse_numeric(args[1], runner)?;
        self.var_table.insert(var_name.to_string(), val);
        Ok(())
    }

    fn evaluate_add_assign_statement(
        &mut self,
        statement: String,
        runner: &LogoRunner,
    ) -> Result<(), String> {
        let args = statement.split_whitespace().collect::<Vec<&str>>();
        if args.len() != 2 {
            return Err("invalid add assign statement".to_string());
        }
        let var_name = args[0];
        if !var_name.starts_with("\"") {
            return Err("invalid variable name".to_string());
        }
        if let Some(val) = self.var_table.get(var_name) {
            let new_val = val + self.parse_numeric(args[1], runner)?;
            self.var_table.insert(var_name.to_string(), new_val);
            Ok(())
        } else {
            Err("variable not found".to_string())
        }
    }
}
