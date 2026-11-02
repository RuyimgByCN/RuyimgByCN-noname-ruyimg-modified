//! Shared native/WASM session operations; binding errors are converted only at the WASM boundary.
use crate::installed_packs::packs_from_json;
use crate::winner_to_protocol_json;
use engine_native::ai::BoundedPlanner;
use engine_native::content::extension::ExtensionPack;
use engine_native::event::Phase;
use engine_native::view::{HandView, PlayerView};
use engine_native::{CommandOutcome, Engine, GameAction, RejectReason};

pub fn engine_from_packs(players: u8, seed: u64, packs_json: &str) -> Result<Engine, String> {
    let packs = packs_from_json(packs_json)?;
    let refs: Vec<&ExtensionPack> = packs.iter().collect();
    if refs.is_empty() {
        Ok(Engine::new(seed, players as usize))
    } else {
        Engine::new_with_extensions(seed, players as usize, &refs).map_err(|e| format!("{e:?}"))
    }
}

pub fn engine_identity_from_packs(
    players: u8,
    seed: u64,
    packs_json: &str,
) -> Result<Engine, String> {
    let packs = packs_from_json(packs_json)?;
    let refs: Vec<&ExtensionPack> = packs.iter().collect();
    Engine::try_new_identity_with_extensions(seed, players as usize, &refs)
}

pub fn engine_from_archive(hex: &str, packs_json: &str) -> Result<Engine, String> {
    let packs = packs_from_json(packs_json)?;
    let refs: Vec<&ExtensionPack> = packs.iter().collect();
    let bytes = hex_decode(hex)?;
    let snapshot =
        engine_native::archive::decode_archive(&bytes).map_err(|e| format!("解码失败: {e:?}"))?;
    if refs.is_empty() {
        Engine::restore(&snapshot).map_err(|e| format!("恢复失败: {e:?}"))
    } else {
        Engine::restore_with_extensions(&snapshot, &refs).map_err(|e| format!("恢复失败: {e:?}"))
    }
}

pub(super) fn prefix_count(engine: &Engine, prefix: &str) -> u32 {
    engine
        .catalog()
        .cards
        .iter()
        .filter(|c| c.name.starts_with(prefix))
        .count() as u32
}

/// 十六进制编解码（无 base64 依赖；存档字节面）。
pub fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// 存档十六进制文本上限（2 字符 = 1 字节）。超出时在分配/解码前拒绝，
/// 避免任意长输入进入 WASM 线性内存。64 MiB 字节对当前 NGSA 载荷有充分余量。
pub const MAX_ARCHIVE_HEX_CHARS: usize = 64 * 1024 * 1024;

/// 严格十六进制解码：先验证 ASCII/偶数长度/上限，再逐字节解析。
///
/// 旧实现按 `s[i..i + 2]` 字符串下标切片；奇数长度、多字节 UTF-8 或非 ASCII
/// 输入会 panic，WASM 侧表现为 `unreachable` 并破坏后续句柄使用。这里在
/// 字节层校验，错误只返回文本，不触碰调用方的原引擎。
pub(super) fn hex_decode(s: &str) -> Result<Vec<u8>, String> {
    hex_decode_bounded(s, MAX_ARCHIVE_HEX_CHARS)
}

pub(super) fn hex_decode_bounded(s: &str, max_chars: usize) -> Result<Vec<u8>, String> {
    if s.len() > max_chars {
        return Err(format!(
            "restore: 存档文本过长（{} 字符，上限 {max_chars}）",
            s.len()
        ));
    }
    if !s.is_ascii() {
        return Err("restore: 非法十六进制：只接受 ASCII 0-9a-fA-F".into());
    }
    let bytes = s.as_bytes();
    if bytes.len() % 2 != 0 {
        return Err(format!(
            "restore: 非法十六进制：长度为奇数（{}）",
            bytes.len()
        ));
    }
    let mut out = Vec::with_capacity(bytes.len() / 2);
    for (index, pair) in bytes.chunks_exact(2).enumerate() {
        let hi = hex_nibble(pair[0]).ok_or_else(|| {
            format!(
                "restore: 非法十六进制字符 {:?}（字节 {}）",
                pair[0] as char,
                index * 2
            )
        })?;
        let lo = hex_nibble(pair[1]).ok_or_else(|| {
            format!(
                "restore: 非法十六进制字符 {:?}（字节 {}）",
                pair[1] as char,
                index * 2 + 1
            )
        })?;
        out.push((hi << 4) | lo);
    }
    Ok(out)
}

fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

/// 命令结果的 JSON 投影（窄面：Accepted/Pending/Rejected 摘要）。
///
/// `Rejected` 附带 `reason`（`RejectReason` 的调试名）：界面必须能区分
/// 「被拒绝」与「为什么被拒绝」，否则伪造卡 id／越权座位只剩无因的失败。
fn outcome_json(engine: &Engine, out: &CommandOutcome) -> String {
    let reason = match out {
        CommandOutcome::Rejected(reason) => Some(format!("{reason:?}")),
        _ => None,
    };
    serde_json::json!({
        "outcome": match out {
            CommandOutcome::Accepted { .. } => "accepted",
            CommandOutcome::Pending { .. } => "pending",
            CommandOutcome::Rejected(_) => "rejected",
            CommandOutcome::Suspended => "suspended",
        },
        "revision": engine.revision(),
        "finished": engine.outcome().is_some(),
        "reason": reason,
    })
    .to_string()
}

fn rejected(engine: &Engine, reason: RejectReason) -> String {
    outcome_json(engine, &CommandOutcome::Rejected(reason))
}

fn phase_name(phase: Phase) -> &'static str {
    match phase {
        Phase::Judge => "judge",
        Phase::Draw => "draw",
        Phase::Play => "play",
        Phase::Discard => "discard",
        Phase::End => "end",
    }
}

/// 内核动作 → 客户端动作 JSON（与 TS `LegalAction` 联合类型同形，见
/// `apps/web-client/src/types/engine.ts`）。
///
/// 为什么在这里投影而不是让客户端解析内核类型：窄壳不暴露内核类型面，
/// 且 `Answer` 的候选值**必须先换算成下标**——窄壳 `answer()` 直取
/// `spec.options[idx]`，候选值只用于展示。换算在权威侧做一次，客户端
/// 拿到的 `selections` 就是可直接回传的 token。
///
/// 找不到候选下标的应答（理论不可达：候选即由同一 `spec.options` 生成）
/// 与当前查询不枚举的国战明置一律不入列，避免给出**不可提交**的动作。
fn action_to_json(engine: &Engine, action: &GameAction) -> Option<serde_json::Value> {
    match action {
        GameAction::NextPhase => Some(serde_json::json!("EndPhase")),
        GameAction::PlayCard { card, targets } => Some(serde_json::json!({
            "PlayCard": { "card": card, "targets": targets },
        })),
        GameAction::ActivateSkill { skill } => Some(serde_json::json!({
            "ActivateSkill": { "skill": skill, "targets": [], "card_targets": [] },
        })),
        GameAction::Answer {
            choice_id,
            selection,
        } => {
            let spec = engine.open_choice().map(|c| c.spec.clone())?;
            let mut selections = Vec::with_capacity(selection.len());
            for option in selection {
                selections.push(spec.options.iter().position(|o| o == option)? as u32);
            }
            Some(serde_json::json!({
                "ResolveChoice": {
                    "choice_id": choice_id,
                    "selections": selections,
                    "cancelled": selection.is_empty(),
                },
            }))
        }
        // 权威查询当前不产出明置候选（国战门控未接线）；出现也不投影。
        GameAction::RevealGeneral { .. } | GameAction::TimeoutAction { .. } => None,
    }
}

/// NC03 权威动作查询的 JSON 投影：`{"seat", "revision", "actions"}`。
///
/// 不经 `wasm_bindgen`，宿主测试（`tests/nc03_legal_actions_export.rs`）可直接
/// 调用，避免走到 `JsValue`。`revision` 是候选成立的状态修订号：客户端可据此
/// 判断候选是否过期（`Engine::legal_actions` 只读，不改状态）。
/// NC05/NC09：正式选将候选（宿主侧，非 wasm-bindgen）。
pub fn general_candidates_json_for(engine: &Engine) -> String {
    let rows: Vec<serde_json::Value> = engine
        .general_candidates()
        .into_iter()
        .filter_map(|def| {
            engine.catalog().general_meta(def).map(|m| {
                serde_json::json!({
                    "def": def,
                    "name": m.0,
                    "skills": m.2,
                })
            })
        })
        .collect();
    serde_json::json!({ "count": rows.len(), "candidates": rows }).to_string()
}

