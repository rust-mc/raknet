use std::collections::HashMap;

use async_std::channel::*;
use async_std::net::SocketAddr;
use async_std::sync::Arc;
use async_std::sync::RwLock;
use log::*;

use crate::internal::{Message, META_INFO, PacketInfo, PacketReader};
use crate::protocol::PacketID;

/// Manages and updates connections.
pub(crate) struct ConnectionHandler {
    connection: Arc<RwLock<HashMap<SocketAddr, Connection>>>,
    // messages that will be received from the Socket and Stream
    message_receiver: Receiver<Message>,
    // packets that will be sent to the Stream
    packet_to_stream: Sender<PacketInfo>,
    // packets that will be sent to the Socket
    packet_to_socket: Sender<PacketInfo>,
}

impl ConnectionHandler {
    pub fn new(
        message_receiver: Receiver<Message>,
        packet_to_stream: Sender<PacketInfo>,
        packet_to_socket: Sender<PacketInfo>,
    ) -> Self {
        Self {
            connection: Arc::new(RwLock::new(HashMap::new())),
            message_receiver,
            packet_to_stream,
            packet_to_socket,
        }
    }

    /// Receives messages from [`RakSocket`] and [`Stream`],
    /// processes them and sends (if necessary) packets to [`RakSocket`] or [`Stream`].
    pub async fn update(&self) {
        while META_INFO.read().await.enabled {
            self.update_connections().await;
        }
    }

    /// Updates the status of connections.
    async fn update_connections(&self) {
        todo!()
    }
}

/// Enumeration showing the connection status
#[derive(Debug, Clone)]
pub enum State {
    Unconnected,
    Connected,
    FullyConnected,
    Disconnected,
}

/// Network connection
#[derive(Debug, Clone)]
pub struct Connection {
    guid: u64,
    addr: SocketAddr,
    mtu: u16,
    state: State,
}

impl Connection {
    /// Creates connection with following parameters
    pub fn new(guid: u64, addr: SocketAddr, mtu: u16) -> Self {
        Connection {
            guid,
            addr,
            mtu,
            state: State::Unconnected,
        }
    }

    /// Processes an incoming unconnected packet.
    pub async fn handle_unconnected(mut buffer: PacketReader) -> Option<PacketReader> {
        let id = buffer.read_u8().unwrap();
        match id.into() {
            PacketID::UnconnectedPing | PacketID::UnconnectedPing2 => todo!(),
            PacketID::OpenConnectionRequest1 => todo!(),
            PacketID::OpenConnectionRequest2 => todo!(),
            PacketID::ConnectionRequest => todo!(),
            PacketID::NewIncomingConnection => todo!(),
            PacketID::Disconnect => todo!(),
            id => {
                debug!("unknown packet (not unconnected); ID: {}", id as u8);
                debug!("body: {:?}", buffer.into_inner().into_inner());
                None
            },
        }
    }
}
