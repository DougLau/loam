use pointy::BBox;
use rosewood::{Point, RTree};

fn main() {
    let rtree = RTree::<f32, Point<(), f32>>::new("points.loam").unwrap();
    //let bbox = BBox::new([(0.0_f32, 0.0), (0.5, 0.5)]);
    //let bbox = BBox::new([(0.0_f32, 0.5), (0.5, 1.0)]);
    //let bbox = BBox::new([(0.5_f32, 0.0), (1.0, 0.5)]);
    let bbox = BBox::new([(0.5_f32, 0.5), (1.0, 1.0)]);
    let mut n = 0;
    for geom in rtree.query(bbox) {
        let geom = geom.unwrap();
        dbg!(geom);
        n += 1;
    }
    dbg!(n);
}
