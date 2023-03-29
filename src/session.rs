use std::io::Cursor;
use byte_order::NumberReader;

pub struct Session;

impl Session {
	pub fn new() -> Self {
		Session
	}

	pub async fn handle_packet(&self, _buffer: NumberReader<Cursor<[u8; 2048]>>) {
		todo!()
	}
}