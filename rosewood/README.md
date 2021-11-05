# Rosewood

A file-based [RTree] for geospatial data, such as `Point`s, `Linestring`s and
`Polygon`s.

### Writing

```rust
use rosewood::{BulkWriter, Point};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut w = BulkWriter::new("points.loam")?;
    let a = Point::new((5.0, 1.0), "point A".to_string());
    let b = Point::new((3.0, 7.3), "point B".to_string());
    w.push(&a)?;
    w.push(&b)?;
    w.finish()?;
    Ok(())
}
```

Each `Geometry` can have associated data, which must implement the `Serialize`
/ `Deserialize` traits from [serde].  In the example above, this is `String`.


[RTree]: https://en.wikipedia.org/wiki/R-tree
[serde]: https://serde.rs
