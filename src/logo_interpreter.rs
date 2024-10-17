use std::collections::HashMap;
use crate::logo_runner::LogoRunner;

pub struct LogoInterpreter {
    source_code: String,
    cursor: usize,
    line_number: usize,
    var_table: HashMap<String, String>,
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
                return self.evaluate_add_assign_statement(expr, runner);
            }
            _ => {
                // TODO find custom procedure
                todo!("unimplemented")
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

    fn evaluate_make_statement(
        &mut self,
        expr: String,
        runner: &LogoRunner,
    ) -> Result<(), String> {
        let args = expr.split_whitespace().collect::<Vec<&str>>();
        if args.len() != 2 {
            return Err("invalid make statement".to_string());
        }
        let var_name = expr;
        if !var_name.starts_with("\"") {
            return Err("invalid variable name".to_string());
        }
        let var_name = expr.chars().skip(1).collect::<String>();
        let val = self.parse_numeric(args[1], runner)?;
        self.var_table.insert(var_name, val.to_string());
        Ok(())
    }

    fn evaluate_add_assign_statement(
        &mut self,
        expr: String,
        runner: &LogoRunner,
    ) -> Result<(), String> {
        let args = expr.split_whitespace().collect::<Vec<&str>>();
        if args.len() != 2 {
            return Err("invalid add assign statement".to_string());
        }
        let var_name = expr;
        if !var_name.starts_with("\"") {
            return Err("invalid variable name".to_string());
        }
        let var_name = expr.chars().skip(1).collect::<String>();
        if let Some(val) = self.var_table.get(var_name.as_str()) {
            let new_val = val + self.parse_numeric(args[1], runner)?;
            self.var_table.insert(var_name.to_string(), new_val);
            Ok(())
        } else {
            Err("variable not found".to_string())
        }
    }
}
