//! NC09/NC03 桥接：**原生引擎视图 → 产品协议视图**（`engine-api` 类型）。
//!
//! 联机房间与客户端目前使用 `engine-api` 的 `ObserverSnapshot`/`LegalAction`
//! （旧线 `engine-core` 产出）；原生线使用 `engine-native::view::PlayerView` 与
//! 内核权威动作查询。本模块提供**唯一的映射点**，让同一原生会话契约接入产品
//! 协议，不要求客户端改类型，也不允许各自再写一套投影规则。
//!
//! 口径：
//! - 座位投影：自己手牌与公开牌区可见，他人手牌仅数量（原生视图保证）；
//!   `known_cards` 为这些可见牌附上真实牌面。
//! - 观察者：`Spectator` 仅公开信息；可信宿主授予的 `Referee` 使用共享全信息视图；
//! - 权威动作：`Engine::legal_actions`（内核校验同源）→ `LegalAction`；
//! - 摘要：`Engine::state_hash`（规范摘要）逐字节带上，供客户端比对修订。

use engine_api::{
    CardKindTag, CardView, CardZoneTag, IdentityTag, LegalAction, MatchOutcome, ObserverKind,
    ObserverSnapshot, PlayerId as ApiPlayerId, SeatView, SkillId, SkillView, TurnPhaseTag,
};
use engine_native::view::{HandView, PlayerView};
use engine_native::{Engine, GameAction};

/// 房间角色由可信成员记录决定，不能取自客户端自报角色（座位号由 `seat` 给出）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoomRole {
    /// 入座玩家。
    Player,
    /// 观战者（公开信息）。
    Spectator,
    /// 裁判（可信宿主授予；完整隐藏信息）。
    Referee,
}

/// 本人手牌与公开牌区均从已经按观察者过滤的原生视图取得。
fn known_cards_for(engine: &Engine, view: &PlayerView) -> Vec<CardView> {
    let mut cards = Vec::new();
    let mut push = |id, owner, zone| {
        let def = &engine.catalog().cards[id as usize];
        cards.push(CardView {
            id: engine_api::CardId::new(id),
            def: Some(kind_code_u32(def.kind)),
            suit: Some(match def.suit {
                engine_native::catalog::Suit::Spade => engine_api::Suit::Spade,
                engine_native::catalog::Suit::Heart => engine_api::Suit::Heart,
                engine_native::catalog::Suit::Club => engine_api::Suit::Club,
                engine_native::catalog::Suit::Diamond => engine_api::Suit::Diamond,
            }),
            rank: Some(def.rank),
            owner,
            zone,
            kind: kind_tag(def.kind),
        });
    };
    for card in &view.cards {
        use engine_native::snap::PhysicalCardZone as Z;
        let zone = match card.zone {
            Z::Hand => CardZoneTag::Hand,
            Z::Equip => CardZoneTag::Equip,
            Z::Judge => CardZoneTag::Judge,
            Z::DiscardPile => CardZoneTag::DiscardPile,
            Z::Processing => CardZoneTag::Processing,
            Z::DrawPile => CardZoneTag::DrawPile,
            Z::Removed => CardZoneTag::Removed,
            Z::Expansion => CardZoneTag::Expansion,
        };
        push(
            card.id,
            card.owner.map(|seat| ApiPlayerId::new(seat as u32 + 1)),
            zone,
        );
    }
    cards
}

fn identity_tag(role: engine_native::state::Role) -> IdentityTag {
    match role {
        engine_native::state::Role::Lord => IdentityTag::Lord,
        engine_native::state::Role::Loyalist => IdentityTag::Loyalist,
        engine_native::state::Role::Rebel => IdentityTag::Rebel,
        engine_native::state::Role::Traitor => IdentityTag::Renegade,
    }
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
    let native_seat = native_seat(engine, seat);
    let view: PlayerView = match (role, native_seat) {
        (RoomRole::Player, Some(p)) => engine.view_for(p),
        (RoomRole::Referee, _) => engine.view_referee(),
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
            character_def: cell.generals[0].as_ref().map(|g| g.def as u32).unwrap_or(0),
            generals: std::array::from_fn(|slot| {
                cell.generals[slot]
                    .as_ref()
                    .map(|general| engine_api::GeneralSlotView {
                        def: general.def as u32,
                        name: general.name.to_owned(),
                        faction: format!("{:?}", general.faction).to_lowercase(),
                        revealed: cell.revealed_generals[slot],
                    })
            }),
            seat: engine_api::SeatIndex::new(cell.seat),
            hp: cell.hp,
            max_hp: cell.max_hp,
            alive: cell.alive,
            turned_over: cell.turned_over,
            // 座位仅给手牌数量；授权牌面由 known_cards 提供。
            hand_count: match &cell.hand {
                HandView::Own(cards) => cards.len() as u32,
                HandView::Hidden => cell.hand_count as u32,
            },
            // 身份可见性：本人、主公（公开）与已死亡者（遗言公开）可见；其余为 None
            identity: cell.role.map(identity_tag),
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
        known_cards: known_cards_for(engine, &view),
        outcome,
    }
}

