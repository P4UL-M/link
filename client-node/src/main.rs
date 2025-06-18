use client_node::client::Client;
use client_node::packet::{Packet, PacketType};
use client_node::utils::ip::{IntoIpv4Bytes, IntoIpv4String};
use client_node::utils::traits::Serializable;
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use indexmap::IndexMap;
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use std::collections::BTreeSet;
use std::io;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Duration, Instant}; // Re-export for top-level access

const MESSAGE_CAPACITY: usize = 50; // Maximum number of messages to keep in the UI

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
    source: [u8; 4],
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
    let client = Arc::new(Client::new().await);
    client.clone().discover().await?;
    println!("Client started with IP: {}", client.my_ip);

    let known_peers: Arc<Mutex<IndexMap<[u8; 4], Instant>>> = Arc::new(Mutex::new(IndexMap::new()));
    // Task 0: Periodic cleanup of known peers
    let cleanup_peers = known_peers.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(1)).await;
            let now = Instant::now();
            let mut peers = cleanup_peers.lock().await;
            peers.retain(|_, last_seen| now.duration_since(*last_seen) < Duration::from_secs(5));
        }
    });

    let mut messages: BTreeSet<MessageEntry> = BTreeSet::new();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<UiEvent>();

    // Task 1: Packet listener
    let client_listener = client.clone();
    let tx_listener = tx.clone();
    let known_peers_listener = known_peers.clone();
    tokio::spawn(async move {
        let mut buf = [0u8; 512];
        loop {
            if let Ok((n, src)) = client_listener.recv_from(&mut buf).await {
                let msg: Packet<String> = Packet::deserialize(&buf[..n]);
                match msg.type_ {
                    PacketType::Discovery => {
                        client_listener.observe(msg.timestamp).await;
                        let newly_added = {
                            // In a block to drop the lock before sending messages
                            let now = Instant::now();
                            let mut peers = known_peers_listener.lock().await;
                            let is_new = !peers.contains_key(&msg.source);
                            peers.insert(msg.source, now);
                            is_new
                        };
                        if newly_added {
                            push_message!(
                                tx_listener,
                                MessageEntry {
                                    timestamp: msg.timestamp,
                                    source: msg.source,
                                    content: format!(
                                        "Discovered peer: {}",
                                        msg.source
                                            .iter()
                                            .map(|b| b.to_string())
                                            .collect::<Vec<String>>()
                                            .join(".")
                                    ),
                                    flag: MessageFlag::Normal
                                }
                            );
                        }
                    }
                    PacketType::Request => {
                        let _ = client_listener.send_ack(&msg, &msg.source).await;
                        if !client_listener.should_process(src.ip(), msg.id).await {
                            continue;
                        }
                        client_listener.observe(msg.timestamp).await;

                        let content = msg.data.unwrap_or_else(|| "No data".into());
                        push_message!(
                            tx_listener,
                            MessageEntry {
                                timestamp: msg.timestamp,
                                source: msg.source,
                                content: format!("<{}> {}", src.ip(), content),
                                flag: MessageFlag::Normal
                            }
                        );
                    }
                    PacketType::Acknowledgment => client_listener.acknowledge(msg.id).await,
                    PacketType::Error => {}
                }
            }
        }
    });

    // Task 2: UI event loop
    let mut input = String::new();
    let mut target_index: isize = -1; // -1 means broadcast
    let mut scroll_offset: u16 = 0;
    enable_raw_mode().unwrap();
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    loop {
        let start_time = Instant::now();

        // Handle input
        if event::poll(std::time::Duration::from_millis(0)).unwrap() {
            if let Event::Key(key_event) = event::read().unwrap() {
                match key_event.code {
                    KeyCode::Char(c) => {
                        input.push(c);
                    }
                    KeyCode::Backspace => {
                        input.pop();
                    }
                    KeyCode::Enter => {
                        if input.is_empty() {
                            continue; // Ignore empty input
                        }
                        // Send the message to the selected target
                        let target = {
                            let peers = known_peers.lock().await;
                            if target_index == -1 || target_index as usize >= peers.len() {
                                None
                            } else {
                                Some(
                                    peers
                                        .get_index(target_index as usize)
                                        .map(|(ip, _)| ip.clone())
                                        .expect("Invalid index"),
                                )
                            }
                        };
                        let client = client.clone();
                        let tx = tx.clone();
                        if let Some(ip) = target {
                            let ts = client.tick().await;
                            let msg = Packet::new(
                                PacketType::Request,
                                client.my_ip.as_str(),
                                ip,
                                ts,
                                Some(input.clone()),
                            );
                            let target_str = ip
                                .iter()
                                .map(|b| b.to_string())
                                .collect::<Vec<_>>()
                                .join(".");
                            let entry = MessageEntry {
                                timestamp: ts,
                                source: client.my_ip.into_ipv4(),
                                content: format!("[{}] {}", target_str, input),
                                flag: MessageFlag::Normal,
                            };
                            push_message!(tx, entry.clone());
                            tokio::spawn(async move {
                                if let Err(_) = client.send_reliable(msg, ip).await {
                                    set_error_flag!(tx, entry);
                                }
                            });
                        } else {
                            let ts = client.tick().await;
                            let msg = Packet::new(
                                PacketType::Request,
                                client.my_ip.as_str(),
                                [255, 255, 255, 255], // Broadcast
                                ts,
                                Some(input.clone()),
                            );
                            let peers = {
                                let peers =
                                    known_peers.lock().await.keys().cloned().collect::<Vec<_>>();
                                peers
                            };
                            let entry = MessageEntry {
                                timestamp: ts,
                                source: client.my_ip.into_ipv4(),
                                content: format!("[broadcast] {}", input),
                                flag: MessageFlag::Normal,
                            };
                            push_message!(tx, entry.clone());
                            tokio::spawn(async move {
                                if let Err(_) = client.send_multicast_reliable(msg, peers).await {
                                    set_error_flag!(tx, entry);
                                }
                            });
                        }
                        input.clear();
                    }
                    KeyCode::Tab => {
                        let peers = known_peers.lock().await;
                        if peers.is_empty() {
                            target_index = -1;
                        } else {
                            target_index = ((target_index + 2) % ((peers.len() + 1) as isize)) - 1;
                        }
                    }
                    KeyCode::Esc => {
                        break;
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

        // Process UI events
        while let Ok(event) = rx.try_recv() {
            match event {
                UiEvent::NewMessage(msg) => {
                    messages.insert(msg);
                    if messages.len() > MESSAGE_CAPACITY {
                        messages.pop_first();
                    }
                    scroll_offset = scroll_offset.saturating_add(1);
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

        let peers_snapshot = {
            let peers = known_peers.lock().await;
            peers.clone()
        };

        let ts = client.get_clock().await;
        let clock_display = format!("{:>8}", ts % 1_000_000);

        // Draw UI with ratatui
        terminal.draw(|f| {
            let size = f.area();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Length(3), Constraint::Min(1)].as_ref())
                .split(size);

            let top_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Min(1), Constraint::Length(10)].as_ref())
                .split(chunks[0]);

            let target_str = {
                let peers = peers_snapshot;
                if target_index == -1 || target_index as usize >= peers.len() {
                    "broadcast".to_string()
                } else {
                    peers
                        .get_index(target_index as usize)
                        .map(|(ip, _)| {
                            ip.iter()
                                .map(|b| b.to_string())
                                .collect::<Vec<_>>()
                                .join(".")
                        })
                        .unwrap_or_else(|| "unknown".to_string())
                }
            };

            let input_block = Paragraph::new(input.as_str()).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("[{}]>", target_str)),
            );
            f.render_widget(input_block, top_chunks[0]);

            let clock_block = Paragraph::new(clock_display.as_str())
                .block(Block::default().borders(Borders::ALL).title("Clock"));
            f.render_widget(clock_block, top_chunks[1]);

            let message_lines: Vec<Line> = messages
                .iter()
                .cloned()
                .map(|msg| {
                    let style = match msg.flag {
                        MessageFlag::Error => Style::default().fg(Color::Red),
                        MessageFlag::Normal => Style::default(),
                    };
                    Line::from(Span::styled(msg.content, style))
                })
                .collect();

            let visible_lines = chunks[1].height.saturating_sub(2); // Subtract borders
            let total_lines = message_lines.len() as u16;
            let max_scroll = total_lines.saturating_sub(visible_lines);
            if scroll_offset > max_scroll {
                scroll_offset = max_scroll;
            }

            let messages_block = Paragraph::new(message_lines)
                .block(Block::default().borders(Borders::ALL).title("Messages"))
                .scroll((scroll_offset, 0));
            f.render_widget(messages_block, chunks[1]);
        })?;

        // Limit the loop to 60 FPS
        let elapsed = start_time.elapsed();
        if elapsed < Duration::from_millis(16) {
            tokio::time::sleep(Duration::from_millis(16) - elapsed).await;
        }
    }

    terminal.show_cursor()?;
    disable_raw_mode().unwrap();
    Ok(())
}
