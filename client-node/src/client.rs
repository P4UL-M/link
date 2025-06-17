use crate::packet::{Packet, PacketType, Serializable};
use crate::utils::ip::IntoIpv4String;
use futures::future::join_all;
use std::collections::HashMap;
use std::collections::HashSet;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tokio::sync::Notify;
use uuid::Uuid;

const PORT: u16 = 34254;
const INTERFACE: &str = "10.10.0.";

const DROPOUT_TEST_RATIO: f32 = 0.8; // 70% chance of packet loss
const DROP_DELAY_MS: u64 = 10; // Delay for dropout test

pub struct Client {
    socket: UdpSocket,
    pub my_ip: String,
    pending_acks: Arc<Mutex<HashMap<uuid::Uuid, Arc<Notify>>>>,
    seen_messages: Arc<Mutex<HashSet<(std::net::IpAddr, uuid::Uuid, std::time::Instant)>>>,
}

impl Client {
    pub async fn new() -> Self {
        let mut my_ip = None;
        for i in 2..=50 {
            let test_ip = format!("{}{}", INTERFACE, i);
            let bind_addr = format!("{}:{}", test_ip, PORT);
            if UdpSocket::bind(&bind_addr).await.is_ok() {
                my_ip = Some(test_ip);
                break;
            }
        }
        let my_ip = my_ip.expect("Failed to find a free IP address in the range");
        let bind_addr = format!("{}:{}", my_ip, PORT);

        let socket = UdpSocket::bind(&bind_addr)
            .await
            .expect("Failed to bind UDP socket");
        socket.set_broadcast(true).expect("Could not set broadcast");

        Client {
            socket,
            my_ip,
            pending_acks: Arc::new(Mutex::new(HashMap::new())),
            seen_messages: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub async fn discover(self: Arc<Self>) -> std::io::Result<()> {
        // Periodically send discovery packets
        tokio::spawn({
            let client = self.clone(); // Clone Arc for the task
            async move {
                loop {
                    let msg = Packet::new(
                        PacketType::Discovery,
                        client.my_ip.as_str(),
                        "255.255.255.255",
                        None,
                    );
                    client.send_to_broadcast(&msg.serialize()).await;
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
            }
        });
        Ok(())
    }

    pub async fn send_to<A: IntoIpv4String>(&self, msg: &[u8], addr: A) -> std::io::Result<usize> {
        // Simulate packet loss for dropout test only on request messages
        if rand::random::<f32>() < DROPOUT_TEST_RATIO && msg[16].eq(&(PacketType::Request as u8)) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Simulated packet loss",
            ));
        }
        // Simulate a delay for dropout test
        if rand::random::<f32>() < DROPOUT_TEST_RATIO && msg[16].eq(&(PacketType::Request as u8)) {
            tokio::time::sleep(Duration::from_millis(
                DROP_DELAY_MS + rand::random::<u64>() % (DROP_DELAY_MS * 4),
            ))
            .await;
        }
        // Utilisation du trait pour obtenir la chaîne d'adresse
        let addr_str = addr.into_ipv4_string();
        let addr = if addr_str.contains(':') {
            addr_str
        } else {
            format!("{}:{}", addr_str, PORT)
        };
        // Check if the address is valid
        let addr: std::net::SocketAddr = addr.parse().expect("Invalid address format");
        self.socket.send_to(msg, addr).await
    }

    pub async fn send_to_broadcast(&self, msg: &[u8]) {
        for i in 2..=50 {
            let ip = format!("{}{}", INTERFACE, i);
            if ip == self.my_ip {
                continue;
            }
            let addr = format!("{}:{}", ip, PORT);
            let _ = self.send_to(msg, addr).await;
        }
    }

    pub async fn send_reliable<A>(&self, msg: Packet, dest: A) -> std::io::Result<()>
    where
        for<'a> &'a A: IntoIpv4String,
    {
        let msg_id = msg.id;
        let notify = Arc::new(Notify::new());

        {
            let mut pending = self.pending_acks.lock().unwrap();
            pending.insert(msg_id, notify.clone());
        }

        let serialized = msg.serialize();

        for _ in 0..5 {
            let _ = self.send_to(&serialized, &dest).await;
            // Wait for ack or timeout
            let notified =
                tokio::time::timeout(Duration::from_millis(250), notify.notified()).await;
            if notified.is_ok() {
                // Acknowledgment received
                let mut pending = self.pending_acks.lock().unwrap();
                pending.remove(&msg_id);
                return Ok(());
            }
        }

        // Cleanup if no ack received
        let mut pending = self.pending_acks.lock().unwrap();
        pending.remove(&msg_id);
        Err(std::io::Error::new(
            std::io::ErrorKind::TimedOut,
            "No ACK received",
        ))
    }

    pub async fn send_multicast_reliable<A>(
        self: Arc<Self>,
        msg: Packet,
        addrs: Vec<A>,
    ) -> std::io::Result<()>
    where
        for<'a> &'a A: IntoIpv4String,
        A: std::marker::Send + std::marker::Sync + 'static,
    {
        let mut tasks = Vec::new();

        for addr in addrs {
            let msg_clone = msg.clone_with_new_id();
            let client = self.clone();

            // Spawn the send_reliable call as a future
            let task = tokio::spawn(async move { client.send_reliable(msg_clone, addr).await });

            tasks.push(task);
        }

        // Wait for all tasks to complete
        let results = join_all(tasks).await;

        // Check if any task failed or any send_reliable returned Err
        if results
            .iter()
            .any(|res| res.is_err() || res.as_ref().unwrap().is_err())
        {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Some sends were unsuccessful",
            ))
        } else {
            Ok(())
        }
    }

    pub async fn recv_from(
        &self,
        buf: &mut [u8],
    ) -> std::io::Result<(usize, std::net::SocketAddr)> {
        self.socket.recv_from(buf).await
    }

    pub fn should_process(&self, src_ip: IpAddr, msg_id: Uuid) -> bool {
        let now = Instant::now();
        let mut seen = self.seen_messages.lock().unwrap();

        // Clean old entries
        seen.retain(|&(_, _, timestamp)| now.duration_since(timestamp).as_secs() < 300);

        // Check for duplicate
        if seen
            .iter()
            .any(|(ip, id, _)| *ip == src_ip && *id == msg_id)
        {
            return false;
        }

        seen.insert((src_ip, msg_id, now));
        true
    }

    pub fn acknowledge(&self, msg_id: uuid::Uuid) {
        if let Ok(mut pending) = self.pending_acks.lock() {
            if let Some(notify) = pending.remove(&msg_id) {
                notify.notify_one();
            }
        }
    }

    pub async fn send_ack(&self, msg: &Packet, src: &str) {
        let ack_msg = Packet::new_with_id(
            msg.id,
            PacketType::Acknowledgment,
            self.my_ip.as_str(),
            msg.source,
            None,
        );
        let _ = self.send_to(&ack_msg.serialize(), src).await;
    }
}
