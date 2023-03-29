use std::error::Error;
use std::io::{Cursor, Read};
use async_std::net;
use byte_order::{ByteOrder, NumberReader};
use log::*;
use crate::session_manager::SessionManager;

pub struct RakServer{
	addr: net::SocketAddr,
	socket: net::UdpSocket,
	session_manager: SessionManager,
}

impl RakServer {
	pub async fn bind(addr: net::SocketAddr) -> Result<Self, Box<dyn Error>> {
		Ok(RakServer {
			addr,
			socket: net::UdpSocket::bind(addr).await?,
			session_manager: SessionManager::new(),
		})
	}

	pub fn local_addr(&self) -> net::SocketAddr {
		self.addr
	}

	pub async fn listen(&mut self) {
		info!("Listen from {}", self.local_addr().to_string());

		loop {
			let mut buffer = [0u8; 2 * 1024];

			loop {
				let (_, addr) = match self.socket.recv_from(&mut buffer).await {
					Ok((n, _)) if n == 0 => continue,
					Ok((n, addr)) => (n, addr),
					Err(e) => {
						error!("failed to read from socket; err = {:?}", e);
						continue;
					}
				};

				let id = buffer[0];
				let cursor = Cursor::new(buffer);
				let reader = NumberReader::with_order(ByteOrder::BE, cursor);

				match id {
					0x01 | 0xc1 | 0x05..=0x08 => {
						self.handle_unconnected(reader).await;
					},
					_ => {
						let session = match self.session_manager.get_from_addr(&addr) {
							Ok(session) => session,
							Err(_) => {
								error!("non-created session [{}] sent a packet: ID {:?}",
								          addr.to_string(),
								          reader.bytes().collect::<Vec<_>>());
								continue
							}
						};
						session.handle_packet(reader).await;
					}
				}
			}
		}
	}

	async fn handle_unconnected(&self, _buffer: NumberReader<Cursor<[u8; 2048]>>) {
		todo!();
	}
}