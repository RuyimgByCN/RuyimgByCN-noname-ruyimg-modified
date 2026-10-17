//! C0 生产侧装载器（ADR 0777、ADR 0778）：已安装扩展包的 `cards.json` →
//! engine-native `ExtensionPack` 的唯一生产实现（演进提案 S11/S12/C0）。
//!
//! 包的全部差异写在包数据里：`pack` 元数据（名称、内容版本、描述、牌名
//! 前缀、开局钩子）、每张牌的 `kind`/`suit`/`rank` 与可选 `range`/
//! `equip_skills`、`skills` 与武将技能的 `mapping`、cardpile 的
//! `assets/pile.json` 比例表。牌种键、宿主行为键与牌种伴随技能由
//! engine-native 的全局表解析（`CardKind::from_key`、
//! `HostBehavior::from_key`、`card_kind_host_skills`），装载器不含逐包代码。
//! 坏包一律返回 [`BridgeError`]，不静默取默认值。名称与名单经
//! `engine_native::content::names` 符号表分配，重复装载/启停不泄漏增长；
//! 总量上限超限拒装（`BridgeError::SymbolTableLimit`，ADR 0779）。
//!
//! 择址说明（ADR 0777）：装载器需要 engine-native 的目录类型与
//! serde_json 解析；engine-native 运行期只依赖 blake3（不引入 serde），
//! content-runtime 也不能反向依赖 engine-native，故落在独立宿主层 crate，
//! 供测试、客户端与服务端共用。

use std::collections::{BTreeMap, BTreeSet};
use std::ops::RangeInclusive;

use engine_native::catalog::{CardDef, CardKind, EquipSlot, Faction, Suit};
use engine_native::content::extension::{ExtensionDef, ExtensionGeneral, ExtensionPack, InitHook};
use engine_native::content::host::{
    card_kind_equip_skills, card_kind_host_skills, HostBehavior, HostSkillDecl,
};
use engine_native::content::names;
use engine_native::content::pile::{PileEntry, PileSpec};
use engine_native::content::standard::Gender;
use serde_json::Value;

