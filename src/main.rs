
use std::{net::{SocketAddr, SocketAddrV4, Ipv4Addr}, io};
use args::Args;
use log::info;
use tcp::OptionalStream;
use clap::Parser;
use tokio::net::TcpListener;

mod serial;
mod args;
mod tcp;
mod cdc;
mod buffer;



#[tokio::main(flavor = "current_thread")]
async fn main() -> io::Result<()> {
    env_logger::init();
    let args = Args::parse();

    let stream = tcp::OptionalStreamType::new_optstream();

    if args.listen {
        let server = TcpListener::bind(args.address).await?;
        info!("Binding server to {}", args.address.to_string());
        tokio::spawn(tcp::run_serverloop(server, stream));
    }

    let device = serial::new_device(0).await?;
    serial::run_usbip(device, SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), args.port))).await?;

    Ok(())
}   
