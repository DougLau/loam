use rosewood::{BulkWriter, gis::Polygons};

fn main() {
    let polys = random_polygons(100, 2);
    let mut writer = BulkWriter::new("polygons.loam").unwrap();
    writer.push(&polys).unwrap();
    writer.finish().unwrap();
}

fn random_polygons(n_points: usize, seed: u64) -> Polygons<f32, ()> {
    fastrand::seed(seed);
    let mut poly = Polygons::new(());
    for _ in 0..n_points {
        let mut ring = Vec::with_capacity(3);
        for _ in 0..3 {
            let x = fastrand::f32();
            let y = fastrand::f32();
            ring.push((x, y));
        }
        poly.push_outer(ring);
    }
    poly
}
