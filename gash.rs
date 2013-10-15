
extern mod extra;

use std::rt::io::file::{FileInfo, FileWriter};
use std::rt::io::Writer;
use std::rt::io::WriterUtil;

use std::{io, run, uint, os, path, task, libc, rt, to_bytes};
use std::to_bytes::{ToBytes};

fn cdImplementation(user_path: ~str) {

	let mut new_path: ~Path = ~path::Path(os::getcwd().to_str());

	match user_path.char_at(0) {
		'.' => {
				if user_path.len() > 1 && user_path.char_at(1) == '.' {
					new_path= ~path::Path(os::getcwd().dirname());
				}
			}
		'/' => {new_path = ~path::Path(user_path); }
		' '	=> {new_path = ~os::homedir().unwrap(); }
		_	=> {
				let new_path_str: ~str = os::getcwd().to_str() + "/" + user_path;
				new_path = ~path::Path(new_path_str);
			}
	}

	if os::path_exists(new_path) {
		os::change_dir(new_path);
	} else {
		println(fmt!("no such directory: %s", user_path));
	}
}

//Used for debugging
fn printLoc(description: ~str, program: ~str, argv: ~[~str]) {

	
	println("======START=====");
	println(fmt!("%s for program %s", description, program));

	for i in range(0, argv.len()) {
		println(fmt!("%s", argv[i]));
	}
	println("======END=====");
	
}


//In order to support background processes, we will separate the portion
//that matches and then executes the relevant instruction
fn matchProcess(program: ~str, mut argv: ~[~str], user_history: ~[~str], retWriter: bool, useInput: bool, mut inputProcessResult: ~[~str]) -> ~run::Process {

	let mut returnProcess: ~run::Process = ~run::Process::new::(&"", &[], run::ProcessOptions::new());

	match program {
                ~"history" if !argv.contains(&~">") && !argv.contains(&~"|") && !argv.contains(&~"<") => {
                		
           				//If we want to use the output of history as part of a pipe or redirection, then we will 
           				//create a printf process that outputs the entire history as a single line. This allows
           				//for the printf process output to be used as part of a future input. Otherwise we just
           				//output the history to the console.
                		if retWriter {

                			let mut hist_output_string: ~str = ~"";

                			for i in range(0, user_history.len()) {

                				if i < user_history.len() - 1 {
                					hist_output_string = hist_output_string + (i+1).to_str() + ":\t" + user_history[i] +"\n";
                				} else {
                					hist_output_string = hist_output_string + (i+1).to_str() + ":\t" + user_history[i];
                				}
                			}

                			let echoArgv: ~[~str] = ~[hist_output_string];
                			returnProcess = ~run::Process::new("printf", echoArgv, run::ProcessOptions::new());
                		} else {

	                		for i in range(0, user_history.len()) {
	                			println(fmt!("%u:\t%s", (i+1), user_history[i]));
	                		}
	                	}
                	}
                ~"cd"		=> {
                		let mut user_dir: ~str = ~" ";
                		
                		if argv.len() > 0 {
                			user_dir = argv[0];
                		}
                		
                		cdImplementation(user_dir);
                	}

                _           => {
                		
						if argv.contains(&~"|") {

							//We will first check if there are any pipes. If there are pipes, then
							//the left hand side of the first pipe will be executed by recursively calling this function and getting
							//its output, and then recursively calling this function again with the right hand side as the new argv
							//and the output of the previous process, which will be used as its stdin input. It stops when there are no
							//more pipes

                			let mut leftArgv: ~[~str] = argv.clone();
                			leftArgv = leftArgv.slice_to(leftArgv.position_elem(&~"|").unwrap()).to_owned();

                			let mut rightArgv: ~[~str] = argv.clone();
                			rightArgv = rightArgv.slice_from(rightArgv.position_elem(&~"|").unwrap()+1).to_owned();
						
                			let nextProgram = rightArgv.remove(0);
                			let mut nextInputProcess: ~run::Process = matchProcess(program.clone(), leftArgv.clone(), user_history.clone(), true, useInput, inputProcessResult.clone());
                			let mut nextInputArray: ~[~str] = nextInputProcess.output().read_lines();
                			matchProcess(nextProgram.clone(), rightArgv.clone(), user_history.clone(), false, true, nextInputArray.clone());

                		} else if argv.contains(&~">") {

                			//The > command (for this shell) is found near the end of a command. 

                			let mut leftArgv: ~[~str] = argv.clone();
                			leftArgv = leftArgv.slice_to(leftArgv.position_elem(&~">").unwrap()).to_owned();

                			//We will recursively call this function on the left hand side of the '>'
                			let mut my_proc: ~run::Process = matchProcess(program.clone(), leftArgv.clone(), user_history.clone(), true, false, ~[]);

                			//If we are using the stdout from a previous command (for the case of pipes)
                			if useInput {

                				let procInWriter = my_proc.input();

        						for i in range(0, inputProcessResult.len()) {
        							procInWriter.write_line(inputProcessResult[i]);
        						}
        						my_proc.close_input();
        					}

                			let mut filePath: ~Path = ~path::Path(argv[argv.position_elem(&~">").unwrap()+1]);

                			//We must use the std::rt::io filewriter since it is non-blocking. The regular std::io will not return
                			//control to other threads until it finishes writing, which can be problematic when running 
                			//non-halting background tasks such as ping google.com > output.txt &
                			let mut myWriter: ~rt::io::file::FileWriter = ~filePath.open_writer(rt::io::OpenOrCreate).unwrap();

                			//We must use write() instead of the shorter write_line() since I couldn't get std::rt::io to cooperate
                			//Also, we must write one line at a time (reading from the process output one line at a time) in order
                			//for writing to a file to work with non-halting processes. For example, if we had a 'ping' command
                			//and used process.output().read_lines(), it would just return an empty array. By using process.output.eof()
                			//in combination with process.output().read_line(), we get each line as it outputs, and thus we can now do
                			//such commands as ping google.com > pingout.txt and the text file will update as new content is generated.
                			while !my_proc.output().eof() {

    							let mut lineToWrite: ~str = my_proc.output().read_line() + "\r\n";
    							if !my_proc.output().eof() {
	    							let mut byteArr: ~[u8] = lineToWrite.to_bytes(false);
	    							let mut arrToWrite: ~[u8] = byteArr.clone();
	    							arrToWrite = arrToWrite.slice_to(arrToWrite.len()-1).to_owned();
	    							myWriter.write(arrToWrite);
    							}
                							
    						}

                		} else if argv.contains(&~"<") {

                			let mut leftArgv: ~[~str] = argv.clone();
                			leftArgv = leftArgv.slice_to(leftArgv.position_elem(&~"<").unwrap()).to_owned();
                			leftArgv.push(argv[argv.position_elem(&~"<").unwrap() + 1]);
                			
                			
                			if retWriter {
                				//If we want to return the process so that it can be piped to the next command
	                			let mut proc_in_ops: run::ProcessOptions = run::ProcessOptions::new();
	                			let mut my_in_proc: ~run::Process = ~run::Process::new(program, leftArgv, proc_in_ops);

	                			returnProcess = my_in_proc;

	                		} else {
	                			//Else just run normally and output to the shell
	                			run::process_status(program, leftArgv);
	                		}

                		} else if retWriter {

                			//If our command's argv has no input or output redirection, but we want to use its output
                			//for the next command (in the case for '>' and '|')
                			let mut my_proc: ~run::Process = ~run::Process::new(program, argv, run::ProcessOptions::new());
                			

                			//If the command uses as input the output of a previous command
                			if useInput {

                				let myWriter = my_proc.input();
        						for i in range(0, inputProcessResult.len()) {
        							myWriter.write_line(inputProcessResult[i]);
        						}

        						my_proc.close_input();

                			}

                			returnProcess = my_proc;

                		} else {
                			//Finally, we reach the case where a command has no input redirection and its 
                			//output is being written to the standard output (console)

                			//If it is using as input the output of a previous command
                			if useInput {

                				let mut my_proc: ~run::Process = ~run::Process::new(program, argv, run::ProcessOptions::new());
        						let myWriter = my_proc.input();

        						
        						for i in range(0, inputProcessResult.len()) {
        							myWriter.write_line(inputProcessResult[i]);
        						}

        						my_proc.close_input();

        						let mut proc_results: ~[~str] = my_proc.output().read_lines();

        						for i in range(0, proc_results.len()) {
        							println(fmt!("%s", proc_results[i]));
        						}

        					} else {
        						//Else, just run the command normally
                				run::process_status(program, argv);
                			}	
                			
                		}
                	}
	}
	
	returnProcess
}

