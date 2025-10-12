//! TGP (Toroidal Grid Protocol) packet definitions
//!
//! Shared packet format for P2P communication between browsers (WebRTC),
//! nodes (WebSocket/WebTransport), and Citadel DHT routing.

use serde::{Deserialize, Serialize};

/// TGP packet header (21 bytes fixed size)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TgpPacketHeader {
    /// Protocol version (currently 1)
    pub version: u8,

    /// Packet type (see `PacketType` enum)
    pub packet_type: u8,

    /// Source hexagonal coordinate (64-bit toroidal mesh address)
    pub source_hex: u64,

    /// Destination hexagonal coordinate (64-bit toroidal mesh address)
    pub dest_hex: u64,

    /// Time-to-live (decremented on each hop)
    pub ttl: u8,

    /// Payload length in bytes (max 65535)
    pub payload_length: u16,
}

impl TgpPacketHeader {
    /// Size of the TGP header in bytes
    pub const SIZE: usize = 21; // version(1) + type(1) + source(8) + dest(8) + ttl(1) + length(2)

    /// Create a new TGP packet header
    pub fn new(packet_type: u8, source_hex: u64, dest_hex: u64, payload_length: u16) -> Self {
        Self {
            version: 1,
            packet_type,
            source_hex,
            dest_hex,
            ttl: 64,
            payload_length,
        }
    }

    /// Serialize header to bytes (big-endian)
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(Self::SIZE);
        bytes.push(self.version);
        bytes.push(self.packet_type);
        bytes.extend_from_slice(&self.source_hex.to_be_bytes());
        bytes.extend_from_slice(&self.dest_hex.to_be_bytes());
        bytes.push(self.ttl);
        bytes.extend_from_slice(&self.payload_length.to_be_bytes());
        bytes
    }

    /// Deserialize header from bytes
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < Self::SIZE {
            return None;
        }

        let version = bytes[0];
        let packet_type = bytes[1];
        let source_hex = u64::from_be_bytes(bytes[2..10].try_into().ok()?);
        let dest_hex = u64::from_be_bytes(bytes[10..18].try_into().ok()?);
        let ttl = bytes[18];
        let payload_length = u16::from_be_bytes(bytes[19..21].try_into().ok()?);

        Some(Self {
            version,
            packet_type,
            source_hex,
            dest_hex,
            ttl,
            payload_length,
        })
    }
}

/// TGP packet types
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PacketType {
    // UBTS/Blockchain packets
    UbtsBlock = 0x01,
    WantList = 0x02,
    DeleteWantList = 0x03,

    // Peer discovery packets
    PeerAnnounce = 0x04,
    PeerRequest = 0x05,

    // DHT packets (Citadel DHT integration)
    DhtGet = 0x06,
    DhtPut = 0x07,
    DhtResponse = 0x08,
}

impl PacketType {
    /// Convert u8 to PacketType
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(PacketType::UbtsBlock),
            0x02 => Some(PacketType::WantList),
            0x03 => Some(PacketType::DeleteWantList),
            0x04 => Some(PacketType::PeerAnnounce),
            0x05 => Some(PacketType::PeerRequest),
            0x06 => Some(PacketType::DhtGet),
            0x07 => Some(PacketType::DhtPut),
            0x08 => Some(PacketType::DhtResponse),
            _ => None,
        }
    }

    /// Convert PacketType to u8
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }
}

/// DHT GET request payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DhtGetRequest {
    /// 32-byte Blake3 hash key
    pub key: [u8; 32],
}

/// DHT PUT request payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DhtPutRequest {
    /// 32-byte Blake3 hash key
    pub key: [u8; 32],

    /// Value to store
    pub value: Vec<u8>,
}

/// DHT RESPONSE payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DhtResponse {
    /// 32-byte Blake3 hash key
    pub key: [u8; 32],

    /// Value if found, None if not found
    pub value: Option<Vec<u8>>,
}

/// Create a complete TGP packet (header + payload)
pub fn create_packet(packet_type: u8, source_hex: u64, dest_hex: u64, payload: &[u8]) -> Vec<u8> {
    let header = TgpPacketHeader::new(packet_type, source_hex, dest_hex, payload.len() as u16);
    let mut packet = header.to_bytes();
    packet.extend_from_slice(payload);
    packet
}

/// Parse a TGP packet and return header + payload
pub fn parse_packet(packet: &[u8]) -> Option<(TgpPacketHeader, &[u8])> {
    let header = TgpPacketHeader::from_bytes(packet)?;
    if packet.len() < TgpPacketHeader::SIZE + header.payload_length as usize {
        return None;
    }
    let payload = &packet[TgpPacketHeader::SIZE..TgpPacketHeader::SIZE + header.payload_length as usize];
    Some((header, payload))
}

