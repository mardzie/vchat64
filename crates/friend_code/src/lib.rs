use std::{
    net::{self, IpAddr, SocketAddr},
    str::FromStr,
};

pub const IPV4_BYTES_COUNT: usize = 4;
pub const IPV6_BYTES_COUNT: usize = 16;
pub const PORT_BYTES_COUNT: usize = 2;
pub const IPV4_ADDR_BYTES_COUNT: usize = IPV4_BYTES_COUNT + PORT_BYTES_COUNT;
pub const IPV6_ADDR_BYTES_COUNT: usize = IPV6_BYTES_COUNT + PORT_BYTES_COUNT;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FriendCode(String);

impl FriendCode {
    pub fn new(addr: SocketAddr) -> Self {
        let mut addr_bytes: Vec<u8>;
        match addr.ip() {
            IpAddr::V4(ipv4) => {
                addr_bytes = Vec::with_capacity(6);
                addr_bytes.extend_from_slice(&ipv4.octets());
            }
            IpAddr::V6(ipv6) => {
                addr_bytes = Vec::with_capacity(18);
                addr_bytes.extend_from_slice(&ipv6.octets());
            }
        };
        addr_bytes.extend_from_slice(&addr.port().to_be_bytes());

        let hex = hex::encode(addr_bytes);

        Self(hex)
    }

    pub fn new_local(port: u16) -> Result<Self, IpError> {
        let ip = match local_ip_address::local_ip() {
            Ok(ip) => ip,
            Err(e) => {
                log::warn!("Failed to get local ip: {}", e);
                return Err(IpError::from(e));
            }
        };

        Ok(Self::from_ip_port(ip, port))
    }

    pub fn new_public(runtime: &tokio::runtime::Runtime, port: u16) -> Result<Self, IpError> {
        let ip = match runtime.block_on(public_ip_address::perform_lookup(None)) {
            Ok(lookup) => lookup.ip,
            Err(e) => {
                log::warn!("Failed to perform public ip lookup.");
                return Err(IpError::from(e));
            }
        };

        Ok(Self::from_ip_port(ip, port))
    }

    pub fn from_ip_port(ip: IpAddr, port: u16) -> Self {
        Self::new(SocketAddr::new(ip, port))
    }

    pub fn from_string_addr(addr: &str) -> Result<Self, Error> {
        let addr = SocketAddr::from_str(addr)?;

        let mut addr_bytes: Vec<u8>;
        match addr.ip() {
            IpAddr::V4(ipv4) => {
                addr_bytes = Vec::with_capacity(6);
                addr_bytes.extend_from_slice(&ipv4.octets());
            }
            IpAddr::V6(ipv6) => {
                addr_bytes = Vec::with_capacity(18);
                addr_bytes.extend_from_slice(&ipv6.octets());
            }
        };
        addr_bytes.extend_from_slice(&addr.port().to_be_bytes());

        let friend_code = hex::encode(addr_bytes);
        Ok(Self(friend_code))
    }

    pub fn from_string_ip(ip: &str, port: u16) -> Result<Self, Error> {
        let mut addr = ip.to_string();
        addr.push(':');
        addr.push_str(&port.to_string());
        Self::from_string_addr(addr.as_str())
    }

    pub fn from_string_friend_code(fc_string: String) -> Result<Self, Error> {
        let fc_string = fc_string.trim().replace(" ", "");

        match Self::check_friend_code_format(&fc_string) {
            Ok(_) => {}
            Err(_) => return Err(Error::InvalidFriendCodeString),
        };

        Ok(Self(fc_string))
    }

    pub fn to_socket_addr(&self) -> SocketAddr {
        let bytes = match hex::decode(&self.0) {
            Ok(bytes) => bytes,
            Err(e) => {
                log::warn!("Failed to decode friend code: {}", e);
                panic!("The internal `String` must always be a valid hex code!");
            }
        };

        let mut port_bytes = [0u8; 2];
        let ip = match bytes.len() {
            IPV4_ADDR_BYTES_COUNT => {
                let mut ip_bytes = [0u8; 4];
                ip_bytes.copy_from_slice(&bytes[..4]);
                port_bytes.copy_from_slice(&bytes[4..6]);

                std::net::IpAddr::from(std::net::Ipv4Addr::from_octets(ip_bytes))
            }
            IPV6_ADDR_BYTES_COUNT => {
                let mut ip_bytes = [0u8; 16];
                ip_bytes.copy_from_slice(&bytes[..16]);
                port_bytes.copy_from_slice(&bytes[16..18]);

                std::net::IpAddr::V6(std::net::Ipv6Addr::from_octets(ip_bytes))
            }
            count => {
                log::warn!("Failed to decode friend code: Found {} bytes", count);
                panic!("The internal `String` must have a valid length!")
            }
        };

        SocketAddr::new(ip, u16::from_be_bytes(port_bytes))
    }

    fn check_friend_code_format(fc: &str) -> Result<(), ()> {
        let bytes = match hex::decode(fc) {
            Ok(bytes) => bytes,
            Err(_) => {
                return Err(());
            }
        };

        match bytes.len() {
            IPV4_ADDR_BYTES_COUNT | IPV6_ADDR_BYTES_COUNT => Ok(()),
            _ => Err(()),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn to_string(&self) -> String {
        self.0
    }

    pub fn to_pretty_string() -> String {}
}

impl From<SocketAddr> for FriendCode {
    fn from(addr: SocketAddr) -> Self {
        Self::new(addr)
    }
}

impl From<FriendCode> for SocketAddr {
    fn from(friend_code: FriendCode) -> Self {
        friend_code.to_socket_addr()
    }
}

impl TryFrom<&str> for FriendCode {
    type Error = Error;

    fn try_from(addr: &str) -> Result<Self, Self::Error> {
        Self::from_string_addr(addr)
    }
}

impl TryFrom<String> for FriendCode {
    type Error = Error;

    fn try_from(addr: String) -> Result<Self, Self::Error> {
        FriendCode::try_from(addr.as_str())
    }
}

impl std::fmt::Display for FriendCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("Address Parse Error: {0}")]
    AddrParse(#[from] net::AddrParseError),
    #[error("Invalid Friend Code String")]
    InvalidFriendCodeString,
}

#[derive(thiserror::Error, Debug)]
pub enum IpError {
    #[error("Local Ip Error: {0}")]
    Local(#[from] local_ip_address::Error),
    #[error("Public Ip Error: {0}")]
    Public(#[from] public_ip_address::error::Error),
}
