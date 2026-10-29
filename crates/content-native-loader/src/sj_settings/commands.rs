//! SJ 设置命令脚本（apply_sj_commands）的会话执行。

use super::*;

pub(super) fn join_names(names: &[String]) -> String {
    if names.is_empty() {
        "-".to_string()
    } else {
        names.join(",")
    }
}

pub(super) fn storage_token(storage: &BTreeMap<String, String>) -> String {
    if storage.is_empty() {
        "-".to_string()
    } else {
        storage
            .iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect::<Vec<_>>()
            .join(",")
    }
}

pub(super) struct Session {
    storage: BTreeMap<String, String>,
    extensions: Vec<String>,
    enables: BTreeMap<String, bool>,
    asset_version: Option<String>,
    config_map: BTreeMap<String, Value>,
    data: BTreeMap<String, Value>,
    console_open: bool,
    environment: bool,
}

pub(super) fn dash_option(text: &str) -> Option<&str> {
    if text == "-" {
        None
    } else {
        Some(text)
    }
}

pub(super) fn plain_error() -> WindowError {
    WindowError {
        message: Some("boom".into()),
        source: None,
        line: Some("3".into()),
        column: Some("4".into()),
        stack: None,
        event: None,
        translate: BTreeMap::new(),
    }
}

/// 供目标平台字符串辅助调用。一条命令串共享一份新状态。
pub fn apply_sj_commands(config_text: &str, commands: &str) -> Result<String, SjError> {
    let config = parse_sj_config(config_text)?;
    let mut session = Session {
        storage: BTreeMap::new(),
        extensions: Vec::new(),
        enables: BTreeMap::new(),
        asset_version: None,
        config_map: BTreeMap::new(),
        data: BTreeMap::new(),
        console_open: false,
        environment: false,
    };
    let mut parts_out = Vec::new();
    for command in commands.split(';').filter(|part| !part.is_empty()) {
        let mut parts = command.split(':');
        let head = parts.next().unwrap_or("");
        match head {
            "seed-ext" => push_missing(&mut session.extensions, parts.next().unwrap_or("")),
            "seed-asset" => session.asset_version = Some(parts.next().unwrap_or("").to_string()),
            "seed-en" => {
                let name = parts.next().unwrap_or("");
                let on = parts.next().unwrap_or("0") == "1";
                session.enables.insert(name.to_string(), on);
            }
            "seed-store" => {
                let key = parts.next().unwrap_or("");
                let value = parts.next().unwrap_or("").to_string();
                session.storage.insert(key.to_string(), value);
            }
            "boot" => {
                if let Some(raw) = command.split_once(':').map(|(_, rest)| rest) {
                    session
                        .storage
                        .insert(ANDROID_KEY.to_string(), raw.to_string());
                }
                let output = boot(&BootInput {
                    storage: session.storage.clone(),
                    extensions: session.extensions.clone(),
                    enables: session.enables.clone(),
                    asset_version: session.asset_version.clone(),
                })?;
                let names: Vec<String> = if output.android_key_removed {
                    let raw = command
                        .split_once(':')
                        .map(|(_, rest)| rest)
                        .unwrap_or("[]");
                    serde_json::from_str::<Vec<String>>(raw).unwrap_or_default()
                } else {
                    vec![SJ.to_string()]
                };
                let flags = names
                    .iter()
                    .map(|name| {
                        format!(
                            "{name}={}",
                            u8::from(output.enables.get(name).copied().unwrap_or(false))
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(",");
                let en = if flags.is_empty() {
                    String::new()
                } else {
                    format!(" en:{flags}")
                };
                let key = if output.android_key_removed {
                    "removed"
                } else {
                    "absent"
                };
                parts_out.push(format!(
                    "boot:{}{en} key:{key}",
                    join_names(&output.extensions)
                ));
                session.storage = output.storage;
                session.extensions = output.extensions;
                session.enables = output.enables;
                session.asset_version = Some(output.asset_version);
            }
            "asset" => {
                let token = session
                    .asset_version
                    .clone()
                    .unwrap_or_else(|| "missing".into());
                let shown = if token.is_empty() {
                    "missing".to_string()
                } else {
                    token
                };
                parts_out.push(format!("asset:{shown}"));
            }
            "version" => {
                let local = parts.next().unwrap_or("");
                let remote = parts.next().unwrap_or("");
                parts_out.push(format!(
                    "version:{}",
                    compare_version(local, remote).token()
                ));
            }
            "mark" => {
                let engine = parts.next().unwrap_or("");
                let local = parts.next().unwrap_or("-");
                let remote = parts.next().unwrap_or("-");
                let noname = parts.next().unwrap_or("-");
                let mark = catalog_mark(
                    engine,
                    dash_option(local),
                    dash_option(remote),
                    dash_option(noname),
                );
                let token = match mark {
                    CatalogMark::Incompatible => "incompatible",
                    CatalogMark::Updatable => "updatable",
                    CatalogMark::Same => "same",
                    CatalogMark::LocalHigher => "higher",
                    CatalogMark::Unmarked => "unmarked",
                };
                parts_out.push(format!("mark:{token}"));
            }
            "select" => {
                let engine = parts.next().unwrap_or("");
                let local = parts.next().unwrap_or("-");
                let remote = parts.next().unwrap_or("-");
                let noname = parts.next().unwrap_or("-");
                let allow_same = parts.next().unwrap_or("0") == "1";
                let allow_higher = parts.next().unwrap_or("0") == "1";
                let chosen = choose_download(&SelectInput {
                    engine_version: engine,
                    local_version: dash_option(local),
                    remote_version: dash_option(remote),
                    noname_version: dash_option(noname),
                    allow_same,
                    allow_higher,
                });
                parts_out.push(format!("select:{}", u8::from(chosen)));
            }
            "pdf" => match open_asset_file(parts.next().unwrap_or("noname_tutorial.pdf")) {
                Err(SjError::AssetRefused) => parts_out.push("asset:refused".into()),
                Ok(()) => parts_out.push("asset:opened".into()),
                Err(err) => return Err(err),
            },
            "console" => {
                let flag = parts.next().unwrap_or("");
                let environment = flag == "env" || session.environment;
                let already = flag == "open" || session.console_open;
                let request = request_console(config.open_tools, environment, already);
                if request.requested {
                    session.console_open = true;
                }
                parts_out.push(format!(
                    "console:{} eruda:{}",
                    u8::from(request.requested),
                    u8::from(request.eruda_started)
                ));
            }
            "fault" => {
                let kind = parts.next().unwrap_or("");
                let fault = match kind {
                    "404" => TransferFault {
                        exception: None,
                        http_status: HttpCode::Number(404),
                    },
                    "text" => TransferFault {
                        exception: None,
                        http_status: HttpCode::Text("404".into()),
                    },
                    "null" => TransferFault {
                        exception: None,
                        http_status: HttpCode::Null,
                    },
                    "host" => TransferFault {
                        exception: Some("Unable to resolve host: example".into()),
                        http_status: HttpCode::Number(500),
                    },
                    "lower" => TransferFault {
                        exception: Some("unable to resolve host".into()),
                        http_status: HttpCode::Number(500),
                    },
                    "missing" => TransferFault {
                        exception: None,
                        http_status: HttpCode::Absent,
                    },
                    _ => TransferFault {
                        exception: None,
                        http_status: HttpCode::Number(500),
                    },
                };
                let action = if classify_fault(&fault) == FaultAction::Skip {
                    "skip"
                } else {
                    "retry"
                };
                parts_out.push(format!("fault:{action}"));
            }
            "external" => {
                let kind = parts.next().unwrap_or("");
                let rest = parts.collect::<Vec<_>>().join(":");
                let external = match kind {
                    "extension" => ExternalKind::Extension { name: rest },
                    "package" => ExternalKind::Package { message: rest },
                    _ => ExternalKind::Other,
                };
                let effect = apply_external(
                    &external,
                    &session.extensions,
                    &session.enables,
                    &session.storage,
                );
                session.extensions = effect.extensions.clone();
                session.enables = effect.enables.clone();
                session.storage = effect.storage.clone();
                let inited = effect
                    .storage
                    .get(INITED_KEY)
                    .map(|text| format!(" inited:{text}"))
                    .unwrap_or_default();
                parts_out.push(format!(
                    "external:{kind} exts:{}{inited} reload:{} restarted:{}",
                    join_names(&effect.extensions),
                    u8::from(effect.reload_requested),
                    u8::from(effect.restarted)
                ));
            }
            "import" => {
                let mode = parts.next().unwrap_or("");
                let keep = parts.next().unwrap_or("");
                let payload = parts.collect::<Vec<_>>().join(":");
                let db = match mode {
                    "db" => true,
                    "nodb" => false,
                    _ => return Err(SjError::BadConfig(format!("unknown import {mode}"))),
                };
                let input = ImportInput {
                    payload,
                    db,
                    keep_existing: keep == "keep",
                    extensions: session.extensions.clone(),
                    storage: session.storage.clone(),
                    config_prefix: "noname_".into(),
                    config: session.config_map.clone(),
                    data: session.data.clone(),
                };
                match import_config(&input) {
                    Ok(output) if !output.applied => {
                        parts_out.push("import:noop reload:0 restarted:0".into())
                    }
                    Ok(output) => {
                        session.extensions = output.extensions.clone();
                        session.storage = output.storage.clone();
                        session.config_map = output.config.clone();
                        session.data = output.data.clone();
                        if let Some(Value::String(asset)) = output.config.get("asset_version") {
                            session.asset_version = Some(asset.clone());
                        }
                        let asset = session.asset_version.clone().unwrap_or_else(|| "-".into());
                        let note = output
                            .data
                            .get("note")
                            .and_then(Value::as_str)
                            .map(|text| format!(" note:{text}"))
                            .unwrap_or_default();
                        let store = if db {
                            String::new()
                        } else {
                            format!(" storage:{}", storage_token(&output.storage))
                        };
                        parts_out.push(format!(
                            "import:ok exts:{} asset:{asset}{note}{store} reload:{} restarted:{}",
                            join_names(&output.extensions),
                            u8::from(output.reload_requested),
                            u8::from(output.restarted)
                        ));
                    }
                    Err(SjError::BadEncoding(_)) => {
                        parts_out.push("import:bad reload:0 restarted:0".into())
                    }
                    Err(SjError::Format) => {
                        parts_out.push("import:format reload:0 restarted:0".into())
                    }
                    Err(err) => return Err(err),
                }
            }
            "onerror" => {
                let preset = parts.next().unwrap_or("plain");
                let report = match preset {
                    "plain" => plain_error(),
                    "bad" => WindowError {
                        source: Some("%".into()),
                        ..plain_error()
                    },
                    "stack" => WindowError {
                        stack: Some("%20x%3B".into()),
                        line: None,
                        column: None,
                        ..plain_error()
                    },
                    _ => return Err(SjError::BadConfig(format!("unknown onerror {preset}"))),
                };
                let formatted = format_window_error(&report)?;
                parts_out.push(format!(
                    "onerror:{} continued:{} looped:{}",
                    formatted.text.replace('\n', "\\n"),
                    u8::from(formatted.continued),
                    u8::from(formatted.looped)
                ));
            }
            "eval" => match eval_update_script(parts.next().unwrap_or("")) {
                Err(SjError::EvalRefused) => parts_out.push("eval:refused".into()),
                Ok(()) => parts_out.push("eval:ran".into()),
                Err(err) => return Err(err),
            },
            "catalog" => {
                let online = parts.next().unwrap_or("1") == "1";
                match load_remote_catalog(online) {
                    Err(SjError::Offline) => parts_out.push("catalog:offline".into()),
                    Err(SjError::NetworkRefused) => parts_out.push("catalog:refused".into()),
                    Ok(_) => parts_out.push("catalog:loaded".into()),
                    Err(err) => return Err(err),
                }
            }
            other => return Err(SjError::BadConfig(format!("unknown command {other}"))),
        }
    }
    Ok(parts_out.join(" "))
}