mod coin;
mod boss_playpack;
mod desktop_adapt;
mod douzhuan;
mod drag_read;
mod elo;
mod fool;
mod martial;
mod omnisearch;
mod shousha;
mod shousha_mvp;
mod sj_settings;
mod update_guard;
pub use coin::{
    apply_coin_commands, buy_item, change_coin, mark_phase_started, open_coin_session,
    parse_coin_config, persist_config, place_bet, restore_coin, settle_for, snapshot_coin,
    start_item, stop_item, toggle_snow_fall, toggle_snow_size, CoinConfig, CoinDisplay, CoinError,
    CoinGate, CoinMode, CoinSession, IdentityRole, SettleOutcome, SettleRequest, ShopItem,
    SnowSize,
};
pub use boss_playpack::{
    apply_boss_play_line, exclude_versus_ladder, filter_allowed, init_outcome, merge_translate,
    normalize_boss_hp, normalize_jiange_hp, parse_boss_play_config, parse_storage, BossPlayConfig,
    Entry, HpKind, InitOutcome, PlayError, PlayStorage,
};
pub use desktop_adapt::{
    apply as apply_desktop_adapt, classify_pointer, export_download, invoke_emoji,
    parse_config as parse_desktop_config, pointer_allowed, restore_snapshot, set_layout, snapshot,
    AppliedProfile, DesktopConfig, DesktopError, DeviceProfile, Download, EmojiBridgeState,
    HostKind, Layout, PointerClass, PointerContext,
};
pub use douzhuan::{
    apply_douzhuan, apply_douzhuan_line, export_plan, format_report as format_douzhuan_report,
    parse_douzhuan_config, parse_plan_text, rank_score, request_from_json, rewrite_skill_text,
    CandidateIn, CharacterIn as DouzhuanCharacterIn, CharacterOut as DouzhuanCharacterOut,
    DouzhuanConfig, DouzhuanError, DouzhuanReport, DouzhuanRequest, PackView, PickIn,
    SkillIn as DouzhuanSkillIn, SkillOut as DouzhuanSkillOut,
};
pub use drag_read::{
    apply_drag_read_line, complete_package_tree, default_extension_name, detect_drop,
    dir_entry_visible, list_dir, parse_drag_config, plan_archive_import, plan_single_extension_js,
    DragConfig, DragError, DropKind, ImportPlan, ERR_BAD_EXTENSION_JS, ERR_NATIVE_PLAY,
    ERR_NOT_EXTENSION, ERR_NO_EXTENSION_JS, SEVENZ_UNSUPPORTED,
};
pub use elo::{
    apply_elo_commands, hand_visible, mark_inserted, mark_opening, parse_elo_config, reorder_draw,
    resolve_difficulty, restore_pile, restrict_pool, snapshot_pile, think_delay, wash, EloCard,
    EloConfig, EloError, EloLevel, EloQuery, EloViewer, Think,
};
pub use fool::{
    apply_fool_line, choose_card_runs, choose_voice, custom_choose_file_runs, derivation_sample,
    fangtong_allows, fangtong_select_len, fix_peiyin, is_phone, landing_offset, minsi_allows,
    parse_fool_config, peiyin_sample, rewrite_derivation, save_liegong, sha_filter_wrapped,
    sha_hint, sha_target_allowed, throw_emotion, ziyuan_allows, AudioSpec, DerivCharacter,
    DerivSkill, FangCard, FoolConfig, FoolError, HintQuery, LiegongSave, SkillAudio, ThrowOut,
    VoiceClip, VoiceOut,
};
pub use martial::{
    ai_order, apply_martial_line, ask_join, assign_seats, attitude, battle_lists, check_result,
    connect_enabled, consider_mighty, double_hp_visible, draft, enemy_battle, filter_pool,
    format_record, free_replace, has_zhu_skill, layout_numbers, mighty_script, on_general_dead,
    online_choose_runs, opening_draw, parse_martial_config, playback_runs, redraw_pool,
    replacement_draw, reselect_pool, take_unique, video_name, DeathQuery, DraftInput, DraftOutput,
    FreeOut, JoinOut, Layout as MartialLayout, MartialConfig, MartialError, MightyOut, MightyPair,
    OverOut,
    RecordBook, ReplaceOut, Seats,
};
pub use omnisearch::{
    debug_execute, debug_inspect, parse_config, EntryKind, PublicEntry, PublicIndex, SearchConfig,
    SearchError,
};
pub use shousha_mvp::{
    apply_mvp_line, parse_mvp_config, parse_mvp_request, MvpConfig, MvpError, MvpFact,
    MvpPlayer, MvpScore, MvpSession, MvpSettlement, MvpStats,
};
pub use shousha::{
    apply_shousha, apply_shousha_line, format_report, parse_shousha_config, CharacterIn,
    CharacterOut, ShoushaConfig, ShoushaError, ShoushaReport, ShoushaRequest, SkillIn, SkillOut,
};
pub use sj_settings::{
    apply_external, apply_sj_commands, boot, catalog_mark, choose_download, classify_fault,
    compare_version, decode_text, decode_uri, encode_text, eval_update_script, filter_downloads,
    finish_download, format_window_error, import_config, load_remote_catalog, open_asset_file,
    parse_sj_config, plan_files, request_console, Actor, BootInput, BootOutput, CatalogEntry,
    CatalogMark, ConsoleRequest, ExternalEffect, ExternalKind, FaultAction, FilePlan, Finish,
    FormattedError, Frame, GameEvent, HttpCode, ImportInput, ImportOutput, InstalledPack,
    SelectInput, SjConfig, SjError, TransferFault, VersionOrder, WindowError,
};
pub use update_guard::{
    apply_update_guard_line, boot_link, can_update, classify_body, compare_update, dedup_broken,
    extension_urls, forced_enables, format_catch, join_changelog, next_error_count, on_remove_link,
    parse_size, parse_update_guard_config, precontent_menu, press_asset_button, press_game_button,
    retry_updates, scan_folders, select_extension_link, select_general_link, settle_broken,
    Attempt, BodyClass, BootLink, BrokenOut, ButtonOut, CatchKind, FolderIn, GuardConfig,
    GuardError, MenuFlags, RetryClass, ScanOut, SelectOutcome, UpdatePrompt, UpdateUrl, CODING_URL,
    FASTGIT_URL, GITHUB_URL, URC_URL,
};

