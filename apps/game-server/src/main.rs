//! Authoritative game server entrypoint.

mod config;
mod lobby;
mod native;
mod native_view;
mod room_native;
mod room_native_actor;
mod outbox;
mod persist;
mod room;
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
    let store = PackageStore::open(&cfg.content_store_root).expect("open content store");
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
    .expect("import extensions");
    let engine_template =
        Engine::from_content_lock(&store, &report.content_lock).expect("load active content lock");
    let (content_lock, content_catalog) = store
        .load_active_catalog()
        .expect("load active content catalog");
    let content_assets = store
        .load_active_asset_index()
        .expect("load active content assets");
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
    .expect("open native content");
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
    ));
    let original_content = ServeDir::new(&cfg.original_content_root);

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/metrics", get(metrics_handler))
        .route("/rooms", get(rooms_handler))
        .route("/content-catalog", get(content_catalog_handler))
        .route(
            "/package-assets/{namespace}/{package_id}/{*asset_id}",
            get(package_asset_handler),
        )
        .route("/native/packages", get(native_packages_handler))
        .route("/native/games", post(native_games_handler))
        .route("/native/matches", post(native_matches_handler))
        .route("/native/matches/restore", post(native_restore_handler))
        .route("/ws", get(ws_handler))
        .nest_service("/original-content", original_content)
        .layer(CorsLayer::permissive())
        .with_state(lobby.clone());

    let addr = cfg.listen_addr;
    info!(%addr, "game-server listening");
    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    axum::serve(listener, app).await.expect("serve");
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

async fn handle_socket(socket: WebSocket, lobby: Arc<Lobby>) {
    let (mut sink, mut stream) = socket.split();
    let (tx, mut rx) = mpsc::channel::<String>(64);
    let conn_id = lobby.alloc_connection_id();

    // Writer task
    let writer = tokio::spawn(async move {
        while let Some(text) = rx.recv().await {
            if sink.send(Message::Text(text.into())).await.is_err() {
                break;
            }
        }
    });

    let outbound = Outbound::new(tx.clone());
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
        outbound: outbound.clone(),
    };

    while let Some(Ok(msg)) = stream.next().await {
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
