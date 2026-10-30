//! engine-native 的 wasm_bindgen 窄壳（T13 客户端产品面切片）。
//!
//! 设计：**窄 API 而非通用 JSON 协议**——旧线 `engine_wasm` 的投影 schema
//! 与原生不兼容，为避免给原生全类型补 serde，本壳只暴露客户端驱动的最小
//! 面：标准目录建局、窄动作（推进/出牌/选择应答）、座位可见视图与开放
//! 选择的 JSON 投影（手写 `serde_json::json!`，`PlayerView`/`ChoiceSpec`
//! 字段量可控）、`auto_step`（`BoundedPlanner` 规划驱动 + 挂起选择自动
//! 应答，game-server `/native` 同款）、存档十六进制往返（NGSA v65）。
//!
//! 扩展包装载：`new_with_packs_json` / `restore_hex_with_packs_json` 接收
//! 安装载荷数组（manifest + cards/characters/pile，兼容旧 cards.json 元素），
//! 经 C0 装载器桥接后走 `new_with_extensions` /
//! `restore_with_extensions`。空数组等于标准目录。
//!
//! 视图投影为纯只读面，不进权威状态与哈希（ADR 0730）。

use wasm_bindgen::prelude::*;

mod installed_packs;
mod session;
pub use session::{
    assign_general_for, engine_from_archive, engine_from_packs, engine_identity_from_packs,
    general_candidates_json_for, hex_encode, legal_actions_json_for, NativeSession,
    MAX_ARCHIVE_HEX_CHARS,
};
#[cfg(test)]
use session::{hex_decode, hex_decode_bounded, prefix_count};

/// 导入期深层校验（GT-014，裁决第 12 项）：对单个安装载荷复用开局装载的
/// 逐包 Rust 校验器（身份字段、包/模块 logic_hash、rule-vm 装载期 verify、
/// 宿主 ABI 窗口、资产桥接），只校验不建局。Worker `validatePackage` 请求
/// 经此裁决，使本地导入与服务端导入在导入期的拒绝行为一致。
#[wasm_bindgen]
pub fn validate_pack_json(pack_json: &str) -> Result<(), String> {
    installed_packs::validate_pack_json(pack_json)
}

#[cfg(test)]
mod restore_boundary {
    use super::*;

    #[test]
    fn rejects_odd_length_before_slicing() {
        for input in ["0", "abc", "a"] {
            let error = hex_decode(input).expect_err("奇数长度必须拒绝");
            assert!(error.contains("奇数"), "{input}: {error}");
        }
    }

    #[test]
    fn rejects_non_ascii_without_panicking() {
        for input in ["中", "00中0", "0０", "é0"] {
            let error = hex_decode(input).expect_err("非 ASCII 必须拒绝");
            assert!(error.contains("ASCII"), "{input}: {error}");
        }
    }

    #[test]
    fn rejects_illegal_ascii_digits() {
        for input in ["zz", "0g", " 0", "0x"] {
            let error = hex_decode(input).expect_err("非法字符必须拒绝");
            assert!(error.contains("非法十六进制字符"), "{input}: {error}");
        }
    }

    #[test]
    fn rejects_oversize_before_allocating() {
        let error = hex_decode_bounded("0011", 3).expect_err("超限必须拒绝");
        assert!(error.contains("过长"), "{error}");
        assert_eq!(hex_decode_bounded("00ff10", 6).unwrap(), vec![0, 255, 16]);
    }

    #[test]
    fn accepts_upper_and_lower_case_hex() {
        assert_eq!(hex_decode("00aF10").unwrap(), vec![0x00, 0xaf, 0x10]);
        assert_eq!(hex_decode("").unwrap(), Vec::<u8>::new());
    }
}

fn err<E: std::fmt::Debug>(e: E) -> JsValue {
    JsValue::from_str(&format!("{e:?}"))
}

fn js_error(message: String) -> JsValue {
    JsValue::from_str(&message)
}

/// WASM binding for the shared native session; existing JS/Rust signatures remain unchanged.
#[wasm_bindgen]
pub struct NativeEngine {
    inner: NativeSession,
}

#[wasm_bindgen]
impl NativeEngine {
    #[wasm_bindgen(constructor)]
    pub fn new(players: u8, seed: u64) -> Result<NativeEngine, JsValue> {
        NativeSession::new(players, seed)
            .map(|inner| NativeEngine { inner })
            .map_err(js_error)
    }

    pub fn set_planner(&mut self, nodes: usize, deadline_ms: u64) {
        self.inner.set_planner(nodes, deadline_ms)
    }

    pub fn revision(&self) -> u64 {
        self.inner.revision()
    }

    pub fn is_finished(&self) -> bool {
        self.inner.is_finished()
    }

    pub fn general_candidates_json(&self) -> String {
        self.inner.general_candidates_json()
    }

    pub fn assign_general(&mut self, seat: u8, name: &str) -> Result<String, JsValue> {
        self.inner.assign_general(seat, name).map_err(js_error)
    }

    pub fn view_json(&self, seat: u8) -> Result<String, JsValue> {
        self.inner.view_json(seat).map_err(js_error)
    }

    pub fn open_choice_json(&self, seat: u8) -> Result<String, JsValue> {
        self.inner.open_choice_json(seat).map_err(js_error)
    }

    pub fn next_phase(&mut self, actor: u32) -> Result<String, JsValue> {
        self.inner.next_phase(actor).map_err(js_error)
    }

    pub fn play_card(
        &mut self,
        actor: u32,
        card: u32,
        targets: Vec<u32>,
    ) -> Result<String, JsValue> {
        self.inner.play_card(actor, card, targets).map_err(js_error)
    }

    pub fn legal_actions_json(&self, seat: u8) -> Result<String, JsValue> {
        self.inner.legal_actions_json(seat).map_err(js_error)
    }

    pub fn activate_skill(&mut self, actor: u32, skill: u16) -> Result<String, JsValue> {
        self.inner.activate_skill(actor, skill).map_err(js_error)
    }

    pub fn answer(
        &mut self,
        observer: u32,
        choice_id: u64,
        selection: Vec<u32>,
    ) -> Result<String, JsValue> {
        self.inner
            .answer(observer, choice_id, selection)
            .map_err(js_error)
    }

    pub fn auto_step(&mut self) -> Result<String, JsValue> {
        self.inner.auto_step().map_err(js_error)
    }

    pub fn state_hash_hex(&self) -> String {
        self.inner.state_hash_hex()
    }

    pub fn log_len(&self) -> u32 {
        self.inner.log_len()
    }

    pub fn public_event_codes_json(&self, start: u32) -> String {
        self.inner.public_event_codes_json(start)
    }

    pub fn save_hex(&self) -> String {
        self.inner.save_hex()
    }

    pub fn restore_hex(&mut self, hex: &str) -> Result<(), JsValue> {
        self.inner.restore_hex(hex).map_err(js_error)
    }

    pub fn restore_hex_checked(&mut self, hex: &str) -> Result<(), String> {
        self.inner.restore_hex_checked(hex)
    }

    pub fn new_identity(players: u8, seed: u64) -> Result<NativeEngine, JsValue> {
        NativeSession::new_identity(players, seed)
            .map(|inner| NativeEngine { inner })
            .map_err(js_error)
    }

    pub fn new_identity_with_packs_json(
        players: u8,
        seed: u64,
        packs_json: &str,
    ) -> Result<NativeEngine, JsValue> {
        NativeSession::new_identity_with_packs_json(players, seed, packs_json)
            .map(|inner| NativeEngine { inner })
            .map_err(js_error)
    }

    pub fn new_with_packs_json(
        players: u8,
        seed: u64,
        packs_json: &str,
    ) -> Result<NativeEngine, JsValue> {
        NativeSession::new_with_packs_json(players, seed, packs_json)
            .map(|inner| NativeEngine { inner })
            .map_err(js_error)
    }

    pub fn restore_hex_with_packs_json(
        &mut self,
        hex: &str,
        packs_json: &str,
    ) -> Result<(), JsValue> {
        self.inner
            .restore_hex_with_packs_json(hex, packs_json)
            .map_err(js_error)
    }

    pub fn cards_with_prefix(&self, prefix: &str) -> u32 {
        self.inner.cards_with_prefix(prefix)
    }
}

