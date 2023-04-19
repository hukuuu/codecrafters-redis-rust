use core::panic;
use std::io::Cursor;

// use anyhow::Ok;
use bytes::{Buf, Bytes, BytesMut};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

#[derive(Debug)]
pub enum Error {
    Incomplete,
    Other(String),
}

#[derive(Clone, Debug)]
pub enum Frame {
    Simple(String),
    Error(String),
    Integer(u64),
    Bulk(Bytes),
    // Null,
    Array(Vec<Frame>),
}

impl Frame {
    fn check(src: &mut Cursor<&[u8]>) -> Result<(), Error> {
        let char = get_u8(src)?;
        match char {
            b'+' => {
                get_line(src)?;
                Ok(())
            }
            b'-' => {
                get_line(src)?;
                Ok(())
            }
            b':' => {
                let _ = get_decimal(src);
                Ok(())
            }
            b'$' => {
                let len: usize = get_decimal(src)?.try_into().unwrap();
                skip(src, len + 2)
            }
            b'*' => {
                let len = get_decimal(src)?;
                for _ in 0..len {
                    Frame::check(src)?;
                }
                Ok(())
            }
            _ => Err(Error::Other("unknown op".to_string())),
        }
    }

    fn parse(src: &mut Cursor<&[u8]>) -> Result<Frame, Error> {
        match get_u8(src)? {
            b'+' => {
                let line = get_line(src)?;
                Ok(Frame::Simple(String::from_utf8(line.into()).unwrap()))
            }
            b'-' => {
                let line = get_line(src)?;
                Ok(Frame::Error(String::from_utf8(line.into()).unwrap()))
            }
            b':' => {
                let number = get_decimal(src)?;
                Ok(Frame::Integer(number))
            }
            b'$' => {
                //hack - just get the len ( to get rid of it, and then read line )
                get_decimal(src)?;
                let line = get_line(src)?;
                Ok(Frame::Bulk(Bytes::copy_from_slice(line)))
            }
            b'*' => {
                let len = get_decimal(src)?.try_into().unwrap();
                let mut res = Vec::with_capacity(len);
                for _ in 0..len {
                    res.push(Frame::parse(src)?);
                }
                Ok(Frame::Array(res))
            }
            other => {
                println!(
                    "unknown: {:?} {:?}",
                    char::from_u32(other as u32).unwrap(),
                    src.position()
                );
                panic!("ebasi");
            }
        }
    }
}

fn skip(src: &mut Cursor<&[u8]>, n: usize) -> Result<(), Error> {
    if dbg!(src.remaining()) < n {
        return Err(Error::Incomplete);
    }
    src.advance(n);
    Ok(())
}

fn get_u8(src: &mut Cursor<&[u8]>) -> Result<u8, Error> {
    if !src.has_remaining() {
        return Err(Error::Incomplete);
    }
    Ok(src.get_u8())
}

// fn peek_u8(src: &mut Cursor<&[u8]>) -> Result<u8, Error> {
//     if !src.has_remaining() {
//         return Err(Error::Incomplete);
//     }
//     let res = src.get_u8();
//     src.set_position(src.position() - 1);
//     Ok(res)
// }

fn get_line<'a>(src: &mut Cursor<&'a [u8]>) -> Result<&'a [u8], Error> {
    let start = src.position() as usize;
    let end = src.get_ref().len() - 1;
    for i in start..end {
        if src.get_ref()[i] == b'\r' && src.get_ref()[i + 1] == b'\n' {
            src.set_position((i + 2) as u64);
            return Ok(&src.get_ref()[start..i]);
        }
    }
    Err(Error::Incomplete)
}

fn get_decimal(src: &mut Cursor<&[u8]>) -> Result<u64, Error> {
    let s = std::str::from_utf8(get_line(src)?).unwrap();
    return Ok(s.parse().unwrap());
}

pub struct Connection {
    stream: TcpStream,
    buffer: BytesMut,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Connection {
            stream: stream,
            buffer: BytesMut::with_capacity(4096),
        }
    }

    pub async fn read_frame(&mut self) -> Result<Option<Frame>, Error> {
        loop {
            if let Some(frame) = self.parse_frame()? {
                return Ok(Some(frame));
            }
            if 0 == self.stream.read_buf(&mut self.buffer).await.unwrap() {
                // The remote closed the connection. For this to be
                // a clean shutdown, there should be no data in the
                // read buffer. If there is, this means that the
                // peer closed the socket while sending a frame.
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    return Err(Error::Other("connection reset by peer".to_string()));
                }
            }
        }
    }

    fn parse_frame(&mut self) -> Result<Option<Frame>, Error> {
        let mut buf = Cursor::new(&self.buffer[..]);

        match Frame::check(&mut buf) {
            Ok(_) => {
                // Get the byte length of the frame
                let len = buf.position() as usize;

                // Reset the internal cursor for the
                // call to `parse`.
                buf.set_position(0);

                // Parse the frame
                let frame = Frame::parse(&mut buf)?;

                // Discard the frame from the buffer
                self.buffer.advance(len);

                // Return the frame to the caller.
                Ok(Some(frame))
            }
            // Not enough data has been buffered
            Err(Error::Incomplete) => Ok(None),
            // An error was encountered
            Err(e) => Err(e.into()),
        }
    }

    pub async fn write_frame(&mut self, frame: Frame) -> Result<(), ()> {
        match frame {
            Frame::Bulk(bytes) => {
                self.stream.write_all(&bytes).await.unwrap();
            }
            _ => panic!("did not expect anything except bytes"),
        }
        Ok(())
    }
}
