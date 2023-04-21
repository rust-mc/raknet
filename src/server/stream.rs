use std::io::Cursor;
use std::pin::Pin;
use std::task::{Context, Poll, ready};

use async_std::{io, stream};
use async_std::channel::*;
use async_std::io::ErrorKind;
use async_std::io::Result;
use async_std::net::{SocketAddr, ToSocketAddrs};
use byte_order::NumberReader;

use crate::internal::PacketInfo;
use crate::Message;

/// A stream of incoming packets that are not related to `RakNet`.
/// This stream is infinite, i.e. waiting for the next packet will never result in [`None`].
/// It is created by the [`incoming`] method on [`Server`].
///
/// [`None`]: https://doc.rust-lang.org/std/option/enum.Option.html#variant.None
/// [`incoming`]: crate::Server::incoming
/// [`Server`]: crate::Server
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
/// Returns and accepts packets that are not associated with the `RakNet` protocol (i.e. `GamePacket`).
pub struct Stream {
    // packets received from the SessionManager
    packet_receiver: Receiver<PacketInfo>,
    // packets to send to the server
    message_sender: Sender<Message>,
}

impl Stream {
    pub(crate) fn new(
        packet_receiver: Receiver<PacketInfo>,
        message_sender: Sender<Message>,
    ) -> Self {
        Self {
            packet_receiver,
            message_sender,
        }
    }

    /// Receives data from the socket.
    ///
    /// On success, returns the number of bytes read.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn main()  -> std::io::Result<()> { async_std::task::block_on(async {
    /// #
    /// use raknet::{Server, Stream};
    ///
    /// let server = Server::new("127.0.0.1:19132", None, None).await?;
    /// let stream = server.stream();
    ///
    /// let mut buf = vec![0u8; 1024];
    /// let n = stream.recv(&mut buf).await?;
    /// println!("Received {} bytes", n);
    ///
    /// # Ok(()) }) }
    /// ```
    pub async fn recv(&self, buffer: &mut [u8]) -> Result<usize> {
        let (bytes, _) = self.recv_from(buffer).await?;
        Ok(bytes)
    }

    /// Receives data from the socket.
    ///
    /// On success, returns the number of bytes read and the origin.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn main()  -> std::io::Result<()> { async_std::task::block_on(async {
    /// #
    /// use raknet::{Server, Stream};
    ///
    /// let server = Server::new("127.0.0.1:19132", None, None).await?;
    /// let stream = server.stream();
    ///
    /// let mut buf = vec![0u8; 1024];
    /// let (n, addr) = stream.recv_from(&mut buf).await?;
    /// println!("Received {} bytes from {}", n, addr);
    ///
    /// # Ok(()) }) }
    /// ```
    pub async fn recv_from(&self, buffer: &mut [u8]) -> Result<(usize, SocketAddr)> {
        let (addr, data) = self.packet_receiver.recv().await
            .map_err( |e| { io::Error::new(ErrorKind::Other, e) })?;

        let bytes = data.len();
        buffer.copy_from_slice(&data);
        Ok((bytes, addr))
    }

    /// Sends data on the socket to the given address.
    ///
    /// On success, returns the number of bytes written.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn main()  -> std::io::Result<()> { async_std::task::block_on(async {
    /// #
    /// use raknet::{Server, Stream};
    ///
    /// let server = Server::new("127.0.0.1:19132", None, None).await?;
    /// let stream = server.stream();
    ///
    ///
    /// let n = stream.send_to(b"Hi there!", "127.0.0.1:19133").await?;
    /// println!("Sent {} bytes", n);
    ///
    /// # Ok(()) }) }
    /// ```
    pub async fn send_to<T: ToSocketAddrs>(&self, buffer: &[u8], addr: T) -> Result<()> {
        let addr = match addr.to_socket_addrs().await?.next() {
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "no addresses to send data to",
                ))
            },
            Some(addr) => addr
        };

        let cursor = Cursor::new(buffer.to_vec());
        let reader = NumberReader::new(cursor);
        let message = Message::Packet(addr, reader);

        self.message_sender.send(message).await
            .map_err( |e| { io::Error::new(ErrorKind::Other, e) })?;
        Ok(())
    }

    pub(crate) fn incoming(&mut self) -> Incoming<'_> {
        Incoming { stream: self }
    }
}
