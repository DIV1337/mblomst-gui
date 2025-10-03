use std::net::{TcpListener, TcpStream};
use std::io::{BufReader, BufRead, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::panic;
use std::time::Duration;

use crate::connection_state::ConnectionState;

pub fn start_server(port: u16, connection_state: Arc<Mutex<ConnectionState>>) { // starts the player called "server"
    let address = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(address).expect("Failed to bind");


    for stream in listener.incoming().take(1) {
        match stream {
            Ok(stream) => {
                println!("Client connected"); // wait for a "client" to connect

                {
                    let mut state = connection_state.lock().unwrap();
                    state.connected = true;
                    state.stream = Some(stream.try_clone().unwrap());
                    state.turn = 1; 
                }

                let state_clone = Arc::clone(&connection_state);
                thread::spawn(move || {
                    let result = panic::catch_unwind(|| {
                        handle_connection(stream, state_clone);
                    });
                    if let Err(e) = result {
                        println!("[Host] panic in thread: {:?}", e);
                    }
                });
            }
            Err(e) => {
                println!("Connection failed: {}", e);
            }
        }
    }
}



pub fn start_client(addr: &str, connection_state: Arc<Mutex<ConnectionState>>) { // starts the player called "client"
    match TcpStream::connect(addr) {
        Ok(stream) => {

            {
                let mut state = connection_state.lock().unwrap();
                state.connected = true;
                state.stream = Some(stream.try_clone().unwrap());
            }

            let state_for_thread = Arc::clone(&connection_state);
            thread::spawn(move || {
                let result = panic::catch_unwind(|| {
                    handle_connection(stream, state_for_thread);
                });
                if let Err(e) = result {
                    println!("[Client] panic in thread: {:?}", e);
                }
            });
        }
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
}

pub fn handle_connection(mut stream: TcpStream, state: Arc<Mutex<ConnectionState>>) {
    stream.set_read_timeout(None); // makes read_line not time out while waiting for a move

    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let is_host = state.lock().unwrap().is_host;
    let role = if is_host { "Host" } else { "Client" };

    let outgoing_rx = {
        let state = state.lock().unwrap();
        state.outgoing_rx.clone()
    };

    let incoming_tx = {
        let state = state.lock().unwrap();
        state.incoming_tx.clone()
    };

    loop {

        // called for writing messages
        while let Ok(msg) = outgoing_rx.try_recv() {
            if let Err(e) = writeln!(stream, "{}", msg) {
                println!("[{}] Failed to write to stream: {}", role, e);
                break;
            }
            if let Err(e) = stream.flush() {
                println!("[{}] Failed to flush stream: {}", role, e);
                break;
            }
        }

        // called for reading messages
        let should_listen = { // only listens if it is the opponents turn
            let state = state.lock().unwrap();
            (state.turn % 2 == 1) != state.is_host
        };

        if should_listen {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => {
                    println!("[{}] Connection closed by peer", role);
                    break;
                }
                Ok(n) => { // if there is something to read
                    let msg = line.trim().to_string();
                    if !msg.is_empty() {
                        if let Err(e) = incoming_tx.send(msg.clone()) { //tries to send the message to incoming
                            println!("[{}] Failed to push to incoming_tx: {}", role, e);
                        } else {
                        }
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // if timeout, in case of no data available, continue loop
                }
                Err(e) => {
                    println!("[{}] Error reading from stream: {}", role, e);
                    break;
                }
            }
        }

        thread::sleep(Duration::from_millis(20)); // gives a bit of time for the loop to catch up and minimizes risk for errors with reading
    }
}

