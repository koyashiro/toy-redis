mod redis;

use std::net::TcpListener;
use std::str;

use serde_resp::{de, ser, RESP};

use crate::redis::RedisDB;

fn main() {
    let mut db = RedisDB::new();

    let listener = TcpListener::bind("0.0.0.0:6379").unwrap();

    for result in listener.incoming() {
        let mut stream = result.unwrap();

        let addr = stream.peer_addr().unwrap();
        println!("connected: {addr}");

        loop {
            let resp: RESP = match de::from_reader(&mut stream) {
                Ok(r) => r,
                Err(e) => {
                    println!("error: {e}");
                    println!("disconnected: {addr}");
                    break;
                }
            };

            let v = match resp {
                RESP::Array(a) => a.unwrap(),
                _ => todo!(),
            };

            if v.is_empty() {
                continue;
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
                    let value = db.get(key);
                    match value {
                        Some(v) => {
                            ser::to_writer(
                                &RESP::BulkString(Some(v.as_bytes().to_owned())),
                                &mut stream,
                            )
                            .unwrap();
                        }
                        None => {
                            ser::to_writer(&RESP::BulkString(None), &mut stream).unwrap();
                        }
                    }
                }
                "del" => {
                    if v.len() != 2 {
                        panic!();
                    }

                    let key = match &v[1] {
                        RESP::BulkString(s) => str::from_utf8(s.as_ref().unwrap()).unwrap(),
                        _ => todo!(),
                    };

                    println!("DEL {key}");
                    let n = db.del(key);
                    ser::to_writer(&RESP::Integer(n), &mut stream).unwrap();
                }
                "flushall" => {
                    if v.len() != 1 {
                        panic!();
                    }

                    println!("FLUSHALL");
                    db.flushall();
                    ser::to_writer(&RESP::SimpleString(String::from("OK")), &mut stream).unwrap();
                }
                "command" => {
                    println!("COMMAND");
                    ser::to_writer(&RESP::SimpleString(String::from("OK")), &mut stream).unwrap();
                }
                _ => {
                    let args = "(TODO)";
                    let error_message = format!(
                        "ERR unknown command `{command}`, with args beginning with: {args}"
                    );
                    println!("{error_message}");
                    ser::to_writer(&RESP::Error(error_message), &mut stream).unwrap();
                }
            }
        }
    }
}