/// 装载错误：坏包、未知键与宿主 API 版本不符给出明确原因。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BridgeError {
    /// JSON 解析失败。
    BadJson(String),
    /// 缺少必需字段。
    MissingField(&'static str),
    /// 字段类型或取值非法。
    InvalidField { field: &'static str, reason: String },
    /// `kind` 不是 engine-native 的牌种键。
    UnknownCardKind(String),
    /// 未知花色。
    UnknownSuit(String),
    /// `mapping` 不是 engine-native 的宿主行为键。
    UnknownBehaviorKey(String),
    /// 未知势力。
    UnknownFaction(String),
    /// 未知性别。
    UnknownGender(String),
    /// 未知开局钩子键。
    UnknownInitHook(String),
    /// 装备技既不是本包 `skills` 映射，也不是牌种伴随技能。
    UnknownEquipSkill(String),
    /// 包内宿主技能重名。
    DuplicateSkill(String),
    /// cards.json 声明的引擎 API 版本与当前运行时不符。
    HostVersionMismatch { declared: u64, current: u32 },
    /// 名称符号表已达总量上限（ADR 0779）：拒绝装载而非无界增长。
    SymbolTableLimit { limit: usize },
}

impl std::fmt::Display for BridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BridgeError::BadJson(e) => write!(f, "包数据解析失败: {e}"),
            BridgeError::MissingField(fld) => write!(f, "包数据缺少字段 {fld}"),
            BridgeError::InvalidField { field, reason } => {
                write!(f, "包数据字段 {field} 非法: {reason}")
            }
            BridgeError::UnknownCardKind(k) => write!(f, "未知牌种键 {k}"),
            BridgeError::UnknownSuit(s) => write!(f, "未知花色 {s}"),
            BridgeError::UnknownBehaviorKey(k) => write!(f, "未知宿主行为键 {k}"),
            BridgeError::UnknownFaction(s) => write!(f, "未知势力 {s}"),
            BridgeError::UnknownGender(s) => write!(f, "未知性别 {s}"),
            BridgeError::UnknownInitHook(s) => write!(f, "未知开局钩子 {s}"),
            BridgeError::UnknownEquipSkill(s) => write!(f, "装备技 {s} 未在本包登记"),
            BridgeError::DuplicateSkill(s) => write!(f, "包内宿主技能重名 {s}"),
            BridgeError::HostVersionMismatch { declared, current } => write!(
                f,
                "包声明的引擎 API 版本 {declared} 与运行时 {current} 不符"
            ),
            BridgeError::SymbolTableLimit { limit } => {
                write!(f, "名称符号表已达总量上限 {limit}，拒绝装载（ADR 0779）")
            }
        }
    }
}

impl std::error::Error for BridgeError {}

/// 装载产物：扩展包 + 未映射（暂缓）牌清单。
pub struct BridgedPack {
    pub pack: ExtensionPack,
    /// cards.json `deferred` 数组的牌名（登记暂缓，不阻断装载）。
    pub deferred: Vec<String>,
}

/// 武将技能装载产物。
pub struct BridgedGenerals {
    pub generals: Vec<ExtensionGeneral>,
    /// 命中宿主行为的技能声明（按名去重，并入 ExtensionPack 用）。
    pub host_skills: Vec<HostSkillDecl>,
    /// 命中宿主行为的技能键（逐武将出现次序；内容锁/断言用）。
    pub mapped_skill_keys: Vec<&'static str>,
    /// 未映射（暂缓）的技能计数。
    pub unmapped_skills: usize,
}

fn invalid(field: &'static str, reason: impl Into<String>) -> BridgeError {
    BridgeError::InvalidField {
        field,
        reason: reason.into(),
    }
}

/// 符号表总量上限门（ADR 0779）：三个生产入口（bridge_pack/
/// bridge_generals/bridge_pile）装载前检查，超限一律拒装。表项不回收，
/// 唯一名称数是单调量——入口检查即足以在重复安装/损坏包流下 fail-fast；
/// 单个包自身可一次 intern 逼近上限（受包文件大小界定，登记为已知边界）。
/// T8 增量 3/6：npack 内 rule-vm 触发模块 → engine-native `VmSkillDecl`
///（数据驱动装载：DSL→编译 npack→装载器→引擎，不重编译）。事件名严
/// 格映射（phase_end/phase_begin/damage/card_used/death）；未知事件名
/// 拒装。filter 走 `map_vm_filter`：精确策略、参数化族、具名编译；
/// 无法编译的名称跳过入 `deferred_filters`（不恒真）。
pub struct VmBridgeOutcome {
    /// 可装载声明。
    pub decls: Vec<engine_native::content::extension::VmSkillDecl>,
    /// 被跳过的 filter 名（不可承载的具名组合 filter）。
    pub deferred_filters: Vec<String>,
}

