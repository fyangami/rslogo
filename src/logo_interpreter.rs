use std::collections::HashMap;

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
        self.cursor += token.len();
        if token.trim().is_empty() {
            // TODO
            return Ok(());
        }
        if Self::is_builtin(token.as_str()) {
            let statement = self.collect_statement(Self::get_terminator(token.as_str()));
            self.cursor += statement.len();
            return self.evaluate_builtin_statement(token.as_str(), statement, runner);
        } else {
            // TODO find custom procedure
        }
        todo!("unimplemented")
    }

    fn evaluate_builtin_statement(
        &self,
        token: &str,
        statement: String,
        runner: &mut LogoRunner,
    ) -> Result<(), String> {
        match token {
            "//" => {
                // skip all comments
                return Ok(());
            }
            "PENUP" => {
                runner.pen_up();
            }
            "PENDOWN" => {
                runner.pen_down();
            }
            "FORWARD" => {
                let distance = self.parse_numeric(statement.as_str())?;
                runner.draw_forward(distance)?;
            }
            "BACK" => {
                let distance = self.parse_numeric(statement.as_str())?;
                runner.draw_backward(distance)?;
            }
            "LEFT" => {
                let distance = self.parse_numeric(statement.as_str())?;
                runner.draw_left(distance)?;
            }
            "RIGHT" => {
                let distance = self.parse_numeric(statement.as_str())?;
                runner.draw_right(distance)?;
            }
            "SETX" => {
                let pos_x = self.parse_numeric(statement.as_str())?;
                runner.set_pos(pos_x, runner.get_pos_y());
            }
            "SETY" => {
                let pos_y = self.parse_numeric(statement.as_str())?;
                runner.set_pos(runner.get_pos_x(), pos_y);
            }
            "TURN" | "SETHEADING" => {
                let degree = self.parse_numeric(statement.as_str())?;
                runner.turn_degree(degree);
            }
            "SETPENCOLOR" => {
                let color = self.parse_numeric(statement.as_str())?;
                if color > 15 || color < 0 {
                    return Err("invalid color".to_string());
                }
                runner.set_color(unsvg::COLORS[color as usize]);
            }
            _ => {}
        }
        todo!()
    }

    fn parse_numeric(&self, statement: &str) -> Result<i32, String> {
        let mut chs = statement.chars();
        match chs.next() {
            Some(ch) if ch == '"' => {
                return Ok(chs
                    .as_str()
                    .parse()
                    .map_err(|_| "invalid numeric expression")?)
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
        todo!()
    }

    fn evaluate_numeric_expr(&self, expr: &str) {
        let mut chars = expr.chars();
        loop {
            let mut token = String::new();
            while let Some(ch) = chars.next() {
                if ch == ' ' {
                    break;
                }
                token.push(ch);
            }
            match token.chars().next() {
                Some(ch) if ch == '"' => {}
                Some(ch) if ch == ':' => {}
                _ => {}
            }
        }
    }

    fn collect_statement(&self, terminator: &str) -> String {
        let mut statement = Vec::new();
        let mut cursor = self.cursor;
        let mut matcher = String::new();
        while let Some(ch) = self.source_code.chars().nth(cursor) {
            matcher.push(ch);
            cursor += 1;
            if matcher == terminator {
                break;
            }
            matcher = matcher.split_off(1);
            statement.push(ch);
        }
        statement.iter().collect::<String>()
    }

    fn next_token(&self) -> String {
        let mut token = Vec::new();
        let mut cursor = self.cursor;
        while let Some(ch) = self.source_code.chars().nth(cursor) {
            cursor += 1;
            if ch == ' ' {
                break;
            }
            token.push(ch);
        }
        token.iter().collect::<String>()
    }

    fn is_builtin(token: &str) -> bool {
        match token {
            "PENUP" | "PENDOWN" | "FORWARD" | "BACK" | "LEFT" | "RIGHT" | "SETX" | "SETY"
            | "SETPENCOLOR" | "SETHEADING" | "MAKE" | "TO" | "IF" | "WHILE" | "//" => true,
            _ => false,
        }
    }

    fn get_terminator(token: &str) -> &str {
        match token {
            "PENUP" | "PENDOWN" | "FORWARD" | "BACK" | "LEFT" | "RIGHT" | "SETX" | "SETY"
            | "//" => "\n",
            _ => "END",
        }
    }
}
