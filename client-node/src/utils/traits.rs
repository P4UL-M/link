pub trait Serializable {
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(data: &[u8]) -> Self;
}

impl Serializable for String {
    fn serialize(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }

    fn deserialize(data: &[u8]) -> Self {
        String::from_utf8(data.to_vec()).unwrap_or_default()
    }
}
