//! L02 SJ Settings：外部扩展名单、配置导入、版本比较和下载筛选。
//!
//! 启动名单、`lib.init.decode`、数据库与非数据库导入、`compareVersion`、
//! 目录标记、勾选确认、文件失败分类和 `window.onerror` 文本按 liuli 的
//! `extension/SJ Settings/extension.js` 计算。Cordova 复制、FileTransfer、
//! 权限、`fetch`、`eval`、eruda、PDF、进度条、`alert` 和真正的
//! `game.reload` / `game.loop` 不执行。

use std::collections::BTreeMap;

use serde_json::{Map, Value};
mod codec;
pub use codec::*;
mod commands;
pub use commands::*;

const SJ: &str = "SJ Settings";
const ANDROID_KEY: &str = "noname_android_extension";
const INITED_KEY: &str = "noname_inited";

/// 包内配置。`openTools` 的初始值是假。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SjConfig {
    /// 来源仓。缺省键解析为 `noname`。
    pub source: String,
    /// 上游脚本路径。
    pub upstream_path: String,
    /// 上游提交。
    pub upstream_commit: String,
    /// 上游 `package.version`。本包装载版本仍是 `1.0.0`。
    pub upstream_version: String,
    /// `openTools`。为真也不启动 eruda。
    pub open_tools: bool,
}

/// SJ Settings 的失败。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SjError {
    /// 配置无法使用。
    BadConfig(String),
    /// 启动名单无法解析。键不会被删掉。
    BadPayload(String),
    /// Base64 或 `decodeURI` 无法使用。
    BadEncoding(String),
    /// 导入对象的形状不对。存储保持原样。
    Format,
    /// PDF 和资源文件不打开。
    AssetRefused,
    /// 不请求扩展目录。
    NetworkRefused,
    /// 调用方声明离线。
    Offline,
    /// 不执行 `update.js`。
    EvalRefused,
}

/// 启动前的存储和扩展名单。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootInput {
    /// `localStorage`。
    pub storage: BTreeMap<String, String>,
    /// `lib.config.extensions`。
    pub extensions: Vec<String>,
    /// `extension_${name}_enable`。缺省视为假。
    pub enables: BTreeMap<String, bool>,
    /// `lib.config.asset_version`。空串视为缺省。
    pub asset_version: Option<String>,
}

/// 启动后的名单。不创建界面。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootOutput {
    /// 成功解析非空键之后，该键已删除。
    pub storage: BTreeMap<String, String>,
    /// 补过缺名之后的扩展名单。
    pub extensions: Vec<String>,
    /// 原来为假或缺省的名字被写成真。
    pub enables: BTreeMap<String, bool>,
    /// 缺省时写成 `无`。
    pub asset_version: String,
    /// 本次是否删过 Android 键。
    pub android_key_removed: bool,
}

/// 一次配置导入。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportInput {
    /// 文件原文。空串是空操作。
    pub payload: String,
    /// `lib.db`。
    pub db: bool,
    /// 仅数据库模式使用。非数据库模式忽略。
    pub keep_existing: bool,
    /// 导入前的 `lib.config.extensions`。数据库模式先记下它。
    pub extensions: Vec<String>,
    /// `localStorage`。
    pub storage: BTreeMap<String, String>,
    /// `lib.configprefix`。在线键是此前缀加 `key`。
    pub config_prefix: String,
    /// 数据库模式的 `lib.config`。
    pub config: BTreeMap<String, Value>,
    /// 数据库模式的 `game` data。
    pub data: BTreeMap<String, Value>,
}

/// 导入结果。成功时 `restarted` 仍为假。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportOutput {
    /// 非数据库模式不改这个名单。
    pub extensions: Vec<String>,
    /// 非数据库模式会先清空，再写回保留键和导入键。
    pub storage: BTreeMap<String, String>,
    /// 数据库模式按导入对象覆盖。
    pub config: BTreeMap<String, Value>,
    /// 数据库模式按导入对象覆盖。值保持 JSON，不转存储字符串。
    pub data: BTreeMap<String, Value>,
    /// 成功时为真。调用方还没有重启。
    pub reload_requested: bool,
    /// 不调用 `game.reload`。
    pub restarted: bool,
    /// 非数据库成功时为真。
    pub cleared: bool,
    /// 空原文为假。
    pub applied: bool,
}

