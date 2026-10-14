//! engine-native 的 wasm_bindgen 窄壳（T13 客户端产品面切片）。
//!
//! 设计：**窄 API 而非通用 JSON 协议**——旧线 `engine_wasm` 的投影 schema
//! 与原生不兼容，为避免给原生全类型补 serde，本壳只暴露客户端驱动的最小
//! 面：标准目录建局、窄动作（推进/出牌/选择应答）、座位可见视图与开放
//! 选择的 JSON 投影（手写 `serde_json::json!`，`PlayerView`/`ChoiceSpec`
//! 字段量可控）、`auto_step`（`BoundedPlanner` 规划驱动 + 挂起选择自动
//! 应答，game-server `/native` 同款）、存档十六进制往返（NGSA v65）。
//!
//! 扩展包装载面（对等旧线 loadContentBundle）暂缓：原生包装载走 npack/
//! 包存储二进制面，wasm 侧接 C0 装载器归后续批（登记于证据）。
//!
//! 视图投影为纯只读面，不进权威状态与哈希（ADR 0730）。

use engine_native::ai::BoundedPlanner;
use engine_native::event::Phase;
use engine_native::view::{HandView, PlayerView};
use engine_native::{CommandOutcome, Engine, GameAction};
use wasm_bindgen::prelude::*;

/// 十六进制编解码（无 base64 依赖；存档字节面）。
fn hex_encode(bytes: &[u8]) -> String {
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
fn outcome_json(engine: &Engine, out: &CommandOutcome) -> String {
    serde_json::json!({
        "outcome": match out {
            CommandOutcome::Accepted { .. } => "accepted",
            CommandOutcome::Pending { .. } => "pending",
            CommandOutcome::Rejected(_) => "rejected",
            CommandOutcome::Suspended => "suspended",
        },
        "revision": engine.revision(),
        "finished": engine.outcome().is_some(),
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
            Some(ChoiceView::Own { id, prompt, options, min, max, allow_cancel }) => {
                serde_json::json!({
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
                    }).collect::<Vec<_>>(),
                })
                .to_string()
            }
            Some(ChoiceView::Other { id, observer, prompt }) => serde_json::json!({
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

    /// 选择应答（选项索引直传：confirm/pass=0；card/player/tag=载荷值）。
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
        let planner =
            BoundedPlanner::new(self.planner_nodes).with_deadline_ms(self.deadline_ms);
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

    /// 存档恢复（标准目录；扩展包恢复走 restore_with_extensions 归后续批）。
    pub fn restore_hex(&mut self, hex: &str) -> Result<(), JsValue> {
        let bytes = hex_decode(hex)?;
        let snapshot = engine_native::archive::decode_archive(&bytes).map_err(err)?;
        self.inner = Engine::restore(&snapshot).map_err(err)?;
        Ok(())
    }
}
