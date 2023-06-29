use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ListSerializable {
    accesses: Vec<usize>,
}

impl ListSerializable {
    pub fn new() -> Self {
        ListSerializable {
            accesses: Vec::new(),
        }
    }

    pub fn add(&mut self, value: usize) {
        self.accesses.push(value);
    }
}

impl Default for ListSerializable {
    fn default() -> Self {
        Self::new()
    }
}
