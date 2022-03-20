// Copyright (c) 2022 voidfield101
// 
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT


use std::{sync::Arc, io, collections::VecDeque};

use log::info;
use tokio::{io::{WriteHalf, ReadHalf, DuplexStream, AsyncWriteExt, AsyncReadExt}, sync::{Mutex, Notify}};
use bytes::{Buf, BytesMut};

pub type SerialWrite = Arc<Mutex<WriteHalf<DuplexStream>>>;
pub type SerialRead = Arc<Mutex<ReadHalf<DuplexStream>>>;
pub type SerialBufferVec = Arc<Mutex<VecDeque<u8>>>;

/**
 * A buffer for a serial connection.
 * Relays data from the internal receive character buffer to the stream.
 * Also relays data from the stream to the interal transmit buffer.
 */
#[derive(Clone)]
pub struct SerialBuffer {
    tx_pair: (SerialRead, SerialWrite),
    tx_buffer: SerialBufferVec,
    rx_buffer: SerialBufferVec,
    notify: Arc<Notify>
}

impl SerialBuffer {
    
    /**
     * Creates a new serial buffer and a connected stream
     * Also starts the internal tokio tasks to relay data to and from the stream
     */
    pub fn new() -> (Self, DuplexStream) {
        let (stream_a, stream_b) = tokio::io::duplex(1024*8);

        let splits = tokio::io::split(stream_a);

        let tx_rcpair = (
            Arc::new(Mutex::new(splits.0)),
            Arc::new(Mutex::new(splits.1)),
        );

        let nb = Self{
            tx_pair: tx_rcpair,
            tx_buffer: Arc::new(Mutex::new(VecDeque::new())),
            rx_buffer: Arc::new(Mutex::new(VecDeque::new())),
            notify: Arc::new(Notify::new())
        };

        tokio::spawn(nb.clone().read_task());
        tokio::spawn(nb.clone().write_task());

        return ( 
            nb,
            stream_b
        );
    }

    /**
     * Reads data from the stream and sends it to the TX (Transmit buffer)
     */
    async fn read_task(self) -> io::Result<()>{
        let mut reader = self.tx_pair.0.lock().await;
        let mut arr_buf = [0 as u8; 1024];
        loop {
            let num = reader.read(&mut arr_buf).await?;
            let mut buffer = self.tx_buffer.lock().await;
            buffer.extend(&arr_buf[..num]);
        }
    }

    /**
     * Reads data from the RX buffer and sends it to the stream
     * Requires a notification to check the RX buffer for new data.
     */
    async fn write_task(self) -> io::Result<()> {
        let mut writer = self.tx_pair.1.lock().await;
        let mut arr_buf = [0 as u8; 1024];
        loop {
            self.notify.notified().await;
            let len;

            {
                let mut buffer = self.rx_buffer.lock().await;
                len = buffer.remaining();
                if len > 0 {
                    buffer.copy_to_slice(&mut arr_buf[..len]);
                }
            }

            if len > 0 {
                writer.write_all(&arr_buf[..len]).await?;
            }
        }
    }

    /**
     * Notify the buffer that the rx_buffer has new data
     * Required since the deque has no mechanism to async tasks about new data.
     */
    pub fn notify_rx(&self){
        self.notify.notify_waiters();
    }

    /**
     * Get the TX buffer containing the bytes that are queued for transmition.
     * WARNING: Do not write to this buffer!
     */
    //TODO: Create a read only wrapper.
    pub fn get_tx_buffer(&self) -> SerialBufferVec {
        return self.tx_buffer.clone();
    }

    /**
     * Get the RX buffer containing the bytes that were received and queued for relaying to the stream.
     * WARNING: Do not pop data from this buffer!
     */
    //TODO: Create a write only (and non destructive read only) wrapper.
    pub fn get_rx_buffer(&self) -> SerialBufferVec {
        return self.rx_buffer.clone();
    }

}