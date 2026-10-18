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
    /// Native path (T6③): drop-in raw package data root for the loader.
    pub native_packages_root: PathBuf,
    /// Native path (T6③): versioned package store for installed native packs.
    pub native_store_root: PathBuf,
    /// NC09：房间后端选择——true 时房间使用**原生权威会话**（`room_native_actor`），
    /// false 时沿用旧线 `engine_core` 房间。默认关闭（切换需显式启用并逐项验收）。
    pub native_rooms: bool,
    /// NC09：原生房间启用的包 id（按命名空间限定见 `native_room_packages`）。
    pub native_room_packages: Vec<String>,
    /// NC09：原生房间包所属命名空间。
    pub native_room_namespace: String,
    /// NC09：原生房间落盘目录（空串 = 不落盘）。
    pub native_room_store: String,
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
            native_packages_root: PathBuf::from("packages/extensions"),
            native_store_root: PathBuf::from("data/native-content"),
            native_rooms: false,
            native_room_packages: Vec::new(),
            native_room_namespace: "upstream-noname".to_string(),
            native_room_store: "data/native-rooms".to_string(),
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
        if let Ok(path) = std::env::var("NONAME_NATIVE_PACKAGES_ROOT") {
            c.native_packages_root = path.into();
        }
        if let Ok(path) = std::env::var("NONAME_NATIVE_STORE_ROOT") {
            c.native_store_root = path.into();
        }
        if let Ok(value) = std::env::var("NONAME_NATIVE_ROOMS") {
            c.native_rooms = matches!(value.as_str(), "1" | "true" | "yes");
        }
        if let Ok(list) = std::env::var("NONAME_NATIVE_ROOM_PACKAGES") {
            c.native_room_packages = list
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
        if let Ok(ns) = std::env::var("NONAME_NATIVE_ROOM_NAMESPACE") {
            c.native_room_namespace = ns;
        }
        if let Ok(dir) = std::env::var("NONAME_NATIVE_ROOM_STORE") {
            c.native_room_store = dir;
        }
        c.extension_verify_key = std::env::var("NONAME_EXTENSION_VERIFY_KEY").ok();
        if let Ok(value) = std::env::var("NONAME_ALLOW_UNSIGNED_EXTENSIONS") {
            c.allow_unsigned_extensions = matches!(value.as_str(), "1" | "true" | "yes");
        }
        c
    }
}
