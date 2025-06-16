use std::collections::HashMap;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::thread;
use tokio::sync::Notify;
use tokio::time::Duration;

use crate::message::{Message, MessageType, Serializable};

const PORT: u16 = 34254;
const INTERFACE: &str = "10.10.0.";

pub struct Client {
    socket: UdpSocket,
    pub my_ip: String,
    pending_acks: Arc<Mutex<HashMap<uuid::Uuid, Arc<Notify>>>>,
}

impl Client {
    pub fn new() -> Self {
        let mut my_ip = None;
        for i in 2..=50 {
            let test_ip = format!("{}{}", INTERFACE, i);
            let bind_addr = format!("{}:{}", test_ip, PORT);
            if UdpSocket::bind(&bind_addr).is_ok() {
                my_ip = Some(test_ip);
                break;
            }
        }
        let my_ip = my_ip.expect("Failed to find a free IP address in the range");
        let bind_addr = format!("{}:{}", my_ip, PORT);

        let socket = UdpSocket::bind(&bind_addr).expect("Failed to bind UDP socket");
        socket.set_broadcast(true).expect("Could not set broadcast");

        Client {
            socket,
            my_ip,
            pending_acks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn discover(&self) -> std::io::Result<()> {
        // Periodically send discovery packets
        thread::spawn({
            let client = self.try_clone()?;
            move || loop {
                let msg = Message::new(
                    MessageType::Discovery,
                    client.my_ip.as_str(),
                    "255.255.255.255",
                    None,
                );
                client.send_to_broadcast(&msg.serialize());
                thread::sleep(Duration::from_secs(2));
            }
        });
        Ok(())
    }

    pub fn send_to(&self, msg: &[u8], addr: &str) -> std::io::Result<usize> {
        self.socket.send_to(msg, addr)
    }

    pub fn send_to_broadcast(&self, msg: &[u8]) {
        for i in 2..=50 {
            let ip = format!("{}{}", INTERFACE, i);
            if ip == self.my_ip {
                continue;
            }
            let addr = format!("{}:{}", ip, PORT);
            let _ = self.send_to(msg, &addr);
        }
    }

    pub async fn send_reliable(&self, msg: Message, dest: &str) -> std::io::Result<()> {
        let msg_id = msg.id;
        let notify = Arc::new(Notify::new());

        {
            let mut pending = self.pending_acks.lock().unwrap();
            pending.insert(msg_id, notify.clone());
        }

        let serialized = msg.serialize();

        for _ in 0..5 {
            self.send_to(&serialized, dest)?;
            // Wait for ack or timeout
            let notified =
                tokio::time::timeout(Duration::from_millis(500), notify.notified()).await;
            if notified.is_ok() {
                // Acknowledgment received
                let mut pending = self.pending_acks.lock().unwrap();
                pending.remove(&msg_id);
                return Ok(());
            }
        }

        // Cleanup if no ack received
        let mut pending = self.pending_acks.lock().unwrap();
        pending.remove(&msg_id);
        Err(std::io::Error::new(
            std::io::ErrorKind::TimedOut,
            "No ACK received",
        ))
    }

    pub fn try_clone(&self) -> std::io::Result<Client> {
        Ok(Client {
            socket: self.socket.try_clone()?,
            my_ip: self.my_ip.clone(),
            pending_acks: Arc::clone(&self.pending_acks),
        })
    }

    pub fn recv_from(&self, buf: &mut [u8]) -> std::io::Result<(usize, std::net::SocketAddr)> {
        self.socket.recv_from(buf)
    }
}
