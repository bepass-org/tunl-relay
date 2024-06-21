use std::net::IpAddr;
use std::str::FromStr;

use bincode::{Decode, Encode};
use serde::Serialize;
use tokio::io::{Error, ErrorKind, Result};

#[derive(clap::ValueEnum, Clone, Default, Debug, Decode, Encode, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Version {
    #[default]
    V1,
    V2,
}

#[derive(Debug, Decode, Encode)]
pub enum Network {
    Tcp,
    Udp,
}

#[derive(Debug, Decode, Encode)]
pub struct Header {
    pub ver: Version,
    pub net: Network,
    pub addr: IpAddr,
    pub port: u16,
}

impl Header {
    pub fn from_v2(buf: &[u8]) -> Result<Self> {
        let (decoded, _) = bincode::decode_from_slice(&buf, bincode::config::standard())
            .map_err(|_| Error::new(ErrorKind::Other, "invalid header format"))?;
        Ok(decoded)
    }

    pub fn from_v1(buf: &[u8]) -> Result<Self> {
        let d1 = buf.iter().position(|&x| x == b'@').ok_or(Error::new(
            ErrorKind::Other,
            "could not find @ in the header",
        ))?;
        let d2 = buf.iter().position(|&x| x == b'$').ok_or(Error::new(
            ErrorKind::Other,
            "could not find $ in the header",
        ))?;

        let net = match &buf[..d1] {
            b"tcp" => Network::Tcp,
            _ => Network::Udp,
        };

        let addr = {
            let ip = String::from_utf8_lossy(&buf[d1 + 1..d2]);
            IpAddr::from_str(&ip).map_err(|_| Error::new(ErrorKind::Other, "invalid ip address"))
        }?;

        let port = {
            let p = String::from_utf8_lossy(&buf[d2 + 1..buf.len() - 1]);
            p.parse::<u16>()
                .map_err(|_| Error::new(ErrorKind::Other, "invalid port number"))
        }?;

        let ver = Version::V1;

        Ok(Self { net, addr, port, ver })
    }
}
