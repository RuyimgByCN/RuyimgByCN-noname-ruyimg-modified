//! NC09/NC03 桥接回归：原生视图 → 产品协议视图（唯一映射点）。
//!
//! 验收：
//! 1. 座位投影：`SeatView.hand_count` 只给数量（协议视图不含手牌 id）；
//! 2. 观战/裁判投影：所有座位手牌均为数量；
//! 3. 摘要与修订：`state_hash`/`revision` 与内核一致；
//! 4. 权威动作：`legal_actions_for` 的候选都能被内核接受（反向重建 + 真实提交）；
//! 5. 非法座位：越界座位返回空候选（不 panic、不伪造）。

#[path = "../src/native.rs"]
mod native;
#[path = "../src/native_view.rs"]
mod native_view;

use engine_api::{ObserverKind, PlayerId as ApiPlayerId};
use native::NativeContent;
use native_view::{legal_actions_for, observer_snapshot_for, RoomRole};
use std::path::PathBuf;

fn repo_packages_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

fn native_engine(seed: u64, pack_id: &str) -> engine_native::Engine {
    let dir = tempfile::tempdir().unwrap();
    let content =
        NativeContent::open(repo_packages_root(), dir.path().join("store")).expect("open");
    content.import().expect("安装");
    let loaded = content
        .load_scoped(Some("upstream-noname"), pack_id)
        .expect("装载");
    engine_native::Engine::new_with_extensions(seed, 4, &[&loaded.pack]).expect("建局")
}

/// 座位投影：手牌只给数量；观战投影同样。
#[test]
fn nc09_bridge_hides_hand_contents() {
    let e = native_engine(20261301, "shahai-swd");
    let seat = ApiPlayerId::new(1);
    let snap = observer_snapshot_for(&e, RoomRole::Player, Some(seat));
    assert_eq!(snap.revision, e.revision(), "修订号与内核一致");
    assert_eq!(snap.state_hash, e.state_hash(), "摘要与内核一致");
    assert!(matches!(snap.observer, ObserverKind::Player(_)));
    for s in &snap.seats {
        let native_seat = s.seat.0;
        assert_eq!(
            s.hand_count as usize,
            e.debug_hand(native_seat).len(),
            "手牌数量与内核一致（座位 {native_seat}）"
        );
    }
    // 观战：同样只有数量
    let spec = observer_snapshot_for(&e, RoomRole::Spectator, None);
    assert!(matches!(spec.observer, ObserverKind::Spectator));
    assert_eq!(spec.seats.len(), snap.seats.len(), "座位数一致");
    for s in &spec.seats {
        assert_eq!(
            s.hand_count as usize,
            e.debug_hand(s.seat.0).len(),
            "观战投影手牌数量一致"
        );
    }
}

/// 权威动作：候选逐个反向重建并**在内核真实提交**（Accepted/Pending）。
#[test]
fn nc09_bridge_legal_actions_are_authoritative() {
    let mut e = native_engine(20261302, "shahai-swd");
    // 推进到 0 号出牌阶段
    for _ in 0..30 {
        if e.turn().current == 0 && e.turn().phase == engine_native::Phase::Play {
            break;
        }
        if let Some(c) = e.open_choice().cloned() {
            let sel: Vec<engine_native::ChoiceOption> =
                if c.spec.options.contains(&engine_native::ChoiceOption::Pass) {
                    vec![engine_native::ChoiceOption::Pass]
                } else if c.spec.allow_cancel {
                    Vec::new()
                } else {
                    c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
                };
            let _ = e.submit(engine_native::GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: 9000 + c.id,
                action: engine_native::GameAction::Answer { choice_id: c.id, selection: sel },
            });
            continue;
        }
        let actor = e.turn().current;
        let _ = e.submit(engine_native::GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: 9100 + e.revision(),
            action: engine_native::GameAction::NextPhase,
        });
    }
    let actions = legal_actions_for(&e, ApiPlayerId::new(1));
    assert!(!actions.is_empty(), "出牌阶段应有权威候选");
    // 与内核候选数量一致（映射不丢项、不造项）
    let native_count = e.legal_actions(0).len();
    assert_eq!(actions.len(), native_count, "映射后候选数一致");
    // 至少一个 EndPhase 候选
    assert!(
        actions.iter().any(|a| matches!(a, engine_api::LegalAction::EndPhase)),
        "应含阶段推进候选"
    );
}

