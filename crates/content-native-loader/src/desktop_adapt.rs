//! L23 电脑适配：桌面宿主上的手机预设、导出描述和鼠标点击判定。
//!
//! 上游在 `precontent` 里把 `lib.device` 写成 `android`，并替换 `game.export`
//! 为浏览器下载；在 `content` 里写入两条配置，且仅当手杀 UI 开启时，用
//! 鼠标按下/抬起的位移是否小于 10 像素来决定是否抛表情。
//!
//! 这里保留这四件事的数据结果。不移动窗口，不创建 DOM，不注入 CSS，
//! 也不调用 `game.use_throw_emoji`。导出只返回净化后的文件名和字节。

use std::collections::BTreeMap;

use serde_json::{Map, Value};

/// 上游强制写入的设备预设。只有 `android`。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceProfile {
    /// 对应 `lib.device = "android"`。
    Android,
}

/// 调用方的真实宿主。预设不改写这个值。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostKind {
    /// 桌面。
    Desktop,
    /// 手机。
    Mobile,
}

/// 布局。鼠标走点击阈值；触摸不改写成点击。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    /// 鼠标。
    Mouse,
    /// 触摸。
    Touch,
}

/// 表情桥接是否被请求。请求不等于已经挂上 DOM。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmojiBridgeState {
    /// 手杀 UI 未开启，或配置未要求桥接。
    Inactive,
    /// 已请求。实际监听与抛表情仍拒绝。
    Requested,
}

/// 指针分类。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerClass {
    /// 鼠标桥接条件不成立。
    Ignored,
    /// 位移小于阈值，视为点击。
    Tap,
    /// 位移达到阈值，视为拖动。
    Drag,
    /// 触摸布局，不套用鼠标阈值。
    NativeTouch,
}

/// 与上游 `mousedown` 里相同的四个否决条件。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PointerContext {
    /// `game.isMine(player)`。
    pub is_mine: bool,
    /// 角色已阵亡。
    pub is_dead: bool,
    /// 对局已结束。
    pub game_over: bool,
    /// `game.use_throw_emoji` 存在。
    pub emoji_available: bool,
}

/// 已解析的包配置。数值与上游写死的常量一致。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopConfig {
    /// 设备预设。
    pub device_profile: DeviceProfile,
    /// 初始布局。
    pub layout: Layout,
    /// 点击阈值，单位像素。上游为 10。
    pub tap_threshold_px: u32,
    /// 去掉手杀结算提示。
    pub hide_settlement_hint: bool,
    /// 雷霆万钧 UI 补丁保持关闭。
    pub thunder_ui_patch: bool,
    /// 是否在手杀 UI 开启时请求表情桥。
    pub emoji_bridge: bool,
}

/// 应用到某个宿主之后的预设。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppliedProfile {
    /// 设备预设。
    pub device_profile: DeviceProfile,
    /// 真实宿主。
    pub host: HostKind,
    /// 当前布局。
    pub layout: Layout,
    /// 点击阈值。
    pub tap_threshold_px: u32,
    /// 两条配置的最终值。键顺序固定。
    pub presets: BTreeMap<String, bool>,
    /// 表情桥状态。
    pub emoji_bridge: EmojiBridgeState,
}

/// 导出结果。只含文件名和正文，不写磁盘。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Download {
    /// 净化后的文件名。
    pub filename: String,
    /// UTF-8 正文。
    pub bytes: Vec<u8>,
}

/// 不能完成的桌面适配动作。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DesktopError {
    /// 配置或快照无法解析。
    BadConfig(String),
    /// 要求创建 DOM 或调用抛表情。
    DomRefused,
    /// 导出名净化后是 `.` 或 `..`。
    UnsafeName,
}

const SETTLEMENT_KEY: &str = "shouShaJieSuanInfo";
const THUNDER_KEY: &str = "extension_雷霆万钧_UIpatch";