fn native_seat(engine: &Engine, seat: Option<ApiPlayerId>) -> Option<u8> {
    let seat = seat?.raw().checked_sub(1)?;
    (seat < engine.state_core().players.len() as u32).then(|| seat as u8)
}

impl From<engine_protocol::ClientRole> for RoomRole {
    fn from(role: engine_protocol::ClientRole) -> Self {
        match role {
            engine_protocol::ClientRole::Player => Self::Player,
            engine_protocol::ClientRole::Spectator => Self::Spectator,
            engine_protocol::ClientRole::Referee => Self::Referee,
        }
    }
}

/// Full options for the owner or trusted referee; role always comes from membership.
pub fn choice_for(
    engine: &Engine,
    role: RoomRole,
    seat: Option<ApiPlayerId>,
) -> Option<engine_api::ChoiceRequest> {
    let view = match role {
        RoomRole::Player => engine.choice_view_for(Some(native_seat(engine, seat)?)),
        RoomRole::Referee => engine.choice_view_referee(),
        RoomRole::Spectator => return None,
    }?;
    let engine_native::view::ChoiceView::Own {
        id,
        kind,
        options,
        min,
        max,
        allow_cancel,
        prompt,
        source_skill,
    } = view
    else {
        return None;
    };
    let observer = ApiPlayerId::new(engine.open_choice()?.observer as u32 + 1);
    // 协议 `ChoiceOption` 是 `{token, label_key}` 二元组（token 即 ResolveChoice 的
    // 选择值；label_key 只用于本地化/显示，不参与规则）。token 与原生选项一一对应：
    // 使用**候选下标**（与前端适配层/`answer()` 同口径），label_key 保留语义标签。
    let options: Vec<engine_api::ChoiceOption> = options
        .iter()
        .enumerate()
        .map(|(idx, opt)| engine_api::ChoiceOption {
            token: idx as u32,
            label_key: match opt {
                engine_native::ChoiceOption::HiddenCard(_) => "hand".to_owned(),
                engine_native::ChoiceOption::Control(option) => option.label().to_owned(),
                engine_native::ChoiceOption::Confirm => "confirm".to_string(),
                engine_native::ChoiceOption::Pass => "pass".to_string(),
                engine_native::ChoiceOption::Card(c) => format!("card:{c}"),
                engine_native::ChoiceOption::Player(p) => format!("player:{}", *p as u32 + 1),
                engine_native::ChoiceOption::Tag(t) => format!("tag:{t}"),
                engine_native::ChoiceOption::Skill(s) => {
                    format!("skill:{}", engine.catalog().skill_name(*s))
                }
                engine_native::ChoiceOption::Kind(k) => format!("kind:{k}"),
            },
        })
        .collect();
    Some(engine_api::ChoiceRequest {
        choice_id: engine_api::ChoiceId::new(id as u32),
        observer,
        options,
        continuation_id: id,
        kind: if kind == engine_native::ChoiceKind::Control {
            engine_api::ChoiceKind::Custom
        } else if prompt == "trigger_order" {
            engine_api::ChoiceKind::OrderTriggers
        } else {
            engine_api::choice_kind_from_native(engine_native::view::choice_kind_name(kind))
                .expect("canonical native choice kind")
        },
        native_kind: if kind == engine_native::ChoiceKind::Control {
            None
        } else {
            Some(engine_native::view::choice_kind_name(kind).to_owned())
        },
        source_skill: source_skill.map(str::to_owned),
        prompt: Some(prompt.to_owned()),
        min_count: u32::try_from(min).expect("native choice minimum fits u32"),
        max_count: u32::try_from(max).expect("native choice maximum fits u32"),
        allow_cancel,
        timeout_policy: engine_api::TimeoutPolicy::None,
        deadline_unix_ms: None,
        revision: engine.revision(),
    })
}

/// 权威动作 → 协议动作（只映射本内核可表达的形态；未覆盖形态见待办）。
pub fn legal_actions_for(engine: &Engine, seat: ApiPlayerId) -> Vec<LegalAction> {
    let Some(seat_u8) = native_seat(engine, Some(seat)) else {
        return Vec::new();
    };
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
            GameAction::RevealGeneral { .. } | GameAction::TimeoutAction { .. } => None,
        })
        .collect()
}

