# Tracker

**Tracker** is a lightweight, always-on Rust-based tracker that indexes fidelity transactions and allows peers to query known addresses.

It uses **Tokio** as the async runtime for networking and background tasks, and integrates with **Tor** for private, decentralized communication with other services.  

---

## Tools Required

- **SQLite** (for local storage)  
- **Tor** (for peer connectivity and onion address handling)  
- **Rust** (toolchain and Cargo)  

### Install SQLite
**Linux (Debian/Ubuntu):**
```bash
sudo apt update
sudo apt install libsqlite3-dev
```

**macOS:**
```bash
brew install sqlite
# If you hit OpenSSL errors:
brew install openssl
```

### Install Tor
Follow this doc: [Tor Setup Guide](https://github.com/citadel-tech/coinswap/blob/master/docs/tor.md)

### Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

---

## Run the Project
```bash
cargo build
cargo run
```

---

## Current Structure

```
tracker/
├── migrations/              # Database migrations
├── src/
│   ├── db/                  # Database module (contains schema,model)
│   ├── indexer/             # Indexer module
│   │   ├── tracker_indexer/ # Tracker indexer
│   │   └── utxo_indexer/    # UTXO indexer
│   ├── server/              # Server module (client requests, peer monitoring)
│   ├── error.rs             # Error definitions
│   ├── handle_error.rs      # Error handling utilities
│   ├── lib.rs               # Library entry point
│   ├── main.rs              # Binary entry point
│   ├── status.rs            # Tracker status management
│   ├── tor.rs               # Tor integration helpers
│   ├── types.rs             # Shared types and request/response enums
│   └── utils.rs             # Utility functions
├── Cargo.toml               # Dependencies and project config
└── README.md
```

---

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌────────────────────────┐
│     Database    │    │     Tracker     │    │        Indexer         │
|                 |◄───┤                 │───►│                        │
└─────────────────┘    └─────────────────┘    └────────────────────────┘
                                │                    │
                       ┌────────▼────────┐           │
                       │     Server      │           │
                       │                 │           │
                       └─────────────────┘           │           
                ┌──────────────────────┐             │
                │  tracker_indexer     │◄────────────┘
                └──────────────────────┘             │
                ┌──────────────────────┐             │
                │    utxo_indexer      │◄────────────┘
                └──────────────────────┘

                              
                      
```

---

## Modules Overview

### 1. Database (db)
- Stores mempool transactions, inputs, UTXOs, and known servers.  
- (may use SQL in future)

### 2. Indexer
- Divided into two sub-indexers:  
  - **tracker_indexer**: Handles transaction indexing related to trackers.  
  - **utxo_indexer**: Manages UTXO and block indexing.  

### 3. Server
- Handles incoming client requests (`Get`, `Watch`),etc.  
- Monitors peer trackers via Tor (ping/pong).  
- Keeps peer status updated in the database.  

---

## Why Tor?
- Connects securely to **makers** over onion addresses.  
- Extracts/stores onion addresses for peer discovery.  
- Ensures tracker communication is private and censorship-resistant.  

---

## Contributing

Contributions are welcome!  

- **Fork** the repository  
- **Create a branch** for your feature or fix  
- **Open a Pull Request** with your changes  

For bugs or feature requests:  
- **Open an Issue** in the repository  

This project is open-source and open for improvements.  

---

