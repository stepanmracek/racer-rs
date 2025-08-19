use super::segment::*;
use super::shape::*;
use crate::car::Car;
use crate::physics::RotRect;
use crate::physics::arc_vs_segment;
use crate::physics::segment_vs_segment;
use crate::track::constant::TRACK_WIDTH;
use macroquad::prelude::*;
use macroquad::rand::{gen_range, rand};
use std::f32::consts::FRAC_PI_2;
use std::rc::Rc;

type TreeNode =
    rstar::primitives::GeomWithData<rstar::primitives::Rectangle<[f32; 2]>, Rc<Segment>>;

pub struct Track {
    segments: Vec<Rc<Segment>>,
    rtree: Option<rstar::RTree<TreeNode>>,
    finish: Option<RotRect>,
}

impl Track {
    pub fn new() -> Self {
        let mut track = Self {
            segments: vec![],
            rtree: None,
            finish: None,
        };
        track.add_shape(Shape::Straight(Straight {
            length: 100.0,
            is_finish: false,
        }));
        track
    }

    pub fn draw(&self, car: &Car) {
        if let Some(rtree) = &self.rtree {
            let pos = car.position();
            let view = Rect::new(pos.x - 300.0, pos.y - 200.0, 600.0, 400.0);
            //draw_rectangle_lines(view.x, view.y, view.w, view.h, 3.0, WHITE);
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

    pub fn nearest_segments(&self, pos: &Vec2, limit: usize) -> Vec<Rc<Segment>> {
        self.rtree
            .as_ref()
            .unwrap()
            .nearest_neighbor_iter(&[pos.x, pos.y])
            .take(limit)
            .map(|g| Rc::clone(&g.data))
            .collect()
    }

    pub fn finish(&self, car_bbox: &RotRect) -> bool {
        if let Some(finish) = &self.finish {
            finish.collide(car_bbox)
        } else {
            false
        }
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
        if (rand() as f32 / u32::MAX as f32) < 0.5 {
            self.add_shape(Shape::Straight(Straight {
                length: gen_range(10.0, 50.0),
                is_finish: false,
            }));
            return;
        }

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

    pub fn add_finish(&mut self) {
        self.add_shape(Shape::Straight(Straight {
            length: 100.0,
            is_finish: true,
        }));

        let last = self.segments.last().unwrap();
        let center = last.start.pos.midpoint(last.end.pos);
        let size = vec2(20.0, TRACK_WIDTH);
        let rotation = (last.end.pos - last.start.pos).to_angle();
        self.finish = Some(RotRect::new(center, size, rotation));
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

pub fn sensor_readings(
    nearest_segments: &Vec<Rc<Segment>>,
    sensor_rays: &Vec<(Vec2, Vec2)>,
) -> Vec<Option<f32>> {
    let mut ans = vec![];
    for (start, end) in sensor_rays {
        let mut nearest: Option<f32> = None;
        const WIDTH_HALF: f32 = TRACK_WIDTH / 2.0;

        for segment in nearest_segments {
            match &segment.shape {
                Shape::Turn(turn) => {
                    let center = turn.center(&segment.start);
                    for d in [-WIDTH_HALF, WIDTH_HALF] {
                        let intersection = arc_vs_segment(
                            &center,
                            turn.radius + d,
                            &(segment.start.pos, segment.end.pos),
                            &(*start, *end),
                        );
                        if let Some(intersection) = intersection {
                            let dist = start.distance_squared(intersection);
                            if nearest.is_none_or(|cur_dist| dist < cur_dist) {
                                nearest = Some(dist);
                            }
                        }
                    }
                }
                Shape::Straight(_) => {
                    for d in [-WIDTH_HALF, WIDTH_HALF] {
                        let shift = segment.start.dir.perp() * d;
                        let intersection = segment_vs_segment(
                            &(*start, *end),
                            &(segment.start.pos + shift, segment.end.pos + shift),
                        );
                        if let Some(intersection) = intersection {
                            let dist = start.distance_squared(intersection);
                            if nearest.is_none_or(|cur_dist| dist < cur_dist) {
                                nearest = Some(dist);
                            }
                        }
                    }
                }
            }
        }

        ans.push(nearest.map(|dist| dist.sqrt()));
    }

    ans
}
