# Loam

This is a Rust library for storing and querying tree-like data structures in
files.  The motivating project is [rosewood], which stores geospatial data in an
[R-Tree].

__Loam__ allows you to store anything which implements `Serialize`.  Data is
appended to the end of the file and never modified once written.  This enables
the use of `mmap` to read files without the risk of undefined behavior.

## Write Example

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = loam::Writer::new("../target/test.loam")?;
    let id = writer.push(&"Don't forget me!")?;
    writer.checkpoint(id)?;
    Ok(())
}
```

## Read Example

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let reader = loam::Reader::new("../target/test.loam")?;
    let id = reader.root()?;
    let msg: String = reader.lookup(id)?;
    dbg!(msg);
    Ok(())
}
```

## File Format

A __loam__ file starts with a __Header__, followed by a series of __Chunks__.

### Header

The header is fixed-length ASCII text (8 bytes).

Field         | Value        | Bytes
--------------|--------------|------
Magic         | `loam`       | 4
Major Version | digits: `00` | 2
Minor Version | digits: `00` | 2

### Chunks

A chunk consists of these fields, serialized using [bincode]:

Field      | Description
-----------|--------------------------------------------------
Length     | Number of bytes in *Data* (variable-size integer)
Data       | Serialized chunk data
Checksum † | CRC-32 of *Length* + *Data* (fixed-size integer)

An __Id__ is the file offset of a chunk.  It can be used to `Deserialize` the
Data field.

† Checksums are only included if the `crc` feature is enabled.

### Checkpoint

A checkpoint is a special chunk containing a fixed-size `u64` of the root
__Id__.  A file must always end with a checkpoint, to allow readers to lookup
the root without needing to scan the entire file.


[bincode]: https://github.com/bincode-org/bincode
[rosewood]: ../rosewood/index.html
[r-tree]: https://en.wikipedia.org/wiki/R-tree