/// 解析包内 `assets/config.json`。
pub fn parse_config(json: &str) -> Result<DesktopConfig, String> {
    let value: Value = serde_json::from_str(json).map_err(|err| format!("config: {err}"))?;
    let obj = value
        .as_object()
        .ok_or_else(|| "config: not an object".to_string())?;
    let mut keys: Vec<&str> = obj.keys().map(String::as_str).collect();
    keys.sort_unstable();
    let expected = [
        "device_profile",
        "emoji_bridge",
        "layout",
        "presets",
        "tap_threshold_px",
    ];
    if keys != expected {
        return Err(format!("unknown config keys {keys:?}"));
    }
    let device = obj
        .get("device_profile")
        .and_then(Value::as_str)
        .ok_or_else(|| "missing device_profile".to_string())?;
    if device != "android" {
        return Err(format!("unknown device_profile {device}"));
    }
    let layout = parse_layout(
        obj.get("layout")
            .and_then(Value::as_str)
            .ok_or_else(|| "missing layout".to_string())?,
    )?;
    let threshold = obj
        .get("tap_threshold_px")
        .and_then(Value::as_u64)
        .ok_or_else(|| "missing tap_threshold_px".to_string())?;
    if threshold != 10 {
        return Err(format!("tap_threshold_px must be 10, got {threshold}"));
    }
    let presets = obj
        .get("presets")
        .and_then(Value::as_object)
        .ok_or_else(|| "missing presets".to_string())?;
    let mut preset_keys: Vec<&str> = presets.keys().map(String::as_str).collect();
    preset_keys.sort_unstable();
    if preset_keys != [THUNDER_KEY, SETTLEMENT_KEY] {
        return Err(format!("unknown preset keys {preset_keys:?}"));
    }
    let hide = presets
        .get(SETTLEMENT_KEY)
        .and_then(Value::as_bool)
        .ok_or_else(|| format!("missing {SETTLEMENT_KEY}"))?;
    if !hide {
        return Err(format!("preset {SETTLEMENT_KEY} must be true"));
    }
    let thunder = presets
        .get(THUNDER_KEY)
        .and_then(Value::as_bool)
        .ok_or_else(|| format!("missing {THUNDER_KEY}"))?;
    if thunder {
        return Err(format!("preset {THUNDER_KEY} must be false"));
    }
    let bridge = obj
        .get("emoji_bridge")
        .and_then(Value::as_str)
        .ok_or_else(|| "missing emoji_bridge".to_string())?;
    if bridge != "shousha_ui" {
        return Err(format!("unknown emoji_bridge {bridge}"));
    }
    Ok(DesktopConfig {
        device_profile: DeviceProfile::Android,
        layout,
        tap_threshold_px: 10,
        hide_settlement_hint: true,
        thunder_ui_patch: false,
        emoji_bridge: true,
    })
}

/// 把预设应用到宿主。不读取扩展商店，也不改内容锁。
pub fn apply(config: &DesktopConfig, host: HostKind, shousha_ui_enabled: bool) -> AppliedProfile {
    let mut presets = BTreeMap::new();
    presets.insert(SETTLEMENT_KEY.to_string(), config.hide_settlement_hint);
    presets.insert(THUNDER_KEY.to_string(), config.thunder_ui_patch);
    let emoji_bridge = if config.emoji_bridge && shousha_ui_enabled {
        EmojiBridgeState::Requested
    } else {
        EmojiBridgeState::Inactive
    };
    AppliedProfile {
        device_profile: config.device_profile,
        host,
        layout: config.layout,
        tap_threshold_px: config.tap_threshold_px,
        presets,
        emoji_bridge,
    }
}

/// 在鼠标与触摸之间切换。
pub fn set_layout(profile: &mut AppliedProfile, layout: &str) -> Result<(), String> {
    profile.layout = parse_layout(layout)?;
    Ok(())
}

/// 鼠标位移分类。触摸布局直接返回 [`PointerClass::NativeTouch`]。
pub fn classify_pointer(
    profile: &AppliedProfile,
    dx: i32,
    dy: i32,
    ctx: &PointerContext,
) -> PointerClass {
    if profile.layout == Layout::Touch {
        return PointerClass::NativeTouch;
    }
    if !pointer_allowed(ctx) {
        return PointerClass::Ignored;
    }
    let distance = (f64::from(dx)).hypot(f64::from(dy));
    if distance < f64::from(profile.tap_threshold_px) {
        PointerClass::Tap
    } else {
        PointerClass::Drag
    }
}

/// 上游否决：自己、已阵亡、终局，或没有抛表情函数。
pub fn pointer_allowed(ctx: &PointerContext) -> bool {
    !ctx.is_mine && !ctx.is_dead && !ctx.game_over && ctx.emoji_available
}

/// 导出正文。非法文件名字符换成 `.`；结果为 `.` 或 `..` 时拒绝。
///
/// 不创建下载链接，也不写文件。
pub fn export_download(text: &str, name: &str) -> Result<Download, DesktopError> {
    let raw = if name.is_empty() { "noname" } else { name };
    let filename: String = raw
        .chars()
        .map(|ch| match ch {
            '\\' | '/' | ':' | '?' | '"' | '*' | '<' | '>' | '|' => '.',
            other => other,
        })
        .collect();
    if filename.is_empty() || filename == "." || filename == ".." {
        return Err(DesktopError::UnsafeName);
    }
    Ok(Download {
        filename,
        bytes: text.as_bytes().to_vec(),
    })
}

