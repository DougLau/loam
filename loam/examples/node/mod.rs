use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Node {
    name: String,
    children: Vec<u64>,
    count: u32,
}

impl Node {
    pub fn new(name: &str) -> Self {
        Node {
            name: name.into(),
            children: vec![],
            count: 0,
        }
    }

    pub fn push(&mut self, id: u64) {
        self.children.push(id);
        self.count += 1;
    }
}
