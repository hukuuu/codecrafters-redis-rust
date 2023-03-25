use bytes::{Bytes, BytesMut};
use tokio::net::TcpStream;

pub enum Frame {
	Simple(String),
	Error(String),
	Integer(u64),
	Bulk(Bytes),
	Null,
	Array(Vec<Frame>)
}


struct Connection {
	stream: TcpStream,
	buffer: BytesMut
}

impl Connection {
	pub fn new(stream: TcpStream) -> Self {
		Connection { stream: stream, buffer: BytesMut::with_capacity(4096) }
	}

	pub async fn read_frame(&mut self) -> Result<Option<Frame>, ()> {
		unimplemented!()
	}

	pub async fn write_frame(&mut self, frame: Frame) -> Result<(), ()> {
		unimplemented!()
	}

}