/// NC05/NC09：按名字正式选将（宿主侧，非 wasm-bindgen）。
pub fn assign_general_for(engine: &mut Engine, seat: u8, name: &str) -> Result<String, String> {
    let seats = engine.view_for(0).players.len();
    if seat as usize >= seats {
        return Err(format!("assign_general: 座位越界（{seat} ≥ {seats}）"));
    }
    engine
        .assign_general_named(seat, name)
        .map_err(|e| format!("assign_general 被拒绝: {e:?}"))?;
    Ok(serde_json::json!({
        "seat": seat,
        "name": name,
        "revision": engine.revision(),
        "state_hash": hex_encode(&engine.state_hash()),
    })
    .to_string())
}

pub fn legal_actions_json_for(engine: &Engine, seat: u8) -> Result<String, String> {
    let seats = engine.view_for(0).players.len();
    if seat as usize >= seats {
        return Err(format!("legal_actions: 座位越界（{seat} ≥ {seats}）"));
    }
    let actions: Vec<serde_json::Value> = engine
        .legal_actions(seat)
        .iter()
        .filter_map(|action| action_to_json(engine, action))
        .collect();
    Ok(serde_json::json!({
        "seat": seat,
        "revision": engine.revision(),
        "actions": actions,
    })
    .to_string())
}

/// 身份可见字符串（NC03/NC09 裁决⑥：观察者过滤后的 identity 随座位对象
/// 下发——本人/主公/已死亡可见，其余存活者 null；命名与协议 `IdentityTag`
/// 及前端 `SeatView.identity` 联合一致）。
fn identity_json(role: Option<engine_native::state::Role>) -> serde_json::Value {
    match role {
        None => serde_json::Value::Null,
        Some(engine_native::state::Role::Lord) => serde_json::json!("Lord"),
        Some(engine_native::state::Role::Loyalist) => serde_json::json!("Loyalist"),
        Some(engine_native::state::Role::Rebel) => serde_json::json!("Rebel"),
        Some(engine_native::state::Role::Traitor) => serde_json::json!("Renegade"),
    }
}

/// 牌面花色字符串（与前端 `CardView.suit` 联合一致）。
fn suit_json(suit: engine_native::catalog::Suit) -> &'static str {
    use engine_native::catalog::Suit;
    match suit {
        Suit::Spade => "Spade",
        Suit::Heart => "Heart",
        Suit::Club => "Club",
        Suit::Diamond => "Diamond",
    }
}

/// 牌面公开规则类别（与服务端 `native_view.rs` 的 `kind_tag` 同映射——
/// 跨入口同契约：已建模六类直映，其余 `Other` 不猜测）。
fn kind_tag_json(kind: engine_native::catalog::CardKind) -> &'static str {
    use engine_native::catalog::CardKind as K;
    match kind {
        K::Sha => "Sha",
        K::Tao => "Tao",
        K::Shan => "Shan",
        K::Jiu => "Jiu",
        K::FireSha => "ShaFire",
        K::LeiSha => "ShaThunder",
        _ => "Other",
    }
}

