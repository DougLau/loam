// geometry.rs
//
// Copyright (c) 2021  Douglas P Lau
//
use pointy::{BBox, Float, Pt};
use serde::{Deserialize, Serialize};

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
/// A polygon is a `Vec` of closed rings, with the first being the outer ring.
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct Polygon<F, D>
where
    F: Float,
{
    /// Polygons in geometry
    polygons: Vec<Vec<Vec<Pt<F>>>>,

    /// Associated data
    data: D,
}

/// Enum of defined geometries
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum GeomType<F, D>
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
        bbox.extend(self.polygons.iter().flatten().flatten());
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
    pub fn new<I, P, R>(rings: I, data: D) -> Self
    where
        I: IntoIterator<Item = R>,
        R: IntoIterator<Item = P>,
        P: Into<Pt<F>>,
    {
        let mut polygon = vec![];
        for ring in rings.into_iter() {
            polygon.push(ring.into_iter().map(|pt| pt.into()).collect());
        }
        let polygons = vec![polygon];
        Self { polygons, data }
    }

    /// Push a polygon
    pub fn push<I, P, R>(&mut self, rings: I)
    where
        I: IntoIterator<Item = R>,
        R: IntoIterator<Item = P>,
        P: Into<Pt<F>>,
    {
        let mut polygon = vec![];
        for ring in rings.into_iter() {
            polygon.push(ring.into_iter().map(|pt| pt.into()).collect());
        }
        self.polygons.push(polygon);
    }
}

impl<F, D> Geometry<F> for GeomType<F, D>
where
    F: Float,
{
    type Data = D;

    fn bbox(&self) -> BBox<F> {
        match self {
            GeomType::Point(p) => p.bbox(),
            GeomType::Linestring(ls) => ls.bbox(),
            GeomType::Polygon(pg) => pg.bbox(),
        }
    }

    fn data(&self) -> &Self::Data {
        match self {
            GeomType::Point(p) => p.data(),
            GeomType::Linestring(ls) => ls.data(),
            GeomType::Polygon(pg) => pg.data(),
        }
    }
}
