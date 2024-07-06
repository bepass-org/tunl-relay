use crate::proto::Version;
use cidr::IpCidr;

use std::net::IpAddr;

use anyhow::{anyhow, Result};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub bind: IpAddr,
    pub port: u16,
    pub version: Version,
    #[serde(default)]
    pub whitelist: Vec<IpCidr>,
    #[serde(default)]
    pub blacklist: Vec<IpCidr>,
}

impl Config {
    pub fn new(config: &str) -> Result<Self> {
        let mut config: Self = match toml::from_str(config) {
            Ok(c) => c,
            Err(e) => return Err(anyhow!("could not parse config file {}", e)),
        };

        // block private networks by default
        let addrs: Vec<IpCidr> = [
            "127.0.0.0/8",
            "::1/128",
            "10.0.0.0/8",
            "172.16.0.0/12",
            "192.168.0.0/16",
            "fd00::/8",
        ]
        .iter()
        .map(|s| s.parse().unwrap())
        .collect();
        config.blacklist.extend_from_slice(&addrs);

        Ok(config)
    }
}
