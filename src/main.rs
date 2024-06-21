mod proto;
mod proxy;

use proto::Version;

use anyhow::{anyhow, Result};
use clap::Parser;

use std::net::IpAddr;

#[derive(Debug, Parser)]
#[clap(author, version)]
pub struct Args {
    #[clap(short, long, default_value = "0.0.0.0")]
    pub bind: IpAddr,
    #[clap(short, long, default_value = "6666")]
    pub port: u16,
    #[clap(short, long, value_enum, default_value_t)]
    pub version: Version,
}

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let args = Args::parse();

    match proxy::run(args.version, args.bind, args.port).await {
        Err(e) => Err(anyhow!("{e}")),
        _ => Ok(()),
    }
}
