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

/// A single point
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct Point<D, F>
where
    F: Float,
{
    pt: Pt<F>,
    data: D,
}

/// A collection of points
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct MultiPoint<D, F>
where
    F: Float,
{
    pts: Vec<Pt<F>>,
    data: D,
}

/// A string of points linked into line segments
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct LineString<D, F>
where
    F: Float,
{
    pts: Vec<Pt<F>>,
    data: D,
}

/// Collection of line strings
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct MultiLineString<D, F>
where
    F: Float,
{
    strings: Vec<LineString<D, F>>,
    data: D,
}

/// A shape containing closed rings of points
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct Polygon<D, F>
where
    F: Float,
{
    rings: Vec<LineString<D, F>>,
    data: D,
}

/// A collection of polygons
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct MultiPolygon<D, F>
where
    F: Float,
{
    polygons: Vec<Polygon<D, F>>,
    data: D,
}

/// Enum of defined geometries
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum GeomType<D, F>
where
    F: Float,
{
    /// Point geometry
    Point(Point<D, F>),

    /// Multi-point geometry
    MultiPoint(MultiPoint<D, F>),

    /// LineString geometry
    LineString(LineString<D, F>),

    /// Multi-LineString geometry
    MultiLineString(MultiLineString<D, F>),

    /// Polygon geometry
    Polygon(Polygon<D, F>),

    /// Multi-polygon geometry
    MultiPolygon(MultiPolygon<D, F>),
}

impl<D, F> Geometry<F> for Point<D, F>
where
    F: Float,
{
    type Data = D;

    fn bbox(&self) -> BBox<F> {
        BBox::from(self.pt)
    }

    fn data(&self) -> &Self::Data {
        &self.data
    }
}

impl<D, F> Point<D, F>
where
    F: Float,
{
    /// Create a new point
    pub fn new<P>(pt: P, data: D) -> Self
    where
        P: Into<Pt<F>>,
    {
        let pt = pt.into();
        Self { pt, data }
    }
}

impl<D, F> Geometry<F> for MultiPoint<D, F>
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

impl<D, F> MultiPoint<D, F>
where
    F: Float,
{
    /// Create a new multi-point
    pub fn new<I, P>(pts: I, data: D) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<Pt<F>>,
    {
        let pts = pts.into_iter().map(|p| p.into()).collect();
        Self { pts, data }
    }
}

impl<D, F> Geometry<F> for LineString<D, F>
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

impl<D, F> LineString<D, F>
where
    F: Float,
{
    /// Create a new line string
    pub fn new<I, P>(pts: I, data: D) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<Pt<F>>,
    {
        let pts = pts.into_iter().map(|p| p.into()).collect();
        Self { pts, data }
    }
}

impl<D, F> Geometry<F> for MultiLineString<D, F>
where
    F: Float,
{
    type Data = D;

    fn bbox(&self) -> BBox<F> {
        self.strings
            .iter()
            .fold(BBox::default(), |bb, ls| bb.extend(&ls.pts))
    }

    fn data(&self) -> &Self::Data {
        &self.data
    }
}

impl<D, F> MultiLineString<D, F>
where
    F: Float,
{
    /// Create a new multi-line string
    pub fn new<I, R>(strings: I, data: D) -> Self
    where
        I: IntoIterator<Item = R>,
        R: Into<LineString<D, F>>,
    {
        let strings = strings.into_iter().map(|r| r.into()).collect();
        Self { strings, data }
    }
}

impl<D, F> Geometry<F> for Polygon<D, F>
where
    F: Float,
{
    type Data = D;

    fn bbox(&self) -> BBox<F> {
        self.rings
            .iter()
            .fold(BBox::default(), |bb, ring| bb.extend(&ring.pts))
    }

    fn data(&self) -> &Self::Data {
        &self.data
    }
}

impl<D, F> Polygon<D, F>
where
    F: Float,
{
    /// Create a new polygon
    pub fn new<I, R>(rings: I, data: D) -> Self
    where
        I: IntoIterator<Item = R>,
        R: Into<LineString<D, F>>,
    {
        let rings = rings.into_iter().map(|r| r.into()).collect();
        Self { rings, data }
    }
}

impl<D, F> Geometry<F> for MultiPolygon<D, F>
where
    F: Float,
{
    type Data = D;

    fn bbox(&self) -> BBox<F> {
        self.polygons.iter().fold(BBox::default(), |bb, pg| {
            pg.rings.iter().fold(bb, |b, ring| b.extend(&ring.pts))
        })
    }

    fn data(&self) -> &Self::Data {
        &self.data
    }
}

impl<D, F> MultiPolygon<D, F>
where
    F: Float,
{
    /// Create a new multi-polygon
    pub fn new<I, R>(polygons: I, data: D) -> Self
    where
        I: IntoIterator<Item = R>,
        R: Into<Polygon<D, F>>,
    {
        let polygons = polygons.into_iter().map(|r| r.into()).collect();
        Self { polygons, data }
    }
}

impl<D, F> Geometry<F> for GeomType<D, F>
where
    F: Float,
{
    type Data = D;

    fn bbox(&self) -> BBox<F> {
        match self {
            GeomType::Point(p) => p.bbox(),
            GeomType::MultiPoint(mp) => mp.bbox(),
            GeomType::LineString(ls) => ls.bbox(),
            GeomType::MultiLineString(mls) => mls.bbox(),
            GeomType::Polygon(pg) => pg.bbox(),
            GeomType::MultiPolygon(mpg) => mpg.bbox(),
        }
    }

    fn data(&self) -> &Self::Data {
        match self {
            GeomType::Point(p) => p.data(),
            GeomType::MultiPoint(mp) => mp.data(),
            GeomType::LineString(ls) => ls.data(),
            GeomType::MultiLineString(mls) => mls.data(),
            GeomType::Polygon(pg) => pg.data(),
            GeomType::MultiPolygon(mpg) => mpg.data(),
        }
    }
}
