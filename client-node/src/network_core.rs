use crate::packet::{Packet, PacketData, PacketType};
use crate::utils::addr::IpAddr;
use futures::future::join_all;
use std::collections::HashMap;
use std::collections::HashSet;
use std::io::{Error, ErrorKind};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use tokio::sync::Notify;
use tokio::time::{Duration, Instant};
use uuid::Uuid;

const PORT: u16 = 34254;
const INTERFACE: [u8; 4] = [10, 10, 0, 0];

const DROPOUT_TEST_RATIO: f32 = 0.0; // 0% chance of packet loss
const DROP_DELAY_MS: u64 = 20; // Delay for dropout test
const NO_CLOCK_SENTINEL: u64 = 0; // Sentinel value for no clock
const PACKET_SIZE: usize = 512; // Maximum size for a packet

pub struct NetworkCore {
    socket: UdpSocket,
    pub my_ip: IpAddr,
    pending_acks: Arc<Mutex<HashMap<Uuid, Arc<Notify>>>>,
    seen_messages: Arc<Mutex<HashSet<(IpAddr, Uuid, Instant)>>>,
    clock: Mutex<u64>, // Logical clock for the NetworkCore
}

impl NetworkCore {
    pub async fn new() -> Self {
        let mut my_ip = None;
        for i in 2..=50 {
            let test_ip = IpAddr::from_interface(INTERFACE, i as u8);
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

        // Get current time in milliseconds since UNIX epoch
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        NetworkCore {
            socket,
            my_ip,
            pending_acks: Arc::new(Mutex::new(HashMap::new())),
            seen_messages: Arc::new(Mutex::new(HashSet::new())),
            clock: Mutex::new(now), // Initialize the logical clock
        }
    }

    pub async fn send_to(&self, msg: &[u8], addr: IpAddr) -> std::io::Result<usize> {
        // Simulate packet loss for dropout test
        if rand::random::<f32>() < DROPOUT_TEST_RATIO {
            return Err(Error::new(ErrorKind::Other, "Simulated packet loss"));
        }
        // Simulate a delay for dropout test
        if rand::random::<f32>() < DROPOUT_TEST_RATIO {
            tokio::time::sleep(Duration::from_millis(
                DROP_DELAY_MS + rand::random::<u64>() % (DROP_DELAY_MS * 4),
            ))
            .await;
        }
        // Check if the packet is too large
        if msg.len() > PACKET_SIZE {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Packet size exceeds maximum limit",
            ));
        }
        // Convert the address to a socket address
        let addr = addr.to_socket_addr(PORT);
        self.socket.send_to(msg, addr).await
    }

    pub async fn send_to_broadcast(&self, msg: &[u8]) {
        for i in 2..=50 {
            let addr = IpAddr::from_interface(INTERFACE, i as u8);
            if addr == self.my_ip {
                continue;
            }
            let _ = self.send_to(msg, addr).await;
        }
    }

    pub async fn send_reliable(&self, msg: Packet, dest: IpAddr) -> std::io::Result<()> {
        let msg_id = msg.id;
        let notify = Arc::new(Notify::new());

        {
            let mut pending = self.pending_acks.lock().await;
            pending.insert(msg_id, notify.clone());
        }

        let serialized = msg.serialize().unwrap();

        for _ in 0..5 {
            let _ = self.send_to(&serialized, dest).await;
            // Wait for ack or timeout
            let notified =
                tokio::time::timeout(Duration::from_millis(300), notify.notified()).await;
            if notified.is_ok() {
                // Acknowledgment received
                let mut pending = self.pending_acks.lock().await;
                pending.remove(&msg_id);
                return Ok(());
            }
        }

        // Cleanup if no ack received
        let mut pending = self.pending_acks.lock().await;
        pending.remove(&msg_id);
        Err(Error::new(ErrorKind::TimedOut, "No ACK received"))
    }

    pub async fn send_multicast_reliable(
        self: Arc<Self>,
        msg: Packet,
        addrs: Vec<IpAddr>,
    ) -> std::io::Result<()> {
        let mut tasks = Vec::new();

        for addr in addrs {
            let msg_clone = msg.clone_with_new_id();
            let net_core = self.clone();

            // Spawn the send_reliable call as a future
            let task = tokio::spawn(async move { net_core.send_reliable(msg_clone, addr).await });

            tasks.push(task);
        }

        // Wait for all tasks to complete
        let results = join_all(tasks).await;

        // Check if any task failed or any send_reliable returned Err
        if results
            .iter()
            .any(|res| res.is_err() || res.as_ref().unwrap().is_err())
        {
            Err(Error::new(ErrorKind::Other, "Some sends were unsuccessful"))
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

    pub async fn should_process(&self, src_ip: IpAddr, msg_id: Uuid) -> bool {
        let now = Instant::now();
        let mut seen = self.seen_messages.lock().await;

        // Clean old entries
        seen.retain(|&(_, _, timestamp)| now.duration_since(timestamp).as_secs() < 30);

        let is_new = seen.insert((src_ip, msg_id, now));
        is_new
    }

    pub async fn acknowledge(&self, msg_id: uuid::Uuid) {
        let mut pending = self.pending_acks.lock().await;
        if let Some(notify) = pending.remove(&msg_id) {
            notify.notify_one();
        }
    }

    pub async fn send_ack(&self, msg: &Packet, src: IpAddr) {
        let ack_msg = Packet::new_with_id(
            msg.id,
            PacketType::Acknowledgment,
            self.my_ip,
            msg.source,
            NO_CLOCK_SENTINEL,
            PacketData::Empty,
        );
        let _ = self.send_to(&ack_msg.serialize().unwrap(), src).await;
    }

    pub async fn send_dns_ack(&self, msg: &Packet, src: IpAddr) {
        let ack_msg = Packet::new_with_id(
            msg.id,
            PacketType::DNSAcknowledgment,
            self.my_ip,
            src,
            NO_CLOCK_SENTINEL,
            PacketData::Empty,
        );
        let _ = self.send_to(&ack_msg.serialize().unwrap(), src).await;
    }

    pub async fn tick(&self) -> u64 {
        // Increment the logical clock
        let mut clock = self.clock.lock().await;
        *clock = clock.wrapping_add(1);
        *clock
    }

    pub async fn observe(&self, timestamp: u64) {
        // Update the logical clock based on observed timestamp
        let mut clock = self.clock.lock().await;
        if timestamp > *clock {
            *clock = timestamp;
        }
    }

    pub async fn get_clock(&self) -> u64 {
        // Get the current value of the logical clock
        let clock = self.clock.lock().await;
        *clock
    }
}
