use crate::logo_runner::LogoRunner;
use std::collections::HashMap;

pub struct LogoInterpreter {
    source_code: String,
    cursor: usize,
    line_number: usize,
    // ignore the contention in single thread
    var_table: Box<HashMap<String, String>>,
}

impl LogoInterpreter {
    pub fn default(source_code: String) -> Self {
        Self::new(source_code, Box::new(HashMap::new()))
    }

    pub fn new(source_code: String, var_table: Box<HashMap<String, String>>) -> Self {
        Self {
            source_code,
            cursor: 0,
            line_number: 1,
            var_table,
        }
    }

    pub fn interpret(&mut self, runner: &mut LogoRunner) -> Result<(), String> {
        loop {
            if self.cursor >= self.source_code.len() {
                return Ok(());
            }
            self.interpret_expr(runner)?;
        }
    }

    fn interpret_expr(&mut self, runner: &mut LogoRunner) -> Result<(), String> {
        let token = self.next_token();
        if token.len() == 0 {
            self.cursor += 1;
        }
        // println!("token: {}", token);
        self.cursor += token.len();
        if token.trim().is_empty() {
            return Ok(());
        }
        let expr = self.collect_expr(Self::get_terminator(&token))?;
        self.cursor += expr.len() + 1;
        match token.as_str() {
            t if Self::is_builtin_fn(t) => {
                return self.evaluate_builtin_fn(t, expr, runner);
            }
            "MAKE" => {
                return self.evaluate_make_statement(expr, runner);
            }
            "ADDASIGN" => {
                return self.evaluate_add_assign_statement(expr, runner, false);
            }
            "IF" | "WHILE" => {
                return self.evaluate_conditional_statement(token, expr, runner);
            }
            "TO" => {
                todo!()
            }
            _ => {
                // TODO find custom procedure
                todo!("unimplemented token")
            }
        }
    }

