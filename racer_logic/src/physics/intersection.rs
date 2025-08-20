use macroquad::prelude::*;

pub fn point_in_angle(point: &Vec2, center: &Vec2, start: &Vec2, end: &Vec2) -> bool {
    let to_pos = *point - *center;
    let to_start = *start - *center;
    let to_end = *end - *center;

    let ab = to_start.perp_dot(to_end);
    let av = to_start.perp_dot(to_pos);
    let bv = to_end.perp_dot(to_pos);
    if ab >= 0.0 {
        av >= 0.0 && bv <= 0.0 // CCW
    } else {
        av <= 0.0 && bv >= 0.0 // CW
    }
}

pub fn arc_vs_segment(
    center: &Vec2,
    radius: f32,
    arc: &(Vec2, Vec2),
    segment: &(Vec2, Vec2),
) -> Option<Vec2> {
    let d = segment.1 - segment.0;
    let f = segment.0 - *center;

    let a = d.length_squared();
    let b = 2.0 * (f.x * d.x + f.y * d.y);
    let c = f.length_squared() - radius.powi(2);

    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        return None;
    }

    let discriminant = discriminant.sqrt();
    let t1 = (-b - discriminant) / (2.0 * a);
    let t2 = (-b + discriminant) / (2.0 * a);

    let mut ts = vec![];
    if (0.0..=1.0).contains(&t1) {
        ts.push((t1, segment.0 + t1 * d));
    }
    if (0.0..=1.0).contains(&t2) {
        ts.push((t2, segment.0 + t2 * d));
    }

    ts.into_iter()
        .filter(|(_t, p)| point_in_angle(p, center, &arc.0, &arc.1))
        .reduce(|a, b| if a.0 < b.0 { a } else { b })
        .map(|(_t, p)| p)
}

pub fn segment_vs_segment(first: &(Vec2, Vec2), second: &(Vec2, Vec2)) -> Option<Vec2> {
    let d1 = first.1 - first.0;
    let d2 = second.1 - second.0;
    let determinant = d1.perp_dot(d2);
    if determinant.abs() < 1e-5 {
        return None;
    }

    let t = ((second.0.x - first.0.x) * d2.y - (second.0.y - first.0.y) * d2.x) / determinant;
    let u = ((second.0.x - first.0.x) * d1.y - (second.0.y - first.0.y) * d1.x) / determinant;

    if (0.0..=1.0).contains(&u) && (0.0..=1.0).contains(&t) {
        Some(first.0 + t * d1)
    } else {
        None
    }
}
