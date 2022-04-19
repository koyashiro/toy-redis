use std::io::Cursor;
use std::net::Incoming;
use std::str;
use std::sync::{Arc, Mutex};

use anyhow::{bail, Result};
use bytes::{Buf, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, ToSocketAddrs};

use super::{RedisDB, RESP};

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
            let (mut socket, socket_addr) = self.listener.accept().await?;
            println!("connected: {socket_addr}");

            let db = Arc::clone(&self.db);

            tokio::spawn(async move {
                let mut buf = BytesMut::with_capacity(4096);

                loop {
                    if socket.read_buf(&mut buf).await? == 0 {
                        if !buf.is_empty() {
                            // TODO
                        }
                        // close connection
                        return Ok(()) as Result<()>;
                    }

                    let len = {
                        let mut cursor = Cursor::new(&buf);
                        loop {
                            let pos = cursor.position();
                            match parse_resp(&mut cursor) {
                                Ok(resp) => match resp {
                                    RESP::Arrays(arrays) => {
                                        let arrays = arrays.unwrap();
                                        let command = match &arrays[0] {
                                            RESP::BulkStrings(bs) => match bs {
                                                Some(bs) => bs.as_slice(),
                                                None => todo!(),
                                            },
                                            _ => todo!(),
                                        };
                                        match command {
                                            b"get" => {
                                                let key = match &arrays[1] {
                                                    RESP::BulkStrings(bs) => match bs {
                                                        Some(bs) => bs.as_slice(),
                                                        None => todo!(),
                                                    },
                                                    _ => todo!(),
                                                };
                                                let value = {
                                                    let db = db.lock().unwrap();
                                                    db.get(key).map(|v| v.to_owned())
                                                };
                                                match value {
                                                    Some(value) => {
                                                        socket.write_all(b"+").await.unwrap();
                                                        socket
                                                            .write_all(value.as_slice())
                                                            .await
                                                            .unwrap();
                                                        socket.write_all(b"\r\n").await.unwrap();
                                                        socket.flush().await.unwrap();
                                                    }
                                                    None => {
                                                        socket.write_all(b"*-1\r\n").await.unwrap();
                                                        socket.flush().await.unwrap();
                                                    }
                                                }
                                            }
                                            b"set" => {
                                                let key = match &arrays[1] {
                                                    RESP::BulkStrings(bs) => match bs {
                                                        Some(bs) => bs.as_slice(),
                                                        None => todo!(),
                                                    },
                                                    _ => todo!(),
                                                };

                                                let value = match &arrays[2] {
                                                    RESP::BulkStrings(bs) => match bs {
                                                        Some(bs) => bs.as_slice(),
                                                        None => todo!(),
                                                    },
                                                    _ => todo!(),
                                                };

                                                {
                                                    let mut db = db.lock().unwrap();
                                                    db.set(key.to_owned(), value.to_owned());
                                                }

                                                socket.write_all(b"+OK\r\n").await;
                                                socket.flush();
                                            }
                                            _ => todo!(),
                                        }
                                    }
                                    _ => todo!(),
                                },
                                Err(_) => {
                                    cursor.set_position(pos);
                                    break;
                                }
                            }
                        }
                        cursor.position() as usize
                    };

                    buf.advance(len);
                }
            });

            println!("disconnected: {socket_addr}");
        }
    }
}

fn parse_resp(cursor: &mut Cursor<&BytesMut>) -> Result<RESP> {
    if cursor.position() as usize == cursor.get_ref().len() {
        bail!("end")
    }

    let i = cursor.position() as usize;
    let prefix = cursor.get_ref()[i];
    match prefix {
        b'+' => parse_simple_strings(cursor),
        b'-' => parse_errors(cursor),
        b'*' => parse_arrays(cursor),
        b'$' => parse_bulk_strings(cursor),
        b':' => parse_integers(cursor),
        _ => {
            todo!();
        }
    }
}

fn get_line<'a>(cursor: &'a mut Cursor<&BytesMut>) -> Result<&'a [u8]> {
    let start = cursor.position() as _;
    let end = cursor.get_ref().len();

    for i in start..end {
        if cursor.get_ref()[i] == b'\r' && cursor.get_ref()[i + 1] == b'\n' {
            cursor.set_position((i + 2) as _);

            let buf = &cursor.get_ref()[start..i];
            return Ok(buf);
        }
    }
    bail!("incoming error")
}

fn parse_simple_strings(cursor: &mut Cursor<&BytesMut>) -> Result<RESP> {
    cursor.set_position(cursor.position() + 1);
    let line = get_line(cursor)?;
    let string = String::from_utf8(line.to_owned())?;
    let resp = RESP::SimpleStrings(string);
    Ok(resp)
}

fn parse_errors(cursor: &mut Cursor<&BytesMut>) -> Result<RESP> {
    todo!();
}

fn parse_integers(cursor: &mut Cursor<&BytesMut>) -> Result<RESP> {
    todo!();
}

fn parse_arrays(cursor: &mut Cursor<&BytesMut>) -> Result<RESP> {
    cursor.set_position(cursor.position() + 1);
    let line = get_line(cursor)?;
    let len: i64 = str::from_utf8(line)?.parse()?;
    if len.is_negative() {
        return Ok(RESP::Arrays(None));
    }
    let len = len as usize;
    let mut vec: Vec<RESP> = Vec::with_capacity(len);
    for _ in 0..len {
        vec.push(parse_resp(cursor)?);
    }
    Ok(RESP::Arrays(Some(vec)))
}

fn parse_bulk_strings(cursor: &mut Cursor<&BytesMut>) -> Result<RESP> {
    cursor.set_position(cursor.position() + 1);
    let line = get_line(cursor)?;
    let len: i64 = str::from_utf8(line)?.parse()?;
    if len.is_negative() {
        return Ok(RESP::Arrays(None));
    }
    let line = get_line(cursor)?;
    Ok(RESP::BulkStrings(Some(line.to_owned())))
}
