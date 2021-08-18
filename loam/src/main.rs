use serde::Serialize;

#[derive(Serialize)]
struct Root {
    name: String,
    count: u32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = Root {
        name: String::from("Root"),
        count: 1,
    };
    let mut w = loam::Writer::new("test.loam")?;
    let id = w.push(&root)?;
    w.checkpoint(id)?;
    Ok(())
}
