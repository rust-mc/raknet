use async_std::channel::unbounded;
use async_std::io::Result;
use async_std::net::SocketAddr;
use async_std::task::spawn;

use crate::internal::{generate_guid, PacketReader};
use crate::server::socket::RakSocket;
pub use crate::server::stream::*;
use crate::session::{Session, SessionManager};

mod socket;
pub mod stream;

/// Basic information about Minecraft Bedrock Server.
#[derive(Debug, Clone)]
pub struct Motd;

impl Motd {
    pub fn new() -> Self {
        Self
    }
}

/// This enum is necessary for [`Server`], [`RakSocket`] and [`Stream`] communication
pub enum Message {
    /// Sends a packet to the [`Server`] for the connected session
    Packet(SocketAddr, PacketReader),
    /// Sends the [`Server`] a session that has passed the first stage of the raknet handshake
    OpenSession(Session),
}

pub struct Server {
    guid: u64,
    motd: Motd,
    addr: SocketAddr,
    stream: Stream,
}

impl Server {
    pub async fn new(addr: SocketAddr, guid: Option<u64>, motd: Option<Motd>) -> Result<Self> {
        let (packet_to_socket, packet_receiver_socket) = unbounded();
        let (packet_to_stream, packet_receiver_stream) = unbounded();
        let (sender_message_socket, message_receiver) = unbounded();
        let sender_message_stream = sender_message_socket.clone();

        let socket = RakSocket::bind(addr.clone(), packet_receiver_socket, sender_message_socket).await?;
        let session_manager = SessionManager::new(message_receiver, packet_to_stream, packet_to_socket);

        let server = Server {
            guid: guid.unwrap_or(generate_guid()),
            motd: motd.unwrap_or(Motd::new()),
            addr,
            stream: Stream::new(packet_receiver_stream, sender_message_stream),
        };

        spawn(async move {
            loop {
                match session_manager.update().await {
                    Ok(_) => continue,
                    Err(_) => break,
                }
            }
        });

        spawn(async move {
            loop {
                match socket.listen().await {
                    Ok(_) => continue,
                    Err(_) => break,
                }
            }
        });

        Ok(server)
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.addr
    }

    pub fn stream(&self) -> &Stream {
        &self.stream
    }

    pub fn incoming(&mut self) -> Incoming<'_> {
        self.stream.incoming()
    }
}
