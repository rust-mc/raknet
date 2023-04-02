use std::io::Cursor;
use byte_order::NumberReader;
use async_std::net;
use std::error::Error;

pub struct SessionManager;

impl SessionManager {
	pub fn new() -> Self {
		SessionManager
	}

	pub fn get_from_addr(&self, _addr: &net::SocketAddr) -> Result<Session, Box<dyn Error>> {
		Ok(Session::new())
	}
}

#[derive(Debug, Clone)]
pub struct Session;

impl Session {
	pub fn new() -> Self {
		Session
	}

	pub async fn handle_packet(&self, _buffer: NumberReader<Cursor<Vec<u8>>>) {
		todo!()
	}
}