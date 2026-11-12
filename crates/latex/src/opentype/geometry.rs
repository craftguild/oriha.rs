//! Cubic Bezier extrema used for exact outline bounds.

pub(super) fn cubic_at(p0: f32, p1: f32, p2: f32, p3: f32, t: f32) -> f32 {
    let mt = 1.0 - t;
    mt * mt * mt * p0 + 3.0 * mt * mt * t * p1 + 3.0 * mt * t * t * p2 + t * t * t * p3
}

pub(super) fn cubic_extrema(p0: f32, p1: f32, p2: f32, p3: f32) -> Vec<f32> {
    let a = -p0 + 3.0 * p1 - 3.0 * p2 + p3;
    let b = 2.0 * (p0 - 2.0 * p1 + p2);
    let c = p1 - p0;

    if a.abs() < f32::EPSILON {
        if b.abs() < f32::EPSILON {
            return Vec::new();
        }
        let t = -c / b;
        return valid_cubic_extremum(t).into_iter().collect();
    }

    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        return Vec::new();
    }

    let sqrt = discriminant.sqrt();
    [(-b + sqrt) / (2.0 * a), (-b - sqrt) / (2.0 * a)]
        .into_iter()
        .filter_map(valid_cubic_extremum)
        .collect()
}

pub(super) fn valid_cubic_extremum(t: f32) -> Option<f32> {
    (t > 0.0 && t < 1.0).then_some(t)
}
