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

pub trait IntoIpv4String {
    fn into_ipv4_string(self) -> String;
}

impl IntoIpv4String for [u8; 4] {
    fn into_ipv4_string(self) -> String {
        format!("{}.{}.{}.{}", self[0], self[1], self[2], self[3])
    }
}

impl IntoIpv4String for &[u8; 4] {
    fn into_ipv4_string(self) -> String {
        format!("{}.{}.{}.{}", self[0], self[1], self[2], self[3])
    }
}

impl IntoIpv4String for &str {
    fn into_ipv4_string(self) -> String {
        self.into_ipv4().into_ipv4_string()
    }
}

impl IntoIpv4String for String {
    fn into_ipv4_string(self) -> String {
        self.as_str().into_ipv4().into_ipv4_string()
    }
}

impl IntoIpv4String for &String {
    fn into_ipv4_string(self) -> String {
        self.as_str().into_ipv4().into_ipv4_string()
    }
}
