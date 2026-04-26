fn euclidean_distance(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    ((x2 - x1) * (x2 - x1) + (y2 - y1) * (y2 - y1)).sqrt()
}

fn rounded_euclidean_distance(x1: f64, y1: f64, x2: f64, y2: f64) -> u32 {
    euclidean_distance(x1, y1, x2, y2).round() as u32
}