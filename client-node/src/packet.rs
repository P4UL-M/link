use crate::utils::addr::IpAddr;
use bincode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const PACKET_SIZE: usize = 512; // Maximum size for a packet

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PacketType {
    Request,
    Acknowledgment,
    Discovery,
    Broadcast,
    DNSDiscovery,
    DNSUpdate,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PacketData {
    Text(String),
    Peers(Vec<IpAddr>),
    Empty,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Packet {
    pub id: Uuid,
    pub type_: PacketType,
    pub source: IpAddr,      // Using a fixed-size array for source IPv4
    pub destination: IpAddr, // Using a fixed-size array for destination IPv4
    pub data: PacketData,
    pub timestamp: u64, // Logical timestamp for the packet
}

impl Packet {
    pub fn new(
        type_: PacketType,
        source: IpAddr,
        destination: IpAddr,
        timestamp: u64,
        data: PacketData,
    ) -> Self {
        Packet {
            id: Uuid::new_v4(),
            type_,
            source: source,
            destination: destination,
            timestamp,
            data,
        }
    }

    pub fn new_with_id(
        id: Uuid,
        type_: PacketType,
        source: IpAddr,
        destination: IpAddr,
        timestamp: u64,
        data: PacketData,
    ) -> Self {
        Packet {
            id,
            type_,
            source,
            destination,
            timestamp,
            data,
        }
    }

    pub fn check_src<A: Into<IpAddr>>(&self, src: A) -> bool {
        self.source == src.into()
    }

    pub fn clone_with_new_id(&self) -> Self {
        Packet {
            id: uuid::Uuid::new_v4(),
            type_: self.type_,
            source: self.source,
            destination: self.destination,
            timestamp: self.timestamp,
            data: self.data.clone(),
        }
    }

    pub fn serialize(&self) -> Result<Vec<u8>, String> {
        bincode::serialize(self).map_err(|e| e.to_string())
    }

    pub fn deserialize(data: &[u8]) -> Result<Self, String> {
        bincode::deserialize(data).map_err(|e| e.to_string())
    }
}

impl std::fmt::Display for Packet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Packet(type: {:?}, source: {:?}, destination: {:?}, timestamp: {:?})",
            self.type_, self.source, self.destination, self.timestamp
        )
    }
}
