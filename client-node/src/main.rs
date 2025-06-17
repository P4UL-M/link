use crate::client::Client;
use crate::packet::{Packet, PacketType, Serializable};
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use indexmap::IndexMap;
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
};
use std::collections::VecDeque;
use std::io::{self};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Duration, Instant, timeout};

mod client;
mod packet;
pub mod utils; // or pub mod ip;
pub use utils::ip::IntoIpv4Bytes; // Re-export for top-level access

const MESSAGE_CAPACITY: usize = 50; // Maximum number of messages to keep in the UI

macro_rules! push_message {
    ($tx:expr, $msg:expr) => {{
        // Send a new message to the UI with event
        let _ = $tx.send(UiEvent::NewMessage($msg));
    }};
}

macro_rules! push_error {
    ($tx:expr, $msg:expr) => {{
        let _ = $tx.send(UiEvent::ErrorMessage($msg));
    }};
}

enum UiEvent {
    NewMessage(String),
    ErrorMessage(String),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Auto-pick a free IP from 10.10.0.2 to 10.10.0.254
    let client = Arc::new(Client::new().await);
    client.clone().discover().await?;
    println!("Client started with IP: {}", client.my_ip);

    let known_peers: Arc<Mutex<IndexMap<[u8; 4], Instant>>> = Arc::new(Mutex::new(IndexMap::new()));
    // Periodically clean up old peers in the background
    let cleanup_peers = known_peers.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(500)).await;
            let now = Instant::now();
            let mut peers = cleanup_peers.lock().await;
            peers.retain(|_, last_seen| now.duration_since(*last_seen) < Duration::from_secs(5));
        }
    });

    let mut messages: VecDeque<String> = VecDeque::with_capacity(MESSAGE_CAPACITY);
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<UiEvent>();
    let mut input = String::new();
    let mut target_index: isize = -1; // -1 means broadcast
    let mut scroll_offset: u16 = 0;
    enable_raw_mode().unwrap();
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    // Listen for incoming packets
    let mut buf = [0u8; 512];
    loop {
        // ! Update: Use non-blocking recv_from
        while let Ok(Ok((n, src))) =
            timeout(Duration::from_millis(1), client.recv_from(&mut buf)).await
        {
            let msg = Packet::deserialize(&buf[..n]);
            match msg.type_ {
                PacketType::Discovery => {
                    let newly_added = {
                        // In a block to drop the lock before sending messages
                        let now = Instant::now();
                        let mut peers = known_peers.lock().await;
                        let is_new = !peers.contains_key(&msg.source);
                        peers.insert(msg.source, now);
                        is_new
                    };
                    if newly_added {
                        push_message!(
                            tx,
                            format!(
                                "Discovered peer: {}",
                                msg.source
                                    .iter()
                                    .map(|b| b.to_string())
                                    .collect::<Vec<String>>()
                                    .join(".")
                            )
                        );
                    }
                }
                PacketType::Request => {
                    let _ = client.send_ack(&msg, &src.to_string()).await;
                    if !client.should_process(src.ip(), msg.id) {
                        continue;
                    }

                    let content = msg
                        .data
                        .as_ref()
                        .map(|d| String::from_utf8_lossy(d).to_string())
                        .unwrap_or_else(|| "No data".into());
                    push_message!(tx, format!("{}> {}", src.ip(), content));
                }
                PacketType::Acknowledgment => client.acknowledge(msg.id),
                PacketType::Error => {
                    push_error!(tx, format!("Error message from {}: {:?}", src, msg))
                }
            }
        }

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
                            let msg = Packet::new(
                                PacketType::Request,
                                client.my_ip.as_str(),
                                ip,
                                Some(input.as_bytes()),
                            );
                            let target_str = ip
                                .iter()
                                .map(|b| b.to_string())
                                .collect::<Vec<_>>()
                                .join(".");

                            tokio::spawn(async move {
                                if let Err(e) = client.send_reliable(msg, &target_str).await {
                                    push_error!(
                                        tx,
                                        format!("Failed to send message to {}: {}", target_str, e)
                                    );
                                }
                            });
                        } else {
                            let msg = Packet::new(
                                PacketType::Request,
                                client.my_ip.as_str(),
                                [255, 255, 255, 255], // Broadcast
                                Some(input.as_bytes()),
                            );
                            tokio::spawn(async move {
                                if let Err(e) = client.send_to_broadcast_reliable(msg).await {
                                    push_error!(
                                        tx,
                                        format!("Failed to send broadcast message: {}", e)
                                    );
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
                    messages.push_back(msg);
                    if messages.len() > MESSAGE_CAPACITY {
                        messages.pop_front();
                    }
                    scroll_offset = scroll_offset.saturating_add(1);
                }
                UiEvent::ErrorMessage(msg) => {
                    messages.push_back(msg);
                    if messages.len() > MESSAGE_CAPACITY {
                        messages.pop_front();
                    }
                    scroll_offset = scroll_offset.saturating_add(1);
                }
            }
        }

        let peers_snapshot = {
            let peers = known_peers.lock().await;
            peers.clone()
        };

        // Draw UI with ratatui
        terminal.draw(|f| {
            let size = f.area();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Length(3), Constraint::Min(1)].as_ref())
                .split(size);

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
            f.render_widget(input_block, chunks[0]);

            let msgs: String = messages.iter().cloned().collect::<Vec<_>>().join("\n");
            let visible_lines = chunks[1].height.saturating_sub(2); // Subtract borders
            let total_lines = msgs.lines().count() as u16;
            let max_scroll = total_lines.saturating_sub(visible_lines);
            if scroll_offset > max_scroll {
                scroll_offset = max_scroll;
            }
            let messages_block = Paragraph::new(msgs)
                .block(Block::default().borders(Borders::ALL).title("Messages"))
                .scroll((scroll_offset, 0));
            f.render_widget(messages_block, chunks[1]);
        })?;
    }

    terminal.show_cursor()?;
    disable_raw_mode().unwrap();
    Ok(())
}
