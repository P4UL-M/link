use crate::message::{Message, MessageType, Serializable};
use std::sync::Arc;

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
                                src.ip()
                            );
                            // Send request to the discovered peer with hello message
                            let hello_msg = Message::new(
                                MessageType::Request,
                                client.my_ip.as_str(),
                                msg.source,
                                Some(
                                    format!(
                                        "Hello from client {}!",
                                        client.my_ip.split('.').collect::<Vec<&str>>()[3]
                                    )
                                    .as_bytes(),
                                ),
                            );
                            let client = client.clone();
                            tokio::spawn(async move {
                                match client.send_reliable(hello_msg, &src.to_string()).await {
                                    Ok(_) => (),
                                    Err(e) => eprintln!("Failed to send request: {}", e),
                                }
                            });
                        }
                    }
                    MessageType::Request => {
                        // ! Think if acknowledgment should be sent before of after duplicate check
                        client.send_ack(&msg, &src.to_string());

                        if !client.should_process(src.ip(), msg.id) {
                            continue;
                        }

                        // Process the request message
                        println!(
                            "Received message from {}: {}",
                            src.ip(),
                            msg.data
                                .as_ref()
                                .map(|d| String::from_utf8_lossy(d))
                                .unwrap_or_else(|| "No data".into())
                        );
                    }
                    MessageType::Acknowledgment => {
                        client.acknowledge(msg.id);
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
