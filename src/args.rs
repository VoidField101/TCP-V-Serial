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
    pub address: SocketAddr,

    #[clap(short, long)]
    pub listen: bool
}
