use async_std::channel::unbounded;
use async_std::io::Result;
use async_std::net::{SocketAddr, ToSocketAddrs};
use async_std::sync::Arc;
use async_std::task;
use async_std::task::spawn;

use crate::connection::ConnectionHandler;
use crate::internal::{generate_guid, META_INFO};
use crate::server::socket::RakSocket;
pub use crate::server::stream::*;

/// Contains a socket that accepts raw data and forwards it further.
mod socket;

/// Contains the stream necessary for the socket to communicate with the caller.
pub mod stream;

/// Basic information about Minecraft Bedrock Server.
#[derive(Debug, Clone, Default)]
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

/// Prepares and launches [`RakSocket`], [`ConnectionHandler`] and returns [`Stream`].
pub struct RakListener {
    stream: Stream,
    socket: Arc<RakSocket>,
    connection_handler: Arc<ConnectionHandler>,
}

impl RakListener {
    /// Prepares channels, [`RakSocket`] and [`ConnectionHandler`], for future launch.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn main()  -> std::io::Result<()> { async_std::task::block_on(async {
    /// use raknet::RakListener;
    ///
    /// let listener = RakListener::bind("127.0.0.1:19132").await?;
    /// # Ok(()) }) }
    /// ```
    pub async fn bind<T: ToSocketAddrs>(addr: T) -> Result<Self> {
        let addr = addr.to_socket_addrs().await?.next().expect("failed to parse socket address");
        let mut meta_info = META_INFO.write().await;
        meta_info.guid = generate_guid();
        meta_info.motd = Motd::default();
        meta_info.enabled = true;

        // channels...
        let (packet_to_socket, packet_receiver_socket) = unbounded();
        let (packet_to_stream, packet_receiver_stream) = unbounded();
        let (sender_message_socket, message_receiver) = unbounded();
        let sender_message_stream = sender_message_socket.clone();

        let socket = RakSocket::bind(addr, packet_receiver_socket, sender_message_socket).await?;
        let session_manager = ConnectionHandler::new(message_receiver, packet_to_stream, packet_to_socket);

        let server = RakListener {
            stream: Stream::new(packet_receiver_stream, sender_message_stream),
            socket: Arc::new(socket),
            connection_handler: Arc::new(session_manager),
        };

        Ok(server)
    }

    /// Starts listening to the socket and processing packets and connections
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn main()  -> std::io::Result<()> { async_std::task::block_on(async {
    /// use raknet::RakListener;
    ///
    /// let listener = RakListener::bind("127.0.0.1:19132").await?;
    /// listener.start();
    /// # Ok(()) }) }
    /// ```
    pub fn start(&self) {
        let socket_s = self.socket.clone();
        let connection_handler_s = self.connection_handler.clone();
        spawn(async move {
            connection_handler_s.update().await;
        });

        spawn(async move {
            socket_s.listen().await;
        });
    }

    /// Returns the local address that this listener is bound to.
    ///
    /// This can be useful, for example, when binding to port 0 to figure out which port was actually bound.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn main()  -> std::io::Result<()> { async_std::task::block_on(async {
    /// use raknet::RakListener;
    ///
    /// let listener = RakListener::bind("127.0.0.1:19132").await?;
    ///
    /// println!("Listen from {}", listener.local_addr()?);
    /// # Ok(()) }) }
    /// ```
    pub fn local_addr(&self) -> Result<SocketAddr> {
        self.socket.local_addr()
    }

    /// Returns [`Stream`]
    pub fn stream(&self) -> &Stream {
        &self.stream
    }

    /// Returns [`Incoming`]
    pub fn incoming(&mut self) -> Incoming<'_> {
        self.stream.incoming()
    }

    /// Modify MOTD string
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn main()  -> std::io::Result<()> { async_std::task::block_on(async {
    /// use raknet::{RakListener, Motd};
    ///
    /// let mut listener = RakListener::bind("127.0.0.1:19132").await?;
    ///
    /// let mut motd = Motd::default();
    /// motd.motd_line1 = "Some Minecraft Bedrock Server".to_string();
    ///
    /// listener.motd(motd);
    /// # Ok(()) }) }
    /// ```
    pub fn motd(&mut self, motd: Motd) {
        let mut meta_info = task::block_on(META_INFO.write());
        meta_info.motd = motd;
    }
}

impl Drop for RakListener {
    fn drop(&mut self) {
        let mut meta_info = task::block_on(META_INFO.write());
        meta_info.enabled = false;
    }
}