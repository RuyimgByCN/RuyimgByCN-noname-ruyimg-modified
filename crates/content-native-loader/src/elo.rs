//! L22 段位制裁：已知牌、制裁力度，以及摸牌和判定前的重排。
//!
//! 上游在牌堆插入时打上 `isEloKnown`，洗牌时清掉。难度大于 1 且有偏向时，
//! 从牌堆顶拿走 `ceil(张数 * 力度)` 张，按调用方给出的分数重排后再放回。
//! 分数代替 `get.value` 或判定函数。不延迟，不说话，也不筛将池。

use serde_json::{json, Value};

/// 牌堆里的一张牌。`score` 由调用方给出。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EloCard {
    /// 测试和快照里的编号。
    pub id: u32,
    /// 对应 `isEloKnown`。
    pub known: bool,
    /// 摸牌时是价值，判定时是判定函数的返回值。
    pub score: i64,
}

/// 包内配置。对应上游 `config` 的三项。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EloConfig {
    /// 来源仓。缺省键解析为 `noname`。
    pub source: String,
    /// `eloPlayers`。为真也不筛将池。
    pub elo_players: bool,
    /// `eloDifficulty` 的原文字。
    pub difficulty: String,
    /// `timeKiller` 的原文字。
    pub time_killer: String,
}

/// 排位环境。不读 `sessionStorage`，也不调用如真似幻的段位函数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EloQuery {
    /// `sessionStorage.Network` 缺省或等于 `online`。
    pub online: bool,
    /// `get.mode()`。只有 `taixuhuanjing` 会把段位档打回关闭。
    pub mode: String,
    /// `helasisy_getPaiWeiInfo('level')`。`None` 表示那个函数不存在。
    pub paiwei_level: Option<i32>,
}

/// 解析后的力度。`eh` 为 1、0 或 -1。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EloLevel {
    /// 偷看倍数。
    pub diff: u32,
    /// 偏向。`1` 照顾己方，`-1` 照顾对方，`0` 不重排。
    pub eh: i8,
}

/// 烧条结果。只计算秒数，不调用 `game.delay`。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Think {
    /// `time * 1.5 * unit`。
    pub delay: f64,
    /// 台词不播放。
    pub chat_refused: bool,
    /// 没有真正等待。
    pub slept: bool,
}

/// 谁在拆牌或顺牌。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EloViewer {
    /// 有 `game.me`。
    pub has_me: bool,
    /// 行动者是自己。
    pub is_me: bool,
    /// `isUnderControl(true)`。
    pub under_control: bool,
}

/// 段位制裁的失败。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EloError {
    /// 配置无法使用。
    BadConfig(String),
    /// 透视抽样不是 0 或 1。
    BadRoll(String),
    /// 思考随机数不在 0 到 1。
    BadRandom(String),
    /// 快照无法恢复。
    BadSnapshot(String),
}

fn source_token(value: Option<&Value>) -> Result<String, EloError> {
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
        Some(_) => Err(EloError::BadConfig("source must be a token".into())),
    }
}

/// 解析包内 `config.json`。
pub fn parse_elo_config(text: &str) -> Result<EloConfig, EloError> {
    let value: Value =
        serde_json::from_str(text).map_err(|err| EloError::BadConfig(err.to_string()))?;
    let obj = value
        .as_object()
        .ok_or_else(|| EloError::BadConfig("config must be an object".into()))?;
    let elo_players = obj
        .get("eloPlayers")
        .and_then(Value::as_bool)
        .ok_or_else(|| EloError::BadConfig("eloPlayers must be a bool".into()))?;
    let difficulty = obj
        .get("eloDifficulty")
        .and_then(Value::as_str)
        .ok_or_else(|| EloError::BadConfig("eloDifficulty must be a string".into()))?
        .to_string();
    let time_killer = obj
        .get("timeKiller")
        .and_then(Value::as_str)
        .ok_or_else(|| EloError::BadConfig("timeKiller must be a string".into()))?
        .to_string();
    let source = source_token(obj.get("source"))?;
    Ok(EloConfig {
        source,
        elo_players,
        difficulty,
        time_killer,
    })
}

