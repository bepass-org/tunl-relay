use std::net::IpAddr;

use crate::proto::*;

use tokio::{
    io::{self, copy_bidirectional, AsyncReadExt},
    net::{TcpListener, TcpStream},
};

pub async fn run(bind: IpAddr, port: u16) -> io::Result<()> {
    let addr = format!("{bind}:{port}");

    let listener = TcpListener::bind(&addr).await?;
    log::info!("Listening {}", &addr);

    loop {
        let (mut conn, _) = listener.accept().await?;

        let mut len = [0u8; 2];
        conn.read(&mut len).await?;

        let mut buf = vec![0u8; u16::from_be_bytes(len) as _];
        conn.read(&mut buf).await?;

        let (header, _): (Header, usize) =
            match bincode::decode_from_slice(&buf, bincode::config::standard()) {
                Ok(h) => h,
                Err(e) => {
                    log::error!("invalid header format: {e}");
                    continue;
                }
            };

        match header.net {
            Network::Tcp => tcp_handler(&mut conn, header.addr, header.port).await?,
            Network::Udp => unimplemented!(),
        }
    }
}

async fn tcp_handler(stream: &mut TcpStream, addr: IpAddr, port: u16) -> io::Result<()> {
    let mut upstream = TcpStream::connect(format!("{addr}:{port}")).await?;
    copy_bidirectional(stream, &mut upstream).await?;
    Ok(())
}
