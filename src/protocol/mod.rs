use std::io;

use crate::internal::PacketReader;

/// Unconnected packets (handshake)
pub mod unconnected;

#[allow(dead_code)]
pub(crate) const MAGIC: [u8; 16] = [
    0x00, 0xff, 0xff, 0x00, 0xfe, 0xfe, 0xfe, 0xfe, 0xfd, 0xfd, 0xfd, 0xfd, 0x12, 0x34, 0x56, 0x78,
];

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum PacketID {
    ConnectedPing = 0x00,
    UnconnectedPing = 0x01,
    UnconnectedPing2 = 0x02,
    ConnectedPong = 0x03,
    OpenConnectionRequest1 = 0x05,
    OpenConnectionReply1 = 0x06,
    OpenConnectionRequest2 = 0x07,
    OpenConnectionReply2 = 0x08,
    ConnectionRequest = 0x09,
    ConnectionRequestAccepted = 0x10,
    NewIncomingConnection = 0x13,
    Disconnect = 0x15,
    IncompatibleProtocol = 0x19,
    UnconnectedPong = 0x1c,
    GamePacket = 0xfe,
    NACK = 0xa0,
    ACK = 0xc0,

    Unknown = 0xff,
}

impl From<u8> for PacketID {
    fn from(value: u8) -> Self {
        match value {
            0x00 => Self::ConnectedPing,
            0x01 => Self::UnconnectedPing,
            0x03 => Self::ConnectedPong,
            0x05 => Self::OpenConnectionRequest1,
            0x06 => Self::OpenConnectionReply1,
            0x07 => Self::OpenConnectionRequest2,
            0x08 => Self::OpenConnectionReply2,
            0x09 => Self::ConnectionRequest,
            0x10 => Self::ConnectionRequestAccepted,
            0x13 => Self::NewIncomingConnection,
            0x15 => Self::Disconnect,
            0x19 => Self::IncompatibleProtocol,
            0x1c => Self::UnconnectedPong,
            0xfe => Self::GamePacket,
            0xa0 => Self::NACK,
            0xc0 => Self::ACK,
            _ => Self::Unknown,
        }
    }
}

pub trait ClientPacket: Sized {
    fn id() -> u8;
    fn parse(reader: PacketReader) -> io::Result<Self>;
}

pub trait ServerPacket: Sized {
    fn id() -> u8;
    fn compose(&self) -> io::Result<PacketReader>;
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use std::str::FromStr;

    use byte_order::{ByteOrder, NumberReader};

    use rakmacro::{ClientPacket, ServerPacket};

    use crate::internal::u24;
    use crate::protocol::{ClientPacket, ServerPacket};

    // The name is not related to the actual package.
    // This is necessary for the `id()` method to work correctly
    #[derive(ServerPacket)]
    #[allow(dead_code)]
    struct ConnectedPing {
        bool: bool,
        ubyte: u8,
        byte: i8,
        ushort: u16,
        short: i16,
        uint: u32,
        int: i32,
        ulong: u64,
        long: i64,
        string: String,
        magic: [u8; 16],
        addr: async_std::net::SocketAddr,
        u24_le: u24,
    }

    // The name is not related to the actual package.
    // This is necessary for the `id()` method to work correctly
    #[derive(ClientPacket)]
    #[allow(dead_code)]
    struct ConnectedPong {
        bool: bool,
        ubyte: u8,
        byte: i8,
        ushort: u16,
        short: i16,
        uint: u32,
        int: i32,
        ulong: u64,
        long: i64,
        string: String,
        magic: [u8; 16],
        addr: async_std::net::SocketAddr,
        u24_le: u24,
    }

    #[test]
    fn server_packet_id() {
        assert_eq!(ConnectedPing::id(), 0x00);
    }

