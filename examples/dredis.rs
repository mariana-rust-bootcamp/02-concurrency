use std::{io, net::SocketAddr};

use anyhow::Result;
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
};
use tracing::{info, warn};

const BUF_SIZE: usize = 4096;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    let addr = "0.0.0.0:6379";
    let listener = TcpListener::bind(addr).await?;
    info!("Dredis listening on: {}", addr);

    loop {
        // 每个tcp链接都会在新端口新起一个socket
        let (stream, raddr) = listener.accept().await?;
        info!("Accept a connection from: {}", raddr);
        tokio::spawn(async move {
            if let Err(e) = process_redis_conn(stream, raddr).await {
                warn!("Error processing connection from {}: {}", raddr, e);
            }
            // Ok::<(), anyhow::Error>(())
        });
    }
}

async fn process_redis_conn(mut stream: TcpStream, raddr: SocketAddr) -> Result<()> {
    loop {
        stream.readable().await?;

        let mut buf = Vec::with_capacity(BUF_SIZE);

        // Try to read data, this may still fail with `WouldBlock`
        // if the readiness event is a false positive.
        match stream.try_read_buf(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                info!("Read {} bytes from {}", n, raddr);
                let line = String::from_utf8_lossy(&buf);
                info!("{:?}", line);
                stream.write_all(b"+OK\r\n").await?;
            }
            // 匹配的高级写法, 相当于将{}中的if挪到外面写
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                return Err(e.into());
            }
        }
    }

    Ok(())
}
