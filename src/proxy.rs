use std::net::IpAddr;

use crate::proto::*;

use tokio::{
    io::{self, copy_bidirectional, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
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
        Network::Udp => udp_handler(stream, header.addr, header.port).await,
    } {
        log::error!("error {e}");
    }
}

async fn tcp_handler(mut stream: TcpStream, addr: IpAddr, port: u16) -> io::Result<()> {
    let mut upstream = TcpStream::connect(format!("{addr}:{port}")).await?;
    copy_bidirectional(&mut stream, &mut upstream).await?;

    Ok(())
}

async fn udp_handler(mut stream: TcpStream, addr: IpAddr, port: u16) -> io::Result<()> {
    // let mut upstream = TcpStream::connect(format!("{addr}:{port}")).await?;
    // copy_bidirectional(&mut stream, &mut upstream).await?;

    let udp_stream = tokio::net::UdpSocket::bind("0.0.0.0:0").await?;
    udp_stream.connect(format!("{addr}:{port}")).await?;

    let mut buf = [0u8; 65535];
    let mut udp_buf = [0u8; 65535];
    loop {
        tokio::select! {
            result = stream.read(&mut buf) => {
                let n = result?;
                if n == 0 {
                    break;
                }
                udp_stream.send(&buf[..n]).await?;
            }

            result = udp_stream.recv(&mut udp_buf) => {
                let n = result?;
                stream.write_all(&udp_buf[..n]).await?;
            }
        }
    }

    Ok(())
}
