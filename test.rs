//
// zhtta.rs
//
// Running on Rust 0.8
//
// Starting code for PS3
// 
// Note: it would be very unwise to run this server on a machine that is
// on the Internet and contains any sensitive files!
//
// University of Virginia - cs4414 Fall 2013
// Weilin Xu and David Evans
// Version 0.3

extern mod extra;

use std::rt::io::*;
use std::rt::io::net::ip::SocketAddr;
use std::io::println;
use std::cell::Cell;
use std::{os, str, io};
use extra::arc;
use std::comm::*;

static PORT:    int = 4414;
static IP: &'static str = "127.0.0.1";
static visitor_count: uint = 0;

fn ip_parser(ip : ~str)->bool{
	
	let ip_str = ip.to_str();
	let mut ip_split: ~[&str]=ip_str.split_str_iter(".").collect();
	if((ip_split[0]=="127" && ip_split[1]=="0") || (ip_split[0]=="128" && ip_split[1]=="143") || (ip_split[0]=="137" && ip_split[1]=="54")){

		//println(fmt!("firstip %?",ip_split[0]));  
		return true;
	}
        else{
		return false;	
	}
}

struct sched_msg {
    stream: Option<std::rt::io::net::tcp::TcpStream>,
    filepath: ~std::path::PosixPath
}

fn main() {
    let req_vec: ~[sched_msg] = ~[];
    let shared_req_vec = arc::RWArc::new(req_vec);
    let add_vec = shared_req_vec.clone();
    let take_vec = shared_req_vec.clone();

        let req_pvec: ~[sched_msg] = ~[];
    let shared_req_pvec = arc::RWArc::new(req_pvec);
    let add_pvec = shared_req_pvec.clone();
    let take_pvec = shared_req_pvec.clone();
    
    let (port, chan) = stream();
    let chan = SharedChan::new(chan);
    let count_arc= arc::RWArc::new(visitor_count);
    // dequeue file requests, and send responses.
    // FIFO
    do spawn {
        let (sm_port, sm_chan) = stream();
        
        // a task for sending responses.
        do spawn {
            loop {
                let mut tf: sched_msg = sm_port.recv(); // wait for the dequeued request to handle
                match io::read_whole_file(tf.filepath) { // killed if file size is larger than memory size.
                    Ok(file_data) => {
                        println(fmt!("begin serving file [%?]", tf.filepath));
                        // A web server should always reply a HTTP header for any legal HTTP request.
                        tf.stream.write("HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream; charset=UTF-8\r\n\r\n".as_bytes());
                        tf.stream.write(file_data);
                        println(fmt!("finish file [%?]", tf.filepath));
                    }
                    Err(err) => {
                        println(err);
                    }
                }
            }
        }
        
        loop {
            port.recv(); // wait for arrving notification
                        if(do take_pvec.read |vec| {vec.len()} > 0){
                                do take_pvec.write |pvec| {
                            if ((*pvec).len() > 0) {
                                // LIFO didn't make sense in service scheduling, so we modify it as FIFO by using shift_opt() rather than pop().
                                let tf_opt: Option<sched_msg> = (*pvec).shift_opt();
                                let tf = tf_opt.unwrap();
                                println(fmt!("shift from queue, size: %ud", (*pvec).len()));
                                sm_chan.send(tf); // send the request to send-response-task to serve.
                            }        
                                }
                        
                        } else {

                        
            do take_vec.write |vec| {
                            if ((*vec).len() > 0) {
                                // LIFO didn't make sense in service scheduling, so we modify it as FIFO by using shift_opt() rather than pop().
                                let tf_opt: Option<sched_msg> = (*vec).shift_opt();
                                let tf = tf_opt.unwrap();
                                println(fmt!("shift from queue, size: %ud", (*vec).len()));
                                sm_chan.send(tf); // send the request to send-response-task to serve.
                            }
                        }
                        }
        }
    }

    let ip = match FromStr::from_str(IP) { Some(ip) => ip, 
                                           None => { println(fmt!("Error: Invalid IP address <%s>", IP));
                                                     return;},
                                         };
                                         
    let socket = net::tcp::TcpListener::bind(SocketAddr {ip: ip, port: PORT as u16});
    
    println(fmt!("Listening on %s:%d ...", ip.to_str(), PORT));

                                    
    let mut acceptor = socket.listen().unwrap();
    
    for stream in acceptor.incoming() {
        let stream = Cell::new(stream);
        
        // Start a new task to handle the each connection
        let child_chan = chan.clone();
        let child_add_vec = add_vec.clone();
                let child_add_pvec = add_pvec.clone();

                let write_count = count_arc.clone();
        do spawn {
                    do write_count.write |count| {
                                *count=*count+1;
                    }
            /*unsafe {
                visitor_count += 1;
            }*/
            
            let mut stream = stream.take();
            let mut buf = [0, ..500];
            stream.read(buf);
            let request_str = str::from_utf8(buf);
            
            let req_group : ~[&str]= request_str.splitn_iter(' ', 3).collect();
            if req_group.len() > 2 {
                let path = req_group[1];
                println(fmt!("Request for path: \n%?", path));
                
                let file_path = ~os::getcwd().push(path.replace("/../", ""));
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
                         </body></html>\r\n", do write_count.read |count|{*count});

                      ip_parser(ip.to_str());
											
					
                    stream.write(response.as_bytes());
                }
                else {
                    // Requests scheduling
                    let msg: sched_msg = sched_msg{stream: stream, filepath: file_path.clone()};
                    let (sm_port, sm_chan) = std::comm::stream();
                    sm_chan.send(msg);

                                        if(ip_parser(ip.to_str())){
                                                do child_add_pvec.write |pvec| {
                                    let msg = sm_port.recv();
                                    (*pvec).push(msg); // enqueue new request.
                                    println("add to priority queue");
                                }
                                        } else {

                                do child_add_vec.write |vec| {
                                    let msg = sm_port.recv();
                                    (*vec).push(msg); // enqueue new request.
                                    println("add to queue");
                                }
                                        }
                    child_chan.send(""); //notify the new arriving request.
                    println(fmt!("get file request: %?", file_path));
                }
            }
            println!("connection terminates")
        }
    }
}
