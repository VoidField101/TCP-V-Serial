// Copyright (c) 2022 voidfield101
// 
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use std::{io, sync::Arc, pin::Pin, task::Poll};

use log::{info, debug};
use tokio::{net::{TcpListener, TcpStream}, sync::{Mutex, Notify, RwLock}, io::{copy_bidirectional, AsyncRead, AsyncWrite}};


/**
 * Relays all data to the inner stream.
 * However when shutdown is called on the Write part, it will continously send EOF on the Read side.
 * The shutdown will not be forwarded to the inner stream.
 * 
 * Allows tokio::io::copy_bidirectional to stop copying when the other stream closed without closing the inner stream
 */
pub struct StopOnceStream<'a, S> {
    stream: &'a mut S,
    eof: bool
}

impl <'a, S> StopOnceStream<'a, S> {

    /**
     * Create a new StopOnceStream with a reference to a inner/original stream
     */
    pub fn new(stream: &'a mut S) -> Self {
        Self {
            stream: stream,
            eof: false
        }
    }

}

impl <S> AsyncRead for StopOnceStream<'_, S> 
    where S: AsyncRead + Send + Sync + Unpin
{

    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<io::Result<()>> {
        
        if self.eof {
            return Poll::Ready(Result::Ok(()));
        }

        let pin_stream = Pin::new(&mut *self.stream);
        return pin_stream.poll_read(cx, buf);
    }
}

impl <S> AsyncWrite for StopOnceStream<'_, S> 
    where S: AsyncWrite + Send + Sync + Unpin
{

    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, io::Error>> {
        Pin::new(&mut *self.stream).poll_write(cx, buf)
    }

    fn poll_flush(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), io::Error>> {
        Pin::new(&mut *self.stream).poll_flush(cx)
    }

    fn poll_shutdown(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), io::Error>> {
        self.eof = true;
        Poll::Ready(Result::Ok(()))
    }
}



/**
 * A container that may contain a stream and change stream later.
 */
pub struct StreamPluginRelay<S>
    where S: AsyncRead + AsyncWrite + Send + Unpin
{
    stream: Arc<Mutex<Option<S>>>,
    change_notif: Arc<Notify>,
    has_stream_store: Arc<RwLock<bool>>
}

impl <S> Clone for StreamPluginRelay<S>
    where S: AsyncRead + AsyncWrite + Send + Sync + Unpin
    
{
    fn clone(&self) -> Self { 
        Self {
            stream: self.stream.clone(),
            change_notif: self.change_notif.clone(),
            has_stream_store: self.has_stream_store.clone(),
        }
    }
}

impl <S> StreamPluginRelay<S>
    where S: AsyncRead + AsyncWrite + Send + Unpin
{
    /**
     * Create a new empty StreamPluginRelay
     */
    pub fn new() -> Self {
        Self {
            stream: Arc::new(Mutex::new(Option::None)),
            change_notif: Arc::new(Notify::new()),
            has_stream_store: Arc::new(RwLock::new(false))
        }
    }

    /**
     * Check wheter a has_stream flag is set
     */
    pub async fn has_stream(&self) -> bool {
        *self.has_stream_store.read().await
    }

    /**
     * Get Stream optional.
     * WARNING: Just because the optional is Some doesn't mean the stream is valid!
     */
    pub fn get_streamrc(&self) -> Arc<Mutex<Option<S>>>{
        self.stream.clone()
    }

    /**
     * Internal function
     * Allows to reset the has_stream flag after the copy task has stopped
     */
    async fn reset_stream(&self){
        *self.has_stream_store.write().await = false;
    }

    /**
     * Set new stream.
     * Make sure the original stream has closed before calling this function
     */
    pub async fn set_stream(&mut self, opt: Option<S>) {
        let has_stream = opt.is_some();
        {
            let mut stream_lock = self.stream.lock().await;
            let mut has_stream_lock = self.has_stream_store.write().await;
            *stream_lock = opt;
            *has_stream_lock = has_stream;
        }
        self.change_notif.notify_waiters();
    }

    /**
     * Await a notification from set_stream
     */
    async fn await_change(&self){
        self.change_notif.notified().await
    }
}

/**
 * Start the relay with a StreamPluginRelay.
 * Will continously wait until optstream contains a stream, copying until it closes and wait again
 */
pub async fn start_relay<OS,S>(optstream: StreamPluginRelay<OS>, stream: &mut S) -> io::Result<()>
    where S: AsyncRead + AsyncWrite + Send + Sync + Unpin,
          OS: AsyncRead + AsyncWrite + Send + Sync + Unpin
{
    loop {

        if optstream.has_stream().await {
            relay_task(&optstream, stream).await
        }
        else {
            debug!("Awaiting Stream");
            optstream.await_change().await;
            debug!("Notification change received");
        }
        
    }
}

async fn relay_task<OS,S>(optstream: &StreamPluginRelay<OS>, stream: &mut S)
    where S: AsyncRead + AsyncWrite + Send + Sync + Unpin,
        OS: AsyncRead + AsyncWrite + Send + Sync + Unpin
{
    let optstream_rc = optstream.get_streamrc();
    let mut optstream_guard = optstream_rc.lock().await;
    if let Some(stream_other) = &mut *optstream_guard {
        debug!("Start copying");
        let result = copy_bidirectional( stream_other,  &mut StopOnceStream::new(stream)).await;
        debug!("Copy stopped {:?}", result);
    }
    optstream.reset_stream().await;
}

/**
 * Run a TCP server where a single client is allowed to connect.
 * A new client is allowed after the previous one disconnects
 */
pub async fn run_serverloop(listener: TcpListener, mut socket: StreamPluginRelay<TcpStream>) -> io::Result<()> {
    loop {
        let (in_socket, in_addr) = listener.accept().await?;
        info!("Incomming connection from IP {} from Port {}", in_addr.ip().to_string(), in_addr.port());
        if !socket.has_stream().await {
            info!("Connection accepted");
            socket.set_stream(Option::Some(in_socket)).await;
            debug!("Set stream done");
        }
        else {
            info!("Rejected! Already connected");
        }
    }
}