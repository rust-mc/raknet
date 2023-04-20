use std::io::Cursor;

use async_std::net::SocketAddr;
use byte_order::NumberReader;
use rand::Rng;

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