pub fn bridge_vm_skill_decls(
    pkg: &content_schema::ContentPackage,
) -> Result<VmBridgeOutcome, BridgeError> {
    use engine_native::event::{EventKind, Timing};
    ensure_symbol_capacity()?;
    let mut out = Vec::new();
    let mut deferred_filters = Vec::new();
    for t in &pkg.trigger_index {
        let module = pkg
            .modules
            .iter()
            .find(|m| m.name == t.module)
            .ok_or_else(|| BridgeError::InvalidField {
                field: "trigger_index.module",
                reason: format!("模块 {} 不在 npack modules 内", t.module),
            })?;
        let event = match t.event.as_str() {
            "phase_end" => EventKind::PhaseEnd,
            "phase_begin" => EventKind::PhaseBegin,
            "damage" => EventKind::Damage,
            "damage_before" => EventKind::DamageBefore,
            "card_used" => EventKind::CardUsed,
            // card_targeted：原生无独立 targeted 帧型，以 CardUsed+targets
            // 表达（增量 17，登记差异）。
            "card_targeted" => EventKind::CardUsed,
            "card_responded" => EventKind::CardResponded,
            "cards_moved" => EventKind::CardsMoved,
            "cards_drawn" => EventKind::CardsDrawn,
            "dying" => EventKind::Dying,
            "dying_after" => EventKind::DyingSaved,
            "death" => EventKind::Death,
            "turn_ended" => EventKind::TurnEnded,
            "hp_lost" => EventKind::LoseHp,
            "hp_recovered" => EventKind::Recover,
            other => {
                return Err(BridgeError::InvalidField {
                    field: "trigger_index.event",
                    reason: format!("VM 技能事件名 {other} 未映射（增量 3 严格面）"),
                })
            }
        };
        let Some(filter) = engine_native::rvm_bridge::map_vm_filter(t.filter.as_str()) else {
            deferred_filters.push(t.filter.clone());
            continue;
        };
        out.push(engine_native::content::extension::VmSkillDecl {
            name: names::intern(t.module.clone()).to_string(),
            module: module.clone(),
            trigger: engine_native::catalog::TriggerMeta {
                event,
                timing: Timing::Normal,
                priority: 0,
                optional: t.optional,
                extra_events: Vec::new(),
            },
            filter,
        });
    }
    Ok(VmBridgeOutcome {
        decls: out,
        deferred_filters,
    })
}

fn ensure_symbol_capacity() -> Result<(), BridgeError> {
    let limit = engine_native::content::names::SYMBOL_TABLE_LIMIT;
    if names::interned_count() >= limit {
        return Err(BridgeError::SymbolTableLimit { limit });
    }
    Ok(())
}

fn parse_json(json: &str) -> Result<Value, BridgeError> {
    serde_json::from_str(json).map_err(|e| BridgeError::BadJson(e.to_string()))
}

fn req_str<'a>(v: &'a Value, key: &str, field: &'static str) -> Result<&'a str, BridgeError> {
    match v.get(key) {
        None | Some(Value::Null) => Err(BridgeError::MissingField(field)),
        Some(Value::String(s)) => Ok(s.as_str()),
        Some(other) => Err(invalid(field, format!("须为字符串，实为 {other}"))),
    }
}

fn opt_str<'a>(
    v: &'a Value,
    key: &str,
    field: &'static str,
) -> Result<Option<&'a str>, BridgeError> {
    match v.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => Ok(Some(s.as_str())),
        Some(other) => Err(invalid(field, format!("须为字符串，实为 {other}"))),
    }
}

fn int_in(x: &Value, field: &'static str, range: RangeInclusive<u64>) -> Result<u64, BridgeError> {
    match x.as_u64() {
        Some(n) if range.contains(&n) => Ok(n),
        _ => Err(invalid(
            field,
            format!("须为 {}–{} 的整数，实为 {x}", range.start(), range.end()),
        )),
    }
}

fn req_int(
    v: &Value,
    key: &str,
    field: &'static str,
    range: RangeInclusive<u64>,
) -> Result<u64, BridgeError> {
    match v.get(key) {
        None | Some(Value::Null) => Err(BridgeError::MissingField(field)),
        Some(x) => int_in(x, field, range),
    }
}

