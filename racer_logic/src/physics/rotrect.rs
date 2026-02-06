use macroquad::prelude::*;

#[derive(Debug, Clone)]
pub struct RotRect {
    center: Vec2,
    half_size: Vec2,
    rotation: f32,
    corners: [Vec2; 4],
    axes: [(Vec2, Vec2); 2],
}

impl RotRect {
    pub fn new(center: Vec2, size: Vec2, rotation: f32) -> Self {
        let half_size = size / 2.0;
        let corners = RotRect::compute_corners(center, half_size, rotation);
        let axes = RotRect::compute_axes(center, half_size, rotation);
        Self {
            center,
            half_size,
            rotation,
            corners,
            axes,
        }
    }

    pub fn update(&mut self, new_center: Vec2, new_rotation: f32) {
        self.center = new_center;
        self.rotation = new_rotation;
        self.corners = RotRect::compute_corners(self.center, self.half_size, self.rotation);
        self.axes = RotRect::compute_axes(self.center, self.half_size, self.rotation);
    }

    fn compute_corners(center: Vec2, half_size: Vec2, rotation: f32) -> [Vec2; 4] {
        let mut corners = [
            vec2(1.0, 1.0),
            vec2(1.0, -1.0),
            vec2(-1.0, 1.0),
            vec2(-1.0, -1.0),
        ];

        for corner in &mut corners {
            let to_corner_dir = Vec2::from_angle(rotation).rotate(half_size * *corner);
            *corner = to_corner_dir + center;
        }
        corners
    }

    fn compute_axes(center: Vec2, half_size: Vec2, rotation: f32) -> [(Vec2, Vec2); 2] {
        let rot = Vec2::from_angle(rotation);
        [
            (
                center + rot.rotate(vec2(0.0, half_size.y)),
                center + rot.rotate(vec2(0.0, -half_size.y)),
            ),
            (
                center + rot.rotate(vec2(half_size.x, 0.0)),
                center + rot.rotate(vec2(-half_size.x, 0.0)),
            ),
        ]
    }

    pub fn collide(&self, other: &RotRect) -> bool {
        let d = (self.center - other.center).length()
            - self.half_size.length()
            - other.half_size.length();
        if d > 0.0 {
            return false;
        }

        for (first, second) in [(self, other), (other, self)] {
            for (axis_start, axis_end) in first.axes {
                let mut ts = vec![];
                for corner in second.corners {
                    let (_, t) = projection_on_line(axis_start, axis_end, corner);
                    ts.push(t);
                }
                let r = min_max(&mut ts.into_iter());
                let hit_range = 0.0..=1.0;
                let start_in = hit_range.contains(r.start());
                let end_in = hit_range.contains(r.end());
                let hit = start_in || end_in || (*r.start() <= 0.0 && *r.end() >= 1.0);
                if !hit {
                    return false;
                }
            }
        }
        true
    }

    pub fn collision_point(&self, other: &RotRect) -> Option<Vec2> {
        let d = (self.center - other.center).length()
            - self.half_size.length()
            - other.half_size.length();
        if d > 0.0 {
            return None;
        }

        for &corner in &self.corners {
            if other.is_point_inside(corner) {
                return Some(corner);
            }
        }

        #[allow(clippy::manual_find)]
        for &corner in &other.corners {
            if self.is_point_inside(corner) {
                return Some(corner);
            }
        }

        None
    }

    fn is_point_inside(&self, point: Vec2) -> bool {
        for (axis_start, axis_end) in self.axes {
            let (_, t) = projection_on_line(axis_start, axis_end, point);
            if !(0.0..=1.0).contains(&t) {
                return false;
            }
        }
        true
    }

    pub fn draw(&self, color: Color) {
        draw_rectangle_ex(
            self.center.x,
            self.center.y,
            self.half_size.x * 2.0,
            self.half_size.y * 2.0,
            DrawRectangleParams {
                offset: vec2(0.5, 0.5),
                rotation: self.rotation,
                color,
            },
        );
    }

    pub fn corners(&self) -> &[Vec2; 4] {
        &self.corners
    }

    pub fn center(&self) -> Vec2 {
        self.center
    }

    pub fn rotation(&self) -> f32 {
        self.rotation
    }
}

fn projection_on_line(line_start: Vec2, line_end: Vec2, point: Vec2) -> (Vec2, f32) {
    let to_end = line_end - line_start;
    let to_point = point - line_start;

    let to_projection = to_end * (to_end.dot(to_point)) / (to_end.dot(to_end));
    let projection = line_start + to_projection;
    let t = to_projection.dot(to_end) / to_end.dot(to_end);

    (projection, t)
}

fn min_max(vals: &mut impl Iterator<Item = f32>) -> std::ops::RangeInclusive<f32> {
    let mut min = f32::MAX;
    let mut max = f32::MIN;
    for v in vals {
        if v > max {
            max = v;
        }
        if v < min {
            min = v;
        }
    }
    min..=max
}
