use std::net::{SocketAddr, ToSocketAddrs};

use color_eyre::eyre::Result;

use crate::{audio::Audio, udp_net::UdpNet};

pub mod error;

pub struct VChat {
    audio: Audio,
    udp_net: UdpNet,
}

impl VChat {
    pub fn new<A>(addr: A) -> Result<Self>
    where
        A: ToSocketAddrs,
    {
        Ok(Self {
            audio: Audio::new(),
            udp_net: UdpNet::new(addr)?,
        })
    }

    pub fn add_address(&self, addr: SocketAddr) -> Result<(), error::Error> {
        let mut addresses = self
            .udp_net
            .addresses
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        addresses.push(addr);
        addresses.sort();
        addresses.dedup();

        Ok(())
    }

    #[inline]
    pub fn get_addresses(&self) -> &std::sync::Arc<std::sync::RwLock<Vec<SocketAddr>>> {
        &self.udp_net.addresses
    }

    #[inline]
    pub fn clear_addresses(&self) {
        self.udp_net
            .addresses
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clear();
    }
}
