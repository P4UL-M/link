use client_node::client::Client;
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

    // Create a new client instance
    let client = Client::new(my_id);
}