fn opt_int(
    v: &Value,
    key: &str,
    field: &'static str,
    range: RangeInclusive<u64>,
) -> Result<Option<u64>, BridgeError> {
    match v.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(x) => int_in(x, field, range).map(Some),
    }
}

fn req_array<'a>(v: &'a Value, key: &str, field: &'static str) -> Result<&'a [Value], BridgeError> {
    match v.get(key) {
        None | Some(Value::Null) => Err(BridgeError::MissingField(field)),
        Some(Value::Array(a)) => Ok(a.as_slice()),
        Some(other) => Err(invalid(field, format!("须为数组，实为 {other}"))),
    }
}

fn opt_array<'a>(v: &'a Value, key: &str, field: &'static str) -> Result<&'a [Value], BridgeError> {
    match v.get(key) {
        None | Some(Value::Null) => Ok(&[]),
        Some(Value::Array(a)) => Ok(a.as_slice()),
        Some(other) => Err(invalid(field, format!("须为数组，实为 {other}"))),
    }
}

/// `mapping` 字段：字符串 = 宿主行为键；null/缺省 = 暂缓（未映射）。
/// 技能映射三态（T12 运筹武将面批 8）：暂缓（mapping null）/内置引用
/// （`builtin:名`，白名单——**只收有原生实现的标准技**；引用不存在的
/// def 会在建局后行为面静默失效，故白名单而非开放查名）/宿主行为。
enum MappedSkill {
    Deferred,
    Builtin(&'static str),
    Host(HostBehavior),
}

/// 内置技能引用白名单：仅收**有原生实现**的标准技（护驾=kernel 主公技
/// 臂、谦逊=乐不思蜀目标校验查询面）。悲歌本体未实现，不入册。
const BUILTIN_SKILL_REFS: [&str; 11] = [
    "护驾", "谦逊", "义从", "酒池", "龙胆", "无双", "闭月", "伤逝", "绝情", "突袭", "流离",
];

fn mapping_of(v: &Value, field: &'static str) -> Result<MappedSkill, BridgeError> {
    match opt_str(v, "mapping", field)? {
        None => Ok(MappedSkill::Deferred),
        Some(key) => {
            if let Some(builtin) = key.strip_prefix("builtin:") {
                return BUILTIN_SKILL_REFS
                    .iter()
                    .find(|b| **b == builtin)
                    .map(|b| MappedSkill::Builtin(b))
                    .ok_or_else(|| BridgeError::UnknownBehaviorKey(key.to_string()));
            }
            HostBehavior::from_key(key)
                .map(MappedSkill::Host)
                .ok_or_else(|| BridgeError::UnknownBehaviorKey(key.to_string()))
        }
    }
}

/// 可选布尔字段：缺省 None；存在则必须为布尔。
fn opt_bool(v: &Value, field: &'static str) -> Result<Option<bool>, BridgeError> {
    match v.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(b @ Value::Bool(_)) => Ok(Some(b.as_bool().unwrap())),
        Some(other) => Err(invalid(field, format!("须为布尔，实为 {other}"))),
    }
}

fn suit_of(suit: &str) -> Result<Suit, BridgeError> {
    Ok(match suit {
        "spade" => Suit::Spade,
        "heart" => Suit::Heart,
        "club" => Suit::Club,
        "diamond" => Suit::Diamond,
        _ => return Err(BridgeError::UnknownSuit(suit.to_string())),
    })
}

fn faction_of(f: &str) -> Result<Faction, BridgeError> {
    Ok(match f {
        "wei" => Faction::Wei,
        "shu" => Faction::Shu,
        "wu" => Faction::Wu,
        "qun" => Faction::Qun,
        _ => return Err(BridgeError::UnknownFaction(f.to_string())),
    })
}

fn gender_of(g: &str) -> Result<Gender, BridgeError> {
    Ok(match g {
        "male" => Gender::Male,
        "female" => Gender::Female,
        _ => return Err(BridgeError::UnknownGender(g.to_string())),
    })
}

