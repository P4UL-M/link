use crate::IntoIpv4Bytes;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MessageType {
    Request,
    Acknowledgment,
    Discovery,
    Error,
}

pub struct Message {
    pub(crate) id: uuid::Uuid,
    pub type_: MessageType,
    pub source: [u8; 4],      // Using a fixed-size array for source IPv4
    pub destination: [u8; 4], // Using a fixed-size array for destination IPv4
    pub data: Option<Vec<u8>>,
}

pub trait Serializable {
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(data: &[u8]) -> Self;
}

impl Message {
    pub fn new<S: IntoIpv4Bytes, D: IntoIpv4Bytes>(
        type_: MessageType,
        source: S,
        destination: D,
        data: Option<&[u8]>,
    ) -> Self {
        Message {
            id: uuid::Uuid::new_v4(),
            type_,
            source: source.into_ipv4(),
            destination: destination.into_ipv4(),
            data: data.map(|d| d.to_vec()),
        }
    }

    pub fn new_with_id<S: IntoIpv4Bytes, D: IntoIpv4Bytes>(
        id: uuid::Uuid,
        type_: MessageType,
        source: S,
        destination: D,
        data: Option<&[u8]>,
    ) -> Self {
        Message {
            id,
            type_,
            source: source.into_ipv4(),
            destination: destination.into_ipv4(),
            data: data.map(|d| d.to_vec()),
        }
    }
}

impl Serializable for Message {
    fn serialize(&self) -> Vec<u8> {
        let mut serialized = Vec::new();
        serialized.extend_from_slice(&self.id.to_bytes_le()); // 16 bytes for UUID
        serialized.push(self.type_ as u8); // 1 byte for message type
        serialized.extend_from_slice(&self.source); // 4 bytes for source IP
        serialized.extend_from_slice(&self.destination); // 4 bytes for destination IP
        if let Some(ref data) = self.data {
            serialized.extend_from_slice(data);
        }
        serialized
    }

    fn deserialize(data: &[u8]) -> Self {
        if data.len() < 25 {
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
            0 => MessageType::Request,
            1 => MessageType::Acknowledgment,
            2 => MessageType::Discovery,
            3 => MessageType::Error,
            _ => panic!("Unknown message type"),
        };

        let source = data[17..21].try_into().expect("Failed to parse source IP");
        let destination = data[21..25]
            .try_into()
            .expect("Failed to parse destination IP");

        let data = if let Some(d) = data.get(25..) {
            Some(d.to_vec())
        } else {
            None
        };

        Message {
            id,
            type_,
            source,
            destination,
            data,
        }
    }
}

impl std::fmt::Debug for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Message")
            .field("type", &self.type_)
            .field("source", &self.source)
            .field("destination", &self.destination)
            .field("data", &self.data)
            .finish()
    }
}

impl std::fmt::Display for Message {
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

impl Clone for Message {
    fn clone(&self) -> Self {
        Message {
            id: self.id,
            type_: self.type_,
            source: self.source,
            destination: self.destination,
            data: self.data.clone(),
        }
    }
}

impl Into<Vec<u8>> for Message {
    fn into(self) -> Vec<u8> {
        self.serialize()
    }
}
