use std::error::Error;
use async_std::net;

pub struct RakServer{
	addr: net::SocketAddr,
	socket: net::UdpSocket,
}

impl RakServer {
	pub async fn bind(addr: net::SocketAddr) -> Result<Self, Box<dyn Error>> {
		Ok(RakServer {
			addr,
			socket: net::UdpSocket::bind(addr).await?,
		})
	}

	pub fn local_addr(&self) -> net::SocketAddr {
		self.addr
	}

	pub async fn listen(&mut self) {
		println!("Listen from {}", self.local_addr().to_string());

		loop {
			let mut buf = [0u8; 1024];

			loop {
				let (_n, _addr) = match self.socket.recv_from(&mut buf).await {
					Ok((n, _)) if n == 0 => {
						println!("!");
						continue
					},
					Ok((n, addr)) => (n, addr),
					Err(e) => {
						eprintln!("failed to read from socket; err = {:?}", e);
						continue;
					}
				};

				println!("{:?}", buf);
			}
		}
	}
}