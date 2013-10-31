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
use std::cmp::*;
use std::str::*;
use std::option::Option;
use extra::priority_queue::*;
use std::hashmap;
//use std::rt::io::pipe;
//use std::rt::io::buffered::*;
//use std::rt::io::support::PathLike;
//use std::path::Path;
//use std::rt::io::file::{FileInfo, FileReader};
//use std::tuple;
//use std::libc;
mod gashk;

static PORT:    int = 4414;
static IP: &'static str = "127.0.0.1";
static visitor_count: uint = 0;
//static map: hashmap::HashMap<~str, ~str> = hashmap::HashMap::new();

fn ip_parser(ip : ~str)->bool{
	println("Inside function: "+ip);
	let ip_str = ip.to_str();
	let ip_split: ~[&str]=ip_str.split_str_iter(".").collect();
	if((ip_split[0]=="127" && ip_split[1]=="0") || (ip_split[0]=="128" && ip_split[1]=="143") || (ip_split[0]=="137" && ip_split[1]=="54")){
		return true;
	}
        else{
		return false;	
	}
}//ip_parser

struct sched_msg {
     stream: Option<std::rt::io::net::tcp::TcpStream>,
     filepath: ~std::path::PosixPath
}
impl Ord for sched_msg {
	fn lt(&self, other:&sched_msg)->bool{
		let temp =other.stream.get_ref();
		let temp2 =self.stream.get_ref();
		unsafe{
			let ref ostreamip=std::cast::transmute_mut(temp).peer_name();
			let ref mystreamip=std::cast::transmute_mut(temp2).peer_name();
			if(!ip_parser(mystreamip.to_str()) && ip_parser(ostreamip.to_str())){
			false;
			}
			if(ip_parser(mystreamip.to_str()) && !ip_parser(ostreamip.to_str())){
			true;
			}
			if( ip_parser(mystreamip.to_str()) && ip_parser(ostreamip.to_str())){
			println("filesize effect");
			return (self.filepath.get_size().unwrap())<(other.filepath.get_size().unwrap());
			}//if
		       }//unsafe
		return false;
	}//fn
}//ord

