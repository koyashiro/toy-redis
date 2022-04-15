use std::collections::HashMap;
use std::io::{stdin, stdout, BufRead, BufReader, Write};

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

    pub fn del(&mut self, key: &str) {
        self.dict.remove(key);
    }

    pub fn flushall(&mut self) {
        self.dict.clear();
    }
}

fn main() {
    let mut db = RedisDB::new();

    let mut reader = BufReader::new(stdin());
    let mut buf = String::new();

    loop {
        print!("> ");
        stdout().flush().unwrap();

        buf.clear();
        reader.read_line(&mut buf).unwrap();
        let inputs: Vec<&str> = buf.trim().split_whitespace().collect();
        if inputs.is_empty() {
            continue;
        }

        let command = inputs[0];
        match command {
            "get" => {
                if inputs.len() != 2 {
                    println!("(error) ERR wrong number of arguments for 'get' command");
                    continue;
                }

                let key = inputs[1];
                let value = db.get(key);
                match value {
                    Some(v) => {
                        println!("{:?}", v);
                    }
                    None => {
                        println!("(nil)");
                    }
                }
            }
            "set" => {
                if inputs.len() != 3 {
                    println!("(error) ERR wrong number of arguments for 'set' command");
                    continue;
                }

                let key = inputs[1];
                let value = inputs[2];
                db.set(key.to_string(), value.to_string());
                println!("OK");
            }
            "del" => {
                if inputs.len() != 2 {
                    println!("(error) ERR wrong number of arguments for 'del' command");
                    continue;
                }

                let key = inputs[1];
                db.del(key);
            }
            "flushall" => {
                db.flushall();
                println!("OK");
            }
            _ => {
                let args = inputs[1..]
                    .iter()
                    .map(|arg| format!("`{arg}`"))
                    .collect::<Vec<String>>()
                    .join(", ");
                println!(
                    "(error) ERR unknown command `{command}`, with args beginning with: {args}",
                )
            }
        }
    }
}
