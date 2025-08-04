use std::{f32::consts::FRAC_PI_2, rc::Rc};

use macroquad::{prelude::*, rand::gen_range};


struct Waypoint {
    pos: Vec2,
    dir: Vec2,
}

impl Waypoint {
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

enum TurnType {
    Left,
    Right,
}

struct Turn {
    radius: f32,
    deg: f32,
    turn_type: TurnType,
}

enum Shape {
    Straigth(f32),
    Turn(Turn),
}

struct Segment {
    start: Waypoint,
    shape: Shape,
}

impl Segment {
    fn end(&self) -> Waypoint {
        match self.shape {
            Shape::Straigth(len) => Waypoint {
                pos: self.start.pos + self.start.dir * len,
                dir: self.start.dir,
            },
            Shape::Turn(ref turn) => {
                let sgn = match turn.turn_type {
                    TurnType::Left => 1.0,
                    TurnType::Right => -1.0,
                };
                let center = self.start.dir.perp().normalize() * turn.radius * sgn + self.start.pos;
                let rot_vector = Vec2::from_angle(sgn * turn.deg.to_radians());
                let to_start = self.start.pos - center;
                Waypoint {
                    pos: rot_vector.rotate(to_start) + center,
                    dir: rot_vector.rotate(self.start.dir).normalize(),
                }
            }
        }
    }

    fn draw(&self) {
        let end = self.end();

        match self.shape {
            Shape::Straigth(_len) => {
                for (d, color, thickness) in
                    [(0.0, BLACK, 30.0), (-15.0, WHITE, 1.0), (15.0, WHITE, 1.0)]
                {
                    let shift = if d != 0.0 {
                        (end.pos - self.start.pos).perp().normalize() * d
                    } else {
                        Vec2::ZERO
                    };
                    draw_line(
                        self.start.pos.x + shift.x,
                        self.start.pos.y + shift.y,
                        end.pos.x + shift.x,
                        end.pos.y + shift.y,
                        thickness,
                        color,
                    );
                }
            }
            Shape::Turn(ref turn) => {
                let sgn = match turn.turn_type {
                    TurnType::Left => 1.0,
                    TurnType::Right => -1.0,
                };
                let center = self.start.dir.perp().normalize() * turn.radius * sgn + self.start.pos;
                let to_start = self.start.pos - center;

                // draw_circle_lines(center.x, center.y, 3.0, 1.0, GREEN);
                let start_deg = to_start.to_angle().to_degrees();
                let arc_start = match turn.turn_type {
                    TurnType::Left => start_deg,
                    TurnType::Right => start_deg - turn.deg,
                };
                for (d, color, thickness) in [
                    (-15.0, BLACK, 30.0),
                    (-15.5, WHITE, 1.0),
                    (14.5, WHITE, 1.0),
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

    fn bbox(&self) -> rstar::AABB<[f32; 2]> {
        rstar::AABB::from_points([self.start.pos.into(), self.end().pos.into()].iter())
    }

    fn hits(&self, pos: &Vec2) -> bool {
        match &self.shape {
            Shape::Straigth(len) => {
                draw_line(self.start.pos.x, self.start.pos.y, self.end().pos.x, self.end().pos.y, 3.0, YELLOW);
                let ab = self.end().pos - self.start.pos;
                let ap = *pos - self.start.pos;
                let proj = ap.dot(ab) / len;
                if proj < 0.0 || proj > *len {
                    return false;
                }

                let closest = self.start.pos + ab.normalize() * proj;
                let dist = (*pos - closest).length();
                dist <= 30.0 / 2.0
            },
            Shape::Turn(turn) => false
        }
    }
}

type TreeNode =
    rstar::primitives::GeomWithData<rstar::primitives::Rectangle<[f32; 2]>, Rc<Segment>>;

pub struct Track {
    segments: Vec<Rc<Segment>>,
    rtree: Option<rstar::RTree<TreeNode>>,
}

impl Track {
    pub fn new() -> Self {
        let mut track = Self {
            segments: vec![],
            rtree: None,
        };
        track.add_shape(Shape::Straigth(100.0));
        track
    }

    pub fn draw(&self, view: &Rect) {
        if let Some(rtree) = &self.rtree {
            let envelope = rstar::AABB::from_corners([view.x, view.y], [view.x + view.w, view.y + view.h]);
            rtree.locate_in_envelope_intersecting(&envelope).for_each(|segment| segment.data.draw());
        }
    }

    pub fn hits(&self, pos: &Vec2) {
        let rtree = &self.rtree.as_ref().unwrap();

        if let Some(segment) = rtree.nearest_neighbor(&[pos.x, pos.y]) {
            if segment.data.hits(pos) {
                draw_circle(pos.x, pos.y, 5.0, YELLOW);
            }
        }
    }

    fn last_end(&self) -> Waypoint {
        self.segments
            .last()
            .map(|last| last.end())
            .unwrap_or_default()
    }

    fn add_shape(&mut self, shape: Shape) {
        self.segments.push(Rc::new(Segment {
            start: self.last_end(),
            shape,
        }));
    }

    pub fn add_random_shape(&mut self) {
        let last_deg = Vec2::from_angle(-FRAC_PI_2)
            .rotate(self.last_end().dir)
            .to_angle()
            .to_degrees();
        assert!(last_deg >= -90.0 && last_deg <= 90.0);

        let max_deg_left = 90.0 - last_deg;
        let max_deg_right = 90.0 + last_deg;
        assert!(max_deg_left >= 0.0 && max_deg_left <= 180.0);
        assert!(max_deg_right >= 0.0 && max_deg_right <= 180.0);

        let mut choices = vec![];
        if max_deg_left >= 30.0 {
            choices.push(TurnType::Left);
        }
        if max_deg_right >= 30.0 {
            choices.push(TurnType::Right);
        }
        assert!(!choices.is_empty());

        let choice = &choices[gen_range(0, choices.len())];
        let shape = match choice {
            TurnType::Left => Shape::Turn(Turn {
                radius: gen_range(25.0, 100.0),
                deg: gen_range(30.0, max_deg_left),
                turn_type: TurnType::Left,
            }),
            TurnType::Right => Shape::Turn(Turn {
                radius: gen_range(25.0, 100.0),
                deg: gen_range(30.0, max_deg_right),
                turn_type: TurnType::Right,
            }),
        };

        self.add_shape(shape);
    }

    pub fn compute_rtree(&mut self) {
        let elements: Vec<_> = self
            .segments
            .iter()
            .map(|segment| TreeNode::new(segment.bbox().into(), Rc::clone(segment)))
            .collect();
        self.rtree = Some(rstar::RTree::<TreeNode>::bulk_load(elements));
    }
}
