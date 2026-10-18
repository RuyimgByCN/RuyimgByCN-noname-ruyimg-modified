//! NC09/NC03 桥接：**原生引擎视图 → 产品协议视图**（`engine-api` 类型）。
//!
//! 联机房间与客户端目前使用 `engine-api` 的 `ObserverSnapshot`/`LegalAction`
//! （旧线 `engine-core` 产出）；原生线使用 `engine-native::view::PlayerView` 与
//! 内核权威动作查询。本模块提供**唯一的映射点**，让同一原生会话契约接入产品
//! 协议，不要求客户端改类型，也不允许各自再写一套投影规则。
//!
//! 口径：
//! - 座位投影：自己手牌可见、他人仅数量（原生 `HandView` 已保证），映射后
//!   `SeatView.hand_count` 仍只给数量——**协议视图不含手牌 id**（授权面例外见待办）；
//! - 观察者：`Spectator`/`Referee` 由角色决定（`Referee` 仍走同一原生投影，
//!   原生视图本身不含他人手牌内容）；
//! - 权威动作：`Engine::legal_actions`（内核校验同源）→ `LegalAction`；
//! - 摘要：`Engine::state_hash`（规范摘要）逐字节带上，供客户端比对修订。

use engine_api::{
    LegalAction, MatchOutcome, ObserverKind, ObserverSnapshot, PlayerId as ApiPlayerId, SeatView,
    SkillId, SkillView, TurnPhaseTag,
};
use engine_native::view::{HandView, PlayerView};
use engine_native::{Engine, GameAction};

/// 房间角色 → 协议观察者种类（座位号由 `seat` 给出）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoomRole {
    /// 入座玩家。
    Player,
    /// 观战者（公开信息）。
    Spectator,
    /// 裁判（完整信息角色；原生投影仍不暴露他人手牌内容）。
    Referee,
}

/// 原生 `Phase` → 协议 `TurnPhaseTag`。
fn phase_tag(p: engine_native::Phase) -> TurnPhaseTag {
    match p {
        engine_native::Phase::Judge => TurnPhaseTag::Judge,
        engine_native::Phase::Draw => TurnPhaseTag::Draw,
        engine_native::Phase::Play => TurnPhaseTag::Play,
        engine_native::Phase::Discard => TurnPhaseTag::Discard,
        engine_native::Phase::End => TurnPhaseTag::End,
    }
}

/// 生成该观察者的协议快照（唯一映射点）。
pub fn observer_snapshot_for(
    engine: &Engine,
    role: RoomRole,
    seat: Option<ApiPlayerId>,
) -> ObserverSnapshot {
    // 原生视图：Player 用座位投影；观战/裁判用「无座位」投影（不含任何手牌 id）
    let view: PlayerView = match (role, seat) {
        (RoomRole::Player, Some(p)) => engine.view_for(p.raw() as u8),
        _ => engine.view_for(u8::MAX),
    };
    let observer = match role {
        RoomRole::Player => ObserverKind::Player(seat.unwrap_or(ApiPlayerId::NONE)),
        RoomRole::Spectator => ObserverKind::Spectator,
        RoomRole::Referee => ObserverKind::Referee,
    };
    let seats: Vec<SeatView> = view
        .players
        .iter()
        .map(|cell| SeatView {
            player: ApiPlayerId::new(cell.seat as u32 + 1),
            character_def: 0,
            seat: engine_api::SeatIndex::new(cell.seat),
            hp: cell.hp,
            max_hp: cell.max_hp,
            alive: cell.alive,
            turned_over: cell.turned_over,
            // 手牌**仅数量**（自己与公开区的手牌内容不进入协议视图）
            hand_count: match &cell.hand {
                HandView::Own(cards) => cards.len() as u32,
                HandView::Hidden => cell.hand_count as u32,
            },
            identity: None,
            // 技能：按名字回查目录定义 id（协议 `SkillView` 需要 def id）
            skills: cell
                .skills
                .iter()
                .filter_map(|name| {
                    engine.catalog().skill_by_name(name).map(|def| {
                        SkillView {
                            id: SkillId::new(def as u32),
                            def: def as u32,
                            enabled: true,
                            use_count: 0,
                            active: false,
                            view_as: false,
                            locked: false,
                        }
                    })
                })
                .collect(),
        })
        .collect();
    // 原生 outcome 只有胜者座位；协议 `MatchOutcome` 为身份局语义枚举。
    // 未终局 → InProgress；终局 → 按原生胜者映射（非身份局按 Loyalist/Rebel 语义
    // 无法判定时用 `LoyalistWin` 占位是**不诚实**的，故统一用 `InProgress` 之外
    // 的可表达项：见下方 TODO 注释与证据 README「未覆盖」。
    let outcome = match engine.outcome() {
        None => MatchOutcome::InProgress,
        Some(o) => match o.winner {
            engine_native::Winner::Player(p) => {
                // 座位 → 玩家 id（1 基）；身份局语义待 NC09 接线时按模式映射
                let _ = p;
                MatchOutcome::InProgress
            }
            _ => MatchOutcome::InProgress,
        },
    };
    ObserverSnapshot {
        revision: engine.revision(),
        state_hash: engine.state_hash(),
        observer,
        round: view.round,
        current_player: ApiPlayerId::new(view.turn as u32 + 1),
        phase: phase_tag(view.phase),
        seats,
        known_cards: Vec::new(),
        outcome,
    }
}

