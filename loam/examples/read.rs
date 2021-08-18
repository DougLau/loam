pub mod node;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let r = loam::Reader::new("test.loam")?;
    let id = r.root()?;
    let root: node::Node = r.lookup(id)?;
    dbg!(root);
    Ok(())
}