/// 校验 cards.json 可选的 `host_api` 字段：存在则必须是与运行时引擎 API
/// 版本（content-runtime `tooling_host_versions().engine_api`）相等的非负整数。
pub fn ensure_host_api_compatible(json: &Value) -> Result<(), BridgeError> {
    let current = content_runtime::tooling_host_versions().engine_api;
    match json.get("host_api") {
        None | Some(Value::Null) => Ok(()),
        Some(v) => match v.as_u64() {
            Some(declared) if declared == u64::from(current) => Ok(()),
            Some(declared) => Err(BridgeError::HostVersionMismatch { declared, current }),
            None => Err(invalid("host_api", format!("须为非负整数，实为 {v}"))),
        },
    }
}

struct PackMeta {
    def: ExtensionDef,
    card_prefix: String,
    init_hook: InitHook,
}

fn pack_meta(parsed: &Value) -> Result<PackMeta, BridgeError> {
    let pack = match parsed.get("pack") {
        None | Some(Value::Null) => return Err(BridgeError::MissingField("pack")),
        Some(p @ Value::Object(_)) => p,
        Some(other) => return Err(invalid("pack", format!("须为对象，实为 {other}"))),
    };
    let name = req_str(pack, "name", "pack.name")?;
    if name.is_empty() {
        return Err(invalid("pack.name", "不能为空"));
    }
    let version = req_int(pack, "version", "pack.version", 1..=u64::from(u32::MAX))? as u32;
    let description = opt_str(pack, "description", "pack.description")?.unwrap_or("");
    let card_prefix = req_str(pack, "card_prefix", "pack.card_prefix")?;
    if card_prefix.is_empty() {
        return Err(invalid(
            "pack.card_prefix",
            "不能为空（牌名须与基础目录隔离）",
        ));
    }
    let init_hook = match opt_str(pack, "init_hook", "pack.init_hook")? {
        None => InitHook::None,
        Some(key) => {
            InitHook::from_key(key).ok_or_else(|| BridgeError::UnknownInitHook(key.to_string()))?
        }
    };
    Ok(PackMeta {
        def: ExtensionDef {
            name: names::intern(name.to_string()),
            version,
            description: names::intern(description.to_string()),
        },
        card_prefix: card_prefix.to_string(),
        init_hook,
    })
}