/// 原生开放选择 → 协议 `ChoiceRequest`（**选项仅观察者可见**：非观察者返回 `None`，
/// 与内核 `choice_view_for` 同口径；观战/裁判返回 `None`，不泄漏选项内容）。
pub fn choice_for(engine: &Engine, seat: Option<ApiPlayerId>) -> Option<engine_api::ChoiceRequest> {
    let seat = seat?;
    let native_seat = seat.raw().saturating_sub(1) as u8;
    let open = engine.open_choice()?;
    if open.observer != native_seat {
        return None;
    }
    // 协议 `ChoiceOption` 是 `{token, label_key}` 二元组（token 即 ResolveChoice 的
    // 选择值；label_key 只用于本地化/显示，不参与规则）。token 与原生选项一一对应：
    // 使用**候选下标**（与前端适配层/`answer()` 同口径），label_key 保留语义标签。
    let options: Vec<engine_api::ChoiceOption> = open
        .spec
        .options
        .iter()
        .enumerate()
        .map(|(idx, opt)| engine_api::ChoiceOption {
            token: idx as u32,
            label_key: match opt {
                engine_native::ChoiceOption::Confirm => "confirm".to_string(),
                engine_native::ChoiceOption::Pass => "pass".to_string(),
                engine_native::ChoiceOption::Card(c) => format!("card:{c}"),
                engine_native::ChoiceOption::Player(p) => format!("player:{}", *p as u32 + 1),
                engine_native::ChoiceOption::Tag(t) => format!("tag:{t}"),
                engine_native::ChoiceOption::Kind(k) => format!("kind:{k}"),
            },
        })
        .collect();
    Some(engine_api::ChoiceRequest {
        choice_id: engine_api::ChoiceId::new(open.id as u32),
        observer: seat,
        options,
        continuation_id: open.id,
        // 原生 `ChoiceKind` 未映射为协议语义枚举时统一 `Custom`（**不猜测**具体语义）
        kind: engine_api::ChoiceKind::Custom,
        min_count: open.spec.min as u8,
        max_count: open.spec.max as u8,
        allow_cancel: open.spec.allow_cancel,
        timeout_policy: engine_api::TimeoutPolicy::None,
        revision: engine.revision(),
    })
}

/// 权威动作 → 协议动作（只映射本内核可表达的形态；未覆盖形态见待办）。
pub fn legal_actions_for(engine: &Engine, seat: ApiPlayerId) -> Vec<LegalAction> {
    let seat_u8 = seat.raw().saturating_sub(1) as u8;
    engine
        .legal_actions(seat_u8)
        .into_iter()
        .filter_map(|a| match a {
            GameAction::NextPhase => Some(LegalAction::EndPhase),
            GameAction::PlayCard { card, targets } => Some(LegalAction::PlayCard {
                card: engine_api::CardId::new(card as u32),
                targets: targets
                    .iter()
                    .map(|t| ApiPlayerId::new(*t as u32 + 1))
                    .collect(),
            }),
            GameAction::Answer {
                choice_id,
                selection,
            } => Some(LegalAction::ResolveChoice {
                choice_id: engine_api::ChoiceId::new(choice_id as u32),
                selections: selection
                    .iter()
                    .map(|opt| match opt {
                        engine_native::ChoiceOption::Pass => u32::MAX,
                        engine_native::ChoiceOption::Tag(t) => *t as u32,
                        engine_native::ChoiceOption::Card(c) => *c as u32,
                        engine_native::ChoiceOption::Player(p) => *p as u32 + 1,
                        engine_native::ChoiceOption::Kind(k) => *k as u32,
                        engine_native::ChoiceOption::Confirm => u32::MAX - 1,
                    })
                    .collect(),
                cancelled: selection.is_empty(),
            }),
            // 主动技/明置：协议需要运行时技能实例 id 与目标/牌输入，原生候选未展开
            // 这些形态（NC10 待办），此处不伪造映射。
            GameAction::ActivateSkill { .. } | GameAction::RevealGeneral { .. } => None,
        })
        .collect()
}
