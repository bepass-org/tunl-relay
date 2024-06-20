use bincode::{Decode, Encode};
use std::net::IpAddr;

#[derive(Decode, Encode)]
pub enum Network {
    Tcp,
    Udp,
}

#[derive(Decode, Encode)]
pub struct Header {
    pub net: Network,
    pub addr: IpAddr,
    pub port: u16,
}
