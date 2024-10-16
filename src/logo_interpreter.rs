use std::path::PathBuf;

use unsvg::Image;

pub struct LogoInterpreter {
    pen_status: PenStatus,
    color: unsvg::Color,
    x: i32,
    y: i32,
    direction: i32,
    output: Image,
}

impl LogoInterpreter {
    pub fn new(size_x: u32, size_y: u32) -> Self {
        let pos_x = (size_x / 2) as i32;
        let pos_y = (size_y / 2) as i32;
        LogoInterpreter {
            pen_status: PenStatus::Up,
            color: unsvg::Color::white(), // TODO
            x: pos_x,
            y: pos_y,
            direction: 0,
            output: Image::new(size_x, size_y),
        }
    }

    pub fn pen_up(&mut self) {
        self.pen_status = PenStatus::Up;
    }

    pub fn pen_down(&mut self) {
        self.pen_status = PenStatus::Down;
    }

    pub fn set_pos(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }

    pub fn draw_forward(&mut self, distance: i32) -> Result<(), String> {
        self.draw(distance, self.direction)
    }

    pub fn draw_backward(&mut self, distance: i32) -> Result<(), String> {
        self.draw(distance, (self.direction + 180) % 360)
    }

    pub fn draw_right(&mut self, distance: i32) -> Result<(), String> {
        self.draw(distance, (self.direction + 90) % 360)
    }

    pub fn draw_left(&mut self, distance: i32) -> Result<(), String> {
        self.draw(distance, (self.direction + 270) % 360)
    }

    fn draw(&mut self, distance: i32, direction: i32) -> Result<(), String> {
        let (x, y) = match self.pen_status {
            PenStatus::Down => self
                .output
                .draw_simple_line(self.x, self.y, direction, distance, self.color)?,
            PenStatus::Up => unsvg::get_end_coordinates(self.x, self.y, direction, distance),
        };
        self.set_pos(x, y);
        Ok(())
    }

    pub fn get_pos_x(&self) -> i32 {
        self.x
    }

    pub fn get_pos_y(&self) -> i32 {
        self.y
    }

    pub fn set_color(&mut self, color: unsvg::Color) {
        self.color = color;
    }

    pub fn turn_degree(&mut self, degree: i32) {
        self.direction = (self.direction + degree) % 360;
    }

    pub fn save(self, path: &PathBuf) -> Result<(), String> {
        match path.extension().and_then(|s| s.to_str()) {
            Some("svg") => self.output.save_svg(path),
            Some("png") => self.output.save_png(path),
            _ => {
                return Err("file extension not supported".to_string());
            }
        }
    }
}

#[derive(Debug, PartialEq)]
enum PenStatus {
    Up,
    Down,
}
