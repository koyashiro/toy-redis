use std::collections::HashMap;

pub struct RedisDB {
    dict: HashMap<Vec<u8>, Vec<u8>>,
}

impl RedisDB {
    pub fn new() -> Self {
        Self {
            dict: HashMap::new(),
        }
    }

    pub fn get(&self, key: &[u8]) -> Option<&Vec<u8>> {
        self.dict.get(key)
    }

    pub fn set(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.dict.insert(key, value);
    }

    pub fn del(&mut self, key: &[u8]) -> i64 {
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
