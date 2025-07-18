use client_node::network_core::NetworkCore;
use client_node::packet::{Packet, PacketData, PacketType};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

pub struct Server {
    network_core: Arc<NetworkCore>,
    cancellation_token: Arc<Mutex<Option<CancellationToken>>>,
}

impl Server {
    pub fn new(network_core: Arc<NetworkCore>) -> Self {
        Server {
            network_core,
            cancellation_token: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn discover(&self) -> std::io::Result<()> {
        // Periodically send discovery packets
        let token = CancellationToken::new();
        {
            let mut guard = self.cancellation_token.lock().await;
            *guard = Some(token.clone());
        }
        tokio::spawn({
            let client = self.network_core.clone(); // Clone Arc for the task
            async move {
                while !token.is_cancelled() {
                    let ts = client.tick().await; // Increment the logical clock
                    let msg: Packet = Packet::new(
                        PacketType::DNSDiscovery,
                        client.my_ip,
                        "255.255.255.255".into(),
                        ts,
                        PacketData::Empty,
                    );
                    client.send_to_broadcast(&msg.serialize().unwrap()).await;
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
            }
        });
        Ok(())
    }

    pub async fn run(&self) -> std::io::Result<()> {
        // Start the discovery process
        self.discover().await?;

        // Here you can add more server logic, such as handling incoming packets
        // or managing connections.

        Ok(())
    }

    pub fn get_network_core(&self) -> Arc<NetworkCore> {
        self.network_core.clone()
    }

    pub async fn stop_discovery(&self) {
        if let Some(token) = self.cancellation_token.lock().await.take() {
            token.cancel();
        }
    }
}
