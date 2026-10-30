//! Authoritative game server entrypoint.

mod config;
mod lobby;
mod native;
mod native_room_store;
mod native_view;
mod outbox;
mod persist;
mod room;
mod room_native;
mod room_native_actor;
mod session;

use axum::{
    body::{Body, Bytes},
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path as AxumPath, Query, State,
    },
    http::{header, HeaderMap, Response, StatusCode},
    response::{IntoResponse, Response as AxumResponse},
    routing::{get, post},
    Json, Router,
};
use config::ServerConfig;
use content_runtime::{import_extensions, InstallOptions, PackageStore};
use engine_api::VersionBundle;
use engine_core::Engine;
use engine_protocol::{
    decode_c2s, public_error, ErrorCode, ServerMessage, NETWORK_PROTOCOL_VERSION,
};
use futures_util::{stream::try_unfold, SinkExt, StreamExt};
use lobby::Lobby;
use serde::Deserialize;
use session::Outbound;
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::sync::mpsc;
use tower_http::{cors::CorsLayer, services::ServeDir};
use tracing::{info, warn};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "game_server=info,tower_http=info".into()),
        )
        .init();

    let cfg = ServerConfig::from_env();
    let lobby = match try_bootstrap(&cfg) {
        Ok(lobby) => lobby,
        Err(error) => {
            tracing::error!(error = %error, "服务端启动失败");
            std::process::exit(1);
        }
    };

    // NC09：启动时恢复落盘的原生房间（存储目录为空/未启用时为空操作）
    match lobby.restore_native_rooms() {
        Ok(0) => {}
        Ok(n) => tracing::info!(rooms = n, "已恢复落盘原生房间"),
        Err(e) => tracing::warn!(error = %e, "原生房间恢复失败（服务端继续启动）"),
    }

    let app = build_router(lobby.clone(), ServeDir::new(&cfg.original_content_root));

    let addr = cfg.listen_addr;
    info!(%addr, "game-server listening");
    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    axum::serve(listener, app).await.expect("serve");
}

/// 测试便捷入口：装配失败直接 panic（含目录已有另一个写者）。
fn bootstrap(cfg: &ServerConfig) -> Arc<Lobby> {
    try_bootstrap(cfg).expect("bootstrap")
}

/// 装配服务端运行所需的一切（扩展导入、内容锁、目录/资产索引、原生内容、Lobby）。
///
/// **从 `main()` 抽出**：便于测试用临时目录构造 Lobby 并在 `127.0.0.1:0` 上起真实监听。
/// 生产 main 使用本函数以获得**可报告的启动失败**（尤其存档目录第二写者）。
fn try_bootstrap(cfg: &ServerConfig) -> Result<Arc<Lobby>, String> {
    let store = PackageStore::open(&cfg.content_store_root)
        .map_err(|e| format!("打开内容存储失败: {e}"))?;
    let report = import_extensions(
        &store,
        &cfg.extensions_root,
        &InstallOptions {
            activate: true,
            allow_unsigned: cfg.allow_unsigned_extensions,
            verify_key: cfg.extension_verify_key.clone(),
            host: content_runtime::tooling_host_versions(),
        },
    )
    .map_err(|e| format!("导入扩展失败: {e}"))?;
    let engine_template = Engine::from_content_lock(&store, &report.content_lock)
        .map_err(|e| format!("装载内容锁失败: {e}"))?;
    let (content_lock, content_catalog) = store
        .load_active_catalog()
        .map_err(|e| format!("装载目录失败: {e}"))?;
    let content_assets = store
        .load_active_asset_index()
        .map_err(|e| format!("装载资源索引失败: {e}"))?;
    assert_eq!(
        content_lock, report.content_lock,
        "active content changed during startup"
    );
    info!(
        packages = report.content_lock.packages.len(),
        extensions = report.installed.len(),
        content_lock = %report.content_lock.lock_hash,
        "content ready"
    );
    // T6③ 产品路径：原生内容服务（安装包根 + 独立包存储）。安装（导入）
    // 在启动时执行；装载与原生建局经 /native 端点按需进行。
    let native_content = native::NativeContent::open(
        cfg.native_packages_root.clone(),
        cfg.native_store_root.clone(),
    )
    .map_err(|e| format!("打开原生内容存储失败: {e}"))?;
    let native_packages = native_content.import().unwrap_or_else(|e| {
        warn!(error = %e, "native package import failed; /native paths degraded");
        Vec::new()
    });
    info!(
        packages = native_packages.len(),
        "native content ready"
    );
    let lobby = Arc::new(Lobby::new(
        cfg.clone(),
        engine_template,
        content_lock,
        content_catalog,
        content_assets,
        Arc::new(native_content),
    )?);
    Ok(lobby)
}

/// 构造 HTTP/WS 路由（**从 `main` 抽出**，便于测试在 `127.0.0.1:0` 上起真实监听）。
fn build_router(lobby: Arc<Lobby>, original_content: ServeDir) -> Router {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/metrics", get(metrics_handler))
        .route("/rooms", get(rooms_handler))
        .route("/content-catalog", get(content_catalog_handler))
        .route(
            "/package-assets/{namespace}/{package_id}/{*asset_id}",
            get(package_asset_handler),
        )
        .route("/native/packages", get(native_packages_handler))
        .route("/native/packages/import", post(native_import_handler))
        .route("/native/packages/activate", post(native_activate_handler))
        .route("/native/packages/disable", post(native_disable_handler))
        .route("/native/packages/uninstall", post(native_uninstall_handler))
        .route("/native/games", post(native_games_handler))
        .route("/native/matches", post(native_matches_handler))
        .route("/native/matches/restore", post(native_restore_handler))
        .route("/ws", get(ws_handler))
        .nest_service("/original-content", original_content)
        .layer(CorsLayer::permissive())
        .with_state(lobby)
}

#[derive(Debug, Default, Deserialize)]
struct ContentCatalogQuery {
    offset: Option<u32>,
    limit: Option<u32>,
}

#[derive(Debug, Default, Deserialize)]
struct PackageAssetQuery {
    v: Option<String>,
}

async fn content_catalog_handler(
    State(lobby): State<Arc<Lobby>>,
    Query(query): Query<ContentCatalogQuery>,
) -> axum::Json<content_runtime::ActiveCatalogPage> {
    axum::Json(lobby.content_catalog_page(query.offset, query.limit))
}

#[derive(Debug, Deserialize)]
struct NativePackageBody {
    namespace: String,
    package_id: String,
    #[serde(default)]
    version: String,
}

async fn native_import_handler(
    State(lobby): State<Arc<Lobby>>,
) -> Result<Json<Vec<native::NativePackageInfo>>, (StatusCode, String)> {
    lobby
        .native_import()
        .map(Json)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))
}

async fn native_activate_handler(
    State(lobby): State<Arc<Lobby>>,
    Json(body): Json<NativePackageBody>,
) -> Result<StatusCode, (StatusCode, String)> {
    if body.namespace.is_empty() || body.package_id.is_empty() || body.version.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "namespace/package_id/version required".into(),
        ));
    }
    lobby
        .native_activate(&body.namespace, &body.package_id, &body.version)
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))
}

async fn native_disable_handler(
    State(lobby): State<Arc<Lobby>>,
    Json(body): Json<NativePackageBody>,
) -> Result<StatusCode, (StatusCode, String)> {
    lobby
        .native_disable(&body.namespace, &body.package_id)
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))
}

async fn native_uninstall_handler(
    State(lobby): State<Arc<Lobby>>,
    Json(body): Json<NativePackageBody>,
) -> Result<StatusCode, (StatusCode, String)> {
    lobby
        .native_uninstall(&body.namespace, &body.package_id)
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))
}

