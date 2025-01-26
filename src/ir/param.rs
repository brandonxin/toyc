use super::Value;

pub struct Param {
    name: String,
}

impl Param {
    pub fn new(name: String) -> Param {
        Param { name }
    }
}

impl Value for Param {
    fn name(&self) -> &str {
        &self.name
    }
}
