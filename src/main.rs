use client_node::{
    packet::{Packet, PacketData, PacketType},
    utils::{addr::IpAddr, key::Key},
};

fn main() {
    println!("Hello, world!");

    let key = Key::generate();

    let msg = Packet::new(
        PacketType::Request,
        IpAddr::from("127.0.0.1"),
        IpAddr::from("192.168.1.1"),
        0,
        PacketData::KeyExchange(key.to_serializable()),
    );

    let serialized = msg.serialize().unwrap();
    println!("Serialized Packet: {:?}", serialized);

    // Display serialized packet size
    println!("Serialized Packet Size: {} bytes", serialized.len());

    let deserialized: Packet = Packet::deserialize(&serialized).unwrap();
    println!("Deserialized Packet: {:?}", deserialized);
}
