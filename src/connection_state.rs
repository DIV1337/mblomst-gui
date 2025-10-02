use std::net::TcpStream;
use crossbeam::channel::{Sender, Receiver};

pub struct ConnectionState {
    pub outgoing_tx: Sender<String>,
    pub outgoing_rx: Receiver<String>,
    pub incoming_tx: Sender<String>,
    pub incoming_rx: Receiver<String>,
    pub stream: Option<TcpStream>,
    pub connected: bool,
    pub is_host: bool,
    pub turn: usize, // 👈 nytt fält för turhantering
}

impl ConnectionState {
    pub fn new() -> Self {
        println!("ConnectionState created");
        let (outgoing_tx, outgoing_rx) = crossbeam::channel::unbounded();
        let (incoming_tx, incoming_rx) = crossbeam::channel::unbounded();

        Self {
            outgoing_tx,
            outgoing_rx,
            incoming_tx,
            incoming_rx,
            stream: None,
            connected: false,
            is_host: false,
            turn: 0, // 👈 initiera turen
        }
    }
}