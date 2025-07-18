use client_node::network_core::NetworkCore;
use client_node::packet::{Packet, PacketData, PacketType};
use client_node::utils::addr::IpAddr;
use client_node::utils::key::Key;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Duration, Instant};
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Status {
    Initializing,
    Ready,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ClientMode {
    Initializing,
    Normal,
    SSL(Key, IpAddr, Status, Instant), // SSL mode with a key and timestamp
}

pub struct Client {
    discovery_token: Arc<Mutex<Option<CancellationToken>>>,
    network_core: Arc<NetworkCore>, // Network core for the client
    mode: Arc<Mutex<ClientMode>>,   // Client mode
}

impl Client {
    pub fn new(network_core: Arc<NetworkCore>) -> Self {
        Client {
            network_core,
            discovery_token: Arc::new(Mutex::new(None)), // Token for discovery cancellation
            mode: Arc::new(Mutex::new(ClientMode::Initializing)), // Default mode
        }
    }

    pub fn get_network_core(&self) -> Arc<NetworkCore> {
        self.network_core.clone()
    }

    pub async fn discover(&self) -> std::io::Result<()> {
        // Periodically send discovery packets
        let token = CancellationToken::new();
        *self.discovery_token.lock().await = Some(token.clone());
        // Periodically send discovery packets
        tokio::spawn({
            let net_core = self.network_core.clone(); // Clone Arc for the task
            async move {
                while !token.is_cancelled() {
                    let ts = net_core.tick().await; // Increment the logical clock
                    let msg: Packet = Packet::new(
                        PacketType::Discovery,
                        net_core.my_ip,
                        "255.255.255.255".into(),
                        ts,
                        PacketData::Empty,
                    );
                    net_core.send_to_broadcast(&msg.serialize().unwrap()).await;
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
            }
        });
        Ok(())
    }

    pub async fn stop_discovery(&self) {
        if let Some(token) = self.discovery_token.lock().await.take() {
            token.cancel();
        }
    }

    pub async fn is_discovering(&self) -> bool {
        self.discovery_token.lock().await.is_some()
    }

    pub async fn set_mode(self: Arc<Self>, mode: ClientMode) {
        {
            let mut guard = self.mode.lock().await;
            *guard = mode;
        }
        let mode_snapshot = self.get_mode().await;
        self.handle_mode_change(mode_snapshot).await;
    }

    async fn handle_mode_change(self: Arc<Self>, mode: ClientMode) {
        // Handle the change in client mode
        match mode {
            ClientMode::Initializing => {}
            ClientMode::Normal => {
                // Logic for normal mode
                if !self.is_discovering().await {
                    self.discover().await.expect("Failed to start discovery");
                }
            }
            ClientMode::SSL(key, server_ip, _, _) => {
                // Logic for SSL mode
                if self.is_discovering().await {
                    self.stop_discovery().await;

                    // Handle SSL not ready state
                    // Start SSL operations with the provided key
                    let message = Packet::new(
                        PacketType::Request,
                        self.network_core.my_ip,
                        server_ip,
                        self.network_core.get_clock().await,
                        PacketData::KeyExchange(key.to_serializable()),
                    );
                    let _ = self.network_core.send_reliable(message, server_ip).await;

                    let client = self.clone();
                    // Start fallback timer
                    tokio::spawn(async move {
                        tokio::time::sleep(Duration::from_secs(5)).await;
                        if let ClientMode::SSL(_, _, status, _) = client.get_mode().await {
                            if status != Status::Ready {
                                *client.mode.lock().await = ClientMode::Normal;
                                if !client.is_discovering().await {
                                    client
                                        .discover()
                                        .await
                                        .expect("Failed to restart discovery");
                                }
                            }
                        }
                    });
                }
            }
        }
    }

    pub async fn get_mode(&self) -> ClientMode {
        // Get the current mode of the client
        let mode = self.mode.lock().await;
        mode.clone()
    }
}