fn main() {
    static CMD_PROMPT: &'static str = "gash > ";

    let mut user_history: ~[~str] = ~[];

    loop {
        print(CMD_PROMPT);
        let mut line = io::stdin().read_line();
        debug!(fmt!("line: %?", line));
        let mut argv: ~[~str] = line.split_iter(' ').filter(|&x| x != "")
                                 .map(|x| x.to_owned()).collect();
        debug!(fmt!("argv %?", argv));
        
        if argv.len() > 0 {


            let mut program: ~str = argv.remove(0);

            //We will only check to see if the user wants to exit or enter
            //some special command that should be handled before executing a 
            //process.
            //Otherwise it will be handled below
            
            match program {
            	~"exit"	=> {break;}
            	~"!!"	=> {
            			line = user_history.pop();
            			//Add it back so that the history mimicks bash
            			user_history.push(line.clone());
            			argv = line.split_iter(' ').filter(|&x| x != "")
                                 .map(|x| x.to_owned()).collect();
                        program = argv.remove(0);
            		}
            	_		=> {}

            }

            //After, push it to the history array
            user_history.push(line);

            //Check to see if the user wants to run it as a background process
            //in which case we will spawn it as an independent task
            if argv.len() > 0 && argv[argv.len()-1] == ~"&" {

            	//println("background");
            	let arg_len:uint = argv.len().clone();
            	argv.remove(arg_len-1);

            	let program_bg: ~str = program.clone();
	            let argv_bg: ~[~str] = argv.clone();
	            let user_history_bg: ~[~str] = user_history.clone();

	            do task::spawn_unlinked {
					matchProcess(program_bg.clone(), argv_bg.clone(), user_history_bg.clone(), false, false, ~[]);
            	}

            } else { //Else, just run normally
            	matchProcess(program.clone(), argv.clone(), user_history.clone(), false, false, ~[]);
            }
        }
    }
}