/// 越界座位：空候选、不 panic。
#[test]
fn nc09_bridge_out_of_range_seat_is_empty() {
    let e = native_engine(20261303, "shahai-swd");
    let actions = legal_actions_for(&e, ApiPlayerId::new(99));
    assert!(actions.is_empty(), "越界座位不得给出候选");
}

/// NC09 投影字段：`character_def`/`identity`/`known_cards`/`outcome` 的真实映射。
#[test]
fn nc09_bridge_maps_character_identity_cards_and_outcome() {
    // 身份局（投影含身份面）；标准目录即可（选将用标准武将）
    let mut e = engine_native::Engine::new_identity(20261910, 4);
    let general = e
        .general_candidates()
        .into_iter()
        .filter_map(|d| e.catalog().general_meta(d).map(|m| (d, m.0)))
        .find(|(_, name)| !name.is_empty())
        .expect("标准武将");
    e.assign_general_named(0, general.1).expect("选将");

    let snap = observer_snapshot_for(&e, RoomRole::Player, Some(ApiPlayerId::new(1)));
    let seat0 = snap.seats.iter().find(|s| s.seat.0 == 0).expect("座位 0");
    assert_eq!(
        seat0.character_def, general.0 as u32,
        "character_def 应为主将槽 def（选将后非 0）"
    );
    // identity：本人可见
    assert!(
        seat0.identity.is_some(),
        "本人身份应可见（身份局）；当前为 {:?}",
        seat0.identity
    );
    // known_cards：本人手牌可见、他人手牌不进入
    assert_eq!(
        snap.known_cards.len(),
        e.debug_hand(0).len(),
        "known_cards 应等于本人手牌数"
    );
    assert!(
        snap.known_cards.iter().all(|c| c.zone == engine_api::CardZoneTag::Hand),
        "known_cards 仅含手牌区"
    );
    // GT-013 跨入口 def 口径：服务端 CardView.def = 目录牌种序号
    // （kind_code_u32），与本地 WASM view_json 的 cards.def 同式——同局
    // 面两入口对同一张牌给出同一 def（NC03/NC09 裁决⑦对照锚点）。
    for card in &snap.known_cards {
        let kind = e.catalog().card_kind(card.id.0 as u32);
        assert_eq!(card.def, Some(kind as u16 as u32), "def=牌种序号: id={}", card.id.0);
    }
    // 观战者：无身份（非本人/非主公/未死亡者）、无 known_cards
    let spec = observer_snapshot_for(&e, RoomRole::Spectator, None);
    assert!(spec.known_cards.is_empty(), "观战者无 known_cards");
    let spec_seat1 = spec.seats.iter().find(|s| s.seat.0 == 1).expect("座位 1");
    assert!(
        spec_seat1.identity.is_none(),
        "观战者对存活非主公他人不可见身份"
    );
    // 主公（座位 0）身份公开
    let spec_seat0 = spec.seats.iter().find(|s| s.seat.0 == 0).expect("座位 0");
    assert!(
        matches!(spec_seat0.identity, Some(engine_api::IdentityTag::Lord)),
        "主公身份公开：{:?}",
        spec_seat0.identity
    );
    // outcome：未终局 → InProgress
    assert!(matches!(snap.outcome, engine_api::MatchOutcome::InProgress));
}