/// 座位视图投影（`PlayerView` 手写 JSON；视图为只读投影不进哈希）。
/// NC03/NC09 裁决⑥⑦（2026-10-28）：座位对象附带观察者过滤的 `identity`；
/// 顶层 `cards` 为**观察者可见牌**（本人手牌 + 全场装备/判定 + 弃牌/处理区，
/// 即 `visible_card_ids`）附真实牌面（def/suit/rank/kind）——他人手牌与
/// 牌堆 id 不在其中。仅扩展本地 `view_json` 内容，Worker/WS 消息结构不变。
fn view_to_json(engine: &Engine, v: &PlayerView) -> serde_json::Value {
    let catalog = engine.catalog();
    let faces = v
        .visible_card_ids()
        .into_iter()
        .map(|id| {
            let def = &catalog.cards[id as usize];
            (
                id.to_string(),
                serde_json::json!({
                    "def": def.kind as u16 as u32,
                    "suit": suit_json(def.suit),
                    "rank": def.rank,
                    "kind": kind_tag_json(def.kind),
                }),
            )
        })
        .collect::<serde_json::Map<_, _>>();
    serde_json::json!({
        "seat": v.seat,
        "players": v.players.iter().map(|p| serde_json::json!({
            "seat": p.seat,
            "generals": std::array::from_fn::<_, 2, _>(|slot| p.generals[slot].as_ref().map(|general| serde_json::json!({
                "def": general.def,
                "name": general.name,
                "faction": format!("{:?}", general.faction).to_lowercase(),
                "revealed": p.revealed_generals[slot],
            }))),
            "characterDef": p.generals[0].as_ref().map(|g| g.def as u32).unwrap_or(0),
            "alive": p.alive,
            "hp": p.hp,
            "maxHp": p.max_hp,
            "hand": match &p.hand {
                HandView::Own(cards) => serde_json::json!({ "own": cards }),
                HandView::Hidden => serde_json::json!({ "hidden": true }),
            },
            "handCount": p.hand_count,
            "equips": p.equips,
            "judge": p.judge,
            "turnedOver": p.turned_over,
            "skills": p.skills,
            "revealedGenerals": p.revealed_generals,
            "identity": identity_json(p.role),
        })).collect::<Vec<_>>(),
        "deckCount": v.deck_count,
        "discard": v.discard,
        "table": v.table,
        "cards": faces,
        "turn": v.turn,
        "phase": phase_name(v.phase),
        "round": v.round,
        // NC09/NC12 修正：终局**胜负类型**必须随视图下发（此前只发 `"finished"`，
        // 前端无法显示胜者）。此处直接序列化协议 `MatchOutcome`（如
        // `{"LoyalistWin":null}` / `{"RebelWin":null}` / `{"RenegadeWin":{"player":3}}` …）。
        "outcome": winner_to_protocol_json(&v.outcome),
        "rulesetVersion": v.ruleset_version,
    })
}

/// One authoritative engine and its planner budget, usable by native hosts.
pub struct NativeSession {
    inner: Engine,
    planner_nodes: usize,
    deadline_ms: u64,
}

impl NativeSession {
    /// 标准目录建局（无扩展；players 为座位数，seed 为随机流种子）。
    pub fn new(players: u8, seed: u64) -> Result<NativeSession, String> {
        Ok(NativeSession {
            inner: Engine::new(seed, players as usize),
            planner_nodes: 64,
            deadline_ms: 50,
        })
    }

    /// 规划预算参数化（T13 策略质量批同款；0=无规划）。
    pub fn set_planner(&mut self, nodes: usize, deadline_ms: u64) {
        self.planner_nodes = nodes;
        self.deadline_ms = deadline_ms;
    }

    pub fn revision(&self) -> u64 {
        self.inner.revision()
    }

    pub fn is_finished(&self) -> bool {
        self.inner.outcome().is_some()
    }

    /// NC05/NC09 正式选将候选（客户端选将 UI 用）。
    pub fn general_candidates_json(&self) -> String {
        general_candidates_json_for(&self.inner)
    }

    /// NC05/NC09 正式选将（按名字；失败返回错误，不静默）。
    pub fn assign_general(&mut self, seat: u8, name: &str) -> Result<String, String> {
        assign_general_for(&mut self.inner, seat, name)
    }

    /// 座位可见视图 JSON（信息隔离面：自己手牌具体 id、他人仅数量）。
    pub fn view_json(&self, seat: u8) -> Result<String, String> {
        let seats = self.inner.view_for(0).players.len();
        if seat as usize >= seats {
            return Err(String::from("view: 座位越界"));
        }
        Ok(view_to_json(&self.inner, &self.inner.view_for(seat)).to_string())
    }