/// Calculate hex coordinate for a peer ID using Blake3 hash
pub fn peer_id_to_hex(peer_id: &str) -> u64 {
    use blake3::Hasher;
    let mut hasher = Hasher::new();
    hasher.update(peer_id.as_bytes());
    let hash = hasher.finalize();

    // Use first 8 bytes of Blake3 hash as u64 coordinate
    u64::from_be_bytes(hash.as_bytes()[0..8].try_into().unwrap())
}

/// Calculate hex distance between two coordinates (toroidal wrapping)
pub fn hex_distance(a: u64, b: u64) -> u64 {
    a.wrapping_sub(b).min(b.wrapping_sub(a))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_serialization() {
        let header = TgpPacketHeader::new(PacketType::UbtsBlock.as_u8(), 0x1234, 0x5678, 100);
        let bytes = header.to_bytes();

        assert_eq!(bytes.len(), TgpPacketHeader::SIZE);

        let parsed = TgpPacketHeader::from_bytes(&bytes).unwrap();
        assert_eq!(parsed.version, 1);
        assert_eq!(parsed.packet_type, PacketType::UbtsBlock.as_u8());
        assert_eq!(parsed.source_hex, 0x1234);
        assert_eq!(parsed.dest_hex, 0x5678);
        assert_eq!(parsed.ttl, 64);
        assert_eq!(parsed.payload_length, 100);
    }

    #[test]
    fn test_packet_creation() {
        let payload = b"Hello, TGP!";
        let packet = create_packet(PacketType::DhtGet.as_u8(), 0x1000, 0x2000, payload);

        assert_eq!(packet.len(), TgpPacketHeader::SIZE + payload.len());

        let (header, parsed_payload) = parse_packet(&packet).unwrap();
        assert_eq!(header.packet_type, PacketType::DhtGet.as_u8());
        assert_eq!(header.source_hex, 0x1000);
        assert_eq!(header.dest_hex, 0x2000);
        assert_eq!(parsed_payload, payload);
    }

    #[test]
    fn test_peer_id_to_hex() {
        let hex1 = peer_id_to_hex("peer-1");
        let hex2 = peer_id_to_hex("peer-2");

        // Different peer IDs should produce different coordinates
        assert_ne!(hex1, hex2);

        // Same peer ID should always produce same coordinate
        assert_eq!(peer_id_to_hex("peer-1"), hex1);
    }

    #[test]
    fn test_hex_distance() {
        // Distance is symmetric
        assert_eq!(hex_distance(100, 200), hex_distance(200, 100));

        // Distance to self is 0
        assert_eq!(hex_distance(100, 100), 0);

        // Toroidal wrapping
        let dist1 = hex_distance(u64::MAX - 10, 10);
        assert_eq!(dist1, 21); // Wraps around
    }

    #[test]
    fn test_dht_get_request() {
        let key = [42u8; 32];
        let request = DhtGetRequest { key };

        let json = serde_json::to_vec(&request).unwrap();
        let packet = create_packet(PacketType::DhtGet.as_u8(), 0x1000, 0x2000, &json);

        let (header, payload) = parse_packet(&packet).unwrap();
        assert_eq!(header.packet_type, PacketType::DhtGet.as_u8());

        let parsed_request: DhtGetRequest = serde_json::from_slice(payload).unwrap();
        assert_eq!(parsed_request.key, key);
    }

    #[test]
    fn test_dht_put_request() {
        let key = [42u8; 32];
        let value = b"test value".to_vec();
        let request = DhtPutRequest { key, value: value.clone() };

        let json = serde_json::to_vec(&request).unwrap();
        let packet = create_packet(PacketType::DhtPut.as_u8(), 0x1000, 0x2000, &json);

        let (header, payload) = parse_packet(&packet).unwrap();
        assert_eq!(header.packet_type, PacketType::DhtPut.as_u8());

        let parsed_request: DhtPutRequest = serde_json::from_slice(payload).unwrap();
        assert_eq!(parsed_request.key, key);
        assert_eq!(parsed_request.value, value);
    }

    #[test]
    fn test_dht_response() {
        let key = [42u8; 32];
        let value = Some(b"response value".to_vec());
        let response = DhtResponse { key, value: value.clone() };

        let json = serde_json::to_vec(&response).unwrap();
        let packet = create_packet(PacketType::DhtResponse.as_u8(), 0x2000, 0x1000, &json);

        let (header, payload) = parse_packet(&packet).unwrap();
        assert_eq!(header.packet_type, PacketType::DhtResponse.as_u8());

        let parsed_response: DhtResponse = serde_json::from_slice(payload).unwrap();
        assert_eq!(parsed_response.key, key);
        assert_eq!(parsed_response.value, value);
    }
}
