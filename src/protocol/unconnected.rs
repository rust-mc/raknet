use rakmacro::{ClientPacket, ServerPacket};

#[derive(Debug, Clone)]
pub struct MTU(u16);

#[derive(Debug, Clone, ClientPacket)]
pub struct UnconnectedPing {
    time: i64,
    magic: [u8; 16],
    client_guid: i64,
}

#[derive(Debug, Clone, ServerPacket)]
pub struct UnconnectedPong {
    time: i64,
    server_guid: i64,
    magic: [u8; 16],
    motd: String,
}

#[derive(Debug, Clone, ClientPacket)]
pub struct OpenConnectionRequest1 {
    magic: [u8; 16],
    protocol_version: u8,
    mtu: MTU,
}

#[derive(Debug, Clone, ServerPacket)]
pub struct OpenConnectionReply1 {
    magic: [u8; 16],
    server_guid: u64,
    security: bool,
    mtu: u16,
}

#[derive(Debug, Clone, ClientPacket)]
pub struct OpenConnectionRequest2 {
    magic: [u8; 16],
    server_address: async_std::net::SocketAddr,
    mtu: u16,
    client_guid: u64,
}

#[derive(Debug, Clone, ServerPacket)]
pub struct OpenConnectionReply2 {
    magic: [u8; 16],
    server_guid: u64,
    client_address: async_std::net::SocketAddr,
    mtu: u16,
    encryption: bool,
}

#[derive(Debug, Clone, ServerPacket)]
pub struct IncompatibleProtocol {
    protocol: u8,
    magic: [u8; 16],
    server_guid: u64,
}
