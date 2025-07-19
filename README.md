# Link - Distributed Fault-Tolerant Chat

A fault-tolerant point-to-point communication application implemented in Rust, allowing communication with other users on the same broadcast area (private network).

## Overview

Link is a distributed chat application with an intuitive and reactive interface that enables easy message exchange with other people. The system uses a custom **TCP over UDP implementation** to ensure communication reliability. If an authentication server is present on the network, nodes automatically switch to **SSL mode** with RSA encryption and identity verification.

## Project Requirements

This project meets the following requirements:
- **Message broadcasting** to up to 9 other nodes
- **Private messaging** between two nodes
- **Point-to-point UDP communication** between nodes
- **Ordered reception** of broadcast and private messages

## Architecture

The project is organized as a Rust workspace with a **decentralized architecture** where each node is an autonomous program:

### Client Node (`client-node/`)
- **Reactive TUI interface**: Interactive terminal with `ratatui` and intuitive controls
- **TCP-over-UDP protocol**: Custom implementation with acknowledgements and retry (5 attempts, 300ms timeout)
- **Lamport logical clock**: Causal message ordering with IP address resolution
- **Automatic discovery**: Discovery broadcasting every 2 seconds, 5-second timeout
- **Encrypted communication**: Hybrid RSA/AES encryption in SSL mode

### Authentication Server (`auth-server/`)
- **Centralized identity management**: RSA key registration and validation
- **Secure key exchange**: Challenge system to verify identities
- **Peer synchronization**: Centralized management of connected nodes list
- **Monitoring interface**: Real-time network monitoring with dedicated TUI

### Analysis and Testing Tools (`data/`)
- **Fault tolerance tests**: Packet loss simulation (0-99% dropout)
- **Latency tests**: Network delay simulation (10ms to 2.5s)
- **Multi-node tests**: Validation with up to 10 simultaneous nodes
- **Python analysis scripts**: Performance graphs and statistics generation
- **Test data**: CSV files and logs for post-mortem analysis

## Key Features

### **Fault Tolerance**
- **UDP reliability**: Acknowledgement system with automatic retry (5 max attempts)
- **Duplicate detection**: Prevention of multiple reception of same messages
- **Automatic cleanup**: Memory management with old message removal (30s)
- **Loss resistance**: Tolerance up to 40% packet loss without impact

### **Message Ordering**
- **Lamport clock**: Guarantees causal order of messages between all nodes
- **Conflict resolution**: Sorting by IP address for concurrent events
- **Automatic synchronization**: Clock initialization to current timestamp
- **FIFO Broadcast**: All messages from a node are delivered in sending order

### **Security (SSL Mode)**
- **Hybrid encryption**: RSA (521 bits) for key exchange, AES for data
- **Challenge authentication**: Identity validation via cryptographic challenge
- **Total confidentiality**: Messages unreadable without recipient's private key
- **Automatic switching**: Transparent SSL activation with auth server present

### **Network and Discovery**
- **P2P Discovery**: Auto-detection of nodes on local network (broadcast every 2s)
- **Direct communication**: Point-to-point without central server (normal mode)
- **Multi-platform support**: Tested on Efrei network and local configurations
- **Virtual network**: Automatic virtual LAN configuration (10.10.0.0/24)

## Installation and Configuration

### Prerequisites
- Rust 2024 edition
- macOS/Linux (tested on macOS)
- Sudo privileges for network configuration

### 1. Clone and compile
```bash
git clone <repository-url>
cd link
cargo build --release
```

### 2. Virtual network configuration
```bash
# Configure virtual LAN
chmod +x setup.sh
./setup.sh

# Clean up (after use)
./revert_virtual_lan.sh
```

### 3. Launch

#### Start authentication server
```bash
cd auth-server
cargo run
```

#### Launch clients
```bash
# Terminal 1
cd client-node
cargo run

# Terminal 2 (new client)
cd client-node 
cargo run
```

## Usage

### Client Interface (TUI Controls)
- **Tab**: Switch between channels (broadcast/private nodes)
- **Up/Down arrows**: Navigate message history
- **Standard keys**: Normal typing with backspace
- **Enter**: Send message
- **Escape**: Quit application

### Operation Modes

#### Normal Mode (Without auth server)
- Direct P2P communication between clients
- Discovery via UDP broadcast
- Plain text messages
- Self-managed peer list

#### SSL Mode (With auth server)
- Automatic registration with server
- Secure RSA key exchange
- AES message encryption
- Centralized peer synchronization
- Automatic fallback to normal mode if server unavailable

## Testing and Validation

### Fault tolerance analysis
The system has been rigorously tested with different scenarios:

