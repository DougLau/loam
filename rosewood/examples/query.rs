use pointy::BBox;
use rosewood::{gis::Point, RTree};

fn main() {
    let rtree = RTree::<f32, Point<f32, ()>>::new("points.loam").unwrap();
    //let bbox = BBox::new([(0.0_f32, 0.0), (0.5, 0.5)]);
    //let bbox = BBox::new([(0.0_f32, 0.5), (0.5, 1.0)]);
    //let bbox = BBox::new([(0.5_f32, 0.0), (1.0, 0.5)]);
    let bbox = BBox::new([(0.5_f32, 0.5), (1.0, 1.0)]);
    let mut n = 0;
    for point in rtree.query(bbox) {
        for pt in point.unwrap().as_points() {
            println!("x: {}, y: {}", pt.x(), pt.y());
        }
        n += 1;
    }
    println!("found: {}", n);
}
