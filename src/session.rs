use std::collections::HashMap;

use async_std::channel::*;
use async_std::net::SocketAddr;
use async_std::sync::Arc;
use async_std::sync::RwLock;
use log::*;

use crate::internal::{PacketInfo, PacketReader};
use crate::Message;
use crate::protocol::PacketID;

/// Manages and updates sessions.
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<SocketAddr, Session>>>,
    // messages that will be received from the Socket and Stream
    message_receiver: Receiver<Message>,
    // packets that will be sent to the Stream
    packet_to_stream: Sender<PacketInfo>,
    // packets that will be sent to the Socket
    packet_to_socket: Sender<PacketInfo>,
}

impl SessionManager {
    pub fn new(
        message_receiver: Receiver<Message>,
        packet_to_stream: Sender<PacketInfo>,
        packet_to_socket: Sender<PacketInfo>,
    ) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            message_receiver,
            packet_to_stream,
            packet_to_socket,
        }
    }

    /// Receives messages from [`Socket`] and [`Stream`],
    /// processes them and sends (if necessary) packets to [`Socket`] or [`Stream`].
    pub async fn update(&self) -> Result<(), ()> {
        self.update_sessions().await?;
        Ok(())
    }

    /// Updates the status of sessions.
    async fn update_sessions(&self) -> Result<(), ()> {

        Ok(())
    }
}

/// Enumeration showing the session status
#[derive(Debug, Clone)]
pub enum State {
    Unconnected,
    Connected,
    FullyConnected,
    Disconnected,
}

/// Network session
#[derive(Debug, Clone)]
pub struct Session {
    guid: u64,
    addr: SocketAddr,
    mtu: u16,
    state: State,
}

impl Session {
    /// Creates session with following parameters
    pub fn new(guid: u64, addr: SocketAddr, mtu: u16) -> Self {
        Session {
            guid,
            addr,
            mtu,
            state: State::Unconnected,
        }
    }

    /// Processes an incoming unconnected packet.
    pub async fn handle_unconnected(mut buffer: PacketReader) {
        let id = buffer.read_u8().unwrap();
        match id.into() {
            PacketID::UnconnectedPing => todo!(),
            PacketID::OpenConnectionRequest1 => todo!(),
            PacketID::OpenConnectionRequest2 => todo!(),
            PacketID::ConnectionRequest => todo!(),
            PacketID::NewIncomingConnection => todo!(),
            PacketID::Disconnect => todo!(),
            id => error!(
                "Unknown packet (not unconnected); ID: {}\nBody: {:?}",
                id as u8,
                buffer.into_inner().into_inner()
            ),
        }
    }
}
