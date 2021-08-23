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
    /// Get bounding box
    fn bbox(&self) -> BBox<F>;
}

/// A collection of points
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct MultiPoint<F>
where
    F: Float,
{
    pts: Vec<Pt<F>>,
}

/// A string of points linked into line segments
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct LineString<F>
where
    F: Float,
{
    pts: Vec<Pt<F>>,
}

/// Collection of line strings
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct MultiLineString<F>
where
    F: Float,
{
    strings: Vec<LineString<F>>,
}

/// A shape containing closed rings of points
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct Polygon<F>
where
    F: Float,
{
    rings: Vec<LineString<F>>,
}

/// A collection of polygons
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct MultiPolygon<F>
where
    F: Float,
{
    polygons: Vec<Polygon<F>>,
}

/// Enum of defined geometries
pub enum GeomType<F>
where
    F: Float,
{
    /// Point geometry
    Point(Pt<F>),

    /// Multi-point geometry
    MultiPoint(MultiPoint<F>),

    /// LineString geometry
    LineString(LineString<F>),

    /// Multi-LineString geometry
    MultiLineString(MultiLineString<F>),

    /// Polygon geometry
    Polygon(Polygon<F>),

    /// Multi-polygon geometry
    MultiPolygon(MultiPolygon<F>),
}

impl<F> Geometry<F> for MultiPoint<F>
where
    F: Float,
{
    fn bbox(&self) -> BBox<F> {
        BBox::new(&self.pts)
    }
}

impl<F> MultiPoint<F>
where
    F: Float,
{
    /// Create a new multi-point
    pub fn new<I, P>(pts: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<Pt<F>>,
    {
        let pts = pts.into_iter().map(|p| p.into()).collect();
        Self { pts }
    }
}

impl<F> Geometry<F> for LineString<F>
where
    F: Float,
{
    fn bbox(&self) -> BBox<F> {
        BBox::new(&self.pts)
    }
}

impl<F> LineString<F>
where
    F: Float,
{
    /// Create a new line string
    pub fn new<I, P>(pts: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<Pt<F>>,
    {
        let pts = pts.into_iter().map(|p| p.into()).collect();
        Self { pts }
    }
}

impl<F> Geometry<F> for MultiLineString<F>
where
    F: Float,
{
    fn bbox(&self) -> BBox<F> {
        self.strings
            .iter()
            .fold(BBox::default(), |bb, ls| bb.extend(&ls.pts))
    }
}

impl<F> MultiLineString<F>
where
    F: Float,
{
    /// Create a new multi-line string
    pub fn new<I, R>(strings: I) -> Self
    where
        I: IntoIterator<Item = R>,
        R: Into<LineString<F>>,
    {
        let strings = strings.into_iter().map(|r| r.into()).collect();
        Self { strings }
    }
}

impl<F> Geometry<F> for Polygon<F>
where
    F: Float,
{
    fn bbox(&self) -> BBox<F> {
        self.rings
            .iter()
            .fold(BBox::default(), |bb, ring| bb.extend(&ring.pts))
    }
}

impl<F> Polygon<F>
where
    F: Float,
{
    /// Create a new polygon
    pub fn new<I, R>(rings: I) -> Self
    where
        I: IntoIterator<Item = R>,
        R: Into<LineString<F>>,
    {
        let rings = rings.into_iter().map(|r| r.into()).collect();
        Self { rings }
    }
}

impl<F> Geometry<F> for MultiPolygon<F>
where
    F: Float,
{
    fn bbox(&self) -> BBox<F> {
        self.polygons.iter().fold(BBox::default(), |bb, pg| {
            pg.rings.iter().fold(bb, |b, ring| b.extend(&ring.pts))
        })
    }
}

impl<F> MultiPolygon<F>
where
    F: Float,
{
    /// Create a new multi-polygon
    pub fn new<I, R>(polygons: I) -> Self
    where
        I: IntoIterator<Item = R>,
        R: Into<Polygon<F>>,
    {
        let polygons = polygons.into_iter().map(|r| r.into()).collect();
        Self { polygons }
    }
}

impl<F> Geometry<F> for GeomType<F>
where
    F: Float,
{
    fn bbox(&self) -> BBox<F> {
        match self {
            GeomType::Point(p) => BBox::from(p),
            GeomType::MultiPoint(mp) => mp.bbox(),
            GeomType::LineString(ls) => ls.bbox(),
            GeomType::MultiLineString(mls) => mls.bbox(),
            GeomType::Polygon(pg) => pg.bbox(),
            GeomType::MultiPolygon(mpg) => mpg.bbox(),
        }
    }
}