    fn evaluate_builtin_fn(
        &self,
        token: &str,
        expr: String,
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
                let distance = self
                    .evaluate_expr(&expr, runner)?
                    .parse()
                    .map_err(|_| format!("invalid argument: {}", expr))?;
                runner.draw_forward(distance)?;
            }
            "BACK" => {
                let distance = self
                    .evaluate_expr(&expr, runner)?
                    .parse()
                    .map_err(|_| format!("invalid argument: {}", expr))?;
                runner.draw_backward(distance)?;
            }
            "LEFT" => {
                let distance = self
                    .evaluate_expr(&expr, runner)?
                    .parse()
                    .map_err(|_| format!("invalid argument: {}", expr))?;
                runner.draw_left(distance)?;
            }
            "RIGHT" => {
                let distance = self
                    .evaluate_expr(&expr, runner)?
                    .parse()
                    .map_err(|_| format!("invalid argument: {}", expr))?;
                runner.draw_right(distance)?;
            }
            "SETX" => {
                let pos_x = self
                    .evaluate_expr(&expr, runner)?
                    .parse()
                    .map_err(|_| format!("invalid argument: {}", expr))?;
                runner.set_pos(pos_x, runner.get_pos_y());
            }
            "SETY" => {
                let pos_y = self
                    .evaluate_expr(&expr, runner)?
                    .parse()
                    .map_err(|_| format!("invalid argument: {}", expr))?;
                runner.set_pos(runner.get_pos_x(), pos_y);
            }
            "TURN" | "SETHEADING" => {
                let degree = self
                    .evaluate_expr(&expr, runner)?
                    .parse()
                    .map_err(|_| format!("invalid argument: {}", expr))?;
                runner.turn_degree(degree);
            }
            "SETPENCOLOR" => {
                let color: i32 = self
                    .evaluate_expr(&expr, runner)?
                    .parse()
                    .map_err(|_| format!("invalid argument: {}", expr))?;
                if color > 15 || color < 0 {
                    return Err("invalid color".to_string());
                }
                runner.set_color(color as usize);
            }
            _ => {}
        }
        Ok(())
    }

    fn evaluate_expr(&self, expr: &str, runner: &LogoRunner) -> Result<String, String> {
        let mut items = expr.split_whitespace().collect::<Vec<&str>>();
        let mut stack: Vec<String> = Vec::new();
        items.reverse();
        for item in items {
            match item {
                "+" | "-" | "*" | "/" => {
                    let left = stack
                        .pop()
                        .ok_or(format!("invalid expression, stack underflow : {}", expr))?
                        .parse::<i32>()
                        .map_err(|e| {
                            format!("invalid expression, right operand is not a number: {}", e)
                        })?;
                    let right = stack
                        .pop()
                        .ok_or(format!("invalid expression, stack underflow : {}", expr))?
                        .parse::<i32>()
                        .map_err(|e| {
                            format!("invalid expression, right operand is not a number: {}", e)
                        })?;

                    let result = match item {
                        "+" => left + right,
                        "-" => left - right,
                        "*" => left * right,
                        "/" => left / right,
                        _ => unreachable!(),
                    };
                    stack.push(result.to_string());
                }
                "EQ" | "NE" | "LT" | "GT" => {
                    let left = stack
                        .pop()
                        .ok_or(format!("invalid expression, stack underflow : {}", expr))?;
                    let right = stack
                        .pop()
                        .ok_or(format!("invalid expression, stack underflow : {}", expr))?;
                    let result = self.logical_op(&left, &right, item)?;
                    stack.push(result.to_string().to_uppercase());
                }
                item if item.starts_with("\"") => {
                    // literal
                    if let Some(literal) = item.strip_prefix("\"") {
                        stack.push(literal.to_string());
                        continue;
                    }
                    return Err(format!("invalid literal: {}", item));
                }
                item if item.starts_with(":") => {
                    // variable
                    if let Some(var_name) = item.strip_prefix(":") {
                        if let Some(var_value) = self.var_table.get(var_name) {
                            stack.push(var_value.to_string());
                            continue;
                        }
                    }
                    return Err(format!("undefined variable: {}", item));
                }
                "XCOR" => stack.push(runner.get_pos_x().to_string()),
                "YCOR" => stack.push(runner.get_pos_y().to_string()),
                "HEADING" => stack.push(runner.get_direction().to_string()),
                "COLOR" => stack.push(runner.get_color_index().to_string()),
                _ => {}
            }
        }
        if stack.len() != 1 {
            return Err(format!("invalid expression: {}", expr));
        }
        Ok(stack.pop().map(|item| item.to_string()).unwrap())
    }

    fn logical_op(&self, left: &str, right: &str, op: &str) -> Result<bool, String> {
        // boolean comparison
        if left == "TRUE" || left == "FALSE" || right == "TRUE" || right == "FALSE" {
            if left != "TRUE" && left != "FALSE" {
                return Err(format!(
                    "invalid expression, right operand is not a boolean: {}",
                    left
                ));
            }
            if right != "TRUE" && right != "FALSE" {
                return Err(format!(
                    "invalid expression, right operand is not a boolean: {}",
                    right
                ));
            }
            let left = left == "TRUE";
            let right = right == "TRUE";
            match op {
                "EQ" => return Ok(left == right),
                "NE" => return Ok(left != right),
                _ => return Err(format!("invalid operator: {}", op)),
            }
        }
        // numeric comparison
        let left = left
            .parse::<i32>()
            .map_err(|_| format!("invalid expression: {}", left))?;
        let right = right
            .parse::<i32>()
            .map_err(|_| format!("invalid expression: {}", right))?;
        match op {
            "GT" => return Ok(left > right),
            "LT" => return Ok(left < right),
            _ => return Err(format!("invalid operator: {}", op)),
        }
    }

    fn collect_expr(&self, terminator: &str) -> Result<String, String> {
        let mut expr = Vec::new();
        let mut cursor = self.cursor;
        let mut matcher = String::new();
        while let Some(ch) = self.source_code.chars().nth(cursor) {
            matcher.push(ch);
            cursor += 1;
            if matcher == terminator {
                return Ok(expr.iter().collect::<String>());
            }
            matcher = matcher.split_off(1);
            expr.push(ch);
        }
        Err(format!(
            "unterminated statement: {}",
            expr.iter().collect::<String>()
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
            | "SETPENCOLOR" | "TURN" | "SETHEADING" | "//" => true,
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

    fn evaluate_make_statement(&mut self, expr: String, runner: &LogoRunner) -> Result<(), String> {
        self.evaluate_add_assign_statement(expr, runner, true)
    }

    fn evaluate_add_assign_statement(
        &mut self,
        expr: String,
        runner: &LogoRunner,
        initialize: bool,
    ) -> Result<(), String> {
        if let Some((var_name, val)) = expr.split_once(" ") {
            let mut parsed_var_name = "";
            if var_name.starts_with("\"") {
                if let Some(var_name) = var_name.strip_prefix("\"") {
                    parsed_var_name = var_name;
                }
            }
            if var_name.starts_with(":") {
                if let Some(var_name) = var_name.strip_prefix(":") {
                    parsed_var_name = self.var_table.get(var_name).ok_or("variable not found")?;
                }
            }
            if parsed_var_name.is_empty() {
                return Err(format!("invalid variable name: {}", var_name));
            }
            let mut old_val = "";
            if let Some(val) = self.var_table.get(parsed_var_name) {
                old_val = val;
            }
            if old_val.is_empty() && !initialize {
                if initialize {
                    old_val = "0"
                } else {
                    return Err(format!("variable not initialized: {}", parsed_var_name));
                }
            }
            let val = self.evaluate_expr(val, runner)?;
            let new_val = old_val
                .parse::<i32>()
                .map_err(|_| "invalid numeric value")?
                + val.parse::<i32>().map_err(|_| "invalid numeric value")?;
            self.var_table
                .insert(parsed_var_name.to_string(), new_val.to_string());
            return Ok(());
        }
        Err(format!("invalid add assign statement: {}", expr))
    }

    fn evaluate_conditional_statement(
        &self,
        token: String,
        expr: String,
        runner: &mut LogoRunner,
    ) -> Result<(), String> {
        let oneshot = token == "IF";
        if let Some((cond_expr, body)) = expr.split_once("\n") {
            loop {
                let result = self.evaluate_expr(cond_expr, runner)?;
                if result == "TRUE" {
                    // condition is true, execute the body
                    if let Some(body_code) = body.trim().strip_suffix("END") {
                        let mut interpreter =
                            LogoInterpreter::new(body_code.to_string(), self.var_table.clone());
                        interpreter.interpret(runner)?;
                    }
                } else {
                    return Ok(());
                }
                if oneshot {
                    return Ok(());
                }
            }
        }
        return Err(format!("invalid conditional statement: {}", expr));
    }
}
