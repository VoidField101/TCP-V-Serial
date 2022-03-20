// Copyright (c) 2022 voidfield101
// 
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use std::net::SocketAddr;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
pub struct Args {

    #[clap(short, long, default_value_t = 3240)]
    /// USB/IP port. 3240 per default.
    pub port: u16,


    #[clap(short, long, parse(try_from_str))]
    /// Address for TCP Socket to connect to (bind address when listen is set)
    pub address: SocketAddr,

    #[clap(short, long)]
    /// Listen to TCP socket instead of connecting
    pub listen: bool
}
