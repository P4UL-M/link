use crate::IntoIpv4Bytes;
use crate::utils::traits::Serializable;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PacketType {
    Request,
    Acknowledgment,
    Discovery,
    Error,
}

pub struct Packet<S: Serializable> {
    pub(crate) id: uuid::Uuid,
    pub type_: PacketType,
    pub source: [u8; 4],      // Using a fixed-size array for source IPv4
    pub destination: [u8; 4], // Using a fixed-size array for destination IPv4
    pub data: Option<S>,
    pub timestamp: u64, // Logical timestamp for the packet
}

impl<S: Serializable + Clone> Packet<S> {
    pub fn new<I: IntoIpv4Bytes, D: IntoIpv4Bytes>(
        type_: PacketType,
        source: I,
        destination: D,
        timestamp: u64,
        data: Option<S>,
    ) -> Self {
        Packet {
            id: uuid::Uuid::new_v4(),
            type_,
            source: source.into_ipv4(),
            destination: destination.into_ipv4(),
            timestamp,
            data,
        }
    }

    pub fn new_with_id<I: IntoIpv4Bytes, D: IntoIpv4Bytes>(
        id: uuid::Uuid,
        type_: PacketType,
        source: I,
        destination: D,
        timestamp: u64,
        data: Option<S>,
    ) -> Self {
        Packet {
            id,
            type_,
            source: source.into_ipv4(),
            destination: destination.into_ipv4(),
            timestamp,
            data: data,
        }
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
}

impl<S: Serializable + Clone> Serializable for Packet<S> {
    fn serialize(&self) -> Vec<u8> {
        let mut serialized = Vec::new();
        serialized.extend_from_slice(&self.id.to_bytes_le()); // 16 bytes for UUID
        serialized.push(self.type_ as u8); // 1 byte for message type
        serialized.extend_from_slice(&self.source); // 4 bytes for source IP
        serialized.extend_from_slice(&self.destination); // 4 bytes for destination IP
        serialized.extend_from_slice(&self.timestamp.to_le_bytes()); // 8 bytes for timestamp
        if let Some(ref data) = self.data {
            serialized.extend_from_slice(data.serialize().as_slice());
        }
        serialized
    }

    fn deserialize(data: &[u8]) -> Self {
        if data.len() < 33 {
            panic!("Data too short to deserialize a Message");
        }
        if data.len() > 512 {
            panic!("Data too long to deserialize a Message");
        }
        if data[16] > 3 {
            panic!("Invalid message type in serialized data");
        }

        let id = uuid::Uuid::from_bytes_le(
            data[0..16]
                .try_into()
                .expect("Failed to parse UUID from data"),
        );

        let type_ = match data[16] {
            0 => PacketType::Request,
            1 => PacketType::Acknowledgment,
            2 => PacketType::Discovery,
            3 => PacketType::Error,
            _ => panic!("Unknown message type"),
        };

        let source = data[17..21].try_into().expect("Failed to parse source IP");
        let destination = data[21..25]
            .try_into()
            .expect("Failed to parse destination IP");
        let timestamp =
            u64::from_le_bytes(data[25..33].try_into().expect("Failed to parse timestamp"));

        let data = if let Some(d) = data.get(33..) {
            Some(S::deserialize(d.as_ref()))
        } else {
            None
        };

        Packet {
            id,
            type_,
            source,
            destination,
            timestamp,
            data: data,
        }
    }
}

impl<S: Serializable + Clone + std::fmt::Debug> std::fmt::Debug for Packet<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Message")
            .field("type", &self.type_)
            .field("source", &self.source)
            .field("destination", &self.destination)
            .field("data", &self.data)
            .finish()
    }
}

impl<S: Serializable + Clone + std::fmt::Debug> std::fmt::Display for Packet<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let source_ip = format!(
            "{}.{}.{}.{}",
            self.source[0], self.source[1], self.source[2], self.source[3]
        );
        let destination_ip = format!(
            "{}.{}.{}.{}",
            self.destination[0], self.destination[1], self.destination[2], self.destination[3]
        );
        write!(
            f,
            "Message(type: {:?}, source: {}, destination: {}, data: {:?})",
            self.type_, source_ip, destination_ip, self.data
        )
    }
}

impl<S: Serializable + Clone> Clone for Packet<S> {
    fn clone(&self) -> Self {
        Packet {
            id: self.id,
            type_: self.type_,
            source: self.source,
            destination: self.destination,
            timestamp: self.timestamp,
            data: self.data.clone(),
        }
    }
}

impl<S: Serializable + Clone> Into<Vec<u8>> for Packet<S> {
    fn into(self) -> Vec<u8> {
        self.serialize()
    }
}
