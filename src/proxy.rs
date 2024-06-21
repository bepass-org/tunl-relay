use std::net::IpAddr;

use crate::proto::*;

use tokio::{
    io::{self, copy_bidirectional, AsyncBufReadExt, AsyncReadExt, BufReader},
    net::{TcpListener, TcpStream},
};

pub async fn run(version: Version, bind: IpAddr, port: u16) -> io::Result<()> {
    let addr = format!("{bind}:{port}");

    let listener = TcpListener::bind(&addr).await?;
    log::info!("Listening {}", &addr);

    loop {
        let (conn, _) = listener.accept().await?;
        let mut stream = BufReader::new(conn);

        let header = match &version {
            Version::V1 => {
                let mut buf = vec![];

                stream.read_until(b'\r', &mut buf).await?;
                if buf.is_empty() {
                    // TODO: error
                    continue;
                }

                Header::from_v1(&buf)?
            }
            Version::V2 => {
                let mut len = [0u8; 2];
                stream.read(&mut len).await?;

                let mut buf = vec![0u8; u16::from_be_bytes(len) as _];
                stream.read(&mut buf).await?;

                Header::from_v2(&buf)?
            }
        };

        log::info!(
            "accepted [{:?}] {}:{}",
            header.net,
            header.addr,
            header.port
        );
        tokio::spawn(handler(header, stream.into_inner()));
    }
}

async fn handler(header: Header, stream: TcpStream) {
    if let Err(e) = match header.net {
        Network::Tcp => tcp_handler(stream, header.addr, header.port).await,
        Network::Udp => unimplemented!(),
    } {
        log::error!("error {e}");
    }
}

async fn tcp_handler(mut stream: TcpStream, addr: IpAddr, port: u16) -> io::Result<()> {
    let mut upstream = TcpStream::connect(format!("{addr}:{port}")).await?;
    copy_bidirectional(&mut stream, &mut upstream).await?;

    Ok(())
}