/// 把配置档折成力度和偏向。
pub fn resolve_difficulty(key: &str, query: &EloQuery) -> EloLevel {
    let mut diff: i64 = 1;
    let mut eh: i8 = 0;
    match key {
        "eeasy" => {
            diff = 4;
            eh = 1;
        }
        "veasy" => {
            diff = 3;
            eh = 1;
        }
        "easy" => {
            diff = 2;
            eh = 1;
        }
        "normal" => {
            diff = 1;
            eh = 0;
        }
        "hard" => {
            diff = 2;
            eh = -1;
        }
        "vhard" => {
            diff = 3;
            eh = -1;
        }
        "ehard" => {
            diff = 4;
            eh = -1;
        }
        "paiwei" => {
            if let Some(level) = query.paiwei_level {
                diff = 3 - i64::from(level);
            }
            if diff > 0 {
                eh = 1;
            } else if diff < 0 {
                eh = -1;
                diff = diff.abs();
            }
            diff += 1;
            if !query.online || query.mode == "taixuhuanjing" {
                eh = 0;
                diff = 1;
            }
        }
        _ => {
            diff = 4;
            eh = 0;
        }
    }
    EloLevel {
        diff: u32::try_from(diff).unwrap_or(u32::MAX),
        eh,
    }
}

/// 开局手牌视为已知。
pub fn mark_opening(cards: &mut [EloCard]) {
    for card in cards {
        card.known = true;
    }
}

/// 塞进牌堆。`no_mark` 对应重排放回时的第三个参数。
pub fn mark_inserted(card: &mut EloCard, no_mark: bool) {
    if !no_mark {
        card.known = true;
    }
}

/// 洗牌清掉全部已知标记。
pub fn wash(cards: &mut [EloCard]) {
    for card in cards {
        card.known = false;
    }
}

fn order_window(window: &mut [EloCard], enemy: bool, eh: i8) {
    let unknown_idx: Vec<usize> = window
        .iter()
        .enumerate()
        .filter(|(_, card)| !card.known)
        .map(|(i, _)| i)
        .collect();
    if unknown_idx.is_empty() {
        return;
    }
    let mut unknown: Vec<EloCard> = unknown_idx.iter().map(|&i| window[i].clone()).collect();
    unknown.sort_by(|a, b| {
        if enemy {
            a.score.cmp(&b.score)
        } else {
            b.score.cmp(&a.score)
        }
    });
    if eh == -1 {
        unknown.reverse();
    }
    for (slot, card) in unknown_idx.into_iter().zip(unknown) {
        window[slot] = card;
    }
}

/// 摸牌或判定前重排牌堆顶。牌堆第 0 张是顶。
pub fn reorder_draw(pile: &[EloCard], draw_num: u32, level: EloLevel, enemy: bool) -> Vec<EloCard> {
    if level.eh == 0 || level.diff <= 1 || pile.len() <= 1 || draw_num < 1 {
        return pile.to_vec();
    }
    let peek = (draw_num as usize)
        .saturating_mul(level.diff as usize)
        .min(pile.len());
    if peek <= 1 {
        return pile.to_vec();
    }
    let mut out = pile.to_vec();
    order_window(&mut out[..peek], enemy, level.eh);
    out
}

/// 拆牌或顺牌是否把目标牌翻开。`roll == 1` 才是 `[1,0,0]` 里会继续的那一次。
pub fn hand_visible(
    level: EloLevel,
    viewer: EloViewer,
    is_friend: bool,
    roll: Option<u8>,
) -> Result<bool, EloError> {
    if level.eh == 0 || !viewer.has_me || viewer.is_me || viewer.under_control || level.diff < 3 {
        return Ok(false);
    }
    if level.diff < 4 {
        match roll {
            Some(1) => {}
            Some(0) => return Ok(false),
            Some(other) => return Err(EloError::BadRoll(format!("roll {other}"))),
            None => return Err(EloError::BadRoll("missing roll".into())),
        }
    }
    Ok((level.eh == 1 && is_friend) || (level.eh == -1 && !is_friend))
}

/// 烧条秒数。离线为 0。不认识的档也是 0。
pub fn think_delay(config: &EloConfig, online: bool, unit: f64) -> Result<Think, EloError> {
    if !unit.is_finite() || !(0.0..=1.0).contains(&unit) {
        return Err(EloError::BadRandom("unit random out of 0..=1".into()));
    }
    let time = if !online {
        0
    } else {
        match config.time_killer.as_str() {
            "none" => 0,
            "light" => 1,
            "middle" => 2,
            "heavy" => 3,
            _ => 0,
        }
    };
    Ok(Think {
        delay: time as f64 * 1.5 * unit,
        chat_refused: true,
        slept: false,
    })
}

/// 将池保持原样。`eloPlayers` 在上游这个文件里没有被读取。
pub fn restrict_pool<T: Clone>(config: &EloConfig, names: &[T]) -> Vec<T> {
    let _ = config.elo_players;
    names.to_vec()
}

