use std::net::{TcpListener, TcpStream};
use std::io::{BufReader, BufRead, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::panic;
use std::time::Duration;

use crate::connection_state::ConnectionState;

pub fn send_message(conn: &Arc<Mutex<ConnectionState>>, msg: &str) {
    if let Some(stream) = &mut conn.lock().unwrap().stream {
        let _ = stream.write_all(format!("{}\n", msg).as_bytes());
        let _ = stream.flush();
    }
}

pub fn start_server(port: u16, connection_state: Arc<Mutex<ConnectionState>>) {
    let address = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(address).expect("Failed to bind");

    println!("Server listening on port {}", port);

    for stream in listener.incoming().take(1) {
        match stream {
            Ok(stream) => {
                println!("Client connected");

                {
                    let mut state = connection_state.lock().unwrap();
                    state.connected = true;
                    state.stream = Some(stream.try_clone().unwrap());
                    state.turn = 1; // 👈 Host börjar med turen
                }

                let state_clone = Arc::clone(&connection_state);
                thread::spawn(move || {
                    println!("[Host] Spawning handle_connection thread...");
                    let result = panic::catch_unwind(|| {
                        handle_connection(stream, state_clone);
                    });
                    if let Err(e) = result {
                        println!("[Host] PANIK i tråden: {:?}", e);
                    }
                });
            }
            Err(e) => {
                println!("Connection failed: {}", e);
            }
        }
    }
}



pub fn start_client(addr: &str, connection_state: Arc<Mutex<ConnectionState>>) {
    match TcpStream::connect(addr) {
        Ok(stream) => {
            println!("Connected to server at {}", addr);

            {
                let mut state = connection_state.lock().unwrap();
                state.connected = true;
                state.stream = Some(stream.try_clone().unwrap());
            }

            let state_for_thread = Arc::clone(&connection_state);
            thread::spawn(move || {
                println!("[Client] Spawning handle_connection thread...");
                let result = panic::catch_unwind(|| {
                    handle_connection(stream, state_for_thread);
                });
                if let Err(e) = result {
                    println!("[Client] PANIK i tråden: {:?}", e);
                }
            });
        }
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
}

pub fn handle_connection(mut stream: TcpStream, state: Arc<Mutex<ConnectionState>>) {
    stream.set_read_timeout(None); 

    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let is_host = state.lock().unwrap().is_host;
    let role = if is_host { "Host" } else { "Client" };
    println!("[{}] Started handle_connection thread", role);

    let outgoing_rx = {
        let state = state.lock().unwrap();
        state.outgoing_rx.clone()
    };

    let incoming_tx = {
        let state = state.lock().unwrap();
        state.incoming_tx.clone()
    };

    loop {
        println!("[{}] Loop still alive", role);

        // --- WRITE outgoing messages ---
        while let Ok(msg) = outgoing_rx.try_recv() {
            println!("[{}] Sending message: {}", role, msg);
            if let Err(e) = writeln!(stream, "{}", msg) {
                println!("[{}] Failed to write to stream: {}", role, e);
                break;
            }
            if let Err(e) = stream.flush() {
                println!("[{}] Failed to flush stream: {}", role, e);
                break;
            }
            println!("[{}] Message sent successfully", role);
        }

        // --- READ incoming messages only when it's opponent's turn ---
        let should_listen = {
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
                Ok(n) => {
                    println!("[{}] Read {} bytes: {:?}", role, n, line);
                    let msg = line.trim().to_string();
                    if !msg.is_empty() {
                        println!("[{}] Received message: {}", role, msg);
                        if let Err(e) = incoming_tx.send(msg.clone()) {
                            println!("[{}] Failed to push to incoming_tx: {}", role, e);
                        } else {
                            println!("[{}] Message pushed to incoming_tx", role);
                        }
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // Timeout — no data available, continue loop
                }
                Err(e) => {
                    println!("[{}] Error reading from stream: {}", role, e);
                    break;
                }
            }
        }

        thread::sleep(Duration::from_millis(20));
    }
}

