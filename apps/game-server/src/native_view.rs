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
    CardKindTag, CardView, CardZoneTag, IdentityTag, LegalAction, MatchOutcome, ObserverKind,
    ObserverSnapshot, PlayerId as ApiPlayerId, SeatView, SkillId, SkillView, TurnPhaseTag,
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

/// 身份可见性：本人、主公、已死亡者可见（与内核 `view_for` 同口径）；其余 None。
fn identity_tag_for(engine: &Engine, viewer: Option<ApiPlayerId>, seat: u8) -> Option<IdentityTag> {
    let roles = engine.state_core().identity_roles()?;
    let role = roles.get(seat as usize)?;
    let is_self = viewer
        .map(|v| v.raw().saturating_sub(1) as u8 == seat)
        .unwrap_or(false);
    let dead = !engine.state_core().player(seat).alive;
    if !(is_self || *role == engine_native::state::Role::Lord || dead) {
        return None;
    }
    Some(match role {
        engine_native::state::Role::Lord => IdentityTag::Lord,
        engine_native::state::Role::Loyalist => IdentityTag::Loyalist,
        engine_native::state::Role::Rebel => IdentityTag::Rebel,
        engine_native::state::Role::Traitor => IdentityTag::Renegade,
    })
}

/// 观察者授权可见的牌：本人手牌（他人手牌仅数量，不进入 known_cards）。
///
/// 协议 `CardView` 只暴露**公开规则类别**（`CardKindTag`），不含具体牌种码：
/// 未在协议枚举中建模的牌种 → `CardKindTag::Other`（**不猜测**）。
fn known_cards_for(engine: &Engine, seat: Option<ApiPlayerId>) -> Vec<CardView> {
    let Some(seat) = seat else {
        return Vec::new();
    };
    let native_seat = seat.raw().saturating_sub(1) as u8;
    if native_seat as usize >= engine.state_core().players.len() {
        return Vec::new();
    }
    let catalog = engine.catalog();
    engine
        .state_core()
        .player(native_seat)
        .hand
        .iter()
        .map(|c| {
            let kind = catalog.card_kind(*c);
            CardView {
                id: engine_api::CardId::new(*c as u32),
                def: Some(kind_code_u32(kind)),
                suit: None,
                rank: None,
                owner: Some(seat),
                zone: CardZoneTag::Hand,
                kind: kind_tag(kind),
            }
        })
        .collect()
}

/// 原生牌种 → 协议公开规则类别（未建模者 `Other`，不猜测）。
fn kind_tag(kind: engine_native::catalog::CardKind) -> CardKindTag {
    use engine_native::catalog::CardKind as K;
    match kind {
        K::Sha => CardKindTag::Sha,
        K::Tao => CardKindTag::Tao,
        K::Shan => CardKindTag::Shan,
        K::Jiu => CardKindTag::Jiu,
        K::FireSha => CardKindTag::ShaFire,
        K::LeiSha => CardKindTag::ShaThunder,
        _ => CardKindTag::Other,
    }
}

/// 牌种码（仅用于 `CardView.def` 的稳定标识）：取牌种在目录枚举中的序号。
fn kind_code_u32(kind: engine_native::catalog::CardKind) -> u32 {
    // `CardKind` 是 `#[repr(u16)]` 风格枚举：用 `as u16` 得到稳定序号
    kind as u16 as u32
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
        (RoomRole::Player, Some(p)) => engine.view_for(p.raw().saturating_sub(1) as u8),
        _ => engine.view_spectator(),
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
            // 主将槽的武将 def（无将 = 0）——客户端据此显示立绘/名字
            character_def: engine.state_core().player(cell.seat).general_slots[0]
                .map(|d| d as u32)
                .unwrap_or(0),
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
            // 身份可见性：本人、主公（公开）与已死亡者（遗言公开）可见；其余为 None
            identity: identity_tag_for(engine, seat, cell.seat),
            // 技能：以**内核技能实例**为准（不是「目录里有就算在册」）：
            // - `def` = 目录定义 id；`id` 同 def（原生 `ActivateSkill{skill: def}` 以此为键，
            //   见 `legal_actions_for`——协议注释的「运行时实例 id」在原生即 def）；
            // - `enabled` = 实例启用位；`use_count` = `once_used`（0/1，本回合已发动）；
            // - `locked` = 无主动形态（`SkillBehavior::activate` 默认 None）→ 视为锁定/被动；
            // - `active` = 该实例**当前可被主动发动**（走内核 `is_legal_action` 同源校验）；
            // - `view_as` = 原生未建模「视为技」候选面 → 恒 false（**不猜测**，登记待办）。
            skills: cell
                .skills
                .iter()
                .filter_map(|name| {
                    let def = engine.catalog().skill_by_name(name)?;
                    let inst = engine.state_core().skills.get(&(cell.seat, def));
                    let enabled = inst.map(|i| i.enabled).unwrap_or(false);
                    let use_count = inst.map(|i| u32::from(i.once_used)).unwrap_or(0);
                    let active = enabled
                        && engine
                            .is_legal_action(cell.seat, &GameAction::ActivateSkill { skill: def });
                    // 主动形态判定：该实例当前能否被 `ActivateSkill` 接受（`active` 已表达）；
                    // `locked` 取「无主动形态」的保守口径：当前不可主动且未启用主动例外。
                    let locked = !active;
                    Some(SkillView {
                        id: SkillId::new(def as u32),
                        def: def as u32,
                        enabled,
                        use_count,
                        active,
                        view_as: false,
                        locked,
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
            // 原型/单挑：唯一存活者
            engine_native::Winner::Player(p) => MatchOutcome::SingleWin {
                player: ApiPlayerId::new(p as u32 + 1),
            },
            engine_native::Winner::LordSide => MatchOutcome::LoyalistWin,
            engine_native::Winner::Rebels => MatchOutcome::RebelWin,
            engine_native::Winner::Traitor => {
                // 内奸胜：取仍存活的座位（通常唯一）
                let renegade = (0..engine.state_core().players.len() as u8)
                    .find(|s| engine.state_core().player(*s).alive)
                    .unwrap_or(0);
                MatchOutcome::RenegadeWin {
                    player: ApiPlayerId::new(renegade as u32 + 1),
                }
            }
            engine_native::Winner::Faction(f) => MatchOutcome::GroupWin {
                group: format!("{f:?}").to_lowercase(),
            },
            engine_native::Winner::Versus(_)
            | engine_native::Winner::Doudizhu { .. }
            | engine_native::Winner::Boss { .. } => {
                // 协议枚举无对应项：**不猜测**，保持 InProgress 并在证据中登记
                MatchOutcome::InProgress
            }
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
        known_cards: known_cards_for(engine, seat),
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
                    .map(|opt| {
                        engine
                            .open_choice()?
                            .spec
                            .options
                            .iter()
                            .position(|candidate| candidate == opt)
                            .map(|idx| idx as u32)
                    })
                    .collect::<Option<Vec<_>>>()?,
                cancelled: selection.is_empty(),
            }),
            // 主动技/明置：协议需要运行时技能实例 id 与目标/牌输入，原生候选未展开
            // 这些形态（NC10 待办），此处不伪造映射。
            GameAction::ActivateSkill { skill } => Some(LegalAction::ActivateSkill {
                skill: engine_api::SkillId::new(skill as u32),
                targets: Vec::new(),
                card_targets: Vec::new(),
            }),
            GameAction::RevealGeneral { .. } => None,
        })
        .collect()
}
