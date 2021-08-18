pub mod node;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut root = node::Node::new("Root");
    root.push(20);
    root.push(40);
    let mut w = loam::Writer::new("test.loam")?;
    let id = w.push(&root)?;
    w.checkpoint(id)?;
    Ok(())
}
