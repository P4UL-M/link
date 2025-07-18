use client_node::utils::addr::IpAddr;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Instant;

/// Converts (IpAddr, Instant) into a deterministic 16-char hex string
pub fn generate_validation_string(ip: &IpAddr, timestamp: &Instant) -> String {
    let duration = timestamp.elapsed().as_nanos(); // relative but unique per run
    let mut hasher = DefaultHasher::new();
    ip.hash(&mut hasher);
    duration.hash(&mut hasher);
    let hash = hasher.finish();
    format!("{:016x}", hash)
}