    #[test]
    fn server_packet_new() {
        let ping = ConnectedPing::new(
            true,
            1u8,
            2i8,
            3u16,
            4i16,
            5u32,
            6i32,
            7u64,
            8i64,
            String::from("Test string"),
            async_std::net::SocketAddr::from_str("127.0.0.1:19132").unwrap(),
            u24::from(9u32),
        );
        assert_eq!(ping.bool, true);
        assert_eq!(ping.ubyte, 1u8);
        assert_eq!(ping.byte, 2i8);
        assert_eq!(ping.ushort, 3u16);
        assert_eq!(ping.short, 4i16);
        assert_eq!(ping.uint, 5u32);
        assert_eq!(ping.int, 6i32);
        assert_eq!(ping.ulong, 7u64);
        assert_eq!(ping.long, 8i64);
        assert_eq!(ping.string, String::from("Test string"));
        assert_eq!(ping.addr.to_string(), String::from("127.0.0.1:19132"));
        assert_eq!(ping.u24_le, 9u32);
    }

    #[test]
    fn server_packet_compose() {
        let ping = ConnectedPing::new(
            true,
            1u8,
            2i8,
            3u16,
            4i16,
            5u32,
            6i32,
            7u64,
            8i64,
            String::from("Test string"),
            async_std::net::SocketAddr::from_str("127.0.0.1:19132").unwrap(),
            u24::from(9u32),
        );
        let packet = ping.compose().unwrap();
        let packet = packet.into_inner().into_inner();
        let eq_packet = Vec::<u8>::from([
            0x00, // id
            0x01, // bool
            0x01, // ubyte
            0x02, // byte
            0x00, 0x03, // ushort
            0x00, 0x04, // short
            0x00, 0x00, 0x00, 0x05, //uint
            0x00, 0x00, 0x00, 0x06, // int
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07, //ulong
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, //long
            0x00, 0x0b, /* len */
            0x54, 0x65, 0x73, 0x74, 0x20, 0x73, 0x74, 0x72, 0x69, 0x6e, 0x67, // string
            0x00, 0xff, 0xff, 0x00, 0xfe, 0xfe, 0xfe, 0xfe, 0xfd, 0xfd, 0xfd, 0xfd, 0x12, 0x34,
            0x56, 0x78, // magic
            0x04, 0x7f, 0x00, 0x00, 0x01, 0x4a, 0xbc, // addr (ip version, ip, port)
            0x09, 0x00, 0x00, // u24_le
        ]);

        assert_eq!(packet, eq_packet);
    }

    #[test]
    fn client_packet_id() {
        assert_eq!(ConnectedPong::id(), 0x03)
    }

    #[test]
    fn client_packet_parse() {
        let raw_packet = Vec::<u8>::from([
            // The packet from the client is stripped of the ID field after processing
            0x01, // bool
            0x01, // ubyte
            0x02, // byte
            0x00, 0x03, // ushort
            0x00, 0x04, // short
            0x00, 0x00, 0x00, 0x05, // uint
            0x00, 0x00, 0x00, 0x06, // int
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07, //ulong
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, //long
            0x00, 0x0b, /* len */
            0x54, 0x65, 0x73, 0x74, 0x20, 0x73, 0x74, 0x72, 0x69, 0x6e, 0x67, // string
            0x00, 0xff, 0xff, 0x00, 0xfe, 0xfe, 0xfe, 0xfe, 0xfd, 0xfd, 0xfd, 0xfd, 0x12, 0x34,
            0x56, 0x78, // magic
            0x04, 0x7f, 0x00, 0x00, 0x01, 0x4a, 0xbc, // addr (ip version, ip, port)
            0x09, 0x00, 0x00, // u24_le
        ]);

        let cursor = Cursor::new(raw_packet);
        let reader = NumberReader::with_order(ByteOrder::BE, cursor);

        let pong = ConnectedPong::parse(reader).unwrap();

        assert_eq!(pong.bool, true);
        assert_eq!(pong.ubyte, 1u8);
        assert_eq!(pong.byte, 2i8);
        assert_eq!(pong.ushort, 3u16);
        assert_eq!(pong.short, 4i16);
        assert_eq!(pong.uint, 5u32);
        assert_eq!(pong.int, 6i32);
        assert_eq!(pong.ulong, 7u64);
        assert_eq!(pong.long, 8i64);
        assert_eq!(pong.string, String::from("Test string"));
        assert_eq!(pong.addr.to_string(), String::from("127.0.0.1:19132"));
        assert_eq!(pong.u24_le, 9u32);
    }
}
