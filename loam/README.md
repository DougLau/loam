# Loam

A Rust library for storing tree-like data structures to files.  The intended
use-case is for querying data which does not fit into RAM.  The motivating
project uses GeoSpatial data in an [RTree].

`Loam` allows you to store anything which implements `Serialize`, and gives you
an __id__ to retrive it later.

## Write API

- Open file in append mode
- Push data where D: Serialize (returning its __id__)
- Checkpoint

## Read API

- Open file for reading
- Lookup root / index Chunk (from last checkpoint)
- struct Chunk has an __id__ item (file offset)
- Lookup a Chunk by __id__ (no borrowed data)
- View a Chunk by __id__ (lifetime bound to loam file)
- Verify checksums for all chunks

## File Format

A `loam` file starts with a header, followed by a series of chunks.  The header
is fixed-length 16-byte *magic* string and version number.

Reading a `loam` file is done with mmap.

Writing is always append-only: data must not be modified once written.

### Header

The header must appear first in the file, and may only appear once.

Part     | Description
---------|------------
Magic    | `loam`
Version  | `0000`

### Chunks

Chunks consist of three parts, serialized using [bincode].

Part     | Description
---------|--------------------------------------------------
Length   | Number of bytes in *Data* (variable-size integer)
Data     | Chunk data (variable-size integer)
Checksum | CRC-32 of *Length* + *Data* (fixed-size integer)

The file offset of a chunk is its __id__.  These can be used by chunks to
reference data in other chunks.

A checkpoint is a special chunk containing a fixed-size `u64` of the root
__id__.  A `loam` file must always end with a checkpoint, to allow readers to
lookup the root without needing to scan the entire file.


[bincode]: https://github.com/bincode-org/bincode
[rtree]: https://en.wikipedia.org/wiki/R-tree
