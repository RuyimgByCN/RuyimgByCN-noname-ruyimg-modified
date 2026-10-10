//! Server configuration.

use std::{net::SocketAddr, path::PathBuf};

/// Runtime config.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Listen address.
    pub listen_addr: SocketAddr,
    /// Max concurrent rooms.
    pub max_rooms: usize,
    /// Mailbox size per room.
    pub room_mailbox: usize,
    /// Persist snapshot every N accepted commands.
    pub snapshot_every: u64,
    /// Soft rate limit: max submits per connection per second.
    pub max_cmds_per_sec: u32,
    /// Read-only original content catalog and asset root.
    pub original_content_root: PathBuf,
    /// Versioned installed package store.
    pub content_store_root: PathBuf,
    /// Drop-in extension directories scanned at process startup.
    pub extensions_root: PathBuf,
    /// Optional verification key for signed drop-in extensions.
    pub extension_verify_key: Option<String>,
    /// Whether local unsigned drop-in extensions are allowed.
    pub allow_unsigned_extensions: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            listen_addr: "0.0.0.0:7700".parse().unwrap(),
            max_rooms: 1024,
            room_mailbox: 256,
            snapshot_every: 20,
            max_cmds_per_sec: 30,
            original_content_root: PathBuf::from("content/original"),
            content_store_root: PathBuf::from("data/server-content"),
            extensions_root: PathBuf::from("extensions"),
            extension_verify_key: None,
            allow_unsigned_extensions: cfg!(debug_assertions),
        }
    }
}

impl ServerConfig {
    /// Load from environment with defaults.
    pub fn from_env() -> Self {
        let mut c = Self::default();
        if let Ok(a) = std::env::var("GAME_SERVER_ADDR") {
            if let Ok(addr) = a.parse() {
                c.listen_addr = addr;
            }
        }
        if let Ok(n) = std::env::var("GAME_SERVER_MAX_ROOMS") {
            if let Ok(v) = n.parse() {
                c.max_rooms = v;
            }
        }
        if let Ok(path) = std::env::var("NONAME_ORIGINAL_CONTENT_ROOT") {
            c.original_content_root = path.into();
        }
        if let Ok(path) = std::env::var("NONAME_CONTENT_STORE_ROOT") {
            c.content_store_root = path.into();
        }
        if let Ok(path) = std::env::var("NONAME_EXTENSIONS_ROOT") {
            c.extensions_root = path.into();
        }
        c.extension_verify_key = std::env::var("NONAME_EXTENSION_VERIFY_KEY").ok();
        if let Ok(value) = std::env::var("NONAME_ALLOW_UNSIGNED_EXTENSIONS") {
            c.allow_unsigned_extensions = matches!(value.as_str(), "1" | "true" | "yes");
        }
        c
    }
}
