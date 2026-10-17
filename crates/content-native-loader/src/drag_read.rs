//! L16 拖拽读取（noname-liuli）的可移植决策。
//!
//! 上游把扩展包（zip）或离线包/素材包拖入 Electron 窗口后解压到扩展目录。
//! 7z 扩展包路径在上游**直接抛错**（`还不支持读取7z格式的扩展`），单文件
//! `extension.js` 路径用 `eval` 取扩展名——本模块**不 eval**，改为从包清单
//! （文件名列表）识别，且单文件路径不执行（登记差异）。其余为纯字符串/清单
//! 逻辑：拖入类型判定、默认扩展名、`extension.js` 存在性与公共前缀校验、
//! 原生 play 名单拒绝、一键完整包目录布局与 `.`/`_` 前缀过滤。
//!
//! 本模块不读写文件、不请求网络、不 eval、不启动 Electron。

use serde_json::Value;

pub const SEVENZ_UNSUPPORTED: &str = "还不支持读取7z格式的扩展。\r\n导入7z格式的扩展请参考万能导入法\r\n然后选择素材包，解压缩到指定扩展目录";
pub const ERR_NO_EXTENSION_JS: &str = "没有extension.js，无法导入";
pub const ERR_NATIVE_PLAY: &str = "禁止安装游戏原生扩展";
pub const ERR_NOT_EXTENSION: &str = "你导入的不是扩展！请选择正确的文件";
pub const ERR_BAD_EXTENSION_JS: &str = "extension.js代码有错误，无法导入！";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DragError {
    BadConfig(String),
    NoExtensionJs,
    NativePlay(String),
    NotExtension,
    BadExtensionJs,
    SevenZUnsupported,
}

impl DragError {
    pub fn message(&self) -> String {
        match self {
            DragError::BadConfig(field) => format!("配置字段非法：{field}"),
            DragError::NoExtensionJs => ERR_NO_EXTENSION_JS.into(),
            DragError::NativePlay(name) => format!("{ERR_NATIVE_PLAY}（{name}）"),
            DragError::NotExtension => ERR_NOT_EXTENSION.into(),
            DragError::BadExtensionJs => ERR_BAD_EXTENSION_JS.into(),
            DragError::SevenZUnsupported => SEVENZ_UNSUPPORTED.into(),
        }
    }
}

/// 拖入文件类型（对齐上游分支顺序）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropKind {
    /// zip 扩展包：走解压导入（需要 `extension.js`）。
    Zip,
    /// 7z（或 mime 含 zip 但文件名不以 `.zip` 结尾）：扩展包导入路径直接抛错。
    SevenZ,
    /// 单个 `extension.js`：上游 eval 取扩展名——本模块不 eval（登记差异）。
    ExtensionJs,
    /// 其它：上游 `return false` 忽略。
    Ignored,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DragConfig {
    pub source: String,
    pub upstream_path: String,
    pub upstream_commit: String,
    pub upstream_version: String,
    pub intro: String,
    pub author: String,
    /// 原生 play 包名单（`lib.config.all.plays`）——拖入这些名字直接拒绝。
    pub native_plays: Vec<String>,
    /// 一键完整包的顶层目录。
    pub complete_folders: Vec<String>,
    /// 一键完整包的顶层文件。
    pub complete_files: Vec<String>,
    pub audio_subfolders: Vec<String>,
    pub image_subfolders: Vec<String>,
    pub extension_subfolders: Vec<String>,
    pub node_modules_subfolders: Vec<String>,
}

/// 拖入类型判定（上游 `files[0].type.indexOf("zip") == -1 && !name.endsWith('.7z')`
/// 的反向分支 + `(name.endsWith('.zip') ? nextDo : nextDo_7z)(1)`）。
pub fn detect_drop(name: &str, mime: &str) -> DropKind {
    let lower = name.to_ascii_lowercase();
    if mime.contains("zip") || lower.ends_with(".7z") {
        if lower.ends_with(".zip") {
            DropKind::Zip
        } else {
            DropKind::SevenZ
        }
    } else if name == "extension.js" {
        DropKind::ExtensionJs
    } else {
        DropKind::Ignored
    }
}

/// 压缩包名 → 默认扩展名（上游 `name.replace(/\.(zip|7z)$/i, '')`）。
pub fn default_extension_name(archive_name: &str) -> String {
    let lower = archive_name.to_ascii_lowercase();
    for suffix in [".zip", ".7z"] {
        if lower.ends_with(suffix) {
            return archive_name[..archive_name.len() - suffix.len()].to_string();
        }
    }
    archive_name.to_string()
}

