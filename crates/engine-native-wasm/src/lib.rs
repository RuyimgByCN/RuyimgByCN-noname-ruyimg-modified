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
//! cards.json 对象数组，经 C0 装载器桥接后走 `new_with_extensions` /
//! `restore_with_extensions`。空数组等于标准目录。
//!
//! 视图投影为纯只读面，不进权威状态与哈希（ADR 0730）。

use engine_native::ai::BoundedPlanner;
use engine_native::content::extension::ExtensionPack;
use engine_native::event::Phase;
use engine_native::view::{HandView, PlayerView};
use engine_native::{CommandOutcome, Engine, GameAction};
use wasm_bindgen::prelude::*;

/// `packs_json` 是 cards.json 对象的 JSON 数组。空数组 = 标准目录。
fn packs_from_json(packs_json: &str) -> Result<Vec<ExtensionPack>, String> {
    let value: serde_json::Value =
        serde_json::from_str(packs_json).map_err(|e| format!("packs JSON: {e}"))?;
    let arr = value
        .as_array()
        .ok_or_else(|| "packs 须为 JSON 数组".to_string())?;
    let mut out = Vec::with_capacity(arr.len());
    for item in arr {
        let text = serde_json::to_string(item).map_err(|e| format!("packs 元素: {e}"))?;
        let bridged = content_native_loader::bridge_pack(&text).map_err(|e| format!("{e:?}"))?;
        out.push(bridged.pack);
    }
    Ok(out)
}

pub fn engine_from_packs(players: u8, seed: u64, packs_json: &str) -> Result<Engine, String> {
    let packs = packs_from_json(packs_json)?;
    let refs: Vec<&ExtensionPack> = packs.iter().collect();
    if refs.is_empty() {
        Ok(Engine::new(seed, players as usize))
    } else {
        Engine::new_with_extensions(seed, players as usize, &refs).map_err(|e| format!("{e:?}"))
    }
}

pub fn engine_from_archive(hex: &str, packs_json: &str) -> Result<Engine, String> {
    let packs = packs_from_json(packs_json)?;
    let refs: Vec<&ExtensionPack> = packs.iter().collect();
    let bytes = hex_decode(hex).map_err(|e| format!("{e:?}"))?;
    let snapshot =
        engine_native::archive::decode_archive(&bytes).map_err(|e| format!("解码失败: {e:?}"))?;
    if refs.is_empty() {
        Engine::restore(&snapshot).map_err(|e| format!("恢复失败: {e:?}"))
    } else {
        Engine::restore_with_extensions(&snapshot, &refs).map_err(|e| format!("恢复失败: {e:?}"))
    }
}

fn prefix_count(engine: &Engine, prefix: &str) -> u32 {
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

fn hex_decode(s: &str) -> Result<Vec<u8>, JsValue> {
    (0..s.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&s[i..i + 2], 16)
                .map_err(|e| JsValue::from_str(&format!("restore: 非法十六进制: {e}")))
        })
        .collect()
}

fn err<E: std::fmt::Debug>(e: E) -> JsValue {
    JsValue::from_str(&format!("{e:?}"))
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
        GameAction::RevealGeneral { .. } => None,
    }
}

/// NC03 权威动作查询的 JSON 投影：`{"seat", "revision", "actions"}`。
///
/// 不经 `wasm_bindgen`，宿主测试（`tests/nc03_legal_actions_export.rs`）可直接
/// 调用，避免走到 `JsValue`。`revision` 是候选成立的状态修订号：客户端可据此
/// 判断候选是否过期（`Engine::legal_actions` 只读，不改状态）。
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

/// 座位视图投影（`PlayerView` 手写 JSON；视图为只读投影不进哈希）。
fn view_to_json(v: &PlayerView) -> serde_json::Value {
    serde_json::json!({
        "seat": v.seat,
        "players": v.players.iter().map(|p| serde_json::json!({
            "seat": p.seat,
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
        })).collect::<Vec<_>>(),
        "deckCount": v.deck_count,
        "discard": v.discard,
        "table": v.table,
        "turn": v.turn,
        "phase": phase_name(v.phase),
        "round": v.round,
        "outcome": v.outcome.as_ref().map(|_| "finished"),
        "rulesetVersion": v.ruleset_version,
    })
}

/// 原生引擎 wasm 壳实例。
#[wasm_bindgen]
pub struct NativeEngine {
    inner: Engine,
    planner_nodes: usize,
    deadline_ms: u64,
}

