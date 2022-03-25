# TCP-V-Serial

This is a small Rust program which takes many parts from the usbip crate [example](https://github.com/jiegec/usbip/blob/master/examples/cdc_acm_serial.rs)

It simply relays data between a raw tcp socket (can act as server or client) 


## How to use

1. You need to install usbip on your system. Packages are usually in the repos on Linux Distributions

2. Make sure you have [Rust](https://www.rust-lang.org/) installed.

3. Clone and build this project
```
$ git clone git@github.com:voidfield101/TCP-V-Serial.git
$ cd TCP-V-Serial
$ cargo build --release
```

4. Run the virtual serial server. This example sets up a TCP server on 8888 and a USB/IP Server on 9999
```
$ RUST_LOG="info" ./target/release/tcpvserial -l -a 127.0.0.1:8888 -p 9999
```

5. Connect bind the USB/IP device
```
$ sudo usbip --tcp-port 9999 attach -r localhost -b 0
```

6. Done. You can now connect to the TCP/Telnet server and use the serial device (example using netcat, cat and echo):
```
$ netcat localhost 8888
Hello World
Hi
Hi
```

```
$ echo "Hello World" > /dev/ttyACM0
$ cat /dev/ttyACM0
Hi
```