    /// 当前开放选择的座位可见 JSON（N10 选择可见性口径：观察者本人含
    /// 选项；他人/观战仅存在面）。无询问返回 null。
    pub fn open_choice_json(&self, seat: u8) -> Result<String, String> {
        use engine_native::view::ChoiceView;
        Ok(match self.inner.choice_view_for(Some(seat)) {
            Some(ChoiceView::Own {
                id,
                kind,
                prompt,
                options,
                min,
                max,
                allow_cancel,
                source_skill,
            }) => {
                let mut choice = serde_json::json!({
                "id": id,
                "observer": seat,
                "prompt": prompt,
                "native_kind": if kind==engine_native::ChoiceKind::Control {None}else{Some(engine_native::view::choice_kind_name(kind))},
                "kind": if kind==engine_native::ChoiceKind::Control{engine_api::ChoiceKind::Custom}else if prompt == "trigger_order" { engine_api::ChoiceKind::OrderTriggers } else { engine_api::choice_kind_from_native(engine_native::view::choice_kind_name(kind)).expect("canonical native choice kind") },
                "min": min,
                "max": max,
                "allowCancel": allow_cancel,
                "options": options.iter().map(|o| match o {
                    engine_native::ChoiceOption::HiddenCard(_)=>serde_json::json!("hand"),
                    engine_native::ChoiceOption::Control(option)=>serde_json::json!(option.label()),
                    engine_native::ChoiceOption::Confirm => serde_json::json!("confirm"),
                    engine_native::ChoiceOption::Pass => serde_json::json!("pass"),
                    engine_native::ChoiceOption::Card(id) => serde_json::json!({ "card": id }),
                    engine_native::ChoiceOption::Player(p) => serde_json::json!({ "player": p }),
                    engine_native::ChoiceOption::Tag(t) => serde_json::json!({ "tag": t }),
                    engine_native::ChoiceOption::Skill(s) => serde_json::json!({ "skill": self.inner.catalog().skill_name(*s) }),
                    engine_native::ChoiceOption::Kind(k) => serde_json::json!({ "kind": k }),
                }).collect::<Vec<_>>(),
                });
                if let Some(source) = source_skill {
                    choice["source_skill"] = serde_json::json!(source);
                }
                choice.to_string()
            }
            Some(ChoiceView::Other {
                id,
                observer,
                prompt,
            }) => serde_json::json!({
                "id": id,
                "observer": observer,
                "prompt": prompt,
                "hidden": true,
            })
            .to_string(),
            None => "null".to_string(),
        })
    }

    /// 阶段推进（当前回合玩家）。
    pub fn next_phase(&mut self, actor: u32) -> Result<String, String> {
        let Ok(actor) = u8::try_from(actor) else {
            return Ok(rejected(&self.inner, RejectReason::UnknownActor));
        };
        let out = self.inner.submit(engine_native::GameCommand {
            actor,
            expected_revision: self.inner.revision(),
            dedup_id: self.inner.revision().wrapping_mul(2).max(1),
            action: GameAction::NextPhase,
        });
        Ok(outcome_json(&self.inner, &out))
    }

    /// 出牌（手牌 id + 目标座位）。
    pub fn play_card(
        &mut self,
        actor: u32,
        card: u32,
        targets: Vec<u32>,
    ) -> Result<String, String> {
        let Ok(actor) = u8::try_from(actor) else {
            return Ok(rejected(&self.inner, RejectReason::UnknownActor));
        };
        let Ok(targets) = targets.into_iter().map(u8::try_from).collect() else {
            return Ok(rejected(&self.inner, RejectReason::IllegalTargets));
        };
        let out = self.inner.submit(engine_native::GameCommand {
            actor,
            expected_revision: self.inner.revision(),
            dedup_id: self.inner.revision().wrapping_mul(2).max(1),
            action: GameAction::PlayCard { card, targets },
        });
        Ok(outcome_json(&self.inner, &out))
    }

    /// NC03 权威动作查询：该座位此刻**可提交**的候选动作 JSON。
    ///
    /// 候选由内核经与正式提交相同的校验产出（`Engine::legal_actions`），
    /// 顺序稳定（应答 → 主动技 → 出牌 → 阶段推进），上限 256；`actions`
    /// 每项与 TS `LegalAction` 同形，可直接用于界面并原样回传。
    ///
    /// 代价：实测约 1.6 ms/次（5 人局，含克隆校验），**按需调用**——前端
    /// 只在构建信封（用户动作/观察刷新）时查一次，并按 `revision` 记忆化。
    pub fn legal_actions_json(&self, seat: u8) -> Result<String, String> {
        legal_actions_json_for(&self.inner, seat)
    }

