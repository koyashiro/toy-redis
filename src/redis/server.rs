use std::io::Cursor;
use std::str;
use std::sync::{Arc, Mutex};

use anyhow::Result;
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

            let mut db = Arc::clone(&self.db);

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

                    let mut cursor = Cursor::new(&buf);
                    loop {
                        let resp = read_frame(&mut cursor);
                        if let Err(FrameParseError::IncomingError) = resp {
                            break;
                        }
                        let resp = match resp {
                            Ok(resp) => resp,
                            Err(FrameParseError::IncomingError) => break,
                            _ => {
                                socket.write_all(b"-ERR\r\n").await?;
                                socket.flush().await?;
                                continue;
                            }
                        };

                        match execute(&mut db, resp) {
                            Ok(rv) => match rv {
                                ExecuteReturnValue::Get(v) => match v {
                                    Some(v) => {
                                        socket.write_all(b"+").await?;
                                        socket.write_all(v.as_slice()).await?;
                                        socket.write_all(b"\r\n").await?;
                                        socket.flush().await?;
                                    }
                                    None => {
                                        socket.write_all(b"$-1\r\n").await?;
                                        socket.flush().await?;
                                    }
                                },
                                ExecuteReturnValue::Set => {
                                    socket.write_all(b"+OK\r\n").await?;
                                    socket.flush().await?;
                                }
                                ExecuteReturnValue::Del(n) => {
                                    socket.write_all(b":").await?;
                                    socket.write_all(n.to_string().as_bytes()).await?;
                                    socket.write_all(b"\r\n").await?;
                                    socket.flush().await?;
                                }
                                ExecuteReturnValue::Flushall => {
                                    socket.write_all(b"+OK\r\n").await?;
                                    socket.flush().await?;
                                }
                            },
                            Err(_) => {
                                socket.write_all(b"-ERR\r\n").await?;
                                socket.flush().await?;
                            }
                        }
                    }
                    let cnt = cursor.position() as usize;
                    buf.advance(cnt);
                }
            });

            println!("disconnected: {socket_addr}");
        }
    }
}

#[derive(Debug)]
pub enum FrameParseError {
    IncomingError,
    InvalidUtf8Error,
}

fn read_frame(cursor: &mut Cursor<&BytesMut>) -> Result<RESP, FrameParseError> {
    let first = match cursor.get_ref().get(cursor.position() as usize) {
        Some(v) => v,
        None => return Err(FrameParseError::IncomingError),
    };

    match first {
        b'+' => read_simple_strings(cursor),
        b'-' => read_errors_strings(cursor),
        b'*' => read_arrays(cursor),
        b'$' => read_bulk_strings(cursor),
        b':' => read_integers(cursor),
        _ => {
            todo!();
        }
    }
}

fn read_line<'a>(cursor: &'a mut Cursor<&BytesMut>) -> Result<&'a [u8], FrameParseError> {
    let start = cursor.position() as _;
    let end = cursor.get_ref().len();

    for i in start..end {
        if cursor.get_ref()[i] == b'\r' && cursor.get_ref()[i + 1] == b'\n' {
            cursor.set_position((i + 2) as _);

            let buf = &cursor.get_ref()[start..i];
            return Ok(buf);
        }
    }

    Err(FrameParseError::IncomingError)
}

fn read_simple_strings(cursor: &mut Cursor<&BytesMut>) -> Result<RESP, FrameParseError> {
    let line = read_line(cursor)?;
    let str = str::from_utf8(&line[1..]).map_err(|_| FrameParseError::InvalidUtf8Error)?;
    Ok(RESP::SimpleStrings(str.to_string()))
}

fn read_errors_strings(cursor: &mut Cursor<&BytesMut>) -> Result<RESP, FrameParseError> {
    let line = read_line(cursor)?;
    let str = str::from_utf8(&line[1..]).map_err(|_| FrameParseError::InvalidUtf8Error)?;
    Ok(RESP::Errors(str.to_string()))
}

