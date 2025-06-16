use crate::message::{Message, MessageType, Serializable};
use std::collections::HashSet;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

mod client;
mod message;
pub mod utils; // or pub mod ip;
pub use utils::ip::IntoIpv4Bytes; // Re-export for top-level access

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Auto-pick a free IP from 10.10.0.2 to 10.10.0.254
    let client = client::Client::new();
    client.discover()?;
    println!("Client started with IP: {}", client.my_ip);

    let known_peers: std::sync::Mutex<Vec<[u8; 4]>> = std::sync::Mutex::new(Vec::new());

    // Listen for incoming packets
    let mut buf = [0u8; 512];
    let client = Arc::new(client);
    let seen_messages: Arc<Mutex<HashSet<(IpAddr, Uuid)>>> = Arc::new(Mutex::new(HashSet::new()));
    loop {
        match client.recv_from(&mut buf) {
            Ok((n, src)) => {
                let msg = Message::deserialize(&buf[..n]);
                // if discovery message, add to known peers
                match msg.type_ {
                    MessageType::Discovery => {
                        let mut peers = known_peers.lock().unwrap();
                        if !peers.contains(&msg.source) {
                            peers.push(msg.source);
                            println!(
                                "Discovered peer: {} from {}",
                                msg.source
                                    .iter()
                                    .map(|b| b.to_string())
                                    .collect::<Vec<String>>()
                                    .join("."),
                                src
                            );
                            // Send request to the discovered peer with hello message
                            let hello_msg = Message::new(
                                MessageType::Request,
                                client.my_ip.as_str(),
                                msg.source,
                                Some(b"Hello from client!"),
                            );
                            let client = client.clone();
                            tokio::spawn(async move {
                                match client.send_reliable(hello_msg, &src.to_string()).await {
                                    Ok(_) => println!("Request sent successfully"),
                                    Err(e) => eprintln!("Failed to send request: {}", e),
                                }
                            });
                        }
                    }
                    MessageType::Request => {
                        println!("Received request from {}: {:?}", src, msg);
                        // Check if the message has already been seen
                        let src_ip = src.ip();
                        let message_key = (src_ip, msg.id);

                        let mut seen = seen_messages.lock().unwrap();
                        if !seen.insert(message_key) {
                            // Already seen — skip processing
                            continue;
                        }
                        // Send acknowledgment back to the requester
                        let ack_msg = Message::new_with_id(
                            msg.id,
                            MessageType::Acknowledgment,
                            client.my_ip.as_str(),
                            msg.source,
                            None,
                        );
                        client
                            .send_to(&ack_msg.serialize(), &src.to_string())
                            .expect("Failed to send acknowledgment");
                    }
                    MessageType::Acknowledgment => {
                        println!("Received acknowledgment from {} ({:?})", src, msg.id);
                    }
                    MessageType::Error => {
                        eprintln!("Error message from {}: {:?}", src, msg);
                    }
                }
            }
            Err(e) => eprintln!("recv error: {}", e),
        }
    }
}
