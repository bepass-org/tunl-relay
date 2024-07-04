use std::net::IpAddr;

use crate::proto::*;

use tokio::{
    io::{self, copy_bidirectional, AsyncReadExt, AsyncWriteExt, BufReader, Error, ErrorKind},
    net::{TcpListener, TcpStream},
};

pub struct Proxy {
    bind: IpAddr,
    port: u16,
    version: Version,
}

impl Proxy {
    pub fn new(version: Version, bind: IpAddr, port: u16) -> Self {
        Self {
            bind,
            port,
            version,
        }
    }

    pub async fn run(&self) -> io::Result<()> {
        let addr = format!("{}:{}", self.bind, self.port);

        let l = TcpListener::bind(&addr).await?;
        log::info!("Listening {}", &addr);

        loop {
            match self.listener(&l).await {
                Err(e) => log::error!("[listener]: {e}"),
                _ => {}
            };
        }
    }

    async fn listener(&self, l: &TcpListener) -> io::Result<()> {
        let (mut stream, _) = l.accept().await?;

        let header = match &self.version {
            Version::V1 => {
                let mut buf = vec![];

                // here we're sure the header length is at least 13 bytes
                let mut chunk = [0u8; 13];
                stream.read_exact(&mut chunk).await?;
                buf.extend_from_slice(&chunk);

                let mut upper = 30; // 30 bytes are enough to see if we're
                                    // reading a valid header or not
                                    // continue reading the header until reach '\n'
                loop {
                    if upper == 0 {
                        return Err(Error::new(ErrorKind::Other, "invalid header"));
                    }

                    let b = stream.read_u8().await?;
                    if b == b'\n' {
                        break;
                    }

                    buf.push(b);
                    upper -= 1;
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
        tokio::spawn(handler(header, stream));

        Ok(())
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
    let (reader, mut writer) = stream.split();
    let mut reader = BufReader::new(reader);

    let udp_stream = tokio::net::UdpSocket::bind("0.0.0.0:0").await?;
    udp_stream.connect(format!("{addr}:{port}")).await?;

    let mut tcp_buf = [0u8; 65535];
    let mut udp_buf = [0u8; 65535];

    loop {
        tokio::select! {
            result = reader.read(&mut tcp_buf) => {
                let n = result?;
                if n == 0 {
                    break;
                }
                udp_stream.send(&tcp_buf[..n]).await?;
            }

            result = udp_stream.recv(&mut udp_buf) => {
                let n = result?;
                writer.write_all(&udp_buf[..n]).await?;
            }
        }
    }

    Ok(())
}