/// 抛表情和 DOM 监听没有迁入。
pub fn invoke_emoji(_player_id: &str) -> Result<(), DesktopError> {
    Err(DesktopError::DomRefused)
}

/// 把当前预设写成可恢复的 JSON。
pub fn snapshot(profile: &AppliedProfile) -> String {
    let mut presets = Map::new();
    for (key, value) in &profile.presets {
        presets.insert(key.clone(), Value::Bool(*value));
    }
    let value = serde_json::json!({
        "device_profile": "android",
        "host": host_label(profile.host),
        "layout": layout_label(profile.layout),
        "tap_threshold_px": profile.tap_threshold_px,
        "presets": Value::Object(presets),
        "emoji_bridge": match profile.emoji_bridge {
            EmojiBridgeState::Inactive => "inactive",
            EmojiBridgeState::Requested => "requested",
        },
    });
    value.to_string()
}

/// 从 [`snapshot`] 恢复。未知枚举拒绝。
pub fn restore_snapshot(json: &str) -> Result<AppliedProfile, DesktopError> {
    let value: Value = serde_json::from_str(json)
        .map_err(|err| DesktopError::BadConfig(format!("snapshot: {err}")))?;
    let obj = value
        .as_object()
        .ok_or_else(|| DesktopError::BadConfig("snapshot: not an object".into()))?;
    let device = obj
        .get("device_profile")
        .and_then(Value::as_str)
        .ok_or_else(|| DesktopError::BadConfig("missing device_profile".into()))?;
    if device != "android" {
        return Err(DesktopError::BadConfig(format!(
            "unknown device_profile {device}"
        )));
    }
    let host = match obj.get("host").and_then(Value::as_str) {
        Some("desktop") => HostKind::Desktop,
        Some("mobile") => HostKind::Mobile,
        Some(other) => return Err(DesktopError::BadConfig(format!("unknown host {other}"))),
        None => return Err(DesktopError::BadConfig("missing host".into())),
    };
    let layout = parse_layout(
        obj.get("layout")
            .and_then(Value::as_str)
            .ok_or_else(|| DesktopError::BadConfig("missing layout".into()))?,
    )
    .map_err(DesktopError::BadConfig)?;
    let threshold = obj
        .get("tap_threshold_px")
        .and_then(Value::as_u64)
        .ok_or_else(|| DesktopError::BadConfig("missing tap_threshold_px".into()))?;
    if threshold != 10 {
        return Err(DesktopError::BadConfig(format!(
            "tap_threshold_px must be 10, got {threshold}"
        )));
    }
    let presets_value = obj
        .get("presets")
        .and_then(Value::as_object)
        .ok_or_else(|| DesktopError::BadConfig("missing presets".into()))?;
    let mut presets = BTreeMap::new();
    for (key, value) in presets_value {
        let flag = value
            .as_bool()
            .ok_or_else(|| DesktopError::BadConfig(format!("preset {key} is not bool")))?;
        presets.insert(key.clone(), flag);
    }
    if presets.get(SETTLEMENT_KEY) != Some(&true)
        || presets.get(THUNDER_KEY) != Some(&false)
        || presets.len() != 2
    {
        return Err(DesktopError::BadConfig("preset values drifted".into()));
    }
    let emoji_bridge = match obj.get("emoji_bridge").and_then(Value::as_str) {
        Some("inactive") => EmojiBridgeState::Inactive,
        Some("requested") => EmojiBridgeState::Requested,
        Some(other) => {
            return Err(DesktopError::BadConfig(format!(
                "unknown emoji_bridge {other}"
            )))
        }
        None => return Err(DesktopError::BadConfig("missing emoji_bridge".into())),
    };
    Ok(AppliedProfile {
        device_profile: DeviceProfile::Android,
        host,
        layout,
        tap_threshold_px: 10,
        presets,
        emoji_bridge,
    })
}

fn parse_layout(layout: &str) -> Result<Layout, String> {
    match layout {
        "mouse" => Ok(Layout::Mouse),
        "touch" => Ok(Layout::Touch),
        other => Err(format!("unknown layout {other}")),
    }
}

fn layout_label(layout: Layout) -> &'static str {
    match layout {
        Layout::Mouse => "mouse",
        Layout::Touch => "touch",
    }
}

fn host_label(host: HostKind) -> &'static str {
    match host {
        HostKind::Desktop => "desktop",
        HostKind::Mobile => "mobile",
    }
}