/// L07 目标平台：公开目录查询。不经 wasm_bindgen，避免宿主测试走到 `JsValue`。
///
/// `translations_json` 是译名对象。`hidden_csv` 是逗号分隔的暗牌实例号。
/// 命中暗牌实例号时返回 `error:hidden`，不返回该牌。
fn omnisearch_query(
    cards_json: &str,
    package_id: &str,
    translations_json: &str,
    query: &str,
    hidden_csv: &str,
) -> String {
    use std::collections::BTreeMap;

    let translations: BTreeMap<String, String> = match serde_json::from_str(translations_json) {
        Ok(map) => map,
        Err(err) => return format!("error:bad:{err}"),
    };
    let mut hidden = Vec::new();
    for part in hidden_csv.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        match part.parse::<u32>() {
            Ok(id) => hidden.push(id),
            Err(_) => return format!("error:bad:{part}"),
        }
    }
    let mut index = content_native_loader::PublicIndex::new();
    index.insert_standard(&engine_native::content::standard::standard_catalog());
    if !cards_json.is_empty() {
        if let Err(err) = index.insert_cards_json(package_id, cards_json, &translations) {
            return format!("error:bad:{err:?}");
        }
    }
    match index.search(query, &hidden) {
        Ok(hits) => {
            if hits.is_empty() {
                return "empty".into();
            }
            hits.iter()
                .map(|hit| {
                    let kind = match hit.kind {
                        content_native_loader::EntryKind::General => "general",
                        content_native_loader::EntryKind::Skill => "skill",
                        content_native_loader::EntryKind::Card => "card",
                    };
                    format!(
                        "hit:{}:{kind}:{}:{}",
                        hit.package_id, hit.id, hit.display_name
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        }
        Err(content_native_loader::SearchError::HiddenCard) => "error:hidden".into(),
        Err(content_native_loader::SearchError::EmptyQuery) => "error:empty".into(),
        Err(content_native_loader::SearchError::EvalRefused) => "error:eval".into(),
        Err(content_native_loader::SearchError::BadCatalog(msg)) => format!("error:bad:{msg}"),
    }
}

/// L23 目标平台：应用电脑适配预设并分类一次指针。
///
/// `layout_override` 为空时用配置里的布局。不创建 DOM。
fn desktop_adapt_apply(
    config_json: &str,
    host: &str,
    shousha_ui_enabled: bool,
    layout_override: &str,
    dx: i32,
    dy: i32,
) -> String {
    let config = match content_native_loader::parse_desktop_config(config_json) {
        Ok(config) => config,
        Err(err) => return format!("error:config:{err}"),
    };
    let host = match host {
        "desktop" => content_native_loader::HostKind::Desktop,
        "mobile" => content_native_loader::HostKind::Mobile,
        other => return format!("error:host:{other}"),
    };
    let mut profile = content_native_loader::apply_desktop_adapt(&config, host, shousha_ui_enabled);
    if !layout_override.is_empty() {
        if let Err(err) = content_native_loader::set_layout(&mut profile, layout_override) {
            return format!("error:layout:{err}");
        }
    }
    let class = content_native_loader::classify_pointer(
        &profile,
        dx,
        dy,
        &content_native_loader::PointerContext {
            is_mine: false,
            is_dead: false,
            game_over: false,
            emoji_available: true,
        },
    );
    let pointer = match class {
        content_native_loader::PointerClass::Ignored => "ignored",
        content_native_loader::PointerClass::Tap => "tap",
        content_native_loader::PointerClass::Drag => "drag",
        content_native_loader::PointerClass::NativeTouch => "touch",
    };
    let bridge = match profile.emoji_bridge {
        content_native_loader::EmojiBridgeState::Inactive => "inactive",
        content_native_loader::EmojiBridgeState::Requested => "requested",
    };
    let host_label = match profile.host {
        content_native_loader::HostKind::Desktop => "desktop",
        content_native_loader::HostKind::Mobile => "mobile",
    };
    let layout = match profile.layout {
        content_native_loader::Layout::Mouse => "mouse",
        content_native_loader::Layout::Touch => "touch",
    };
    let presets = profile
        .presets
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join(",");
    format!("profile:android host:{host_label} layout:{layout} bridge:{bridge} pointer:{pointer} presets:{presets}")
}

/// L23 导出：返回净化后的文件名和字节长度，不写磁盘。
fn desktop_adapt_export(text: &str, name: &str) -> String {
    match content_native_loader::export_download(text, name) {
        Ok(download) => format!("file:{}:{}", download.filename, download.bytes.len()),
        Err(content_native_loader::DesktopError::UnsafeName) => "error:unsafe".into(),
        Err(err) => format!("error:{err:?}"),
    }
}

/// U03 目标平台：对已安装的 pile.json 套倍率公式。不生成随机点数。
fn cardpile_scale(pile_json: &str, factors_json: &str, deck_len: u32) -> String {
    let spec = match content_native_loader::bridge_pile(pile_json) {
        Ok(spec) => spec,
        Err(err) => return format!("error:pile:{err:?}"),
    };
    let factors = match content_native_loader::parse_pile_factors(factors_json) {
        Ok(factors) => factors,
        Err(err) => return format!("error:factors:{err}"),
    };
    match engine_native::content::pile::scale_plan(&spec, &factors, deck_len) {
        Ok(engine_native::content::pile::ScaleStatus::Unconfigured) => "unconfigured".into(),
        Ok(engine_native::content::pile::ScaleStatus::NoAddition) => "none".into(),
        Ok(engine_native::content::pile::ScaleStatus::Added(buckets)) => {
            let sum: u32 = buckets.iter().map(|bucket| bucket.count).sum();
            let sha = buckets
                .iter()
                .find(|bucket| bucket.source == "sha" && bucket.suit == "diamond")
                .map(|bucket| bucket.count)
                .unwrap_or(0);
            format!("added:{sum}:sha:diamond:{sha}")
        }
        Err(err) => format!("error:scale:{err:?}"),
    }
}

/// L23 抛表情：拒绝。
fn desktop_adapt_invoke() -> String {
    match content_native_loader::invoke_emoji("player-2") {
        Err(content_native_loader::DesktopError::DomRefused) => "dom_refused".into(),
        Ok(()) => "invoked".into(),
        Err(err) => format!("error:{err:?}"),
    }
}

/// U04 目标平台：跑金币命令。不创建画布，效果一律拒绝。
fn coin_apply(config_json: &str, gate: &str, mode: &str, commands: &str) -> String {
    let play = content_native_loader::CoinGate {
        mode: content_native_loader::CoinMode::Other,
        chess_leader: false,
        video: false,
        connect: false,
        online: false,
        phase_started: false,
    };
    let gate = match gate {
        "play" => play,
        "chess-leader" => content_native_loader::CoinGate {
            mode: content_native_loader::CoinMode::Chess,
            chess_leader: true,
            ..play
        },
        "connect" => content_native_loader::CoinGate {
            connect: true,
            ..play
        },
        "video" => content_native_loader::CoinGate {
            video: true,
            ..play
        },
        other => return format!("error:gate:{other}"),
    };
    let mode = match mode {
        "identity" => content_native_loader::CoinMode::Identity,
        "other" => content_native_loader::CoinMode::Other,
        "chess" => content_native_loader::CoinMode::Chess,
        other => return format!("error:mode:{other}"),
    };
    match content_native_loader::apply_coin_commands(config_json, gate, mode, commands) {
        Ok(text) => format!("{text} effect:refused"),
        Err(err) => format!("error:{err:?}"),
    }
}

/// L22 目标平台：跑段位制裁命令。负数段位表示没有排位函数。不延迟，不说话，不筛将池。
fn elo_apply(config_json: &str, online: bool, mode: &str, level: i32, commands: &str) -> String {
    let level = if level < 0 { None } else { Some(level) };
    match content_native_loader::apply_elo_commands(config_json, online, mode, level, commands) {
        Ok(text) => text,
        Err(err) => format!("error:{err:?}"),
    }
}

/// L02 目标平台：跑 SJ Settings 命令。不下载，不执行脚本，不打开 PDF，不启动 eruda。
fn sj_apply(config_json: &str, commands: &str) -> String {
    match content_native_loader::apply_sj_commands(config_json, commands) {
        Ok(text) => text,
        Err(err) => format!("error:{err:?}"),
    }
}

/// L15 目标平台：跑手杀全扩。不播放奶龙音频，不改界面。
fn shousha_apply(config_json: &str, request_json: &str) -> String {
    match content_native_loader::apply_shousha_line(config_json, request_json) {
        Ok(text) => text,
        Err(err) => format!("error:{err:?}"),
    }
}

/// L13 目标平台：跑手杀MVP 结算统计。不画界面，不播音，不检测作弊输入。
fn shousha_mvp_apply(config_json: &str, request_json: &str) -> String {
    match content_native_loader::apply_mvp_line(config_json, request_json) {
        Ok(text) => text,
        Err(err) => format!("error:{err:?}"),
    }
}

/// L17 目标平台：跑斗转星移。不打开菜单，不播放音频，不重启，不执行文本。
fn douzhuan_apply(config_json: &str, request_json: &str) -> String {
    match content_native_loader::apply_douzhuan_line(config_json, request_json) {
        Ok(text) => text,
        Err(err) => format!("error:{err:?}"),
    }
}

/// L18 目标平台：跑更新保护。不请求网络，不写盘，不执行文本，不睡眠。
fn update_guard_apply(config_json: &str, request_json: &str) -> String {
    match content_native_loader::apply_update_guard_line(config_json, request_json) {
        Ok(text) => text,
        Err(err) => format!("error:{err:?}"),
    }
}

/// L16 目标平台：跑拖拽读取的拖入决策。不 eval、不读写文件、不请求网络、不启 Electron。
fn drag_read_apply(config_json: &str, request_json: &str) -> String {
    match content_native_loader::apply_drag_read_line(config_json, request_json) {
        Ok(text) => text,
        Err(err) => format!("error:{err:?}"),
    }
}

/// U02/L03 目标平台：跑 boss play 包的装配决策。不读写 localStorage、不加载模式。
fn boss_play_apply(config_json: &str, request_json: &str) -> String {
    match content_native_loader::apply_boss_play_line(config_json, request_json) {
        Ok(text) => text,
        Err(err) => format!("error:{err:?}"),
    }
}

/// L26 目标平台：跑笨蛋插件的仍在执行的决策。不画界面，不播放，不睡眠。
fn fool_apply(config_json: &str, request_json: &str) -> String {
    match content_native_loader::apply_fool_line(config_json, request_json) {
        Ok(text) => text,
        Err(err) => format!("error:{err:?}"),
    }
}

/// L21 目标平台：跑武皇争霸的仍在执行的决策。不画界面，不采样，不睡眠。
fn martial_apply(config_json: &str, request_json: &str) -> String {
    match content_native_loader::apply_martial_line(config_json, request_json) {
        Ok(text) => text,
        Err(err) => format!("error:{err:?}"),
    }
}

/// L07 调试入口：不执行脚本。
fn omnisearch_debug(source: &str) -> String {
    match content_native_loader::debug_execute(source) {
        Err(content_native_loader::SearchError::EvalRefused) => "eval_refused".into(),
        Ok(text) => format!("executed:{text}"),
        Err(err) => format!("error:{err:?}"),
    }
}

/// NC08 跨目标门禁用的宿主侧入口（**非 wasm-bindgen 导出**）：让集成测试
/// （`tests/nc08_cross_target.rs`）直接驱动与 WASM 导出相同的实现路径，
/// 不依赖浏览器或 wasm 运行时。
#[doc(hidden)]
pub mod test_support {
    pub use super::{
        assign_general_for, engine_from_archive, engine_from_packs, engine_identity_from_packs,
        general_candidates_json_for, hex_encode,
    };
}

#[cfg(test)]
mod pack_roundtrip {
    use super::*;

    const PACKS: &str = r#"[{"pack":{"name":"t13-pack","version":1,"description":"t13","card_prefix":"t13p_"},"cards":[{"name":"extra","kind":"sha","suit":"spade","rank":7}]}]"#;

    #[test]
    fn installed_pack_enters_new_match_and_restore() {
        let eng = engine_from_packs(4, 17001, PACKS).expect("建局");
        assert_eq!(prefix_count(&eng, "t13p_"), 1, "扩展牌进入新对局目录");
        let hex = hex_encode(&engine_native::archive::encode_archive(&eng.save()));

        match engine_from_archive(&hex, "[]") {
            Err(e) => assert!(!e.is_empty(), "{e}"),
            Ok(_) => panic!("标准目录不得恢复扩展局"),
        }

        let restored = engine_from_archive(&hex, PACKS).expect("同包恢复");
        assert_eq!(prefix_count(&restored, "t13p_"), 1, "恢复后扩展牌仍在");

        let wrong = PACKS.replace("\"version\":1", "\"version\":2");
        match engine_from_archive(&hex, &wrong) {
            Err(e) => assert!(e.contains("恢复失败"), "{e}"),
            Ok(_) => panic!("错版包必须拒绝"),
        }
        match engine_from_archive(&hex, "[]") {
            Err(_) => {}
            Ok(_) => panic!("缺包（空集合）必须拒绝"),
        }
    }

    /// U06 目标平台：真实欢乐 cards.json 经字符串辅助进对局、同包恢复、
    /// 空包与错版拒绝。不调用 wasm_bindgen 方法。
    #[test]
    fn huanle_pack_json_enters_wasm_helper() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../packages/extensions/huanle-upstream-noname/assets/cards.json");
        let raw = std::fs::read_to_string(&path).expect("读欢乐 cards.json");
        assert!(
            raw.contains("\"version\": 1"),
            "仓库源的 pack.version 带空格"
        );
        let packs = format!("[{raw}]");
        let eng = engine_from_packs(3, 17017, &packs).expect("建局");
        assert_eq!(
            prefix_count(&eng, "hnl_"),
            7,
            "七张欢乐牌进入 wasm 辅助建局"
        );
        let hex = hex_encode(&engine_native::archive::encode_archive(&eng.save()));
        match engine_from_archive(&hex, "[]") {
            Err(e) => assert!(e.contains("恢复失败"), "{e}"),
            Ok(_) => panic!("标准目录不得恢复欢乐局"),
        }
        let restored = engine_from_archive(&hex, &packs).expect("同包恢复");
        assert_eq!(prefix_count(&restored, "hnl_"), 7);
        let wrong = raw.replacen("\"version\": 1", "\"version\": 2", 1);
        assert_ne!(wrong, raw);
        match engine_from_archive(&hex, &format!("[{wrong}]")) {
            Err(e) => assert!(e.contains("恢复失败"), "{e}"),
            Ok(_) => panic!("错版欢乐包必须拒绝"),
        }
    }

    /// L06 目标平台：真实五行 cards.json 经字符串辅助进对局。
    #[test]
    fn wuxing_pack_json_enters_wasm_helper() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../packages/extensions/wuxing-upstream-liuli/assets/cards.json");
        let raw = std::fs::read_to_string(&path).expect("读五行 cards.json");
        assert!(raw.contains("\"version\": 1"));
        let packs = format!("[{raw}]");
        let eng = engine_from_packs(3, 17018, &packs).expect("建局");
        assert_eq!(prefix_count(&eng, "wxl_"), 1, "五行盘进入 wasm 辅助建局");
        let hex = hex_encode(&engine_native::archive::encode_archive(&eng.save()));
        match engine_from_archive(&hex, "[]") {
            Err(e) => assert!(e.contains("恢复失败"), "{e}"),
            Ok(_) => panic!("标准目录不得恢复五行局"),
        }
        let restored = engine_from_archive(&hex, &packs).expect("同包恢复");
        assert_eq!(prefix_count(&restored, "wxl_"), 1);
        let wrong = raw.replacen("\"version\": 1", "\"version\": 2", 1);
        match engine_from_archive(&hex, &format!("[{wrong}]")) {
            Err(e) => assert!(e.contains("恢复失败"), "{e}"),
            Ok(_) => panic!("错版五行包必须拒绝"),
        }
    }

    /// L07 目标平台：真实欢乐 cards.json 的公开查询，暗牌号与 eval 都拒绝。
    #[test]
    fn omnisearch_pack_json_enters_wasm_helper() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../packages/extensions/huanle-upstream-noname");
        let cards = std::fs::read_to_string(root.join("assets/cards.json")).expect("cards");
        let npack = std::fs::read_to_string(root.join("package.npack.json")).expect("npack");
        let pkg: serde_json::Value = serde_json::from_str(&npack).unwrap();
        let translations = serde_json::to_string(&pkg["translations"]["zh-CN"]).unwrap();
        let hits = omnisearch_query(&cards, "huanle-cards", &translations, "苦无", "");
        assert!(hits.contains("hit:huanle-cards:card:kuwu:苦无"), "{hits}");
        assert!(!hits.contains("hnl_"), "{hits}");
        let standard = omnisearch_query("", "standard", "{}", "曹操", "");
        assert!(
            standard.contains("hit:standard:general:曹操:曹操"),
            "{standard}"
        );
        assert_eq!(
            omnisearch_query(&cards, "huanle-cards", &translations, "0", "0"),
            "error:hidden"
        );
        assert_eq!(
            omnisearch_query(&cards, "huanle-cards", &translations, "   ", ""),
            "error:empty"
        );
        assert_eq!(
            omnisearch_debug("eval(game.playAudio('card','male','sha'))"),
            "eval_refused"
        );
    }

    /// L23 目标平台：真实 config.json 的预设、点击阈值、导出和 DOM 拒绝。
    #[test]
    fn desktop_adapt_pack_json_enters_wasm_helper() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../packages/extensions/desktop-adapt-upstream-liuli/assets/config.json");
        let config = std::fs::read_to_string(&path).expect("读电脑适配 config.json");
        let tap = desktop_adapt_apply(&config, "desktop", false, "", 3, 4);
        assert!(tap.contains("profile:android"), "{tap}");
        assert!(tap.contains("host:desktop"), "{tap}");
        assert!(tap.contains("layout:mouse"), "{tap}");
        assert!(tap.contains("bridge:inactive"), "{tap}");
        assert!(tap.contains("pointer:tap"), "{tap}");
        assert!(tap.contains("shouShaJieSuanInfo=true"), "{tap}");
        assert!(tap.contains("extension_雷霆万钧_UIpatch=false"), "{tap}");
        let drag = desktop_adapt_apply(&config, "desktop", true, "", 10, 0);
        assert!(drag.contains("bridge:requested"), "{drag}");
        assert!(drag.contains("pointer:drag"), "{drag}");
        let touch = desktop_adapt_apply(&config, "mobile", false, "touch", 0, 0);
        assert!(touch.contains("host:mobile"), "{touch}");
        assert!(touch.contains("pointer:touch"), "{touch}");
        assert_eq!(desktop_adapt_export("hello", "../a:b"), "file:...a.b:5");
        assert_eq!(desktop_adapt_export("x", ".."), "error:unsafe");
        assert_eq!(desktop_adapt_invoke(), "dom_refused");
        assert!(desktop_adapt_apply("{}", "desktop", false, "", 0, 0).starts_with("error:config:"));
        assert!(desktop_adapt_apply(&config, "browser", false, "", 0, 0).starts_with("error:host:"));
    }

    /// U04 目标平台：真实 config.json 的购买、下注门槛和画布拒绝。
    #[test]
    fn coin_pack_json_enters_wasm_helper() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../packages/extensions/coin-upstream-noname/assets/config.json");
        let config = std::fs::read_to_string(&path).expect("读金币 config.json");
        let broke = coin_apply(&config, "play", "other", "buy:blink");
        assert!(broke.starts_with("error:"), "{broke}");
        let funded = config.replace("\"coin\": 0", "\"coin\": 30");
        let bought = coin_apply(&funded, "play", "other", "buy:blink");
        assert!(bought.contains("balance:20"), "{bought}");
        assert!(bought.contains("owned:blink"), "{bought}");
        assert!(bought.contains("running:blink"), "{bought}");
        assert!(bought.contains("effect:refused"), "{bought}");
        let bet = coin_apply(&funded, "play", "other", "bet;bet");
        assert!(bet.starts_with("error:"), "{bet}");
        let leader = coin_apply(&funded, "chess-leader", "chess", "buy:star");
        assert!(leader.starts_with("error:"), "{leader}");
        let settled = coin_apply(&config, "play", "identity", "settle:win:0");
        assert!(settled.contains("balance:36"), "{settled}");
        assert!(coin_apply("{}", "play", "other", "").starts_with("error:"));
        assert!(coin_apply(&config, "browser", "other", "").starts_with("error:gate:"));
        assert!(settled.contains("source:noname"), "{settled}");
    }

    /// L05 目标平台：liuli 仓 config.json。数值与 noname 仓相同，来源字段不同。
    #[test]
    fn liuli_coin_pack_json_enters_wasm_helper() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../packages/extensions/coin-upstream-liuli/assets/config.json");
        let config = std::fs::read_to_string(&path).expect("读 liuli 金币 config.json");
        assert!(config.contains("\"source\": \"noname-liuli\""), "{config}");
        let broke = coin_apply(&config, "play", "other", "buy:blink");
        assert!(broke.starts_with("error:"), "{broke}");
        let funded = config.replace("\"coin\": 0", "\"coin\": 30");
        let bought = coin_apply(&funded, "play", "other", "buy:blink");
        assert!(bought.contains("balance:20"), "{bought}");
        assert!(bought.contains("owned:blink"), "{bought}");
        assert!(bought.contains("source:noname-liuli"), "{bought}");
        assert!(bought.contains("effect:refused"), "{bought}");
        let bet = coin_apply(&funded, "play", "other", "bet;bet");
        assert!(bet.starts_with("error:"), "{bet}");
        let leader = coin_apply(&funded, "chess-leader", "chess", "buy:star");
        assert!(leader.starts_with("error:"), "{leader}");
        let settled = coin_apply(&config, "play", "identity", "settle:win:0");
        assert!(settled.contains("balance:36"), "{settled}");
        assert!(settled.contains("source:noname-liuli"), "{settled}");
        assert!(coin_apply(
            &config.replace("\"noname-liuli\"", "\"\""),
            "play",
            "other",
            ""
        )
        .starts_with("error:"));
    }

    /// L22 目标平台：真实 config.json 的力度、重排、透视和烧条。不睡眠，不说台词。
    #[test]
    fn elo_pack_json_enters_wasm_helper() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../packages/extensions/elo-sanction-upstream-liuli/assets/config.json");
        let config = std::fs::read_to_string(&path).expect("读段位制裁 config.json");
        assert!(config.contains("\"source\": \"noname-liuli\""));
        assert!(config.contains("段位制裁/extension.js"));
        let paiwei = elo_apply(
            &config,
            true,
            "identity",
            -1,
            "diff;order:friend;order:enemy;visible:friend:1;think:0.5;pool",
        );
        assert_eq!(
            paiwei,
            "diff:2 eh:1 top:3 top:2 visible:0 delay:0 chat:refused slept:0 pool:caocao,liubei"
        );
        let offline = elo_apply(&config, false, "identity", 0, "diff;order:friend");
        assert_eq!(offline, "diff:1 eh:0 top:2");
        let taixu = elo_apply(&config, true, "taixuhuanjing", 0, "diff;order:enemy");
        assert_eq!(taixu, "diff:1 eh:0 top:2");
        let level6 = elo_apply(
            &config,
            true,
            "identity",
            6,
            "diff;order:enemy;visible:enemy",
        );
        assert_eq!(level6, "diff:4 eh:-1 top:3 visible:1");
        let eeasy = config.replace("\"paiwei\"", "\"eeasy\"");
        let helped = elo_apply(
            &eeasy,
            false,
            "identity",
            -1,
            "diff;order:friend;visible:friend:9;visible:enemy;visible:me;pool",
        );
        assert_eq!(
            helped,
            "diff:4 eh:1 top:3 visible:1 visible:0 visible:0 pool:caocao,liubei"
        );
        let ehard = config.replace("\"paiwei\"", "\"ehard\"");
        let harsh = elo_apply(
            &ehard,
            false,
            "identity",
            -1,
            "diff;order:friend;order:enemy;think:0.5",
        );
        assert_eq!(
            harsh,
            "diff:4 eh:-1 top:2 top:3 delay:0 chat:refused slept:0"
        );
        let heavy = ehard.replace("\"none\"", "\"heavy\"");
        let thinking = elo_apply(&heavy, true, "identity", -1, "think:0.5");
        assert_eq!(thinking, "delay:2.25 chat:refused slept:0");
        let off_pool = config.replace("\"eloPlayers\": true", "\"eloPlayers\": false");
        assert_eq!(
            elo_apply(&off_pool, true, "identity", -1, "pool"),
            "pool:caocao,liubei"
        );
        assert!(elo_apply(
            &config.replace("\"noname-liuli\"", "\"\""),
            true,
            "identity",
            -1,
            "diff"
        )
        .starts_with("error:"));
        assert!(elo_apply(&config, true, "identity", -1, "think:1.5").starts_with("error:"));
        let veasy = config.replace("\"paiwei\"", "\"veasy\"");
        assert!(elo_apply(&veasy, true, "identity", -1, "visible:friend:2").starts_with("error:"));
    }

    /// L02 目标平台：真实 config.json 的导入、版本比较和外部包。不请求网络。
    #[test]
    fn sj_pack_json_enters_wasm_helper() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../packages/extensions/sj-settings-upstream-liuli/assets/config.json");
        let config = std::fs::read_to_string(&path).expect("读 SJ Settings config.json");
        assert!(config.contains("\"source\": \"noname-liuli\""));
        assert!(config.contains("SJ Settings/extension.js"));
        assert!(config.contains("\"upstream_version\": \"1.4204\""));
        assert!(config.contains("\"openTools\": false"));
        let chain = sj_apply(
            &config,
            "boot;asset;version:1.2:1.10;version:1.10:1.2;version:1:1.0;version:无:1.2;version:1.2:无;version:1.0.0:1.0;version:1.0.0:1.0.1;version::1.2;pdf;console;fault:404;fault:other",
        );
        assert_eq!(
            chain,
            "boot:SJ Settings en:SJ Settings=1 key:absent asset:无 version:false version:true version:equal version:true version:true version:undefined version:true version:true asset:refused console:0 eruda:0 fault:skip fault:retry"
        );
        let pinned = sj_apply(
            &config,
            "boot;import:db:keep:eyJjb25maWciOnsiZXh0ZW5zaW9ucyI6WyLnlLIiLCJTSiBTZXR0aW5ncyJdLCJhc3NldF92ZXJzaW9uIjoi5pegIn0sImRhdGEiOnsibm90ZSI6Iuivl+esuiJ9fQ==",
        );
        assert_eq!(
            pinned,
            "boot:SJ Settings en:SJ Settings=1 key:absent import:ok exts:甲,SJ Settings asset:无 note:诗笺 reload:1 restarted:0"
        );
        assert_eq!(
            sj_apply(&config, "boot:[\"甲\"]"),
            "boot:甲 en:甲=1 key:removed"
        );
        assert_eq!(
            sj_apply(&config, "import:nodb:drop:"),
            "import:noop reload:0 restarted:0"
        );
        assert_eq!(
            sj_apply(&config, "import:db:keep:!!!!"),
            "import:bad reload:0 restarted:0"
        );
        assert_eq!(
            sj_apply(&config, "import:db:keep:bnVsbA=="),
            "import:format reload:0 restarted:0"
        );
        assert_eq!(
            sj_apply(&config, "boot;external:extension:甲;external:package:blob;external:other"),
            "boot:SJ Settings en:SJ Settings=1 key:absent external:extension exts:SJ Settings,甲 reload:1 restarted:0 external:package exts:SJ Settings,甲 inited:blob reload:1 restarted:0 external:other exts:SJ Settings,甲 inited:blob reload:0 restarted:0"
        );
        assert_eq!(
            sj_apply(&config, "eval:update.js;catalog:0;catalog:1;pdf"),
            "eval:refused catalog:offline catalog:refused asset:refused"
        );
        let tools = config.replace("\"openTools\": false", "\"openTools\": true");
        assert_eq!(
            sj_apply(&tools, "console;console"),
            "console:1 eruda:0 console:0 eruda:0"
        );
        assert!(
            sj_apply(&config.replace("\"noname-liuli\"", "\"\""), "boot").starts_with("error:")
        );
        assert!(sj_apply(&config, "boot:null").starts_with("error:"));
        assert!(sj_apply(&config, "onerror:bad").starts_with("error:"));
    }

    /// L15 目标平台：真实 config.json 的改名、将包筛选和禁将。不播放音频。
    #[test]
    fn shousha_pack_json_enters_wasm_helper() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../packages/extensions/shousha-full-upstream-liuli/assets/config.json");
        let config = std::fs::read_to_string(&path).expect("读手杀全扩 config.json");
        assert!(config.contains("\"source\": \"noname-liuli\""));
        assert!(config.contains("手杀全扩/extension.js"));
        assert!(config.contains("8485dbb29ff16f9b258a766b1a17ef75f883aade"));
        assert!(!config.contains("helaAudio"));
        assert!(!config.contains("clsPrefixName"));
        let request = r#"{"mode":"identity","banBt":false,"characters":[{"id":"liubei","gender":"male","pack":"standard","name":"刘备"},{"id":"re_liubei","gender":"male","pack":"standard","name":"手杀刘备"},{"id":"outsider","gender":"male","pack":"diy","name":"路人"},{"id":"sunwukong","gender":"male","pack":"diy","name":"孙悟空"},{"id":"txhj_wolf","gender":"male","pack":"diy","name":"狼"},{"id":"xin","gender":"male","pack":"standard","name":"新杀赵云"},{"id":"ol","gender":"male","pack":"standard","name":"OL刘备"}],"packs":["diy","standard","standard","huicui","tw"],"skills":[{"id":"draw","info":"摸一张牌"},{"id":"ended","info":"已经有。"},{"id":"paren","info":"括号）"},{"id":"foo_map_bar","info":"不加"},{"id":"blank","info":"   "}]}"#;
        let line = shousha_apply(&config, request);
        assert_eq!(
            line,
            "packs:standard,huicui | liubei:刘备:-,re_liubei:界刘备:-,outsider:路人:forbidai,sunwukong:孙悟空:forbidai,txhj_wolf:狼:forbidai+unseen,xin:赵云:forbidai,ol:OL刘备:forbidai | forbid:- | sunben:界孙策 | skills:draw=摸一张牌。;ended=已经有。;paren=括号）;foo_map_bar=不加;blank=    | audio:0 | loop:0"
        );
        let doudizhu = request.replace("\"mode\":\"identity\"", "\"mode\":\"doudizhu\"");
        let exempt = shousha_apply(&config, &doudizhu);
        assert!(exempt.contains("sunwukong:孙悟空:-"), "{exempt}");
        assert!(!exempt.contains("sunwukong:孙悟空:forbidai"), "{exempt}");
        let taixu = request.replace("\"mode\":\"identity\"", "\"mode\":\"taixuhuanjing\"");
        let wolf = shousha_apply(&config, &taixu);
        assert!(wolf.contains("txhj_wolf:狼:forbidai"), "{wolf}");
        assert!(!wolf.contains("txhj_wolf:狼:forbidai+unseen"), "{wolf}");
        let all = config.replace("\"shoushanameX\": \"shousha\"", "\"shoushanameX\": \"all\"");
        let renamed = shousha_apply(&all, request);
        assert!(renamed.contains("ol:刘备:forbidai"), "{renamed}");
        assert!(renamed.contains("re_liubei:界刘备:-"), "{renamed}");
        let pools = config.replace("\"huanyuan\": false", "\"huanyuan\": true");
        let filtered = shousha_apply(
            &pools,
            r#"{"mode":"identity","banBt":false,"characters":[{"id":"liubei","gender":"male","pack":"standard","name":"刘备"},{"id":"guanyu","gender":"male","pack":"standard","name":"关羽"}],"packs":["diy","standard","huicui","tw","sp","refresh","shiji"],"skills":[]}"#,
        );
        assert!(
            filtered.starts_with(
                "packs:standard,sp,refresh,shiji | liubei:刘备:forbidai,guanyu:关羽:- |"
            ),
            "{filtered}"
        );
        let banned = config.replace("\"jinjiang2\": false", "\"jinjiang2\": true");
        let ban_line = shousha_apply(
            &banned,
            r#"{"mode":"identity","banBt":false,"characters":[{"id":"shen_diaochan","gender":"female","pack":"extra","name":"甲"},{"id":"kept_extra","gender":"male","pack":"extra","name":"甲"},{"id":"boss_zhaoyun","gender":"male","pack":"extra","name":"甲"},{"id":"ol_xiaoqiao","gender":"female","pack":"refresh","name":"甲"},{"id":"kept_refresh","gender":"male","pack":"refresh","name":"甲"},{"id":"old_maliang","gender":"male","pack":"sp","name":"甲"},{"id":"kept_sp","gender":"male","pack":"sp","name":"甲"},{"id":"m1","gender":"male","pack":"mobile","name":"甲"},{"id":"outsider","gender":"male","pack":"diy","name":"甲"},{"id":"fanchou","gender":"male","pack":"extra","name":"甲"},{"id":"boss_custom","gender":"male","pack":"diy","name":"甲"}],"packs":[],"packMembers":{"mobile":["m1"],"extra":["shen_diaochan","kept_extra","boss_zhaoyun"],"refresh":["ol_xiaoqiao","kept_refresh"],"sb":["sb1"],"yijiang":["yj1"],"shenhua":["sh1"],"standard":["liubei"],"xinghuoliaoyuan":["xh1"],"sp":["old_maliang","kept_sp"],"jsrg":["js1"],"shiji":["sj1"]},"skills":[]}"#,
        );
        assert!(
            ban_line.contains("forbid:shen_diaochan,ol_xiaoqiao,old_maliang,outsider |"),
            "{ban_line}"
        );
        assert!(ban_line.contains("fanchou:甲:-"), "{ban_line}");
        assert!(ban_line.contains("boss_zhaoyun:甲:"), "{ban_line}");
        assert!(
            !ban_line.contains("forbid:shen_diaochan,ol_xiaoqiao,old_maliang,outsider,fanchou"),
            "{ban_line}"
        );
        let bt = shousha_apply(
            &banned,
            r#"{"mode":"identity","banBt":true,"characters":[{"id":"shen_diaochan","gender":"female","pack":"extra","name":"甲"},{"id":"kept_extra","gender":"male","pack":"extra","name":"甲"},{"id":"kept_refresh","gender":"male","pack":"refresh","name":"甲"},{"id":"shen_zhugeliang","gender":"male","pack":"extra","name":"甲"},{"id":"fanchou","gender":"male","pack":"extra","name":"甲"},{"id":"boss_zhaoyun","gender":"male","pack":"extra","name":"甲"},{"id":"boss_custom","gender":"male","pack":"diy","name":"甲"}],"packs":[],"packMembers":{"extra":["shen_diaochan","kept_extra","boss_zhaoyun"]},"skills":[]}"#,
        );
        assert!(
            bt.contains("forbid:shen_diaochan,kept_refresh,fanchou |"),
            "{bt}"
        );
        assert!(bt.contains("shen_zhugeliang:甲:"), "{bt}");
        let nailong = config.replace("\"nailong\": false", "\"nailong\": true");
        let quiet = shousha_apply(&nailong, request);
        assert!(quiet.ends_with("audio:0 | loop:0"), "{quiet}");
        assert!(shousha_apply(
            &config.replace("\"source\": \"noname-liuli\"", "\"source\": \"\""),
            request
        )
        .starts_with("error:BadConfig"));
        assert!(shousha_apply(
            &banned,
            r#"{"mode":"identity","banBt":false,"characters":[],"packs":[],"packMembers":{"extra":[]},"skills":[]}"#,
        )
        .starts_with("error:MissingPack"));
        assert!(shousha_apply(
            &config,
            r#"{"mode":"identity","banBt":false,"characters":[{"id":"blank","gender":"male","pack":"standard","name":""}],"packs":[],"skills":[]}"#,
        )
        .starts_with("error:BadName"));
    }

    /// L17 目标平台：真实 config.json 的禁将方案、导入和描述。不播放音频，不重启。
    #[test]
    /// L13 目标平台：真实 config.json 的 MVP 计分与结算分组。不画界面。
    #[test]
    fn shousha_mvp_pack_json_enters_wasm_helper() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../packages/extensions/shousha-mvp-upstream-liuli/assets/config.json");
        let config = std::fs::read_to_string(&path).expect("读手杀MVP config.json");
        assert!(config.contains("\"source\": \"noname-liuli\""));
        assert!(config.contains("手杀MVP/extension.js"));
        assert!(config.contains("8485dbb29ff16f9b258a766b1a17ef75f883aade"));
        assert!(!config.contains("playAudio"));
        assert!(!config.contains("innerHTML"));
        assert!(!config.contains("randomName"));
        let request = r#"{"viewer":3,"result":"战斗胜利","mode":"identity","players":[{"id":0,"name":"p0","identity":"zhu","alive":false,"friends":[1],"enemies":[2,3]},{"id":1,"name":"p1","identity":"zhong","alive":true,"friends":[0],"enemies":[2,3]},{"id":2,"name":"p2","identity":"fan","alive":false,"friends":[],"enemies":[0,1]},{"id":3,"name":"p3","identity":"nei","alive":true,"friends":[],"enemies":[2]}],"facts":[{"seq":1,"kind":"useCard","player":0,"damageTag":true},{"seq":2,"kind":"damage","source":0,"target":2,"num":3,"attitudeNegative":true},{"seq":3,"kind":"recover","player":0,"source":1,"num":2,"friendOrSelf":true},{"seq":4,"kind":"die","victim":2,"source":0,"victimEnemy":true}]}"#;
        let line = shousha_mvp_apply(&config, request);
        assert_eq!(
            line,
            "enable:1 fix:1 | facts:4 dup:0 gated:0 | scores:0:14/0/0/0/82/96,1:0/4/0/0/100/104,2:0/0/0/0/82/82,3:0/0/0/0/102/102 | stats:0:3/0/0/1/1,1:0/0/0/0/0,2:0/3/0/0/0,3:0/0/0/0/0 | rank:1,3,0,2 | mvp:1 | win:3 fail:1,0,2 tie:-"
        );
        let gated = config.replace("\"shoushajiesuan\": true", "\"shoushajiesuan\": false");
        let off = shousha_mvp_apply(&gated, request);
        assert!(off.contains("enable:0"), "{off}");
        assert!(off.contains("scores:0:0/0/0/0/100/100"), "{off}");
        assert!(off.ends_with("| win:- fail:- tie:-"), "{off}");
        let unknown = request.replace("\"kind\":\"die\"", "\"kind\":\"wat\"");
        assert!(shousha_mvp_apply(&config, &unknown).starts_with("error:UnknownFact"));
        assert!(shousha_mvp_apply("{", request).starts_with("error:Json"));
    }

    #[test]
    fn douzhuan_pack_json_enters_wasm_helper() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../packages/extensions/douzhuan-xingyi-upstream-liuli/assets/config.json");
        let config = std::fs::read_to_string(&path).expect("读斗转星移 config.json");
        assert!(config.contains("\"source\": \"noname-liuli\""));
        assert!(config.contains("斗转星移/extension.js"));
        assert!(config.contains("8485dbb29ff16f9b258a766b1a17ef75f883aade"));
        assert!(!config.contains("playAudio"));
        assert!(!config.contains("game.reload"));
        assert!(!config.contains("eval("));
        let request = r#"{"mode":"identity","characters":[{"id":"liubei","tags":[]},{"id":"guanyu","tags":["unseen"]},{"id":"hidden","tags":["forbidai"]},{"id":"notag"}],"skills":[{"id":"draw","info":"伤害不大于2且不小于1，阴阳"}]}"#;
        let line = douzhuan_apply(&config, request);
        assert_eq!(
            line,
            "mode:identity | type:forbidai | gate:0 | plan:- | banned:- | lock:guanyu,hidden | now:guanyu,hidden | tags:liubei=-,guanyu=unseen,hidden=forbidai | names:1=方案一,2=方案二,3=方案三 | plans:- | packs:- | sort:- | pick:- | info:draw=伤害不大于2且不小于1，阴阳 | dyn:- | would:0 | reload:0 | audio:0 | eval:0"
        );
        let banned = douzhuan_apply(
            &config,
            r#"{"mode":"identity","network":"online","modePlans":{"identity":1},"importText":"['liubei','caocao']","importIndex":1,"characters":[{"id":"liubei","tags":[]},{"id":"guanyu","tags":["unseen"]},{"id":"caocao"}],"skills":[]}"#,
        );
        assert!(
            banned.contains("gate:1 | plan:1 | banned:liubei,caocao |"),
            "{banned}"
        );
        assert!(
            banned.contains("tags:liubei=forbidai,guanyu=unseen |"),
            "{banned}"
        );
        assert!(banned.contains("plans:1=liubei+caocao |"), "{banned}");
        assert!(banned.ends_with("reload:0 | audio:0 | eval:0"), "{banned}");
        let offline = douzhuan_apply(
            &config,
            r#"{"mode":"identity","network":"offline","modePlans":{"identity":1},"plans":{"1":["liubei"]},"characters":[{"id":"liubei","tags":[]}],"skills":[]}"#,
        );
        assert!(offline.contains("gate:0 |"), "{offline}");
        assert!(offline.contains("tags:liubei=- |"), "{offline}");
        let described = config.replace("\"miaoshuXiuGai\": false", "\"miaoshuXiuGai\": true");
        let rewritten = douzhuan_apply(&described, request);
        assert!(
            rewritten.contains("info:draw=伤害≤2且≥1，阳阴 |"),
            "{rewritten}"
        );
        assert!(
            rewritten.ends_with("reload:0 | audio:0 | eval:0"),
            "{rewritten}"
        );
        let picked = douzhuan_apply(
            &config,
            r#"{"mode":"doudizhu","characters":[],"skills":[],"pick":{"zhu":true,"double":false,"roll":0.9,"candidates":[{"id":"a","rarity":"legend"},{"id":"b","rarity":"legend"}]}}"#,
        );
        assert!(picked.contains("sort:a,b | pick:b |"), "{picked}");
        assert!(douzhuan_apply(
            &config.replace("\"source\": \"noname-liuli\"", "\"source\": \"\""),
            request
        )
        .starts_with("error:BadConfig"));
        assert!(douzhuan_apply(
            &config,
            r#"{"mode":"identity","characters":[],"importText":"alert(1)","importIndex":1}"#
        )
        .starts_with("error:BadPlan"));
    }

    /// U03 目标平台：真实 pile.json 的倍率追加、截断和未配置。
    #[test]
    fn cardpile_pack_json_enters_wasm_helper() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../packages/extensions/cardpile-upstream-noname/assets/pile.json");
        let pile = std::fs::read_to_string(&path).expect("读牌堆");
        let ones = r#"{"guohe":1,"huosha":1,"jiu":1,"leisha":1,"nanman":1,"sha":1,"shan":1,"shunshou":1,"tao":1,"tiesuo":1,"wanjian":1,"wuxie":1}"#;
        assert_eq!(cardpile_scale(&pile, ones, 207), "added:113:sha:diamond:6");
        assert_eq!(
            cardpile_scale(&pile, ones, 600),
            "added:1005:sha:diamond:53"
        );
        assert_eq!(cardpile_scale(&pile, ones, 160), "none");
        assert_eq!(cardpile_scale(&pile, r#"{"sha":1}"#, 207), "unconfigured");
        assert!(cardpile_scale(&pile, r#"{"nope":1}"#, 207).starts_with("error:factors:"));
        assert!(cardpile_scale("{", ones, 207).starts_with("error:pile:"));
    }

    /// L04 目标平台：liuli 仓 pile.json。条目与 noname 仓相同，来源字段不同。
    #[test]
    fn liuli_cardpile_pack_json_enters_wasm_helper() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../packages/extensions/cardpile-upstream-liuli/assets/pile.json");
        let pile = std::fs::read_to_string(&path).expect("读 liuli 牌堆");
        assert!(pile.contains("\"source\": \"noname-liuli\""), "{pile}");
        let ones = r#"{"guohe":1,"huosha":1,"jiu":1,"leisha":1,"nanman":1,"sha":1,"shan":1,"shunshou":1,"tao":1,"tiesuo":1,"wanjian":1,"wuxie":1}"#;
        assert_eq!(cardpile_scale(&pile, ones, 207), "added:113:sha:diamond:6");
        assert_eq!(
            cardpile_scale(&pile, ones, 600),
            "added:1005:sha:diamond:53"
        );
        assert_eq!(cardpile_scale(&pile, ones, 160), "none");
        assert_eq!(cardpile_scale(&pile, r#"{"sha":1}"#, 207), "unconfigured");
        assert!(cardpile_scale(&pile, r#"{"nope":1}"#, 207).starts_with("error:factors:"));
    }

    /// L18 目标平台：真实 config.json。检查按钮不下载，登录错误不执行文本。
    #[test]
    fn update_guard_pack_json_enters_wasm_helper() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../packages/extensions/update-guard-upstream-liuli/assets/config.json");
        let config = std::fs::read_to_string(&path).expect("读更新保护 config.json");
        assert!(config.contains("\"source\": \"noname-liuli\""));
        assert!(config.contains("更新保护/extension.js"));
        assert!(config.contains("8485dbb29ff16f9b258a766b1a17ef75f883aade"));
        assert!(config.contains("\"upstream_version\": \"1.5\""));
        assert!(!config.contains("eval("));
        assert!(!config.contains("game.reload"));
        assert!(!config.contains("fetch("));
        let line = update_guard_apply(&config, "{}");
        assert_eq!(
            line,
            "link:URC | url:https://unitedrhythmized.club/libccy/noname | remove:coding | scan:甲,丙 | size:1.00KB | cmp:prompt | log:甲；丙 | broken:- | would:0 | game:当前版本已是最新 | asset:当前素材已是最新 | asset_flag:1 | updating:0 | select:github | xuanwu:0 | body:login | retry:manual | lock:十周年UI,手杀ui | can:1 | eval:0 | fetch:0 | write:0 | reload:0 | sleep:0 | dom:0"
        );
        let refused = update_guard_apply(&config, r#"{"body":"alert(1)"}"#);
        assert!(refused.contains("body:refused |"), "{refused}");
        assert!(
            refused.contains("eval:0 | fetch:0 | write:0 |"),
            "{refused}"
        );
        let kept = update_guard_apply(&config, r#"{"confirmBroken":true}"#);
        assert!(kept.contains("broken:a,b | would:1 |"), "{kept}");
        assert!(kept.contains("reload:0 |"), "{kept}");
        assert!(update_guard_apply(
            &config.replace("\"source\": \"noname-liuli\"", "\"source\": \"\""),
            "{}"
        )
        .starts_with("error:BadConfig"));
    }

    /// L16 目标平台：真实 config.json。zip 计划、7z 拒绝、原生 play 拒绝、不 eval。
    #[test]
    fn drag_read_pack_json_enters_wasm_helper() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../packages/extensions/drag-read-upstream-liuli/assets/config.json");
        let config = std::fs::read_to_string(&path).expect("读拖拽读取 config.json");
        assert!(config.contains("\"source\": \"noname-liuli\""));
        assert!(config.contains("拖拽读取/extension.js"));
        assert!(config.contains("8485dbb29ff16f9b258a766b1a17ef75f883aade"));
        assert!(config.contains("\"upstream_version\": \"1.81\""));
        assert!(!config.contains("eval("));
        assert!(!config.contains("fetch("));
        let ok = drag_read_apply(
            &config,
            r#"{"name":"我的扩展.zip","mime":"application/zip","files":["我的扩展/extension.js","我的扩展/card/a.json"],"dir":["card",".git","_x","image"]}"#,
        );
        assert!(
            ok.starts_with("kind:zip | default:我的扩展 | prefix:我的扩展/ |"),
            "{ok}"
        );
        assert!(ok.contains("dir:card,image |"), "{ok}");
        assert!(
            ok.ends_with("| err:- | eval:0 | fetch:0 | write:0 | dom:0"),
            "{ok}"
        );
        let seven = drag_read_apply(&config, r#"{"name":"我的扩展.7z","mime":""}"#);
        assert!(seven.starts_with("kind:7z |"), "{seven}");
        assert!(seven.contains("err:还不支持读取7z格式的扩展"), "{seven}");
        let native = drag_read_apply(
            &config,
            r#"{"name":"boss.zip","mime":"application/zip","extname":"boss","files":["boss/extension.js"]}"#,
        );
        assert!(
            native.contains("err:禁止安装游戏原生扩展（boss）"),
            "{native}"
        );
        let single = drag_read_apply(&config, r#"{"name":"extension.js","mime":""}"#);
        assert!(single.starts_with("kind:extension.js |"), "{single}");
        assert!(
            single.contains("err:extension.js代码有错误，无法导入！"),
            "{single}"
        );
        assert!(drag_read_apply(&config, "not json").starts_with("error:BadConfig"));
        assert!(drag_read_apply(
            &config.replace("\"source\": \"noname-liuli\"", "\"source\": \"\""),
            "{}"
        )
        .starts_with("error:BadConfig"));
    }

    /// U02/L03 目标平台：两仓 boss play 包的真实 config.json。装配/归一/门控。
    #[test]
    fn boss_play_pack_json_enters_wasm_helper() {
        let req = r#"{"mode":"prototype","enableai":true,"skills":["a","versus_ladder","b"],"storage":{"boss":{"甲":{"hp":4,"isBossAllowed":true},"乙":{"hp":"3"}},"versus":{"丙":{"hp":"3/4","tags":["bossallowed"]}},"translate":{"x":"y"}}}"#;
        for (dir, source) in [
            ("boss-play-upstream-noname", "noname"),
            ("boss-play-upstream-liuli", "noname-liuli"),
        ] {
            let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
                "../../packages/extensions/{dir}/assets/config.json"
            ));
            let config = std::fs::read_to_string(&path).expect("读 boss play config.json");
            assert!(config.contains(&format!("\"source\": \"{source}\"")));
            assert!(config.contains("挑战武将") && config.contains("剑阁武将"));
            assert!(!config.contains("localStorage"));
            assert!(!config.contains("eval("));
            let line = boss_play_apply(&config, req);
            assert!(
                line.starts_with("skipped:0 | boss:2 | jiange:1 | chars:3 | forbidai:0 |"),
                "{line}"
            );
            assert!(line.contains("allowed_jiange:1 | skills:2 |"), "{line}");
            let tafang = boss_play_apply(&config, r#"{"mode":"tafang"}"#);
            assert!(
                tafang.starts_with("skipped:1 | boss:0 | jiange:0 |"),
                "{tafang}"
            );
            assert!(boss_play_apply(&config, "not json").starts_with("error:BadConfig"));
        }
    }

    /// L26 目标平台：真实 config.json。不画界面，不播放，不睡眠。
    #[test]
    fn fool_pack_json_enters_wasm_helper() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../packages/extensions/fool-plugin-upstream-liuli/assets/config.json");
        let config = std::fs::read_to_string(&path).expect("读笨蛋插件 config.json");
        assert!(config.contains("\"source\": \"noname-liuli\""));
        assert!(config.contains("笨蛋插件/extension.js"));
        assert!(config.contains("8485dbb29ff16f9b258a766b1a17ef75f883aade"));
        assert!(config.contains("\"upstream_version\": \"2.0.2\""));
        assert!(!config.contains("eval("));
        assert!(!config.contains("game.reload"));
        assert!(!config.contains("fetch("));
        let line = fool_apply(&config, "{}");
        assert_eq!(
            line,
            "zy:1 | zy_over:0 | ms:1 | ft_hand:1 | ft_fang:0 | ft_next:1 | ft_over:0 | ft_sel:2 | ft_need:3 | hint:强中加伤 | wrap:1 | allowed:1 | phone:1 | peiyin:- | deriv:- | throw:1 | move:0.7 | rot:1 | land:1 | sampled:0 | video:1 | voice:rende | file:skill/rende1.mp3 | play:0 | cardopt:0 | custom:0 | reload:0 | eval:0 | fetch:0 | write:0 | sleep:0 | dom:0 | audio:0"
        );
        let quiet = fool_apply(
            &config,
            r#"{"selfTarget":true,"ua":"Mozilla/5.0 (Windows NT 10.0)"}"#,
        );
        assert!(quiet.contains("throw:0 | move:- | rot:0 |"), "{quiet}");
        assert!(quiet.contains("phone:0 |"), "{quiet}");
        assert!(quiet.contains("play:0 |"), "{quiet}");
        assert!(quiet.contains("sleep:0 | dom:0 | audio:0"), "{quiet}");
        let closed = fool_apply(
            &config.replace(
                "\"Fool_liegong\": \"shousha\"",
                "\"Fool_liegong\": \"close\"",
            ),
            "{}",
        );
        assert!(closed.contains("hint:- | wrap:0 |"), "{closed}");
        let calc_off = fool_apply(
            &config.replace("\"Fool_skill\": true", "\"Fool_skill\": false"),
            "{}",
        );
        assert!(calc_off.contains("zy:- | zy_over:- |"), "{calc_off}");
        assert!(fool_apply(
            &config.replace("\"source\": \"noname-liuli\"", "\"source\": \"\""),
            "{}"
        )
        .starts_with("error:BadConfig"));
    }

    /// L21 目标平台：真实 config.json。不画界面，不采样，不睡眠。
    #[test]
    fn martial_pack_json_enters_wasm_helper() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../packages/extensions/martial-upstream-liuli/assets/config.json");
        let config = std::fs::read_to_string(&path).expect("读武皇争霸 config.json");
        assert!(config.contains("\"source\": \"noname-liuli\""));
        assert!(config.contains("武皇争霸/extension.js"));
        assert!(config.contains("8485dbb29ff16f9b258a766b1a17ef75f883aade"));
        assert!(config.contains("\"upstream_version\": \"1.3\""));
        assert!(!config.contains("eval("));
        assert!(!config.contains("game.reload"));
        assert!(!config.contains("fetch("));
        let line = martial_apply(&config, "{}");
        assert_eq!(
            line,
            "registered:1 | mode:normal | connect:0 | double:0 | hpRule:hejiansan | hpUi:0 | change:0 | free:0 | show:5 | battle:5 | hide:2 | width:5 | num:5 | hiddenN:4 | size:20 | ratio:1.2 | first:0 | side:3 | zhuSeat:1 | fanSeat:0 | zhuSkill:0 | hidden:a,b,c,d | friend:a,b,c,d,e | enemy:j,i,h,g,f | zhu:j,i,h,g,f | fan:a,b,c,d,e | openZhu:j | openFan:a | drawZhu:4 | drawFan:5 | join:- | hp:0 | foe:0 | script:extension/武皇/mightyFoe.js | scriptRan:0 | online:0 | playback:0 | coeff:- | sampled:0 | dom:0 | sleep:0 | eval:0 | fetch:0 | write:0 | reload:0 | dialog:0"
        );
        let second = martial_apply(&config, r#"{"first":1}"#);
        assert!(
            second.contains("first:1 | side:1 | zhuSeat:0 | fanSeat:1"),
            "{second}"
        );
        assert!(
            second.contains("friend:a,b,c,e,f | enemy:j,i,h,d,g"),
            "{second}"
        );
        assert!(second.contains("zhu:a,b,c,e,f | fan:j,i,h,d,g"), "{second}");
        assert!(second.contains("sampled:0 | dom:0 | sleep:0"), "{second}");
        let joined = martial_apply(
            &config.replace("\"double_character\": false", "\"double_character\": true"),
            r#"{"join":true}"#,
        );
        assert!(joined.contains("join:1 | hp:0 |"), "{joined}");
        assert!(joined.contains("openZhu:j/i | openFan:a/b"), "{joined}");
        assert!(joined.contains("hpUi:1"), "{joined}");
        let off = martial_apply(
            &config.replace("\"enable\": true", "\"enable\": false"),
            "{}",
        );
        assert!(off.starts_with("registered:0 |"), "{off}");
        assert!(off.contains("scriptRan:0"), "{off}");
        assert!(martial_apply(
            &config.replace("\"source\": \"noname-liuli\"", "\"source\": \"\""),
            "{}"
        )
        .starts_with("error:BadConfig"));
    }
}

/// 终局胜负 → 协议形态（NC09/NC12）。**纯函数**，便于在宿主侧单测：
/// `{LoyalistWin:null}` / `{RebelWin:null}` / `{RenegadeWin:null}` /
/// `{SingleWin:{player:n}}`（**1 基座位**）/ `{GroupWin:{group:..}}`；
/// `None` → `null`；**未建模的模式胜者**（对决/斗地主/Boss）→ 字符串 `"finished"`（不猜测语义）。
pub fn winner_to_protocol_json(outcome: &Option<engine_native::Winner>) -> serde_json::Value {
    match outcome {
        None => serde_json::Value::Null,
        Some(engine_native::Winner::LordSide) => serde_json::json!({ "LoyalistWin": null }),
        Some(engine_native::Winner::Rebels) => serde_json::json!({ "RebelWin": null }),
        Some(engine_native::Winner::Traitor) => serde_json::json!({ "RenegadeWin": null }),
        Some(engine_native::Winner::Player(p)) => {
            serde_json::json!({ "SingleWin": { "player": *p as u32 + 1 } })
        }
        Some(engine_native::Winner::Faction(f)) => {
            serde_json::json!({ "GroupWin": { "group": format!("{f:?}").to_lowercase() } })
        }
        Some(_) => serde_json::json!("finished"),
    }
}

#[cfg(test)]
mod winner_protocol_tests {
    use super::winner_to_protocol_json;
    use engine_native::Winner;

    #[test]
    fn identity_outcomes_map_to_protocol_shapes() {
        assert_eq!(
            winner_to_protocol_json(&Some(Winner::LordSide)),
            serde_json::json!({"LoyalistWin": null})
        );
        assert_eq!(
            winner_to_protocol_json(&Some(Winner::Rebels)),
            serde_json::json!({"RebelWin": null})
        );
        assert_eq!(
            winner_to_protocol_json(&Some(Winner::Traitor)),
            serde_json::json!({"RenegadeWin": null})
        );
    }

    #[test]
    fn single_and_group_and_none_and_unmodelled() {
        // 1 基座位：内部 3 → 对外 4
        assert_eq!(
            winner_to_protocol_json(&Some(Winner::Player(3))),
            serde_json::json!({"SingleWin": {"player": 4}})
        );
        let g =
            winner_to_protocol_json(&Some(Winner::Faction(engine_native::catalog::Faction::Shu)));
        assert!(g.get("GroupWin").is_some(), "势力胜应映射为 GroupWin：{g}");
        assert_eq!(winner_to_protocol_json(&None), serde_json::Value::Null);
        // 未建模（对决/斗地主/Boss）→ 保持 "finished"
        assert_eq!(
            winner_to_protocol_json(&Some(Winner::Versus(0))),
            serde_json::json!("finished")
        );
        assert_eq!(
            winner_to_protocol_json(&Some(Winner::Doudizhu { lord_won: true })),
            serde_json::json!("finished")
        );
    }
}
