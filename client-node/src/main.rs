use crate::client::{Client, ClientMode, Status};
use client_node::network_core::NetworkCore;
use client_node::packet::{PACKET_SIZE, Packet, PacketData, PacketType};
use client_node::utils::addr::IpAddr;
use client_node::utils::key::{Key, PublicKey};
use crossterm::event::KeyEventKind;
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

mod client;

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
    Encrypted,
    Normal,
    Error,
    Info,
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
        KeyOption(Option<PublicKey>),
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
                        let newly_added = {
                            // In a block to drop the lock before sending messages
                            let now = Instant::now();
                            let mut peers = known_peers_listener.lock().await;
                            peers.insert(msg.source, PeerInfo::LastSeen(now)).is_none()
                        };
                        if newly_added {
                            push_message!(
                                tx_listener,
                                MessageEntry {
                                    timestamp: msg.timestamp,
                                    source: msg.source.into(),
                                    content: format!("Discovered peer: {}", msg.source),
                                    flag: MessageFlag::Info
                                }
                            );
                        }
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
                                        content: format!("<{}> {}", msg.source, content),
                                        flag: MessageFlag::Normal
                                    }
                                );
                            }
                            PacketData::SSLText(content) => {
                                push_message!(
                                    tx_listener,
                                    MessageEntry {
                                        timestamp: msg.timestamp,
                                        source: msg.source,
                                        content: format!(
                                            "<{}> {}",
                                            msg.source,
                                            client_key.decode_message(&content)
                                        ),
                                        flag: MessageFlag::Encrypted
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
                                    push_message!(
                                        tx_listener,
                                        MessageEntry {
                                            timestamp: msg.timestamp,
                                            source: msg.source,
                                            content: format!(
                                                "Switched to SSL mode with {}",
                                                msg.source
                                            ),
                                            flag: MessageFlag::Info
                                        }
                                    );
                                }
                            }
                            PacketData::KeyResponse(ip, key) => {
                                if let ClientMode::SSL(_, server_ip, status, _) =
                                    client_listener.get_mode().await
                                {
                                    if server_ip != msg.source {
                                        continue; // Ignore if the source is not the expected server IP
                                    }
                                    if status != Status::Ready {
                                        // If already in SSL mode, ignore the response
                                        continue;
                                    }
                                    // Store the key for later use
                                    let mut peers = known_peers_listener.lock().await;
                                    peers.insert(
                                        ip,
                                        PeerInfo::KeyOption(Some(PublicKey::from_serializable(
                                            &key,
                                        ))),
                                    );
                                    push_message!(
                                        tx_listener,
                                        MessageEntry {
                                            timestamp: msg.timestamp,
                                            source: msg.source,
                                            content: format!(
                                                "Received key from {}: {}",
                                                msg.source, ip
                                            ),
                                            flag: MessageFlag::Info
                                        }
                                    );
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
                            peers.retain(|ip, info| match info {
                                PeerInfo::KeyOption(_) => new_peers.contains(ip),
                                _ => true,
                            });
                            // Add new peers from the DNS update
                            for peer in new_peers {
                                if peer != ntc.my_ip {
                                    if let Some(PeerInfo::KeyOption(Some(_))) = peers.get(&peer) {
                                        // Skip if the peer already has a key
                                        continue;
                                    }
                                    let newly_added = peers.insert(peer, PeerInfo::KeyOption(None));
                                    if newly_added.is_none() {
                                        // If the peer was not already known, notify the UI
                                        push_message!(
                                            tx_listener,
                                            MessageEntry {
                                                timestamp: msg.timestamp,
                                                source: peer,
                                                content: format!("New peer: {}", peer),
                                                flag: MessageFlag::Info
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
    let mut input = String::new();
    let mut target_index: isize = -1; // -1 means broadcast
    let mut scroll_offset: u16 = 0;
    enable_raw_mode().unwrap();
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let ntc = client.get_network_core();
    // Main event loop
    loop {
        let start_time = Instant::now();

        // Handle input
        if event::poll(std::time::Duration::from_millis(0)).unwrap() {
            if let Event::Key(key_event) = event::read().unwrap() {
                if key_event.kind != KeyEventKind::Press {
                    continue;
                }
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
                                        .map(|(ip, info)| (*ip, info.clone()))
                                        .expect("Invalid index"),
                                )
                            }
                        };
                        let tx = tx.clone();
                        if let Some((ip, info)) = target {
                            let ts = ntc.tick().await;
                            let (data, flag) = match info {
                                PeerInfo::KeyOption(Some(key)) => (
                                    PacketData::SSLText(key.encode_message(&input)),
                                    MessageFlag::Encrypted,
                                ),
                                _ => (PacketData::Text(input.clone()), MessageFlag::Normal),
                            };
                            let msg = Packet::new(PacketType::Request, ntc.my_ip, ip, ts, data);
                            let target_str = ip.to_string();
                            let entry = MessageEntry {
                                timestamp: ts,
                                source: ntc.my_ip,
                                content: format!("[{}] {}", target_str, input),
                                flag,
                            };
                            push_message!(tx, entry.clone());
                            let ntc = ntc.clone();
                            tokio::spawn(async move {
                                if let Err(_) = ntc.send_reliable(msg, ip).await {
                                    set_error_flag!(tx, entry);
                                }
                            });
                        } else {
                            let ts = ntc.tick().await;
                            let msg = Packet::new(
                                PacketType::Request,
                                ntc.my_ip,
                                [255, 255, 255, 255].into(), // Broadcast
                                ts,
                                PacketData::Text(input.clone()),
                            );
                            let peers = {
                                let peers =
                                    known_peers.lock().await.keys().cloned().collect::<Vec<_>>();
                                peers
                            };
                            let entry = MessageEntry {
                                timestamp: ts,
                                source: ntc.my_ip,
                                content: format!("[broadcast] {}", input),
                                flag: MessageFlag::Normal,
                            };
                            push_message!(tx, entry.clone());
                            let ntc = ntc.clone();
                            tokio::spawn(async move {
                                if let Err(_) =
                                    ntc.clone().send_multicast_reliable(msg, peers).await
                                {
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

                            // If in ssl mode, try to get the public key
                            if let ClientMode::SSL(_, server_ip, status, _) =
                                client.get_mode().await
                            {
                                if status == Status::Ready && target_index >= 0 {
                                    if let Some((ip, info)) = peers.get_index(target_index as usize)
                                    {
                                        if let PeerInfo::KeyOption(None) = info {
                                            // If the key is not known, request it
                                            let ts = ntc.tick().await;
                                            let msg = Packet::new(
                                                PacketType::Request,
                                                ntc.my_ip,
                                                server_ip,
                                                ts,
                                                PacketData::KeyRequest(ip.clone()),
                                            );
                                            let ntc = ntc.clone();
                                            tokio::spawn(async move {
                                                let _ = ntc.send_reliable(msg, server_ip).await;
                                            });
                                        }
                                    }
                                }
                            }
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

        let ts = ntc.get_clock().await;
        let mode = client.get_mode().await;
        let clock_display = format!("{:>8}", ts % 1_000_000);
        let mode_display = match mode {
            ClientMode::Normal => format!("{:>8}", "Normal"),
            ClientMode::SSL(_, _, _, _) => format!("{:>8}", "SSL"),
            ClientMode::Initializing => format!("{:>8}", "Initializing"),
        };

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
                .constraints(
                    [
                        Constraint::Min(1),
                        Constraint::Length(10),
                        Constraint::Length(10),
                    ]
                    .as_ref(),
                )
                .split(chunks[0]);

            let target_str = {
                let peers = peers_snapshot;
                if target_index == -1 || target_index as usize >= peers.len() {
                    "broadcast".to_string()
                } else {
                    peers
                        .get_index(target_index as usize)
                        .map(|(ip, _)| ip.to_string())
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

            let mode_block = Paragraph::new(mode_display.as_str())
                .block(Block::default().borders(Borders::ALL).title("Mode"));
            f.render_widget(mode_block, top_chunks[2]);

            let message_lines: Vec<Line> = messages
                .iter()
                .cloned()
                .map(|msg| match msg.flag {
                    MessageFlag::Error => {
                        Line::from(Span::styled(msg.content, Style::default().fg(Color::Red)))
                    }
                    MessageFlag::Normal => Line::from(Span::styled(msg.content, Style::default())),
                    MessageFlag::Encrypted => {
                        let mut parts = msg.content.split(" ");
                        let prefix = parts.next().unwrap_or("[error]").to_string();
                        let msg = parts.collect::<Vec<_>>().join(" ");
                        Line::from(vec![
                            Span::styled(prefix + " ", Style::default().fg(Color::Green)),
                            Span::styled(msg, Style::default()),
                        ])
                    }
                    MessageFlag::Info => Line::from(vec![
                        Span::styled("<INFO> ", Style::default().fg(Color::Blue)),
                        Span::styled(msg.content, Style::default()),
                    ]),
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
