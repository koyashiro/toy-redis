use std::collections::HashMap;
use std::net::TcpListener;
use std::str::{self};

use serde_resp::{de, ser, RESP};

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

impl Default for RedisDB {
    fn default() -> Self {
        Self::new()
    }
}

fn main() {
    let mut db = RedisDB::new();

    let listener = TcpListener::bind("0.0.0.0:6379").unwrap();

    for result in listener.incoming() {
        let mut stream = result.unwrap();

        let addr = stream.peer_addr().unwrap();
        println!("connected: {addr}");

        loop {
            let resp: RESP = de::from_reader(&mut stream).unwrap();

            let v = match resp {
                RESP::Array(a) => a.unwrap(),
                _ => todo!(),
            };

            if v.is_empty() {
                panic!("v is empty");
            }

            let command = match &v[0] {
                RESP::BulkString(s) => str::from_utf8(s.as_ref().unwrap()).unwrap().to_lowercase(),
                _ => todo!(),
            };

            match command.as_str() {
                "set" => {
                    if v.len() != 3 {
                        panic!();
                    }

                    let key = match &v[1] {
                        RESP::BulkString(s) => str::from_utf8(s.as_ref().unwrap()).unwrap(),
                        _ => todo!(),
                    };

                    let value = match &v[2] {
                        RESP::BulkString(s) => str::from_utf8(s.as_ref().unwrap()).unwrap(),
                        _ => todo!(),
                    };

                    println!("SET {key} {value}");
                    db.set(key.to_string(), value.to_string());
                    ser::to_writer(&RESP::SimpleString(String::from("OK")), &mut stream).unwrap();
                }
                "get" => {
                    if v.len() != 2 {
                        panic!();
                    }

                    let key = match &v[1] {
                        RESP::BulkString(s) => str::from_utf8(s.as_ref().unwrap()).unwrap(),
                        _ => todo!(),
                    };

                    println!("GET {key}");
                    let value = db.get(key).unwrap();
                    ser::to_writer(
                        &RESP::BulkString(Some(value.as_bytes().to_owned())),
                        &mut stream,
                    )
                    .unwrap();
                }
                "command" => {
                    println!("COMMAND");
                    ser::to_writer(&RESP::SimpleString(String::from("OK")), &mut stream).unwrap();
                }
                _ => todo!(),
            }
        }
    }
}