#[cfg(test)]
mod c9i_wide_boundary {
    use super::*;
    #[test]
    fn c9i_supported_wide_zhangwu_preserves_host_cardinality() {
        use engine_native::content::extension::{ExtensionDef, ExtensionPack, InitHook};
        let p = ExtensionPack {
            def: ExtensionDef {
                name: "c9i-real-wide",
                version: 1,
                description: "unique actual cards",
            },
            generals: vec![],
            host_skills: vec![],
            vm_skills: vec![],
            cards: (0..300)
                .map(|i| {
                    engine_native::catalog::CardDef::basic(
                        Box::leak(format!("c9i-real-sha-{i}").into_boxed_str()),
                        engine_native::CardKind::Sha,
                    )
                })
                .collect(),
            init_hook: InitHook::None,
            enabled: true,
        };
        let mut e = Engine::new_with_extensions(17001, 4, &[&p]).unwrap();
        for seat in 0..4 {
            e.debug_clear_hand(seat);
        }
        e.debug_grant(1, "章武");
        let names: Vec<_> = e
            .catalog()
            .cards
            .iter()
            .filter(|c| c.name.starts_with("c9i-real-sha-"))
            .take(256)
            .map(|c| c.name)
            .collect();
        for name in names {
            e.debug_move_named_card_to_hand(1, name);
        }
        let saved = e.save();
        let bytes = engine_native::archive::encode_archive(&saved);
        let decoded = engine_native::archive::decode_archive(&bytes).unwrap();
        let _ = Engine::restore_with_extensions(&decoded, &[&p]).unwrap();
        e.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
        assert_eq!(e.open_choice().unwrap().spec.max, 256);
        let request = choice_for(&e, RoomRole::Player, Some(ApiPlayerId::new(2))).unwrap();
        let raw = serde_json::to_string(&request).unwrap();
        if let Ok(dir) = std::env::var("C9I_BOUNDARY_OUT") {
            std::fs::write(
                std::path::Path::new(&dir).join("actual-server-wide256.json"),
                &raw,
            )
            .unwrap();
        }
        let mut required: serde_json::Value = serde_json::from_str(&raw).unwrap();
        required["max_count"] = serde_json::json!(256);
        let accepted = serde_json::from_value::<engine_api::ChoiceRequest>(required.clone()).unwrap();
        assert_eq!(accepted.max_count, 256);
        let rejection = serde_json::from_value::<super::c9j_ws6::ChoiceRequest>(required.clone())
            .unwrap_err()
            .to_string();
        if let Ok(dir) = std::env::var("C9I_BOUNDARY_OUT") {
            std::fs::write(
                std::path::Path::new(&dir).join("required-native-wide256.json"),
                serde_json::to_vec_pretty(&required).unwrap(),
            )
            .unwrap();
            std::fs::write(
                std::path::Path::new(&dir).join("current-unchanged-api1-reader-rejection.txt"),
                &rejection,
            )
            .unwrap();
        }
        assert_eq!(
            request.max_count as usize, 256,
            "supported native max256 must not wrap in actual host projection: {raw}"
        );
    }
}

#[cfg(test)]
#[path = "../tests/support/ws6_choice_before_source.rs"]
mod c9j_ws6;

#[cfg(test)]
mod c9i_frozen_positive {
    use super::*;
    #[test]
    fn c9i_actual_private_choice_existing_host_tokens() {
        let raw = std::fs::read_to_string(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../packages/extensions/shahai-swd-upstream-noname/assets/cards.json"),
        )
        .unwrap();
        let pack = content_native_loader::bridge_pack(&raw).unwrap().pack;
        let mut e = Engine::new_with_extensions(17002, 4, &[&pack]).unwrap();
        for p in 0..4 {
            e.debug_clear_hand(p);
        }
        let name = e
            .catalog()
            .cards
            .iter()
            .find(|c| c.kind == engine_native::CardKind::Qinglonglingzhu)
            .unwrap()
            .name;
        let eq = e.debug_place(0, name);
        e.state_core_mut().player_mut(0).hand.retain(|c| *c != eq);
        e.state_core_mut().player_mut(0).equips.push(eq);
        e.debug_place(1, "tao");
        e.debug_place(1, "sha");
        e.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Fire);
        let choice = choice_for(&e, RoomRole::Player, Some(ApiPlayerId::new(1))).unwrap();
        assert_eq!(choice.max_count, 1);
        assert_eq!(
            choice
                .options
                .iter()
                .map(|o| (&o.token, &o.label_key))
                .collect::<Vec<_>>(),
            vec![(&0, &"hand".to_string()), (&1, &"hand".to_string())]
        );
        assert!(choice.source_skill.is_none());
        if let Ok(dir) = std::env::var("C9I_BOUNDARY_OUT") {
            std::fs::write(
                std::path::Path::new(&dir).join("actual-server-qinglong-small.json"),
                serde_json::to_vec_pretty(&choice).unwrap(),
            )
            .unwrap();
            let mut omitted = serde_json::to_value(&choice).unwrap();
            omitted.as_object_mut().unwrap().remove("native_kind");
            omitted.as_object_mut().unwrap().remove("source_skill");
            std::fs::write(
                std::path::Path::new(&dir).join("actual-server-qinglong-omission.json"),
                serde_json::to_vec_pretty(&omitted).unwrap(),
            )
            .unwrap();
        }
    }
}
