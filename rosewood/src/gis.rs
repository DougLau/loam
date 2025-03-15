// geometry.rs
//
// Copyright (c) 2021-2025  Douglas P Lau
//
//! Data types for GIS
use pointy::{BBox, Bounded, Bounds, Float, Pt, Seg};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// GIS geometry which can be stored in an RTree
pub trait Gis<F>
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
///
/// This geometry is one or more GIS points, along with associated data.
#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
pub struct Points<F, D>
where
    F: Float,
{
    /// Points in geometry
    pts: Vec<Pt<F>>,

    /// Associated data
    data: D,
}

/// Line string
#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
pub struct Linestring<F>
where
    F: Float,
{
    /// Points in line string
    pts: Vec<Pt<F>>,
}

/// Segment iterator for Linestring
struct SegIter<'a, F>
where
    F: Float,
{
    /// Point iterator
    iter: std::slice::Iter<'a, Pt<F>>,

    /// Previous point
    ppt: Option<Pt<F>>,
}

impl<F> Iterator for SegIter<'_, F>
where
    F: Float,
{
    type Item = Seg<F>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ppt.is_none() {
            self.ppt = Some(*self.iter.next()?);
        }
        let ppt = self.ppt?;
        let pt = self.iter.next();
        self.ppt = pt.copied();
        pt.map(|p| Seg::new(ppt, p))
    }
}

/// Line string geometry
///
/// This geometry is one or more GIS line strings, along with associated data.
#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
pub struct Linestrings<F, D>
where
    F: Float,
{
    /// Line strings in geometry
    lines: Vec<Linestring<F>>,

    /// Associated data
    data: D,
}

/// Polygon
#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
pub struct Polygon<F>
where
    F: Float,
{
    /// Points in polygon
    pts: Vec<Pt<F>>,
}

/// Polygon geometry
///
/// This geometry is one or more GIS polygons, along with associated data.
/// A polygon is a `Vec` of closed rings.  The winding order determines whether
/// a ring is "outer" or "inner".
#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
pub struct Polygons<F, D>
where
    F: Float,
{
    /// Polygons in geometry
    rings: Vec<Polygon<F>>,

    /// Associated data
    data: D,
}

/// Enum of defined geometries
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum Geom<F, D>
where
    F: Float,
{
    /// Point geometry
    Point(Points<F, D>),

    /// Linestring geometry
    Linestring(Linestrings<F, D>),

    /// Polygon geometry
    Polygon(Polygons<F, D>),
}

impl<F, D> Gis<F> for Points<F, D>
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

impl<F, D> Points<F, D>
where
    F: Float,
{
    /// Create new point geometry
    pub fn new(data: D) -> Self {
        let pts = Vec::new();
        Self { pts, data }
    }

    /// Add a point
    pub fn push<P>(&mut self, pt: P)
    where
        P: Into<Pt<F>>,
    {
        self.pts.push(pt.into());
    }

    /// Get point iterator
    pub fn iter(&self) -> impl Iterator<Item = &Pt<F>> {
        self.pts.iter()
    }
}

impl<F> Bounded<F> for &Linestring<F>
where
    F: Float,
{
    fn bounded_by(self, bbox: BBox<F>) -> bool {
        self.segments().any(|seg| seg.bounded_by(bbox))
    }
}

impl<F> Linestring<F>
where
    F: Float,
{
    /// Create a new line string
    fn new<I, P>(pts: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<Pt<F>>,
    {
        let pts = pts.into_iter().map(|pt| pt.into()).collect();
        Linestring { pts }
    }

    /// Get point iterator
    pub fn iter(&self) -> impl Iterator<Item = &Pt<F>> {
        self.pts.iter()
    }

    /// Get line segment iterator
    pub fn segments(&self) -> impl Iterator<Item = Seg<F>> + '_ {
        let iter = self.pts.iter();
        SegIter { iter, ppt: None }
    }
}

impl<F, D> Gis<F> for Linestrings<F, D>
where
    F: Float,
{
    type Data = D;

    fn bbox(&self) -> BBox<F> {
        let mut bbox = BBox::default();
        for line in self.lines.iter() {
            bbox.extend(line.iter());
        }
        bbox
    }

    fn data(&self) -> &Self::Data {
        &self.data
    }
}

impl<F, D> Bounded<F> for &Linestrings<F, D>
where
    F: Float,
{
    fn bounded_by(self, bbox: BBox<F>) -> bool {
        self.iter().any(|lines| lines.bounded_by(bbox))
    }
}

impl<F, D> Linestrings<F, D>
where
    F: Float,
{
    /// Create new line string geometry
    pub fn new(data: D) -> Self {
        let lines = Vec::new();
        Self { lines, data }
    }

    /// Push a line string
    pub fn push<I, P>(&mut self, pts: I)
    where
        I: IntoIterator<Item = P>,
        P: Into<Pt<F>>,
    {
        self.lines.push(Linestring::new(pts));
    }

    /// Get line string iterator
    pub fn iter(&self) -> impl Iterator<Item = &Linestring<F>> {
        self.lines.iter()
    }
}

/// Border around bounding box
///
/// The border is eight regions around the box, including the 4 cardinal and 4
/// ordinal directions.
#[derive(Clone, Copy, Debug, Default)]
struct BoundBorder {
    below: bool,
    below_left: bool,
    left: bool,
    above_left: bool,
    above: bool,
    above_right: bool,
    right: bool,
    below_right: bool,
}

