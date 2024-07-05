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
    pub whitelist: Vec<IpCidr>,
}

impl Config {
    pub fn new(config: &str) -> Result<Self> {
        match toml::from_str(config) {
            Ok(c) => Ok(c),
            Err(e) => Err(anyhow!("could not parse config file {}", e)),
        }
    }
}