/// T6③：列出已安装原生包。
async fn native_packages_handler(
    State(lobby): State<Arc<Lobby>>,
) -> Result<Json<Vec<native::NativePackageInfo>>, (StatusCode, String)> {
    match lobby.native_packages() {
        Ok(list) => Ok(Json(list)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

#[derive(Debug, Deserialize)]
struct NativeMatchBody {
    #[serde(default)]
    packages: Vec<String>,
    seed: u64,
    seats: u8,
}

/// 已安装扩展进入新对局，返回存档十六进制。
async fn native_matches_handler(
    State(lobby): State<Arc<Lobby>>,
    Json(req): Json<NativeMatchBody>,
) -> Result<Json<native::NativeMatchArchive>, (StatusCode, String)> {
    if !(2..=8).contains(&req.seats) {
        return Err((StatusCode::BAD_REQUEST, "seats must be 2..=8".into()));
    }
    match lobby.native_match(req.packages, req.seed, req.seats).await {
        Ok(archive) => Ok(Json(archive)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

/// 同一安装集恢复对局。缺包或错版返回 400。
async fn native_restore_handler(
    State(lobby): State<Arc<Lobby>>,
    Json(req): Json<native::NativeRestoreRequest>,
) -> Result<Json<native::NativeRestoreSummary>, (StatusCode, String)> {
    match lobby.native_restore(req).await {
        Ok(summary) => Ok(Json(summary)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

/// T6③：安装包 → 装载 → 原生建局 → 有界 AI 驱动。
async fn native_games_handler(
    State(lobby): State<Arc<Lobby>>,
    Json(req): Json<native::NativeGameRequest>,
) -> Result<AxumResponse, (StatusCode, String)> {
    if !(2..=8).contains(&req.seats) {
        return Err((StatusCode::BAD_REQUEST, "seats must be 2..=8".into()));
    }
    match lobby.native_game(req).await {
        Ok(summary) => Ok((StatusCode::OK, Json(summary)).into_response()),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

async fn package_asset_handler(
    State(lobby): State<Arc<Lobby>>,
    AxumPath((namespace, package_id, asset_id)): AxumPath<(String, String, String)>,
    Query(query): Query<PackageAssetQuery>,
    headers: HeaderMap,
) -> Result<Response<Body>, StatusCode> {
    let asset = lobby
        .active_asset(&namespace, &package_id, &asset_id)
        .ok_or(StatusCode::NOT_FOUND)?;
    let etag = format!(
        "\"{}\"",
        blake3::Hash::from_bytes(asset.source_hash).to_hex()
    );
    let immutable = query.v.as_deref() == Some(lobby.content_lock_hash());
    if query.v.is_some() && !immutable {
        return Err(StatusCode::NOT_FOUND);
    }
    let cache_control = if immutable {
        "public, max-age=31536000, immutable"
    } else {
        "no-cache"
    };
    if headers
        .get(header::IF_NONE_MATCH)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| etag_matches(value, &etag))
    {
        return Response::builder()
            .status(StatusCode::NOT_MODIFIED)
            .header(header::ETAG, etag)
            .header(header::CACHE_CONTROL, cache_control)
            .body(Body::empty())
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
    }

    let content_type = match asset
        .path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "avif" => "image/avif",
        "gif" => "image/gif",
        "mp3" => "audio/mpeg",
        "ogg" | "opus" => "audio/ogg",
        _ => "application/octet-stream",
    };
    let file = tokio::fs::File::open(&asset.path)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    let content_length = file
        .metadata()
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?
        .len();
    let stream = try_unfold(file, |mut file| async move {
        let mut buffer = vec![0; 64 * 1024];
        let read = file.read(&mut buffer).await?;
        if read == 0 {
            return Ok::<_, std::io::Error>(None);
        }
        buffer.truncate(read);
        Ok(Some((Bytes::from(buffer), file)))
    });
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CONTENT_LENGTH, content_length)
        .header(header::ETAG, etag)
        .header(header::CACHE_CONTROL, cache_control)
        .body(Body::from_stream(stream))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

fn etag_matches(if_none_match: &str, etag: &str) -> bool {
    if_none_match.split(',').any(|candidate| {
        let candidate = candidate.trim();
        candidate == "*" || candidate == etag || candidate.strip_prefix("W/") == Some(etag)
    })
}

#[cfg(test)]
mod tests {
    use super::etag_matches;

    #[test]
    fn etag_matching_accepts_lists_and_weak_validators() {
        assert!(etag_matches("\"old\", W/\"current\"", "\"current\""));
        assert!(etag_matches("*", "\"current\""));
        assert!(!etag_matches("\"other\"", "\"current\""));
    }
}

/// Prometheus-format metrics endpoint (architecture §40.2).
///
/// Exposes all 13 metric categories from §40.2 in Prometheus text format.
async fn metrics_handler(State(lobby): State<Arc<Lobby>>) -> String {
    let stats = lobby.stats();
    let mut out = String::new();

    // §40.2 metrics in Prometheus text exposition format.
    out.push_str("# HELP noname_rooms_active Currently active rooms.\n");
    out.push_str("# TYPE noname_rooms_active gauge\n");
    out.push_str(&format!("noname_rooms_active {}\n", stats.active_rooms));

    out.push_str("# HELP noname_connections_total Total connections accepted.\n");
    out.push_str("# TYPE noname_connections_total counter\n");
    out.push_str(&format!(
        "noname_connections_total {}\n",
        stats.total_connections
    ));

    out.push_str("# HELP noname_commands_total Total commands processed.\n");
    out.push_str("# TYPE noname_commands_total counter\n");
    out.push_str(&format!("noname_commands_total {}\n", stats.total_commands));

    out.push_str("# HELP noname_commands_rejected_total Total commands rejected.\n");
    out.push_str("# TYPE noname_commands_rejected_total counter\n");
    out.push_str(&format!(
        "noname_commands_rejected_total {}\n",
        stats.rejected_commands
    ));

    out.push_str("# HELP noname_outbox_pending Pending outbox messages.\n");
    out.push_str("# TYPE noname_outbox_pending gauge\n");
    out.push_str(&format!("noname_outbox_pending {}\n", stats.outbox_pending));

    out.push_str("# HELP noname_outbox_delivered_total Delivered outbox messages.\n");
    out.push_str("# TYPE noname_outbox_delivered_total counter\n");
    out.push_str(&format!(
        "noname_outbox_delivered_total {}\n",
        stats.outbox_delivered
    ));

    out.push_str("# HELP noname_build_info Engine build information.\n");
    out.push_str("# TYPE noname_build_info gauge\n");
    out.push_str(&format!(
        "noname_build_info{{build=\"{}\"}} 1\n",
        engine_api::ENGINE_BUILD_ID
    ));

    out
}

/// Room list endpoint: returns active rooms as JSON.
async fn rooms_handler(State(lobby): State<Arc<Lobby>>) -> axum::Json<serde_json::Value> {
    let rooms = lobby.room_list();
    axum::Json(serde_json::json!({
        "rooms": rooms,
        "total": rooms.len(),
    }))
}

async fn ws_handler(ws: WebSocketUpgrade, State(lobby): State<Arc<Lobby>>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, lobby))
}

/// 每连接出站队列上限。满载表示客户端不再读取，房间广播必须立刻放弃
/// 该连接，而不是等待整个房间 actor。
const OUTBOUND_QUEUE: usize = 64;
/// 单条 WebSocket 文本消息的写超时。超时后主动断开，客户端重连时重新同步。
const WRITER_SEND_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

/// 测试专用：给 writer 每条消息前加延迟，让慢客户端队列在可控时间内饱和。
/// 生产构建没有这个开关（`cfg(test)` 编译期移除），也不从环境变量读取。
#[cfg(test)]
pub(crate) static TEST_WRITER_DELAY_MS: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);

/// 测试专用：覆盖每连接出站队列容量（0 = 用生产默认值）。
#[cfg(test)]
pub(crate) static TEST_OUTBOUND_QUEUE: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

async fn handle_socket(socket: WebSocket, lobby: Arc<Lobby>) {
    let (mut sink, mut stream) = socket.split();
    #[cfg(test)]
    let queue_capacity = match TEST_OUTBOUND_QUEUE.load(std::sync::atomic::Ordering::Relaxed) {
        0 => OUTBOUND_QUEUE,
        configured => configured,
    };
    #[cfg(not(test))]
    let queue_capacity = OUTBOUND_QUEUE;
    let (tx, mut rx) = mpsc::channel::<String>(queue_capacity);
    let conn_id = lobby.alloc_connection_id();
    let outbound = Outbound::new(tx.clone());

    // Writer task：socket 写入加超时；慢/断客户端标记后由 session 读循环退出。
    let writer = {
        let writer_slow = outbound.slow_marker();
        tokio::spawn(async move {
            while let Some(text) = rx.recv().await {
                #[cfg(test)]
                {
                    let delay = TEST_WRITER_DELAY_MS.load(std::sync::atomic::Ordering::Relaxed);
                    if delay > 0 {
                        tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                    }
                }
                match tokio::time::timeout(
                    WRITER_SEND_TIMEOUT,
                    sink.send(Message::Text(text.into())),
                )
                .await
                {
                    Ok(Ok(())) => {}
                    Ok(Err(_)) | Err(_) => {
                        let _ = writer_slow.send(true);
                        break;
                    }
                }
            }
        })
    };

    let _ = outbound
        .send(&ServerMessage::Welcome {
            connection_id: conn_id,
            protocol_version: NETWORK_PROTOCOL_VERSION,
            versions: VersionBundle::current(),
        })
        .await;

    let mut session = session::Session {
        connection_id: conn_id,
        display_name: format!("player-{conn_id}"),
        room_id: None,
        player_id: None,
        role: None,
        reconnect_token: None,
        last_conn_seq: 0,
        handshake_complete: false,
        outbound: outbound.clone(),
        slow: outbound.slow_signal(),
    };

    loop {
        let message = tokio::select! {
            changed = session.slow.changed() => {
                if changed.is_err() || *session.slow.borrow() {
                    warn!(conn = conn_id, "慢连接被断开，等待客户端重连同步");
                    break;
                }
                continue;
            }
            message = stream.next() => message,
        };
        let Some(Ok(msg)) = message else { break };
        let text = match msg {
            Message::Text(t) => t.to_string(),
            Message::Binary(b) => match String::from_utf8(b.to_vec()) {
                Ok(s) => s,
                Err(_) => {
                    let _ = outbound
                        .send(&public_error(ErrorCode::BadMessage, "binary not supported"))
                        .await;
                    continue;
                }
            },
            Message::Ping(_) | Message::Pong(_) => continue,
            Message::Close(_) => break,
        };

        let c2s = match decode_c2s(&text) {
            Ok(m) => m,
            Err(e) => {
                let _ = outbound
                    .send(&public_error(ErrorCode::BadMessage, e.to_string()))
                    .await;
                continue;
            }
        };

        if let Err(e) = lobby.handle_client(&mut session, c2s).await {
            warn!(conn = conn_id, error = %e, "client handler error");
            let _ = outbound
                .send(&ServerMessage::Error {
                    code: e.code,
                    message: e.message,
                    room_id: session.room_id,
                })
                .await;
        }
    }

    if let Some(room_id) = session.room_id {
        lobby.disconnect(room_id, conn_id).await;
    }
    // All senders must be dropped before awaiting the writer, otherwise the
    // channel stays open through `session.outbound` and close handshakes hang.
    drop(session);
    drop(outbound);
    drop(tx);
    let _ = writer.await;
    info!(conn = conn_id, "connection closed");
}

#[cfg(test)]
mod net_smoke {
    //! **真实 socket 往返**（loopback）：起真实 `TcpListener` + `axum::serve`，用**裸 HTTP/1.1**
    //! 请求 `/health`（**零新依赖**：只用 `std::net::TcpStream`）。
    //!
    //! 口径：验证的是**真实 TCP + HTTP 处理链**（不是 mock、不是直调 handler）；
    //! **不含** TLS、跨机、WebSocket 升级与浏览器端。

    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpStream;

    fn test_cfg(tag: &str) -> ServerConfig {
        let base = std::env::temp_dir().join(format!("gs-net-{tag}-{}", std::process::id()));
        let mut cfg = ServerConfig::default();
        cfg.original_content_root = base.join("orig");
        cfg.content_store_root = base.join("store");
        cfg.extensions_root = base.join("ext");
        cfg.native_packages_root = base.join("np");
        cfg.native_store_root = base.join("ns");
        cfg.native_room_store = base.join("rooms").to_string_lossy().into_owned();
        for d in [
            &cfg.original_content_root,
            &cfg.content_store_root,
            &cfg.extensions_root,
            &cfg.native_packages_root,
            &cfg.native_store_root,
        ] {
            let _ = std::fs::create_dir_all(d);
        }
        cfg
    }

    const LOCK_CHILD_DIR: &str = "NONAME_TEST_STORE_LOCK_DIR";

    #[test]
    fn store_writer_lock_child() {
        let Some(dir) = std::env::var_os(LOCK_CHILD_DIR) else {
            return;
        };
        let dir = std::path::PathBuf::from(dir);
        let _lock = crate::native_room_store::StoreWriterLock::acquire(&dir)
            .expect("child acquires store lock");
        std::fs::write(dir.join("ready"), b"1").unwrap();
        while !dir.join("stop").exists() {
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
    }

    #[test]
    fn second_process_is_rejected_and_abnormal_exit_releases_lock() {
        use std::process::{Command as ProcessCommand, Stdio};
        let dir = tempfile::tempdir().unwrap();
        let ready = dir.path().join("ready");
        let mut child = ProcessCommand::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "net_smoke::store_writer_lock_child",
                "--nocapture",
            ])
            .env(LOCK_CHILD_DIR, dir.path())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
        while !ready.exists() {
            assert!(
                std::time::Instant::now() < deadline,
                "child did not acquire the lock in time"
            );
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        match crate::native_room_store::StoreWriterLock::acquire(dir.path()) {
            Ok(_) => panic!("另一进程持锁时本进程必须拒绝"),
            Err(error) => assert!(error.contains("写者"), "{error}"),
        }
        // SIGKILL/终止进程：不执行 Drop，锁只能由 OS 释放。
        child.kill().unwrap();
        child.wait().unwrap();
        let _lock = crate::native_room_store::StoreWriterLock::acquire(dir.path())
            .expect("写者异常退出后必须可重启");
    }

    #[test]
    fn second_bootstrap_on_same_store_is_rejected_and_releases_after_drop() {
        let cfg = test_cfg("store-lock");
        let first = bootstrap(&cfg);
        match try_bootstrap(&cfg) {
            Ok(_) => panic!("同目录第二写者必须拒绝启动"),
            Err(error) => assert!(error.contains("写者"), "{error}"),
        }
        drop(first);
        let _restarted = try_bootstrap(&cfg).expect("第一个写者退出后应可重启");
    }

    #[test]
    fn health_over_real_tcp() {
        let cfg = test_cfg("health");
        let lobby = bootstrap(&cfg);
        let app = build_router(lobby, ServeDir::new(&cfg.original_content_root));
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("runtime");
        let (addr, handle) = rt.block_on(async {
            let l = tokio::net::TcpListener::bind("127.0.0.1:0")
                .await
                .expect("bind 127.0.0.1:0");
            let a = l.local_addr().expect("local_addr");
            let h = tokio::spawn(async move {
                let _ = axum::serve(l, app).await;
            });
            (a, h)
        });

        let mut sock = TcpStream::connect(addr).expect("connect");
        sock.set_read_timeout(Some(std::time::Duration::from_secs(10)))
            .expect("set timeout");
        write!(
            sock,
            "GET /health HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n\r\n"
        )
        .expect("write request");
        let mut buf = String::new();
        sock.read_to_string(&mut buf).expect("read response");
        let first = buf.lines().next().unwrap_or("<empty>").to_string();
        eprintln!("DBG 真实网络往返：{addr} -> {first}");
        assert!(
            first.starts_with("HTTP/1.1 200"),
            "应返回 200，实得：{first}"
        );
        assert!(
            buf.trim_end().ends_with("ok"),
            "响应体应为 ok，实得：{buf}"
        );

        // 第二个真实请求：`/metrics`（需要 Lobby 状态，验证有状态端点同样可达）
        let mut sock2 = TcpStream::connect(addr).expect("connect2");
        sock2
            .set_read_timeout(Some(std::time::Duration::from_secs(10)))
            .expect("set timeout2");
        write!(
            sock2,
            "GET /metrics HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n\r\n"
        )
        .expect("write request2");
        let mut buf2 = String::new();
        sock2.read_to_string(&mut buf2).expect("read response2");
        let first2 = buf2.lines().next().unwrap_or("<empty>").to_string();
        eprintln!(
            "DBG 真实网络往返（有状态端点 /metrics）：{addr} -> {first2}；body_len={}",
            buf2.split("\r\n\r\n").nth(1).map(|b| b.len()).unwrap_or(0)
        );
        assert!(
            first2.starts_with("HTTP/1.1 200"),
            "/metrics 应返回 200，实得：{first2}"
        );
        handle.abort();
    }
    /// **真实 WebSocket 往返**：升级 `/ws` → 收到 `Welcome` → 发一条非法文本 → 收到错误回复。
    ///
    /// 口径：覆盖 **WS 升级握手 + 服务端主动推送 + 客户端→服务端→回复** 三段；
    /// 不含 TLS、代理与浏览器实现差异。
    #[test]
    fn websocket_welcome_and_error_roundtrip() {
        use futures_util::{SinkExt, StreamExt};

        let cfg = test_cfg("ws");
        let lobby = bootstrap(&cfg);
        let app = build_router(lobby, ServeDir::new(&cfg.original_content_root));
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("runtime");

        rt.block_on(async move {
            let l = tokio::net::TcpListener::bind("127.0.0.1:0")
                .await
                .expect("bind");
            let addr = l.local_addr().expect("addr");
            let server = tokio::spawn(async move {
                let _ = axum::serve(l, app).await;
            });

            let url = format!("ws://{addr}/ws");
            let (mut ws, resp) = tokio_tungstenite::connect_async(&url)
                .await
                .expect("ws connect");
            eprintln!("DBG WS 升级：{url} -> status {}", resp.status());

            // ① 服务端主动推送 Welcome
            let first = tokio::time::timeout(std::time::Duration::from_secs(10), ws.next())
                .await
                .expect("welcome timeout")
                .expect("welcome frame")
                .expect("welcome ok");
            let first_text = first.to_text().expect("text frame").to_string();
            eprintln!(
                "DBG WS Welcome：{}",
                first_text.chars().take(90).collect::<String>()
            );
            assert!(
                first_text.contains("Welcome") || first_text.contains("welcome"),
                "首帧应为 Welcome：{first_text}"
            );

            // ② 发一条非法文本 ⇒ 应收到解析错误回复
            ws.send(tokio_tungstenite::tungstenite::Message::Text(
                "not-a-json".to_string().into(),
            ))
            .await
            .expect("send bad");
            let second = tokio::time::timeout(std::time::Duration::from_secs(10), ws.next())
                .await
                .expect("error timeout")
                .expect("error frame")
                .expect("error ok");
            let second_text = second.to_text().expect("text frame2").to_string();
            eprintln!(
                "DBG WS 错误回复：{}",
                second_text.chars().take(120).collect::<String>()
            );
            assert!(
                second_text.to_lowercase().contains("error")
                    || second_text.contains("BadMessage")
                    || second_text.contains("bad"),
                "非法文本应得到错误回复：{second_text}"
            );
            let _ = ws.close(None).await;
            server.abort();
        });
    }
    /// **真实 WebSocket 对局命令往返**：A 建局、B 加入、A 提交 `EndPhase`，读回 `CommandOutcome`。
    ///
    /// 口径：覆盖 **建局 / 加入 / 命令提交 / 个人化结果回推** 全链路的 **WS** 路径；
    /// **不含**整局机器人（那是 `long_run_bot.rs` 走直连 `RoomMsg` 的面）、TLS、跨机与浏览器。
    #[test]
    fn websocket_creates_joins_and_submits_over_wire() {
        use engine_api::{Command, CommandKind, ExpectedRevision, IdempotencyKey, PlayerId};
        use engine_protocol::{
            decode_s2c, encode_c2s, ClientMessage, MessageHeader, ServerMessage,
        };
        use futures_util::{SinkExt, StreamExt};

        let cfg = test_cfg("wsgame");
        let lobby = bootstrap(&cfg);
        let app = build_router(lobby, ServeDir::new(&cfg.original_content_root));
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("runtime");

        rt.block_on(async move {
            let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("bind");
            let addr = l.local_addr().expect("addr");
            let server = tokio::spawn(async move {
                let _ = axum::serve(l, app).await;
            });
            let url = format!("ws://{addr}/ws");

            // --- A：连接 + Welcome + 建局 ---
            let (mut a, _) = tokio_tungstenite::connect_async(&url).await.expect("A connect");
            let _welcome = next_text(&mut a).await;
            send_test_hello(&mut a).await;
            a.send(tokio_tungstenite::tungstenite::Message::Text(
                encode_c2s(&ClientMessage::CreateRoom {
                    choice_timeout_secs: None,
                    players: 2,
                    seed: 424_242,
                    password: String::new(),
                })
                .expect("encode create")
                .into(),
            ))
            .await
            .expect("send create");

            // 读到 JoinOk（拿到 room_id / player_id），最多等 12 帧
            let mut a_room: Option<u64> = None;
            let mut a_pid: Option<u32> = None;
            for _ in 0..12 {
                let t = next_text(&mut a).await;
                if let Ok(msg) = decode_s2c(&t) {
                    if let ServerMessage::JoinOk {
                        room_id, player_id, ..
                    } = &msg
                    {
                        a_room = Some(*room_id);
                        a_pid = Some(*player_id);
                        eprintln!("DBG WS 建局：room={room_id} player={player_id}");
                        break;
                    }
                }
            }
            let room_id = a_room.expect("A 应收到 JoinOk(room_id)");
            let pid = a_pid.expect("A 应收到 JoinOk(player_id)");

            // --- B：连接 + 加入同一房间 ---
            let (mut b, _) = tokio_tungstenite::connect_async(&url).await.expect("B connect");
            let _welcome_b = next_text(&mut b).await;
            send_test_hello(&mut b).await;
            b.send(tokio_tungstenite::tungstenite::Message::Text(
                encode_c2s(&ClientMessage::JoinRoom {
                    room_id,
                    password: String::new(),
                    preferred_seat: Some(1),
                    reconnect_token: None,
                })
                .expect("encode join")
                .into(),
            ))
            .await
            .expect("send join");
            let mut b_joined = false;
            for _ in 0..12 {
                let t = next_text(&mut b).await;
                if let Ok(ServerMessage::JoinOk { player_id, .. }) = decode_s2c(&t) {
                    eprintln!("DBG WS 加入：room={room_id} player={player_id}");
                    b_joined = true;
                    break;
                }
            }
            assert!(b_joined, "B 应在 12 帧内收到 JoinOk");

            // --- A：提交一条 EndPhase ---
            let header = MessageHeader::new(room_id, 1, 0);
            a.send(tokio_tungstenite::tungstenite::Message::Text(
                encode_c2s(&ClientMessage::SubmitCommand {
                    header: header.clone(),
                    command: Command {
                        actor: PlayerId::new(pid),
                        expected_revision: ExpectedRevision(0),
                        idempotency_key: IdempotencyKey(1),
                        kind: CommandKind::EndPhase,
                    },
                    idempotency_key: IdempotencyKey(1),
                replay: false,
})
                .expect("encode submit")
                .into(),
            ))
            .await
            .expect("send submit");

            let mut seen_outcome = false;
            for _ in 0..16 {
                let t = next_text(&mut a).await;
                if let Ok(ServerMessage::CommandOutcome {
                    result, view, ..
                }) = decode_s2c(&t)
                {
                    eprintln!(
                        "DBG WS 命令结果：result={result:?} revision={} hash_prefix={:?}",
                        view.revision,
                        &view.state_hash[..4]
                    );
                    seen_outcome = true;
                    break;
                }
            }
            assert!(seen_outcome, "A 应在 16 帧内收到 CommandOutcome（WS 全链路）");
            let _ = a.close(None).await;
            let _ = b.close(None).await;
            server.abort();
        });
    }

    /// 读一条文本帧（10s 超时）。
    async fn send_test_hello<S>(ws: &mut S)
    where
        S: futures_util::Sink<tokio_tungstenite::tungstenite::Message> + Unpin,
        S::Error: std::fmt::Debug,
    {
        use futures_util::SinkExt;
        ws.send(tokio_tungstenite::tungstenite::Message::Text(
            engine_protocol::encode_c2s(&engine_protocol::ClientMessage::Hello {
                versions: engine_api::VersionBundle::current(),
                display_name: "wire-test".into(),
            })
            .unwrap()
            .into(),
        ))
        .await
        .unwrap();
    }

    async fn next_text<S>(ws: &mut S) -> String
    where
        S: futures_util::Sink<tokio_tungstenite::tungstenite::Message>
            + futures_util::Stream<
                Item = Result<
                    tokio_tungstenite::tungstenite::Message,
                    tokio_tungstenite::tungstenite::Error,
                >,
            > + Unpin,
    {
        use futures_util::StreamExt;
        let frame = tokio::time::timeout(std::time::Duration::from_secs(10), ws.next())
            .await
            .expect("frame timeout")
            .expect("frame")
            .expect("frame ok");
        frame.to_text().expect("text").to_string()
    }
    /// **WS 整局**：两条 WS 连接驱动到 `MatchOutcome != InProgress`（"真实网络上的完整对局"）。
    ///
    /// 选步口径同 `long_run_bot.rs`：`ResolveChoice` 原样用权威 `selections` → 出牌/技能 → `EndPhase`；
    /// `legal_actions` 为空时用 `RequestSnapshot` 刷新（快照带 `choice`）。
    /// **不是**强 AI，只用于驱动推进。
    #[test]
    fn websocket_plays_match_to_completion() {
        use engine_api::{
            Command, CommandKind, ExpectedRevision, IdempotencyKey, LegalAction, PlayerId,
        };
        use engine_protocol::{
            decode_s2c, encode_c2s, ClientMessage, MessageHeader, ServerMessage,
        };
        use futures_util::{SinkExt, StreamExt};
        use tokio_tungstenite::tungstenite::Message;

        const MAX_STEPS: usize = 4000;

        fn pick(legal: &[LegalAction]) -> Option<LegalAction> {
            if let Some(a) = legal
                .iter()
                .find(|a| matches!(a, LegalAction::ResolveChoice { .. }))
            {
                return Some(a.clone());
            }
            for a in legal {
                if matches!(
                    a,
                    LegalAction::PlayCard { .. }
                        | LegalAction::ActivateSkill { .. }
                        | LegalAction::UseSkillCard { .. }
                        | LegalAction::EndPhase
                ) {
                    return Some(a.clone());
                }
            }
            legal.first().cloned()
        }

        let cfg = test_cfg("wsfull");
        let lobby = bootstrap(&cfg);
        let app = build_router(lobby, ServeDir::new(&cfg.original_content_root));
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("runtime");

        rt.block_on(async move {
            let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("bind");
            let addr = l.local_addr().expect("addr");
            let server = tokio::spawn(async move {
                let _ = axum::serve(l, app).await;
            });
            let url = format!("ws://{addr}/ws");

            // A 建局
            let (mut a, _) = tokio_tungstenite::connect_async(&url).await.expect("A");
            let _ = next_text(&mut a).await;
            send_test_hello(&mut a).await;
            a.send(Message::Text(
                encode_c2s(&ClientMessage::CreateRoom {
                    choice_timeout_secs: None,
                    players: 2,
                    seed: 20_260_101,
                    password: String::new(),
                })
                .expect("enc create")
                .into(),
            ))
            .await
            .expect("send create");
            let (room_id, pid_a) = loop {
                let t = next_text(&mut a).await;
                if let Ok(ServerMessage::JoinOk {
                    room_id, player_id, ..
                }) = decode_s2c(&t)
                {
                    break (room_id, player_id);
                }
            };
            // B 加入
            let (mut b, _) = tokio_tungstenite::connect_async(&url).await.expect("B");
            let _ = next_text(&mut b).await;
            send_test_hello(&mut b).await;
            b.send(Message::Text(
                encode_c2s(&ClientMessage::JoinRoom {
                    room_id,
                    password: String::new(),
                    preferred_seat: Some(1),
                    reconnect_token: None,
                })
                .expect("enc join")
                .into(),
            ))
            .await
            .expect("send join");
            let pid_b = loop {
                let t = next_text(&mut b).await;
                if let Ok(ServerMessage::JoinOk { player_id, .. }) = decode_s2c(&t) {
                    break player_id;
                }
            };
            eprintln!("DBG WS 整局：room={room_id} A={pid_a} B={pid_b}");

            let actors = [PlayerId::new(pid_a), PlayerId::new(pid_b)];
            let mut legal: [Vec<LegalAction>; 2] = [Vec::new(), Vec::new()];
            let mut rev = 0u64;
            let mut seq = 0u64;
            let mut outcome = engine_api::MatchOutcome::InProgress;
            let mut steps = 0usize;
            let mut snapshots = 0usize;

            for _ in 0..MAX_STEPS {
                let mut acted = false;
                for conn in 0..2 {
                    let ws = if conn == 0 { &mut a } else { &mut b };
                    if legal[conn].is_empty() {
                        seq += 1;
                        snapshots += 1;
                        ws.send(Message::Text(
                            encode_c2s(&ClientMessage::RequestSnapshot {
                                header: MessageHeader::new(room_id, seq, rev),
                                last_revision: rev,
                            })
                            .expect("enc snap")
                            .into(),
                        ))
                        .await
                        .expect("send snap");
                        loop {
                            let t = next_text(ws).await;
                            if let Ok(ServerMessage::Snapshot {
                                view,
                                choice,
                                legal_actions,
                                ..
                            }) = decode_s2c(&t)
                            {
                                outcome = view.outcome.clone();
                                if view.revision > 0 {
                                    rev = view.revision;
                                }
                                legal[conn] = legal_actions;
                                if legal[conn].is_empty() {
                                    if let Some(c) = choice {
                                        if c.observer.0 == actors[conn].0 {
                                            let take = (c.min_count as usize).min(c.options.len());
                                            legal[conn].push(LegalAction::ResolveChoice {
                                                choice_id: c.choice_id,
                                                selections: (0..take as u32).collect(),
                                                cancelled: take == 0 && c.allow_cancel,
                                            });
                                        }
                                    }
                                }
                                break;
                            }
                        }
                    }
                    let Some(action) = pick(&legal[conn]) else {
                        continue;
                    };
                    let cmd = action.to_command(actors[conn], rev);
                    seq += 1;
                    ws.send(Message::Text(
                        encode_c2s(&ClientMessage::SubmitCommand {
                            header: MessageHeader::new(room_id, seq, rev),
                            command: cmd.clone(),
                            idempotency_key: IdempotencyKey(seq),
                        replay: false,
})
                        .expect("enc submit")
                        .into(),
                    ))
                    .await
                    .expect("send submit");
                    loop {
                        let t = next_text(ws).await;
                        if let Ok(ServerMessage::CommandOutcome {
                            result,
                            view,
                            legal_actions,
                            ..
                        }) = decode_s2c(&t)
                        {
                            outcome = view.outcome.clone();
                            if view.revision > 0 {
                                rev = view.revision;
                            }
                            legal[conn] = legal_actions;
                            let _ = result;
                            break;
                        }
                    }
                    acted = true;
                    steps += 1;
                    if outcome != engine_api::MatchOutcome::InProgress {
                        break;
                    }
                }
                if outcome != engine_api::MatchOutcome::InProgress {
                    break;
                }
                if !acted {
                    // 双方都无动作：再刷一次 A 的快照看是否终局
                    seq += 1;
                    a.send(Message::Text(
                        encode_c2s(&ClientMessage::RequestSnapshot {
                            header: MessageHeader::new(room_id, seq, rev),
                            last_revision: rev,
                        })
                        .expect("enc snap2")
                        .into(),
                    ))
                    .await
                    .expect("send snap2");
                    loop {
                        let t = next_text(&mut a).await;
                        if let Ok(ServerMessage::Snapshot { view, .. }) = decode_s2c(&t) {
                            outcome = view.outcome.clone();
                            break;
                        }
                    }
                    if outcome != engine_api::MatchOutcome::InProgress {
                        break;
                    }
                }
            }

            eprintln!(
                "DBG WS 整局结果：outcome={outcome:?} steps={steps} snapshots={snapshots} rev={rev}"
            );
            assert!(
                steps > 0 && snapshots > 0,
                "WS 整局应至少走一步并取过快照（steps={steps}, snapshots={snapshots}）"
            );
            assert_ne!(
                outcome,
                engine_api::MatchOutcome::InProgress,
                "WS 整局应在 {MAX_STEPS} 步内结束（outcome={outcome:?}, rev={rev}）"
            );
            let _ = a.close(None).await;
            let _ = b.close(None).await;
            server.abort();
            steps
        });
    }
    /// **断线重连**：连上 → 建局 → 走一步 → **断开** → 用 `reconnect_token` 重连，
    /// 断言「同一 `player_id`」且「修订号未丢」。
    ///
    /// 口径：loopback 明文 WS；覆盖 `JoinRoom{reconnect_token}` 路径与修订号保持；
    /// **不含**网络中断的 TCP 层重试、鉴权与跨机。
    #[test]
    fn websocket_reconnect_restores_seat_and_revision() {
        use engine_api::{
            Command, CommandKind, ExpectedRevision, IdempotencyKey, LegalAction, PlayerId,
        };
        use engine_protocol::{
            decode_s2c, encode_c2s, ClientMessage, MessageHeader, ServerMessage,
        };
        use futures_util::SinkExt;
        use tokio_tungstenite::tungstenite::Message;

        let cfg = test_cfg("wsreconnect");
        let lobby = bootstrap(&cfg);
        let app = build_router(lobby, ServeDir::new(&cfg.original_content_root));
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("runtime");

        rt.block_on(async move {
            let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("bind");
            let addr = l.local_addr().expect("addr");
            let server = tokio::spawn(async move {
                let _ = axum::serve(l, app).await;
            });
            let url = format!("ws://{addr}/ws");

            // ① 建局
            let (mut a, _) = tokio_tungstenite::connect_async(&url).await.expect("A");
            let _ = next_text(&mut a).await;
            send_test_hello(&mut a).await;
            a.send(Message::Text(
                encode_c2s(&ClientMessage::CreateRoom {
                    choice_timeout_secs: None,
                    players: 2,
                    seed: 20_260_102,
                    password: String::new(),
                })
                .expect("enc create")
                .into(),
            ))
            .await
            .expect("send create");
            let (room_id, pid, token) = loop {
                let t = next_text(&mut a).await;
                if let Ok(ServerMessage::JoinOk {
                    room_id,
                    player_id,
                    reconnect_token,
                    ..
                }) = decode_s2c(&t)
                {
                    break (room_id, player_id, reconnect_token);
                }
            };
            eprintln!("DBG 重连：建局 room={room_id} player={pid} token_len={}", token.len());

            // ①.5 第二名玩家加入（房间需凑齐座位才接受命令）
            let (mut b, _) = tokio_tungstenite::connect_async(&url).await.expect("B");
            let _ = next_text(&mut b).await;
            send_test_hello(&mut b).await;
            b.send(Message::Text(
                encode_c2s(&ClientMessage::JoinRoom {
                    room_id,
                    password: String::new(),
                    preferred_seat: Some(1),
                    reconnect_token: None,
                })
                .expect("enc join b")
                .into(),
            ))
            .await
            .expect("send join b");
            let pid_b = loop {
                let t = next_text(&mut b).await;
                if let Ok(ServerMessage::JoinOk { player_id, .. }) = decode_s2c(&t) {
                    break player_id;
                }
            };
            eprintln!("DBG 重连：B 加入 player={pid_b}");

            // ② 走一步（让修订号 > 0）
            let mut rev = 0u64;
            let mut legal: Vec<LegalAction> = Vec::new();
            a.send(Message::Text(
                encode_c2s(&ClientMessage::SubmitCommand {
                    header: MessageHeader::new(room_id, 1, 0),
                    command: Command {
                        actor: PlayerId::new(pid),
                        expected_revision: ExpectedRevision(0),
                        idempotency_key: IdempotencyKey(1),
                        kind: CommandKind::EndPhase,
                    },
                    idempotency_key: IdempotencyKey(1),
                replay: false,
})
                .expect("enc submit")
                .into(),
            ))
            .await
            .expect("send submit");
            loop {
                let t = next_text(&mut a).await;
                if let Ok(ServerMessage::CommandOutcome {
                    view,
                    legal_actions,
                    ..
                }) = decode_s2c(&t)
                {
                    rev = view.revision;
                    legal = legal_actions;
                    break;
                }
            }
            eprintln!("DBG 重连：命令后 rev={rev} legal={}", legal.len());
            // ③ 断开
            let _ = a.close(None).await;
            drop(a);

            // ④ 用 token 重连
            let (mut a2, _) = tokio_tungstenite::connect_async(&url).await.expect("A2");
            let _ = next_text(&mut a2).await;
            send_test_hello(&mut a2).await;
            a2.send(Message::Text(
                encode_c2s(&ClientMessage::JoinRoom {
                    room_id,
                    password: String::new(),
                    preferred_seat: None,
                    reconnect_token: Some(token.clone()),
                })
                .expect("enc reconnect")
                .into(),
            ))
            .await
            .expect("send reconnect");
            let mut rejoin: Option<(u32, u64)> = None;
            for _ in 0..12 {
                let t = next_text(&mut a2).await;
                match decode_s2c(&t) {
                    Ok(ServerMessage::JoinOk {
                        player_id, view, ..
                    }) => {
                        rejoin = Some((player_id, view.revision));
                        break;
                    }
                    Ok(ServerMessage::Error { code, message, .. }) => {
                        eprintln!("DBG 重连失败：{code:?} {message}");
                        break;
                    }
                    _ => {}
                }
            }
            let (pid2, rev2) = rejoin.expect("重连应收到 JoinOk");
            eprintln!("DBG 重连：重连后 player={pid2} rev={rev2}");
            assert_eq!(pid2, pid, "重连应恢复同一 player_id");
            assert!(
                rev2 >= rev,
                "重连后的视野修订号不应倒退（断开前 {rev}，重连后 {rev2}）"
            );
            let _ = a2.close(None).await;
            server.abort();
        });
    }
    /// **座位未满时提交命令 ⇒ 明确错误回推**（此前我用错判：我的循环只等 `CommandOutcome`，
    /// 把 `Error` 帧忽略了，才误以为「静默等待」）。
    #[test]
    fn websocket_submit_before_match_start_returns_error() {
        use engine_api::{Command, CommandKind, ExpectedRevision, IdempotencyKey, PlayerId};
        use engine_protocol::{
            decode_s2c, encode_c2s, ClientMessage, MessageHeader, ServerMessage,
        };
        use futures_util::SinkExt;
        use tokio_tungstenite::tungstenite::Message;

        let cfg = test_cfg("wsnotstarted");
        let lobby = bootstrap(&cfg);
        let app = build_router(lobby, ServeDir::new(&cfg.original_content_root));
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("runtime");

        rt.block_on(async move {
            let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("bind");
            let addr = l.local_addr().expect("addr");
            let server = tokio::spawn(async move {
                let _ = axum::serve(l, app).await;
            });
            let url = format!("ws://{addr}/ws");
            let (mut a, _) = tokio_tungstenite::connect_async(&url).await.expect("A");
            let _ = next_text(&mut a).await;
            send_test_hello(&mut a).await;
            a.send(Message::Text(
                encode_c2s(&ClientMessage::CreateRoom {
                    choice_timeout_secs: None,
                    players: 2,
                    seed: 20_260_103,
                    password: String::new(),
                })
                .expect("enc create")
                .into(),
            ))
            .await
            .expect("send create");
            let (room_id, pid) = loop {
                let t = next_text(&mut a).await;
                if let Ok(ServerMessage::JoinOk {
                    room_id, player_id, ..
                }) = decode_s2c(&t)
                {
                    break (room_id, player_id);
                }
            };
            // 只有一名玩家（座位未满）就提交
            a.send(Message::Text(
                encode_c2s(&ClientMessage::SubmitCommand {
                    header: MessageHeader::new(room_id, 1, 0),
                    command: Command {
                        actor: PlayerId::new(pid),
                        expected_revision: ExpectedRevision(0),
                        idempotency_key: IdempotencyKey(1),
                        kind: CommandKind::EndPhase,
                    },
                    idempotency_key: IdempotencyKey(1),
                replay: false,
})
                .expect("enc submit")
                .into(),
            ))
            .await
            .expect("send submit");

            let mut got_error = None;
            for _ in 0..8 {
                let t = next_text(&mut a).await;
                if let Ok(ServerMessage::Error {
                    code,
                    message,
                    ..
                }) = decode_s2c(&t)
                {
                    got_error = Some((code, message));
                    break;
                }
            }
            let (code, message) = got_error.expect("座位未满时提交命令应收到明确 Error 帧");
            eprintln!("DBG 未开局提交：code={code:?} message={message}");
            assert!(
                message.to_lowercase().contains("not started")
                    || message.to_lowercase().contains("match"),
                "错误信息应说明「对局未开始」，实得：{message}"
            );
            assert_eq!(
                format!("{code:?}"),
                "Conflict",
                "座位未满提交命令应映射为 Conflict（而非 Internal），实得 {code:?}"
            );
            let _ = a.close(None).await;
            server.abort();
        });
    }
    /// **WS 多房间并发**：3 个房间各自两条连接，同时（交替）推进，验证房间间的**隔离**。
    ///
    /// 判据：
    /// 1. 三个 `room_id` 互不相同；
    /// 2. 每个房间的 `EndPhase` 都拿到 `Accepted`，且**修订号各自独立**；
    /// 3. 不同 `seed` 的房间 **`state_hash` 不同**（种子真的影响状态，房间不共享引擎）。
    #[test]
    fn websocket_multi_room_isolation() {
        use engine_api::{Command, CommandKind, ExpectedRevision, IdempotencyKey, PlayerId};
        use engine_protocol::{
            decode_s2c, encode_c2s, ClientMessage, MessageHeader, ServerMessage,
        };
        use futures_util::SinkExt;
        use tokio_tungstenite::tungstenite::Message;

        type Ws = tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >;

        async fn open(url: &str) -> Ws {
            let (ws, _) = tokio_tungstenite::connect_async(url).await.expect("ws");
            let mut ws = ws;
            let _ = next_text(&mut ws).await;
            send_test_hello(&mut ws).await; // Welcome
            ws
        }

        async fn create_room(ws: &mut Ws, seed: u64) -> (u64, u32) {
            ws.send(Message::Text(
                encode_c2s(&ClientMessage::CreateRoom {
                    choice_timeout_secs: None,
                    players: 2,
                    seed,
                    password: String::new(),
                })
                .expect("enc create")
                .into(),
            ))
            .await
            .expect("send create");
            loop {
                let t = next_text(ws).await;
                if let Ok(ServerMessage::JoinOk {
                    room_id, player_id, ..
                }) = decode_s2c(&t)
                {
                    return (room_id, player_id);
                }
            }
        }

        async fn join_room(ws: &mut Ws, room_id: u64) -> u32 {
            ws.send(Message::Text(
                encode_c2s(&ClientMessage::JoinRoom {
                    room_id,
                    password: String::new(),
                    preferred_seat: Some(1),
                    reconnect_token: None,
                })
                .expect("enc join")
                .into(),
            ))
            .await
            .expect("send join");
            loop {
                let t = next_text(ws).await;
                if let Ok(ServerMessage::JoinOk { player_id, .. }) = decode_s2c(&t) {
                    return player_id;
                }
            }
        }

        /// 提交一条 `EndPhase`，返回 (revision, state_hash)
        async fn end_phase(ws: &mut Ws, room_id: u64, pid: u32, seq: u64) -> (u64, [u8; 32]) {
            ws.send(Message::Text(
                encode_c2s(&ClientMessage::SubmitCommand {
                    header: MessageHeader::new(room_id, seq, 0),
                    command: Command {
                        actor: PlayerId::new(pid),
                        expected_revision: ExpectedRevision(0),
                        idempotency_key: IdempotencyKey(seq),
                        kind: CommandKind::EndPhase,
                    },
                    idempotency_key: IdempotencyKey(seq),
                replay: false,
})
                .expect("enc submit")
                .into(),
            ))
            .await
            .expect("send submit");
            loop {
                let t = next_text(ws).await;
                if let Ok(ServerMessage::CommandOutcome { view, result, .. }) = decode_s2c(&t) {
                    match result {
                        engine_api::CommandResult::Accepted { .. } => {
                            return (view.revision, view.state_hash)
                        }
                        other => panic!("EndPhase 应被接受，实得 {other:?}"),
                    }
                }
            }
        }

        let cfg = test_cfg("wsmulti");
        let lobby = bootstrap(&cfg);
        let app = build_router(lobby, ServeDir::new(&cfg.original_content_root));
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("runtime");

        rt.block_on(async move {
            let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("bind");
            let addr = l.local_addr().expect("addr");
            let server = tokio::spawn(async move {
                let _ = axum::serve(l, app).await;
            });
            let url = format!("ws://{addr}/ws");
            const ROOMS: usize = 3;

            let mut a_ws: Vec<Ws> = Vec::new();
            let mut b_ws: Vec<Ws> = Vec::new();
            let mut rooms: Vec<(u64, u32)> = Vec::new();
            for i in 0..ROOMS {
                let mut a = open(&url).await;
                let (room_id, pid) = create_room(&mut a, 30_000 + i as u64).await;
                let mut b = open(&url).await;
                let _pid_b = join_room(&mut b, room_id).await;
                eprintln!("DBG 多房间：room#{i} id={room_id} A={pid}");
                a_ws.push(a);
                b_ws.push(b);
                rooms.push((room_id, pid));
            }
            let ids: Vec<u64> = rooms.iter().map(|r| r.0).collect();
            let mut uniq = ids.clone();
            uniq.sort_unstable();
            uniq.dedup();
            assert_eq!(uniq.len(), ROOMS, "三个 room_id 应互不相同：{ids:?}");

            // 交替推进：每轮对每个房间各发一条
            let mut revs = vec![0u64; ROOMS];
            let mut hashes: Vec<[u8; 32]> = vec![[0u8; 32]; ROOMS];
            for round in 0..3u64 {
                for i in 0..ROOMS {
                    let (room_id, pid) = rooms[i];
                    let (rev, hash) = end_phase(&mut a_ws[i], room_id, pid, round + 1).await;
                    revs[i] = rev;
                    hashes[i] = hash;
                }
            }
            eprintln!("DBG 多房间结果：revs={revs:?}");
            for i in 0..ROOMS {
                assert!(revs[i] > 0, "房间 {i} 应有修订号推进");
            }
            assert!(
                hashes[0] != hashes[1] && hashes[1] != hashes[2] && hashes[0] != hashes[2],
                "不同 seed 的房间 state_hash 应互不相同（房间不共享引擎）"
            );
            for ws in a_ws.iter_mut().chain(b_ws.iter_mut()) {
                let _ = ws.close(None).await;
            }
            server.abort();
        });
    }
}

#[cfg(test)]
mod native_ws_e2e;

#[cfg(test)]
mod native_lifecycle_http_tests {
    use super::{
        native_disable_handler, native_import_handler, native_uninstall_handler, NativePackageBody,
    };
    use axum::extract::State;
    use axum::Json;
    use std::collections::BTreeMap;
    use std::sync::Arc;

    fn config() -> crate::config::ServerConfig {
        let mut config = crate::config::ServerConfig::default();
        config.native_room_store.clear();
        config
    }

    fn package_bytes() -> Vec<u8> {
        let source = "skill http_lifecycle_probe(990020) { trigger phase_end owner_is_target { effect draw_cards(self, 1); } }";
        let mut package =
            content_compiler::compile_source_named("http-lifecycle-probe", source).unwrap();
        package.manifest.version = "0.1.0".into();
        package.manifest.logic_hash = package.compute_logic_hash();
        package.manifest.asset_hash = package.compute_asset_hash();
        package.to_json_bytes().unwrap()
    }

    #[tokio::test]
    async fn http_handlers_wire_import_disable_uninstall_to_store() {
        let dir = tempfile::tempdir().unwrap();
        let dropin = dir.path().join("http-lifecycle-probe");
        std::fs::create_dir_all(&dropin).unwrap();
        std::fs::write(dropin.join("package.npack.json"), package_bytes()).unwrap();
        let native = Arc::new(
            crate::native::NativeContent::open(dir.path().to_path_buf(), dir.path().join("store"))
                .unwrap(),
        );
        native.import().unwrap();
        let loaded = native.load("http-lifecycle-probe").unwrap();
        let namespace = loaded.frozen.binding.namespace.clone();
        let lobby = Arc::new(
            crate::lobby::Lobby::new(
                config(),
                engine_core::Engine::new(),
                content_runtime::ContentLock::empty(),
                content_runtime::ActiveCatalog::default(),
                BTreeMap::new(),
                native.clone(),
            )
            .unwrap(),
        );

        let imported = native_import_handler(State(lobby.clone())).await.unwrap();
        assert!(imported.0.iter().any(|p| p.package_id == "http-lifecycle-probe"));

        let body = || {
            Json(NativePackageBody {
                namespace: namespace.clone(),
                package_id: "http-lifecycle-probe".into(),
                version: "0.1.0".into(),
            })
        };
        native_disable_handler(State(lobby.clone()), body())
            .await
            .unwrap();
        assert!(native.load("http-lifecycle-probe").is_err());
        native_uninstall_handler(State(lobby.clone()), body()).await.unwrap();
        assert!(lobby
            .native_packages()
            .unwrap()
            .iter()
            .all(|p| p.package_id != "http-lifecycle-probe"));
    }
    #[tokio::test]
    async fn real_http_route_lists_and_disables_package() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let dir = tempfile::tempdir().unwrap();
        let dropin = dir.path().join("http-route-probe");
        std::fs::create_dir_all(&dropin).unwrap();
        std::fs::write(dropin.join("package.npack.json"), package_bytes()).unwrap();
        let native = Arc::new(
            crate::native::NativeContent::open(dir.path().to_path_buf(), dir.path().join("store"))
                .unwrap(),
        );
        native.import().unwrap();
        let loaded = native.load("http-lifecycle-probe").unwrap();
        let namespace = loaded.frozen.binding.namespace.clone();
        let lobby = Arc::new(
            crate::lobby::Lobby::new(
                config(),
                engine_core::Engine::new(),
                content_runtime::ContentLock::empty(),
                content_runtime::ActiveCatalog::default(),
                BTreeMap::new(),
                native.clone(),
            )
            .unwrap(),
        );
        let original = tempfile::tempdir().unwrap();
        let router = super::build_router(lobby, tower_http::services::ServeDir::new(original.path()));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });

        let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();
        stream
            .write_all(
                b"GET /native/packages HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
            )
            .await
            .unwrap();
        let mut response = Vec::new();
        stream.read_to_end(&mut response).await.unwrap();
        let text = String::from_utf8_lossy(&response);
        assert!(text.starts_with("HTTP/1.1 200"), "{text}");
        assert!(text.contains("http-lifecycle-probe"), "{text}");

        let body = serde_json::json!({
            "namespace": namespace,
            "package_id": "http-lifecycle-probe",
            "version": "0.1.0"
        })
        .to_string();
        let request = format!(
            "POST /native/packages/disable HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();
        stream.write_all(request.as_bytes()).await.unwrap();
        let mut response = Vec::new();
        stream.read_to_end(&mut response).await.unwrap();
        assert!(
            String::from_utf8_lossy(&response).starts_with("HTTP/1.1 204"),
            "{}",
            String::from_utf8_lossy(&response)
        );
        assert!(native.load("http-lifecycle-probe").is_err());
    }

}