#### Packet loss tests
- **0-40% dropout**: No message loss
- **99% dropout**: ~6% messages received (matches theory: 5 attempts × 1% = 5%)
- **Theoretical formula**: `P(received) = (1-d) × (1 + d + d² + d³ + d⁴)` where d = dropout rate

#### Latency and concurrency tests
- **Up to 2.5s delay**: Perfect ordering thanks to Lamport clock
- **10 simultaneous nodes**: 0% ordering error
- **6 concurrent senders**: Correct handling of >3500 messages with concurrent events

### Available test scenarios
- **Variable dropout**: 0%, 20%, 40%, 50%, 60%, 80%, 90%, 95%, 99%
- **Variable latency**: 10ms, 400ms, 800ms, 1000ms, 1600ms, 2500ms
- **Multi-node**: Tests with 2, 5, and 10 participants
- **Multi-sender**: Tests with 1, 2, and 6 simultaneous senders
- **High volume**: Tests with 1000+ messages for statistical validation

### Analysis scripts
```bash
cd data
python dropout_graph.py  # Generates performance graphs
```

## Technical Implementation

### Custom network protocol
The system implements a **reliability layer over UDP**:

```rust
// Example of send_reliable function
pub async fn send_reliable(&self, msg: Packet, dest: IpAddr) -> std::io::Result<()> {
    for _ in 0..5 {  // 5 maximum attempts
        self.send_to(&serialized, dest).await;
        // Wait for ACK or 300ms timeout
        if ack_received { return Ok(()); }
    }
    Err("No ACK received")  // Failure after 1.5s total
}
```

### Lamport clock ordering
- **Events**: Only message sending increments the clock
- **Synchronization**: Reception updates clock to max(local, received)
- **Sorting**: Order by timestamp, then by IP address on ties
- **Initialization**: Clock initialized to current timestamp (avoids "old" messages)

### Protocol Structure

#### Packet Types
- `Request`: Reliable communication request
- `Acknowledgment`: Acknowledgement receipt
- `Discovery`: Peer discovery (broadcast every 2s)
- `Broadcast`: General broadcast to all nodes
- `DNSDiscovery/Update/Acknowledgment`: Distributed DNS management

#### Data Formats
- `Text`: Plain text messages (normal mode)
- `SSLText`: AES encrypted messages (SSL mode)
- `KeyExchange`: RSA public key exchange  
- `KeyVerification/Validation`: Authentication challenge
- `KeyRequest/Response`: Request for specific peer's key
- `Peers`: Known peers list (synchronization)
- `Multipart`: Fragmented messages for large content

## Development

### Architecture Choices
- **Rust**: Precise memory management, safe concurrency, rigorous error handling
- **Autonomous nodes**: Each instance is an independent executable
- **Real sockets**: Communication only via UDP (no shared memory)
- **Asynchronous programming**: Task parallelization with `tokio`

### Parallelized Tasks
- Known peers cleanup
- Network packet listening  
- Peer discovery
- Message sending
- UI event processing
- Interface rendering
- Mode switching
- RSA key exchange
- Acknowledgement sending

### Code Structure
```
src/
├── main.rs              # Demonstration entry point
client-node/src/
├── lib.rs               # Library exports
├── main.rs              # Main TUI client with reactive interface
├── client.rs            # Client logic and state management
├── network_core.rs      # UDP network core + reliable protocol
├── packet.rs            # Packet definition and serialization
├── message.rs           # Message handling and history
└── utils/               # Utilities (IP, cryptography)
auth-server/src/
├── main.rs              # Authentication server with TUI
├── server.rs            # Server logic and key management
└── utils/               # Server utilities (crypto challenge)
data/                    # Analysis scripts and test data
├── dropout_graph.py     # Performance graph generation
├── lorem*.txt           # Test files for long messages
└── */                   # Result folders by scenario
```

### Component Communication
- **Mutex**: Protection of shared data (peers, keys, acknowledgements)
- **Channels**: Inter-task communication for UI events
- **Arc**: Reference sharing between async tasks
- **Notify**: Signaling for message acknowledgements

### Main Dependencies
- `tokio`: Asynchronous runtime for concurrency
- `ratatui` + `crossterm`: Reactive TUI interface
- `serde` + `bincode`: Packet serialization
- `rsa` + `aes` + `ctr`: Hybrid RSA/AES cryptography
- `uuid`: Unique identifiers for messages
- `indexmap`: Ordered collections for peers
- `futures`: Tools for asynchronous programming

## Project Link

🔗 **GitHub**: https://github.com/P4UL-M/link

## Authors

Developed by **Paul Mairesse** as part of an academic project on fault-tolerant distributed systems.