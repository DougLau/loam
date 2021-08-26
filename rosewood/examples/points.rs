use rosewood::{BulkWriter, Point};

fn main() {
    let pts = random_points(100, 2);
    let mut writer = BulkWriter::new("points.loam").unwrap();
    for pt in pts {
        writer.push(&pt).unwrap();
    }
    writer.finish().unwrap();
}

fn random_points(n_points: usize, seed: u64) -> Vec<Point<(), f32>> {
    fastrand::seed(seed);
    let mut pts = Vec::with_capacity(n_points);
    for _ in 0..n_points {
        let x = fastrand::f32();
        let y = fastrand::f32();
        pts.push(Point::new((x, y), ()));
    }
    pts
}
