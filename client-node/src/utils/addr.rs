use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;
use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct IpAddr {
    pub address: [u8; 4], // Using a fixed-size array for IPv4 address
}

impl IpAddr {
    pub fn new(address: [u8; 4]) -> Self {
        IpAddr { address }
    }

    pub fn from_interface(interface: [u8; 4], index: u8) -> Self {
        let mut address = interface;
        address[3] = index;
        IpAddr { address }
    }

    pub fn from_string(addr: &str) -> Result<Self, String> {
        let parts: Vec<&str> = addr.split('.').collect();
        if parts.len() != 4 {
            return Err(format!("Invalid IP address format: {}", addr));
        }
        let mut address = [0; 4];
        for (i, part) in parts.iter().enumerate() {
            match part.parse::<u8>() {
                Ok(num) => address[i] = num,
                Err(_) => return Err(format!("Invalid byte in IP address: {}", part)),
            }
        }
        Ok(IpAddr { address })
    }

    pub fn to_socket_addr(&self, port: u16) -> std::net::SocketAddr {
        let addr = format!(
            "{}.{}.{}.{}:{}",
            self.address[0], self.address[1], self.address[2], self.address[3], port
        );
        addr.parse().expect("Invalid socket address format")
    }
}

impl Into<[u8; 4]> for IpAddr {
    fn into(self) -> [u8; 4] {
        self.address
    }
}

impl From<[u8; 4]> for IpAddr {
    fn from(address: [u8; 4]) -> Self {
        IpAddr { address }
    }
}

impl From<Ipv4Addr> for IpAddr {
    fn from(addr: Ipv4Addr) -> Self {
        IpAddr {
            address: addr.octets(),
        }
    }
}

impl From<std::net::IpAddr> for IpAddr {
    fn from(addr: std::net::IpAddr) -> Self {
        if let std::net::IpAddr::V4(v4) = addr {
            IpAddr::from(v4)
        } else {
            panic!("Only IPv4 addresses are supported")
        }
    }
}

impl From<&str> for IpAddr {
    fn from(addr: &str) -> Self {
        IpAddr::from_string(addr).expect("Invalid IP address format")
    }
}

impl FromStr for IpAddr {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        IpAddr::from_string(s)
    }
}

impl Debug for IpAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{}.{}.{}",
            self.address[0], self.address[1], self.address[2], self.address[3]
        )
    }
}

impl Display for IpAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{}.{}.{}",
            self.address[0], self.address[1], self.address[2], self.address[3]
        )
    }
}