/// `compareVersion` 的四种结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionOrder {
    /// 远端更新。对应 `true`。
    Newer,
    /// 本地更高。对应 `false`。
    Older,
    /// 相同。对应 `'equal'`。
    Equal,
    /// 没有可比出先后。对应 `undefined`。
    Unordered,
}

/// 目录里一条扩展旁边的标记。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatalogMark {
    /// `lib.version != nonameVersion`，用字符串不等，不走 `compareVersion`。
    Incompatible,
    /// `compareVersion` 为 `true`。
    Updatable,
    /// `compareVersion` 为 `'equal'`。
    Same,
    /// `false` 或 `undefined`。
    LocalHigher,
    /// 没有可比较的版本字符串。
    Unmarked,
}

/// 勾选下载时调用方已经回答的确认。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectInput<'a> {
    /// `lib.version`。
    pub engine_version: &'a str,
    /// 已安装扩展的 `version`。缺省表示没有可比较的本地包。
    pub local_version: Option<&'a str>,
    /// 目录里的 `version`。缺省按默认参数 `''` 比较。
    pub remote_version: Option<&'a str>,
    /// 目录里的 `nonameVersion`。缺省表示不是字符串。
    pub noname_version: Option<&'a str>,
    /// 版本相同的确认。
    pub allow_same: bool,
    /// 本地可能更高的确认。
    pub allow_higher: bool,
}

/// 调用方给出的目录条目。本模块不下载它。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogEntry {
    /// 目录版本。
    pub version: Option<String>,
    /// 要求的无名杀版本。
    pub noname_version: Option<String>,
    /// `files`。`None` 表示不是数组。
    pub files: Option<Vec<String>>,
}

/// 已安装包的版本。`None` 表示包在，但没有 version 字段。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstalledPack {
    /// `lib.extensionPack[name].version`。
    pub version: Option<String>,
}

/// `FileTransfer` 错误的 http 码。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpCode {
    /// `http_status === null`。
    Null,
    /// 属性不存在。
    Absent,
    /// 数字码。
    Number(i64),
    /// 字符串码。
    Text(String),
}

/// 单个文件的失败。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferFault {
    /// `e.exception`。不是字符串时为 `None`。
    pub exception: Option<String>,
    /// `e.http_status`。
    pub http_status: HttpCode,
}

/// 失败后是跳过还是重试。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaultAction {
    /// 主机解析失败、http 码为 null，或 404。
    Skip,
    /// 其余失败。
    Retry,
}

/// 下载结束后的启用结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finish {
    /// 有文件数组的名字被补进名单。
    pub extensions: Vec<String>,
    /// 本次写成真的启用项，并保留原来的项。
    pub enables: BTreeMap<String, bool>,
    /// 名单非空且调用方确认重启。
    pub reload_requested: bool,
    /// 不调用 `game.reload`。
    pub restarted: bool,
}

/// 其他应用交来的导入结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExternalKind {
    /// `type == 'extension'`。
    Extension { name: String },
    /// `type == 'package'`。
    Package { message: String },
    /// 其余，包括空结果。
    Other,
}

/// 外部导入的效果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalEffect {
    /// 扩展名单。
    pub extensions: Vec<String>,
    /// 启用项。
    pub enables: BTreeMap<String, bool>,
    /// `localStorage`。
    pub storage: BTreeMap<String, String>,
    /// 扩展和整包都会请求重启。
    pub reload_requested: bool,
    /// 不调用 `game.reload`。
    pub restarted: bool,
}

/// 控制台请求。`eruda_started` 保持假。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConsoleRequest {
    /// 配置开启、环境不是打包环境、控制台尚未打开。
    pub requested: bool,
    /// 不调用 `eruda.init`。
    pub eruda_started: bool,
}