/// 牌堆快照。
pub fn snapshot_pile(pile: &[EloCard]) -> String {
    json!(pile
        .iter()
        .map(|card| json!({"id": card.id, "known": card.known, "score": card.score}))
        .collect::<Vec<_>>())
    .to_string()
}

/// 从 [`snapshot_pile`] 恢复。
pub fn restore_pile(text: &str) -> Result<Vec<EloCard>, EloError> {
    let value: Value =
        serde_json::from_str(text).map_err(|err| EloError::BadSnapshot(err.to_string()))?;
    let list = value
        .as_array()
        .ok_or_else(|| EloError::BadSnapshot("pile".into()))?;
    let mut out = Vec::with_capacity(list.len());
    for item in list {
        let obj = item
            .as_object()
            .ok_or_else(|| EloError::BadSnapshot("card".into()))?;
        let id = obj
            .get("id")
            .and_then(Value::as_u64)
            .ok_or_else(|| EloError::BadSnapshot("id".into()))?;
        let id = u32::try_from(id).map_err(|_| EloError::BadSnapshot("id".into()))?;
        let known = obj
            .get("known")
            .and_then(Value::as_bool)
            .ok_or_else(|| EloError::BadSnapshot("known".into()))?;
        let score = obj
            .get("score")
            .and_then(Value::as_i64)
            .ok_or_else(|| EloError::BadSnapshot("score".into()))?;
        out.push(EloCard { id, known, score });
    }
    Ok(out)
}

fn sample_pile() -> Vec<EloCard> {
    vec![
        EloCard {
            id: 2,
            known: false,
            score: 1,
        },
        EloCard {
            id: 3,
            known: false,
            score: 9,
        },
        EloCard {
            id: 4,
            known: false,
            score: 4,
        },
        EloCard {
            id: 1,
            known: true,
            score: 0,
        },
    ]
}

fn format_delay(delay: f64) -> String {
    if delay.fract() == 0.0 {
        format!("{}", delay as i64)
    } else {
        format!("{delay}")
    }
}

/// 供目标平台字符串辅助调用。
pub fn apply_elo_commands(
    config_text: &str,
    online: bool,
    mode: &str,
    level: Option<i32>,
    commands: &str,
) -> Result<String, EloError> {
    let config = parse_elo_config(config_text)?;
    let query = EloQuery {
        online,
        mode: mode.to_string(),
        paiwei_level: level,
    };
    let resolved = resolve_difficulty(&config.difficulty, &query);
    let mut parts_out = Vec::new();
    for command in commands.split(';').filter(|part| !part.is_empty()) {
        let mut parts = command.split(':');
        match parts.next().unwrap_or("") {
            "diff" => parts_out.push(format!("diff:{} eh:{}", resolved.diff, resolved.eh)),
            "order" => {
                let enemy = parts.next() == Some("enemy");
                let pile = reorder_draw(&sample_pile(), 2, resolved, enemy);
                parts_out.push(format!(
                    "top:{}",
                    pile.first().map(|card| card.id).unwrap_or(0)
                ));
            }
            "visible" => {
                let who = parts.next().unwrap_or("");
                let roll = match parts.next() {
                    Some(text) => Some(
                        text.parse::<u8>()
                            .map_err(|_| EloError::BadRoll(text.into()))?,
                    ),
                    None => None,
                };
                let viewer = match who {
                    "me" => EloViewer {
                        has_me: true,
                        is_me: true,
                        under_control: false,
                    },
                    "control" => EloViewer {
                        has_me: true,
                        is_me: false,
                        under_control: true,
                    },
                    "absent" => EloViewer {
                        has_me: false,
                        is_me: false,
                        under_control: false,
                    },
                    "friend" | "enemy" => EloViewer {
                        has_me: true,
                        is_me: false,
                        under_control: false,
                    },
                    other => return Err(EloError::BadConfig(format!("unknown viewer {other}"))),
                };
                let shown = hand_visible(resolved, viewer, who != "enemy", roll)?;
                parts_out.push(format!("visible:{}", u8::from(shown)));
            }
            "think" => {
                let unit = parts
                    .next()
                    .unwrap_or("0")
                    .parse::<f64>()
                    .map_err(|_| EloError::BadRandom("unit".into()))?;
                let think = think_delay(&config, online, unit)?;
                parts_out.push(format!(
                    "delay:{} chat:{} slept:{}",
                    format_delay(think.delay),
                    if think.chat_refused {
                        "refused"
                    } else {
                        "said"
                    },
                    u8::from(think.slept)
                ));
            }
            "pool" => {
                let names = restrict_pool(&config, &["caocao", "liubei"]);
                parts_out.push(format!("pool:{}", names.join(",")));
            }
            other => return Err(EloError::BadConfig(format!("unknown command {other}"))),
        }
    }
    Ok(parts_out.join(" "))
}

