use std::str;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use tokio::io;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::{TcpListener, ToSocketAddrs};

use super::RedisDB;

pub struct RedisServer {
    db: Arc<Mutex<RedisDB>>,
    listener: TcpListener,
}

impl RedisServer {
    pub async fn bind<A: ToSocketAddrs>(addr: A) -> Result<Self> {
        let db = Arc::new(Mutex::new(RedisDB::new()));
        let listener = TcpListener::bind(addr).await?;
        Ok(Self { db, listener })
    }

    pub async fn run(&mut self) -> Result<()> {
        {
            let local_addr = self.listener.local_addr()?;
            println!("listening on {local_addr}");
        }

        loop {
            let (socket, socket_addr) = self.listener.accept().await?;
            println!("connected: {socket_addr}");

            let (r, w) = io::split(socket);
            let mut reader = BufReader::new(r);
            let mut writer = BufWriter::new(w);

            let db = Arc::clone(&self.db);

            tokio::spawn(async move {
                loop {
                    let mut buf = [0; 1];
                    let n = reader.read_exact(&mut buf).await?;
                    if n == 0 {
                        break;
                    }
                    if buf[0] != b'*' {
                        writer.write_all(b"-ERR\r\n").await?;
                        writer.flush().await?;
                        panic!("expected: {}, actual: {}", b'*', buf[0]);
                    }

                    let length = {
                        let mut buf = Vec::new();
                        loop {
                            let n = reader.read_until(b'\n', &mut buf).await?;
                            if n == 0 {
                                // TODO:
                                break;
                            }
                            if buf.ends_with(b"\r\n") {
                                break;
                            }
                        }
                        let l: i64 = str::from_utf8(&buf)?.trim_end_matches("\r\n").parse()?;
                        l
                    };

                    let mut inputs: Vec<Vec<u8>> = Vec::new();
                    for _ in 0..length {
                        let mut buf = [0; 1];
                        let n = reader.read_exact(&mut buf).await?;
                        if n == 0 {
                            return Ok(());
                        }
                        if buf[0] != b'$' {
                            writer.write_all(b"-ERR\r\n").await?;
                            writer.flush().await?;
                            panic!("expected: {}, actual: {}", b'$', buf[0]);
                        }

                        let length = {
                            let mut buf = Vec::new();
                            loop {
                                let n = reader.read_until(b'\n', &mut buf).await?;
                                if n == 0 {
                                    // TODO:
                                    break;
                                }
                                if buf.ends_with(b"\r\n") {
                                    break;
                                }
                            }
                            let l: i64 = str::from_utf8(&buf)?.trim_end_matches("\r\n").parse()?;
                            l
                        };

                        let mut buf = vec![0; length as usize];
                        let n = reader.read_exact(&mut buf).await?;
                        if n == 0 {
                            // TODO:
                            break;
                        }
                        inputs.push(buf);

                        let mut buf = Vec::new();
                        loop {
                            let n = reader.read_until(b'\n', &mut buf).await?;
                            if n == 0 {
                                // TODO:
                                break;
                            }
                            if buf.ends_with(b"\r\n") {
                                break;
                            }
                        }
                    }

                    let command = inputs[0].as_slice();

                    match command {
                        b"get" => {
                            let key = inputs[1].as_slice();
                            let value = {
                                let db = db.lock().unwrap();
                                db.get(key).map(|v| v.to_owned())
                            };
                            match value {
                                Some(value) => {
                                    writer.write_all(b"+").await?;
                                    writer.write_all(&value).await?;
                                    writer.write_all(b"\r\n").await?;
                                }
                                None => {
                                    writer.write_all(b"$-1\r\n").await?;
                                }
                            }
                        }
                        b"set" => {
                            let key = inputs[1].as_slice();
                            let value = inputs[2].as_slice();
                            {
                                let mut db = db.lock().unwrap();
                                db.set(key.to_owned(), value.to_owned());
                            }
                            writer.write_all(b"+OK\r\n").await?;
                        }
                        b"del" => {
                            let key = inputs[1].as_slice();
                            let n = {
                                let mut db = db.lock().unwrap();
                                db.del(key)
                            };
                            writer.write_all(b":").await?;
                            writer.write_all(n.to_string().as_bytes()).await?;
                            writer.write_all(b"\r\n").await?;
                        }
                        b"flushall" => {
                            {
                                let mut db = db.lock().unwrap();
                                db.flushall();
                            }
                            writer.write_all(b"+OK\r\n").await?;
                        }
                        _ => writer.write_all(b"-ERR\r\n").await?,
                    }
                    writer.flush().await?;
                }
                Ok(()) as Result<()>
            });

            println!("disconnected: {socket_addr}");
        }
    }
}