/// 计划中的文件名。不拼下载主机，也不取文件。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilePlan {
    /// 目录里的文件名。空数组保持为空。
    pub names: Vec<String>,
    /// 不发起网络请求。
    pub fetched: bool,
    /// 不执行脚本。
    pub evaled: bool,
}

/// `window.onerror` 里的一层事件。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    /// `event.name`。
    pub name: String,
    /// `event.step`。
    pub step: String,
    /// `event.parent`。
    pub parent: Option<Box<Frame>>,
}

/// 事件上的玩家、目标或来源。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Actor {
    /// `player.name`。
    pub name: String,
    /// `get.distance`。`None` 表示 `Infinity`。
    pub distance: Option<i64>,
}

/// 出错时的事件。没有事件时不写 event 行。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameEvent {
    /// 当前事件和它的父事件。
    pub frame: Frame,
    /// `evt.player`。
    pub player: Option<Actor>,
    /// `evt.target`。
    pub target: Option<Actor>,
    /// `evt.source`。
    pub source: Option<Actor>,
    /// `evt.skill`。
    pub skill: Option<String>,
    /// `evt.card.name`。
    pub card: Option<String>,
}

/// 调用方交给格式化函数的错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowError {
    /// `msg`。`None` 打印 `undefined`。
    pub message: Option<String>,
    /// `src`。`None` 走 `decodeURI(undefined)`。
    pub source: Option<String>,
    /// 行号。
    pub line: Option<String>,
    /// 列号。
    pub column: Option<String>,
    /// `err.stack`。空串不附加。
    pub stack: Option<String>,
    /// `_status.event`。
    pub event: Option<GameEvent>,
    /// `lib.translate`。空串视为假。
    pub translate: BTreeMap<String, String>,
}

/// 错误文本。不弹窗，也不回到 `game.loop`。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormattedError {
    /// 告警原文。
    pub text: String,
    /// `errstop` 为假时上游会继续循环。这里不继续。
    pub continued: bool,
    /// 不调用 `game.loop`。
    pub looped: bool,
}

fn source_token(value: Option<&Value>) -> Result<String, SjError> {
    match value {
        None => Ok("noname".to_string()),
        Some(Value::String(text))
            if !text.is_empty()
                && text
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_') =>
        {
            Ok(text.clone())
        }
        Some(_) => Err(SjError::BadConfig("source must be a token".into())),
    }
}

fn required_text(obj: &Map<String, Value>, key: &str) -> Result<String, SjError> {
    match obj.get(key).and_then(Value::as_str) {
        Some(text) if !text.is_empty() => Ok(text.to_string()),
        _ => Err(SjError::BadConfig(format!(
            "{key} must be a non-empty string"
        ))),
    }
}

/// 解析包内 `config.json`。
pub fn parse_sj_config(text: &str) -> Result<SjConfig, SjError> {
    let value: Value =
        serde_json::from_str(text).map_err(|err| SjError::BadConfig(err.to_string()))?;
    let obj = value
        .as_object()
        .ok_or_else(|| SjError::BadConfig("config must be an object".into()))?;
    let open_tools = obj
        .get("openTools")
        .and_then(Value::as_bool)
        .ok_or_else(|| SjError::BadConfig("openTools must be a bool".into()))?;
    Ok(SjConfig {
        source: source_token(obj.get("source"))?,
        upstream_path: required_text(obj, "upstream_path")?,
        upstream_commit: required_text(obj, "upstream_commit")?,
        upstream_version: required_text(obj, "upstream_version")?,
        open_tools,
    })
}

fn push_missing(names: &mut Vec<String>, name: &str) {
    if !names.iter().any(|item| item == name) {
        names.push(name.to_string());
    }
}

fn union_names(first: &[String], second: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for name in first.iter().chain(second.iter()) {
        push_missing(&mut out, name);
    }
    out
}

