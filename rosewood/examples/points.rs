use rosewood::{gis::Points, BulkWriter};

fn main() {
    let pts = random_points(100, 2);
    let mut writer = BulkWriter::new("points.loam").unwrap();
    writer.push(&pts).unwrap();
    writer.finish().unwrap();
}

fn random_points(n_points: usize, seed: u64) -> Points<f32, ()> {
    fastrand::seed(seed);
    let mut pts = Points::new(());
    for _ in 0..n_points {
        let x = fastrand::f32();
        let y = fastrand::f32();
        pts.push((x, y));
    }
    pts
}
