use std::io::Cursor;

use async_std::net::SocketAddr;
use async_std::sync::RwLock;
use byte_order::NumberReader;
use lazy_static::lazy_static;
use rand::Rng;

use crate::connection::Connection;
use crate::Motd;

pub use self::u24_impl::*;

/// Implementation for a u24
/// A u24 in 3 bytes (24 bits) wide number
pub mod u24_impl;

pub type PacketReader = NumberReader<Cursor<Vec<u8>>>;
pub type PacketInfo = (SocketAddr, Vec<u8>);

pub fn generate_guid() -> u64 {
    let mut rng = rand::thread_rng();
    rng.gen::<u64>()
}

pub const PROTOCOL_VERSION: u8 = 11;

pub struct MetaInfo {
    pub enabled: bool,
    pub motd: Motd,
    pub guid: u64,
}

lazy_static! {
    pub static ref META_INFO: RwLock<MetaInfo> = RwLock::new(MetaInfo {
        enabled: false,
        motd: Motd::default(),
        guid: 0,
    });
}

/// This enumeration is necessary for internal communication of [`ConnectionHandler`], [`RakSocket`] and [`Stream`]
pub enum Message {
    /// Sends a packet to the [`ConnectionHandler`] for the connected session
    Packet(SocketAddr, PacketReader),
    /// Sends the [`ConnectionHandler`] a session that has passed the first stage of the raknet handshake
    OpenSession(Connection),
}