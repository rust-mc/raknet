use std::io::Cursor;
use std::pin::Pin;
use std::task::{Context, Poll, ready};

use async_std::{io, stream};
use async_std::channel::*;
use async_std::io::ErrorKind;
use async_std::io::Result;
use async_std::net::SocketAddr;
use byte_order::NumberReader;

use crate::internal::PacketInfo;
use crate::Message;

pub struct Incoming<'a> {
    stream: &'a mut Stream,
}

impl stream::Stream for Incoming<'_> {
    type Item = PacketInfo;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let res = ready!(Pin::new(&mut self.stream.packet_receiver).poll_next(cx));
        Poll::Ready(res)
    }
}

/// Communication between the [`Server`] and the caller.
/// Returns and accepts packets that are not associated with the `RakNet` protocol. (i.e. `GamePacket`)
pub struct Stream {
    // Packets received from the server
    packet_receiver: Receiver<(SocketAddr, Vec<u8>)>,
    // Packets to send to the server
    message_sender: Sender<Message>,
}

impl Stream {
    pub(crate) fn new(
        packet_receiver: Receiver<(SocketAddr, Vec<u8>)>,
        message_sender: Sender<Message>,
    ) -> Self {
        Self {
            packet_receiver,
            message_sender,
        }
    }

    pub async fn recv(&self, buffer: &mut [u8]) -> Result<usize> {
        let (bytes, _) = self.recv_from(buffer).await?;
        Ok(bytes)
    }

    pub async fn recv_from(&self, buffer: &mut [u8]) -> Result<(usize, SocketAddr)> {
        let (addr, data) = match self.packet_receiver.recv().await {
            Ok((addr, data)) => (addr, data),
            Err(e) => return Err(io::Error::new(ErrorKind::Other, e)),
        };

        let bytes = data.len();
        buffer.copy_from_slice(&data);
        Ok((bytes, addr))
    }

    pub async fn send_to(&self, buffer: &[u8], addr: SocketAddr) -> Result<()> {
        let cursor = Cursor::new(buffer.to_vec());
        let reader = NumberReader::new(cursor);
        let message = Message::Packet(addr, reader);
        match self.message_sender.send(message).await {
            Ok(_) => {}
            Err(e) => return Err(io::Error::new(ErrorKind::Other, e)),
        };
        Ok(())
    }

    pub fn incoming(&mut self) -> Incoming<'_> {
        Incoming { stream: self }
    }
}
