use client_node::client::Client;
use client_node::packet::{PACKET_SIZE, Packet, PacketData, PacketType};
use client_node::utils::addr::IpAddr;
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
use std::io;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

mod server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Simulate a unique client ID
    let my_id = rand::random::<u16>();
    println!("Client ID: {}", my_id);

    // Create a new client instance
    let client = Arc::new(Client::new().await);
    let server = Server::new(client);

    // Start the server discovery process
    server.discover().await?;

    let known_peers: Arc<Mutex<IndexMap<IpAddr, Instant>>> = Arc::new(Mutex::new(IndexMap::new()));

    // Task 0: Periodic cleanup of known peers
    let cleanup_client = server.get_client();
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
                let ts = cleanup_client.tick().await;
                let peers: Vec<IpAddr> = peers.keys().cloned().collect();
                let msg = Packet::new(
                    PacketType::DNSUpdate,
                    cleanup_client.my_ip,
                    "255.255.255.255".into(),
                    ts,
                    PacketData::Peers(peers.clone()),
                );
                let _ = cleanup_client
                    .clone()
                    .send_multicast_reliable(msg, peers)
                    .await;
            }
        }
    });

    // Task 1: Listen for incoming packets
    let client_recv = server.get_client();
    let peers_recv = known_peers.clone();
    tokio::spawn(async move {
        let mut buf = [0u8; PACKET_SIZE];
        loop {
            if let Ok((n, _src)) = client_recv.recv_from(&mut buf).await {
                let msg: Packet = Packet::deserialize(&buf[..n]).unwrap();
                match msg.type_ {
                    PacketType::Acknowledgment => {
                        let mut peers = peers_recv.lock().await;
                        if let None = peers.insert(msg.source, Instant::now()) {
                            // Send update to all clients
                            let ts = client_recv.tick().await;
                            let peers: Vec<IpAddr> = peers.keys().cloned().collect();
                            let msg = Packet::new(
                                PacketType::DNSUpdate,
                                client_recv.my_ip,
                                "255.255.255.255".into(),
                                ts,
                                PacketData::Peers(peers.clone()),
                            );
                            let _ = client_recv
                                .clone()
                                .send_multicast_reliable(msg, peers)
                                .await;
                        }
                    }
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
