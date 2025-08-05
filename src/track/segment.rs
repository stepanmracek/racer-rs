use super::constant::*;
use super::shape::*;
use macroquad::prelude::*;

pub struct Segment {
    pub start: Waypoint,
    pub shape: Shape,
    pub end: Waypoint,
}

impl Segment {
    pub fn new(start: Waypoint, shape: Shape) -> Self {
        let end = match shape {
            Shape::Straigth(len) => Waypoint {
                pos: start.pos + start.dir * len,
                dir: start.dir,
            },
            Shape::Turn(ref turn) => {
                let sgn = match turn.turn_type {
                    TurnType::Left => 1.0,
                    TurnType::Right => -1.0,
                };
                let center = start.dir.perp() * turn.radius * sgn + start.pos;
                let rot_vector = Vec2::from_angle(sgn * turn.deg.to_radians());
                let to_start = start.pos - center;
                Waypoint {
                    pos: rot_vector.rotate(to_start) + center,
                    dir: rot_vector.rotate(start.dir),
                }
            }
        };
        Self { start, shape, end }
    }

    pub fn draw(&self) {
        let track_color = Color::from_rgba(32, 32, 32, 255);
        match self.shape {
            Shape::Straigth(_len) => {
                for (d, color, thickness) in [
                    (0.0, track_color, TRACK_WIDTH),
                    (-TRACK_WIDTH / 2.0, WHITE, 1.0),
                    (TRACK_WIDTH / 2.0, WHITE, 1.0),
                ] {
                    let shift = if d != 0.0 {
                        (self.end.pos - self.start.pos).perp().normalize() * d
                    } else {
                        Vec2::ZERO
                    };
                    draw_line(
                        self.start.pos.x + shift.x,
                        self.start.pos.y + shift.y,
                        self.end.pos.x + shift.x,
                        self.end.pos.y + shift.y,
                        thickness,
                        color,
                    );
                }
            }
            Shape::Turn(ref turn) => {
                let center = turn.center(&self.start);
                let to_start = self.start.pos - center;

                // draw_circle_lines(center.x, center.y, 3.0, 1.0, GREEN);
                let start_deg = to_start.to_angle().to_degrees();
                let arc_start = match turn.turn_type {
                    TurnType::Left => start_deg,
                    TurnType::Right => start_deg - turn.deg,
                };
                for (d, color, thickness) in [
                    (-TRACK_WIDTH / 2.0, track_color, TRACK_WIDTH),
                    (-TRACK_WIDTH / 2.0 - 0.5, WHITE, 1.0),
                    (TRACK_WIDTH / 2.0 - 0.5, WHITE, 1.0),
                ] {
                    draw_arc(
                        center.x,
                        center.y,
                        32,
                        turn.radius + d,
                        arc_start,
                        thickness,
                        turn.deg,
                        color,
                    );
                }
            }
        }

        // self.start.draw();
        // end.draw();
    }

    pub fn bbox(&self) -> rstar::AABB<[f32; 2]> {
        rstar::AABB::from_points([self.start.pos.into(), self.end.pos.into()].iter())
    }

    pub fn hits(&self, pos: &Vec2) -> bool {
        match &self.shape {
            Shape::Straigth(len) => {
                let ab = self.end.pos - self.start.pos;
                let ap = *pos - self.start.pos;
                let proj = ap.dot(ab) / len;
                if proj < 0.0 || proj > *len {
                    return false;
                }

                let closest = self.start.pos + ab.normalize() * proj;
                let dist = (*pos - closest).length();
                dist <= TRACK_WIDTH / 2.0
            }
            Shape::Turn(turn) => {
                let center = turn.center(&self.start);
                let to_pos = *pos - center;
                let len = to_pos.length();
                if len > turn.radius + TRACK_WIDTH / 2.0 || len < turn.radius - TRACK_WIDTH / 2.0 {
                    return false;
                }

                let to_start = self.start.pos - center;
                let to_end = self.end.pos - center;

                let ab = to_start.perp_dot(to_end);
                let av = to_start.perp_dot(to_pos);
                let bv = to_end.perp_dot(to_pos);
                if ab >= 0.0 {
                    av >= 0.0 && bv <= 0.0 // CCW
                } else {
                    av <= 0.0 && bv >= 0.0 // CW
                }
            }
        }
    }
}
