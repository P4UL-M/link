pub trait Serializable {
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(data: &[u8]) -> Result<Self, String>
    where
        Self: Sized;
}

impl Serializable for String {
    fn serialize(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }

    fn deserialize(data: &[u8]) -> Result<Self, String> {
        String::from_utf8(data.to_vec()).map_err(|e| e.to_string())
    }
}