/// 装载主入口：cards.json → `ExtensionPack` + 暂缓清单。
///
/// 宿主技能次序：先按牌首次出现的次序登记牌种伴随技能，再登记 `skills`
/// 数组中有 `mapping` 的条目（次序决定目录技能编号，属确定性规则）。
/// 声明 `range` 的牌按通用武器建模（牌种记为 `Weapon`），`kind` 须为
/// 武器栏牌种。
pub fn bridge_pack(json: &str) -> Result<BridgedPack, BridgeError> {
    ensure_symbol_capacity()?;
    let parsed = parse_json(json)?;
    ensure_host_api_compatible(&parsed)?;
    let meta = pack_meta(&parsed)?;

    let mut cards = Vec::new();
    let mut host_skills: Vec<HostSkillDecl> = Vec::new();
    // cards 缺省为空：纯武将包（角色包管线）合法。
    for (i, c) in opt_array(&parsed, "cards", "cards")?.iter().enumerate() {
        let name = req_str(c, "name", "cards[].name")?;
        let kind_key = req_str(c, "kind", "cards[].kind")?;
        let kind = CardKind::from_key(kind_key)
            .ok_or_else(|| BridgeError::UnknownCardKind(kind_key.to_string()))?;
        let suit = suit_of(req_str(c, "suit", "cards[].suit")?)?;
        let rank = req_int(c, "rank", "cards[].rank", 1..=13)? as u8;
        let def_name = names::intern(format!("{}{}_{}", meta.card_prefix, name, i));
        let mut def = match opt_int(c, "range", "cards[].range", 1..=u64::from(u32::MAX))? {
            Some(range) => {
                if kind.equip_slot() != Some(EquipSlot::Weapon) {
                    return Err(invalid(
                        "cards[].range",
                        format!("{kind_key} 不是武器栏牌种，不能声明射程"),
                    ));
                }
                CardDef::weapon(def_name, range as u32)
            }
            None => CardDef::basic(def_name, kind),
        }
        .with_suit_rank(suit, rank);
        if opt_bool(c, "link_on_equip")?.unwrap_or(false) {
            def = def.with_link_on_equip();
        }
        let mut equip_skills = card_kind_equip_skills(kind).to_vec();
        for s in opt_array(c, "equip_skills", "cards[].equip_skills")? {
            let s = s.as_str().ok_or_else(|| {
                invalid("cards[].equip_skills", format!("元素须为字符串，实为 {s}"))
            })?;
            equip_skills.push(names::intern(s.to_string()));
        }
        def.equip_skills = equip_skills;
        for decl in card_kind_host_skills(kind) {
            if !host_skills.iter().any(|d| d.name == decl.name) {
                host_skills.push(decl.clone());
            }
        }
        cards.push(def);
    }

    for sk in opt_array(&parsed, "skills", "skills")? {
        let key = req_str(sk, "name", "skills[].name")?;
        // cards.json 面暂无内置引用需求：Deferred/Builtin 均跳过
        if let MappedSkill::Host(behavior) = mapping_of(sk, "skills[].mapping")? {
            host_skills.push(HostSkillDecl {
                name: names::intern(key.to_string()),
                behavior,
            });
        }
    }

    let mut registered = BTreeSet::new();
    for d in &host_skills {
        if !registered.insert(d.name) {
            return Err(BridgeError::DuplicateSkill(d.name.to_string()));
        }
    }
    for def in &cards {
        for s in &def.equip_skills {
            if !registered.contains(s) {
                return Err(BridgeError::UnknownEquipSkill(s.to_string()));
            }
        }
    }

    let mut deferred = Vec::new();
    for d in opt_array(&parsed, "deferred", "deferred")? {
        deferred.push(req_str(d, "name", "deferred[].name")?.to_string());
    }

    // generals 缺省为空：卡牌包合法；同文件声明的武将经同一 pack 段装载。
    // 武将命中的宿主技能声明追加在伴随技能与 skills 数组之后（次序决定
    // 目录技能编号，属确定性规则的一部分）。
    let generals_out = bridge_generals_inner(&parsed, false)?;
    for decl in generals_out.host_skills {
        match host_skills.iter().find(|d| d.name == decl.name) {
            Some(d) if d.behavior != decl.behavior => {
                return Err(BridgeError::DuplicateSkill(decl.name.to_string()));
            }
            Some(_) => {}
            None => host_skills.push(decl),
        }
    }
    let pack = ExtensionPack {
        def: meta.def,
        generals: generals_out.generals,
        cards,
        host_skills,
        // T8 增量 1：npack 内 rule-vm 模块桥接属增量 2（登记）；装载器
        // 当前不产出 vm 技能声明。
        vm_skills: Vec::new(),
        init_hook: meta.init_hook,
        enabled: true,
    };
    Ok(BridgedPack { pack, deferred })
}

/// 武将装载（角色包管线）：generals 数组 → `ExtensionGeneral` + 命中宿主
/// 行为的技能声明。势力、性别、体力上限缺失或非法均报错。
pub fn bridge_generals(json: &str) -> Result<BridgedGenerals, BridgeError> {
    ensure_symbol_capacity()?;
    let parsed = parse_json(json)?;
    ensure_host_api_compatible(&parsed)?;
    bridge_generals_inner(&parsed, true)
}

/// `require_generals`：独立入口必需；全量入口（bridge_pack）对纯卡牌包
/// 容许缺省。
fn bridge_generals_inner(
    parsed: &Value,
    require_generals: bool,
) -> Result<BridgedGenerals, BridgeError> {
    let generals_json = if require_generals {
        req_array(&parsed, "generals", "generals")?
    } else {
        opt_array(&parsed, "generals", "generals")?
    };
    let mut generals = Vec::new();
    let mut mapped_skill_keys = Vec::new();
    let mut host_skills: Vec<HostSkillDecl> = Vec::new();
    let mut unmapped = 0usize;
    for g in generals_json {
        let name = req_str(g, "name", "generals[].name")?;
        let mut skills: Vec<&'static str> = Vec::new();
        for sk in req_array(g, "skills", "generals[].skills")? {
            let key = req_str(sk, "name", "generals[].skills[].name")?;
            match mapping_of(sk, "generals[].skills[].mapping")? {
                MappedSkill::Host(behavior) => {
                    let interned = names::intern(key.to_string());
                    match host_skills.iter().find(|d| d.name == interned) {
                        Some(d) if d.behavior != behavior => {
                            return Err(BridgeError::DuplicateSkill(key.to_string()));
                        }
                        Some(_) => {}
                        None => host_skills.push(HostSkillDecl {
                            name: interned,
                            behavior,
                        }),
                    }
                    skills.push(interned);
                    mapped_skill_keys.push(interned);
                }
                // 内置引用：技能 def 已在标准目录（白名单保证实现存在）
                // ——只 intern 进武将技能列表，不产出 host 行为。
                MappedSkill::Builtin(builtin) => {
                    skills.push(names::intern(builtin.to_string()));
                }
                MappedSkill::Deferred => unmapped += 1,
            }
        }
        let faction = faction_of(req_str(g, "faction", "generals[].faction")?)?;
        let gender = gender_of(req_str(g, "gender", "generals[].gender")?)?;
        let max_hp = req_int(g, "hp", "generals[].hp", 1..=u64::from(u8::MAX))? as u32;
        generals.push(ExtensionGeneral {
            name: names::intern(name.to_string()),
            faction,
            skills: names::intern_list(skills),
            max_hp,
            gender,
        });
    }
    Ok(BridgedGenerals {
        generals,
        host_skills,
        mapped_skill_keys,
        unmapped_skills: unmapped,
    })
}

