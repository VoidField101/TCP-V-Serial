// Copyright (c) 2022 voidfield101
// 
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use std::{sync::Arc, net::SocketAddr, io, fmt::Error, time::Duration};

use log::info;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use usbip::{UsbDevice, ClassCode, cdc::{CDC_ACM_SUBCLASS, UsbCdcAcmHandler}, UsbInterfaceHandler, UsbIpServer};

use crate::cdc;

pub type UsbHandlerBox = Arc<tokio::sync::Mutex<Box<dyn UsbInterfaceHandler + Send>>>;

pub async fn new_device(index:u32) -> io::Result<(UsbDevice, UsbHandlerBox)> {
    let handler = Arc::new(tokio::sync::Mutex::new(Box::new(cdc::UsbCdcAcmStreamHandler::new()?) as Box<dyn usbip::UsbInterfaceHandler + Send>));
    
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


pub async fn run_usbip(device: (UsbDevice, UsbHandlerBox), addr: SocketAddr) -> io::Result<()>{
    //Might extend to multiple devices later
    let server = UsbIpServer::new_simulated(vec![device.0]);

    let mut stream;
    
    if let Some(acm) = device.1.lock().await.as_any().downcast_mut::<cdc::UsbCdcAcmStreamHandler>() {
        stream = acm.get_stream();
    }
    else {
        panic!("Cast failed");
    }


    tokio::spawn(usbip::server(addr, server));
    loop {
        let mut rx = stream.lock().await;
        let mut str = String::new();
        let mut buffer = [0 as u8;1024];
        let read = rx.read(&mut buffer).await?;
        if read > 0 {
            info!("[R] {}",  String::from_utf8_lossy(&buffer[..read]));
        }
        /*tokio::time::sleep(Duration::from_secs(1)).await;
        let mut tx = stream.lock().await;
        tx.write_all("HI\n".as_bytes()).await?;*/
    }
}





