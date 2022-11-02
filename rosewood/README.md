# Rosewood

A file-based [RTree] for geospatial data, such as `Point`s, `Linestring`s and
`Polygon`s.

### Writing

```rust
use rosewood::{BulkWriter, gis::Points};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut w = BulkWriter::new("points.loam")?;
    let mut a = Points::new("point A".to_string());
    a.push((5.0, 1.0));
    let mut b = Points::new("point B".to_string());
    b.push((3.0, 7.3));
    w.push(&a)?;
    w.push(&b)?;
    w.finish()?;
    Ok(())
}
```

`GisData` can have associated data, which must implement the `Serialize`
/ `Deserialize` traits from [serde].  In the example above, this is `String`.


[RTree]: https://en.wikipedia.org/wiki/R-tree
[serde]: https://serde.rs
