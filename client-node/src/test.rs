use crate::client::{Client, ClientMode, Status};
use client_node::network_core::NetworkCore;
use client_node::packet::{PACKET_SIZE, Packet, PacketData, PacketType};
use client_node::utils::addr::IpAddr;
use client_node::utils::key::Key;
use crossterm::event::KeyEventKind;
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use indexmap::IndexMap;
use std::collections::BTreeSet;
use std::io;
use std::io::Write;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Duration, Instant}; // Re-export for top-level access

mod client;

const MESSAGE_CAPACITY: usize = 1000; // Maximum number of messages to keep in the UI

macro_rules! push_message {
    ($tx:expr, $msg:expr) => {{
        // Send a new message to the UI with event
        let _ = $tx.send(UiEvent::NewMessage($msg));
    }};
}

macro_rules! set_error_flag {
    ($tx:expr, $id:expr) => {{
        // Set a flag for a message in the UI
        let _ = $tx.send(UiEvent::SetFlag($id, MessageFlag::Error));
    }};
}

enum UiEvent {
    NewMessage(MessageEntry),
    SetFlag(MessageEntry, MessageFlag),
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
enum MessageFlag {
    Normal,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MessageEntry {
    timestamp: u64,
    source: IpAddr,
    content: String,
    flag: MessageFlag,
}

impl Ord for MessageEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.timestamp
            .cmp(&other.timestamp)
            .then_with(|| self.source.cmp(&other.source))
    }
}