/// 目录项是否可见（上游 `getFileListSync`：`i[0] != '.' && i[0] != '_'`）。
pub fn dir_entry_visible(name: &str) -> bool {
    !name.starts_with('.') && !name.starts_with('_')
}

/// 目录项过滤（保序）。
pub fn list_dir(names: &[&str]) -> Vec<String> {
    names
        .iter()
        .filter(|n| dir_entry_visible(n))
        .map(|n| (*n).to_string())
        .collect()
}

/// `extension.js` 候选（上游 `zip3.file(/\bextension.js$/)`）。
fn is_extension_js(path: &str) -> bool {
    path == "extension.js" || path.ends_with("/extension.js")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportPlan {
    pub extension_name: String,
    pub default_name: String,
    pub prefix: String,
    pub candidates: Vec<String>,
}

/// 解压导入计划（上游 `nextDo` index==1 的校验链；`extname` 为用户在弹窗中
/// 输入的名字——纯函数不弹窗，由调用方给出）。
///
/// 校验顺序：`extension.js` 存在 → 原生 play 名单 → 公共前缀。
pub fn plan_archive_import(
    files: &[&str],
    archive_name: &str,
    extname: &str,
    native_plays: &[String],
) -> Result<ImportPlan, DragError> {
    let candidates: Vec<String> = files
        .iter()
        .filter(|f| is_extension_js(f))
        .map(|f| (*f).to_string())
        .collect();
    if candidates.is_empty() {
        return Err(DragError::NoExtensionJs);
    }
    if native_plays.iter().any(|p| p == extname) {
        return Err(DragError::NativePlay(extname.to_string()));
    }
    // 取路径最短的候选，前缀 = 其目录（含尾 `/`）。
    let shortest = candidates
        .iter()
        .min_by_key(|c| c.len())
        .expect("candidates non-empty");
    let prefix = match shortest.rfind('/') {
        Some(i) => shortest[..=i].to_string(),
        None => String::new(),
    };
    if !candidates.iter().all(|c| c.starts_with(&prefix)) {
        return Err(DragError::NotExtension);
    }
    Ok(ImportPlan {
        extension_name: extname.to_string(),
        default_name: default_extension_name(archive_name),
        prefix,
        candidates,
    })
}

/// 单文件 `extension.js` 导入：上游 eval 取 `name`——本模块不 eval，
/// 返回登记差异错误（须改走带清单的包路径）。
pub fn plan_single_extension_js(_file_name: &str) -> Result<ImportPlan, DragError> {
    Err(DragError::BadExtensionJs)
}

/// 一键完整包的文件/目录布局（上游 `generateZip` 的固定清单，保序）。
pub fn complete_package_tree(config: &DragConfig) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for f in &config.complete_files {
        out.push(f.clone());
    }
    for d in &config.complete_folders {
        out.push(format!("{d}/"));
    }
    out.push("cache/".into());
    out.push("audio/".into());
    for s in &config.audio_subfolders {
        out.push(format!("audio/{s}/"));
    }
    out.push("image/".into());
    for s in &config.image_subfolders {
        out.push(format!("image/{s}/"));
    }
    out.push("extension/".into());
    for s in &config.extension_subfolders {
        out.push(format!("extension/{s}/"));
    }
    out.push("node_modules/".into());
    for s in &config.node_modules_subfolders {
        out.push(format!("node_modules/{s}/"));
    }
    out
}

pub fn parse_drag_config(text: &str) -> Result<DragConfig, DragError> {
    let value: Value =
        serde_json::from_str(text).map_err(|_| DragError::BadConfig("json".into()))?;
    let obj = value
        .as_object()
        .ok_or_else(|| DragError::BadConfig("json".into()))?;
    let source = req_str(obj, "source")?;
    if source.is_empty() {
        return Err(DragError::BadConfig("source".into()));
    }
    Ok(DragConfig {
        source: source.to_string(),
        upstream_path: req_str(obj, "upstream_path")?.to_string(),
        upstream_commit: req_str(obj, "upstream_commit")?.to_string(),
        upstream_version: req_str(obj, "upstream_version")?.to_string(),
        intro: req_str(obj, "intro")?.to_string(),
        author: req_str(obj, "author")?.to_string(),
        native_plays: req_str_list(obj, "native_plays")?,
        complete_folders: req_str_list(obj, "complete_folders")?,
        complete_files: req_str_list(obj, "complete_files")?,
        audio_subfolders: req_str_list(obj, "audio_subfolders")?,
        image_subfolders: req_str_list(obj, "image_subfolders")?,
        extension_subfolders: req_str_list(obj, "extension_subfolders")?,
        node_modules_subfolders: req_str_list(obj, "node_modules_subfolders")?,
    })
}

