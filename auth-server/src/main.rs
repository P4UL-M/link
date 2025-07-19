use crate::utils::generate_validation_string;
use client_node::network_core::NetworkCore;
use client_node::packet::{PACKET_SIZE, Packet, PacketData, PacketType};
use client_node::utils::addr::IpAddr;
use client_node::utils::key::{PublicKey, SerializableKey};
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use indexmap::IndexMap;
use rand;
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use server::Server;
use std::collections::HashMap;
use std::io;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

mod server;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Simulate a unique client ID
    let my_id = rand::random::<u16>();
    println!("Client ID: {}", my_id);

    // Create a new client instance
    let ntc = Arc::new(NetworkCore::new().await);
    let server = Server::new(ntc);

    // Start the server discovery process
    server.discover().await?;

    let known_peers: Arc<Mutex<IndexMap<IpAddr, Instant>>> = Arc::new(Mutex::new(IndexMap::new()));
    let known_keys: Arc<Mutex<IndexMap<IpAddr, PublicKey>>> = Arc::new(Mutex::new(IndexMap::new()));

    // Task 0: Periodic cleanup of known peers
    let cleanup_ntc = server.get_network_core();
    let cleanup_peers = known_peers.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(1)).await;
            let now = Instant::now();
            let mut peers = cleanup_peers.lock().await;
            let lenght_before = peers.len();
            peers.retain(|_, last_seen| now.duration_since(*last_seen) < Duration::from_secs(10));
            if peers.len() != lenght_before {
                // Send update to all clients
                let ts = cleanup_ntc.tick().await;
                let peers: Vec<IpAddr> = peers.keys().cloned().collect();
                let msg = Packet::new(
                    PacketType::DNSUpdate,
                    cleanup_ntc.my_ip,
                    "255.255.255.255".into(),
                    ts,
                    PacketData::Peers(peers.clone()),
                );
                let _ = cleanup_ntc
                    .clone()
                    .send_multicast_reliable(msg, peers)
                    .await;
            }
        }
    });

    // Task 0.5: Periodic update broadcast
    let update_ntc = server.get_network_core();
    let update_peers = known_peers.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(8)).await;
            let ts = update_ntc.tick().await;
            let peers: Vec<IpAddr> = {
                let peers = update_peers.lock().await;
                peers.keys().cloned().collect()
            };
            let msg = Packet::new(
                PacketType::DNSUpdate,
                update_ntc.my_ip,
                "255.255.255.255".into(),
                ts,
                PacketData::Peers(peers.clone()),
            );
            let _ = update_ntc.clone().send_multicast_reliable(msg, peers).await;
        }
    });

    // Task 1: Listen for incoming packets
    let recv_ntc = server.get_network_core();
    let recv_peers = known_peers.clone();
    let recv_keys = known_keys.clone();
    let cypher_tests: Arc<Mutex<HashMap<String, SerializableKey>>> =
        Arc::new(Mutex::new(HashMap::new()));
    tokio::spawn(async move {
        let mut buf = [0u8; PACKET_SIZE];
        loop {
            if let Ok((n, _src)) = recv_ntc.recv_from(&mut buf).await {
                let packet = Packet::deserialize(&buf[..n]);
                if packet.is_err() {
                    eprintln!("Failed to deserialize packet: {}", packet.unwrap_err());
                    continue;
                }
                let msg = packet.unwrap();
                match msg.type_ {
                    PacketType::DNSAcknowledgment => {
                        let mut peers = recv_peers.lock().await;
                        if let None = peers.insert(msg.source, Instant::now()) {
                            // Send update to all clients
                            let ts = recv_ntc.tick().await;
                            let peers: Vec<IpAddr> = peers.keys().cloned().collect();
                            let msg = Packet::new(
                                PacketType::DNSUpdate,
                                recv_ntc.my_ip,
                                "255.255.255.255".into(),
                                ts,
                                PacketData::Peers(peers.clone()),
                            );
                            let _ = recv_ntc.clone().send_multicast_reliable(msg, peers).await;
                        }
                    }
                    PacketType::Request => {
                        let _ = recv_ntc.send_ack(&msg, msg.source).await;
                        if !recv_ntc.should_process(msg.source, msg.id).await {
                            continue; // Skip processing if the message is not new
                        }
                        recv_ntc.observe(msg.timestamp).await;

                        match msg.data {
                            PacketData::KeyExchange(key) => {
                                let test = generate_validation_string(&msg.source, &Instant::now());
                                cypher_tests.lock().await.insert(test.clone(), key.clone());
                                let cypher_tests = cypher_tests.clone();
                                let cleanup_test = test.clone();
                                // Make a cleanup task to remove old tests five seconds later
                                tokio::spawn(async move {
                                    tokio::time::sleep(Duration::from_secs(5)).await;
                                    cypher_tests.lock().await.remove(&cleanup_test);
                                });
                                let ntc = recv_ntc.clone();
                                tokio::spawn(async move {
                                    let key = PublicKey::from_serializable(&key);
                                    let ts = ntc.tick().await;
                                    let response = Packet::new(
                                        PacketType::Request,
                                        ntc.my_ip,
                                        msg.source,
                                        ts,
                                        PacketData::KeyVerification(key.encode_message(&test)),
                                    );
                                    let _ = ntc.send_reliable(response, msg.source).await;
                                });
                            }
                            PacketData::KeyVerification(data) => {
                                let test = String::from_utf8(data).unwrap();
                                if let Some(key) = cypher_tests.lock().await.remove(&test) {
                                    recv_keys
                                        .lock()
                                        .await
                                        .insert(msg.source, PublicKey::from_serializable(&key));

                                    let ts = recv_ntc.tick().await;
                                    let response = Packet::new(
                                        PacketType::Request,
                                        recv_ntc.my_ip,
                                        msg.source,
                                        ts,
                                        PacketData::KeyValidation,
                                    );
                                    let ntc = recv_ntc.clone();
                                    tokio::spawn(async move {
                                        let _ = ntc.send_reliable(response, msg.source).await;
                                    });
                                }
                            }
                            PacketData::KeyRequest(ip) => {
                                if (recv_keys.lock().await).contains_key(&ip) {
                                    let ts = recv_ntc.tick().await;
                                    let response = Packet::new(
                                        PacketType::Request,
                                        recv_ntc.my_ip,
                                        msg.source,
                                        ts,
                                        PacketData::KeyResponse(
                                            ip,
                                            recv_keys
                                                .lock()
                                                .await
                                                .get(&ip)
                                                .unwrap()
                                                .to_serializable(),
                                        ),
                                    );
                                    let ntc = recv_ntc.clone();
                                    tokio::spawn(async move {
                                        let _ = ntc.send_reliable(response, msg.source).await;
                                    });
                                } else {
                                    println!("No key found for {}", ip);
                                }
                            }
                            _ => {}
                        }
                    }
                    PacketType::Acknowledgment => recv_ntc.acknowledge(msg.id).await,
                    _ => {}
                }
            }
        }
    });

    // Task 2: Render loop to display known peers
    let mut scroll_offset: u16 = 0;
    enable_raw_mode().unwrap();
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    loop {
        // Handle terminal events
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc => {
                        break; // Exit on Escape key
                    }
                    KeyCode::Up => {
                        if scroll_offset > 0 {
                            scroll_offset -= 1;
                        }
                    }
                    KeyCode::Down => {
                        scroll_offset = scroll_offset.saturating_add(1);
                    }
                    _ => {}
                }
            }
        }

        // Render the terminal UI
        let peers_snapshot = {
            let peers = known_peers.lock().await;
            peers.clone()
        };

        terminal.draw(|f| {
            let size = f.area();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(100)].as_ref())
                .split(size);

            let mut lines = Vec::new();
            let width = chunks[0].width as usize; // get width of the widget
            lines.push(Line::from(vec![
                Span::styled(
                    "Peer",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(ratatui::style::Modifier::BOLD),
                ),
                Span::raw(" ".repeat(width.saturating_sub(25))),
                Span::styled(
                    "Last Seen",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(ratatui::style::Modifier::BOLD),
                ),
            ]));

            for (ip, last_seen) in peers_snapshot.iter() {
                let ip_str = ip.to_string();
                let duration = last_seen.elapsed();
                let display_time = if duration.as_secs() < 1 {
                    "<1s".to_string()
                } else {
                    format!("{}s", duration.as_secs())
                };

                let style = if duration.as_secs() > 3 {
                    Style::default().fg(Color::Red)
                } else {
                    Style::default()
                };

                lines.push(Line::from(vec![
                    Span::styled(ip_str, style),
                    Span::raw(" ".repeat(width.saturating_sub(25))),
                    Span::styled(display_time, style),
                ]));
            }

            let visible_lines = chunks[0].height.saturating_sub(2); // Subtract borders
            let total_lines = lines.len() as u16;
            let max_scroll = total_lines.saturating_sub(visible_lines);
            if scroll_offset > max_scroll {
                scroll_offset = max_scroll;
            }

            let paragraph = Paragraph::new(lines)
                .block(Block::default().borders(Borders::ALL).title("Known Peers"))
                .style(Style::default().fg(Color::White))
                .wrap(ratatui::widgets::Wrap { trim: false })
                .scroll((scroll_offset, 0));

            f.render_widget(paragraph, chunks[0]);
        })?;
    }

    terminal.show_cursor()?;
    disable_raw_mode().unwrap();
    Ok(())
}
