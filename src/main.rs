mod config;
mod proto;
mod proxy;

use config::Config;
use proxy::Proxy;

use anyhow::{anyhow, Result};
use clap::Parser;

#[derive(Debug, Parser)]
#[clap(author, version)]
pub struct Args {
    #[clap(short, long)]
    pub config: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let args = Args::parse();

    let config = match std::fs::read_to_string(args.config) {
        Ok(c) => Config::new(&c),
        _ => panic!("could not find the config file"),
    }?;

    let proxy = Proxy::new(config);
    match proxy.run().await {
        Err(e) => Err(anyhow!("{e}")),
        _ => Ok(()),
    }
}
