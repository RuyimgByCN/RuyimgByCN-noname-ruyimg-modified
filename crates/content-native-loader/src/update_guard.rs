//! L18 更新保护的仍在运行的决策。
//!
//! `extension.js` 里检查游戏更新和检查素材更新都会排定两秒后的提示然后 `return`。
//! 其后的下载、`eval` 和写文件在块注释中，不执行。`canUpdate` 直接返回 true。
//! 本模块不请求网络，不写盘，不 `eval`，不打开菜单，不睡眠。

use serde_json::Value;

pub const CODING_URL: &str = "https://ghproxy.com/https://raw.githubusercontent.com/Show-K/noname";
pub const GITHUB_URL: &str = "https://raw.githubusercontent.com/Show-K/noname";
pub const FASTGIT_URL: &str = "https://raw.fgit.ml/libccy/noname";
pub const URC_URL: &str = "https://unitedrhythmized.club/libccy/noname";

const PLAYS: &[&str] = &["coin", "boss", "wuxing", "cardpile"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuardError {
    BadConfig(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuardConfig {
    pub source: String,
    pub upstream_path: String,
    pub upstream_commit: String,
    pub upstream_version: String,
    pub cancel_forbid: bool,
    pub auto_add_exts: bool,
    pub update_link: String,
    pub broken_file: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UpdateUrl {
    pub key: &'static str,
    pub url: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootLink {
    pub link: String,
    pub url: String,
    pub url_assigned: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectOutcome {
    pub link: String,
    pub url: String,
    pub accepted: bool,
    pub cache_cleared: bool,
    pub alert: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderIn {
    pub name: String,
    pub files: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanOut {
    pub added: Vec<String>,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdatePrompt {
    Same,
    LocalNewer,
    Prompt,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrokenOut {
    pub files: Vec<String>,
    pub would_redownload: bool,
    pub download_called: bool,
    pub reload_called: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ButtonOut {
    pub acted: bool,
    pub updating: bool,
    pub updating_for_asset: bool,
    pub unwanted: bool,
    pub alert: String,
    pub fetch_called: bool,
    pub write_called: bool,
    pub reload_called: bool,
    pub sleep_called: bool,
    pub eval_called: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatchKind {
    Object {
        url: String,
        error: String,
        progress: bool,
        message: String,
    },
    Abort,
    FailedFetch,
    Text(String),
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BodyClass {
    UserNotLogin,
    EvalRefused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Attempt {
    Ready,
    Fail,
    Login,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryClass {
    Manual,
    Ready,
    Login,
    MaxRetries,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MenuFlags {
    pub delete_removed: bool,
    pub enable_removed: bool,
    pub lock_enable: bool,
}

pub fn extension_urls() -> &'static [UpdateUrl] {
    const URLS: &[UpdateUrl] = &[
        UpdateUrl {
            key: "coding",
            url: CODING_URL,
        },
        UpdateUrl {
            key: "github",
            url: GITHUB_URL,
        },
        UpdateUrl {
            key: "fastgit",
            url: FASTGIT_URL,
        },
        UpdateUrl {
            key: "URC",
            url: URC_URL,
        },
    ];
    URLS
}

pub fn can_update() -> bool {
    true
}

pub fn parse_update_guard_config(text: &str) -> Result<GuardConfig, GuardError> {
    let value: Value =
        serde_json::from_str(text).map_err(|_| GuardError::BadConfig("json".into()))?;
    let obj = value
        .as_object()
        .ok_or_else(|| GuardError::BadConfig("json".into()))?;
    let source = required_str(obj, "source")?;
    if source.is_empty() {
        return Err(GuardError::BadConfig("source".into()));
    }
    let upstream_path = required_str(obj, "upstream_path")?;
    if upstream_path.is_empty() {
        return Err(GuardError::BadConfig("upstream_path".into()));
    }
    let upstream_commit = required_str(obj, "upstream_commit")?;
    if upstream_commit.is_empty() {
        return Err(GuardError::BadConfig("upstream_commit".into()));
    }
    let upstream_version = required_str(obj, "upstream_version")?;
    if upstream_version.is_empty() {
        return Err(GuardError::BadConfig("upstream_version".into()));
    }
    let broken_file = match obj.get("brokenFile") {
        None => Vec::new(),
        Some(Value::Array(items)) => items
            .iter()
            .map(|item| {
                item.as_str()
                    .map(str::to_string)
                    .ok_or_else(|| GuardError::BadConfig("brokenFile".into()))
            })
            .collect::<Result<Vec<_>, _>>()?,
        Some(_) => return Err(GuardError::BadConfig("brokenFile".into())),
    };
    Ok(GuardConfig {
        source: source.to_string(),
        upstream_path: upstream_path.to_string(),
        upstream_commit: upstream_commit.to_string(),
        upstream_version: upstream_version.to_string(),
        cancel_forbid: required_bool(obj, "cancelForbid")?,
        auto_add_exts: required_bool(obj, "autoAddExts")?,
        update_link: match obj.get("update_link") {
            None => String::new(),
            Some(Value::String(text)) => text.clone(),
            Some(_) => return Err(GuardError::BadConfig("update_link".into())),
        },
        broken_file,
    })
}

fn required_str<'a>(
    obj: &'a serde_json::Map<String, Value>,
    key: &'static str,
) -> Result<&'a str, GuardError> {
    match obj.get(key) {
        Some(Value::String(text)) => Ok(text),
        _ => Err(GuardError::BadConfig(key.into())),
    }
}

fn required_bool(
    obj: &serde_json::Map<String, Value>,
    key: &'static str,
) -> Result<bool, GuardError> {
    match obj.get(key) {
        Some(Value::Bool(value)) => Ok(*value),
        _ => Err(GuardError::BadConfig(key.into())),
    }
}

pub fn boot_link(stored: &str, current_url: &str) -> BootLink {
    if stored.is_empty() {
        BootLink {
            link: "URC".into(),
            url: URC_URL.into(),
            url_assigned: true,
        }
    } else {
        BootLink {
            link: stored.into(),
            url: current_url.into(),
            url_assigned: false,
        }
    }
}

pub fn on_remove_link() -> BootLink {
    BootLink {
        link: "coding".into(),
        url: CODING_URL.into(),
        url_assigned: true,
    }
}

pub fn select_extension_link(current: &str, item: &str) -> SelectOutcome {
    if item == current {
        return outcome(current, true, false, None);
    }
    if item == "xuanwu" {
        return outcome(
            current,
            false,
            false,
            Some("此更新源已弃用，请不要再使用此更新源".into()),
        );
    }
    if let Some(url) = url_of(item) {
        SelectOutcome {
            link: item.into(),
            url: url.into(),
            accepted: true,
            cache_cleared: true,
            alert: None,
        }
    } else {
        outcome(
            current,
            false,
            false,
            Some(format!("选择的更新源({item})不存在")),
        )
    }
}

pub fn select_general_link(current: &str, item: &str) -> SelectOutcome {
    if let Some(url) = url_of(item) {
        SelectOutcome {
            link: item.into(),
            url: url.into(),
            accepted: true,
            cache_cleared: false,
            alert: None,
        }
    } else {
        outcome(
            current,
            false,
            false,
            Some(format!("选择的更新源({item})不存在")),
        )
    }
}

fn outcome(
    current: &str,
    accepted: bool,
    cache_cleared: bool,
    alert: Option<String>,
) -> SelectOutcome {
    SelectOutcome {
        link: current.into(),
        url: url_of(current).unwrap_or("").into(),
        accepted,
        cache_cleared,
        alert,
    }
}

fn url_of(key: &str) -> Option<&'static str> {
    extension_urls()
        .iter()
        .find(|item| item.key == key)
        .map(|item| item.url)
}

pub fn scan_folders(auto_add: bool, existing: &[String], folders: &[FolderIn]) -> ScanOut {
    if !auto_add {
        return ScanOut {
            added: Vec::new(),
            message: String::new(),
        };
    }
    let mut added = Vec::new();
    for folder in folders {
        if existing.iter().any(|name| name == &folder.name)
            || PLAYS.contains(&folder.name.as_str())
            || folder.name == "SJ Settings"
        {
            continue;
        }
        if folder.files.iter().any(|file| file == "extension.js") {
            added.push(folder.name.clone());
        }
    }
    let message = if added.is_empty() {
        String::new()
    } else {
        format!("已识别扩展：{}", added.join("、"))
    };
    ScanOut { added, message }
}

pub fn parse_size(limit: f64) -> String {
    let (scaled, unit) = if limit < 1024.0 {
        (limit, "B")
    } else if limit < 1024.0 * 1024.0 {
        (limit / 1024.0, "KB")
    } else if limit < 1024.0 * 1024.0 * 1024.0 {
        (limit / (1024.0 * 1024.0), "MB")
    } else {
        (limit / (1024.0 * 1024.0 * 1024.0), "GB")
    };
    let size = format!("{scaled:.2}{unit}");
    let index = size.find('.').map(|pos| pos as i32).unwrap_or(-1);
    let dou = js_slice(&size, index + 1, 2);
    if dou == "00" {
        return format!(
            "{}{}",
            js_slice(&size, 0, index),
            js_slice(&size, index + 3, 2)
        );
    }
    size
}

fn js_slice(text: &str, start: i32, end: i32) -> String {
    let len = text.len() as i32;
    let mut from = start;
    let mut to = end;
    if from < 0 {
        from += len;
    }
    if to < 0 {
        to += len;
    }
    if from < 0 {
        from = 0;
    }
    if to < 0 {
        to = 0;
    }
    if from > len {
        from = len;
    }
    if to > len {
        to = len;
    }
    if from >= to {
        return String::new();
    }
    text[from as usize..to as usize].to_string()
}

fn js_number(token: &str) -> f64 {
    let value = token.parse::<f64>().unwrap_or(f64::NAN);
    if value.is_nan() {
        0.0
    } else {
        value
    }
}

pub fn compare_update(local: &str, remote: &str) -> UpdatePrompt {
    if local == remote {
        return UpdatePrompt::Same;
    }
    let left: Vec<f64> = local.split('.').map(js_number).collect();
    let right: Vec<f64> = remote.split('.').map(js_number).collect();
    for index in 0..left.len().min(right.len()) {
        let current = left[index];
        let server = right[index];
        if server > current {
            break;
        }
        if current > server {
            return UpdatePrompt::LocalNewer;
        }
    }
    UpdatePrompt::Prompt
}

pub fn join_changelog(lines: &[String]) -> String {
    if lines.is_empty() {
        return String::new();
    }
    let mut text = lines[0].clone();
    for line in lines.iter().skip(1) {
        if !line.contains("://") {
            text.push('；');
            text.push_str(line);
        }
    }
    text
}

pub fn dedup_broken(files: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for file in files {
        if !out.iter().any(|kept| kept == file) {
            out.push(file.clone());
        }
    }
    out
}

pub fn settle_broken(files: Option<&[String]>, confirm: bool) -> BrokenOut {
    let Some(files) = files else {
        return BrokenOut {
            files: Vec::new(),
            would_redownload: false,
            download_called: false,
            reload_called: false,
        };
    };
    let files = dedup_broken(files);
    if files.is_empty() || !confirm {
        return BrokenOut {
            files: if confirm { files } else { Vec::new() },
            would_redownload: false,
            download_called: false,
            reload_called: false,
        };
    }
    BrokenOut {
        files,
        would_redownload: true,
        download_called: false,
        reload_called: false,
    }
}

pub fn press_game_button(label: &str) -> ButtonOut {
    if label != "检查游戏更新" {
        return idle_button();
    }
    ButtonOut {
        acted: true,
        updating: false,
        updating_for_asset: false,
        unwanted: false,
        alert: "当前版本已是最新".into(),
        fetch_called: false,
        write_called: false,
        reload_called: false,
        sleep_called: false,
        eval_called: false,
    }
}

pub fn press_asset_button(label: &str) -> ButtonOut {
    if label != "检查素材更新" {
        return idle_button();
    }
    ButtonOut {
        acted: true,
        updating: false,
        updating_for_asset: true,
        unwanted: false,
        alert: "当前素材已是最新".into(),
        fetch_called: false,
        write_called: false,
        reload_called: false,
        sleep_called: false,
        eval_called: false,
    }
}

fn idle_button() -> ButtonOut {
    ButtonOut {
        acted: false,
        updating: false,
        updating_for_asset: false,
        unwanted: false,
        alert: String::new(),
        fetch_called: false,
        write_called: false,
        reload_called: false,
        sleep_called: false,
        eval_called: false,
    }
}

pub fn next_error_count(current: u32) -> u32 {
    let next = current.saturating_add(1);
    if next > 5 {
        0
    } else {
        next
    }
}

pub fn format_catch(kind: &CatchKind, update_url: &str, errors: u32) -> (Option<String>, u32) {
    let alert = match kind {
        CatchKind::Object {
            url,
            error,
            progress,
            message,
        } => {
            let mut translated = None;
            for item in extension_urls() {
                if url.starts_with(item.url) {
                    translated = translate(item.key);
                    break;
                }
            }
            let shown = url.replacen(&format!("{update_url}/master/"), "", 1);
            Some(if let Some(name) = translated {
                if *progress {
                    format!("更新源:{name}\n网络请求目标：{shown}\n提示:{message}")
                } else {
                    format!("更新源:{name}\n网络请求目标：{shown}\n状态消息或状态码：{error}\n提示:{message}")
                }
            } else if *progress {
                format!("网络请求目标：{url}\n提示:{message}")
            } else {
                format!("网络请求目标：{url}\n状态消息或状态码：{error}\n提示:{message}")
            })
        }
        CatchKind::Abort => Some("网络连接超时".into()),
        CatchKind::FailedFetch => Some("网络请求失败".into()),
        CatchKind::Text(text) => Some(text.clone()),
        CatchKind::Other => None,
    };
    (alert, next_error_count(errors))
}

fn translate(key: &str) -> Option<&'static str> {
    match key {
        "coding" => Some("Coding"),
        "github" => Some("GitHub"),
        "fastgit" => Some("GitHub镜像"),
        "xuanwu" => Some("玄武镜像"),
        _ => None,
    }
}

pub fn classify_body(text: &str) -> (BodyClass, bool) {
    if let Ok(data) = serde_json::from_str::<Value>(text) {
        if let Some(message) = data.get("msg").and_then(Value::as_object) {
            if message.get("user_not_login").and_then(Value::as_str) == Some("用户未登录") {
                return (BodyClass::UserNotLogin, false);
            }
        }
    }
    (BodyClass::EvalRefused, false)
}

pub fn retry_updates(
    has_download: bool,
    cached: bool,
    attempts: &[Attempt],
) -> (RetryClass, &'static str, bool) {
    if !has_download {
        return (
            RetryClass::Manual,
            "此版本不支持游戏内更新，请手动更新",
            false,
        );
    }
    if cached {
        return (RetryClass::Ready, "ready", false);
    }
    let mut index = 0;
    let mut failures = 0;
    let mut ready = false;
    while !ready && failures < 5 {
        let Some(attempt) = attempts.get(index) else {
            return (RetryClass::Other, "遇到其他错误, 请重试", false);
        };
        index += 1;
        match attempt {
            Attempt::Ready => ready = true,
            Attempt::Login => return (RetryClass::Login, "user_not_login", false),
            Attempt::Fail => failures += 1,
        }
    }
    if failures == 5 && !ready {
        (
            RetryClass::MaxRetries,
            "达到最大重试次数(5次), 请重试",
            false,
        )
    } else if ready {
        (RetryClass::Ready, "ready", false)
    } else {
        (RetryClass::Other, "遇到其他错误, 请重试", false)
    }
}

pub fn forced_enables(cancel_forbid: bool, names: Option<&[String]>) -> Vec<String> {
    match (cancel_forbid, names) {
        (true, Some(names)) => names.to_vec(),
        _ => Vec::new(),
    }
}

pub fn precontent_menu(cancel_forbid: bool) -> MenuFlags {
    MenuFlags {
        delete_removed: true,
        enable_removed: true,
        lock_enable: cancel_forbid,
    }
}

pub fn apply_update_guard_line(
    config_text: &str,
    request_text: &str,
) -> Result<String, GuardError> {
    let config = parse_update_guard_config(config_text)?;
    let request: Value =
        serde_json::from_str(request_text).map_err(|_| GuardError::BadConfig("request".into()))?;
    let obj = request
        .as_object()
        .ok_or_else(|| GuardError::BadConfig("request".into()))?;
    let current_url = optional_str(obj, "currentUrl")?.unwrap_or("");
    let boot = boot_link(&config.update_link, current_url);
    let folders = match obj.get("folders") {
        None => default_folders(),
        Some(Value::Array(items)) => items.iter().map(folder_in).collect::<Result<Vec<_>, _>>()?,
        Some(_) => return Err(GuardError::BadConfig("folders".into())),
    };
    let existing = match obj.get("existing") {
        None => vec!["已有".into()],
        Some(Value::Array(items)) => string_list(items, "existing")?,
        Some(_) => return Err(GuardError::BadConfig("existing".into())),
    };
    let scan = scan_folders(config.auto_add_exts, &existing, &folders);
    let limit = match obj.get("limit") {
        None => 1024.0,
        Some(Value::Number(number)) => number
            .as_f64()
            .ok_or_else(|| GuardError::BadConfig("limit".into()))?,
        Some(_) => return Err(GuardError::BadConfig("limit".into())),
    };
    let local = optional_str(obj, "local")?.unwrap_or("1.2");
    let remote = optional_str(obj, "remote")?.unwrap_or("1.10");
    let changelog = match obj.get("changelog") {
        None => vec!["甲".into(), "https://a".into(), "丙".into()],
        Some(Value::Array(items)) => string_list(items, "changelog")?,
        Some(_) => return Err(GuardError::BadConfig("changelog".into())),
    };
    let broken = match obj.get("broken") {
        None => Some(vec!["a".into(), "b".into(), "a".into()]),
        Some(Value::Null) => None,
        Some(Value::Array(items)) => Some(string_list(items, "broken")?),
        Some(_) => return Err(GuardError::BadConfig("broken".into())),
    };
    let confirm = match obj.get("confirmBroken") {
        None => false,
        Some(Value::Bool(value)) => *value,
        Some(_) => return Err(GuardError::BadConfig("confirmBroken".into())),
    };
    let settled = settle_broken(broken.as_deref(), confirm);
    let game_label = optional_str(obj, "gameLabel")?.unwrap_or("检查游戏更新");
    let asset_label = optional_str(obj, "assetLabel")?.unwrap_or("检查素材更新");
    let game = press_game_button(game_label);
    let asset = press_asset_button(asset_label);
    let select_current = optional_str(obj, "selectCurrent")?.unwrap_or("URC");
    let select_item = optional_str(obj, "selectItem")?.unwrap_or("github");
    let selected = select_extension_link(select_current, select_item);
    let xuanwu = select_extension_link(select_current, "xuanwu");
    let body = optional_str(obj, "body")?.unwrap_or(r#"{"msg":{"user_not_login":"用户未登录"}}"#);
    let (class, eval_called) = classify_body(body);
    let has_download = match obj.get("hasDownload") {
        None => false,
        Some(Value::Bool(value)) => *value,
        Some(_) => return Err(GuardError::BadConfig("hasDownload".into())),
    };
    let cached = match obj.get("cached") {
        None => true,
        Some(Value::Bool(value)) => *value,
        Some(_) => return Err(GuardError::BadConfig("cached".into())),
    };
    let (retry, _, retry_eval) = retry_updates(has_download, cached, &[]);
    let lock_names = match obj.get("lockNames") {
        None => Some(vec!["十周年UI".into(), "手杀ui".into()]),
        Some(Value::Null) => None,
        Some(Value::Array(items)) => Some(string_list(items, "lockNames")?),
        Some(_) => return Err(GuardError::BadConfig("lockNames".into())),
    };
    let locked = forced_enables(config.cancel_forbid, lock_names.as_deref());
    if eval_called || retry_eval || game.eval_called || asset.eval_called {
        return Err(GuardError::BadConfig("eval".into()));
    }
    let removed = on_remove_link();
    Ok(format!(
        "link:{} | url:{} | remove:{} | scan:{} | size:{} | cmp:{} | log:{} | broken:{} | would:{} | game:{} | asset:{} | asset_flag:{} | updating:{} | select:{} | xuanwu:{} | body:{} | retry:{} | lock:{} | can:{} | eval:0 | fetch:0 | write:0 | reload:0 | sleep:0 | dom:0",
        boot.link,
        boot.url,
        removed.link,
        dash_list(&scan.added),
        parse_size(limit),
        match compare_update(local, remote) {
            UpdatePrompt::Same => "same",
            UpdatePrompt::LocalNewer => "local",
            UpdatePrompt::Prompt => "prompt",
        },
        dash_text(&join_changelog(&changelog)),
        dash_list(&settled.files),
        if settled.would_redownload { "1" } else { "0" },
        dash_text(&game.alert),
        dash_text(&asset.alert),
        if asset.updating_for_asset { "1" } else { "0" },
        if game.updating { "1" } else { "0" },
        selected.link,
        if xuanwu.accepted { "1" } else { "0" },
        match class {
            BodyClass::UserNotLogin => "login",
            BodyClass::EvalRefused => "refused",
        },
        match retry {
            RetryClass::Manual => "manual",
            RetryClass::Ready => "ready",
            RetryClass::Login => "login",
            RetryClass::MaxRetries => "max",
            RetryClass::Other => "other",
        },
        dash_list(&locked),
        if can_update() { "1" } else { "0" },
    ))
}

fn optional_str<'a>(
    obj: &'a serde_json::Map<String, Value>,
    key: &'static str,
) -> Result<Option<&'a str>, GuardError> {
    match obj.get(key) {
        None => Ok(None),
        Some(Value::String(text)) => Ok(Some(text)),
        Some(_) => Err(GuardError::BadConfig(key.into())),
    }
}

fn string_list(items: &[Value], key: &'static str) -> Result<Vec<String>, GuardError> {
    items
        .iter()
        .map(|item| {
            item.as_str()
                .map(str::to_string)
                .ok_or_else(|| GuardError::BadConfig(key.into()))
        })
        .collect()
}

fn folder_in(value: &Value) -> Result<FolderIn, GuardError> {
    let obj = value
        .as_object()
        .ok_or_else(|| GuardError::BadConfig("folders".into()))?;
    let name = required_str(obj, "name")?.to_string();
    let files = match obj.get("files") {
        Some(Value::Array(items)) => string_list(items, "files")?,
        _ => return Err(GuardError::BadConfig("files".into())),
    };
    Ok(FolderIn { name, files })
}

fn default_folders() -> Vec<FolderIn> {
    vec![
        FolderIn {
            name: "coin".into(),
            files: vec!["extension.js".into()],
        },
        FolderIn {
            name: "甲".into(),
            files: vec!["extension.js".into()],
        },
        FolderIn {
            name: "SJ Settings".into(),
            files: vec!["extension.js".into()],
        },
        FolderIn {
            name: "乙".into(),
            files: vec!["readme.md".into()],
        },
        FolderIn {
            name: "丙".into(),
            files: vec!["extension.js".into()],
        },
    ]
}

fn dash_list(items: &[String]) -> String {
    if items.is_empty() {
        "-".into()
    } else {
        items.join(",")
    }
}

fn dash_text(text: &str) -> String {
    if text.is_empty() {
        "-".into()
    } else {
        text.to_string()
    }
}