    /// 主动技发动（`skill` 取自权威查询的 `ActivateSkill` 候选）。
    ///
    /// 内核 `GameAction::ActivateSkill` 不接受目标：技能内部的选择走开放
    /// 选择应答，因此窄壳也不接目标参数（前端传目标即协议误用）。
    pub fn activate_skill(&mut self, actor: u32, skill: u16) -> Result<String, String> {
        let Ok(actor) = u8::try_from(actor) else {
            return Ok(rejected(&self.inner, RejectReason::UnknownActor));
        };
        let out = self.inner.submit(engine_native::GameCommand {
            actor,
            expected_revision: self.inner.revision(),
            dedup_id: self.inner.revision().wrapping_mul(2).max(1),
            action: GameAction::ActivateSkill { skill },
        });
        Ok(outcome_json(&self.inner, &out))
    }

    /// 选择应答（`selection` 是**候选下标**，直取 `spec.options[idx]`：与
    /// [`NativeSession::legal_actions_json`] 的 `ResolveChoice.selections` 同口径；
    /// 空数组 = 取消）。
    pub fn answer(
        &mut self,
        observer: u32,
        choice_id: u64,
        selection: Vec<u32>,
    ) -> Result<String, String> {
        let Ok(actor) = u8::try_from(observer) else {
            return Ok(rejected(&self.inner, RejectReason::UnknownActor));
        };
        if actor as usize >= self.inner.state_core().players.len() {
            return Ok(rejected(&self.inner, RejectReason::UnknownActor));
        }
        let Some(c) = self.inner.open_choice() else {
            return Err(String::from("answer: 无开放选择"));
        };
        if c.observer != actor {
            return Ok(rejected(&self.inner, RejectReason::WrongPlayer));
        }
        if c.id != choice_id {
            return Err(String::from("answer: choice_id 不匹配"));
        }
        let mut sel = Vec::new();
        for (idx, v) in selection.iter().enumerate() {
            let Some(o) = c.spec.options.get(*v as usize) else {
                return Err(String::from("answer: 选项索引越界"));
            };
            let _ = idx;
            sel.push(o.clone());
        }
        let out = self.inner.submit(engine_native::GameCommand {
            actor,
            expected_revision: self.inner.revision(),
            dedup_id: self.inner.revision().wrapping_mul(2).max(1),
            action: GameAction::Answer {
                choice_id: c.id,
                selection: sel,
            },
        });
        Ok(outcome_json(&self.inner, &out))
    }

    /// 一步自动推进：挂起选择按 min/首选自动应答；否则当前座位经
    /// `BoundedPlanner` 规划并应用（game-server `/native` 同款）。返回
    /// 一步摘要 JSON。
    pub fn auto_step(&mut self) -> Result<String, String> {
        if self.inner.outcome().is_some() {
            return Ok(serde_json::json!({ "done": true }).to_string());
        }
        if let Some(c) = self.inner.open_choice() {
            let obs = c.observer;
            let action = engine_native::ai::projected_choice_action(
                self.inner.ai_session(obs).choice().expect("open observer choice"),
            ).expect("observer owns the choice");
            let out = self.inner.submit(engine_native::GameCommand {
                actor: obs,
                expected_revision: self.inner.revision(),
                dedup_id: self.inner.revision().wrapping_mul(2).max(1),
                action,
            });
            return Ok(serde_json::json!({
                "kind": "answer",
                "outcome": outcome_json(&self.inner, &out),
            })
            .to_string());
        }
        let seat = self.inner.turn().current;
        let planner = BoundedPlanner::new(self.planner_nodes);
        // Zero disables the native clock so node-bounded WASM/native comparisons use the same policy.
        let planner = if self.deadline_ms == 0 {
            planner
        } else {
            planner.with_deadline_ms(self.deadline_ms)
        };
        let plan = self.inner.plan_action(seat, &planner);
        match self.inner.apply_plan(&plan) {
            Ok(out) => Ok(serde_json::json!({
                "kind": "plan",
                "seat": seat,
                "outcome": outcome_json(&self.inner, &out),
                "finished": self.inner.outcome().is_some(),
            })
            .to_string()),
            Err(e) => Ok(serde_json::json!({
                "kind": "plan",
                "seat": seat,
                "error": match e {
                    engine_native::ai::ApplyPlanError::Stale { .. } => "stale",
                    engine_native::ai::ApplyPlanError::Cancelled => "cancelled",
                },
            })
            .to_string()),
        }
    }