impl BoundBorder {
    /// Add bounds to border
    fn add_bounds(&mut self, b: Bounds) -> bool {
        match b {
            Bounds::Below => self.below = true,
            Bounds::BelowLeft => self.below_left = true,
            Bounds::Left => self.left = true,
            Bounds::AboveLeft => self.above_left = true,
            Bounds::Above => self.above = true,
            Bounds::AboveRight => self.above_right = true,
            Bounds::Right => self.right = true,
            Bounds::BelowRight => self.below_right = true,
            Bounds::Within => return true,
        }
        false
    }

    /// Check if border is surrounding bounds
    ///
    /// If there are no gaps of 3 or more cardinal/ordinal directions, the shape
    /// is surrounding the box.  This can trigger false positives, but is much
    /// simpler than the "correct" algorithm.
    fn is_surrounding(&self) -> bool {
        if !(self.below | self.below_left | self.left) {
            return false;
        }
        if !(self.below_left | self.left | self.above_left) {
            return false;
        }
        if !(self.left | self.above_left | self.above) {
            return false;
        }
        if !(self.above_left | self.above | self.above_right) {
            return false;
        }
        if !(self.above | self.above_right | self.right) {
            return false;
        }
        if !(self.above_right | self.right | self.below_right) {
            return false;
        }
        if !(self.right | self.below_right | self.below) {
            return false;
        }
        if !(self.below_right | self.below | self.below_left) {
            return false;
        }
        true
    }
}

impl<F> Bounded<F> for &Polygon<F>
where
    F: Float,
{
    fn bounded_by(self, bbox: BBox<F>) -> bool {
        let mut border = BoundBorder::default();
        self.segments().any(|seg| {
            seg.bounded_by(bbox)
                || border.add_bounds(bbox.check(seg.p0.x, seg.p0.y))
        }) || border.is_surrounding()
    }
}

impl<F> Polygon<F>
where
    F: Float,
{
    /// Create a new polygon
    fn new<I, P>(pts: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<Pt<F>>,
    {
        let pts = pts.into_iter().map(|pt| pt.into()).collect();
        Polygon { pts }
    }

    /// Check if a polygon has clockwise winding order
    fn is_clockwise(&self) -> bool {
        if let Some(ext) = self.find_extreme_point() {
            let len = self.pts.len();
            let a = if ext > 0 { ext - 1 } else { len - 1 };
            let b = if ext < len - 1 { ext + 1 } else { 0 };
            // Make two vectors as edges pointing toward the extreme point
            let v0 = self.pts[a] - self.pts[ext];
            let v1 = self.pts[b] - self.pts[ext];
            // Cross product determines the winding order
            (v0 * v1) > F::zero()
        } else {
            false
        }
    }

    /// Find an extreme point on the convex hull of a polygon
    fn find_extreme_point(&self) -> Option<usize> {
        self.pts
            .iter()
            .enumerate()
            .min_by(|a, b| {
                (a.1.x, a.1.y)
                    .partial_cmp(&(b.1.x, b.1.y))
                    .unwrap_or(Ordering::Greater)
            })
            .map(|e| e.0)
    }

    /// Get point iterator
    pub fn iter(&self) -> impl Iterator<Item = &Pt<F>> {
        self.pts.iter()
    }

    /// Get line segment iterator
    pub fn segments(&self) -> impl Iterator<Item = Seg<F>> + '_ {
        let iter = self.pts.iter();
        SegIter { iter, ppt: None }
    }
}

impl<F, D> Gis<F> for Polygons<F, D>
where
    F: Float,
{
    type Data = D;

    fn bbox(&self) -> BBox<F> {
        let mut bbox = BBox::default();
        for ring in &self.rings {
            bbox.extend(ring.iter());
        }
        bbox
    }

    fn data(&self) -> &Self::Data {
        &self.data
    }
}

impl<F, D> Bounded<F> for &Polygons<F, D>
where
    F: Float,
{
    fn bounded_by(self, bbox: BBox<F>) -> bool {
        self.iter().any(|poly| poly.bounded_by(bbox))
    }
}

impl<F, D> Polygons<F, D>
where
    F: Float,
{
    /// Create new polygon geometry
    pub fn new(data: D) -> Self {
        let rings = Vec::new();
        Self { rings, data }
    }

    /// Push an outer polygon
    pub fn push_outer<I, P>(&mut self, ring: I)
    where
        I: IntoIterator<Item = P>,
        P: Into<Pt<F>>,
    {
        let mut ring = Polygon::new(ring);
        if !ring.is_clockwise() {
            ring.pts.reverse();
        }
        self.rings.push(ring);
    }

    /// Push an inner polygon
    pub fn push_inner<I, P>(&mut self, ring: I)
    where
        I: IntoIterator<Item = P>,
        P: Into<Pt<F>>,
    {
        let mut ring = Polygon::new(ring);
        if ring.is_clockwise() {
            ring.pts.reverse();
        }
        self.rings.push(ring);
    }

    /// Get polygon iterator
    pub fn iter(&self) -> impl Iterator<Item = &Polygon<F>> {
        self.rings.iter()
    }
}

impl<F, D> Gis<F> for Geom<F, D>
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
        let ring = Polygon::new([(0.0, 0.0), (1.0, 0.0), (0.0, 1.0)]);
        assert_eq!(false, ring.is_clockwise());
        let ring = Polygon::new([(0.0, 0.0), (0.0, 1.0), (1.0, 0.0)]);
        assert_eq!(true, ring.is_clockwise());
    }
}