#[wasm_bindgen]
impl NativeEngine {
    /// 标准目录建局（无扩展；players 为座位数，seed 为随机流种子）。
    #[wasm_bindgen(constructor)]
    pub fn new(players: u8, seed: u64) -> Result<NativeEngine, JsValue> {
        Ok(NativeEngine {
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

    /// 座位可见视图 JSON（信息隔离面：自己手牌具体 id、他人仅数量）。
    pub fn view_json(&self, seat: u8) -> Result<String, JsValue> {
        let seats = self.inner.view_for(0).players.len();
        if seat as usize >= seats {
            return Err(JsValue::from_str("view: 座位越界"));
        }
        Ok(view_to_json(&self.inner.view_for(seat)).to_string())
    }

    /// 当前开放选择的座位可见 JSON（N10 选择可见性口径：观察者本人含
    /// 选项；他人/观战仅存在面）。无询问返回 null。
    pub fn open_choice_json(&self, seat: u8) -> Result<String, JsValue> {
        use engine_native::view::ChoiceView;
        Ok(match self.inner.choice_view_for(Some(seat)) {
            Some(ChoiceView::Own {
                id,
                prompt,
                options,
                min,
                max,
                allow_cancel,
            }) => serde_json::json!({
                "id": id,
                "observer": seat,
                "prompt": prompt,
                "min": min,
                "max": max,
                "allowCancel": allow_cancel,
                "options": options.iter().map(|o| match o {
                    engine_native::ChoiceOption::Confirm => serde_json::json!("confirm"),
                    engine_native::ChoiceOption::Pass => serde_json::json!("pass"),
                    engine_native::ChoiceOption::Card(id) => serde_json::json!({ "card": id }),
                    engine_native::ChoiceOption::Player(p) => serde_json::json!({ "player": p }),
                    engine_native::ChoiceOption::Tag(t) => serde_json::json!({ "tag": t }),
                    engine_native::ChoiceOption::Kind(k) => serde_json::json!({ "kind": k }),
                }).collect::<Vec<_>>(),
            })
            .to_string(),
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
    pub fn next_phase(&mut self, actor: u32) -> Result<String, JsValue> {
        let out = self.inner.submit(engine_native::GameCommand {
            actor: actor as u8,
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
    ) -> Result<String, JsValue> {
        let out = self.inner.submit(engine_native::GameCommand {
            actor: actor as u8,
            expected_revision: self.inner.revision(),
            dedup_id: self.inner.revision().wrapping_mul(2).max(1),
            action: GameAction::PlayCard {
                card,
                targets: targets.into_iter().map(|t| t as u8).collect(),
            },
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
    pub fn legal_actions_json(&self, seat: u8) -> Result<String, JsValue> {
        legal_actions_json_for(&self.inner, seat).map_err(|e| JsValue::from_str(&e))
    }

    /// 主动技发动（`skill` 取自权威查询的 `ActivateSkill` 候选）。
    ///
    /// 内核 `GameAction::ActivateSkill` 不接受目标：技能内部的选择走开放
    /// 选择应答，因此窄壳也不接目标参数（前端传目标即协议误用）。
    pub fn activate_skill(&mut self, actor: u32, skill: u16) -> Result<String, JsValue> {
        let out = self.inner.submit(engine_native::GameCommand {
            actor: actor as u8,
            expected_revision: self.inner.revision(),
            dedup_id: self.inner.revision().wrapping_mul(2).max(1),
            action: GameAction::ActivateSkill { skill },
        });
        Ok(outcome_json(&self.inner, &out))
    }

    /// 选择应答（`selection` 是**候选下标**，直取 `spec.options[idx]`：与
    /// [`NativeEngine::legal_actions_json`] 的 `ResolveChoice.selections` 同口径；
    /// 空数组 = 取消）。
    pub fn answer(
        &mut self,
        observer: u32,
        choice_id: u64,
        selection: Vec<u32>,
    ) -> Result<String, JsValue> {
        let Some(c) = self.inner.open_choice() else {
            return Err(JsValue::from_str("answer: 无开放选择"));
        };
        if c.id != choice_id {
            return Err(JsValue::from_str("answer: choice_id 不匹配"));
        }
        let mut sel = Vec::new();
        for (idx, v) in selection.iter().enumerate() {
            let Some(o) = c.spec.options.get(*v as usize) else {
                return Err(JsValue::from_str("answer: 选项索引越界"));
            };
            let _ = idx;
            sel.push(o.clone());
        }
        let obs = c.observer;
        let out = self.inner.submit(engine_native::GameCommand {
            actor: obs,
            expected_revision: self.inner.revision(),
            dedup_id: self.inner.revision().wrapping_mul(2).max(1),
            action: GameAction::Answer {
                choice_id: c.id,
                selection: sel,
            },
        });
        let _ = observer;
        Ok(outcome_json(&self.inner, &out))
    }

    /// 一步自动推进：挂起选择按 min/首选自动应答；否则当前座位经
    /// `BoundedPlanner` 规划并应用（game-server `/native` 同款）。返回
    /// 一步摘要 JSON。
    pub fn auto_step(&mut self) -> Result<String, JsValue> {
        if self.inner.outcome().is_some() {
            return Ok(serde_json::json!({ "done": true }).to_string());
        }
        if let Some(c) = self.inner.open_choice() {
            let take = c.spec.min.max(1).min(c.spec.options.len());
            let sel: Vec<_> = c
                .spec
                .options
                .iter()
                .filter(|o| matches!(o, engine_native::ChoiceOption::Card(_)))
                .take(take)
                .cloned()
                .collect();
            let sel = if sel.is_empty() {
                vec![c.spec.options[0].clone()]
            } else {
                sel
            };
            let (cid, obs) = (c.id, c.observer);
            let out = self.inner.submit(engine_native::GameCommand {
                actor: obs,
                expected_revision: self.inner.revision(),
                dedup_id: self.inner.revision().wrapping_mul(2).max(1),
                action: GameAction::Answer {
                    choice_id: cid,
                    selection: sel,
                },
            });
            return Ok(serde_json::json!({
                "kind": "answer",
                "outcome": outcome_json(&self.inner, &out),
            })
            .to_string());
        }
        let seat = self.inner.turn().current;
        let planner = BoundedPlanner::new(self.planner_nodes).with_deadline_ms(self.deadline_ms);
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

    /// 存档十六进制（NGSA v65 字节面）。
    pub fn save_hex(&self) -> String {
        hex_encode(&engine_native::archive::encode_archive(&self.inner.save()))
    }

    /// 存档恢复（标准目录）。扩展局须走 [`restore_hex_with_packs_json`]。
    pub fn restore_hex(&mut self, hex: &str) -> Result<(), JsValue> {
        let bytes = hex_decode(hex)?;
        let snapshot = engine_native::archive::decode_archive(&bytes).map_err(err)?;
        self.inner = Engine::restore(&snapshot).map_err(err)?;
        Ok(())
    }

    /// 已安装扩展的 cards.json 数组建局。空数组与 [`new`] 相同。
    pub fn new_with_packs_json(
        players: u8,
        seed: u64,
        packs_json: &str,
    ) -> Result<NativeEngine, JsValue> {
        let inner =
            engine_from_packs(players, seed, packs_json).map_err(|e| JsValue::from_str(&e))?;
        Ok(NativeEngine {
            inner,
            planner_nodes: 64,
            deadline_ms: 50,
        })
    }

    /// 用同一组 cards.json 恢复扩展局。包集合或版本不符时拒绝，不替换当前对局。
    pub fn restore_hex_with_packs_json(
        &mut self,
        hex: &str,
        packs_json: &str,
    ) -> Result<(), JsValue> {
        self.inner = engine_from_archive(hex, packs_json).map_err(|e| JsValue::from_str(&e))?;
        Ok(())
    }

    /// 目录中牌名以前缀开头的张数（扩展牌并入对局的可观察计数）。
    pub fn cards_with_prefix(&self, prefix: &str) -> u32 {
        prefix_count(&self.inner, prefix)
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
    pub use super::{engine_from_archive, engine_from_packs, hex_encode};
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
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../packages/extensions/shousha-mvp-upstream-liuli/assets/config.json",
        );
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
        assert!(ok.starts_with("kind:zip | default:我的扩展 | prefix:我的扩展/ |"), "{ok}");
        assert!(ok.contains("dir:card,image |"), "{ok}");
        assert!(ok.ends_with("| err:- | eval:0 | fetch:0 | write:0 | dom:0"), "{ok}");
        let seven = drag_read_apply(&config, r#"{"name":"我的扩展.7z","mime":""}"#);
        assert!(seven.starts_with("kind:7z |"), "{seven}");
        assert!(seven.contains("err:还不支持读取7z格式的扩展"), "{seven}");
        let native = drag_read_apply(
            &config,
            r#"{"name":"boss.zip","mime":"application/zip","extname":"boss","files":["boss/extension.js"]}"#,
        );
        assert!(native.contains("err:禁止安装游戏原生扩展（boss）"), "{native}");
        let single = drag_read_apply(&config, r#"{"name":"extension.js","mime":""}"#);
        assert!(single.starts_with("kind:extension.js |"), "{single}");
        assert!(single.contains("err:extension.js代码有错误，无法导入！"), "{single}");
        assert!(drag_read_apply(&config, "not json").starts_with("error:BadConfig"));
        assert!(drag_read_apply(&config.replace("\"source\": \"noname-liuli\"", "\"source\": \"\""), "{}")
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
            let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join(format!("../../packages/extensions/{dir}/assets/config.json"));
            let config = std::fs::read_to_string(&path).expect("读 boss play config.json");
            assert!(config.contains(&format!("\"source\": \"{source}\"")));
            assert!(config.contains("挑战武将") && config.contains("剑阁武将"));
            assert!(!config.contains("localStorage"));
            assert!(!config.contains("eval("));
            let line = boss_play_apply(&config, req);
            assert!(line.starts_with("skipped:0 | boss:2 | jiange:1 | chars:3 | forbidai:0 |"), "{line}");
            assert!(line.contains("allowed_jiange:1 | skills:2 |"), "{line}");
            let tafang = boss_play_apply(&config, r#"{"mode":"tafang"}"#);
            assert!(tafang.starts_with("skipped:1 | boss:0 | jiange:0 |"), "{tafang}");
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
        assert!(second.contains("first:1 | side:1 | zhuSeat:0 | fanSeat:1"), "{second}");
        assert!(second.contains("friend:a,b,c,e,f | enemy:j,i,h,d,g"), "{second}");
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