    /// 权威状态 hash（32 字节 hex）。只读投影，不推进修订号。
    pub fn state_hash_hex(&self) -> String {
        hex_encode(&self.inner.state_hash())
    }

    /// 当前事件日志长度（宿主据此记录已消费位置）。
    pub fn log_len(&self) -> u32 {
        self.inner.log().len().min(u32::MAX as usize) as u32
    }

    /// 自 `start` 起的公开事件码 JSON 数组；不含牌 id 或选项内容。
    pub fn public_event_codes_json(&self, start: u32) -> String {
        serde_json::to_string(&self.inner.public_event_codes_from(start as usize))
            .unwrap_or_else(|_| "[]".into())
    }

    /// 存档十六进制（NGSA v65 字节面）。
    pub fn save_hex(&self) -> String {
        hex_encode(&engine_native::archive::encode_archive(&self.inner.save()))
    }

    /// 存档恢复（标准目录）。扩展局须走 [`restore_hex_with_packs_json`]。
    ///
    /// 先在局部变量完成解码与恢复，全部成功后才替换当前引擎；任何坏输入
    /// 都只返回受控错误，原句柄、原局与后续 `save_hex`/命令仍可用。
    pub fn restore_hex(&mut self, hex: &str) -> Result<(), String> {
        self.restore_hex_checked(hex)
    }

    /// [`restore_hex`] 的宿主可测版本（不构造 `JsValue`）。
    pub fn restore_hex_checked(&mut self, hex: &str) -> Result<(), String> {
        let bytes = hex_decode(hex)?;
        let snapshot =
            engine_native::archive::decode_archive(&bytes).map_err(|e| format!("{e:?}"))?;
        let restored = Engine::restore(&snapshot).map_err(|e| format!("{e:?}"))?;
        self.inner = restored;
        Ok(())
    }

    /// **身份局**建局（NC09 身份局端到端：浏览器侧需要非原型模式）。
    ///
    /// 与 [`new`] 的区别只在 `GameMode`：身份局由 `Engine::new_identity` 装配
    /// （主公/忠臣/反贼/内奸），胜负结果走 `Winner::{LordSide, Rebels, Traitor}`。
    pub fn new_identity(players: u8, seed: u64) -> Result<NativeSession, String> {
        Self::new_identity_with_packs_json(players, seed, "[]")
    }

    /// 身份局与扩展一起装载；身份和扩展初始化使用同一规则随机流。
    pub fn new_identity_with_packs_json(
        players: u8,
        seed: u64,
        packs_json: &str,
    ) -> Result<NativeSession, String> {
        Ok(NativeSession {
            inner: engine_identity_from_packs(players, seed, packs_json)?,
            planner_nodes: 64,
            deadline_ms: 50,
        })
    }

    /// 已安装扩展载荷数组建局。空数组与 [`new`] 相同。
    pub fn new_with_packs_json(
        players: u8,
        seed: u64,
        packs_json: &str,
    ) -> Result<NativeSession, String> {
        let inner = engine_from_packs(players, seed, packs_json)?;
        Ok(NativeSession {
            inner,
            planner_nodes: 64,
            deadline_ms: 50,
        })
    }

    /// 用同一组安装载荷恢复扩展局。包集合或版本不符时拒绝，不替换当前对局。
    pub fn restore_hex_with_packs_json(
        &mut self,
        hex: &str,
        packs_json: &str,
    ) -> Result<(), String> {
        self.inner = engine_from_archive(hex, packs_json)?;
        Ok(())
    }

    /// 目录中牌名以前缀开头的张数（扩展牌并入对局的可观察计数）。
    pub fn cards_with_prefix(&self, prefix: &str) -> u32 {
        prefix_count(&self.inner, prefix)
    }
}
