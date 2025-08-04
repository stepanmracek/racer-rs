use std::{f32::consts::FRAC_PI_2, rc::Rc};

use macroquad::{prelude::*, rand::gen_range};

const TRACK_WIDTH: f32 = 30.0;

#[derive(Clone)]
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

impl Turn {
    fn center(&self, start: &Waypoint) -> Vec2 {
        let sgn = match self.turn_type {
            TurnType::Left => 1.0,
            TurnType::Right => -1.0,
        };
        start.dir.perp() * self.radius * sgn + start.pos
    }
}

enum Shape {
    Straigth(f32),
    Turn(Turn),
}

struct Segment {
    start: Waypoint,
    shape: Shape,
    end: Waypoint,
}

impl Segment {
    fn new(start: Waypoint, shape: Shape) -> Self {
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

    fn draw(&self) {
        match self.shape {
            Shape::Straigth(_len) => {
                for (d, color, thickness) in [
                    (0.0, BLACK, TRACK_WIDTH),
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
                    (-TRACK_WIDTH / 2.0, BLACK, TRACK_WIDTH),
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

    fn bbox(&self) -> rstar::AABB<[f32; 2]> {
        rstar::AABB::from_points([self.start.pos.into(), self.end.pos.into()].iter())
    }

    fn hits(&self, pos: &Vec2) -> bool {
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
            let envelope =
                rstar::AABB::from_corners([view.x, view.y], [view.x + view.w, view.y + view.h]);
            rtree
                .locate_in_envelope_intersecting(&envelope)
                .for_each(|segment| segment.data.draw());
        }
    }

    pub fn on_track(&self, pos: &Vec2) -> bool {
        let rtree = &self.rtree.as_ref().unwrap();

        for segment in rtree.nearest_neighbor_iter(&[pos.x, pos.y]).take(2) {
            if segment.data.hits(pos) {
                // draw_circle(pos.x, pos.y, 5.0, YELLOW);
                return true;
            }
        }
        false
    }

    fn last_end(&self) -> Waypoint {
        self.segments
            .last()
            .map(|last| last.end.clone())
            .unwrap_or_default()
    }

    fn add_shape(&mut self, shape: Shape) {
        self.segments
            .push(Rc::new(Segment::new(self.last_end(), shape)));
    }

    pub fn add_random_shape(&mut self) {
        let last_deg = Vec2::from_angle(-FRAC_PI_2)
            .rotate(self.last_end().dir)
            .to_angle()
            .to_degrees();
        assert!((-90.0..=90.0).contains(&last_deg));

        let max_deg_left = 90.0 - last_deg;
        let max_deg_right = 90.0 + last_deg;
        assert!((0.0..=180.0).contains(&max_deg_left));
        assert!((0.0..=180.0).contains(&max_deg_right));

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
                radius: gen_range(TRACK_WIDTH, 100.0),
                deg: gen_range(30.0, max_deg_left),
                turn_type: TurnType::Left,
            }),
            TurnType::Right => Shape::Turn(Turn {
                radius: gen_range(TRACK_WIDTH, 100.0),
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
