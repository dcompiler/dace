use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ListSerializable <T> {
    accesses: Vec<T>,
}

impl<T: > ListSerializable <T> {
    pub fn new() -> Self {
        ListSerializable {
            accesses: Vec::new(),
        }
    }

    pub fn add(&mut self, value: T) {
        self.accesses.push(value);
    }

    pub fn get_vec(&self) -> &Vec<T> {
        &self.accesses
    }

}

impl<T> Default for ListSerializable <T> {
    fn default() -> Self {
        Self::new()
    }
}
