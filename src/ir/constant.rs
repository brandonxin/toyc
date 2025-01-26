use super::Value;

pub struct Constant {
    name: String,
    value: u64,
}

impl Constant {
    pub fn new(_name: String, value: u64) -> Constant {
        Constant {
            name: format!("${value}"),
            value,
        }
    }

    pub fn value(&self) -> u64 {
        self.value
    }
}

impl Value for Constant {
    fn name(&self) -> &str {
        &self.name
    }
}
