use std::error::Error;
use std::io::{Cursor, Read};
use async_std::net;
use byte_order::{ByteOrder, NumberReader};
use log::*;
use crate::protocol::PacketID;
use crate::session::SessionManager;

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
				let cursor = Cursor::new(buffer.to_vec());
				let reader = NumberReader::with_order(ByteOrder::BE, cursor);

				match id {
					0x01 | 0x05 | 0x07 | 0x09 | 0x13 | 0x15 => {
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

	async fn handle_unconnected(&self, mut buffer: NumberReader<Cursor<Vec<u8>>>) {

		let id = buffer.read_u8().unwrap();
		match id.into() {
			PacketID::UnconnectedPing => todo!(),
			PacketID::OpenConnectionRequest1 => todo!(),
			PacketID::OpenConnectionRequest2 => todo!(),
			PacketID::ConnectionRequest => todo!(),
			PacketID::NewIncomingConnection => todo!(),
			PacketID::Disconnect => todo!(),
			id => error!("Unknown packet; ID: {}; Body: {:?}", id as u8, buffer.bytes().collect::<Vec<_>>())
		}
	}
}