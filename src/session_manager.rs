use std::error::Error;
use std::net;
use crate::session::Session;

pub struct SessionManager;

impl SessionManager {
	pub fn new() -> Self {
		SessionManager
	}

	pub fn get_from_addr(&self, _addr: &net::SocketAddr) -> Result<Session, Box<dyn Error>> {
		Ok(Session::new())
	}
}