/// 处理 Android 扩展键。解析失败时不改调用方持有的输入。
pub fn boot(input: &BootInput) -> Result<BootOutput, SjError> {
    let mut storage = input.storage.clone();
    let mut extensions = input.extensions.clone();
    let mut enables = input.enables.clone();
    let listed = if let Some(raw) = input.storage.get(ANDROID_KEY) {
        if raw.is_empty() {
            None
        } else {
            let value: Value =
                serde_json::from_str(raw).map_err(|err| SjError::BadPayload(err.to_string()))?;
            let list = value
                .as_array()
                .ok_or_else(|| SjError::BadPayload("extension list".into()))?;
            let mut names = Vec::with_capacity(list.len());
            for item in list {
                let name = item
                    .as_str()
                    .ok_or_else(|| SjError::BadPayload("extension name".into()))?;
                names.push(name.to_string());
            }
            storage.remove(ANDROID_KEY);
            Some(names)
        }
    } else {
        None
    };
    let android_key_removed = listed.is_some();
    let names = listed.unwrap_or_else(|| vec![SJ.to_string()]);
    for name in &names {
        push_missing(&mut extensions, name);
        if !enables.get(name).copied().unwrap_or(false) {
            enables.insert(name.clone(), true);
        }
    }
    let asset_version = match &input.asset_version {
        Some(text) if !text.is_empty() => text.clone(),
        _ => "无".to_string(),
    };
    Ok(BootOutput {
        storage,
        extensions,
        enables,
        asset_version,
        android_key_removed,
    })
}

fn split_parts(text: &str) -> Vec<f64> {
    text.split('.')
        .map(|item| js_number(item).filter(|value| *value != 0.0).unwrap_or(0.0))
        .collect()
}

impl VersionOrder {
    fn token(self) -> &'static str {
        match self {
            VersionOrder::Older => "false",
            VersionOrder::Newer => "true",
            VersionOrder::Equal => "equal",
            VersionOrder::Unordered => "undefined",
        }
    }
}

/// 比较本地版本和远端版本。
pub fn compare_version(local: &str, remote: &str) -> VersionOrder {
    if local == remote {
        return VersionOrder::Equal;
    }
    if let (Some(left), Some(right)) = (js_number(local), js_number(remote)) {
        return if left > right {
            VersionOrder::Older
        } else if left < right {
            VersionOrder::Newer
        } else {
            VersionOrder::Equal
        };
    }
    let left = split_parts(local);
    let right = split_parts(remote);
    if left.len() == 1 && left[0] == 0.0 {
        if right.len() > 1 || right.first().copied().unwrap_or(0.0) > 0.0 {
            VersionOrder::Newer
        } else {
            VersionOrder::Unordered
        }
    } else if right.len() == 1 && right[0] == 0.0 {
        VersionOrder::Newer
    } else {
        let width = left.len().min(right.len());
        for index in 0..width {
            if left[index] > right[index] {
                return VersionOrder::Older;
            }
            if right[index] > left[index] {
                return VersionOrder::Newer;
            }
        }
        VersionOrder::Unordered
    }
}

/// 目录条目的版本标记。
pub fn catalog_mark(
    engine_version: &str,
    local_version: Option<&str>,
    remote_version: Option<&str>,
    noname_version: Option<&str>,
) -> CatalogMark {
    if let Some(required) = noname_version {
        if engine_version != required {
            return CatalogMark::Incompatible;
        }
    }
    match remote_version {
        Some(remote) if !remote.is_empty() => match local_version {
            Some(local) => match compare_version(local, remote) {
                VersionOrder::Newer => CatalogMark::Updatable,
                VersionOrder::Equal => CatalogMark::Same,
                VersionOrder::Older | VersionOrder::Unordered => CatalogMark::LocalHigher,
            },
            None => CatalogMark::Unmarked,
        },
        _ => CatalogMark::Unmarked,
    }
}

/// 勾选后是否留下这个扩展。不兼容直接取消。
pub fn choose_download(input: &SelectInput<'_>) -> bool {
    if let Some(required) = input.noname_version {
        if input.engine_version != required {
            return false;
        }
    }
    match input.local_version {
        Some(local) => match compare_version(local, input.remote_version.unwrap_or("")) {
            VersionOrder::Equal => input.allow_same,
            VersionOrder::Older => input.allow_higher,
            VersionOrder::Newer | VersionOrder::Unordered => true,
        },
        None => true,
    }
}