fn req_str<'a>(
    obj: &'a serde_json::Map<String, Value>,
    key: &'static str,
) -> Result<&'a str, DragError> {
    match obj.get(key) {
        Some(Value::String(text)) => Ok(text),
        _ => Err(DragError::BadConfig(key.into())),
    }
}

fn req_str_list(
    obj: &serde_json::Map<String, Value>,
    key: &'static str,
) -> Result<Vec<String>, DragError> {
    match obj.get(key) {
        Some(Value::Array(items)) => items
            .iter()
            .map(|item| {
                item.as_str()
                    .map(str::to_string)
                    .ok_or_else(|| DragError::BadConfig(key.into()))
            })
            .collect::<Result<Vec<_>, _>>(),
        _ => Err(DragError::BadConfig(key.into())),
    }
}

/// 目标平台字符串辅助（engine-native-wasm）：把一次拖入决策压成一行，
/// 供 wasm 侧回归。请求字段：`name`/`mime`/`extname`/`files`/`dir`。
/// 输出永不包含 `eval:1`/`fetch:1`/`write:1`/`dom:1`——本模块不执行这些。
pub fn apply_drag_read_line(config_json: &str, request_json: &str) -> Result<String, DragError> {
    let config = parse_drag_config(config_json)?;
    let value: Value =
        serde_json::from_str(request_json).map_err(|_| DragError::BadConfig("request".into()))?;
    let obj = value
        .as_object()
        .ok_or_else(|| DragError::BadConfig("request".into()))?;
    let name = obj.get("name").and_then(Value::as_str).unwrap_or("");
    let mime = obj.get("mime").and_then(Value::as_str).unwrap_or("");
    let extname = obj
        .get("extname")
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| default_extension_name(name));
    let files: Vec<String> = match obj.get("files") {
        None => Vec::new(),
        Some(Value::Array(items)) => items
            .iter()
            .map(|i| {
                i.as_str()
                    .map(str::to_string)
                    .ok_or_else(|| DragError::BadConfig("files".into()))
            })
            .collect::<Result<Vec<_>, _>>()?,
        Some(_) => return Err(DragError::BadConfig("files".into())),
    };
    let dir: Vec<String> = match obj.get("dir") {
        None => Vec::new(),
        Some(Value::Array(items)) => items
            .iter()
            .map(|i| {
                i.as_str()
                    .map(str::to_string)
                    .ok_or_else(|| DragError::BadConfig("dir".into()))
            })
            .collect::<Result<Vec<_>, _>>()?,
        Some(_) => return Err(DragError::BadConfig("dir".into())),
    };

    let kind = detect_drop(name, mime);
    let kind_str = match kind {
        DropKind::Zip => "zip",
        DropKind::SevenZ => "7z",
        DropKind::ExtensionJs => "extension.js",
        DropKind::Ignored => "ignored",
    };
    let files_refs: Vec<&str> = files.iter().map(String::as_str).collect();
    let (default, prefix, err) = match kind {
        DropKind::Zip => match plan_archive_import(&files_refs, name, &extname, &config.native_plays)
        {
            Ok(plan) => (plan.default_name, plan.prefix, "-".to_string()),
            Err(e) => (default_extension_name(name), String::new(), e.message()),
        },
        DropKind::SevenZ => (
            default_extension_name(name),
            String::new(),
            SEVENZ_UNSUPPORTED.to_string(),
        ),
        DropKind::ExtensionJs => (extname, String::new(), ERR_BAD_EXTENSION_JS.to_string()),
        DropKind::Ignored => (extname, String::new(), "-".to_string()),
    };
    let dir_refs: Vec<&str> = dir.iter().map(String::as_str).collect();
    let kept = list_dir(&dir_refs).join(",");
    let kept = if kept.is_empty() { "-".into() } else { kept };
    Ok(format!(
        "kind:{kind_str} | default:{default} | prefix:{prefix} | tree:{} | dir:{kept} | err:{err} | eval:0 | fetch:0 | write:0 | dom:0",
        complete_package_tree(&config).len()
    ))
}
