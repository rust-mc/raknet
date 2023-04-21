use std::io::Cursor;

use async_std::channel::*;
use async_std::io::Result;
use async_std::net::ToSocketAddrs;
use async_std::net::UdpSocket;
use async_std::net::SocketAddr;
use byte_order::{ByteOrder, NumberReader};
use log::*;

use crate::internal::PacketInfo;
use crate::Message;
use crate::session::Session;

/// A socket that communicates over the `RakNet` protocol.
pub(crate) struct RakSocket {
    socket: UdpSocket,
    // packets to write to the socket
    packet_receiver: Receiver<PacketInfo>,
    // messages that will be sent to the SessionManager
    message_sender: Sender<Message>,
}

impl RakSocket {
    /// Creates a socket with the specified address.
    pub async fn bind<T: ToSocketAddrs>(
        addr: T,
        packet_receiver: Receiver<PacketInfo>,
        message_sender: Sender<Message>,
    ) -> Result<Self> {
        Ok(Self {
            socket: UdpSocket::bind(addr).await?,
            packet_receiver,
            message_sender,
        })
    }

    /// Listens to UDP socket and transmits SessionManager packets.
    ///
    /// This function is in a thread loop created by the Server.
    ///
    /// Returns an error when the program is terminated.
    pub async fn listen(&self) -> Result<()> {
        debug!(
            "Listen from {}",
            self.local_addr()?
        );
        let mut buffer = vec![0u8; 4096];
        let (bytes, _addr) = match self.socket.recv_from(&mut buffer).await {
            Ok((n, _)) if n == 0 => return Ok(()),
            Ok((n, addr)) => (n, addr),
            Err(e) => {
                debug!("failed to read from socket; err = {:?}", e);
                return Ok(());
            }
        };

        buffer.truncate(bytes);
        let id = buffer.first().unwrap().clone();
        let cursor = Cursor::new(buffer);
        let reader = NumberReader::with_order(ByteOrder::BE, cursor);

        match id {
            0x01 | 0x05 | 0x07 | 0x09 | 0x13 | 0x15 => {
                Session::handle_unconnected(reader).await;
            }
            _ => {
                todo!() // session packet
            }
        }

        Ok(())
    }

    /// Returns the actual socket address.
    pub fn local_addr(&self) -> Result<SocketAddr> {
        self.socket.local_addr()
    }
}
