use args::Args;
use clap::Parser;
use log::info;
use std::{
    io,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
};
use tokio::{
    net::{TcpListener, TcpSocket},
};

mod args;
mod buffer;
mod cdc;
mod relay;
mod serial;

#[tokio::main(flavor = "current_thread")]
async fn main() -> io::Result<()> {
    //TODO: Return a regular Result with the Error trait. (Currently no custom errors are allowed)
    env_logger::init();
    let args = Args::parse();

    let mut stream = relay::StreamPluginRelay::new();
    let stream2 = stream.clone();

    if args.listen {
        let server = TcpListener::bind(args.address).await?;
        info!("Binding server to {}", args.address.to_string());
        tokio::spawn(relay::run_serverloop(server, stream));
    } else {
        let tcp_socket;
        if let SocketAddr::V4(_) = args.address {
            tcp_socket = TcpSocket::new_v4()?;
        } else if let SocketAddr::V6(_) = args.address {
            tcp_socket = TcpSocket::new_v6()?;
        } else {
            return Err(io::Error::from(io::ErrorKind::InvalidInput));
        }
        stream
            .set_stream(Some(tcp_socket.connect(args.address).await?))
            .await;
        info!("Connected to: {}", args.address.to_string());
    }

    let device = serial::new_device(0).await?;
    serial::run_usbip(
        device,
        SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), args.port)),
        stream2,
    )
    .await?;

    Ok(())
}
