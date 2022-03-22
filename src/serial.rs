// Copyright (c) 2022 voidfield101
// 
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use std::{sync::Arc, net::SocketAddr, io, fmt::Error, time::Duration};

use log::info;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream};
use usbip::{UsbDevice, ClassCode, cdc::{CDC_ACM_SUBCLASS, UsbCdcAcmHandler}, UsbInterfaceHandler, UsbIpServer};

use crate::{cdc, relay::{StreamPluginRelay, start_relay}};

pub type UsbHandlerBox = Arc<tokio::sync::Mutex<Box<dyn UsbInterfaceHandler + Send>>>;

/**
 * Create a new UsbDevice for virtual serial port.
 * Returns both the device and the handler (handler required for reading and writing)
 */
pub async fn new_device(index:u32) -> io::Result<(UsbDevice, UsbHandlerBox)> {
    let handler = Arc::new(tokio::sync::Mutex::new(Box::new(cdc::UsbCdcAcmStreamHandler::new()) as Box<dyn usbip::UsbInterfaceHandler + Send>));
    
    let mut device = UsbDevice::new(index).with_interface(
        ClassCode::CDC as u8, 
        CDC_ACM_SUBCLASS,
        0x00,
        "FakeSerial",
        UsbCdcAcmHandler::endpoints(),
        handler.clone()).await;

    device.string_pool.insert(device.string_manufacturer, "V-Serial (Fake)".to_string());
    device.string_pool.insert(device.string_product, "TCP V-Serial Adapter".to_string());

    return Ok((device, handler));
}

/**
 * Runs the UsbIp server and starts relaying information between the tcp socket and usb handler.
 * UsbHandlerBox needs to contain a UsbCdcAcmStreamHandler, task will panic otherwise
 */
pub async fn run_usbip(device: (UsbDevice, UsbHandlerBox), addr: SocketAddr, tcpstream: StreamPluginRelay<TcpStream>) -> io::Result<()>{
    //Might extend to multiple devices later
    let server = UsbIpServer::new_simulated(vec![device.0]);

    let mut stream;
    let acm_handler;
    
    if let Some(acm) = device.1.lock().await.as_any().downcast_mut::<cdc::UsbCdcAcmStreamHandler>() {
        stream = acm.get_stream();
        acm_handler = acm.start_buffer();
    }
    else {
        //TODO: Make this an error instead of a panic
        panic!("Cast failed");
    }


    tokio::spawn(usbip::server(addr, server));

    let mut rx = stream.lock().await;
    start_relay(tcpstream, &mut *rx).await?;

    Ok(())   
}





