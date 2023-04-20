use std::io::Cursor;

use async_std::channel::*;
use async_std::io::Result;
use async_std::net::ToSocketAddrs;
use async_std::net::UdpSocket;
use byte_order::{ByteOrder, NumberReader};
use log::*;

use crate::internal::PacketInfo;
use crate::Message;
use crate::session::Session;

pub(crate) struct RakSocket {
    socket: UdpSocket,
    // Packets to write to the socket
    packet_receiver: Receiver<PacketInfo>,
    // Messages that will be sent to the server
    message_sender: Sender<Message>,
}

impl RakSocket {
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

    pub async fn listen(&self) -> Result<()> {
        debug!(
            "Listen from {}",
            self.socket.local_addr().unwrap().to_string()
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
}