fn main() {
    let req_vec: ~[sched_msg] = ~[];
    let req_queue: PriorityQueue<sched_msg> = PriorityQueue::from_vec(req_vec);
    let shared_req_vec = arc::RWArc::new(req_queue);
    let add_vec = shared_req_vec.clone();
    let take_vec = shared_req_vec.clone();
    let (port, chan) = stream();
    let chan = SharedChan::new(chan);
    let count_arc= arc::RWArc::new(visitor_count);
    let map: hashmap::HashMap<~str, ~str> = hashmap::HashMap::new();
    let map_arc=arc::RWArc::new(map);	
    //let mut map: hashmap::HashMap<~str, ~str> = hashmap::HashMap::new();
	
    // dequeue file requests, and send responses.
    do spawn {
        let (sm_port, sm_chan) = stream();

			loop {
		        port.recv(); 
		        do take_vec.write |vec| {
		            if ((*vec).len() > 0) {
println(fmt!("queue size before popping: %u", (*vec).len()));
		                let tf_opt: Option<sched_msg> = Some((*vec).pop());
					    println(fmt!("queue size after popping: %u", (*vec).len()));
						let tf_cell=Cell::new(tf_opt.unwrap());
		                sm_chan.send(tf_cell.take()); // send the request to send-response-task to serve.
					}//capacity
		        }//end write
	               	
				let mut tf: sched_msg = sm_port.recv(); // wait for the dequeued request to handle
                match io::read_whole_file(tf.filepath) { // killed if file size is larger than memory size.
                    Ok(file_data) => {
		//caching	
			let atime=tf.filepath.get_atime();
			let accessed =	match atime{
					Some(access_time)=>{ 
					println(fmt!("access time %?", access_time.first()));
					access_time.first();					
					},
					None()=>{
					}
				};//match

			let mtime=tf.filepath.get_mtime();
			let modified =	match mtime{
					Some(modified_time)=>{ 
					println(fmt!("modified time %?", modified_time.first()));
					modified_time.first();
					},
					None()=>{
					}
				};//match

			//println(fmt!("access time %?", access_split[0]));
			println(fmt!("created time %?", tf.filepath.get_ctime().unwrap()));
			//let mtime_str=tf.filepath.get_mtime().unwrap().to_str();
			//let modified_str = mtime_str.slice(1, mtime_str.len()-1);
			//println(fmt!("modified time %?", tf.filepath.get_mtime().unwrap()));
	                //println(fmt!("begin serving file [%?]", tf.filepath));
          if(tf.filepath.to_str().contains("shtml")||modified<accessed){
			match io::read_whole_file_str(tf.filepath){
			Ok(file_strings)=>{			
					
//		let file_cell = Cell::new(file_strings);
		//start gash SSI stuff here
		
			println("HTML file with SSI detected");
			println(fmt!("serve file: %?", tf.filepath));

			if(file_strings.contains("<!--#exec cmd")){
				let cmd:~[&str]=file_strings.split_str_iter("\"").collect();//to be refined
				println("gash command: " +cmd[1].to_owned());
				let mut response:~str=~"HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n"+file_strings; 
			//	let mpipe = os::pipe();
				let popt = ::gashk::handle_cmdline(cmd[1]);
				let out_bytes=popt.unwrap().output;
				let s = str::from_utf8(out_bytes);
				//let fread = std::io::BytesWriter
		response = replace(response, "<!--#exec cmd=\""+cmd[1]+"\" -->","<p>"+s+"</p>");		
	println("This is the response: "+response);		
	tf.stream.write(response.as_bytes());
					}//end if contains
			let edit_map = map_arc.clone();
			let ref filepath = tf.filepath;
			let mut file_content=~"";
			let file_path = &Path(filepath.to_str());
			println(fmt!("%?", file_strings));
			file_content=file_strings.clone();	
			let file_cell = Cell::new(file_content);
			do edit_map.write |file_map|{
			file_map.find_or_insert(tf.filepath.to_str(),file_cell.take()); //need buf or something to add contents of file as value in map
			}

			println(fmt!("%?", file_strings));

		},//Ok
		Err(err)=>{println(err);}
			}//match
  			}//if modified or shtml, open as string
            //  tf.stream.write("HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream; charset=UTF-8\r\n\r\n".as_bytes());
              //          tf.stream.write(file_data);
                        println(fmt!("finish file [%?]", tf.filepath));
			},//ok
                        Err(err) => {println(err);}
                    }//match on too big?


			}//loop


        
        
  		
}//spawn

    let ip = match FromStr::from_str(IP) { 
			Some(ip) => ip,                                        				None => { println(fmt!("Error: Invalid IP address <%s>", IP));
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
	let write_count = count_arc.clone();
	
	do spawn {
           do write_count.write |count| {
              *count=*count+1;
	    }

		let mut dstream = stream.take();
		let mut buf = [0, ..500];
		dstream.read(buf);
		let clostream = Cell::new(dstream);
		let request_str = str::from_utf8(buf);        
		let req_group : ~[&str]= request_str.splitn_iter(' ', 3).collect();
            if req_group.len() > 2 {
                let path = req_group[1];
                //println(fmt!("Request for path: \n%?", path));               
                let file_path = ~os::getcwd().push(path.replace("/../", ""));
                if !os::path_exists(file_path) || os::path_is_dir(file_path) {
                    //println(fmt!("Request received:\n%s", request_str));
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

                    clostream.take().write(response.as_bytes());
                }//end file req if
                else {
                    // Requests scheduling
		        let ostream = clostream.take().unwrap();
			let msg: sched_msg = sched_msg{stream:Some(ostream),filepath: 				file_path.clone()};//
		    
	             	     let (sm_port, sm_chan) = std::comm::stream();
	                    sm_chan.send(msg);
				//if(ip_parser(matchstream)){
                               do child_add_vec.write |vec| {
                                    let msg = sm_port.recv();
                                    (*vec).push(msg); // enqueue new request.
                                        }//end do
	
                    		child_chan.send(""); //notify the new arriving request.
        	}//end file req else          
		 }//end array long enough
            println!("connection terminates")
        }//end spawn

    }//end for stream : acceptor
}//end main
