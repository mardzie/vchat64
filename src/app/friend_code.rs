use std::net::{IpAddr, SocketAddr};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FriendCode(String);

impl FriendCode {
    fn new(addr: SocketAddr) -> Self {
        let mut addr_bytes: Vec<u8> = Vec::with_capacity(18); // 18 bytes to make space for ipv6 + port or ipv4 + port.
        match addr.ip() {
            std::net::IpAddr::V4(ipv4) => {
                addr_bytes.extend_from_slice(&ipv4.octets());
            }
            std::net::IpAddr::V6(ipv6) => {
                addr_bytes.extend_from_slice(&ipv6.octets());
            }
        };

        addr_bytes.extend_from_slice(&addr.port().to_be_bytes());

        let hex = hex::encode(addr_bytes);
        Self(
            hex.chars()
                .collect::<Vec<char>>()
                .chunks(2)
                .map(|chunk| chunk.iter().collect::<String>())
                .collect::<Vec<String>>()
                .join(" "),
        )
    }

    pub fn from_ip_port(ip: IpAddr, port: u16) -> Self {
        Self::new(SocketAddr::new(ip, port))
    }
    
    pub fn from_string(addr: String) -> Result<Self, ()> {}
    
    pub fn from_string_ip(ip: String, port: u16) -> Result<Self, ()> {}

    pub fn new_local(port: u16) -> Result<Self, String> {
        let ip = match local_ip_address::local_ip() {
            Ok(ip) => ip,
            Err(e) => {
                log::warn!("Failed to get local ip: {}", e);
                return Err("Failed to fetch local friend code!".to_string());
            }
        };

        Ok(Self::from_ip_port(ip, port))
    }

    pub fn new_public(runtime: &tokio::runtime::Runtime, port: u16) -> Result<Self, String> {
        let ip = match runtime.block_on(public_ip_address::perform_lookup(None)) {
            Ok(lookup) => lookup.ip,
            Err(_) => {
                log::warn!("Failed to perform public ip lookup.");
                return Err("Failed to fetch public friend code!".to_string());
            }
        };

        Ok(Self::from_ip_port(ip, port))
    }

    fn to_socket_addr(&self) -> Result<SocketAddr, ()> {
        friend_code = friend_code.replace(" ", "");

        let bytes = match hex::decode(friend_code) {
            Ok(bytes) => bytes,
            Err(e) => {
                log::warn!("Failed to decode friend code: {}", e);
                return Err(());
            }
        };

        let mut port_bytes = [0u8; 2];
        let ip = match bytes.len() {
            6 => {
                let mut ip_bytes = [0u8; 4];
                ip_bytes.copy_from_slice(&bytes[..4]);
                port_bytes.copy_from_slice(&bytes[4..6]);

                std::net::IpAddr::from(std::net::Ipv4Addr::from_octets(ip_bytes))
            }
            18 => {
                let mut ip_bytes = [0u8; 16];
                ip_bytes.copy_from_slice(&bytes[..16]);
                port_bytes.copy_from_slice(&bytes[16..18]);

                std::net::IpAddr::V6(std::net::Ipv6Addr::from_octets(ip_bytes))
            }
            bytes => {
                log::warn!("Failed to decode friend code: Found {} bytes", bytes);
                return Err(());
            }
        };

        Ok(SocketAddr::new(ip, u16::from_be_bytes(port_bytes)))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<SocketAddr> for FriendCode {
    fn from(addr: SocketAddr) -> Self {
        Self::new(addr)
    }
}

impl TryFrom<FriendCode> for SocketAddr {
    type Error = ();

    fn try_from(friend_code: FriendCode) -> Result<Self, Self::Error> {
        friend_code.to_socket_addr(friend_code)
    }
}

impl std::fmt::Display for FriendCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