pub const L22_ELO_SKILL_ID: u32 = 1_926_012_001;

pub fn make_l22_vm_skill_decl(
    name_str: &str,
    event: engine_native::event::EventKind,
    idx: usize,
) -> engine_native::content::extension::VmSkillDecl {
    let mod_name = format!("{name_str}#l22elo");
    let mut code = Vec::with_capacity(14);
    code.push(0x57); // HOST_SKILL
    code.push(0x10); // CONST_I32
    code.extend_from_slice(&(super::DECISION34_STORAGE_DISPATCH_KEY as i32).to_le_bytes());
    code.push(0x10); // CONST_I32
    code.extend_from_slice(&(L22_ELO_SKILL_ID as i32).to_le_bytes());
    code.push(0x5A); // HOST_EFFECT_EXT
    code.extend_from_slice(&649u16.to_le_bytes()); // STORAGE_SET = 649
    code.push(0x01); // HALT
    let module = engine_native::rvm_bridge::BytecodeModule {
        logic_hash: engine_native::rvm_bridge::BytecodeModule::compute_logic_hash(
            &mod_name, &code, 0, 1, 0,
        ),
        name: mod_name,
        code,
        local_count: 0,
        max_stack: 4,
        entry_pc: 0,
        caps_mask: 1,
    };
    let skill_def_id = L22_ELO_SKILL_ID.wrapping_add(idx as u32);
    engine_native::content::extension::VmSkillDecl {
        declaration_hash: None,
        presentation_binding: None,
        name: name_str.to_string(),
        module,
        trigger: engine_native::catalog::TriggerMeta {
            event,
            timing: engine_native::event::Timing::Normal,
            priority: 0,
            order_selectable: false,
            optional: false,
            extra_events: Vec::new(),
        },
        filter: engine_native::rvm_bridge::VmFilterPolicy::Handlers(std::sync::Arc::new(
            engine_native::rvm_bridge::VmHandlerSet {
                skill_def_id,
                active: None,
                triggers: Vec::new(),
                view_as: Vec::new(),
                modifiers: Vec::new(),
                limits: engine_native::rvm_bridge::VmActiveLimits::default(),
            },
        )),
        r05_identity: None,
    }
}

/// Build the formal `ExtensionPack` for `L22` (`段位制裁`, `upstream-liuli/elo-sanction`),
/// registering all 5 global skills (`_eloMechanism`, `_eloInitCards`, `_eloDraw`,
/// `_eloHandcard`, `_timeKiller`).
pub fn elo_sanction_extension_pack() -> engine_native::content::extension::ExtensionPack {
    let vm_skills = engine_native::kernel::L22_ELO_SKILLS
        .iter()
        .enumerate()
        .map(|(idx, &(name, event))| make_l22_vm_skill_decl(name, event, idx))
        .collect();
    engine_native::content::extension::ExtensionPack {
        def: engine_native::content::extension::ExtensionDef {
            name: "elo-sanction",
            version: 1,
            description: "段位制裁",
        },
        enabled: true,
        cards: Vec::new(),
        generals: Vec::new(),
        host_skills: Vec::new(),
        vm_skills,
        init_hook: engine_native::content::extension::InitHook::None,
    }
}

/// Apply parsed `L22` (`段位制裁`) `EloConfig` (`eloPlayers`, `eloDifficulty`, `timeKiller`)
/// and `EloQuery` onto a live `Engine` match.
pub fn elo_apply_engine_config(
    engine: &mut engine_native::Engine,
    cfg: &EloConfig,
    query: &EloQuery,
) {
    let level = resolve_difficulty(&cfg.difficulty, query);
    let tk_val = match cfg.time_killer.as_str() {
        "light" => 1,
        "middle" | "medium" => 2,
        "heavy" => 3,
        _ => 0,
    };
    let core = engine.state_core_mut();
    engine_native::kernel::elo_set_global_scalar(
        core,
        "elo_players_enable",
        i32::from(cfg.elo_players),
    );
    engine_native::kernel::elo_set_global_scalar(core, "elo_eh", i32::from(level.eh));
    engine_native::kernel::elo_set_global_scalar(core, "elo_players_mode", i32::from(level.eh));
    engine_native::kernel::elo_set_global_scalar(core, "elo_difficulty_val", level.diff as i32);
    engine_native::kernel::elo_set_global_scalar(core, "elo_online", i32::from(query.online));
    engine_native::kernel::elo_set_global_scalar(core, "elo_time_killer_val", tk_val);
}
