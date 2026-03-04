use std::{
    net::ToSocketAddrs,
    sync::{Arc, atomic::AtomicBool},
};

use crate::udp_net::UdpNet;

mod error;

#[derive(Debug)]
pub struct VoiceNet {
    udp_net: UdpNet,
}

impl VoiceNet {
    pub fn new<A>(addr: A, exit: Arc<AtomicBool>) -> Result<Self, error::Error>
    where
        A: ToSocketAddrs,
    {
        let (udp_net, sender, receiver) = UdpNet::new(addr, exit)?;
        
        Ok(Self {
            udp_net,
        })
    }
}
