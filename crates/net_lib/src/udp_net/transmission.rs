use std::net::{SocketAddr, ToSocketAddrs};

use crate::{traits::Bytes, udp_net::error};

#[allow(dead_code)]
pub trait Sender<P: Bytes> {
    fn send(&mut self, packet: &P) -> Result<(), error::SendError>;

    fn send_to(&mut self, packet: &P, addr: impl ToSocketAddrs) -> Result<(), error::SendError>;

    fn send_to_all(
        &mut self,
        packet: &P,
        addrs: &[impl ToSocketAddrs],
    ) -> Result<(), error::SendError>;
}

#[allow(dead_code)]
pub trait Receiver<P: Bytes> {
    fn peek(&mut self) -> Result<P, error::PeekError>;

    fn peek_from(&mut self) -> Result<(P, SocketAddr), error::PeekError>;

    fn recv(&mut self) -> Result<P, error::RecvError>;

    fn recv_from(&mut self) -> Result<(P, SocketAddr), error::PeekError>;
}
