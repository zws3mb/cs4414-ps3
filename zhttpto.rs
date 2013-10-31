//
// zhttpto.rs
//
// Reference solution for PS1
// Running on Rust 0.8
//
// Note that this code has serious security risks!  You should not run it 
// on any system with access to sensitive files.
//
// Special thanks to Kiet Tran for providing code we incorporated into this.
// 
// University of Virginia - cs4414 Fall 2013
// Weilin Xu and David Evans
// Version 0.2

/*
Total: connections 1000 requests 1000 replies 0 test-duration 46.928 s

Connection rate: 21.3 conn/s (46.9 ms/conn, <=919 concurrent connections)
Connection time [ms]: min 0.0 avg 0.0 max 0.0 median 0.0 stddev 0.0
Connection time [ms]: connect 2517.3
Connection length [replies/conn]: 0.000

Request rate: 21.3 req/s (46.9 ms/req)
Request size [B]: 68.0

Reply rate [replies/s]: min 0.0 avg 0.0 max 0.0 stddev 0.0 (9 samples)
Reply time [ms]: response 18416.4 transfer 0.0
Reply size [B]: header 0.0 content 0.0 footer 0.0 (total 0.0)
Reply status: 1xx=0 2xx=0 3xx=0 4xx=0 5xx=0

CPU time [s]: user 0.10 system 8.10 (user 0.2% system 17.3% total 17.5%)
Net I/O: 1.4 KB/s (0.0*10^6 bps)

Errors: total 1000 client-timo 168 socket-timo 0 connrefused 0 connreset 832
Errors: fd-unavail 0 addrunavail 0 ftab-full 0 other 0



Total: connections 1000 requests 1000 replies 14 test-duration 37.496 s

Connection rate: 26.7 conn/s (37.5 ms/conn, <=993 concurrent connections)
Connection time [ms]: min 244.4 avg 10416.1 max 30243.6 median 4135.5 stddev 10626.1
Connection time [ms]: connect 95.9
Connection length [replies/conn]: 1.000

Request rate: 26.7 req/s (37.5 ms/req)
Request size [B]: 68.0

Reply rate [replies/s]: min 0.2 avg 0.4 max 1.4 stddev 0.4 (7 samples)
Reply time [ms]: response 8986.5 transfer 1428.6
Reply size [B]: header 74.0 content 56.0 footer 0.0 (total 130.0)
Reply status: 1xx=0 2xx=14 3xx=0 4xx=0 5xx=0

CPU time [s]: user 0.09 system 9.76 (user 0.2% system 26.0% total 26.3%)
Net I/O: 1.8 KB/s (0.0*10^6 bps)

Errors: total 986 client-timo 986 socket-timo 0 connrefused 0 connreset 0
Errors: fd-unavail 0 addrunavail 0 ftab-full 0 other 0

*/

extern mod extra;

use std::rt::io::*;
use std::rt::io::net::ip::{SocketAddr, Ipv4Addr};
use std::io::println;
use std::cell::Cell;
use std::task;
use std::{os, str, io};

static PORT:    int = 4414;
static IPV4_LOOPBACK: &'static str = "127.0.0.1";
static mut visitor_count: uint = 0;


fn main() {
    let socket = net::tcp::TcpListener::bind(SocketAddr {ip: Ipv4Addr(127,0,0,1), port: PORT as u16});
    
    println(fmt!("Listening on tcp port %d ...", PORT));
    let mut acceptor = socket.listen().unwrap();
    
    // we can limit the incoming connection count.
    //for stream in acceptor.incoming().take(10 as uint) {
    for stream in acceptor.incoming() {
        println!("Saw connection!");
        let stream = Cell::new(stream);
        // Start a task to handle the connection
        do task::spawn {
            unsafe {
                visitor_count += 1;
            }
            let mut stream = stream.take();
            let mut buf = [0, ..500];
            stream.read(buf);
            let request_str = str::from_utf8(buf);
            
            let req_group : ~[&str]= request_str.splitn_iter(' ', 3).collect();
            if req_group.len() > 2 {
                let path = req_group[1];
                println(fmt!("Request for path: \n%?", path));
                
                let file_path = &os::getcwd().push(path.replace("/../", ""));
                if !os::path_exists(file_path) || os::path_is_dir(file_path) {
                    println(fmt!("Request received:\n%s", request_str));
                    let response: ~str = fmt!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n
                         <doctype !html><html><head><title>Hello, Rust!</title>
                         <style>body { background-color: #111; color: #FFEEAA }
                                h1 { font-size:2cm; text-align: center; color: black; text-shadow: 0 0 4mm red}
                                h2 { font-size:2cm; text-align: center; color: black; text-shadow: 0 0 4mm green}
                         </style></head>
                         <body>
                         <h1>Greetings, Krusty!</h1>
                         <h2>Visitor count: %u</h2>
                         </body></html>\r\n", unsafe{visitor_count});

                    stream.write(response.as_bytes());
                }
                else {
                    println(fmt!("serve file: %?", file_path));
                    match io::read_whole_file(file_path) {
                        Ok(file_data) => {
                            stream.write(file_data);
                        }
                        Err(err) => {
                            println(err);
                        }
                    }
                }
            }
            println!("connection terminates")
        }
    }
}