fn versions_same(local: &Option<String>, remote: &Option<String>) -> bool {
    match (local, remote) {
        (None, None) => true,
        (Some(left), Some(right)) => left == right,
        _ => false,
    }
}

/// 筛掉目录里没有文件数组的名字。版本相同才看确认。保持原顺序。
pub fn filter_downloads(
    requested: &[String],
    catalog: &BTreeMap<String, CatalogEntry>,
    installed: &[String],
    packs: &BTreeMap<String, InstalledPack>,
    allow_same: bool,
) -> Vec<String> {
    requested
        .iter()
        .filter(|name| {
            let Some(entry) = catalog.get(*name) else {
                return false;
            };
            if entry.files.is_none() {
                return false;
            }
            if !installed.iter().any(|item| item == *name) {
                return true;
            }
            let Some(pack) = packs.get(*name) else {
                return true;
            };
            if versions_same(&pack.version, &entry.version) {
                allow_same
            } else {
                true
            }
        })
        .cloned()
        .collect()
}

/// 主机解析失败、空 http 码和 404 跳过，其余重试。
pub fn classify_fault(fault: &TransferFault) -> FaultAction {
    if fault
        .exception
        .as_deref()
        .is_some_and(|text| text.starts_with("Unable to resolve host"))
    {
        return FaultAction::Skip;
    }
    match &fault.http_status {
        HttpCode::Null => FaultAction::Skip,
        HttpCode::Number(404) => FaultAction::Skip,
        HttpCode::Text(text) if text == "404" => FaultAction::Skip,
        _ => FaultAction::Retry,
    }
}

/// 结束下载。只有带文件数组的名字才启用。空文件数组也算数组。
pub fn finish_download(
    names: &[String],
    catalog: &BTreeMap<String, CatalogEntry>,
    extensions: &[String],
    enables: &BTreeMap<String, bool>,
    confirm_reload: bool,
) -> Finish {
    let mut extensions = extensions.to_vec();
    let mut enables = enables.clone();
    for name in names {
        if catalog
            .get(name)
            .and_then(|entry| entry.files.as_ref())
            .is_some()
        {
            push_missing(&mut extensions, name);
            enables.insert(name.clone(), true);
        }
    }
    Finish {
        extensions,
        enables,
        reload_requested: confirm_reload && !names.is_empty(),
        restarted: false,
    }
}

/// 其他应用的导入结果。扩展名已有时不重复添加。
pub fn apply_external(
    kind: &ExternalKind,
    extensions: &[String],
    enables: &BTreeMap<String, bool>,
    storage: &BTreeMap<String, String>,
) -> ExternalEffect {
    let mut extensions = extensions.to_vec();
    let mut enables = enables.clone();
    let mut storage = storage.clone();
    match kind {
        ExternalKind::Extension { name } => {
            push_missing(&mut extensions, name);
            enables.insert(name.clone(), true);
            ExternalEffect {
                extensions,
                enables,
                storage,
                reload_requested: true,
                restarted: false,
            }
        }
        ExternalKind::Package { message } => {
            storage.insert(INITED_KEY.to_string(), message.clone());
            ExternalEffect {
                extensions,
                enables,
                storage,
                reload_requested: true,
                restarted: false,
            }
        }
        ExternalKind::Other => ExternalEffect {
            extensions,
            enables,
            storage,
            reload_requested: false,
            restarted: false,
        },
    }
}

/// 打开 PDF 或资源文件。始终拒绝。
pub fn open_asset_file(name: &str) -> Result<(), SjError> {
    let _ = name;
    Err(SjError::AssetRefused)
}

/// 是否请求控制台。请求成功也不调用 `eruda.init`。
pub fn request_console(open_tools: bool, environment: bool, already_open: bool) -> ConsoleRequest {
    ConsoleRequest {
        requested: open_tools && !environment && !already_open,
        eruda_started: false,
    }
}

/// 获取在线目录。离线直接返回，在线也不发起请求。
pub fn load_remote_catalog(online: bool) -> Result<BTreeMap<String, CatalogEntry>, SjError> {
    if !online {
        Err(SjError::Offline)
    } else {
        Err(SjError::NetworkRefused)
    }
}

