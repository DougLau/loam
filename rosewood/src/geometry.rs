// geometry.rs
//
// Copyright (c) 2021  Douglas P Lau
//
use pointy::{BBox, Float, Pt};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// Geometry which can be stored in an RTree
pub trait Geometry<F>
where
    F: Float,
{
    /// Data associated with geometry
    type Data;

    /// Get bounding box
    fn bbox(&self) -> BBox<F>;

    /// Get associated data
    fn data(&self) -> &Self::Data;
}

/// Point geometry
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct Point<F, D>
where
    F: Float,
{
    /// Points in geometry
    pts: Vec<Pt<F>>,

    /// Associated data
    data: D,
}

/// Line string geometry
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct Linestring<F, D>
where
    F: Float,
{
    /// Line strings in geometry
    strings: Vec<Vec<Pt<F>>>,

    /// Associated data
    data: D,
}

/// Polygon geometry
///
/// A polygon is a `Vec` of closed rings.  The winding order determines whether
/// a ring is "outer" or "inner".
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct Polygon<F, D>
where
    F: Float,
{
    /// Polygons in geometry
    rings: Vec<Vec<Pt<F>>>,

    /// Associated data
    data: D,
}

/// Enum of defined geometries
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum Geom<F, D>
where
    F: Float,
{
    /// Point geometry
    Point(Point<F, D>),

    /// Linestring geometry
    Linestring(Linestring<F, D>),

    /// Polygon geometry
    Polygon(Polygon<F, D>),
}

impl<F, D> Geometry<F> for Point<F, D>
where
    F: Float,
{
    type Data = D;

    fn bbox(&self) -> BBox<F> {
        BBox::new(&self.pts)
    }

    fn data(&self) -> &Self::Data {
        &self.data
    }
}

impl<F, D> Point<F, D>
where
    F: Float,
{
    /// Create a new point geometry
    pub fn new<P>(pt: P, data: D) -> Self
    where
        P: Into<Pt<F>>,
    {
        let pts = vec![pt.into()];
        Self { pts, data }
    }

    /// Add a point
    pub fn push<P>(&mut self, pt: P)
    where
        P: Into<Pt<F>>,
    {
        self.pts.push(pt.into());
    }

    /// Borrow points
    pub fn as_points(&self) -> &[Pt<F>] {
        &self.pts
    }
}

impl<F, D> Geometry<F> for Linestring<F, D>
where
    F: Float,
{
    type Data = D;

    fn bbox(&self) -> BBox<F> {
        let mut bbox = BBox::default();
        bbox.extend(self.strings.iter().flatten());
        bbox
    }

    fn data(&self) -> &Self::Data {
        &self.data
    }
}

impl<F, D> Linestring<F, D>
where
    F: Float,
{
    /// Create a new line string geometry
    pub fn new<I, P>(pts: I, data: D) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<Pt<F>>,
    {
        let pts = pts.into_iter().map(|pt| pt.into()).collect();
        let strings = vec![pts];
        Self { strings, data }
    }

    /// Push a line string
    pub fn push<I, P>(&mut self, pts: I)
    where
        I: IntoIterator<Item = P>,
        P: Into<Pt<F>>,
    {
        let pts = pts.into_iter().map(|pt| pt.into()).collect();
        self.strings.push(pts);
    }
}

impl<F, D> Geometry<F> for Polygon<F, D>
where
    F: Float,
{
    type Data = D;

    fn bbox(&self) -> BBox<F> {
        let mut bbox = BBox::default();
        bbox.extend(self.rings.iter().flatten());
        bbox
    }

    fn data(&self) -> &Self::Data {
        &self.data
    }
}

impl<F, D> Polygon<F, D>
where
    F: Float,
{
    /// Create a new polygon geometry
    pub fn new(data: D) -> Self {
        let rings = vec![];
        Self { rings, data }
    }

    /// Push an outer polygon
    pub fn push_outer<I, P>(&mut self, ring: I)
    where
        I: IntoIterator<Item = P>,
        P: Into<Pt<F>>,
    {
        let mut ring: Vec<_> = ring.into_iter().map(|pt| pt.into()).collect();
        if !is_clockwise(&ring) {
            ring.reverse();
        }
        self.rings.push(ring);
    }

    /// Push an inner polygon
    pub fn push_inner<I, P>(&mut self, ring: I)
    where
        I: IntoIterator<Item = P>,
        P: Into<Pt<F>>,
    {
        let mut ring: Vec<_> = ring.into_iter().map(|pt| pt.into()).collect();
        if is_clockwise(&ring) {
            ring.reverse();
        }
        self.rings.push(ring);
    }
}

/// Check if a ring of points has clockwise winding order
fn is_clockwise<F>(ring: &[Pt<F>]) -> bool
where
    F: Float,
{
    if let Some(ext) = find_extreme_point(ring) {
        let len = ring.len();
        let a = if ext > 0 { ext - 1 } else { len - 1 };
        let b = if ext < len - 1 { ext + 1 } else { 0 };
        // Make two vectors as edges pointing toward the extreme point
        let v0 = ring[a] - ring[ext];
        let v1 = ring[b] - ring[ext];
        // Cross product determines the winding order
        (v0 * v1) > F::zero()
    } else {
        false
    }
}

/// Find an extreme point on the convex hull of a polygon
fn find_extreme_point<F>(ring: &[Pt<F>]) -> Option<usize>
where
    F: Float,
{
    ring.iter()
        .enumerate()
        .min_by(|a, b| {
            (a.1.x(), a.1.y())
                .partial_cmp(&(b.1.x(), b.1.y()))
                .unwrap_or(Ordering::Greater)
        })
        .map(|e| e.0)
}

impl<F, D> Geometry<F> for Geom<F, D>
where
    F: Float,
{
    type Data = D;

    fn bbox(&self) -> BBox<F> {
        match self {
            Geom::Point(p) => p.bbox(),
            Geom::Linestring(ls) => ls.bbox(),
            Geom::Polygon(pg) => pg.bbox(),
        }
    }

    fn data(&self) -> &Self::Data {
        match self {
            Geom::Point(p) => p.data(),
            Geom::Linestring(ls) => ls.data(),
            Geom::Polygon(pg) => pg.data(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn clockwise() {
        let ring = [(0.0, 0.0).into(), (1.0, 0.0).into(), (0.0, 1.0).into()];
        assert_eq!(false, is_clockwise(&ring));
        let ring = [(0.0, 0.0).into(), (0.0, 1.0).into(), (1.0, 0.0).into()];
        assert_eq!(true, is_clockwise(&ring));
    }
}