impl PartialOrd for MessageEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Auto-pick a free IP from 10.10.0.2 to 10.10.0.254
    let ntc = Arc::new(NetworkCore::new().await);
    let client = Arc::new(Client::new(ntc));

    client.clone().set_mode(ClientMode::Normal).await;

    #[derive(Clone, Debug)]
    enum PeerInfo {
        LastSeen(Instant),
        KeyOption(()),
    }
    let known_peers: Arc<Mutex<IndexMap<IpAddr, PeerInfo>>> = Arc::new(Mutex::new(IndexMap::new()));

    // Task 0: Periodic cleanup of known peers
    let cleanup_peers = known_peers.clone();
    let cleanup_client = client.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(10)).await;
            // Cleanup known peers that haven't been seen for 5 seconds
            let now = Instant::now();
            let mut peers = cleanup_peers.lock().await;
            peers.retain(|_, info| match info {
                PeerInfo::LastSeen(last_seen) => {
                    now.duration_since(*last_seen) < Duration::from_secs(5)
                }
                PeerInfo::KeyOption(_) => true,
            });

            // Reset SSL mode if the server hasn't been seen for 10 seconds
            if let ClientMode::SSL(_, _, _, timestamp) = cleanup_client.get_mode().await
                && timestamp.elapsed() > Duration::from_secs(10)
            {
                cleanup_client.clone().set_mode(ClientMode::Normal).await;
            }
        }
    });

    let mut messages: BTreeSet<MessageEntry> = BTreeSet::new();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<UiEvent>();

    // Task 1: Packet listener
    let client_listener = client.clone();
    let ntc = client.get_network_core();
    let tx_listener = tx.clone();
    let known_peers_listener = known_peers.clone();
    let client_key = Key::generate(); // Generate a new key for the client
    tokio::spawn(async move {
        let mut buf = [0u8; PACKET_SIZE];
        loop {
            if let Ok((n, src)) = ntc.recv_from(&mut buf).await {
                let msg: Packet = Packet::deserialize(&buf[..n]).unwrap();
                if !msg.check_src(src.ip()) {
                    continue;
                }
                match msg.type_ {
                    PacketType::Discovery => {
                        ntc.observe(msg.timestamp).await;
                        let _ = {
                            // In a block to drop the lock before sending messages
                            let now = Instant::now();
                            let mut peers = known_peers_listener.lock().await;
                            peers.insert(msg.source, PeerInfo::LastSeen(now)).is_none()
                        };
                    }
                    PacketType::Request => {
                        let _ = ntc.send_ack(&msg, msg.source).await;
                        if !ntc.should_process(msg.source, msg.id).await {
                            continue;
                        }
                        ntc.observe(msg.timestamp).await;

                        match msg.data {
                            PacketData::Text(content) => {
                                push_message!(
                                    tx_listener,
                                    MessageEntry {
                                        timestamp: msg.timestamp,
                                        source: msg.source,
                                        content: content,
                                        flag: MessageFlag::Normal
                                    }
                                );
                            }
                            PacketData::KeyVerification(data) => {
                                if let ClientMode::SSL(key, server_ip, status, _) =
                                    client_listener.get_mode().await
                                {
                                    if server_ip != msg.source {
                                        continue; // Ignore if the source is not the expected server IP
                                    }
                                    if status == Status::Ready {
                                        // If already in SSL mode, ignore the verification
                                        continue;
                                    }
                                    let message = key.decode_message(&data);
                                    let message = Packet::new(
                                        PacketType::Request,
                                        ntc.my_ip,
                                        msg.source,
                                        msg.timestamp,
                                        PacketData::KeyVerification(message.into_bytes()),
                                    );
                                    let ntc = ntc.clone();
                                    tokio::spawn(async move {
                                        let _ = ntc.send_reliable(message, msg.source).await;
                                    });
                                }
                            }
                            PacketData::KeyValidation => {
                                if let ClientMode::SSL(key, server_ip, status, _) =
                                    client_listener.get_mode().await
                                {
                                    if server_ip != msg.source {
                                        continue; // Ignore if the source is not the expected server IP
                                    }
                                    if status == Status::Ready {
                                        // If already in SSL mode, ignore the validation
                                        continue;
                                    }
                                    // Switch to SSL mode with the discovered key
                                    client_listener
                                        .clone()
                                        .set_mode(ClientMode::SSL(
                                            key,
                                            msg.source,
                                            Status::Ready,
                                            Instant::now(),
                                        ))
                                        .await;
                                }
                            }
                            _ => {} // Ignore other packet data types
                        }
                    }
                    PacketType::Acknowledgment => ntc.acknowledge(msg.id).await,
                    PacketType::Broadcast => {}
                    PacketType::DNSDiscovery => {
                        ntc.observe(msg.timestamp).await;
                        ntc.send_dns_ack(&msg, msg.source).await;
                        if let ClientMode::SSL(client_key, server_ip, status, _) =
                            client_listener.get_mode().await
                        {
                            if server_ip != msg.source {
                                continue; // Ignore if the source is not the expected server IP
                            }
                            client_listener
                                .clone()
                                .set_mode(ClientMode::SSL(
                                    client_key,
                                    server_ip,
                                    status,
                                    Instant::now(),
                                ))
                                .await;
                        } else {
                            // If not in SSL mode, switch to SSL mode with the discovered key
                            client_listener
                                .clone()
                                .set_mode(ClientMode::SSL(
                                    client_key.clone(),
                                    msg.source,
                                    Status::Initializing,
                                    Instant::now(),
                                ))
                                .await;
                        }
                    }
                    PacketType::DNSUpdate => {
                        if client_listener.is_discovering().await {
                            continue; // Ignore DNS updates during discovery
                        }
                        let _ = ntc.send_ack(&msg, msg.source).await;
                        ntc.observe(msg.timestamp).await;
                        // Update the known peers with DNS discovery content
                        let mut peers = known_peers_listener.lock().await;
                        if let PacketData::Peers(new_peers) = msg.data {
                            // Remove peers that are not in the new list
                            peers.retain(|ip, _| new_peers.contains(ip)); // ! this line is dangerous as it remove peers that are not known from the auth server but still exist on the network like no ssl node
                            // Add new peers from the DNS update
                            for peer in new_peers {
                                if peer != ntc.my_ip {
                                    let newly_added = peers.insert(peer, PeerInfo::KeyOption(()));
                                    if newly_added.is_none() {
                                        // If the peer was not already known, notify the UI
                                        push_message!(
                                            tx_listener,
                                            MessageEntry {
                                                timestamp: msg.timestamp,
                                                source: peer,
                                                content: format!("New peer: {}", peer),
                                                flag: MessageFlag::Normal
                                            }
                                        );
                                    }
                                }
                            }
                        }
                    }
                    PacketType::DNSAcknowledgment => {}
                }
            }
        }
    });

    // Task 2: UI event loop
    let ntc = client.get_network_core();
    let peers = known_peers.clone();

    let mut file_path: Option<String> = None;

    // Check if there is an argument in the command line call
    let mut args = std::env::args();
    if args.len() > 1 {
        let target = args.nth(1).unwrap();
        // Check if the argument is a valid file path
        if std::path::Path::new(&target).exists() {
            file_path = Some(target);
        } else {
            eprintln!("Invalid file path: {}", target);
            return Ok(());
        }
    }

    // Read the file if it exists
    let mut file_lines: Vec<String> = Vec::new();
    if let Some(path) = file_path {
        match std::fs::read_to_string(&path) {
            Ok(content) => {
                file_lines = content.lines().map(|line| line.to_string()).collect();
            }
            Err(_) => {
                eprintln!("Failed to read file: {}", path);
                return Ok(());
            }
        }
    }

    enable_raw_mode().unwrap();

    // Main event loop
    loop {
        let start_time = Instant::now();

        if event::poll(std::time::Duration::from_millis(0)).unwrap() {
            if let Event::Key(key_event) = event::read().unwrap() {
                if key_event.kind != KeyEventKind::Press {
                    continue;
                }
                match key_event.code {
                    KeyCode::Esc => {
                        println!("Exiting...\r");
                        io::stdout().flush().unwrap();
                        break;
                    }
                    _ => {}
                }
            }
        }

        // Process UI events
        while let Ok(event) = rx.try_recv() {
            match event {
                UiEvent::NewMessage(msg) => {
                    messages.insert(msg);
                    if messages.len() > MESSAGE_CAPACITY {
                        messages.pop_first();
                    }
                }
                UiEvent::SetFlag(timestamp, flag) => {
                    if let Some(existing) = messages.take(&timestamp) {
                        let mut updated = existing;
                        updated.flag = flag;
                        messages.insert(updated);
                    }
                }
            }
        }

        // Send every line from the file as broadcast messages
        if !file_lines.is_empty() && peers.lock().await.len() > 0 {
            let line = file_lines.remove(0);

            let tx = tx.clone();
            let ts = ntc.tick().await;
            let msg = Packet::new(
                PacketType::Request,
                ntc.my_ip,
                [255, 255, 255, 255].into(), // Broadcast
                ts,
                PacketData::Text(line.clone()),
            );
            let peers = {
                let peers = known_peers.lock().await.keys().cloned().collect::<Vec<_>>();
                peers
            };
            let entry = MessageEntry {
                timestamp: ts,
                source: ntc.my_ip,
                content: line.clone(),
                flag: MessageFlag::Normal,
            };
            push_message!(tx, entry.clone());
            let ntc = ntc.clone();
            tokio::spawn(async move {
                if let Err(_) = ntc.clone().send_multicast_reliable(msg, peers).await {
                    set_error_flag!(tx, entry);
                }
                println!("Sent message: {}\r", line);
                io::stdout().flush().unwrap();
            });
        }

        // Limit the loop to 60 FPS
        let elapsed = start_time.elapsed();
        if elapsed < Duration::from_millis(16) {
            tokio::time::sleep(Duration::from_millis(16) - elapsed).await;
        }
    }

    // Write every message to an output file
    let output_path = format!("output-{}.csv", ntc.my_ip);
    let mut output_file = std::fs::File::create(output_path).expect("Failed to create output file");
    for message in messages.iter() {
        let line = format!(
            "{:#?},{},{},{}\n",
            message.flag, message.timestamp, message.source, message.content
        );
        output_file
            .write_all(line.as_bytes())
            .expect("Failed to write to output file");
    }

    disable_raw_mode().unwrap();

    Ok(())
}
