use async_std::channel::unbounded;
use async_std::io::Result;
use async_std::net::{SocketAddr, ToSocketAddrs};
use async_std::task::spawn;

use crate::internal::{generate_guid, PacketReader};
use crate::server::socket::RakSocket;
pub use crate::server::stream::*;
use crate::session::{Session, SessionManager};

/// Contains a socket that accepts raw data and forwards it further.
mod socket;

/// Contains the stream necessary for the socket to communicate with the caller.
pub mod stream;

/// Basic information about Minecraft Bedrock Server.
#[derive(Debug, Clone)]
pub struct Motd {
    pub edition: String,
    pub motd_line1: String,
    pub protocol_version: u16,
    pub version_name: String,
    pub max_player_count: u32,
    pub motd_line2: String,
    pub game_mode: String,
    pub game_mode_numeric: u8,
}

/// This enumeration is necessary for internal communication of [`SessionManager`], [`RakSocket`] and [`Stream`]
pub(crate) enum Message {
    /// Sends a packet to the [`SessionManager`] for the connected session
    Packet(SocketAddr, PacketReader),
    /// Sends the [`SessionManager`] a session that has passed the first stage of the raknet handshake
    OpenSession(Session),
}

/// The server runs all system components.
pub struct Server {
    guid: u64,
    motd: Motd,
    addr: SocketAddr,
    stream: Stream,
}

impl Server {
    /// Launches [`SessionManager`] and [`RakSocket`].
    pub async fn new<T: ToSocketAddrs>(addr: T, guid: Option<u64>, motd: Option<Motd>) -> Result<Self> {
        let addr = addr.to_socket_addrs().await?.next().expect("failed to parse socket address");

        // channels...
        let (packet_to_socket, packet_receiver_socket) = unbounded();
        let (packet_to_stream, packet_receiver_stream) = unbounded();
        let (sender_message_socket, message_receiver) = unbounded();
        let sender_message_stream = sender_message_socket.clone();

        let socket = RakSocket::bind(addr, packet_receiver_socket, sender_message_socket).await?;
        let session_manager = SessionManager::new(message_receiver, packet_to_stream, packet_to_socket);

        let server = Server {
            guid: guid.unwrap_or(generate_guid()),
            motd: motd.unwrap_or(Motd::new()),
            addr: socket.local_addr()?, // in order to find out which port was actually bound
            stream: Stream::new(packet_receiver_stream, sender_message_stream),
        };

        spawn(async move {
            loop {
                // exit the loop when the Stream is dropped
                match session_manager.update().await {
                    Ok(_) => continue,
                    Err(_) => break,
                }
            }
        });

        spawn(async move {
            loop {
                // exit the loop when the Stream is dropped
                match socket.listen().await {
                    Ok(_) => continue,
                    Err(_) => break,
                }
            }
        });

        Ok(server)
    }

    /// Returns the local address that this listener is bound to.
    ///
    /// This can be useful, for example, when binding to port 0 to figure out which port was actually bound.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn main()  -> std::io::Result<()> { async_std::task::block_on(async {
    /// #
    /// use raknet::{Server, Stream};
    ///
    /// let server = Server::new("127.0.0.1:19132", None, None).await?;
    /// let addr = server.local_addr();
    ///
    /// # Ok(()) }) }
    /// ```
    pub fn local_addr(&self) -> SocketAddr {
        self.addr
    }

    /// Returns [`Stream`]
    pub fn stream(&self) -> &Stream {
        &self.stream
    }

    /// Returns [`Incoming`]
    pub fn incoming(&mut self) -> Incoming<'_> {
        self.stream.incoming()
    }
}