fn read_integers(cursor: &mut Cursor<&BytesMut>) -> Result<RESP, FrameParseError> {
    let line = read_line(cursor)?;
    let str = str::from_utf8(&line[1..]).map_err(|_| FrameParseError::InvalidUtf8Error)?;
    let i = str
        .parse::<i64>()
        .map_err(|_| FrameParseError::InvalidUtf8Error)?;
    Ok(RESP::Integers(i))
}

fn read_bulk_strings(cursor: &mut Cursor<&BytesMut>) -> Result<RESP, FrameParseError> {
    let pos = cursor.position();

    let line = read_line(cursor)?;
    let str = str::from_utf8(&line[1..]).map_err(|_| FrameParseError::InvalidUtf8Error)?;
    let len = str
        .parse::<i64>()
        .map_err(|_| FrameParseError::InvalidUtf8Error)?;

    if len.is_negative() {
        return Ok(RESP::BulkStrings(None));
    }
    let len = len as usize;

    let start = cursor.position() as usize;
    match cursor.get_ref().get(start..start + len) {
        Some(v) => {
            let result = read_line(cursor);
            if let Err(FrameParseError::IncomingError) = result {
                // rollback
                cursor.set_position(pos);
                return Err(FrameParseError::IncomingError);
            }

            let vec = v.to_vec();
            Ok(RESP::BulkStrings(Some(vec)))
        }
        None => {
            // rollback
            cursor.set_position(pos);
            Err(FrameParseError::IncomingError)
        }
    }
}

fn read_arrays(cursor: &mut Cursor<&BytesMut>) -> Result<RESP, FrameParseError> {
    let pos = cursor.position();

    let line = read_line(cursor)?;
    let str = str::from_utf8(&line[1..]).map_err(|_| FrameParseError::InvalidUtf8Error)?;
    let len = str
        .parse::<i64>()
        .map_err(|_| FrameParseError::InvalidUtf8Error)?;

    if len.is_negative() {
        return Ok(RESP::BulkStrings(None));
    }

    let mut vec: Vec<RESP> = vec![];
    for _ in 0..len {
        let resp = read_frame(cursor);
        let resp = resp.map_err(|e| {
            // rollback
            cursor.set_position(pos);
            e
        })?;
        vec.push(resp);
    }

    Ok(RESP::Arrays(Some(vec)))
}

enum ExecuteReturnValue {
    Get(Option<Vec<u8>>),
    Set,
    Del(i64),
    Flushall,
}

fn execute(db: &mut Arc<Mutex<RedisDB>>, resp: RESP) -> Result<ExecuteReturnValue, ()> {
    let array = match resp {
        RESP::Arrays(Some(a)) => a,
        _ => return Err(()),
    };
    let command = match array.get(0) {
        Some(c) => c,
        None => return Err(()),
    };
    let command = match command {
        RESP::BulkStrings(Some(c)) => c.as_slice(),
        _ => return Err(()),
    };
    match command {
        b"get" => {
            if array.len() != 2 {
                return Err(());
            }
            let key = match &array[1] {
                RESP::BulkStrings(Some(c)) => c,
                _ => return Err(()),
            };
            let db = db.lock().unwrap();
            let value = db.get(key);
            Ok(ExecuteReturnValue::Get(value.map(|v| v.to_owned())))
        }
        b"set" => {
            if array.len() != 3 {
                return Err(());
            }
            let key = match &array[1] {
                RESP::BulkStrings(Some(c)) => c,
                _ => return Err(()),
            };
            let value = match &array[2] {
                RESP::BulkStrings(Some(c)) => c,
                _ => return Err(()),
            };
            let mut db = db.lock().unwrap();
            db.set(key.to_owned(), value.to_owned());
            Ok(ExecuteReturnValue::Set)
        }
        b"del" => {
            if array.len() != 2 {
                return Err(());
            }
            let key = match &array[1] {
                RESP::BulkStrings(Some(c)) => c,
                _ => return Err(()),
            };
            let mut db = db.lock().unwrap();
            let i = db.del(key);
            Ok(ExecuteReturnValue::Del(i))
        }
        b"flushall" => {
            if array.len() != 1 {
                return Err(());
            }
            let mut db = db.lock().unwrap();
            db.flushall();
            Ok(ExecuteReturnValue::Flushall)
        }
        _ => Err(()),
    }
}
