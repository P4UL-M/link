pub trait IntoIpv4Bytes {
    fn into_ipv4(self) -> [u8; 4];
}

impl IntoIpv4Bytes for &str {
    fn into_ipv4(self) -> [u8; 4] {
        self.split('.')
            .map(|s| s.parse::<u8>().unwrap_or(0))
            .collect::<Vec<u8>>()
            .try_into()
            .unwrap_or([0; 4])
    }
}

impl IntoIpv4Bytes for [u8; 4] {
    fn into_ipv4(self) -> [u8; 4] {
        self
    }
}

impl IntoIpv4Bytes for &[u8; 4] {
    fn into_ipv4(self) -> [u8; 4] {
        *self
    }
}