/// 不执行目录脚本。
pub fn eval_update_script(source: &str) -> Result<(), SjError> {
    let _ = source;
    Err(SjError::EvalRefused)
}

/// 文件计划。空数组不补一个下载地址。
pub fn plan_files(files: &[String]) -> FilePlan {
    FilePlan {
        names: files.to_vec(),
        fetched: false,
        evaled: false,
    }
}

fn show_part(value: &Option<String>) -> String {
    value.clone().unwrap_or_else(|| "undefined".into())
}

fn translated<'a>(map: &'a BTreeMap<String, String>, key: &str) -> Option<&'a str> {
    map.get(key)
        .map(String::as_str)
        .filter(|text| !text.is_empty())
}

/// 组装 `window.onerror` 的文本。解码失败时返回错误，不中断对局循环。
pub fn format_window_error(report: &WindowError) -> Result<FormattedError, SjError> {
    let source = match &report.source {
        None => "undefined".to_string(),
        Some(text) => {
            let decoded = decode_uri(text)?;
            if decoded.is_empty() {
                "undefined".to_string()
            } else {
                decoded
            }
        }
    };
    let mut text = format!(
        "错误文件: {source}\n错误信息: {}\n行号: {}\n列号: {}",
        show_part(&report.message),
        show_part(&report.line),
        show_part(&report.column)
    );
    let mut printed = false;
    if let Some(event) = &report.event {
        text.push_str(&format!(
            "\nevent.name: {}\nevent.step: {}",
            event.frame.name, event.frame.step
        ));
        if let Some(parent) = &event.frame.parent {
            text.push_str(&format!(
                "\nevent.parent.name: {}\nevent.parent.step: {}",
                parent.name, parent.step
            ));
            if let Some(grand) = &parent.parent {
                text.push_str(&format!(
                    "\nevent.parent.parent.name: {}\nevent.parent.parent.step: {}",
                    grand.name, grand.step
                ));
            }
        }
        let has_detail = event.player.is_some()
            || event.target.is_some()
            || event.source.is_some()
            || event.skill.is_some()
            || event.card.is_some();
        if has_detail {
            text.push_str("\n-------------");
        }
        if let Some(player) = &event.player {
            if let Some(label) = translated(&report.translate, &player.name) {
                text.push_str(&format!("\nplayer: {label}[{}]", player.name));
                if let Some(distance) = player.distance {
                    text.push_str(&format!("\n座位号: {}", distance + 1));
                }
                printed = true;
            }
        }
        if let Some(actor) = &event.target {
            if let Some(label) = translated(&report.translate, &actor.name) {
                text.push_str(&format!("\ntarget: {label}[{}]", actor.name));
                printed = true;
            }
        }
        if let Some(actor) = &event.source {
            if let Some(label) = translated(&report.translate, &actor.name) {
                text.push_str(&format!("\nsource: {label}[{}]", actor.name));
                printed = true;
            }
        }
        if let Some(skill) = &event.skill {
            if let Some(label) = translated(&report.translate, skill) {
                text.push_str(&format!("\nskill: {label}[{skill}]"));
                printed = true;
            }
        }
        if let Some(card) = &event.card {
            if let Some(label) = translated(&report.translate, card) {
                text.push_str(&format!("\ncard: {label}[{card}]"));
                printed = true;
            }
        }
    }
    if !printed {
        text.push_str("\n-------------");
    }
    if let Some(stack) = report.stack.as_deref().filter(|item| !item.is_empty()) {
        text.push('\n');
        text.push_str(&decode_uri(stack)?);
    }
    Ok(FormattedError {
        text,
        continued: false,
        looped: false,
    })
}

fn join_element(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::Array(items) => items.iter().map(join_element).collect::<Vec<_>>().join(","),
        other => dom_string(other),
    }
}