/// 牌堆比例表装载（T6①）：已安装 cardpile 包的 `assets/pile.json` →
/// `PileSpec`。字段缺失/类型非法给出明确原因；未知牌名/花色、空条目与
/// 零张数由 [`engine_native::content::pile::extension_pack_from_pile`]
/// 在构建期校验。`counts` 键经 `BTreeMap` 排序——生成卡牌序（目录编号、
/// 发牌序的一部分）与解析器实现无关，仅由数据决定。
pub fn bridge_pile(json: &str) -> Result<PileSpec, BridgeError> {
    ensure_symbol_capacity()?;
    let parsed = parse_json(json)?;
    let source = req_str(&parsed, "source", "pile.json source")?.to_string();
    let total = req_int(&parsed, "total", "pile.json total", 1..=u64::from(u32::MAX))? as u32;
    let mut entries = Vec::new();
    for e in req_array(&parsed, "entries", "pile.json entries")? {
        let card = req_str(e, "card", "pile.json entries[].card")?.to_string();
        let counts_json = match e.get("counts") {
            None | Some(Value::Null) => {
                return Err(BridgeError::MissingField("pile.json entries[].counts"))
            }
            Some(Value::Object(m)) => m,
            Some(other) => {
                return Err(invalid(
                    "pile.json entries[].counts",
                    format!("须为对象，实为 {other}"),
                ))
            }
        };
        let counts: BTreeMap<String, u32> = counts_json
            .iter()
            .map(|(suit, n)| {
                let n = int_in(n, "pile.json entries[].counts", 1..=u64::from(u32::MAX))? as u32;
                Ok((suit.clone(), n))
            })
            .collect::<Result<BTreeMap<_, _>, BridgeError>>()?;
        entries.push(PileEntry {
            card,
            counts: counts.into_iter().collect(),
        });
    }
    Ok(PileSpec {
        source,
        total,
        entries,
    })
}

const PILE_FACTOR_CARDS: &[&str] = &[
    "guohe", "huosha", "jiu", "leisha", "nanman", "sha", "shan", "shunshou", "tao", "tiesuo",
    "wanjian", "wuxie",
];

/// 解析牌堆倍率。键必须是比例表里的牌名，值必须是有限非负数。
///
/// 缺某个牌名不是这里的错误：[`engine_native::content::pile::scale_plan`]
/// 会把它看成上游未配置，从而不追加牌。
pub fn parse_pile_factors(json: &str) -> Result<BTreeMap<String, f64>, String> {
    let value: Value = serde_json::from_str(json).map_err(|err| format!("factors: {err}"))?;
    let obj = value
        .as_object()
        .ok_or_else(|| "factors: not an object".to_string())?;
    let mut factors = BTreeMap::new();
    for (key, raw) in obj {
        if !PILE_FACTOR_CARDS.contains(&key.as_str()) {
            return Err(format!("unknown pile factor {key}"));
        }
        let number = raw
            .as_f64()
            .ok_or_else(|| format!("pile factor {key} is not a number"))?;
        if !number.is_finite() || number < 0.0 {
            return Err(format!(
                "pile factor {key} must be a finite non-negative number"
            ));
        }
        factors.insert(key.clone(), number);
    }
    Ok(factors)
}
