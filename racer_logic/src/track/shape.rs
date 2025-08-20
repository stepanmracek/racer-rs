use macroquad::prelude::*;

#[derive(Clone)]
pub struct Waypoint {
    pub pos: Vec2,
    pub dir: Vec2,
}

impl Waypoint {
    #[allow(dead_code)]
    fn draw(&self) {
        draw_circle_lines(self.pos.x, self.pos.y, 5.0, 1.0, YELLOW);
        let end = self.pos + (self.dir * 20.0);
        draw_line(self.pos.x, self.pos.y, end.x, end.y, 1.0, YELLOW);
    }
}

impl Default for Waypoint {
    fn default() -> Self {
        Waypoint {
            pos: vec2(0.0, 0.0),
            dir: vec2(0.0, 1.0),
        }
    }
}

pub enum TurnType {
    Left,
    Right,
}

pub struct Turn {
    pub radius: f32,
    pub deg: f32,
    pub turn_type: TurnType,
}

impl Turn {
    pub fn center(&self, start: &Waypoint) -> Vec2 {
        let sgn = match self.turn_type {
            TurnType::Left => 1.0,
            TurnType::Right => -1.0,
        };
        start.dir.perp() * self.radius * sgn + start.pos
    }
}

pub struct Straight {
    pub length: f32,
    pub is_finish: bool,
}

pub enum Shape {
    Straight(Straight),
    Turn(Turn),
}
