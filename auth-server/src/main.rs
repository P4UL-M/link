use rand;
use std::collections::HashMap;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn main() {
    // Simulate a unique client ID
    let my_id = rand::random::<u16>();
    println!("Client ID: {}", my_id);

    // Bind socket to listen on all interfaces, using a specific port
    let socket = UdpSocket::bind("0.0.0.0:34254").expect("Failed to bind random port");
    println!(
        "Client {} listening on {}",
        my_id,
        socket.local_addr().unwrap()
    );
    socket.set_broadcast(true).expect("Could not set broadcast");

    let clients = Arc::new(Mutex::new(
        HashMap::<_, (u16, u16, std::time::Instant)>::new(),
    ));
    let clients_render = Arc::clone(&clients);

    thread::spawn(move || {
        loop {
            {
                let mut clients = clients_render.lock().unwrap();
                let now = std::time::Instant::now();
                clients.retain(|_, &mut (_, _, timestamp)| {
                    now.duration_since(timestamp) <= Duration::from_secs(3)
                });
                print!("\x1B[2J\x1B[1;1H"); // Clear screen
                println!("--- Known Clients ---");
                for (addr, (id, port, _)) in clients.iter() {
                    println!("{} -> id: {}, port: {}", addr, id, port);
                }
            }
            thread::sleep(Duration::from_millis(1000));
        }
    });

    // Receiver: listens for broadcasts from others
    let mut buf = [0; 512];
    loop {
        match socket.recv_from(&mut buf) {
            Ok((n, src)) => {
                if n >= 4 {
                    let client_id = u16::from_be_bytes([buf[0], buf[1]]);
                    let port = u16::from_be_bytes([buf[2], buf[3]]);
                    // println!("Discovered peer {}: id={} port={}", src, client_id, port);
                    let mut map = clients.lock().unwrap();
                    map.insert(src, (client_id, port, std::time::Instant::now()));
                }
            }
            Err(e) => eprintln!("Receive error: {}", e),
        }
    }
}
