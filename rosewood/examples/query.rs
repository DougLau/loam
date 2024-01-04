use pointy::{BBox, Bounded};
use rosewood::{gis::{Points, Polygons}, RTree};

fn points() {
    let rtree = RTree::<f32, Points<f32, ()>>::new("points.loam").unwrap();
    let bbox = BBox::new([(0.0_f32, 0.0), (0.5, 0.5)]);
    let mut n = 0;
    for points in rtree.query(bbox) {
        for pt in points.unwrap().iter() {
            if pt.bounded_by(bbox) {
                println!("{pt:?}");
                n += 1;
            }
        }
    }
    println!("found: {n}");
}

fn polygons() {
    let rtree = RTree::<f32, Polygons<f32, ()>>::new("polygons.loam").unwrap();
    let bbox = BBox::new([(0.0_f32, 0.0), (0.5, 0.5)]);
    let mut n = 0;
    for polygons in rtree.query(bbox) {
        for poly in polygons.unwrap().iter() {
            if poly.bounded_by(bbox) {
                println!("{poly:?}");
                n += 1;
            }
        }
    }
    println!("found: {n}");
}

fn main() {
    points();
    //polygons();
}
