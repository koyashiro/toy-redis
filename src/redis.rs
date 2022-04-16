use std::collections::HashMap;

pub struct RedisDB {
    dict: HashMap<String, String>,
}

impl RedisDB {
    pub fn new() -> Self {
        RedisDB {
            dict: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.dict.get(key)
    }

    pub fn set(&mut self, key: String, value: String) {
        self.dict.insert(key, value);
    }

    pub fn del(&mut self, key: &str) -> i64 {
        match self.dict.remove(key) {
            Some(_) => 1,
            None => 0,
        }
    }

    pub fn flushall(&mut self) {
        self.dict.clear();
    }
}

impl Default for RedisDB {
    fn default() -> Self {
        Self::new()
    }
}