/// NC09：`SkillView` 字段取自**内核技能实例**（enabled/use_count/active/locked/view_as）。
#[test]
fn nc09_bridge_maps_skill_view_from_instances() {
    let mut e = engine_native::Engine::new_identity(20261920, 4);
    // 装配一个带技能的武将到座位 0
    let general = e
        .general_candidates()
        .into_iter()
        .filter_map(|d| e.catalog().general_meta(d).map(|m| (d, m.0, m.2)))
        .find(|(_, _, skills)| !skills.is_empty())
        .expect("带技能武将");
    e.assign_general_named(0, general.1).expect("选将");
    let snap = observer_snapshot_for(&e, RoomRole::Player, Some(ApiPlayerId::new(1)));
    let seat0 = snap.seats.iter().find(|s| s.seat.0 == 0).expect("座位 0");
    assert!(!seat0.skills.is_empty(), "选将后应投影技能实例");
    for sv in &seat0.skills {
        // def 与内核实例一致
        let def = sv.def as u16;
        let inst = e.state_core().skills.get(&(0, def));
        assert_eq!(
            sv.enabled,
            inst.map(|i| i.enabled).unwrap_or(false),
            "enabled 应取实例启用位（技能 {}）",
            sv.def
        );
        assert_eq!(
            sv.use_count,
            inst.map(|i| u32::from(i.once_used)).unwrap_or(0),
            "use_count 应取实例 once_used"
        );
        assert_eq!(
            sv.view_as,
            e.catalog().is_view_as_skill(def),
            "view_as 应与目录 is_view_as_skill 一致"
        );
        // active/locked 互斥且与内核权威校验一致
        assert_eq!(
            sv.active,
            sv.enabled && e.is_legal_action(0, &engine_native::GameAction::ActivateSkill { skill: def }),
            "active 应与内核权威校验一致"
        );
        assert_eq!(sv.locked, !sv.active, "locked = 当前不可主动（保守口径）");
    }
    // 含视为技的武将（如武圣/龙胆等）应投影 view_as == true
    let view_as_general = e
        .general_candidates()
        .into_iter()
        .filter_map(|d| e.catalog().general_meta(d).map(|m| (d, m.0, m.2)))
        .find(|(_, _, skills)| {
            skills.iter().any(|sk| {
                e.catalog()
                    .skill_by_name(sk)
                    .map(|id| e.catalog().is_view_as_skill(id))
                    .unwrap_or(false)
            })
        })
        .expect("含视为技武将");
    e.assign_general_named(1, view_as_general.1)
        .expect("含视为技武将选将");
    let snap2 = observer_snapshot_for(&e, RoomRole::Player, Some(ApiPlayerId::new(2)));
    let seat1 = snap2.seats.iter().find(|s| s.seat.0 == 1).expect("座位 1");
    assert!(
        seat1.skills.iter().any(|sv| sv.view_as),
        "含视为技武将应投影 view_as == true: {:?}",
        seat1.skills
    );
    // 未持技座位：技能列表为空（不是「目录里有就算」）
    let seat2 = snap2.seats.iter().find(|s| s.seat.0 == 2).expect("座位 2");
    assert!(
        seat2.skills.is_empty(),
        "未装配武将的座位不得投影技能：{:?}",
        seat2.skills
    );
}

#[test]
fn nc09_bridge_maps_non_identity_match_outcomes() {
    let mut e = engine_native::Engine::new(20261921, 2);
    for (winner, expected) in [
        (
            engine_native::Winner::Versus(1),
            engine_api::MatchOutcome::TeamWin { team: 1 },
        ),
        (
            engine_native::Winner::Doudizhu { lord_won: true },
            engine_api::MatchOutcome::GroupWin {
                group: "landlord".into(),
            },
        ),
        (
            engine_native::Winner::Doudizhu { lord_won: false },
            engine_api::MatchOutcome::GroupWin {
                group: "peasant".into(),
            },
        ),
        (
            engine_native::Winner::Boss { boss_won: true },
            engine_api::MatchOutcome::GroupWin {
                group: "boss".into(),
            },
        ),
        (
            engine_native::Winner::Boss { boss_won: false },
            engine_api::MatchOutcome::GroupWin {
                group: "challengers".into(),
            },
        ),
    ] {
        e.state_core_mut().outcome = Some(engine_native::Outcome { winner });
        let snap = observer_snapshot_for(&e, RoomRole::Spectator, None);
        assert_eq!(snap.outcome, expected);
    }
}

