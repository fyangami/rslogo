use crate::{logo_runner::LogoRunner, r#const};
use r#const::*;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub struct LogoInterpreter {
    source_code: String,
    cursor: usize,
    // ignore the contention in single thread
    var_table: Arc<Mutex<HashMap<String, String>>>,
    procedure_table: Arc<Mutex<HashMap<String, LogoProcedure>>>,
    arg_vars_table: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct LogoProcedure {
    source_code: String,
    args: Vec<String>,
}

impl LogoInterpreter {
    pub fn default(source_code: String) -> Self {
        Self::new(
            source_code,
            Arc::new(Mutex::new(HashMap::new())),
            Arc::new(Mutex::new(HashMap::new())),
            HashMap::new(),
        )
    }

    pub fn new(
        source_code: String,
        var_table: Arc<Mutex<HashMap<String, String>>>,
        procedure_table: Arc<Mutex<HashMap<String, LogoProcedure>>>,
        arg_vars_table: HashMap<String, String>,
    ) -> Self {
        Self {
            source_code,
            cursor: 0,
            var_table,
            arg_vars_table,
            procedure_table,
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
        self.cursor += token.len();
        let token: &str = token.trim();
        if token.is_empty() {
            self.cursor += 1;
            return Ok(());
        }
        let expr = self.collect_expr(Self::get_terminator(&token))?;
        self.cursor += expr.len();
        match token {
            COMMENT => {
                // skip comments
                Ok(())
            }
            t if Self::is_builtin_fn(t) => self.evaluate_builtin_fn(t, expr, runner),
            MAKE => self.evaluate_make_statement(expr, runner),
            ADDASSIGN => self.evaluate_add_assign_statement(expr, runner),
            IF | WHILE => self.evaluate_conditional_statement(token, expr, runner),
            TO => self.evaluate_procedure_definition(expr, runner),
            _ => self.find_evaluate_procedure(token, expr, runner),
        }
    }

    fn evaluate_builtin_fn(
        &self,
        token: &str,
        expr: String,
        runner: &mut LogoRunner,
    ) -> Result<(), String> {
        let val = self.evaluate_expr(&expr, runner)?;
        match token {
            PENUP | PENDOWN => {
                if val.len() != 0 {
                    return Err(format!("invalid argument: {}", expr));
                }
                if token == "PENUP" {
                    runner.pen_up();
                } else {
                    runner.pen_down();
                }
            }
            FORWARD | BACK | LEFT | RIGHT | SETX | SETY | SETPENCOLOR | TURN | SETHEADING => {
                if val.len() != 1 {
                    return Err(format!("invalid argument: {}", expr));
                }
                let val: i32 = val
                    .iter()
                    .next()
                    .unwrap()
                    .parse()
                    .map_err(|_| format!("invalid argument: {}", expr))?;
                match token {
                    FORWARD => runner.draw_forward(val)?,
                    BACK => runner.draw_backward(val)?,
                    LEFT => runner.draw_left(val)?,
                    RIGHT => runner.draw_right(val)?,
                    SETX => runner.set_pos(val, runner.get_pos_y()),
                    SETY => runner.set_pos(runner.get_pos_x(), val),
                    SETPENCOLOR => {
                        if val > 15 || val < 0 {
                            return Err("invalid color".to_string());
                        }
                        runner.set_color(val as usize);
                    }
                    TURN | SETHEADING => runner.turn_degree(val),
                    _ => unreachable!(),
                }
            }
            _ => return Err(format!("unimplemented builtin function: {}", token)),
        }
        Ok(())
    }

    fn evaluate_expr(&self, expr: &str, runner: &LogoRunner) -> Result<Vec<String>, String> {
        let mut items = expr.trim().split_whitespace().collect::<Vec<&str>>();
        let mut stack: Vec<String> = Vec::new();
        items.reverse();
        for item in items {
            match item {
                PLUS | MINUS | TIMES | DIVIDE => {
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
                        PLUS => left + right,
                        MINUS => left - right,
                        TIMES => left * right,
                        DIVIDE => left / right,
                        _ => unreachable!(),
                    };
                    stack.push(result.to_string());
                }
                EQ | NE | LT | GT => {
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
                        let vt = self.var_table.lock().map_err(|e| e.to_string())?;
                        if let Some(var_value) = vt.get(var_name) {
                            stack.push(var_value.to_string());
                            continue;
                        }
                        // find in args table
                        if let Some(var_value) = self.arg_vars_table.get(var_name) {
                            stack.push(var_value.to_string());
                            continue;
                        }
                    }
                    return Err(format!("undefined variable: {}", item));
                }
                XCOR => stack.push(runner.get_pos_x().to_string()),
                YCOR => stack.push(runner.get_pos_y().to_string()),
                HEADING => stack.push(runner.get_direction().to_string()),
                COLOR => stack.push(runner.get_color_index().to_string()),
                _ => {}
            }
        }
        stack.reverse();
        Ok(stack.to_vec())
    }

    fn logical_op(&self, left: &str, right: &str, op: &str) -> Result<bool, String> {
        // boolean comparison
        if left == TRUE || left == FALSE || right == TRUE || right == FALSE {
            if left != TRUE && left != FALSE {
                return Err(format!(
                    "invalid expression, right operand is not a boolean: {}",
                    left
                ));
            }
            if right != TRUE && right != FALSE {
                return Err(format!(
                    "invalid expression, right operand is not a boolean: {}",
                    right
                ));
            }
            let left = left == TRUE;
            let right = right == TRUE;
            match op {
                EQ => return Ok(left == right),
                NE => return Ok(left != right),
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
            GT => return Ok(left > right),
            LT => return Ok(left < right),
            EQ => return Ok(left == right),
            NE => return Ok(left != right),
            _ => return Err(format!("invalid operator: {}", op)),
        }
    }

    fn collect_expr(&self, terminator: &str) -> Result<String, String> {
        let mut expr = Vec::new();
        let mut cursor = self.cursor;
        let mut skip = 0;
        while let Some(ch) = self.source_code.chars().nth(cursor) {
            expr.push(ch);
            let start_pos: i32 = cursor as i32 - terminator.len() as i32 + 1;
            if start_pos >= 0 {
                let matcher = &self.source_code[start_pos as usize..cursor + 1];
                // dirty fix for nested statement
                if terminator == "]" {
                    if ch == '[' {
                        skip += 1;
                    }
                }
                if matcher == "]" {
                    skip -= 1;
                }
                if matcher == terminator && skip <= 0 {
                    return Ok(expr.iter().collect::<String>());
                }
            }
            cursor += 1;
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
            PENUP | PENDOWN | FORWARD | BACK | LEFT | RIGHT | SETX | SETY | SETPENCOLOR | TURN
            | SETHEADING => true,
            _ => false,
        }
    }

    fn get_terminator(token: &str) -> &str {
        match token {
            PENUP | PENDOWN | FORWARD | BACK | LEFT | RIGHT | SETX | SETY | SETPENCOLOR | TURN
            | SETHEADING | MAKE | END | ADDASSIGN => "\n",
            TO => "END",
            IF | WHILE => "]",
            _ => "\n",
        }
    }

    fn evaluate_make_statement(&mut self, expr: String, runner: &LogoRunner) -> Result<(), String> {
        let result = self.evaluate_expr(&expr, runner)?;
        if result.len() != 2 {
            return Err(format!("invalid make statement: {}", expr));
        }
        let var_name = result[0].to_string();
        let val = result[1].to_string();
        let mut vt = self.var_table.lock().map_err(|e| e.to_string())?;
        vt.insert(var_name, val);
        Ok(())
    }

    fn evaluate_add_assign_statement(
        &mut self,
        expr: String,
        runner: &LogoRunner,
    ) -> Result<(), String> {
        let result = self.evaluate_expr(&expr, runner)?;
        if result.len() != 2 {
            return Err(format!("invalid addassign statement: {}", expr));
        }
        let var_name = result[0].to_string();
        let val: String = result[1].to_string();
        let mut vt = self.var_table.lock().map_err(|e| e.to_string())?;
        let old_val = vt
            .get(&var_name)
            .ok_or(format!("variable not found:{} ", var_name))?
            .to_string();
        let new_val = old_val
            .parse::<i32>()
            .map_err(|_| format!("invalid numeric value: {}", old_val))?
            + val
                .parse::<i32>()
                .map_err(|_| format!("invalid numeric value: {}", val))?;
        vt.insert(var_name, new_val.to_string());
        Ok(())
    }

    fn evaluate_conditional_statement(
        &self,
        token: &str,
        expr: String,
        runner: &mut LogoRunner,
    ) -> Result<(), String> {
        let oneshot = token == IF;
        if let Some((mut cond_expr, body)) = expr.trim().split_once("\n") {
            cond_expr = cond_expr
                .trim()
                .strip_suffix("[")
                .ok_or(format!("invalid conditional statement: {}", cond_expr))?;
            let mut count = 0;
            loop {
                if count > 10 {
                    return Ok(());
                }
                count += 1;
                let result = self.evaluate_expr(cond_expr, runner)?;
                if result.len() != 1 {
                    return Err(format!("invalid conditional expression: {:?}", result));
                }
                if result.iter().next().unwrap() == TRUE {
                    // condition is true, execute the body
                    if let Some(body_code) = body.trim().strip_suffix("]") {
                        let mut interpreter = LogoInterpreter::new(
                            body_code.to_string(),
                            self.var_table.clone(),
                            self.procedure_table.clone(),
                            self.arg_vars_table.clone(),
                        );
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

    fn evaluate_procedure_definition(
        &mut self,
        expr: String,
        runner: &mut LogoRunner,
    ) -> Result<(), String> {
        if let Some((definition_expr, body)) = expr.trim().split_once("\n") {
            if let Some((procedure_name, procedure_expr)) = definition_expr.split_once(" ") {
                let result: Vec<String> = self.evaluate_expr(procedure_expr, runner)?;
                if result.len() < 1 {
                    return Err(format!("invalid procedure definition: {}", expr));
                }
                if let Some(body_code) = body.trim().strip_suffix("END") {
                    let mut pt = self.procedure_table.lock().map_err(|e| e.to_string())?;
                    pt.insert(
                        procedure_name.to_string(),
                        LogoProcedure {
                            source_code: body_code.to_string(),
                            args: result,
                        },
                    );
                    return Ok(());
                }
            }
        }
        return Err(format!("invalid procedure definition: {}", expr));
    }

    fn find_evaluate_procedure(
        &mut self,
        token: &str,
        expr: String,
        runner: &mut LogoRunner,
    ) -> Result<(), String> {
        let pt = self.procedure_table.lock().map_err(|e| e.to_string())?;
        if let Some(procedure) = pt.get(token) {
            let arg_vars = self.evaluate_expr(&expr, runner)?;
            if procedure.args.len() != arg_vars.len() {
                return Err(format!(
                    "invalid number of arguments for procedure: {}",
                    token
                ));
            }
            let mut arg_vars_table = HashMap::new();
            arg_vars.iter().enumerate().for_each(|(i, v)| {
                arg_vars_table.insert(procedure.args[i].to_string(), v.to_string());
            });
            let mut interpreter = LogoInterpreter::new(
                procedure.source_code.to_string(),
                self.var_table.clone(),
                self.procedure_table.clone(),
                arg_vars_table,
            );
            drop(pt);
            return interpreter.interpret(runner);
        }
        return Err(format!("unknown procedure: {}", token));
    }
}
