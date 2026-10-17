//! L02 SJ Settings：外部扩展名单、配置导入、版本比较和下载筛选。
//!
//! 启动名单、`lib.init.decode`、数据库与非数据库导入、`compareVersion`、
//! 目录标记、勾选确认、文件失败分类和 `window.onerror` 文本按 liuli 的
//! `extension/SJ Settings/extension.js` 计算。Cordova 复制、FileTransfer、
//! 权限、`fetch`、`eval`、eruda、PDF、进度条、`alert` 和真正的
//! `game.reload` / `game.loop` 不执行。

use std::collections::BTreeMap;

use serde_json::{Map, Value};

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

fn b64_val(byte: u8) -> Option<u8> {
    match byte {
        b'A'..=b'Z' => Some(byte - b'A'),
        b'a'..=b'z' => Some(byte - b'a' + 26),
        b'0'..=b'9' => Some(byte - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

fn base64_encode(data: &[u8]) -> String {
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    let mut index = 0;
    while index + 3 <= data.len() {
        let packed =
            ((data[index] as u32) << 16) | ((data[index + 1] as u32) << 8) | data[index + 2] as u32;
        out.push(TABLE[((packed >> 18) & 63) as usize] as char);
        out.push(TABLE[((packed >> 12) & 63) as usize] as char);
        out.push(TABLE[((packed >> 6) & 63) as usize] as char);
        out.push(TABLE[(packed & 63) as usize] as char);
        index += 3;
    }
    if index < data.len() {
        let rest = data.len() - index;
        let second = if rest == 2 { data[index + 1] as u32 } else { 0 };
        let packed = ((data[index] as u32) << 16) | (second << 8);
        out.push(TABLE[((packed >> 18) & 63) as usize] as char);
        out.push(TABLE[((packed >> 12) & 63) as usize] as char);
        if rest == 2 {
            out.push(TABLE[((packed >> 6) & 63) as usize] as char);
            out.push('=');
        } else {
            out.push('=');
            out.push('=');
        }
    }
    out
}

fn base64_decode(text: &str) -> Result<Vec<u8>, SjError> {
    if text.is_empty() || text.len() % 4 != 0 {
        return Err(SjError::BadEncoding("base64 length".into()));
    }
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        let chunk = &bytes[index..index + 4];
        let pad = chunk.iter().rev().take_while(|byte| **byte == b'=').count();
        if pad > 2 || (pad > 0 && index + 4 != bytes.len()) {
            return Err(SjError::BadEncoding("base64 pad".into()));
        }
        let mut vals = [0u8; 4];
        for (slot, byte) in chunk.iter().enumerate() {
            if *byte == b'=' {
                if slot < 4 - pad {
                    return Err(SjError::BadEncoding("base64 pad".into()));
                }
            } else if slot >= 4 - pad {
                return Err(SjError::BadEncoding("base64 pad".into()));
            } else {
                vals[slot] =
                    b64_val(*byte).ok_or_else(|| SjError::BadEncoding("base64 alphabet".into()))?;
            }
        }
        let packed = ((vals[0] as u32) << 18)
            | ((vals[1] as u32) << 12)
            | ((vals[2] as u32) << 6)
            | vals[3] as u32;
        out.push((packed >> 16) as u8);
        if pad < 2 {
            out.push((packed >> 8) as u8);
        }
        if pad < 1 {
            out.push(packed as u8);
        }
        index += 4;
    }
    Ok(out)
}

/// `lib.init.encode`。BMP 字符与上游的 UTF-8 展开一致。非 BMP 拒绝。
pub fn encode_text(text: &str) -> Result<String, SjError> {
    if text.chars().any(|ch| (ch as u32) > 0xFFFF) {
        return Err(SjError::BadEncoding("non-bmp".into()));
    }
    Ok(base64_encode(text.as_bytes()))
}

/// `lib.init.decode`。非法 Base64 或非法 UTF-8 是导入错误。
pub fn decode_text(text: &str) -> Result<String, SjError> {
    let bytes = base64_decode(text)?;
    String::from_utf8(bytes).map_err(|_| SjError::BadEncoding("utf8".into()))
}

fn hex_val(ch: char) -> Option<u8> {
    match ch {
        '0'..='9' => Some((ch as u8) - b'0'),
        'a'..='f' => Some((ch as u8) - b'a' + 10),
        'A'..='F' => Some((ch as u8) - b'A' + 10),
        _ => None,
    }
}

fn is_reserved(ch: char) -> bool {
    matches!(
        ch,
        ';' | '/' | '?' | ':' | '@' | '&' | '=' | '+' | '$' | ',' | '#'
    )
}

fn utf8_width(lead: u8) -> Result<usize, SjError> {
    let mut width = 0;
    let mut bits = lead;
    while bits & 0x80 != 0 {
        width += 1;
        bits = bits.wrapping_shl(1);
        if width > 4 {
            break;
        }
    }
    if width < 2 || width > 4 {
        Err(SjError::BadEncoding("uri".into()))
    } else {
        Ok(width)
    }
}

/// `decodeURI`。保留字符的百分号序列原样留下。非法序列返回错误。
pub fn decode_uri(text: &str) -> Result<String, SjError> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::new();
    let mut index = 0;
    while index < chars.len() {
        if chars[index] != '%' {
            out.push(chars[index]);
            index += 1;
            continue;
        }
        if index + 2 >= chars.len() {
            return Err(SjError::BadEncoding("uri".into()));
        }
        let lead = (hex_val(chars[index + 1]).ok_or_else(|| SjError::BadEncoding("uri".into()))?
            << 4)
            | hex_val(chars[index + 2]).ok_or_else(|| SjError::BadEncoding("uri".into()))?;
        if lead & 0x80 == 0 {
            let ch = char::from(lead);
            if is_reserved(ch) {
                out.push('%');
                out.push(chars[index + 1]);
                out.push(chars[index + 2]);
            } else {
                out.push(ch);
            }
            index += 3;
            continue;
        }
        let width = utf8_width(lead)?;
        let mut octets = vec![lead];
        let mut cursor = index + 3;
        for _ in 1..width {
            if cursor >= chars.len() || chars[cursor] != '%' || cursor + 2 >= chars.len() {
                return Err(SjError::BadEncoding("uri".into()));
            }
            let next = (hex_val(chars[cursor + 1])
                .ok_or_else(|| SjError::BadEncoding("uri".into()))?
                << 4)
                | hex_val(chars[cursor + 2]).ok_or_else(|| SjError::BadEncoding("uri".into()))?;
            if next & 0xC0 != 0x80 {
                return Err(SjError::BadEncoding("uri".into()));
            }
            octets.push(next);
            cursor += 3;
        }
        let decoded = String::from_utf8(octets).map_err(|_| SjError::BadEncoding("uri".into()))?;
        out.push_str(&decoded);
        index = cursor;
    }
    Ok(out)
}

fn js_number(raw: &str) -> Option<f64> {
    let text = raw.trim();
    if text.is_empty() {
        return Some(0.0);
    }
    if text == "Infinity" || text == "+Infinity" {
        return Some(f64::INFINITY);
    }
    if text == "-Infinity" {
        return Some(f64::NEG_INFINITY);
    }
    let (sign, hex) = if let Some(rest) = text
        .strip_prefix("+0x")
        .or_else(|| text.strip_prefix("+0X"))
    {
        (1.0, Some(rest))
    } else if let Some(rest) = text
        .strip_prefix("-0x")
        .or_else(|| text.strip_prefix("-0X"))
    {
        (-1.0, Some(rest))
    } else if let Some(rest) = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
        (1.0, Some(rest))
    } else {
        (1.0, None)
    };
    if let Some(rest) = hex {
        if rest.is_empty() || !rest.chars().all(|ch| ch.is_ascii_hexdigit()) {
            return None;
        }
        return u64::from_str_radix(rest, 16)
            .ok()
            .map(|value| sign * value as f64);
    }
    let mut body = text.to_string();
    if let Some(stripped) = body.strip_prefix('+') {
        body = stripped.to_string();
    }
    if body.ends_with('.') {
        body.push('0');
    }
    if body.starts_with('.') {
        body.insert(0, '0');
    }
    if body.contains("+.0") || body.contains("-.0") {
        return None;
    }
    let numeric = body
        .chars()
        .all(|ch| ch.is_ascii_digit() || matches!(ch, '.' | '-' | '+' | 'e' | 'E'))
        && body.matches('.').count() <= 1
        && body.matches('-').count() + body.matches('+').count() <= 2;
    if !numeric {
        return None;
    }
    body.parse::<f64>()
        .ok()
        .filter(|value| value.is_finite() || value.is_infinite())
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

fn join_names(names: &[String]) -> String {
    if names.is_empty() {
        "-".to_string()
    } else {
        names.join(",")
    }
}

fn storage_token(storage: &BTreeMap<String, String>) -> String {
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

struct Session {
    storage: BTreeMap<String, String>,
    extensions: Vec<String>,
    enables: BTreeMap<String, bool>,
    asset_version: Option<String>,
    config_map: BTreeMap<String, Value>,
    data: BTreeMap<String, Value>,
    console_open: bool,
    environment: bool,
}

fn dash_option(text: &str) -> Option<&str> {
    if text == "-" {
        None
    } else {
        Some(text)
    }
}

fn plain_error() -> WindowError {
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