fn dom_string(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(flag) => if *flag { "true" } else { "false" }.to_string(),
        Value::String(text) => text.clone(),
        Value::Number(number) => {
            if let Some(integer) = number.as_i64() {
                integer.to_string()
            } else if let Some(integer) = number.as_u64() {
                integer.to_string()
            } else if let Some(real) = number.as_f64() {
                if real.is_finite() && real.fract() == 0.0 && real.abs() < 1e15 {
                    format!("{}", real as i64)
                } else {
                    format!("{real}")
                }
            } else {
                "NaN".to_string()
            }
        }
        Value::Array(items) => items.iter().map(join_element).collect::<Vec<_>>().join(","),
        Value::Object(_) => "[object Object]".to_string(),
    }
}

fn require_string_array(value: &Value) -> Result<Vec<String>, SjError> {
    let list = value.as_array().ok_or(SjError::Format)?;
    let mut names = Vec::with_capacity(list.len());
    for item in list {
        names.push(item.as_str().ok_or(SjError::Format)?.to_string());
    }
    Ok(names)
}

fn strings_value(names: &[String]) -> Value {
    Value::Array(names.iter().cloned().map(Value::String).collect())
}

fn noop_import(input: &ImportInput) -> ImportOutput {
    ImportOutput {
        extensions: input.extensions.clone(),
        storage: input.storage.clone(),
        config: input.config.clone(),
        data: input.data.clone(),
        reload_requested: false,
        restarted: false,
        cleared: false,
        applied: false,
    }
}

/// 导入配置。空原文不重启。格式错误不清存储。
pub fn import_config(input: &ImportInput) -> Result<ImportOutput, SjError> {
    if input.payload.is_empty() {
        return Ok(noop_import(input));
    }
    let text = decode_text(&input.payload)?;
    let value: Value = serde_json::from_str(&text).map_err(|_| SjError::Format)?;
    if input.db {
        let obj = value.as_object().ok_or(SjError::Format)?;
        let config_value = obj.get("config").ok_or(SjError::Format)?;
        let data_value = obj.get("data").ok_or(SjError::Format)?;
        let config_obj = config_value.as_object().ok_or(SjError::Format)?;
        let data_obj = data_value.as_object().ok_or(SjError::Format)?;
        let imported = match config_obj.get("extensions") {
            Some(item) => Some(require_string_array(item)?),
            None => None,
        };
        let captured = input.extensions.clone();
        let mut config = input.config.clone();
        for (key, item) in config_obj {
            config.insert(key.clone(), item.clone());
        }
        let mut data = input.data.clone();
        for (key, item) in data_obj {
            data.insert(key.clone(), item.clone());
        }
        let mut extensions = imported.unwrap_or_else(|| captured.clone());
        if input.keep_existing {
            extensions = union_names(&extensions, &captured);
        } else {
            extensions = union_names(&extensions, &[SJ.to_string()]);
        }
        config.insert("extensions".to_string(), strings_value(&extensions));
        return Ok(ImportOutput {
            extensions,
            storage: input.storage.clone(),
            config,
            data,
            reload_requested: true,
            restarted: false,
            cleared: false,
            applied: true,
        });
    }
    if !value.is_object() && !value.is_array() {
        return Err(SjError::Format);
    }
    let mut storage = BTreeMap::new();
    if let Some(saved) = input
        .storage
        .get(INITED_KEY)
        .filter(|item| !item.is_empty())
    {
        storage.insert(INITED_KEY.to_string(), saved.clone());
    }
    let online_key = format!("{}key", input.config_prefix);
    if let Some(saved) = input
        .storage
        .get(&online_key)
        .filter(|item| !item.is_empty())
    {
        storage.insert(online_key, saved.clone());
    }
    match &value {
        Value::Object(map) => {
            for (key, item) in map {
                storage.insert(key.clone(), dom_string(item));
            }
        }
        Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                storage.insert(index.to_string(), dom_string(item));
            }
        }
        _ => return Err(SjError::Format),
    }
    Ok(ImportOutput {
        extensions: input.extensions.clone(),
        storage,
        config: input.config.clone(),
        data: input.data.clone(),
        reload_requested: true,
        restarted: false,
        cleared: true,
        applied: true,
    })
}
