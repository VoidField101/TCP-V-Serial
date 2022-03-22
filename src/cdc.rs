// Copyright (c) 2022 voidfield101
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT
//
// Modified version of the

use bytes::Buf;

use std::{any::Any, io, sync::Arc};
use tokio::{io::DuplexStream, sync::Mutex, task::JoinHandle};

use async_trait::async_trait;
use usbip::{
    Direction, EndpointAttributes, SetupPacket, UsbEndpoint, UsbInterface, UsbInterfaceHandler,
};

use crate::buffer::SerialBuffer;

#[derive(Clone)]
pub struct UsbCdcAcmStreamHandler {
    buffer: SerialBuffer,
    stream: Arc<Mutex<DuplexStream>>,
}

impl UsbCdcAcmStreamHandler {
    /**
     * Create new UsbCdcAcmStreamHandler
     */
    pub fn new() -> Self {
        let (buffer, stream) = SerialBuffer::new();

        return Self {
            buffer: buffer,
            stream: Arc::new(Mutex::new(stream)),
        };
    }

    pub fn start_buffer(&self) -> (JoinHandle<io::Result<()>>, JoinHandle<io::Result<()>>) {
        self.buffer.start_tasks()
    }

    /**
     * Get the stream to read and write to the serial bus.
     */
    pub fn get_stream(&self) -> Arc<Mutex<DuplexStream>> {
        return self.stream.clone();
    }

    #[allow(unused)]
    pub fn endpoints() -> Vec<UsbEndpoint> {
        vec![
            // state notification
            UsbEndpoint {
                address: 0x81,                                   // IN
                attributes: EndpointAttributes::Interrupt as u8, // Interrupt
                max_packet_size: 0x08,                           // 8 bytes
                interval: 10,
            },
            // bulk in
            UsbEndpoint {
                address: 0x82,                              // IN
                attributes: EndpointAttributes::Bulk as u8, // Bulk
                max_packet_size: 512,                       // 512 bytes
                interval: 0,
            },
            // bulk out
            UsbEndpoint {
                address: 0x02,                              // OUT
                attributes: EndpointAttributes::Bulk as u8, // Bulk
                max_packet_size: 512,                       // 512 bytes
                interval: 0,
            },
        ]
    }
}

#[async_trait]
impl UsbInterfaceHandler for UsbCdcAcmStreamHandler {
    async fn handle_urb(
        &mut self,
        _interface: &UsbInterface,
        ep: UsbEndpoint,
        _setup: SetupPacket,
        req: &[u8],
    ) -> io::Result<Vec<u8>> {
        if ep.attributes == EndpointAttributes::Interrupt as u8 {
            // interrupt
            if let Direction::In = ep.direction() {
                // interrupt in
                return Ok(vec![]);
            }
        } else {
            // bulk
            if let Direction::Out = ep.direction() {
                let bor = self.buffer.get_rx_buffer();
                let mut rx = bor.lock().await;
                rx.extend(req);
                self.buffer.notify_rx();
                return Ok(vec![]);
            } else {
                let mut buffers = [0 as u8; 512];
                let bor = self.buffer.get_tx_buffer();
                let mut tx = bor.lock().await;

                let len = tx.remaining();
                if len > 0 {
                    tx.copy_to_slice(&mut buffers[..len]);
                    return Ok(buffers[..len].to_vec());
                } else {
                    return Ok(vec![]);
                }
            }
        }
        Ok(vec![])
    }

    fn get_class_specific_descriptor(&self) -> Vec<u8> {
        return vec![
            // Header
            0x05, // bFunctionLength
            0x24, // CS_INTERFACE
            0x00, // Header
            0x10, 0x01, // CDC 1.2
            // ACM
            0x04, // bFunctionLength
            0x24, // CS_INTERFACE
            0x02, // ACM
            0x00, // Capabilities
        ];
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}
