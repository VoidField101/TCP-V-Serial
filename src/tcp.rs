// Copyright (c) 2022 voidfield101
// 
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use std::{io, sync::{Arc, Mutex}};

use log::info;
use tokio::{net::{TcpListener, TcpStream}};


pub type OptionalStreamType = Arc<Mutex<Option<TcpStream>>>;

pub trait OptionalStream {
    fn new_optstream() -> OptionalStreamType;
    fn has_stream(&self) -> bool;

    fn with_stream<T, R>(&self, f: T) -> Option<R>
        where T: FnOnce(&TcpStream) -> R;

    fn set_stream(&mut self, opt: Option<TcpStream>);
}

impl OptionalStream for OptionalStreamType {
    fn new_optstream() -> OptionalStreamType {
        return Arc::new(Mutex::new(Option::None))
    }

    fn has_stream(&self) -> bool {
        self.lock().unwrap().is_some()
    }

    fn with_stream<T, R>(&self, f: T) -> Option<R>
        where T: FnOnce(&TcpStream) -> R
    {
        let socket_opt = self.lock().unwrap();
        
        if let Some(socket) = &*socket_opt {
            return Some(f(socket));
        }
        else {
            return None;
        }
    }

    fn set_stream(&mut self, opt: Option<TcpStream>) {
        *self.lock().unwrap() = opt;
    }

}


pub async fn run_serverloop(listener: TcpListener, mut socket: OptionalStreamType) -> io::Result<()> {
    loop {
        let (in_socket, in_addr) = listener.accept().await?;
        info!("Incomming connection from IP {} from Port {}", in_addr.ip().to_string(), in_addr.port());
        if !socket.has_stream() {
            info!("Connection accepted");
            socket.set_stream(Option::Some(in_socket));
        }
        else {
            info!("Rejected! Already connected");
        }
    }
}