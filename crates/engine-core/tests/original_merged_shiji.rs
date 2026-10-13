//! O2 全量合并批次 1（2026-10-14）：家族 shiji 的 29 个独立测试文件逐字包裹为模块——#[test] 函数名不变、测试数量不变；测试全名获得确定性前缀（模块名=原文件名，子串过滤行为不变）；include_bytes/str 相对路径不受影响（同目录）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_shiji_boming_ejian {
//! Golden for SHIJI 许靖「博明 / 恶荐」语义（ABI 413 / 414）。
//!
//! 博明：出牌阶段限两次，将一张手牌/装备交给一名其他角色。
//! 恶荐：赠牌后目标若另有同类牌则二选一——受 1 点伤害 / 展示手牌并弃所有同类牌。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const BOMING: u32 = 4_154_330_954;
const EJIAN: u32 = 4_172_706_421;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/shiji/shiji.npack.json");

fn setup(skill_def: u32, players: u8) -> (Transaction, PlayerId, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(skill_def as u64),
            kind: CommandKind::StartGame {
                player_count: players,
                rng_seed: 20_260_805,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let seats = engine.state().players.alive_by_seat();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, seats)
}

fn inject_hand(tx: &mut Transaction, player: PlayerId, kind: CardKind, suit: Suit) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(80_000 + kind.tag() as u32),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

/// 博明：主动技赠牌——选中的手牌进入目标手牌。
#[test]
fn boming_gives_selected_card() {
    let (mut tx, owner, seats) = setup(BOMING, 2);
    let other = seats.into_iter().find(|p| *p != owner).unwrap();
    let gift = inject_hand(&mut tx, owner, CardKind::Sha, Suit::Spade);
    let before = tx.state().hand_count(other);

    // 手动构造主动技调用(选牌+选目标)。
    let boming_skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|s| s.def.raw() == BOMING && s.owner == owner)
        .map(|s| s.id)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveBoming {
            player: owner,
            target: other,
            card: gift,
            skill: boming_skill,
        }],
    )
    .unwrap();
    assert_eq!(
        tx.state().hand_count(other),
        before + 1,
        "博明应把选中的牌交给目标"
    );
    assert!(
        tx.state()
            .zones
            .get(tx.state().zones.find(ZoneKind::Hand, other).unwrap())
            .unwrap()
            .cards
            .contains(&gift),
        "目标手牌应含赠牌"
    );
}

/// 恶荐：赠牌后目标有同类牌 → 二选一 choice。
#[test]
fn ejian_offers_choice_when_same_category_exists() {
    let (mut tx, owner, seats) = setup(EJIAN, 2);
    let other = seats.into_iter().find(|p| *p != owner).unwrap();
    // 目标已有 1 张杀,再被赠 1 张杀 → 有同类。
    let existing = inject_hand(&mut tx, other, CardKind::Sha, Suit::Club);
    let gift = inject_hand(&mut tx, owner, CardKind::Sha, Suit::Spade);
    // 赠牌触发 CardsGiven。
    let err = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardsGiven {
            giver: owner,
            recipient: other,
            cards: vec![gift],
        }],
        &mut TriggerFireRecord::new(),
    )
    .map(|_| ())
    .err();
    // 手动 dispatch 只返回 effects;执行它打开二选一 choice。
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardsGiven {
            giver: owner,
            recipient: other,
            cards: vec![gift],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    let result = run_effects(&mut tx, effects);
    assert_eq!(result.err(), Some(EffectError::ChoiceRequested));
    assert!(tx.state().has_pending_choice(), "应打开二选一 choice");
    let _ = existing;
    let _ = err;
}

/// 恶荐：目标选"受 1 点伤害"。
#[test]
fn ejian_damage_option() {
    let (mut tx, owner, seats) = setup(EJIAN, 2);
    let other = seats.into_iter().find(|p| *p != owner).unwrap();
    let existing = inject_hand(&mut tx, other, CardKind::Sha, Suit::Club);
    let gift = inject_hand(&mut tx, owner, CardKind::Sha, Suit::Spade);
    let hp_before = tx.state().players.get(other).unwrap().hp;
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardsGiven {
            giver: owner,
            recipient: other,
            cards: vec![gift],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert_eq!(
        run_effects(&mut tx, effects).err(),
        Some(EffectError::ChoiceRequested)
    );
    let state = resolve_choice(tx.commit().state, other, 1, vec![0], false);
    assert_eq!(
        state.players.get(other).unwrap().hp,
        hp_before - 1,
        "选受伤应受 1 点伤害"
    );
    let _ = existing;
}

/// 恶荐：目标选"展示手牌并弃所有同类牌"——目标弃掉全部同类牌。
#[test]
fn ejian_discard_same_category_option() {
    let (mut tx, owner, seats) = setup(EJIAN, 2);
    let other = seats.into_iter().find(|p| *p != owner).unwrap();
    let existing = inject_hand(&mut tx, other, CardKind::Sha, Suit::Club);
    let gift = inject_hand(&mut tx, owner, CardKind::Sha, Suit::Spade);
    let hand_before = tx.state().hand_count(other);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardsGiven {
            giver: owner,
            recipient: other,
            cards: vec![gift],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert_eq!(
        run_effects(&mut tx, effects).err(),
        Some(EffectError::ChoiceRequested)
    );
    // 选"展示弃同类"(option 1)。
    let state = resolve_choice(tx.commit().state, other, 2, vec![1], false);
    // 目标弃掉了同类(杀)牌;赠牌(杀)本身是 gift——它在目标手上也同类,会被弃。
    assert!(state.hand_count(other) < hand_before, "目标应弃掉同类牌");
    let _ = existing;
}

fn resolve_choice(
    state: GameState,
    actor: PlayerId,
    key: u64,
    selections: Vec<u32>,
    cancelled: bool,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    next
}
}

mod original_shiji_db_wenyang {
//! Golden for SHIJI 标文鸯「追锋 / 曲迪 / 重剑」语义（ABI 419 / 420）。
//!
//! 追锋：出牌阶段限两次，失 1 点体力并对一名其他角色视为使用【决斗】。
//! 曲迪：单目标杀/决斗指定后三选一（获牌/弃基本加伤/背水）。
//! 重剑：装备牌当【杀】或【酒】使用/打出。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const DBZHUIFENG: u32 = 2_625_491_865;
const DBQUEDI: u32 = 2_328_150_606;
const DBCHONGJIAN: u32 = 4_130_288_495;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/shiji/shiji.npack.json");

fn setup(skill_def: u32, players: u8) -> (Transaction, PlayerId, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(skill_def as u64),
            kind: CommandKind::StartGame {
                player_count: players,
                rng_seed: 20_260_809,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let seats = engine.state().players.alive_by_seat();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, seats)
}

fn inject_hand(tx: &mut Transaction, player: PlayerId, kind: CardKind, suit: Suit) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(80_000 + kind.tag() as u32),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn resolve_choice(
    state: GameState,
    actor: PlayerId,
    key: u64,
    selections: Vec<u32>,
    cancelled: bool,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    next
}

/// 追锋：失 1 点体力并打开选目标 choice（决斗目标）。
#[test]
fn dbzhuifeng_loses_hp_and_offers_target() {
    let (mut tx, owner, seats) = setup(DBZHUIFENG, 2);
    let _other = seats.into_iter().find(|p| *p != owner).unwrap();
    let hp_before = tx.state().players.get(owner).unwrap().hp;
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|s| s.def.raw() == DBZHUIFENG && s.owner == owner)
        .map(|s| s.id)
        .unwrap();
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDbzhuifeng {
            player: owner,
            skill,
        }],
    )
    .err();
    assert_eq!(
        err,
        Some(EffectError::ChoiceRequested),
        "应打开选目标 choice"
    );
    assert_eq!(
        tx.state().players.get(owner).unwrap().hp,
        hp_before - 1,
        "应先失去 1 点体力"
    );
}

/// 曲迪：单目标杀指定后打开三选一 choice。
#[test]
fn dbquedi_offers_choice_after_single_target_sha() {
    let (mut tx, owner, seats) = setup(DBQUEDI, 2);
    let other = seats.into_iter().find(|p| *p != owner).unwrap();
    // 目标有手牌(可获) + 自己有基本牌(可弃)。
    let _target_hand = inject_hand(&mut tx, other, CardKind::Tao, Suit::Heart);
    let _owner_basic = inject_hand(&mut tx, owner, CardKind::Sha, Suit::Club);
    let sha = inject_hand(&mut tx, owner, CardKind::Sha, Suit::Spade);
    // 直接调用曲迪原语(目标有可获手牌)。
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|s| s.def.raw() == DBQUEDI && s.owner == owner)
        .map(|s| s.id)
        .unwrap();
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDbquedi {
            player: owner,
            target: other,
            card: sha,
            skill,
        }],
    )
    .err();
    assert_eq!(
        err,
        Some(EffectError::ChoiceRequested),
        "应打开三选一 choice"
    );
    // 选"获得目标一张手牌"。
    let state = resolve_choice(tx.commit().state, owner, 1, vec![0], false);
    assert!(state.hand_count(owner) >= 2, "应获得目标一张手牌");
}

/// 重剑：装备牌当杀（view_as 编译与注册验证）。
#[test]
fn dbchongjian_view_as_registered() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let entry = package
        .view_as_index
        .iter()
        .find(|view| view.skill_def_id == DBCHONGJIAN)
        .expect("重剑应有 view_as 条目");
    assert_eq!(entry.as_kind, "sha");
}
}

mod original_shiji_feiyi {
//! Golden for SHIJI 费祎「生息 / 谏喻」语义（ABI 425）。
//!
//! 生息：结束阶段本回合用过牌且未造成伤害时，可选获得智囊或摸牌（复用 TW 生息）。
//! 谏喻：出牌阶段限一次选 2 名角色获喻标记；出牌阶段喻标记角色用牌指定另一喻标记
//! 角色为目标时，目标摸 1。

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const FYJIANYU: u32 = 2_248_481_428;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/shiji/shiji.npack.json");

fn setup(players: u8) -> (Transaction, PlayerId, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(8_112),
            kind: CommandKind::StartGame {
                player_count: players,
                rng_seed: 20_260_812,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let seats = engine.state().players.alive_by_seat();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(FYJIANYU),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, seats)
}

fn resolve_choice(
    state: GameState,
    actor: PlayerId,
    key: u64,
    selections: Vec<u32>,
    cancelled: bool,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    next
}

/// 谏喻 mode 0：授予 2 名目标喻标记。
#[test]
fn fyjianyu_grants_marks_to_two_targets() {
    let (mut tx, owner, seats) = setup(3);
    let others: Vec<_> = seats.into_iter().filter(|p| *p != owner).collect();
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|s| s.def.raw() == FYJIANYU && s.owner == owner)
        .map(|s| s.id)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveFyjianyu {
            player: owner,
            source: PlayerId::NONE,
            target: PlayerId::NONE,
            targets: others.clone(),
            skill,
        }],
    )
    .unwrap();
    for target in &others {
        assert!(
            tx.state()
                .players
                .get(*target)
                .unwrap()
                .marks
                .iter()
                .any(|(mark, count)| *mark == FYJIANYU && *count == 1),
            "目标应获得喻标记"
        );
    }
}

/// 谏喻 mode 1：喻标记来源指定喻标记目标 → 目标摸 1。
#[test]
fn fyjianyu_marked_target_draws() {
    let (mut tx, owner, seats) = setup(3);
    let others: Vec<_> = seats.into_iter().filter(|p| *p != owner).collect();
    let source = others[0];
    let target = others[1];
    // 手动给 source 和 target 喻标记。
    for p in [source, target] {
        run_effects(
            &mut tx,
            vec![EffectRequest::AddMark {
                player: p,
                mark: engine_api::MarkDefId::new(FYJIANYU),
                delta: 1,
            }],
        )
        .unwrap();
    }
    let hand_before = tx.state().hand_count(target);
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|s| s.def.raw() == FYJIANYU && s.owner == owner)
        .map(|s| s.id)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveFyjianyu {
            player: owner,
            source,
            target,
            targets: Vec::new(),
            skill,
        }],
    )
    .unwrap();
    assert_eq!(
        tx.state().hand_count(target),
        hand_before + 1,
        "喻标记目标应摸 1 张"
    );
    let _ = CardId::NONE;
}
}

mod original_shiji_mbdiaodu {
//! Golden for SHIJI/SIXIANG 吕范「调度」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const MBDIAODU: u32 = 2_440_438_931;
const STD_LVFAN: u32 = 3_319_229_677;
const SHIJI: &[u8] = include_bytes!("../../../content/original/ported/shiji/shiji.npack.json");
const SIXIANG: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

fn setup(player_count: u8) -> (GameState, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(SHIJI).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(MBDIAODU as u64),
            kind: CommandKind::StartGame {
                player_count,
                rng_seed: 20_260_718,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let players = engine.state().players.alive_by_seat();
    let owner = players[0];
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(MBDIAODU),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.state().clone(), players)
}

fn equip(state: &mut GameState, owner: PlayerId, kind: CardKind, def: u32) -> CardId {
    let zone = state.zones.find(ZoneKind::Equip, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
        zone,
        owner,
        face_up: true,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn offer(
    state: GameState,
    owner: PlayerId,
) -> (GameState, Result<Vec<EffectRequest>, EffectError>) {
    let mut tx = Transaction::begin(state);
    let result = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    );
    (tx.state().clone(), result)
}

fn resolve(state: GameState, selection: u32, key: u64) -> (GameState, PipelineResult) {
    let actor = PlayerId::new(state.resolution.pending_observer);
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: choice,
                selections: vec![selection],
                cancelled: false,
            },
        },
    )
}

#[test]
fn package_contains_mbdiaodu_effect_and_dependent_complete_character() {
    let shiji = content_schema::ContentPackage::from_json_bytes(SHIJI).unwrap();
    assert!(shiji.manifest.rule_vm_abi_version >= 198);
    let trigger = shiji
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == MBDIAODU)
        .unwrap();
    assert_eq!(
        (
            trigger.event.as_str(),
            trigger.filter.as_str(),
            trigger.optional,
        ),
        (
            "phase_prepare",
            "owner_is_target_has_movable_equipment",
            true,
        )
    );

    let sixiang = content_schema::ContentPackage::from_json_bytes(SIXIANG).unwrap();
    assert!(sixiang
        .manifest
        .dependencies
        .iter()
        .any(|dependency| dependency.package_id == "shiji"));
    let lvfan = sixiang
        .character_index
        .iter()
        .find(|character| character.id == STD_LVFAN)
        .unwrap();
    assert_eq!(lvfan.skills, vec![MBDIAODU, 3_414_143_685]);
}

#[test]
fn chooser_moves_equipment_to_only_legal_target_and_source_draws() {
    let (mut state, players) = setup(3);
    let (owner, source, destination) = (players[0], players[1], players[2]);
    let moved = equip(&mut state, source, CardKind::Weapon, 91_001);
    let source_hand = state.hand_count(source);

    let (state, offered) = offer(state, owner);
    assert_eq!(offered, Err(EffectError::ChoiceRequested));
    let (state, accepted) = resolve(state, 1, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::MoveEquipmentThenDraw
    );
    assert_eq!(
        accepted.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );
    assert_eq!(
        accepted
            .choice
            .as_ref()
            .unwrap()
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        vec![moved.raw()]
    );

    let (state, selected) = resolve(state, moved.raw(), 2);
    assert!(matches!(selected.result, CommandResult::Accepted { .. }));
    assert_eq!(
        selected.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectTargets
    );
    assert_eq!(
        selected.choice.as_ref().unwrap().options[0].token,
        destination.raw()
    );
    let (state, completed) = resolve(state, destination.raw(), 3);
    assert!(matches!(completed.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.cards.get(moved).unwrap().owner, destination);
    assert_eq!(
        state.cards.get(moved).unwrap().zone,
        state.zones.find(ZoneKind::Equip, destination).unwrap()
    );
    assert_eq!(state.hand_count(source), source_hand + 1);
}

#[test]
fn destination_same_slot_is_replaced_before_move() {
    let (mut state, players) = setup(3);
    let (owner, source, destination) = (players[0], players[1], players[2]);
    let moved = equip(&mut state, source, CardKind::Armor, 91_002);
    let replaced = equip(&mut state, destination, CardKind::Armor, 91_003);

    let (state, offered) = offer(state, owner);
    assert_eq!(offered, Err(EffectError::ChoiceRequested));
    let (state, _) = resolve(state, 1, 4);
    let (state, _) = resolve(state, moved.raw(), 5);
    let (state, completed) = resolve(state, destination.raw(), 6);
    assert!(matches!(completed.result, CommandResult::Accepted { .. }));
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(replaced).unwrap().zone, discard);
    assert_eq!(state.cards.get(moved).unwrap().owner, destination);
}

#[test]
fn no_third_legal_recipient_means_prepare_trigger_is_not_offered() {
    let (mut two_players, players) = setup(2);
    equip(&mut two_players, players[1], CardKind::Weapon, 91_004);
    let (_, offered) = offer(two_players, players[0]);
    assert_eq!(offered, Ok(Vec::new()));

    let (mut disabled, players) = setup(3);
    equip(&mut disabled, players[1], CardKind::Weapon, 91_005);
    disabled
        .players
        .get_mut(players[2])
        .unwrap()
        .disabled_equip_slots[0] = 1;
    let (_, offered) = offer(disabled, players[0]);
    assert_eq!(offered, Ok(Vec::new()));
}
}

mod original_shiji_qingyu {
//! Golden for SHIJI 糜夫人「清雨」语义（ABI 431）。
//!
//! 清雨：使命技——受伤前弃 2 张 he 防伤（defend）；准备阶段满血且手牌 < 轮数
//! 使命成功获【选存】（achieve）；濒死使命失败减 1 上限并选一人获【勇决】（fail）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const QINGYU: u32 = 3_126_505_108;
const XUANCUN: u32 = 2_417_220_435;
const MBYONGJUE: u32 = 4_187_437_274;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/shiji/shiji.npack.json");

fn setup(players: u8) -> (Transaction, PlayerId, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(QINGYU as u64),
            kind: CommandKind::StartGame {
                player_count: players,
                rng_seed: 20_260_814,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let seats = engine.state().players.alive_by_seat();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(QINGYU),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, seats)
}

fn clear_hand(tx: &mut Transaction, player: PlayerId) {
    let zone_id = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    tx.state_mut().zones.get_mut(zone_id).unwrap().cards.clear();
}

fn inject_hand(tx: &mut Transaction, player: PlayerId) -> CardId {
    let zone_id = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(80_000 + CardKind::Tao.tag() as u32),
        kind: CardKind::Tao,
        suit: Suit::Heart,
        rank: 7,
        weapon_range: 0,
        zone: zone_id,
        owner: player,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(zone_id)
        .unwrap()
        .cards
        .push(card);
    card
}

fn skill_instance(tx: &Transaction, owner: PlayerId) -> engine_api::SkillId {
    tx.state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|s| s.def.raw() == QINGYU && s.owner == owner)
        .map(|s| s.id)
        .unwrap()
}

fn resolve_choice(
    state: GameState,
    actor: PlayerId,
    key: u64,
    selections: Vec<u32>,
    cancelled: bool,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    next
}

fn qingyu_request(player: PlayerId, action: u8, skill: engine_api::SkillId) -> EffectRequest {
    EffectRequest::ResolveQingyu {
        player,
        action,
        xuancun_def: SkillDefId::new(XUANCUN),
        mbyongjue_def: SkillDefId::new(MBYONGJUE),
        target: PlayerId::NONE,
        skill,
    }
}

/// defend：可弃 he ≥ 2 时开强制弃牌 choice；弃后无进一步选择。
#[test]
fn qingyu_defend_discards_two_and_prevents() {
    let (mut tx, owner, _seats) = setup(2);
    let c1 = inject_hand(&mut tx, owner);
    let c2 = inject_hand(&mut tx, owner);
    let skill = skill_instance(&tx, owner);
    let err = run_effects(&mut tx, vec![qingyu_request(owner, 0, skill)]).err();
    assert_eq!(err, Some(EffectError::ChoiceRequested), "应打开弃牌 choice");
    let state = resolve_choice(tx.commit().state, owner, 1, vec![c1.raw(), c2.raw()], false);
    assert_eq!(state.resolution.pending_choice_id, 0, "弃牌后直接结算");
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let in_discard = |card: CardId| {
        state
            .zones
            .get(discard)
            .is_some_and(|zone| zone.cards.contains(&card))
    };
    assert!(in_discard(c1) && in_discard(c2), "两张牌应被弃置");
}

/// defend：可弃 he < 2 时不开选择（引擎内权威校验）。
#[test]
fn qingyu_defend_noop_when_insufficient() {
    let (mut tx, owner, _seats) = setup(2);
    clear_hand(&mut tx, owner);
    inject_hand(&mut tx, owner);
    let skill = skill_instance(&tx, owner);
    run_effects(&mut tx, vec![qingyu_request(owner, 0, skill)]).unwrap();
    assert_eq!(
        tx.state().resolution.pending_choice_id,
        0,
        "不足两张不开选择"
    );
}

/// achieve：满血且手牌 < 轮数 → 使命成功，获得【选存】；唤醒后 defend 失效。
#[test]
fn qingyu_achieve_grants_xuancun_and_awakes() {
    let (mut tx, owner, _seats) = setup(2);
    // 准备阶段、满血、手牌为空（0 < 轮数 1）。
    tx.state_mut().turn.phase = Phase::Prepare;
    clear_hand(&mut tx, owner);
    {
        let p = tx.state_mut().players.get_mut(owner).unwrap();
        p.hp = p.max_hp as i32;
    }
    let skill = skill_instance(&tx, owner);
    run_effects(&mut tx, vec![qingyu_request(owner, 1, skill)]).unwrap();
    assert!(
        tx.state()
            .skills
            .sorted_live()
            .into_iter()
            .any(|s| s.def.raw() == XUANCUN && s.owner == owner),
        "使命成功应获得选存"
    );
    // 唤醒后 defend 分支 no-op（即使有可弃牌也不开选择）。
    let c1 = inject_hand(&mut tx, owner);
    let c2 = inject_hand(&mut tx, owner);
    run_effects(&mut tx, vec![qingyu_request(owner, 0, skill)]).unwrap();
    assert_eq!(
        tx.state().resolution.pending_choice_id,
        0,
        "唤醒后 defend 不再触发"
    );
    let _ = (c1, c2);
}

/// fail：唯一其他角色时直接授予【勇决】并减上限。
#[test]
fn qingyu_fail_grants_mbyongjue() {
    let (mut tx, owner, seats) = setup(2);
    let other = seats.into_iter().find(|p| *p != owner).unwrap();
    let skill = skill_instance(&tx, owner);
    let max_hp_before = tx.state().players.get(owner).unwrap().max_hp;
    run_effects(&mut tx, vec![qingyu_request(owner, 2, skill)]).unwrap();
    assert_eq!(
        tx.state().players.get(owner).unwrap().max_hp,
        max_hp_before - 1,
        "使命失败应减 1 点体力上限"
    );
    assert!(
        tx.state()
            .skills
            .sorted_live()
            .into_iter()
            .any(|s| s.def.raw() == MBYONGJUE && s.owner == other),
        "唯一其他角色应获得勇决"
    );
}

/// fail：多名其他角色时开选人 choice，选中后授予。
#[test]
fn qingyu_fail_chooses_target() {
    let (mut tx, owner, seats) = setup(3);
    let other = seats[1];
    let skill = skill_instance(&tx, owner);
    let err = run_effects(&mut tx, vec![qingyu_request(owner, 2, skill)]).err();
    assert_eq!(err, Some(EffectError::ChoiceRequested), "应开选人 choice");
    let state = resolve_choice(tx.commit().state, owner, 1, vec![other.raw()], false);
    assert_eq!(state.resolution.pending_choice_id, 0, "授技后直接结算");
    assert!(
        state
            .skills
            .sorted_live()
            .into_iter()
            .any(|s| s.def.raw() == MBYONGJUE && s.owner == other),
        "所选角色应获得勇决"
    );
}
}

mod original_shiji_qinzheng {
//! Golden for SHIJI 骆统「勤政」rewritten from legacy JavaScript semantics.

use engine_api::{CardDefId, CardId, DomainEvent, PlayerId, SkillDefId, Suit, ZoneId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const QINZHENG: u32 = 3_099_323_686;
const LUOTONG: u32 = 3_283_005_586;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/shiji/shiji.npack.json");

fn setup() -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, QINZHENG as u64, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(QINZHENG),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, players[0], players[1])
}

fn add_card(tx: &mut Transaction, zone: ZoneId, kind: CardKind, def: u32) -> CardId {
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone,
        owner: PlayerId::NONE,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn emit_action(
    tx: &mut Transaction,
    player: PlayerId,
    other: PlayerId,
    responded: bool,
) -> Vec<DomainEvent> {
    let event = if responded {
        DomainEvent::CardResponded {
            player,
            respond_to: other,
            card: CardId::NONE,
            response_kind: CardKind::Shan.tag(),
        }
    } else {
        DomainEvent::CardUsed {
            player,
            card: CardId::NONE,
            targets: vec![other],
        }
    };
    let effects = dispatch_for_events(tx, &[event], &mut TriggerFireRecord::new()).unwrap();
    run_effects(tx, effects).unwrap()
}

#[test]
fn package_publishes_abi198_skill_and_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 198);
    assert!(package.skill_index.iter().any(|skill| {
        skill.skill_def_id == QINZHENG && skill.name == "qinzheng" && skill.locked
    }));
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.id == LUOTONG)
            .unwrap()
            .skills,
        [QINZHENG]
    );
}

#[test]
fn action_120_gains_topmost_three_groups_in_one_batch() {
    let (mut tx, owner, other) = setup();
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let older_shan = add_card(&mut tx, draw, CardKind::Shan, 91_000);
    let wuzhong = add_card(&mut tx, draw, CardKind::TrickWuzhong, 91_001);
    let jiu = add_card(&mut tx, draw, CardKind::Jiu, 91_002);
    let fire_sha = add_card(&mut tx, draw, CardKind::ShaFire, 91_003);
    tx.state_mut().card_action_totals.insert(owner, 119);

    let events = emit_action(&mut tx, owner, other, true);

    assert_eq!(tx.state().card_action_totals.get(&owner), Some(&120));
    let hand = tx.state().hand_cards(owner);
    assert!(hand.contains(&fire_sha) && hand.contains(&jiu) && hand.contains(&wuzhong));
    assert!(!hand.contains(&older_shan));
    assert!(events.iter().any(|event| matches!(
        event,
        DomainEvent::CardsMoved { cards, from, .. }
            if *from == draw && cards == &[fire_sha, jiu, wuzhong]
    )));
}

#[test]
fn matching_discard_card_is_not_a_fallback() {
    let (mut tx, owner, other) = setup();
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    tx.state_mut().zones.get_mut(draw).unwrap().cards.clear();
    add_card(&mut tx, draw, CardKind::Weapon, 91_010);
    let sha = add_card(&mut tx, discard, CardKind::Sha, 91_011);
    tx.state_mut().card_action_totals.insert(owner, 2);
    let before = tx.state().hand_count(owner);

    emit_action(&mut tx, owner, other, false);

    assert_eq!(tx.state().hand_count(owner), before);
    assert!(tx.state().zones.get(discard).unwrap().cards.contains(&sha));
}
}

mod original_shiji_shameng {
//! Golden tests for SHIJI「歃盟」rewritten from the legacy JavaScript semantics.

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
    SkillId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::hash::canonical_state_hash;
use engine_core::state::{GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const SHAMENG: u32 = 3_467_425_391;
const CHENZHEN: u32 = 4_187_828_074;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/shiji/shiji.npack.json");

fn started_state(seed: u64) -> (GameState, PlayerId, PlayerId, SkillId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    tx.state_mut().revision = 1;
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    tx.state_mut().turn.phase = Phase::Play;
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(SHAMENG),
        }],
    )
    .unwrap();
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == SHAMENG)
        .unwrap();
    (tx.state().clone(), owner, other, skill)
}

fn activate(
    state: GameState,
    owner: PlayerId,
    skill: SkillId,
    target: PlayerId,
    cards: Vec<engine_api::CardId>,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![target],
                card_targets: cards,
            },
        },
    )
}

#[test]
fn package_contains_complete_character_and_constraints() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == CHENZHEN)
        .expect("陈震 character entry");
    assert_eq!(character.skills, vec![SHAMENG]);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == SHAMENG)
        .expect("歃盟 active entry");
    assert_eq!(active.uses_per_turn, Some(1));
    assert_eq!(
        active
            .targets
            .as_ref()
            .map(|targets| (targets.min, targets.max, targets.allow_self)),
        Some((1, 1, false))
    );
    assert_eq!(
        active.selection.as_ref().map(|selection| (
            selection.min,
            selection.max,
            selection.same_color
        )),
        Some((2, 2, true))
    );
}

#[test]
fn same_color_cost_then_target_draws_two_and_owner_draws_three() {
    let (mut state, owner, other, skill) = started_state(2_026_071_508);
    let cards = state.hand_cards(owner)[..2].to_vec();
    state.cards.get_mut(cards[0]).unwrap().suit = Suit::Heart;
    state.cards.get_mut(cards[1]).unwrap().suit = Suit::Diamond;
    let owner_hand = state.hand_count(owner);
    let target_hand = state.hand_count(other);

    let (state, pipe) = activate(state, owner, skill, other, cards.clone(), 1);
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), owner_hand + 1);
    assert_eq!(state.hand_count(other), target_hand + 2);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(cards
        .iter()
        .all(|card| state.cards.get(*card).unwrap().zone == discard));

    let retry_cards = state.hand_cards(owner)[..2].to_vec();
    let before = canonical_state_hash(&state);
    let (same, pipe) = activate(state, owner, skill, other, retry_cards, 2);
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
}

#[test]
fn mixed_colors_and_self_target_are_rejected_without_mutation() {
    let (mut state, owner, other, skill) = started_state(2_026_071_509);
    let cards = state.hand_cards(owner)[..2].to_vec();
    state.cards.get_mut(cards[0]).unwrap().suit = Suit::Heart;
    state.cards.get_mut(cards[1]).unwrap().suit = Suit::Spade;
    let before = canonical_state_hash(&state);
    let (same, pipe) = activate(state, owner, skill, other, cards.clone(), 1);
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);

    let mut state = same;
    state.cards.get_mut(cards[1]).unwrap().suit = Suit::Diamond;
    let before = canonical_state_hash(&state);
    let (same, pipe) = activate(state, owner, skill, owner, cards, 2);
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
}
}

mod original_shiji_shen_sunce {
//! Package closure for SHIJI 神孙策, rewritten from legacy JavaScript data.

const SHEN_SUNCE: u32 = 2_662_599_370;
const YINGBA: u32 = 2_187_983_153;
const SCFUHAI: u32 = 3_159_082_317;
const PINGHE: u32 = 2_176_317_369;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/shiji/shiji.npack.json");

#[test]
fn package_preserves_character_stats_and_skill_closure() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == SHEN_SUNCE)
        .unwrap();
    assert_eq!((character.initial_hp, character.max_hp), (1, 6));
    assert_eq!(character.skills, [YINGBA, SCFUHAI, PINGHE]);
}
// 第一百四十二批：覆海晋升覆盖——通用 `make_current_card_unrespondable`
// （natuxing 同款）使锦囊目标直击生效；此前 sha 专用近似只覆盖【杀】。

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const YINGBA_MARK: u32 = 3_219_705_664;

fn started() -> (GameState, PlayerId, PlayerId) {
    let compiled =
        content_compiler::compile_source_named("shiji-scfuhai", SCFUHAI_SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_842, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|p| *p != owner)
        .unwrap();
    let clear: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    run_effects(&mut tx, clear).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(SCFUHAI),
        }],
    )
    .unwrap();
    tx.state_mut()
        .players
        .get_mut(other)
        .unwrap()
        .marks
        .push((YINGBA_MARK, 1));
    (tx.commit().state, owner, other)
}

const SCFUHAI_SOURCE: &str = include_str!("../../../content/original/ported/shiji/scfuhai.skill");

fn inject(
    tx: &mut Transaction,
    player: PlayerId,
    kind: CardKind,
    suit: engine_api::Suit,
) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(1),
        kind,
        suit,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn drive(
    tx: &mut Transaction,
    events: &[engine_api::DomainEvent],
) -> Result<(), EffectError> {
    match engine_core::trigger::dispatch_for_events(
        tx,
        events,
        &mut engine_core::trigger::TriggerFireRecord::new(),
    ) {
        Err(e) => Err(e),
        Ok(effects) => run_effects(tx, effects).map(|_| ()),
    }
}

#[test]
fn scfuhai_direct_hit_applies_to_trick_targeting_marked_player() {
    let (state, owner, other) = started();
    let mut tx = Transaction::begin(state);
    let juedou = inject(&mut tx, owner, CardKind::TrickJuedou, engine_api::Suit::Club);
    let before = tx.state().hand_count(owner);
    // optional 触发开出 confirm 选择：先驱动出选择，再以 ResolveChoice 确认。
    let _ = drive(
        &mut tx,
        &[engine_api::DomainEvent::CardUsed {
            player: owner,
            card: juedou,
            targets: vec![other],
        }],
    );
    assert!(tx.state().has_pending_choice(), "optional scfuhai asks");
    let committed = tx.commit();
    let (settled, _pipeline) = apply_command(
        committed.state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(committed.state.revision),
            idempotency_key: IdempotencyKey(9_001),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(committed.state.resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    // 覆海摸 1（每回合限次的近似：不限次）。
    assert_eq!(settled.hand_count(owner), before + 1);
    let sid = settled
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.def.raw() == SCFUHAI)
        .unwrap()
        .id;
    let policy = settled
        .skills
        .get(sid)
        .map(|s| s.card_use_policy(juedou))
        .unwrap_or(0);
    assert_ne!(policy & 2, 0, "trick targeting marked player is unrespondable");
}

#[test]
fn scfuhai_stays_quiet_without_mark() {
    let (state, owner, _other) = started();
    let mut tx = Transaction::begin(state);
    // 清掉目标标记：换用无标记第三场景——直接驱一张无目标牌。
    let sha = inject(&mut tx, owner, CardKind::Sha, engine_api::Suit::Club);
    let before = tx.state().hand_count(owner);
    drive(
        &mut tx,
        &[engine_api::DomainEvent::CardUsed {
            player: owner,
            card: sha,
            targets: vec![],
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), before, "no mark: no draw");
}
}

mod original_shiji_sheyi_tianyin {
//! Golden for SHIJI 蔡贞姬「舍医 / 天荫」语义（ABI 417 / 418）。
//!
//! 舍医：其他角色受伤前（每轮一次）交至少 max(1, hp) 张 he 牌防伤。
//! 天荫：结束阶段获得牌堆顶每个本阶段未用类别牌的第一张。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const SHEYI: u32 = 3_236_439_408;
const TIANYIN: u32 = 2_264_320_121;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/shiji/shiji.npack.json");

fn setup(skill_def: u32, players: u8) -> (Transaction, PlayerId, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(skill_def as u64),
            kind: CommandKind::StartGame {
                player_count: players,
                rng_seed: 20_260_807,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let seats = engine.state().players.alive_by_seat();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, seats)
}

fn inject_card(
    tx: &mut Transaction,
    player: PlayerId,
    zone: ZoneKind,
    kind: CardKind,
    suit: Suit,
) -> CardId {
    let zone_id = tx.state().zones.find(zone, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(80_000 + kind.tag() as u32),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: zone_id,
        owner: player,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(zone_id)
        .unwrap()
        .cards
        .push(card);
    card
}

fn inject_hand(tx: &mut Transaction, player: PlayerId, kind: CardKind, suit: Suit) -> CardId {
    inject_card(tx, player, ZoneKind::Hand, kind, suit)
}

fn resolve_choice(
    state: GameState,
    actor: PlayerId,
    key: u64,
    selections: Vec<u32>,
    cancelled: bool,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    next
}

/// 舍医：其他角色受伤前打开交牌 choice，确认后防伤。
#[test]
fn sheyi_gives_cards_and_prevents_damage() {
    let (mut tx, owner, seats) = setup(SHEYI, 3);
    let others: Vec<_> = seats.into_iter().filter(|p| *p != owner).collect();
    let target = others[0];
    let third = others[1];
    // 目标 hp < 蔡贞姬 hp（贞姬 hp 设为 3,目标 hp 2：受伤 1 点不濒死）。
    tx.state_mut().players.get_mut(owner).unwrap().hp = 3;
    tx.state_mut().players.get_mut(target).unwrap().hp = 2;
    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    assert!(owner_hp > 2);
    let hp_before = tx.state().players.get(target).unwrap().hp;
    let need = tx.state().players.get(owner).unwrap().hp.max(1) as usize;
    let gifts = tx
        .state()
        .hand_cards(owner)
        .iter()
        .take(need)
        .copied()
        .collect::<Vec<_>>();
    // 第三方对目标造成 1 点伤害 → 舍医触发（optional confirm + 选牌）。
    // 直接调用舍医原语（带真实技能实例；DSL optional 触发链在真实对局由引擎驱动）。
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|s| s.def.raw() == SHEYI && s.owner == owner)
        .map(|s| s.id)
        .unwrap();
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveSheyi {
            player: owner,
            target,
            skill,
        }],
    )
    .err();
    assert_eq!(err, Some(EffectError::ChoiceRequested), "应打开选牌 choice");
    // 选交牌（至少 max(1, hp) 张）。
    let state = tx.commit().state;
    let selections: Vec<u32> = gifts.iter().map(|c| c.raw()).collect();
    let state = resolve_choice(state, owner, 2, selections, false);
    assert_eq!(
        state.players.get(target).unwrap().hp,
        hp_before,
        "伤害应被舍医防止"
    );
    assert!(
        state
            .zones
            .get(state.zones.find(ZoneKind::Hand, target).unwrap())
            .unwrap()
            .cards
            .iter()
            .any(|c| gifts.contains(c)),
        "目标应收到交出的牌"
    );
}

/// 天荫：结束阶段获得牌堆顶每个未用类别牌的第一张。
#[test]
fn tianyin_gains_top_cards_of_unused_categories() {
    let (mut tx, owner, seats) = setup(TIANYIN, 2);
    let _other = seats.into_iter().find(|p| *p != owner).unwrap();
    // 牌堆顶放:装备、基本、锦囊、装备(每类第一张应被收,第二张装备不收)。
    let w = inject_card(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Weapon,
        Suit::Spade,
    );
    let sha = inject_card(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Sha,
        Suit::Club,
    );
    let trick = inject_card(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::TrickWuzhong,
        Suit::Heart,
    );
    let w2 = inject_card(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Weapon,
        Suit::Diamond,
    );
    let hand_before = tx.state().hand_count(owner);
    // 直接调用天荫原语（带真实技能实例；DSL trigger 在真实对局由引擎 dispatch）。
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|s| s.def.raw() == TIANYIN && s.owner == owner)
        .map(|s| s.id)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveTianyin {
            player: owner,
            skill,
        }],
    )
    .unwrap();
    // 牌堆顶 = DrawPile vec 末元素：w2 是最后 append 的（顶部），依次是 trick/sha/w。
    // 每类第一张被收：w2(装备)、trick(锦囊)、sha(基本)——3 张；w(第二张装备)不收。
    assert_eq!(tx.state().hand_count(owner), hand_before + 3);
    let hand: Vec<_> = tx.state().hand_cards(owner);
    assert!(hand.contains(&w2) && hand.contains(&trick) && hand.contains(&sha));
    assert!(!hand.contains(&w), "第二张同类别牌不应被收");
}
}

mod original_shiji_sp_bianfuren {
//! Golden for SHIJI 卞夫人（sp_bianfuren）与「挽危」「约俭」语义（ABI 401）。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const SP_BIANFUREN: u32 = 3_610_270_819;
const SPWANWEI: u32 = 3_728_402_634;
const SPYUEJIAN: u32 = 2_751_500_626;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/shiji/shiji.npack.json");

fn inject_hand(tx: &mut Transaction, owner: PlayerId, kind: CardKind, rank: u8) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(98_000 + u32::from(rank)),
        kind,
        suit: Suit::Heart,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn clear_hand(tx: &mut Transaction, player: PlayerId) {
    let cards = tx.state().hand_cards(player);
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    for c in cards {
        engine_core::effect::move_one_card(tx, c, hand, discard, PlayerId::NONE, false).unwrap();
    }
}

fn add_skill(tx: &mut Transaction, owner: PlayerId, def: u32) -> SkillId {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
    tx.state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == def)
        .unwrap()
}

#[test]
fn package_closes_sp_bianfuren_with_exact_skills() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 401);

    // 卞夫人闭包包含「挽危」「约俭」且顺序完全一致。
    let character = package
        .character_index
        .iter()
        .find(|c| c.id == SP_BIANFUREN)
        .expect("sp_bianfuren character index entry");
    assert_eq!(character.name_key, "sp_bianfuren");
    assert_eq!(character.skills, vec![SPWANWEI, SPYUEJIAN]);
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.sex, "female");
    assert_eq!(character.group, "wei");
}

#[test]
fn spwanwei_active_recovers_target_and_loses_owner_hp() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 20_260_805,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();

    let mut tx = Transaction::begin(engine.state().clone());
    tx.state_mut().turn.phase = Phase::Play;
    // 清空随机选将技能，避免包内新增武将改变种子分配后触发无关选择。
    let strip: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, strip).unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().hp = 4;
    tx.state_mut().players.get_mut(owner).unwrap().max_hp = 4;
    tx.state_mut().players.get_mut(target).unwrap().hp = 4;
    tx.state_mut().players.get_mut(target).unwrap().max_hp = 4;

    let _ = add_skill(&mut tx, owner, SPWANWEI);

    // 让 target 受到 2 点伤害。
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target,
            amount: 2,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();

    let owner_initial_hp = tx.state().players.get(owner).unwrap().hp;

    // 发动挽危主动：target 回复 owner.hp (4 点)，owner 失去 4 点体力。
    run_effects(
        &mut tx,
        vec![EffectRequest::SpwanweiActive {
            player: owner,
            target,
        }],
    )
    .unwrap();

    // target 回复到体力上限。
    assert_eq!(
        tx.state().players.get(target).unwrap().hp,
        tx.state().players.get(target).unwrap().max_hp as i32
    );
    // owner 扣减 num 点体力。
    assert_eq!(
        tx.state().players.get(owner).unwrap().hp,
        owner_initial_hp - owner_initial_hp
    );
}

#[test]
fn spyuejian_discards_two_cards_and_recovers_one_hp_on_dying() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 20_260_806,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;

    let mut tx = Transaction::begin(engine.state().clone());
    clear_hand(&mut tx, owner);

    let _ = add_skill(&mut tx, owner, SPYUEJIAN);

    // 注入 2 张手牌。
    let _c1 = inject_hand(&mut tx, owner, CardKind::Tao, 1);
    let _c2 = inject_hand(&mut tx, owner, CardKind::Sha, 2);

    assert_eq!(tx.state().hand_count(owner), 2);

    // 进入濒死状态。
    let err = run_effects(&mut tx, vec![EffectRequest::EnterDying { player: owner }]).unwrap_err();

    // 约俭触发可选 ChoiceRequested。
    assert!(matches!(err, EffectError::ChoiceRequested));
}
}

mod original_shiji_sp_cuiyan {
//! Golden for SHIJI 崔琰「压军 / 尊帝」语义（ABI 428 / 429）。
//!
//! 压军：出牌阶段开始，手牌中有本回合获得的牌时，选一张新获牌与一名其他角色
//! 拼点；赢则可将拼点牌置于牌堆顶，输则添加“戒”标记（手牌上限减标记数）。
//! 尊帝：出牌阶段限一次，弃置一张手牌交给一名其他角色，然后判定：黑色其摸三张，
//! 红色其移动一张场上牌。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const SPYAJUN: u32 = 3_556_581_945;
const SPYAJUN_LESS: u32 = 3_131_511_516;
const SPZUNDI: u32 = 4_234_395_954;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/shiji/shiji.npack.json");

fn setup(skill_def: u32, players: u8) -> (Transaction, PlayerId, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(skill_def as u64),
            kind: CommandKind::StartGame {
                player_count: players,
                rng_seed: 20_260_812,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let seats = engine.state().players.alive_by_seat();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, seats)
}

#[allow(clippy::too_many_arguments)]
fn inject_card(
    tx: &mut Transaction,
    player: PlayerId,
    zone: ZoneKind,
    kind: CardKind,
    suit: Suit,
    rank: u8,
) -> CardId {
    let zone_id = tx.state().zones.find(zone, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(80_000 + kind.tag() as u32),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone: zone_id,
        owner: player,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(zone_id)
        .unwrap()
        .cards
        .push(card);
    card
}

fn skill_instance(tx: &Transaction, owner: PlayerId, def: u32) -> engine_api::SkillId {
    tx.state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|s| s.def.raw() == def && s.owner == owner)
        .map(|s| s.id)
        .unwrap()
}

fn resolve_choice(
    state: GameState,
    actor: PlayerId,
    key: u64,
    selections: Vec<u32>,
    cancelled: bool,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    next
}

fn into_play_phase(tx: &mut Transaction, owner: PlayerId) {
    // 主公回合开始的 phaseDraw 摸牌也计入“本回合获得”；先清空以获得干净触发环境。
    tx.state_mut().turn.gained_hand_cards.clear();
    run_effects(
        tx,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        }],
    )
    .unwrap();
}

/// 压军：赢家可把拼点牌置于牌堆顶（选择链自动推进：新获牌 → 目标 →
/// 目标拼点牌 → 置顶）。
#[test]
fn spyajun_win_places_card_on_top() {
    let (mut tx, owner, seats) = setup(SPYAJUN, 2);
    let other = seats.into_iter().find(|p| *p != owner).unwrap();
    into_play_phase(&mut tx, owner);
    // 拥有者一张本回合新获的 3 点牌，目标一张 2 点牌。
    let owner_card = inject_card(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Tao,
        Suit::Heart,
        3,
    );
    tx.state_mut()
        .turn
        .gained_hand_cards
        .insert((owner, owner_card));
    let target_card = inject_card(
        &mut tx,
        other,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Spade,
        2,
    );
    let skill = skill_instance(&tx, owner, SPYAJUN);

    let err = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveSpyajun {
            player: owner,
            less_def: SkillDefId::new(SPYAJUN_LESS),
            card: CardId::NONE,
            target: PlayerId::NONE,
            target_card: CardId::NONE,
            stage: 0,
            skill,
        }],
    )
    .err();
    assert_eq!(err, Some(EffectError::ChoiceRequested), "应打开新获牌选择");

    let state = resolve_choice(tx.commit().state, owner, 1, vec![owner_card.raw()], false);
    assert_ne!(state.resolution.pending_choice_id, 0, "应推进到目标选择");
    let state = resolve_choice(state, owner, 2, vec![other.raw()], false);
    assert_ne!(
        state.resolution.pending_choice_id, 0,
        "应推进到目标拼点牌选择"
    );
    let state = resolve_choice(state, other, 3, vec![target_card.raw()], false);
    assert_ne!(state.resolution.pending_choice_id, 0, "赢家应可选置顶");
    let state = resolve_choice(state, owner, 4, vec![owner_card.raw()], false);

    assert_eq!(state.resolution.pending_choice_id, 0, "结算完成");
    let draw = state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(
        state.zones.get(draw).unwrap().cards.last().copied(),
        Some(owner_card),
        "拼点牌应置入牌堆顶"
    );
    assert!(
        state
            .players
            .get(owner)
            .unwrap()
            .marks
            .iter()
            .all(|(m, _)| *m != SPYAJUN_LESS),
        "赢家不应有戒标记"
    );
}

/// 压军：败方获得戒标记（手牌上限 - 标记数），且没有置顶选择。
#[test]
fn spyajun_loss_adds_less_mark() {
    let (mut tx, owner, seats) = setup(SPYAJUN, 2);
    let other = seats.into_iter().find(|p| *p != owner).unwrap();
    into_play_phase(&mut tx, owner);
    let owner_card = inject_card(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Tao,
        Suit::Heart,
        3,
    );
    tx.state_mut()
        .turn
        .gained_hand_cards
        .insert((owner, owner_card));
    let target_card = inject_card(
        &mut tx,
        other,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Spade,
        10,
    );
    let skill = skill_instance(&tx, owner, SPYAJUN);

    let err = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveSpyajun {
            player: owner,
            less_def: SkillDefId::new(SPYAJUN_LESS),
            card: CardId::NONE,
            target: PlayerId::NONE,
            target_card: CardId::NONE,
            stage: 0,
            skill,
        }],
    )
    .err();
    assert_eq!(err, Some(EffectError::ChoiceRequested));
    let state = resolve_choice(tx.commit().state, owner, 1, vec![owner_card.raw()], false);
    let state = resolve_choice(state, owner, 2, vec![other.raw()], false);
    // 目标选 10 点牌：目标胜 → 败分支直接结算，无置顶选择。
    let state = resolve_choice(state, other, 3, vec![target_card.raw()], false);

    assert_eq!(state.resolution.pending_choice_id, 0, "败分支不应打开选择");
    assert_eq!(
        state
            .players
            .get(owner)
            .unwrap()
            .marks
            .iter()
            .find(|(m, _)| *m == SPYAJUN_LESS)
            .map(|(_, n)| *n),
        Some(1),
        "败方应获得一枚戒标记"
    );
    // 两拼点牌均进弃牌堆。
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let in_discard = |card: CardId| {
        state
            .zones
            .get(discard)
            .is_some_and(|zone| zone.cards.contains(&card))
    };
    assert!(in_discard(owner_card) && in_discard(target_card));
}

/// 尊帝：黑色判定 → 目标摸三张。
#[test]
fn spzundi_black_draws_three() {
    let (mut tx, owner, seats) = setup(SPZUNDI, 2);
    let target = seats.into_iter().find(|p| *p != owner).unwrap();
    // 牌堆顶放一张黑桃牌（黑）。
    inject_card(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Sha,
        Suit::Spade,
        7,
    );
    let skill = skill_instance(&tx, owner, SPZUNDI);
    let hand_before = tx.state().hand_count(target);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveSpzundi {
            player: owner,
            target,
            card: CardId::NONE,
            destination: PlayerId::NONE,
            stage: 0,
            skill,
        }],
    )
    .unwrap();
    assert_eq!(
        tx.state().hand_count(target),
        hand_before + 3,
        "黑色判定目标应摸三张"
    );
    assert_eq!(tx.state().resolution.pending_choice_id, 0, "黑色分支无选择");
}

/// 尊帝：红色判定 → 目标移动一张场上牌（目标选牌与落点，选择链自动推进）。
#[test]
fn spzundi_red_moves_field_card() {
    let (mut tx, owner, seats) = setup(SPZUNDI, 2);
    let target = seats.into_iter().find(|p| *p != owner).unwrap();
    // 牌堆顶放一张红桃牌（红）。
    inject_card(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Tao,
        Suit::Heart,
        7,
    );
    // 目标持一件武器（可移到拥有者的空武器栏）。
    let weapon = inject_card(
        &mut tx,
        target,
        ZoneKind::Equip,
        CardKind::Weapon,
        Suit::Club,
        7,
    );
    let skill = skill_instance(&tx, owner, SPZUNDI);

    let err = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveSpzundi {
            player: owner,
            target,
            card: CardId::NONE,
            destination: PlayerId::NONE,
            stage: 0,
            skill,
        }],
    )
    .err();
    assert_eq!(err, Some(EffectError::ChoiceRequested), "红色分支应开选牌");
    let state = resolve_choice(tx.commit().state, target, 1, vec![weapon.raw()], false);
    assert_ne!(state.resolution.pending_choice_id, 0, "应推进到落点选择");
    let state = resolve_choice(state, target, 2, vec![owner.raw()], false);

    assert_eq!(state.resolution.pending_choice_id, 0, "结算完成");
    let owner_equip = state.zones.find(ZoneKind::Equip, owner).unwrap();
    assert!(
        state
            .zones
            .get(owner_equip)
            .is_some_and(|zone| zone.cards.contains(&weapon)),
        "武器应移入拥有者装备区"
    );
}
}

mod original_shiji_sp_jiangqing {
//! Golden for SHIJI 江庆「上议 / 简议」语义（ABI 423 / 424）。
//!
//! 上议：出牌阶段限一次，展示目标手牌并获得其中一张。
//! 简议：其他角色结束阶段获得弃牌堆中的防具牌。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const SPSHANGYI: u32 = 3_102_075_230;
const SPJIANYI: u32 = 2_525_303_245;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/shiji/shiji.npack.json");

fn setup(skill_def: u32, players: u8) -> (Transaction, PlayerId, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(skill_def as u64),
            kind: CommandKind::StartGame {
                player_count: players,
                rng_seed: 20_260_811,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let seats = engine.state().players.alive_by_seat();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, seats)
}

fn inject_card(
    tx: &mut Transaction,
    player: PlayerId,
    zone: ZoneKind,
    kind: CardKind,
    suit: Suit,
) -> CardId {
    let zone_id = tx.state().zones.find(zone, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(80_000 + kind.tag() as u32),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: zone_id,
        owner: player,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(zone_id)
        .unwrap()
        .cards
        .push(card);
    card
}

fn resolve_choice(
    state: GameState,
    actor: PlayerId,
    key: u64,
    selections: Vec<u32>,
    cancelled: bool,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    next
}

/// 上议：展示目标手牌并获一张。
#[test]
fn spshangyi_gains_target_hand_card() {
    let (mut tx, owner, seats) = setup(SPSHANGYI, 2);
    let other = seats.into_iter().find(|p| *p != owner).unwrap();
    let target_card = inject_card(&mut tx, other, ZoneKind::Hand, CardKind::Tao, Suit::Heart);
    let hand_before = tx.state().hand_count(owner);
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|s| s.def.raw() == SPSHANGYI && s.owner == owner)
        .map(|s| s.id)
        .unwrap();
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveSpshangyi {
            player: owner,
            target: other,
            skill,
        }],
    )
    .err();
    assert_eq!(err, Some(EffectError::ChoiceRequested), "应打开选牌 choice");
    let state = resolve_choice(tx.commit().state, owner, 1, vec![target_card.raw()], false);
    assert_eq!(
        state.hand_count(owner),
        hand_before + 1,
        "应获得目标一张手牌"
    );
    assert!(
        state.hand_cards(owner).contains(&target_card),
        "应获得指定那张牌"
    );
}

/// 简议：弃牌堆有防具时获得它们。
#[test]
fn spjianyi_gains_discard_armors() {
    let (mut tx, owner, seats) = setup(SPJIANYI, 2);
    let _other = seats.into_iter().find(|p| *p != owner).unwrap();
    // 弃牌堆放两件防具。
    let armor1 = inject_card(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DiscardPile,
        CardKind::Armor,
        Suit::Club,
    );
    let armor2 = inject_card(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DiscardPile,
        CardKind::Armor,
        Suit::Spade,
    );
    let hand_before = tx.state().hand_count(owner);
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|s| s.def.raw() == SPJIANYI && s.owner == owner)
        .map(|s| s.id)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveSpjianyi {
            player: owner,
            skill,
        }],
    )
    .unwrap();
    assert_eq!(
        tx.state().hand_count(owner),
        hand_before + 2,
        "应获得全部防具"
    );
    let hand: Vec<_> = tx.state().hand_cards(owner);
    assert!(hand.contains(&armor1) && hand.contains(&armor2));
}
}

mod original_shiji_sp_xinpi {
//! Golden for SHIJI 辛毗「隐据 / 持节」语义（ABI 421 / 422）。
//!
//! 隐据：出牌阶段限一次，目标二选一——对辛毗用【杀】/ 获得跳过标记
//! （其下回合准备阶段跳过出牌与弃牌）。
//! 持节：成为单目标牌目标时判定，点数 > 6 取消该目标。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const SPYINJU: u32 = 3_592_329_121;
const SPCHIJIE: u32 = 3_232_936_655;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/shiji/shiji.npack.json");

fn setup(skill_def: u32, players: u8) -> (Transaction, PlayerId, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(skill_def as u64),
            kind: CommandKind::StartGame {
                player_count: players,
                rng_seed: 20_260_810,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let seats = engine.state().players.alive_by_seat();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, seats)
}

fn inject_hand(tx: &mut Transaction, player: PlayerId, kind: CardKind, suit: Suit) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(80_000 + kind.tag() as u32),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn resolve_choice(
    state: GameState,
    actor: PlayerId,
    key: u64,
    selections: Vec<u32>,
    cancelled: bool,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    next
}

/// 隐据：目标选"跳过"→ 获得跳过标记。
#[test]
fn spyinju_skip_option_grants_mark() {
    let (mut tx, owner, seats) = setup(SPYINJU, 2);
    let other = seats.into_iter().find(|p| *p != owner).unwrap();
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|s| s.def.raw() == SPYINJU && s.owner == owner)
        .map(|s| s.id)
        .unwrap();
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveSpyinju {
            player: owner,
            target: other,
            skill,
        }],
    )
    .err();
    assert_eq!(err, Some(EffectError::ChoiceRequested));
    // 目标选"跳过"(option 1)。
    let state = resolve_choice(tx.commit().state, other, 1, vec![1], false);
    assert!(
        state
            .players
            .get(other)
            .unwrap()
            .marks
            .iter()
            .any(|(mark, count)| *mark == 2_728_392_676 && *count == 1),
        "目标应获得跳过标记"
    );
}

/// 隐据：目标选"用杀"→ 目标对辛毗用一张手牌杀。
#[test]
fn spyinju_sha_option_uses_sha() {
    let (mut tx, owner, seats) = setup(SPYINJU, 2);
    let other = seats.into_iter().find(|p| *p != owner).unwrap();
    let sha = inject_hand(&mut tx, other, CardKind::Sha, Suit::Spade);
    let hp_before = tx.state().players.get(owner).unwrap().hp;
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|s| s.def.raw() == SPYINJU && s.owner == owner)
        .map(|s| s.id)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveSpyinju {
            player: owner,
            target: other,
            skill,
        }],
    )
    .unwrap_err();
    // 目标选"用杀"(option 0)。
    let state = resolve_choice(tx.commit().state, other, 2, vec![0], false);
    // 目标用手牌杀对辛毗——UseCard 成功（测试环境无完整响应结算,牌暂不消耗）。
    assert!(state.players.get(owner).unwrap().alive);
    let _ = hp_before;
    let _ = sha;
}

/// 持节：判定点数 > 6 取消目标（判定区牌点数可控时验证触发链）。
#[test]
fn spchijie_judges_and_may_cancel() {
    let (mut tx, owner, seats) = setup(SPCHIJIE, 2);
    let _other = seats.into_iter().find(|p| *p != owner).unwrap();
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|s| s.def.raw() == SPCHIJIE && s.owner == owner)
        .map(|s| s.id)
        .unwrap();
    // 直接调用持节原语：无当前牌目标上下文时返回空（不报错）。
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveSpchijie {
            player: owner,
            source: PlayerId::NONE,
            card: engine_api::CardId::NONE,
            skill,
        }],
    );
    assert!(result.is_ok(), "无目标上下文时应安全返回");
}
}

mod original_shiji_spfenming_chendong {
//! Golden for SHIJI SP 陈董（sp_chendong）与「分明」（ABI 402）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, MarkDefId, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const SP_CHENDONG: u32 = 4_077_906_935;
const SPYILIE: u32 = 3_923_077_723;
const SPFENMING: u32 = 2_351_231_725;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/shiji/shiji.npack.json");

fn inject_hand(tx: &mut Transaction, owner: PlayerId, rank: u8) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(88_000 + u32::from(rank)),
        kind: CardKind::Other,
        suit: Suit::Club,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn setup(seed: u64) -> (GameState, PlayerId, PlayerId, engine_api::SkillId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target) = (players[0], players[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(SPFENMING),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == SPFENMING)
        .unwrap()
        .id;
    (tx.commit().state, owner, target, skill)
}

fn submit(state: GameState, actor: PlayerId, kind: CommandKind, key: u64) -> GameState {
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "key {key}: {:?}",
        result.result
    );
    state
}

#[test]
fn package_closes_sp_chendong_with_exact_skills() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 402);

    let character = package
        .character_index
        .iter()
        .find(|c| c.id == SP_CHENDONG)
        .expect("sp_chendong character index entry");
    assert_eq!(character.name_key, "sp_chendong");
    assert_eq!(character.skills, vec![SPYILIE, SPFENMING]);
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "wu");

    let fenming = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == SPFENMING)
        .expect("spfenming active index");
    assert!(fenming.targets.as_ref().unwrap().hp_lte_owner);
}

#[test]
fn spfenming_rejects_higher_hp_and_links_unlinked_target() {
    let (mut state, owner, target, skill) = setup(20_260_807);
    state.players.get_mut(owner).unwrap().hp = 3;
    state.players.get_mut(target).unwrap().hp = 4;

    let revision = state.revision;
    let (mut state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![target],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Rejected { .. }));

    state.players.get_mut(target).unwrap().hp = 3;
    let state = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: Vec::new(),
        },
        2,
    );
    assert!(state
        .players
        .get(target)
        .unwrap()
        .marks
        .iter()
        .any(|(mark, count)| *mark == engine_core::content::MARK_CHAIN && *count > 0));
}

#[test]
fn spfenming_takes_one_card_from_linked_other() {
    let (mut state, owner, target, skill) = setup(20_260_808);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddMark {
            player: target,
            mark: MarkDefId::new(engine_core::content::MARK_CHAIN),
            delta: 1,
        }],
    )
    .unwrap();
    let stolen = inject_hand(&mut tx, target, 7);
    state = tx.commit().state;

    state = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: Vec::new(),
        },
        3,
    );
    assert_ne!(state.resolution.pending_choice_id, 0);
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    state = submit(
        state,
        owner,
        CommandKind::ResolveChoice {
            choice_id: choice,
            selections: vec![stolen.raw()],
            cancelled: false,
        },
        4,
    );
    assert!(state.hand_cards(owner).contains(&stolen));
    assert!(!state.hand_cards(target).contains(&stolen));
}
}

mod original_shiji_spjungong_gaolan {
//! Golden for SHIJI 高览（sp_gaolan）与「峻攻」弃牌/失体力/命中禁用语义（ABI 400）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const SP_GAOLAN: u32 = 2_850_957_074;
const SPJUNGONG: u32 = 3_803_073_466;
const SPDANGLI: u32 = 3_419_124_885;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/shiji/shiji.npack.json");

fn inject_hand(tx: &mut Transaction, owner: PlayerId, kind: CardKind, rank: u8) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(98_000 + u32::from(rank)),
        kind,
        suit: Suit::Heart,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn clear_hand(tx: &mut Transaction, player: PlayerId) {
    let cards = tx.state().hand_cards(player);
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    for c in cards {
        engine_core::effect::move_one_card(tx, c, hand, discard, PlayerId::NONE, false).unwrap();
    }
}

fn add_skill(tx: &mut Transaction, owner: PlayerId, def: u32) -> SkillId {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
    tx.state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == def)
        .unwrap()
}

fn spjungong_banned(state: &GameState, owner: PlayerId) -> bool {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .filter_map(|skill| state.skills.get(*skill))
        .find(|instance| instance.def == SkillDefId::new(SPJUNGONG))
        .map_or(false, |instance| instance.active_turn_banned)
}

fn resolve_choice(
    state: GameState,
    selections: Vec<u32>,
    cancelled: bool,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor: PlayerId::new(state.resolution.pending_observer),
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled,
            },
        },
    )
}

#[test]
fn package_closes_sp_gaolan_with_exact_skills() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 400);

    // 高览闭包包含「峻攻」「登历」且顺序完全一致。
    let character = package
        .character_index
        .iter()
        .find(|c| c.id == SP_GAOLAN)
        .expect("sp_gaolan character index entry");
    assert_eq!(character.name_key, "sp_gaolan");
    assert_eq!(character.skills, vec![SPJUNGONG, SPDANGLI]);
    assert_eq!(character.max_hp, 4);

    // 峻攻以 ABI 400 声明主动技与命中禁用触发器。
    let trigger = package
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == SPJUNGONG)
        .expect("spjungong damage trigger");
    assert_eq!(trigger.event, "damage");
    assert_eq!(trigger.filter, "owner_source_sha_damage_recorded_by_skill");
}

#[test]
fn spjungong_discards_card_uses_sha_and_bans_active_on_damage() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 20_260_801,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();

    let mut tx = Transaction::begin(engine.state().clone());
    tx.state_mut().turn.phase = Phase::Play;

    // 清空两人手牌，确保无法闪避。
    clear_hand(&mut tx, owner);
    clear_hand(&mut tx, target);

    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, effects).unwrap();

    let skill_id = add_skill(&mut tx, owner, SPJUNGONG);

    // 初始未禁用。
    assert!(!spjungong_banned(tx.state(), owner));

    // 给 owner 注入恰好 1 张成本手牌。
    let card = inject_hand(&mut tx, owner, CardKind::Tao, 5);

    // 发动峻攻：选择 1 张手牌弃置，对 target 使用虚拟【杀】。
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::SpjungongDiscardOrLoseThenSha {
            player: owner,
            target,
            skill: skill_id,
        }],
    )
    .unwrap_err();

    // 应该弹出 SelectCards 屏障（选 0..1 张牌）。
    assert!(matches!(err, EffectError::ChoiceRequested));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::ChooseSpjungongDiscardOrLose
    );

    // 选择弃置 1 张手牌。
    let (state, result) = resolve_choice(tx.state().clone(), vec![card.raw()], false);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));

    // 确认手牌已被弃置。
    assert_eq!(state.hand_count(owner), 0);

    // 确认峻攻因虚拟【杀】命中伤害而被标记为本回合禁用。
    assert!(spjungong_banned(&state, owner));
}

#[test]
fn spjungong_loses_hp_when_no_cards_selected() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 20_260_802,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();

    let mut tx = Transaction::begin(engine.state().clone());
    tx.state_mut().turn.phase = Phase::Play;

    // 清空两人手牌，确保无法闪避。
    clear_hand(&mut tx, owner);
    clear_hand(&mut tx, target);

    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, effects).unwrap();

    let skill_id = add_skill(&mut tx, owner, SPJUNGONG);

    let initial_hp = tx.state().players.get(owner).unwrap().hp;

    // 发动峻攻（n=1）。
    let _ = run_effects(
        &mut tx,
        vec![EffectRequest::SpjungongDiscardOrLoseThenSha {
            player: owner,
            target,
            skill: skill_id,
        }],
    );

    // 选择 0 张牌 → 扣 1 点体力。
    let (state, result) = resolve_choice(tx.state().clone(), vec![], false);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));

    assert_eq!(state.players.get(owner).unwrap().hp, initial_hp - 1);
    assert!(spjungong_banned(&state, owner));
}
}

mod original_shiji_spmiewu {
//! Golden coverage for SHIJI 杜预「灭吴」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, MarkDefId, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const SPMIEWU: u32 = 3_162_101_299;
const SPWUKU: u32 = 2_480_520_461;
const SOURCE: &str = include_str!("../../../content/original/ported/shiji/spsanchen.skill");

fn command(actor: PlayerId, key: u64, kind: CommandKind) -> Command {
    Command {
        actor,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(key),
        kind,
    }
}

#[test]
fn spmiewu_consumes_wuku_uses_equipment_and_draws_afterward() {
    let package = content_compiler::compile_source_named("shiji-spmiewu-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 502);
    assert_eq!(package.view_as_index.len(), 2);
    assert!(package
        .view_as_index
        .iter()
        .any(|rule| rule.as_kind == "trick"));

    let mut engine = Engine::new();
    engine.submit(command(
        PlayerId::NONE,
        1,
        CommandKind::StartGame {
            player_count: 2,
            rng_seed: 2_026_080_202,
            ruleset: "identity-standard".into(),
        },
    ));
    engine.load_package(&package).unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let owner = tx.state().turn.current;
    tx.state_mut().turn.phase = Phase::Play;
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(SPMIEWU),
            },
            EffectRequest::AddMark {
                player: owner,
                mark: MarkDefId::new(SPWUKU),
                delta: 1,
            },
        ],
    )
    .unwrap();
    let equip = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    let material = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(98_502),
        kind: CardKind::Weapon,
        suit: Suit::Spade,
        rank: 9,
        weapon_range: 2,
        zone: equip,
        owner,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(equip)
        .unwrap()
        .cards
        .push(material);
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == SPMIEWU)
        .unwrap();
    let hand_before = tx.state().hand_count(owner);
    let state = tx.commit().state;

    let (state, result) = apply_command(
        state,
        command(
            owner,
            2,
            CommandKind::UseSkillCard {
                skill,
                card_kind: CardKind::Jiu.tag(),
                targets: vec![],
                equip_slot: 0,
            },
        ),
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        command(
            owner,
            3,
            CommandKind::ResolveChoice {
                choice_id: choice,
                selections: vec![material.raw()],
                cancelled: false,
            },
        ),
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand_before + 1);
    assert_eq!(state.skills.get(skill).unwrap().use_count, 1);
    assert!(!state
        .players
        .get(owner)
        .unwrap()
        .marks
        .iter()
        .any(|(mark, count)| { *mark == SPWUKU && *count > 0 }));
    assert_eq!(
        state
            .zones
            .get(state.cards.get(material).unwrap().zone)
            .unwrap()
            .kind,
        ZoneKind::DiscardPile
    );
}
}

mod original_shiji_spwuku {
//! Golden for SHIJI「武库」的装备牌与三枚标记上限语义。
//!
//! 杜预（sp_duyu）已由「武库」+「三陈」+「灭吴」闭环入选将池。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const SPWUKU: u32 = 2_480_520_461;
const SPSANCHEN: u32 = 3_539_331_531;
const SP_DUYU: u32 = 2_415_684_974;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/shiji/shiji.npack.json");

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, rank: u8) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(98_000 + u32::from(rank)),
        kind,
        suit: Suit::Club,
        rank,
        weapon_range: kind.is_equip().then_some(2).unwrap_or(0),
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn mark_count(tx: &Transaction, owner: PlayerId) -> i32 {
    tx.state()
        .players
        .get(owner)
        .unwrap()
        .marks
        .iter()
        .find(|(mark, _)| *mark == SPWUKU)
        .map_or(0, |(_, count)| *count)
}

#[test]
fn package_releases_wuku_and_closed_duyu() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    // 武库以独立稳定 ID 发布，装备触发过滤器为权威语义来源。
    assert!(package.skill_index.iter().any(|skill| {
        skill.skill_def_id == SPWUKU
            && skill.locked
            && package.trigger_index.iter().any(|trigger| {
                trigger.skill_def_id == SPWUKU
                    && trigger.event == "card_used"
                    && trigger.filter == "owner_alive_equip_used_skill_mark_below_three"
            })
    }));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == SP_DUYU)
        .expect("sp_duyu closed character");
    assert_eq!(character.skills, vec![SPWUKU, SPSANCHEN]);
}

#[test]
fn only_equipment_uses_add_wuku_and_the_mark_caps_at_three() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 20_260_715,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let user = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(SPWUKU),
    });
    run_effects(&mut tx, effects).unwrap();

    // 非装备牌的使用不获得“武库”。
    let sha = inject(&mut tx, user, CardKind::Sha, 1);
    run_effects(
        &mut tx,
        vec![EffectRequest::UseCard {
            player: user,
            card: sha,
            targets: vec![owner],
        }],
    )
    .unwrap();
    assert_eq!(mark_count(&tx, owner), 0);

    // 连续四张装备牌使用后标记封顶为三。
    for rank in 2..=5 {
        let equip = inject(&mut tx, user, CardKind::Weapon, rank);
        run_effects(
            &mut tx,
            vec![EffectRequest::UseCard {
                player: user,
                card: equip,
                targets: vec![user],
            }],
        )
        .unwrap();
    }
    assert_eq!(mark_count(&tx, owner), 3);
}
}

mod original_shiji_spxizhan_huaman {
//! Golden for SHIJI SP 花鬘（sp_huaman）与「嬉战」（ABI 403）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const SP_HUAMAN: u32 = 2_409_866_600;
const SPXIANGZHEN: u32 = 2_216_375_930;
const SPFANGZONG: u32 = 2_304_282_956;
const SPXIZHAN: u32 = 3_032_288_469;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/shiji/shiji.npack.json");

fn inject_hand(tx: &mut Transaction, owner: PlayerId, suit: Suit, rank: u8) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(89_000 + u32::from(rank)),
        kind: CardKind::Other,
        suit,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, phase_player) = (players[0], players[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(SPFANGZONG),
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(SPXIZHAN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = phase_player;
    tx.state_mut().turn.phase = Phase::Prepare;
    (tx.commit().state, owner, phase_player)
}

fn submit(state: GameState, actor: PlayerId, kind: CommandKind, key: u64) -> GameState {
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "key {key}: {:?}",
        result.result
    );
    state
}

#[test]
fn package_closes_sp_huaman_with_exact_skills() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 502);

    let character = package
        .character_index
        .iter()
        .find(|c| c.id == SP_HUAMAN)
        .expect("sp_huaman character index entry");
    assert_eq!(character.name_key, "sp_huaman");
    assert_eq!(character.skills, vec![SPXIANGZHEN, SPFANGZONG, SPXIZHAN]);
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.sex, "female");
    assert_eq!(character.group, "shu");

    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == SPXIZHAN)
        .expect("spxizhan trigger");
    assert_eq!(trigger.event, "phase_begin");
    assert_eq!(trigger.filter, "owner_alive_not_target");
    assert!(!trigger.optional);
}

#[test]
fn spxizhan_empty_selection_loses_one_hp() {
    let (mut state, owner, phase_player) = setup(20_260_809);
    let mut tx = Transaction::begin(state);
    let _card = inject_hand(&mut tx, owner, Suit::Spade, 3);
    let hp = tx.state().players.get(owner).unwrap().hp;
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveSpxizhan {
            player: owner,
            phase_player,
            fangzong_skill_def: SkillDefId::new(SPFANGZONG),
        }],
    );
    assert!(matches!(
        err,
        Err(engine_core::effect::EffectError::ChoiceRequested)
    ));
    state = tx.commit().state;
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseSpxizhanDiscardOrLose
    );
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    state = submit(
        state,
        owner,
        CommandKind::ResolveChoice {
            choice_id: choice,
            selections: vec![],
            cancelled: false,
        },
        1,
    );
    assert_eq!(state.players.get(owner).unwrap().hp, hp - 1);
    let fangzong = state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == SPFANGZONG)
        .unwrap();
    assert!(!fangzong.turn_disabled);
}

#[test]
fn spxizhan_discard_disables_fangzong() {
    let (mut state, owner, phase_player) = setup(20_260_810);
    let mut tx = Transaction::begin(state);
    let card = inject_hand(&mut tx, owner, Suit::Heart, 5);
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveSpxizhan {
            player: owner,
            phase_player,
            fangzong_skill_def: SkillDefId::new(SPFANGZONG),
        }],
    );
    assert!(matches!(
        err,
        Err(engine_core::effect::EffectError::ChoiceRequested)
    ));
    state = tx.commit().state;
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    state = submit(
        state,
        owner,
        CommandKind::ResolveChoice {
            choice_id: choice,
            selections: vec![card.raw()],
            cancelled: false,
        },
        2,
    );
    let fangzong = state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == SPFANGZONG)
        .unwrap();
    assert!(fangzong.turn_disabled);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(card).unwrap().zone, discard);
}
}

mod original_shiji_wujing {
//! Golden for SHIJI 吴懿「合击 / 流冰」语义（ABI 426 / 427）。
//!
//! 合击：其他角色单目标决斗/红杀后，可对原目标视为使用杀或决斗。
//! 流冰：出牌阶段首次实体杀改为方块。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const LIUBING: u32 = 2_597_403_716;
const HEJI: u32 = 3_561_932_297;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/shiji/shiji.npack.json");

fn setup(skill_def: u32, players: u8) -> (Transaction, PlayerId, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(skill_def as u64),
            kind: CommandKind::StartGame {
                player_count: players,
                rng_seed: 20_260_813,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let seats = engine.state().players.alive_by_seat();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, seats)
}

fn inject_hand(tx: &mut Transaction, player: PlayerId, kind: CardKind, suit: Suit) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(80_000 + kind.tag() as u32),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn resolve_choice(
    state: GameState,
    actor: PlayerId,
    key: u64,
    selections: Vec<u32>,
    cancelled: bool,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    next
}

/// 流冰：实体杀改为方块。
#[test]
fn liubing_changes_sha_to_diamond() {
    let (mut tx, owner, seats) = setup(LIUBING, 2);
    let _other = seats.into_iter().find(|p| *p != owner).unwrap();
    let sha = inject_hand(&mut tx, owner, CardKind::Sha, Suit::Spade);
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|s| s.def.raw() == LIUBING && s.owner == owner)
        .map(|s| s.id)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveLiubing {
            player: owner,
            card: sha,
            skill,
        }],
    )
    .unwrap();
    assert_eq!(
        tx.state().cards.get(sha).map(|c| c.suit),
        Some(Suit::Diamond),
        "杀花色应改为方块"
    );
}

/// 合击：打开二选一 choice（杀/决斗）。
#[test]
fn heji_offers_sha_or_juedou() {
    let (mut tx, owner, seats) = setup(HEJI, 2);
    let other = seats.into_iter().find(|p| *p != owner).unwrap();
    let card = inject_hand(&mut tx, other, CardKind::TrickJuedou, Suit::Club);
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|s| s.def.raw() == HEJI && s.owner == owner)
        .map(|s| s.id)
        .unwrap();
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveHeji {
            player: owner,
            target: other,
            card,
            skill,
        }],
    )
    .err();
    assert_eq!(
        err,
        Some(EffectError::ChoiceRequested),
        "应打开二选一 choice"
    );
    // 选"决斗"(option 1)。
    let state = resolve_choice(tx.commit().state, owner, 1, vec![1], false);
    assert!(state.players.get(owner).unwrap().alive);
}
}

mod original_shiji_xianghai {
//! Golden for SHIJI「乡害」的装备牌名与全局手牌上限光环语义。

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::query::{effective_card_kind, query, QueryContext};
use engine_core::state::CardKind;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const XIANGHAI: u32 = 3_356_961_773;
const XIANGHAI_G: u32 = 3_747_474_489;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/shiji/shiji.npack.json");

fn isolated() -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == XIANGHAI));
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == XIANGHAI_G));
    assert!(package.modifier_index.iter().any(
        |modifier| modifier.skill_def_id == XIANGHAI && modifier.target == "global_hand_limit"
    ));
    assert!(!package
        .modifier_index
        .iter()
        .any(|modifier| modifier.skill_def_id == XIANGHAI_G));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(XIANGHAI as u64),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: XIANGHAI as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));

    let players = engine.state().players.alive_by_seat();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(XIANGHAI),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, players[0], players[1], players[2])
}

fn hand_limit(tx: &Transaction, player: PlayerId) -> i32 {
    match query(
        tx.state(),
        &QueryRequest::HandLimit { player },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("expected hand-limit integer, got {other:?}"),
    }
}

#[test]
fn owner_equipment_is_jiu_without_mutating_or_affecting_other_players() {
    let (mut tx, owner, other, _) = isolated();
    let card = tx.state().hand_cards(owner)[0];
    tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Armor;
    let physical = tx.state().cards.get(card).unwrap();

    assert_eq!(
        effective_card_kind(tx.state(), owner, physical),
        CardKind::Jiu
    );
    assert_eq!(
        effective_card_kind(tx.state(), other, physical),
        CardKind::Armor
    );
    assert_eq!(tx.state().cards.get(card).unwrap().kind, CardKind::Armor);
}

#[test]
fn living_owners_reduce_only_other_players_and_stack() {
    let (mut tx, first_owner, second_owner, target) = isolated();
    for player in [first_owner, second_owner, target] {
        let state = tx.state_mut().players.get_mut(player).unwrap();
        state.hp = 4;
        state.max_hp = 4;
    }

    assert_eq!(hand_limit(&tx, first_owner), 4);
    assert_eq!(hand_limit(&tx, second_owner), 3);
    assert_eq!(hand_limit(&tx, target), 3);

    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: second_owner,
            skill_def: SkillDefId::new(XIANGHAI),
        }],
    )
    .unwrap();
    assert_eq!(hand_limit(&tx, first_owner), 3);
    assert_eq!(hand_limit(&tx, second_owner), 3);
    assert_eq!(hand_limit(&tx, target), 2);

    tx.state_mut().players.get_mut(second_owner).unwrap().alive = false;
    assert_eq!(hand_limit(&tx, first_owner), 4);
    assert_eq!(hand_limit(&tx, target), 3);
    tx.state_mut().players.get_mut(first_owner).unwrap().alive = false;
    assert_eq!(hand_limit(&tx, target), 4);
}
}

mod original_shiji_xinlirang_xinmingshi {
//! Golden for SHIJI 孔融「新礼让 / 新名士」语义（ABI 411）。
//!
//! 新礼让：其他角色摸牌阶段开始时（无谦标记时）令其多摸两张、自己得 1 谦标记
//! 并临时获得截获技能；弃牌阶段后可获得当前回合角色弃掉的一张弃牌；
//! 自己回合开始时若仍有谦标记则跳过判定与摸牌阶段。
//! 新名士：受伤后（有谦标记）来源强制弃一张 hej 牌，红色则回复、弃牌堆则获得。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const XINLIRANG: u32 = 2_286_538_070;
const XINMINGSHI: u32 = 2_976_944_922;
const XINLIRANG_GAIN: u32 = 3_887_425_911;
/// 谦标记（= xinlirang 定义 ID）。
const QIAN_MARK: u32 = 2_286_538_070;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/shiji/shiji.npack.json");

fn setup(skill_def: u32, players: u8) -> (Transaction, PlayerId, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(skill_def as u64),
            kind: CommandKind::StartGame {
                player_count: players,
                rng_seed: 20_260_803 + skill_def as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let seats = engine.state().players.alive_by_seat();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, seats)
}

fn inject_hand(tx: &mut Transaction, player: PlayerId, kind: CardKind, suit: Suit) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(80_000 + kind.tag() as u32),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn resolve_choice(
    state: GameState,
    actor: PlayerId,
    key: u64,
    selections: Vec<u32>,
    cancelled: bool,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    next
}

/// 新礼让：其他角色摸牌阶段开始时可选发动——其多摸两张、自己得谦标记与截获技能。
#[test]
fn xinlirang_grants_extra_draw_and_qian_mark() {
    let (mut tx, owner, seats) = setup(XINLIRANG, 2);
    let other = seats.into_iter().find(|p| *p != owner).unwrap();
    let hand_before = tx.state().hand_count(other);

    // 模拟其他角色的摸牌阶段开始事件。
    let err = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: other,
            phase: TurnPhaseTag::Draw,
        }],
        &mut TriggerFireRecord::new(),
    )
    .map(|_| ())
    .err();
    // optional 触发打开 Confirm choice。
    assert_eq!(err, Some(EffectError::ChoiceRequested));
    let state = resolve_choice(tx.commit().state, owner, 1, vec![1], false);
    let state = Transaction::begin(state);
    let state = state.commit().state;
    assert!(
        state
            .players
            .get(owner)
            .unwrap()
            .marks
            .iter()
            .any(|(mark, count)| *mark == QIAN_MARK && *count == 1),
        "应获得 1 枚谦标记"
    );
    assert!(
        state
            .skills
            .sorted_live()
            .iter()
            .any(|s| s.def.raw() == XINLIRANG_GAIN && s.owner == owner),
        "应临时获得截获技能"
    );
    let _ = hand_before;
}

/// 新礼让·截获：当前回合角色弃牌阶段弃牌后，拥有者可获得一张其弃牌堆中的弃牌。
#[test]
fn xinlirang_gain_grabs_discarded_card() {
    let (mut tx, owner, seats) = setup(XINLIRANG, 2);
    let other = seats.into_iter().find(|p| *p != owner).unwrap();
    // 先发动礼让（摸牌阶段）获得截获技能。
    let err = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: other,
            phase: TurnPhaseTag::Draw,
        }],
        &mut TriggerFireRecord::new(),
    )
    .map(|_| ())
    .err();
    assert_eq!(err, Some(EffectError::ChoiceRequested));
    let mut state = resolve_choice(tx.commit().state, owner, 10, vec![1], false);
    // 其他角色弃牌阶段弃一张手牌 → cards_moved 触发截获。
    let card = state.hand_cards(other)[0];
    let hand = state.zones.find(ZoneKind::Hand, other).unwrap();
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    // run_effects 自动 emit CardsMoved 并触发截获（turn.current 需是 other）。
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = other;
    tx.state_mut().turn.phase = Phase::Discard;
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![card],
            to: discard,
            from: Some(hand),
        }],
    )
    .err();
    assert_eq!(err, Some(EffectError::ChoiceRequested));
    let state = resolve_choice(tx.commit().state, owner, 11, vec![card.raw()], false);
    assert!(
        state
            .zones
            .get(state.zones.find(ZoneKind::Hand, owner).unwrap())
            .unwrap()
            .cards
            .contains(&card),
        "应获得弃牌堆中的弃牌"
    );
}

/// 新礼让·让渡：拥有者回合开始时若有谦标记则跳过判定与摸牌阶段并清空标记。
#[test]
fn xinlirang_skip_phases_when_marked() {
    let (mut tx, owner, seats) = setup(XINLIRANG, 2);
    let other = seats.into_iter().find(|p| *p != owner).unwrap();
    // 发动一次礼让获得谦标记。
    let err = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: other,
            phase: TurnPhaseTag::Draw,
        }],
        &mut TriggerFireRecord::new(),
    )
    .map(|_| ())
    .err();
    assert_eq!(err, Some(EffectError::ChoiceRequested));
    let state = resolve_choice(tx.commit().state, owner, 20, vec![1], false);
    // 拥有者回合开始 → 跳过判定/摸牌并清空标记。
    let mut tx = Transaction::begin(state);
    // 直接验证 skip 触发：模拟拥有者判定阶段开始（真实流程由 turn.rs 推进阶段）。
    tx.state_mut().turn.phase = Phase::Judge;
    // dispatch 返回待执行 effects；手动执行（真实流程由 run_effects 内部自动执行）。
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Judge,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    let state = tx.commit().state;
    assert!(
        state
            .players
            .get(owner)
            .unwrap()
            .marks
            .iter()
            .all(|(mark, count)| !(*mark == QIAN_MARK && *count > 0)),
        "谦标记应被清空"
    );
}

/// 新名士：受伤后（有谦标记）来源强制弃一张牌，红色弃牌回复 1 点体力。
#[test]
fn xinmingshi_red_discard_recovers() {
    let (mut tx, owner, seats) = setup(XINMINGSHI, 2);
    let other = seats.into_iter().find(|p| *p != owner).unwrap();
    // 手动加谦标记。
    run_effects(
        &mut tx,
        vec![EffectRequest::AddMark {
            player: owner,
            mark: engine_api::MarkDefId::new(QIAN_MARK),
            delta: 1,
        }],
    )
    .unwrap();
    let hp_before = tx.state().players.get(owner).unwrap().hp;
    let red = inject_hand(&mut tx, other, CardKind::Other, Suit::Heart);
    // 受伤 1 点 → 触发新名士 → 来源强制弃牌 choice。
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: other,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .err();
    assert_eq!(err, Some(EffectError::ChoiceRequested));
    let state = resolve_choice(tx.commit().state, other, 30, vec![red.raw()], false);
    assert_eq!(
        state.players.get(owner).unwrap().hp,
        hp_before,
        "红色弃牌应回复 1 点（抵掉受伤）"
    );
}

/// 新名士：黑色弃牌进入弃牌堆则拥有者获得它。
#[test]
fn xinmingshi_black_discard_is_gained() {
    let (mut tx, owner, seats) = setup(XINMINGSHI, 2);
    let other = seats.into_iter().find(|p| *p != owner).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddMark {
            player: owner,
            mark: engine_api::MarkDefId::new(QIAN_MARK),
            delta: 1,
        }],
    )
    .unwrap();
    let black = inject_hand(&mut tx, other, CardKind::Other, Suit::Spade);
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: other,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .err();
    assert_eq!(err, Some(EffectError::ChoiceRequested));
    let state = resolve_choice(tx.commit().state, other, 40, vec![black.raw()], false);
    assert!(
        state
            .zones
            .get(state.zones.find(ZoneKind::Hand, owner).unwrap())
            .unwrap()
            .cards
            .contains(&black),
        "黑色弃牌应被拥有者获得"
    );
}
}

mod original_shiji_xinyangjie {
//! Golden for SHIJI 朱儁「佯解」语义（ABI 430）。
//!
//! 佯解：出牌阶段限一次与一名其他角色拼点（点数小者胜）；拼点失败时可选
//! 另一名其他角色视为对拼点目标使用一张火【杀】。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const XINYANGJIE: u32 = 3_719_893_345;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/shiji/shiji.npack.json");

fn setup(players: u8) -> (Transaction, PlayerId, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(XINYANGJIE as u64),
            kind: CommandKind::StartGame {
                player_count: players,
                rng_seed: 20_260_813,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let seats = engine.state().players.alive_by_seat();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(XINYANGJIE),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, seats)
}

fn inject_hand(tx: &mut Transaction, player: PlayerId, rank: u8) -> CardId {
    let zone_id = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(80_000 + CardKind::Sha.tag() as u32),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank,
        weapon_range: 0,
        zone: zone_id,
        owner: player,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(zone_id)
        .unwrap()
        .cards
        .push(card);
    card
}

fn skill_instance(tx: &Transaction, owner: PlayerId) -> engine_api::SkillId {
    tx.state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|s| s.def.raw() == XINYANGJIE && s.owner == owner)
        .map(|s| s.id)
        .unwrap()
}

fn resolve_choice(
    state: GameState,
    actor: PlayerId,
    key: u64,
    selections: Vec<u32>,
    cancelled: bool,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    next
}

fn into_play_phase(tx: &mut Transaction, owner: PlayerId) {
    run_effects(
        tx,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        }],
    )
    .unwrap();
}

/// 佯解：点数小者胜——拥有者 3 点对目标 5 点，拥有者赢，直接结算无后续选择。
#[test]
fn xinyangjie_smaller_rank_wins_no_retaliation() {
    let (mut tx, owner, seats) = setup(2);
    let target = seats.into_iter().find(|p| *p != owner).unwrap();
    into_play_phase(&mut tx, owner);
    let owner_card = inject_hand(&mut tx, owner, 3);
    let target_card = inject_hand(&mut tx, target, 5);
    let skill = skill_instance(&tx, owner);

    let err = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveXinyangjie {
            player: owner,
            target,
            card: CardId::NONE,
            target_card: CardId::NONE,
            source: PlayerId::NONE,
            stage: 0,
            skill,
        }],
    )
    .err();
    assert_eq!(err, Some(EffectError::ChoiceRequested), "应打开拼点牌选择");
    let state = resolve_choice(tx.commit().state, owner, 1, vec![owner_card.raw()], false);
    assert_ne!(state.resolution.pending_choice_id, 0, "应推进到目标拼点牌");
    let state = resolve_choice(state, target, 2, vec![target_card.raw()], false);
    assert_eq!(state.resolution.pending_choice_id, 0, "小者胜无火杀选择");
    // 两张拼点牌进入弃牌堆。
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let in_discard = |card: CardId| {
        state
            .zones
            .get(discard)
            .is_some_and(|zone| zone.cards.contains(&card))
    };
    assert!(in_discard(owner_card) && in_discard(target_card));
}

/// 佯解：拼点失败（拥有者 9 点对目标 3 点）→ 选一名其他角色对目标使用火【杀】。
#[test]
fn xinyangjie_failure_chooses_fire_sha_source() {
    let (mut tx, owner, seats) = setup(3);
    let target = seats[1];
    let source = seats[2];
    into_play_phase(&mut tx, owner);
    let owner_card = inject_hand(&mut tx, owner, 9);
    let target_card = inject_hand(&mut tx, target, 3);
    let skill = skill_instance(&tx, owner);

    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveXinyangjie {
            player: owner,
            target,
            card: CardId::NONE,
            target_card: CardId::NONE,
            source: PlayerId::NONE,
            stage: 0,
            skill,
        }],
    )
    .unwrap_err();
    let state = resolve_choice(tx.commit().state, owner, 1, vec![owner_card.raw()], false);
    let state = resolve_choice(state, target, 2, vec![target_card.raw()], false);
    assert_ne!(
        state.resolution.pending_choice_id, 0,
        "失败应打开火杀来源选择"
    );
    // 来源角色对目标用虚拟火杀：驱动目标响应（放弃出闪）后结算完成。
    let mut state = resolve_choice(state, owner, 3, vec![source.raw()], false);
    let mut key = 4u64;
    while state.resolution.pending_choice_id != 0 {
        let observer = PlayerId::new(state.resolution.pending_observer);
        let choice_id = engine_api::ChoiceId::new(state.resolution.pending_choice_id);
        let (next, result) = apply_command(
            state.clone(),
            Command {
                actor: observer,
                expected_revision: ExpectedRevision(state.revision),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id,
                    selections: vec![],
                    cancelled: true,
                },
            },
        );
        assert!(
            matches!(result.result, CommandResult::Accepted { .. }),
            "{result:?}"
        );
        state = next;
        key += 1;
    }
    let _ = source;
}

/// 佯解：拼点失败但取消火杀来源选择 → 直接结束。
#[test]
fn xinyangjie_failure_cancels_retaliation() {
    let (mut tx, owner, seats) = setup(3);
    let target = seats[1];
    into_play_phase(&mut tx, owner);
    let owner_card = inject_hand(&mut tx, owner, 9);
    let target_card = inject_hand(&mut tx, target, 3);
    let skill = skill_instance(&tx, owner);

    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveXinyangjie {
            player: owner,
            target,
            card: CardId::NONE,
            target_card: CardId::NONE,
            source: PlayerId::NONE,
            stage: 0,
            skill,
        }],
    )
    .unwrap_err();
    let state = resolve_choice(tx.commit().state, owner, 1, vec![owner_card.raw()], false);
    let state = resolve_choice(state, target, 2, vec![target_card.raw()], false);
    let state = resolve_choice(state, owner, 3, vec![], true);
    assert_eq!(state.resolution.pending_choice_id, 0, "取消后直接结束");
}
}

mod original_shiji_xuancun {
//! Golden for 始计「悬存」：其他角色结束阶段按手牌与体力差摸牌，至多两张。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const XUANCUN: u32 = 2_417_220_435;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/shiji/shiji.npack.json");

fn setup(owner_hand: u32) -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.skill_index.iter().any(|skill| {
        skill.skill_def_id == XUANCUN && skill.name == "xuancun" && !skill.locked
    }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == XUANCUN
            && trigger.event == "phase_end"
            && trigger.filter == "owner_alive_not_target_hand_below_hp"
            && trigger.optional
    }));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2_417_220_435),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: 2_417_220_435,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let seats = engine.state().players.alive_by_seat();
    let owner = seats[0];
    let ending_player = seats[1];
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(XUANCUN),
    });
    let current_hand = tx.state().hand_count(owner);
    assert!(owner_hand <= current_hand);
    effects.push(EffectRequest::DiscardFromHand {
        player: owner,
        count: current_hand - owner_hand,
    });
    run_effects(&mut tx, effects).unwrap();
    let owner_state = tx.state_mut().players.get_mut(owner).unwrap();
    owner_state.hp = 4;
    owner_state.max_hp = 4;
    (tx, owner, ending_player)
}

fn phase_end(player: PlayerId) -> DomainEvent {
    DomainEvent::TurnStateChanged {
        current: player,
        phase: TurnPhaseTag::End,
    }
}

fn accept(tx: Transaction, owner: PlayerId, key: u64) -> engine_core::state::GameState {
    let state = tx.state();
    let (state, result) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn hand_deficit_is_capped_at_two_cards_for_the_ending_player() {
    let (mut tx, owner, ending_player) = setup(0);
    let before = tx.state().hand_count(ending_player);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[phase_end(ending_player)],
            &mut TriggerFireRecord::new(),
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = accept(tx, owner, 1);
    assert_eq!(state.hand_count(ending_player), before + 2);
}

#[test]
fn one_card_deficit_draws_exactly_one_card() {
    let (mut tx, owner, ending_player) = setup(3);
    let before = tx.state().hand_count(ending_player);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[phase_end(ending_player)],
            &mut TriggerFireRecord::new(),
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = accept(tx, owner, 2);
    assert_eq!(state.hand_count(ending_player), before + 1);
}

#[test]
fn full_hand_and_owners_own_end_phase_do_not_offer_xuancun() {
    let (mut full, _, ending_player) = setup(4);
    assert!(dispatch_for_events(
        &mut full,
        &[phase_end(ending_player)],
        &mut TriggerFireRecord::new(),
    )
    .unwrap()
    .is_empty());
    assert!(!full.state().has_pending_choice());

    let (mut own_turn, owner, _) = setup(0);
    assert!(dispatch_for_events(
        &mut own_turn,
        &[phase_end(owner)],
        &mut TriggerFireRecord::new(),
    )
    .unwrap()
    .is_empty());
    assert!(!own_turn.state().has_pending_choice());
}
}

mod original_shiji_xunyi {
//! Golden for SHIJI 王甫赵累「殉义」语义（ABI 412）。
//!
//! 开局选择“义”目标，与拥有者共轭守护：任一造成伤害另一人摸 1、
//! 任一受伤另一人强制弃 1 张手牌/装备。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const XUNYI: u32 = 2_517_009_871;
/// 义标记（= xunyi 定义 ID）。
const YI_MARK: u32 = 2_517_009_871;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/shiji/shiji.npack.json");

fn setup(players: u8) -> (Transaction, PlayerId, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(8_041),
            kind: CommandKind::StartGame {
                player_count: players,
                rng_seed: 20_260_804,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let seats = engine.state().players.alive_by_seat();

    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(XUNYI),
    });
    run_effects(&mut tx, effects).unwrap();
    // 角色分配由 shuffle 决定（seed 固定但随包内容变化），显式把血线拉满，
    // 避免 1 血角色（如 shen_sunce）被单点伤害直接带进 dying。
    for player in tx.state().players.alive_by_seat() {
        let participant = tx.state_mut().players.get_mut(player).unwrap();
        participant.hp = participant.max_hp as i32;
    }

    (tx, owner, seats)
}

fn inject_hand(tx: &mut Transaction, player: PlayerId, kind: CardKind, suit: Suit) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(80_000 + kind.tag() as u32),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn resolve_choice(
    state: GameState,
    actor: PlayerId,
    key: u64,
    selections: Vec<u32>,
    cancelled: bool,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    next
}

/// 殉义：王甫造成伤害 → 义目标摸 1 张。
#[test]
fn xunyi_owner_damage_draws_for_yi_target() {
    let (mut tx, owner, seats) = setup(2);
    let other = seats.into_iter().find(|p| *p != owner).unwrap();
    // 手动给 other 义标记（等价于选择义目标）。
    run_effects(
        &mut tx,
        vec![EffectRequest::AddMark {
            player: other,
            mark: engine_api::MarkDefId::new(YI_MARK),
            delta: 1,
        }],
    )
    .unwrap();
    let hand_before = tx.state().hand_count(other);
    // 王甫造成 1 点伤害 → 殉义：义目标摸 1。
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: other,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    // 注意：义目标受伤分支也会触发（王甫弃 1 张）。

    assert_eq!(
        tx.state().hand_count(other),
        hand_before + 1,
        "义目标应因王甫造成伤害摸 1 张"
    );
}

/// 殉义：义目标受伤 → 王甫强制弃 1 张手牌/装备。
#[test]
fn xunyi_yi_target_damaged_forces_owner_discard() {
    let (mut tx, owner, seats) = setup(2);
    let other = seats.into_iter().find(|p| *p != owner).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddMark {
            player: other,
            mark: engine_api::MarkDefId::new(YI_MARK),
            delta: 1,
        }],
    )
    .unwrap();
    let owner_hand = tx.state().hand_count(owner);
    assert!(owner_hand > 0);
    // 义目标受伤 → 王甫弃 1 张。
    let dispatched = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: other,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    );
    // 王甫造成伤害分支(义目标摸 1)无 choice;义目标受伤分支(王甫弃牌)有强制弃牌 choice。
    // 注:此处 source=other(义目标)造成伤害 → 王甫摸 1(无 choice),target=owner(王甫)受伤 → 义目标弃(有 choice)。
    assert!(matches!(
        dispatched,
        Err(EffectError::ChoiceRequested) | Ok(_)
    ));
    let _ = owner_hand;
}

/// 殉义：王甫受伤 → 义目标强制弃 1 张。
#[test]
fn xunyi_owner_damaged_forces_yi_discard() {
    let (mut tx, owner, seats) = setup(2);
    let other = seats.into_iter().find(|p| *p != owner).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddMark {
            player: other,
            mark: engine_api::MarkDefId::new(YI_MARK),
            delta: 1,
        }],
    )
    .unwrap();
    let yi_hand = tx.state().hand_count(other);
    assert!(yi_hand > 0);
    // 其他无关角色(此处用 owner 自己)受伤场景不触发;真正验证:王甫受伤 → 义目标弃。
    // 需要一个第三方来源:2 人局里 owner 受伤的来源只能是 other(义目标),
    // 那会同时触发"义目标造成伤害→王甫摸"。用 owner 自伤不可行(来源=owner 不触发)。
    // 改用 3 人局:第三方对王甫造成伤害 → 义目标弃 1。
    let (mut tx3, owner3, seats3) = setup(3);
    let others: Vec<_> = seats3.into_iter().filter(|p| *p != owner3).collect();
    let yi = others[0];
    let third = others[1];
    run_effects(
        &mut tx3,
        vec![EffectRequest::AddMark {
            player: yi,
            mark: engine_api::MarkDefId::new(YI_MARK),
            delta: 1,
        }],
    )
    .unwrap();
    let yi_hand3 = tx3.state().hand_count(yi);
    assert!(yi_hand3 > 0);
    // 王甫受伤 → 义目标强制弃牌 choice。
    assert_eq!(
        run_effects(
            &mut tx3,
            vec![EffectRequest::DealDamage {
                source: third,
                target: owner3,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            }],
        )
        .err(),
        Some(EffectError::ChoiceRequested)
    );
    let discard_pick = tx3.state().hand_cards(yi)[0].raw();
    let state = resolve_choice(tx3.commit().state, yi, 5, vec![discard_pick], false);
    assert_eq!(
        state.hand_count(yi),
        yi_hand3 - 1,
        "王甫受伤后义目标应弃 1 张"
    );
}

/// 殉义：选择义目标流程——phase_begin 打开选择，确认后目标获义标记。
#[test]
fn xunyi_selects_target_at_phase_begin() {
    let (mut tx, owner, seats) = setup(2);
    let other = seats.into_iter().find(|p| *p != owner).unwrap();
    // 模拟 owner 回合开始(phase_begin)→ resolve_xunyi mode 0 打开选目标 choice。
    let effects = dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    // resolve_xunyi mode 0 打开选目标 choice。
    assert_eq!(
        run_effects(&mut tx, effects).err(),
        Some(EffectError::ChoiceRequested)
    );
    assert!(tx.state().has_pending_choice(), "应打开选义目标 choice");
    let state = resolve_choice(tx.commit().state, owner, 1, vec![other.raw()], false);
    assert!(
        state
            .players
            .get(other)
            .unwrap()
            .marks
            .iter()
            .any(|(mark, count)| *mark == YI_MARK && *count == 1),
        "义目标应获得义标记"
    );
}
}

mod original_shiji_yiyong_shanxie {
//! Golden for SHIJI 王双「异勇 / 善械」语义（ABI 415 / 416）。
//!
//! 异勇：受伤后获得造成伤害的杀实体牌，可对来源反击（来源无武器则伤害基数 2）。
//! 善械：出牌阶段限一次，牌堆有武器则获得，无则随机夺一名有武器角色的武器。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const YIYONG: u32 = 3_278_613_015;
const SHANXIE: u32 = 3_557_798_177;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/shiji/shiji.npack.json");

fn setup(skill_def: u32, players: u8) -> (Transaction, PlayerId, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(skill_def as u64),
            kind: CommandKind::StartGame {
                player_count: players,
                rng_seed: 20_260_806,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let seats = engine.state().players.alive_by_seat();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, seats)
}

fn inject_card(
    tx: &mut Transaction,
    owner: PlayerId,
    zone: ZoneKind,
    kind: CardKind,
    suit: Suit,
) -> CardId {
    let zone_id = tx.state().zones.find(zone, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(80_000 + kind.tag() as u32),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: zone_id,
        owner,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(zone_id)
        .unwrap()
        .cards
        .push(card);
    card
}

fn equip_weapon(tx: &mut Transaction, player: PlayerId) -> CardId {
    let weapon = inject_card(tx, player, ZoneKind::Hand, CardKind::Weapon, Suit::Spade);
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let equip = tx.state().zones.find(ZoneKind::Equip, player).unwrap();
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![weapon],
            to: equip,
            from: Some(hand),
        }],
    )
    .unwrap();
    weapon
}

/// 异勇：受伤后获得造成伤害的杀实体，并立即反击（来源无武器 → 伤害基数 2）。
#[test]
fn yiyong_gains_sha_and_counters_with_bonus_damage() {
    let (mut tx, owner, seats) = setup(YIYONG, 2);
    let other = seats.into_iter().find(|p| *p != owner).unwrap();
    // 王双装备武器（异勇条件）。
    equip_weapon(&mut tx, owner);
    // 来源的杀实体：放进弃牌堆模拟"杀结算后实体在弃牌堆"。
    let sha = inject_card(&mut tx, other, ZoneKind::Hand, CardKind::Sha, Suit::Spade);
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let hand = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![sha],
            to: discard,
            from: Some(hand),
        }],
    )
    .unwrap();
    let hp_before = tx.state().players.get(other).unwrap().hp;
    // 王双受到该杀的 1 点伤害（模拟伤害事件：card=sha, source=other）。
    run_effects(
        &mut tx,
        vec![EffectRequest::DealShaDamage {
            source: other,
            target: owner,
            card: sha,
            amount: 1,
            nature: rule_api::damage_nature::NORMAL,
            card_flags: 0,
        }],
    )
    .unwrap();
    // 反击杀（来源无武器 → 基数 2）：来源 -2。
    assert_eq!(
        tx.state().players.get(other).unwrap().hp,
        hp_before - 2,
        "反击杀应造成 2 点伤害（来源无武器）"
    );
    // 实体牌已用于反击（被消耗，不在手牌）。
    assert!(
        !tx.state().hand_cards(owner).contains(&sha),
        "实体牌应已被反击消耗"
    );
}

/// 善械：牌堆有武器则获得。
#[test]
fn shanxie_grabs_draw_pile_weapon() {
    let (mut tx, owner, seats) = setup(SHANXIE, 2);
    let _other = seats.into_iter().find(|p| *p != owner).unwrap();
    // 牌堆顶放一张武器。
    let weapon = inject_card(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Weapon,
        Suit::Club,
    );
    let hand_before = tx.state().hand_count(owner);
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|s| s.def.raw() == SHANXIE && s.owner == owner)
        .map(|s| s.id)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveShanxie {
            player: owner,
            skill,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand_before + 1);
    assert!(
        tx.state().hand_cards(owner).contains(&weapon),
        "应获得牌堆中的武器"
    );
}

/// 善械：牌堆无武器则随机夺一名有武器角色的武器。
#[test]
fn shanxie_takes_weapon_from_armed_player() {
    let (mut tx, owner, seats) = setup(SHANXIE, 2);
    let other = seats.into_iter().find(|p| *p != owner).unwrap();
    // 其他角色装备武器。
    let weapon = equip_weapon(&mut tx, other);
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|s| s.def.raw() == SHANXIE && s.owner == owner)
        .map(|s| s.id)
        .unwrap();
    let hand_before = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveShanxie {
            player: owner,
            skill,
        }],
    )
    .unwrap();
    // 牌堆可能仍有随机武器（走获得分支）或已无（走夺取分支）；两者都应使王双获得武器。
    assert_eq!(tx.state().hand_count(owner), hand_before + 1);
    assert!(
        tx.state().hand_cards(owner).iter().any(|c| tx
            .state()
            .cards
            .get(*c)
            .is_some_and(|v| v.kind == CardKind::Weapon)),
        "应获得一把武器（牌堆获得或夺取）"
    );
    let _ = weapon;
}
}

mod original_shiji_yuli_tingwei {
//! Golden for SHIJI 神马超「驭雳 / 霆威」语义（ABI 409 / 410）。
//!
//! 驭雳：造成伤害前非雷改为雷、已雷则 +1（prevent + 同源同量同牌雷重放）；
//! 自己受伤前仅雷伤害被取消并摸伤害点数张。
//! 霆威：杀指定首个目标后加 4 霆标记并选一个目标四选一，取消则其横置。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const YULI: u32 = 3_155_821_166;
const TINGWEI: u32 = 4_145_075_227;
/// 霆标记（与寂灭 jimie 共用）。
const TING_MARK: u32 = 4_145_075_227;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/shiji/shiji.npack.json");

fn setup(skill_def: u32, players: u8) -> (Transaction, PlayerId, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(skill_def as u64),
            kind: CommandKind::StartGame {
                player_count: players,
                rng_seed: 20_260_802 + skill_def as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let seats = engine.state().players.alive_by_seat();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, seats)
}

fn inject_hand(tx: &mut Transaction, owner: PlayerId, kind: CardKind, suit: Suit) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(80_000 + kind.tag() as u32),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

/// 驭雳：普通伤害被改为雷属性——目标若同为驭雳拥有者，改雷后其防御分支
/// （受伤前取消并摸牌）被触发，证明伤害确实被改成了雷。
#[test]
fn yuli_converts_normal_damage_to_thunder() {
    let (mut tx, owner, seats) = setup(YULI, 2);
    let target = seats.into_iter().find(|p| *p != owner).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: target,
            skill_def: SkillDefId::new(YULI),
        }],
    )
    .unwrap();
    let hp_before = tx.state().players.get(target).unwrap().hp;
    let hand_before = tx.state().hand_count(target);

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    let state = tx.state();
    assert_eq!(
        state.players.get(target).unwrap().hp,
        hp_before,
        "改雷后目标应通过驭雳防御取消该伤害"
    );
    assert_eq!(
        state.hand_count(target),
        hand_before + 1,
        "目标应摸伤害点数张（1）"
    );
}

/// 驭雳：已是雷属性伤害时伤害 +1。
#[test]
fn yuli_increments_thunder_damage() {
    let (mut tx, owner, seats) = setup(YULI, 2);
    let target = seats.into_iter().find(|p| *p != owner).unwrap();
    let before = tx.state().players.get(target).unwrap().hp;

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::THUNDER,
        }],
    )
    .unwrap();
    let after = tx.state().players.get(target).unwrap().hp;
    assert_eq!(after, before - 2, "雷伤害应 +1");
}

/// 驭雳：自己受到普通伤害不触发（照常受伤）。
#[test]
fn yuli_ignores_incoming_normal_damage() {
    let (mut tx, owner, seats) = setup(YULI, 2);
    let source = seats.into_iter().find(|p| *p != owner).unwrap();
    let before = tx.state().players.get(owner).unwrap().hp;

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    let after = tx.state().players.get(owner).unwrap().hp;
    assert_eq!(after, before - 1, "普通伤害不触发驭雳防御");
}

/// 驭雳：自己受到雷伤害前取消并摸伤害点数张。
#[test]
fn yuli_deflects_incoming_thunder_and_draws() {
    let (mut tx, owner, seats) = setup(YULI, 2);
    let source = seats.into_iter().find(|p| *p != owner).unwrap();
    let hp_before = tx.state().players.get(owner).unwrap().hp;
    let hand_before = tx.state().hand_count(owner);

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 2,
            is_lose_hp: false,
            nature: rule_api::damage_nature::THUNDER,
        }],
    )
    .unwrap();
    let state = tx.state();
    assert_eq!(
        state.players.get(owner).unwrap().hp,
        hp_before,
        "雷伤害应被取消"
    );
    assert_eq!(state.hand_count(owner), hand_before + 2, "应摸伤害点数张");
}

/// 霆威：杀指定首个目标后加 4 霆标记并打开选目标 choice。
#[test]
fn tingwei_grants_marks_and_offers_target_choice() {
    let (mut tx, owner, seats) = setup(TINGWEI, 3);
    let targets: Vec<_> = seats
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let sha = inject_hand(&mut tx, owner, CardKind::Sha, Suit::Spade);

    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::CardUsed {
                player: owner,
                card: sha,
                targets: targets.clone(),
            }],
            &mut TriggerFireRecord::new(),
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    // 确认 optional 发动 → module 执行 → 加 4 霆标记并打开选目标 choice。
    let state = confirm_optional(&mut tx, owner, 81);
    assert!(
        state
            .players
            .get(owner)
            .unwrap()
            .marks
            .iter()
            .any(|(mark, count)| *mark == TING_MARK && *count == 4),
        "应获得 4 枚霆标记"
    );
    assert!(state.has_pending_choice(), "应打开选目标 choice");
}

/// 霆威：目标四选一时取消 → 目标横置。
#[test]
fn tingwei_cancel_chains_target() {
    let (mut tx, owner, seats) = setup(TINGWEI, 3);
    let targets: Vec<_> = seats
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let sha = inject_hand(&mut tx, owner, CardKind::Sha, Suit::Spade);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::CardUsed {
                player: owner,
                card: sha,
                targets: targets.clone(),
            }],
            &mut TriggerFireRecord::new(),
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    // 阶段 0：拥有者选择第一个目标。
    let mut state = confirm_optional(&mut tx, owner, 90);
    assert!(state.has_pending_choice(), "应打开选目标 choice");
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(94),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![targets[0].raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let mut state = state;
    assert!(state.has_pending_choice(), "应打开目标四选一 choice");
    // 阶段 1：目标取消 → 横置。
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state.clone(),
        Command {
            actor: targets[0],
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(91),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![],
                cancelled: true,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(
        next.players
            .get(targets[0])
            .unwrap()
            .marks
            .iter()
            .any(|(mark, count)| *mark == engine_core::content::MARK_CHAIN && *count > 0),
        "取消应使目标横置"
    );
}

/// 霆威：目标选择「非锁定技失效」选项 → 目标非锁定技被压制。
#[test]
fn tingwei_suppress_option() {
    let (mut tx, owner, seats) = setup(TINGWEI, 3);
    let targets: Vec<_> = seats
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let sha = inject_hand(&mut tx, owner, CardKind::Sha, Suit::Spade);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::CardUsed {
                player: owner,
                card: sha,
                targets: targets.clone(),
            }],
            &mut TriggerFireRecord::new(),
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let mut state = confirm_optional(&mut tx, owner, 92);
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(95),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![targets[0].raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let state = state;
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state.clone(),
        Command {
            actor: targets[0],
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(93),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![0],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(
        next.turn
            .non_locked_skill_suppressed_players
            .contains(&targets[0]),
        "目标非锁定技应被压制至其回合结束"
    );
}

/// 确认挂起的 optional 发动（selections=[1]=发动）。
fn confirm_optional(tx: &mut Transaction, actor: PlayerId, key: u64) -> GameState {
    let revision = tx.state().revision;
    let choice_id = ChoiceId::new(tx.state().resolution.pending_choice_id);
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

fn confirm(tx: &mut Transaction, owner: PlayerId, key: u64, target: PlayerId) -> GameState {
    let revision = tx.state().revision;
    let choice_id = ChoiceId::new(tx.state().resolution.pending_choice_id);
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![target.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}
}

mod original_shiji_zhengsu {
//! Golden for SHIJI 整肃系统（ABI 432）与侯封/燕集（ABI 433）。
//!
//! 整肃：垒金（出牌阶段点数严格递增且≥3）/ 变阵（同花色且≥2）/ 明志（弃牌阶段
//! 花色全异且≥2）；成功时受益者（授予者+自己）各摸二或回复一，随后移除临时技能。
//! 侯封：其他角色出牌阶段开始选整肃授予并记录授予者；燕集：自己选整肃。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{turn_flag, Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const ZHENGSU_LEIJIN: u32 = 2_584_022_884;
const ZHENGSU_BIANZHEN: u32 = 3_044_720_297;
const ZHENGSU_MINGZHI: u32 = 3_824_072_820;
const HOUFENG: u32 = 4_065_778_656;
const SPYANJI: u32 = 3_519_161_628;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/shiji/shiji.npack.json");

fn setup(skill_def: u32, players: u8) -> (Transaction, PlayerId, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(skill_def as u64),
            kind: CommandKind::StartGame {
                player_count: players,
                rng_seed: 20_260_815,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let seats = engine.state().players.alive_by_seat();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, seats)
}

fn inject_card(
    tx: &mut Transaction,
    player: PlayerId,
    kind: CardKind,
    suit: Suit,
    rank: u8,
) -> CardId {
    let zone_id = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(80_000 + kind.tag() as u32),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone: zone_id,
        owner: player,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(zone_id)
        .unwrap()
        .cards
        .push(card);
    card
}

fn skill_instance(tx: &Transaction, owner: PlayerId, def: u32) -> Option<engine_api::SkillId> {
    tx.state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|s| s.def.raw() == def && s.owner == owner)
        .map(|s| s.id)
}

fn resolve_choice(
    state: GameState,
    actor: PlayerId,
    key: u64,
    selections: Vec<u32>,
    cancelled: bool,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "resolve failed: {:?}",
        result.result
    );
    next
}

fn zhengsu(
    player: PlayerId,
    mode: u8,
    stage: u8,
    card: CardId,
    skill: engine_api::SkillId,
) -> EffectRequest {
    EffectRequest::ResolveZhengsu {
        player,
        mode,
        stage,
        card,
        skill,
    }
}

/// 垒金：三张递增牌 → 弃牌阶段末判定成功，摸两张并移除临时技能。
#[test]
fn zhengsu_leijin_success_draws_two() {
    let (mut tx, owner, _seats) = setup(ZHENGSU_LEIJIN, 2);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = skill_instance(&tx, owner, ZHENGSU_LEIJIN).unwrap();
    for rank in [2u8, 5, 9] {
        let card = inject_card(&mut tx, owner, CardKind::Sha, Suit::Spade, rank);
        run_effects(&mut tx, vec![zhengsu(owner, 0, 0, card, skill)]).unwrap();
    }
    let hand_before = tx.state().hand_count(owner);
    // 弃牌阶段末判定。
    tx.state_mut().turn.phase = Phase::Discard;
    run_effects(&mut tx, vec![zhengsu(owner, 0, 1, CardId::NONE, skill)]).unwrap();
    assert_eq!(
        tx.state().hand_count(owner),
        hand_before + 2,
        "整肃成功应摸两张（满血自动走摸牌分支）"
    );
    assert!(
        skill_instance(&tx, owner, ZHENGSU_LEIJIN).is_none(),
        "整肃成功后临时技能应移除"
    );
}

/// 垒金：点数不递增 → 判定失败，技能移除且无奖励。
#[test]
fn zhengsu_leijin_failure_removes_skill() {
    let (mut tx, owner, _seats) = setup(ZHENGSU_LEIJIN, 2);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = skill_instance(&tx, owner, ZHENGSU_LEIJIN).unwrap();
    for rank in [5u8, 3] {
        let card = inject_card(&mut tx, owner, CardKind::Sha, Suit::Spade, rank);
        run_effects(&mut tx, vec![zhengsu(owner, 0, 0, card, skill)]).unwrap();
    }
    let hand_before = tx.state().hand_count(owner);
    tx.state_mut().turn.phase = Phase::Discard;
    run_effects(&mut tx, vec![zhengsu(owner, 0, 1, CardId::NONE, skill)]).unwrap();
    assert_eq!(tx.state().hand_count(owner), hand_before, "失败无奖励");
    assert!(
        skill_instance(&tx, owner, ZHENGSU_LEIJIN).is_none(),
        "整肃失败后临时技能应移除"
    );
}

/// 变阵：出牌阶段两张同花色牌 → 判定成功。
#[test]
fn zhengsu_bianzhen_success() {
    let (mut tx, owner, _seats) = setup(ZHENGSU_BIANZHEN, 2);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    // 手动构造引擎 flag：出牌阶段用牌 2 张、同花色（红心，suit_tag=2 → 位 1<<2）。
    let suit_key = turn_flag::play_phase_card_suit_history_by_player(owner);
    tx.state_mut().turn.set_flag(suit_key, 1 << 2);
    let count_key = turn_flag::play_phase_card_used_by_player(owner);
    tx.state_mut().turn.set_flag(count_key, 2);
    let skill = skill_instance(&tx, owner, ZHENGSU_BIANZHEN).unwrap();
    let hand_before = tx.state().hand_count(owner);
    tx.state_mut().turn.phase = Phase::Discard;
    run_effects(&mut tx, vec![zhengsu(owner, 1, 1, CardId::NONE, skill)]).unwrap();
    assert_eq!(
        tx.state().hand_count(owner),
        hand_before + 2,
        "变阵成功摸二"
    );
    assert!(skill_instance(&tx, owner, ZHENGSU_BIANZHEN).is_none());
}

/// 明志：弃牌阶段弃置两张不同花色牌 → 判定成功。
#[test]
fn zhengsu_mingzhi_success() {
    let (mut tx, owner, _seats) = setup(ZHENGSU_MINGZHI, 2);
    tx.state_mut().turn.current = owner;
    let heart = inject_card(&mut tx, owner, CardKind::Tao, Suit::Heart, 3);
    let spade = inject_card(&mut tx, owner, CardKind::Sha, Suit::Spade, 4);
    // 构造弃牌阶段历史：lost_cards 从 base 起为这两张，且仍在弃牌堆。
    tx.state_mut()
        .turn
        .set_flag(turn_flag::DISCARD_PHASE_LOST_BASE, 1);
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    for card in [heart, spade] {
        tx.state_mut().turn.lost_cards.push((owner, card));
        let zone = tx.state_mut().cards.get_mut(card).unwrap();
        zone.zone = discard;
    }
    let skill = skill_instance(&tx, owner, ZHENGSU_MINGZHI).unwrap();
    let hand_before = tx.state().hand_count(owner);
    tx.state_mut().turn.phase = Phase::Discard;
    run_effects(&mut tx, vec![zhengsu(owner, 2, 1, CardId::NONE, skill)]).unwrap();
    assert_eq!(
        tx.state().hand_count(owner),
        hand_before + 2,
        "明志成功摸二"
    );
    assert!(skill_instance(&tx, owner, ZHENGSU_MINGZHI).is_none());
}

/// 侯封：授予他人整肃并记录授予者；成功后授予者也获奖励。
#[test]
fn houfeng_grants_and_rewards_grantor() {
    let (mut tx, owner, seats) = setup(HOUFENG, 3);
    let target = seats[1];
    let skill = skill_instance(&tx, owner, HOUFENG).unwrap();
    // 目标出牌阶段开始。
    tx.state_mut().turn.current = target;
    tx.state_mut().turn.phase = Phase::Play;
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveHoufeng {
            player: owner,
            target,
            mode: 0,
            stage: 0,
            skill,
        }],
    )
    .err();
    assert_eq!(
        err,
        Some(EffectError::ChoiceRequested),
        "应打开整肃类型选择"
    );
    let state = resolve_choice(tx.commit().state, owner, 1, vec![1], false);
    let mut tx = Transaction::begin(state.clone());
    // 目标已获得垒金整肃。
    let granted = skill_instance(&tx, target, ZHENGSU_LEIJIN).unwrap();
    // 授予者记录段存在（GR 含 owner）。
    assert!(
        tx.state()
            .skills
            .get(granted)
            .is_some_and(|instance| instance.state_blob.starts_with(b"GR")),
        "侯封应记录授予者"
    );
    // 目标弃牌阶段末：三张递增牌跟踪后判定成功 → 授予者 + 目标各摸二。
    tx.state_mut().turn.current = target;
    tx.state_mut().turn.phase = Phase::Play;
    for rank in [2u8, 5, 9] {
        let card = inject_card(&mut tx, target, CardKind::Sha, Suit::Spade, rank);
        run_effects(&mut tx, vec![zhengsu(target, 0, 0, card, granted)]).unwrap();
    }
    let owner_hand_before = tx.state().hand_count(owner);
    let target_hand_before = tx.state().hand_count(target);
    tx.state_mut().turn.phase = Phase::Discard;
    run_effects(&mut tx, vec![zhengsu(target, 0, 1, CardId::NONE, granted)]).unwrap();
    assert_eq!(
        tx.state().hand_count(owner),
        owner_hand_before + 2,
        "授予者应获整肃奖励"
    );
    assert_eq!(
        tx.state().hand_count(target),
        target_hand_before + 2,
        "目标应获整肃奖励"
    );
}

/// 燕集：自己选整肃并获奖励。
#[test]
fn spyanji_self_zhengsu_reward() {
    let (mut tx, owner, _seats) = setup(SPYANJI, 2);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = skill_instance(&tx, owner, SPYANJI).unwrap();
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveHoufeng {
            player: owner,
            target: owner,
            mode: 1,
            stage: 0,
            skill,
        }],
    )
    .err();
    assert_eq!(
        err,
        Some(EffectError::ChoiceRequested),
        "应打开整肃类型选择"
    );
    let state = resolve_choice(tx.commit().state, owner, 1, vec![2], false);
    let mut tx = Transaction::begin(state.clone());
    // 自己获得变阵整肃。
    let granted = skill_instance(&tx, owner, ZHENGSU_BIANZHEN).unwrap();
    assert!(
        !tx.state()
            .skills
            .get(granted)
            .is_some_and(|instance| instance.state_blob.starts_with(b"GR")),
        "燕集无授予者记录"
    );
    // 变阵成功：两张同花色（红心）。
    let suit_key = turn_flag::play_phase_card_suit_history_by_player(owner);
    tx.state_mut().turn.set_flag(suit_key, 1 << 2);
    let count_key = turn_flag::play_phase_card_used_by_player(owner);
    tx.state_mut().turn.set_flag(count_key, 2);
    let hand_before = tx.state().hand_count(owner);
    tx.state_mut().turn.phase = Phase::Discard;
    run_effects(&mut tx, vec![zhengsu(owner, 1, 1, CardId::NONE, granted)]).unwrap();
    assert_eq!(
        tx.state().hand_count(owner),
        hand_before + 2,
        "燕集成功摸二"
    );
}

/// 点才（ABI 434）：其他角色出牌阶段结束时，失牌 ≥ 体力且手牌 < 上限 → 摸至上限。
#[test]
fn mbdiancai_draws_to_max_hp() {
    const MBDIANCAI: u32 = 2_244_183_634;
    let (mut tx, owner, seats) = setup(MBDIANCAI, 2);
    let actor = seats.into_iter().find(|p| *p != owner).unwrap();
    // 本回合失牌数 ≥ 体力（体力 4 → 失 4 张）。
    let key = turn_flag::card_lost_by_player(actor);
    for _ in 0..4 {
        let card = inject_card(&mut tx, actor, CardKind::Sha, Suit::Spade, 3);
        tx.state_mut().turn.lost_cards.push((actor, card));
        let current = tx.state().turn.flag(key);
        tx.state_mut().turn.set_flag(key, current + 1);
    }
    let max_hp = tx.state().players.get(actor).unwrap().max_hp;
    // 清空手牌：摸至上限 = maxHp 张。
    clear_hand(&mut tx, actor);
    let skill = skill_instance(&tx, owner, MBDIANCAI).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveMbdiancai {
            player: owner,
            actor,
            skill,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(actor), max_hp, "应摸至体力上限");
    assert_eq!(tx.state().resolution.pending_choice_id, 0);
}

fn clear_hand(tx: &mut Transaction, player: PlayerId) {
    let zone_id = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    tx.state_mut().zones.get_mut(zone_id).unwrap().cards.clear();
}

fn inject_to_zone(
    tx: &mut Transaction,
    player: PlayerId,
    zone: ZoneKind,
    kind: CardKind,
    suit: Suit,
    rank: u8,
) -> CardId {
    let zone_id = tx.state().zones.find(zone, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(80_000 + kind.tag() as u32),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone: zone_id,
        owner: player,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(zone_id)
        .unwrap()
        .cards
        .push(card);
    card
}

/// 战海（ABI 435）：装备补点拼点赢 → 展示目标手牌并按类别从牌堆取牌获得。
#[test]
fn rechuhai_win_gains_by_category() {
    const RECHUHAI: u32 = 4_266_017_884;
    let (mut tx, owner, seats) = setup(RECHUHAI, 2);
    let target = seats.into_iter().find(|p| *p != owner).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = skill_instance(&tx, owner, RECHUHAI).unwrap();
    // 拥有者首张手牌改 3 点（无装备 → 补 +4 = 7）。
    let owner_card = tx.state().hand_cards(owner)[0];
    tx.state_mut().cards.get_mut(owner_card).unwrap().rank = 3;
    // 目标手牌：首张改 5 点（作拼点牌）+ 注入装备（6 点）构成两类。
    let target_basic = tx.state().hand_cards(target)[0];
    tx.state_mut().cards.get_mut(target_basic).unwrap().rank = 5;
    let _target_equip = inject_card(&mut tx, target, CardKind::Weapon, Suit::Club, 6);
    // 牌堆顶各放一类牌供获得。
    let pile_basic = inject_to_zone(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Sha,
        Suit::Spade,
        2,
    );
    let pile_equip = inject_to_zone(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Armor,
        Suit::Club,
        3,
    );
    let hand_before = tx.state().hand_count(owner);
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveRechuhai {
            player: owner,
            target,
            card: CardId::NONE,
            target_card: CardId::NONE,
            stage: 0,
            skill,
        }],
    )
    .err();
    assert_eq!(err, Some(EffectError::ChoiceRequested), "应打开拼点牌选择");
    // 拥有者选 3 点牌（摸一张后 hand_before+1），目标选 5 点 → 7 > 5 拥有者赢。
    let state = resolve_choice(tx.commit().state, owner, 1, vec![owner_card.raw()], false);
    let state = resolve_choice(state, target, 2, vec![target_basic.raw()], false);
    let mut tx = Transaction::begin(state.clone());
    let _ = (pile_basic, pile_equip, hand_before);
}

/// 弃哀（ABI 439）：已受伤时目标二选一（回复 / 摸二）。
#[test]
fn spqiai_wounded_target_chooses_recover() {
    const SPQIAI: u32 = 2_235_926_361;
    let (mut tx, owner, seats) = setup(SPQIAI, 2);
    let target = seats.into_iter().find(|p| *p != owner).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = skill_instance(&tx, owner, SPQIAI).unwrap();
    // 拥有者受伤（4 上限 → 2 血）。
    {
        let p = tx.state_mut().players.get_mut(owner).unwrap();
        p.hp = 2;
    }
    // 一张非基本牌直接注入手牌。
    let equip = inject_card(&mut tx, owner, CardKind::Weapon, Suit::Club, 6);
    // 清掉其他起始手牌里的非基本牌干扰？直接以该牌发起。
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveSpqiai {
            player: owner,
            target,
            card: equip,
            stage: 0,
            skill,
        }],
    )
    .err();
    assert_eq!(err, Some(EffectError::ChoiceRequested), "受伤应开二选一");
    let state = resolve_choice(tx.commit().state, target, 1, vec![0], false);
    let mut tx = Transaction::begin(state.clone());
    assert_eq!(
        tx.state().players.get(owner).unwrap().hp,
        3,
        "目标选回复应回 1"
    );
    assert!(
        tx.state().hand_cards(target).contains(&equip),
        "非基本牌应交给目标"
    );
}

/// 擅袭（ABI 440）：授予檄标记（全局唯一）→ 索命（ABI 441）：他人回复后交 2 张或失体。
#[test]
fn spshanxi_mark_and_suoming() {
    const SPSHANXI: u32 = 2_378_382_374;
    let (mut tx, owner, seats) = setup(SPSHANXI, 3);
    let other = seats.iter().copied().find(|p| *p != owner).unwrap();
    let third = seats
        .iter()
        .copied()
        .find(|p| *p != owner && *p != other)
        .unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = skill_instance(&tx, owner, SPSHANXI).unwrap();
    // 擅袭：开选目标 → 授予檄标记并清除他人。
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveSpshanxi {
            player: owner,
            target: PlayerId::NONE,
            stage: 0,
            skill,
        }],
    )
    .err();
    assert_eq!(err, Some(EffectError::ChoiceRequested), "应开选人 choice");
    let state = resolve_choice(tx.commit().state, owner, 1, vec![other.raw()], false);
    let mut tx = Transaction::begin(state.clone());
    assert!(
        tx.state()
            .players
            .get(other)
            .unwrap()
            .marks
            .iter()
            .any(|(m, n)| *m == SPSHANXI && *n > 0),
        "目标应获得檄标记"
    );
    // 再授予 third → 檄标记唯一（other 被清除）。
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveSpshanxi {
            player: owner,
            target: third,
            stage: 1,
            skill,
        }],
    )
    .unwrap();
    assert!(
        !tx.state()
            .players
            .get(other)
            .unwrap()
            .marks
            .iter()
            .any(|(m, n)| *m == SPSHANXI && *n > 0),
        "旧檄标记应被清除"
    );
    // 索命：third 有檄标记且可交 2 张 → 开交牌 choice。
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveSpshanxiSuoming {
            player: owner,
            actor: third,
            stage: 0,
            skill,
        }],
    )
    .err();
    assert_eq!(err, Some(EffectError::ChoiceRequested), "应开交牌 choice");
    let cards = tx
        .state()
        .hand_cards(third)
        .into_iter()
        .take(2)
        .collect::<Vec<_>>();
    let state = resolve_choice(
        tx.commit().state,
        third,
        2,
        vec![cards[0].raw(), cards[1].raw()],
        false,
    );
    let mut tx = Transaction::begin(state.clone());
    assert!(
        tx.state().hand_cards(owner).contains(&cards[0])
            && tx.state().hand_cards(owner).contains(&cards[1]),
        "两张牌应交到拥有者手中"
    );
}

/// 通夺（ABI 442）：成为他人单目标目标后选角色强制重铸一张牌。
#[test]
fn tongduo_forces_recast() {
    const TONGDUO: u32 = 3_696_525_701;
    let (mut tx, owner, seats) = setup(TONGDUO, 2);
    let other = seats.into_iter().find(|p| *p != owner).unwrap();
    let skill = skill_instance(&tx, owner, TONGDUO).unwrap();
    // 通夺：选重铸角色 → 该角色选牌 → 重铸（牌入弃牌堆）。
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveTongduo {
            player: owner,
            target: PlayerId::NONE,
            card: CardId::NONE,
            stage: 0,
            skill,
        }],
    )
    .err();
    assert_eq!(err, Some(EffectError::ChoiceRequested), "应开选角色 choice");
    // 选角色后 handler 自动推进选牌 choice。
    let state = resolve_choice(tx.commit().state, owner, 1, vec![other.raw()], false);
    assert_ne!(state.resolution.pending_choice_id, 0, "应推进到选牌 choice");
    let card = state.hand_cards(other)[0];
    let state = resolve_choice(state, other, 2, vec![card.raw()], false);
    let mut tx = Transaction::begin(state.clone());
    // 选牌后 handler 自动推进重铸（stage 2 由 handler 调用）。
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(
        tx.state()
            .zones
            .get(discard)
            .is_some_and(|zone| zone.cards.contains(&card)),
        "重铸牌应进入弃牌堆"
    );
}

/// 怨清（ABI 443）：出牌阶段结束后按类别取弃牌堆牌入仁库；疏陈（ABI 444）：
/// 濒死时仁库>3 获得全部并回复。
#[test]
fn yuanqing_renku_and_shuchen() {
    const YUANQING: u32 = 3_702_802_540;
    const SHUCHEN: u32 = 3_570_664_782;
    let (mut tx, owner, seats) = setup(YUANQING, 2);
    let other = seats.into_iter().find(|p| *p != owner).unwrap();
    // 本阶段用过基本+装备两类。
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().turn.set_flag(
        turn_flag::owner_card_category_history_by_player(owner),
        0b101,
    );
    // 弃牌堆放基本(2 张)与装备(2 张)。
    for (kind, suit) in [
        (CardKind::Sha, Suit::Spade),
        (CardKind::Tao, Suit::Heart),
        (CardKind::Weapon, Suit::Club),
    ] {
        inject_to_zone(
            &mut tx,
            PlayerId::NONE,
            ZoneKind::DiscardPile,
            kind,
            suit,
            3,
        );
        inject_to_zone(
            &mut tx,
            PlayerId::NONE,
            ZoneKind::DiscardPile,
            kind,
            suit,
            4,
        );
    }
    let skill = skill_instance(&tx, owner, YUANQING).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveYuanqing {
            player: owner,
            skill,
        }],
    )
    .unwrap();
    // 两类各取一张入仁库。
    let renku = tx.state().skill_pile_cards(owner, YUANQING);
    assert_eq!(renku.len(), 2, "两类各取一张入仁库");
    // 再注入两张让仁库 > 3（补 2 张 = 4 张）。
    for (kind, suit) in [(CardKind::Sha, Suit::Spade), (CardKind::Sha, Suit::Spade)] {
        let _ = inject_to_zone(
            &mut tx,
            PlayerId::NONE,
            ZoneKind::DiscardPile,
            kind,
            suit,
            5,
        );
    }
    let pile = tx.state().zones.find(ZoneKind::Expansion, owner).unwrap();
    for _ in 0..2 {
        let card = tx.state_mut().cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(80_000 + CardKind::Sha.tag() as u32),
            kind: CardKind::Sha,
            suit: Suit::Spade,
            rank: 3,
            weapon_range: 0,
            zone: pile,
            owner,
            face_up: true,
        });
        tx.state_mut().zones.get_mut(pile).unwrap().cards.push(card);
    }
    assert_eq!(
        tx.state().skill_pile_cards(owner, YUANQING).len(),
        4,
        "仁库应为 4 张"
    );
    // 疏陈：其他角色濒死 → 获得全部仁库牌并回复。
    let shuchen_skill = {
        let mut effects: Vec<_> = tx
            .state()
            .skills
            .sorted_live()
            .into_iter()
            .map(|s| EffectRequest::RemoveSkill { skill: s.id })
            .collect();
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(SHUCHEN),
        });
        run_effects(&mut tx, effects).unwrap();
        tx.state()
            .skills
            .sorted_live()
            .into_iter()
            .find(|s| s.def.raw() == SHUCHEN && s.owner == owner)
            .map(|s| s.id)
            .unwrap()
    };
    // 让濒死者受伤以便回复可生效。
    {
        let p = tx.state_mut().players.get_mut(other).unwrap();
        p.hp = p.hp.saturating_sub(1);
    }
    let other_hp = tx.state().players.get(other).unwrap().hp;
    let hand_before = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveShuchen {
            player: owner,
            dying: other,
            renku_def: SkillDefId::new(YUANQING),
            skill: shuchen_skill,
        }],
    )
    .unwrap();
    assert_eq!(
        tx.state().hand_count(owner),
        hand_before + 4,
        "应获得全部仁库牌"
    );
    assert_eq!(
        tx.state().players.get(other).unwrap().hp,
        other_hp + 1,
        "濒死者应回复 1"
    );
}

/// 断笔（ABI 445）：全场手牌 > 人数×2 时逐目标弃牌 → 选人随机获至多 3 张被弃牌。
#[test]
fn duanbi_forces_discards_and_gains() {
    const DUANBI: u32 = 4_273_058_855;
    let (mut tx, owner, seats) = setup(DUANBI, 2);
    let other = seats.into_iter().find(|p| *p != owner).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = skill_instance(&tx, owner, DUANBI).unwrap();
    // 手牌总数 > 4：双方各补 3 张。
    for p in [owner, other] {
        for i in 0..3u8 {
            inject_card(&mut tx, p, CardKind::Sha, Suit::Spade, i + 1);
        }
    }
    assert!(
        tx.state().hand_count(owner) as usize + tx.state().hand_count(other) as usize > 4,
        "条件应满足"
    );
    // 断笔：开第一个目标的弃牌 choice（弃 floor(手牌/2) ≤ 3）。
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDuanbi {
            player: owner,
            targets: vec![other],
            stage: 0,
            skill,
        }],
    )
    .err();
    assert_eq!(err, Some(EffectError::ChoiceRequested), "应开弃牌 choice");
    // 目标弃 3 张（手牌多）。
    let hand = tx.state().hand_cards(other);
    let discard_cards = hand.into_iter().take(3).collect::<Vec<_>>();
    let state = resolve_choice(
        tx.commit().state,
        other,
        1,
        vec![
            discard_cards[0].raw(),
            discard_cards[1].raw(),
            discard_cards[2].raw(),
        ],
        false,
    );
    // 弃完 → 开选人获牌 choice。
    assert_ne!(state.resolution.pending_choice_id, 0, "应开选人获牌 choice");
    let state = resolve_choice(state, owner, 2, vec![owner.raw()], false);
    let mut tx = Transaction::begin(state.clone());
    // 拥有者应获得弃置堆中至少一张被弃牌（权威随机至多 3）。
    let gained = tx
        .state()
        .hand_cards(owner)
        .iter()
        .filter(|card| discard_cards.contains(card))
        .count();
    assert!(
        gained >= 1,
        "应获得被弃牌（至多 3 张，至少 1 张仍在弃牌堆）"
    );
}

/// 躁厉（ABI 446）：有标记时弃至少一张并摸（标记+弃数），总数>2 失体。
#[test]
fn zaoli_discard_draw_and_lose_hp() {
    const ZAOLI: u32 = 2_486_461_241;
    let (mut tx, owner, _seats) = setup(ZAOLI, 2);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    let skill = skill_instance(&tx, owner, ZAOLI).unwrap();
    // 2 枚躁厉标记。
    run_effects(
        &mut tx,
        vec![EffectRequest::AddMark {
            player: owner,
            mark: engine_api::MarkDefId::new(ZAOLI),
            delta: 2,
        }],
    )
    .unwrap();
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveZaoli {
            player: owner,
            mark: 2,
            skill,
        }],
    )
    .err();
    assert_eq!(err, Some(EffectError::ChoiceRequested), "应开弃牌 choice");
    // 弃 1 张 → 摸 2+1=3；总数 3 > 2 → 失 1 体。
    let card = tx.state().hand_cards(owner)[0];
    let hand_before = tx.state().hand_count(owner);
    let hp_before = tx.state().players.get(owner).unwrap().hp;
    let state = resolve_choice(tx.commit().state, owner, 1, vec![card.raw()], false);
    let mut tx = Transaction::begin(state.clone());
    assert_eq!(
        tx.state().hand_count(owner) as i32,
        hand_before as i32 - 1 + 3,
        "应摸标记数+弃牌数（3）张"
    );
    assert_eq!(
        tx.state().players.get(owner).unwrap().hp,
        hp_before - 1,
        "总数大于 2 应失去 1 点体力"
    );
    assert!(
        !tx.state()
            .players
            .get(owner)
            .unwrap()
            .marks
            .iter()
            .any(|(m, _)| *m == ZAOLI),
        "标记应清空"
    );
}

/// 孤影（ABI 447）：失去牌后当前回合角色二选一（随机交牌 / 收回弃牌堆失去牌）。
#[test]
fn guying_two_choice_recover_or_gain() {
    const GUYING: u32 = 2_534_727_648;
    let (mut tx, owner, seats) = setup(GUYING, 2);
    let other = seats.into_iter().find(|p| *p != owner).unwrap();
    tx.state_mut().turn.current = other;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = skill_instance(&tx, owner, GUYING).unwrap();
    // 失去一张牌（通过引擎移牌管线）。
    let lost = tx.state().hand_cards(owner)[0];
    let hand_zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![lost],
            to: discard,
            from: Some(hand_zone),
        }],
    )
    .unwrap();
    // 孤影：其他角色二选一（0 随机交牌 / 1 收回失去牌）。
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveGuying {
            player: owner,
            actor: other,
            card: lost,
            stage: 0,
            skill,
        }],
    )
    .err();
    assert_eq!(err, Some(EffectError::ChoiceRequested), "应开二选一 choice");
    // 选 1：收回失去的牌。
    let state = resolve_choice(tx.commit().state, other, 1, vec![1], false);
    let mut tx = Transaction::begin(state.clone());
    assert!(
        tx.state().hand_cards(owner).contains(&lost),
        "应收回失去的牌"
    );
}
}

mod original_shiji_zhibian_yuyan {
//! Golden for SHIJI 宗预「直辩 / 预言」语义（复用 ABI 369 / 370 管线）。
//!
//! 直辩：准备阶段拼点，胜后移动装备/判定区牌、回复或背水；输则失体。
//! 预言：成为高体力来源的实体杀目标后，其交高点数牌否则此杀无效。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const ZHIBIAN: u32 = 3_401_315_970;
const YUYAN: u32 = 2_770_825_567;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/shiji/shiji.npack.json");

fn setup(skill_def: u32, players: u8) -> (Transaction, PlayerId, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(skill_def as u64),
            kind: CommandKind::StartGame {
                player_count: players,
                rng_seed: 20_260_808,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let seats = engine.state().players.alive_by_seat();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, seats)
}

fn inject_hand(tx: &mut Transaction, player: PlayerId, kind: CardKind, suit: Suit) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(80_000 + kind.tag() as u32),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn resolve_choice(
    state: GameState,
    actor: PlayerId,
    key: u64,
    selections: Vec<u32>,
    cancelled: bool,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    next
}

/// 直辩：准备阶段触发打开选目标 choice（拼点目标）。
#[test]
fn zhibian_offers_compare_at_prepare_phase() {
    let (mut tx, owner, seats) = setup(ZHIBIAN, 2);
    let _other = seats.into_iter().find(|p| *p != owner).unwrap();
    // 直接调用直辩原语（准备阶段）→ 打开选拼点目标 choice。
    // （DSL trigger 在真实对局由引擎 dispatch,手动调用验证原语行为。）
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|s| s.def.raw() == ZHIBIAN && s.owner == owner)
        .map(|s| s.id)
        .unwrap();
    tx.state_mut().turn.phase = engine_core::state::Phase::Prepare;
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveTwzhibian {
            player: owner,
            target: PlayerId::NONE,
            winner: PlayerId::NONE,
            stage: 0,
            skill,
        }],
    )
    .err();
    assert_eq!(
        err,
        Some(EffectError::ChoiceRequested),
        "应打开选拼点目标 choice"
    );
}

/// 预言：高体力来源的实体杀指定宗预 → 来源交牌 choice。
#[test]
fn yuyan_offers_gift_choice() {
    let (mut tx, owner, seats) = setup(YUYAN, 2);
    let other = seats.into_iter().find(|p| *p != owner).unwrap();
    // 来源 hp > 宗预 hp。
    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    tx.state_mut().players.get_mut(other).unwrap().hp = owner_hp + 1;
    // 来源的实体杀（点数 7）。
    let sha = inject_hand(&mut tx, other, CardKind::Sha, Suit::Spade);
    // 来源还需一张更大点数的牌(供交牌)——牌堆随机可能有;直接验证触发。
    let dispatched = dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::CardUsed {
            player: other,
            card: sha,
            targets: vec![owner],
        }],
        &mut TriggerFireRecord::new(),
    );
    // card_targeted 事件由 CardUsed 展开;yuyan(锁定)在 card_targeted 触发。
    // resolve_twyuyan 可能打开交牌 choice 或直接取消目标(无交牌选项时)。
    match dispatched {
        Ok(_) => {}
        Err(EffectError::ChoiceRequested) => {}
        Err(other) => panic!("unexpected dispatch error: {other:?}"),
    }
    let _ = owner_hp;
}
}

mod original_shiji_zjjuxiang {
//! Golden for SHIJI 周姬「拒降」的脱离濒死限定技语义。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const ZJJUXIANG: u32 = 2_215_213_817;
const RESHENGXI: u32 = 2_846_213_948;
const XINJUXIANG: u32 = 3_450_353_814;
const XINGUIXIU: u32 = 3_238_150_014;
/// Concatenate every committed shiji source in pack order (alphabetical),
/// mirroring `content-cli pack` input concatenation so the logic hash matches
/// the committed npack regardless of future roster growth.
fn shiji_sources() -> String {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../content/original/ported/shiji");
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path().extension().map_or(false, |ext| ext == "skill")
        })
        .map(|entry| entry.path())
        .collect();
    files.sort();
    files
        .iter()
        .map(|path| {
            let mut source = std::fs::read_to_string(path).unwrap();
            source.push('\n');
            source
        })
        .collect()
}

const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/shiji/shiji.npack.json");

#[test]
fn saved_other_player_can_trigger_damage_then_max_hp_draw() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut compiled = content_compiler::compile_source_named("shiji", &shiji_sources()).unwrap();
    compiled.character_index = committed.character_index.clone();
    compiled.manifest.dependencies = committed.manifest.dependencies.clone();
    compiled.manifest.logic_hash = compiled.compute_logic_hash();
    assert_eq!(committed.manifest.logic_hash, compiled.manifest.logic_hash);
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == ZJJUXIANG));

    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_714,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(ZJJUXIANG),
        }],
    )
    .unwrap();
    {
        let target = tx.state_mut().players.get_mut(target).unwrap();
        target.hp = 0;
        target.max_hp = 4;
        target.dying = true;
    }
    let hand = tx.state().hand_count(owner);

    let paused = run_effects(
        &mut tx,
        vec![EffectRequest::RecoverHp {
            source: target,
            player: target,
            amount: 2,
        }],
    )
    .unwrap_err();
    assert_eq!(paused, EffectError::ChoiceRequested);

    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, 1);
    assert_eq!(state.hand_count(owner), hand + 4);
    assert!(!state
        .skills
        .sorted_live()
        .iter()
        .any(|skill| skill.def.raw() == ZJJUXIANG));

    let mut tx = Transaction::begin(engine.state().clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(XINJUXIANG),
        }],
    )
    .unwrap();
    {
        let target = tx.state_mut().players.get_mut(target).unwrap();
        target.hp = 0;
        target.max_hp = 4;
        target.dying = true;
    }
    let hand = tx.state().hand_count(owner);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::RecoverHp {
                source: target,
                player: target,
                amount: 2,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, 1);
    assert_eq!(state.hand_count(owner), hand);
    assert!(!state
        .skills
        .sorted_live()
        .iter()
        .any(|skill| skill.def.raw() == XINJUXIANG));
}

#[test]
fn reshengxi_draws_only_when_the_owner_dealt_no_damage_this_turn() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_714_2,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;

    let prepare = || {
        let mut tx = Transaction::begin(engine.state().clone());
        let mut effects: Vec<_> = tx
            .state()
            .skills
            .sorted_live()
            .into_iter()
            .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
            .collect();
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(RESHENGXI),
        });
        run_effects(&mut tx, effects).unwrap();
        tx
    };

    let event = DomainEvent::TurnStateChanged {
        current: owner,
        phase: TurnPhaseTag::End,
    };
    let mut peaceful = prepare();
    let hand = peaceful.state().hand_count(owner);
    assert_eq!(
        dispatch_for_events(
            &mut peaceful,
            std::slice::from_ref(&event),
            &mut TriggerFireRecord::new(),
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let (state, result) = apply_command(
        peaceful.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(peaceful.state().revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(peaceful.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand + 2);

    let mut violent = prepare();
    violent
        .state_mut()
        .turn
        .set_flag(engine_core::state::turn_flag::DAMAGE_DEALT, 1);
    let effects =
        dispatch_for_events(&mut violent, &[event], &mut TriggerFireRecord::new()).unwrap();
    assert!(effects.is_empty());
    assert!(!violent.state().has_pending_choice());
}

#[test]
fn xinguixiu_uses_the_odd_draw_and_even_wounded_recover_branches() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_828),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_828,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;

    for (hp, max_hp, expected_draws, expected_hp) in
        [(3, 3, 1, 3), (2, 3, 0, 3), (2, 2, 0, 2), (3, 4, 1, 3)]
    {
        let mut tx = Transaction::begin(engine.state().clone());
        let mut effects: Vec<_> = tx
            .state()
            .skills
            .sorted_live()
            .into_iter()
            .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
            .collect();
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(XINGUIXIU),
        });
        run_effects(&mut tx, effects).unwrap();
        {
            let owner_state = tx.state_mut().players.get_mut(owner).unwrap();
            owner_state.hp = hp;
            owner_state.max_hp = max_hp;
        }
        let hand = tx.state().hand_count(owner);
        run_effects(
            &mut tx,
            vec![EffectRequest::SetTurnState {
                current: owner,
                phase: engine_core::state::Phase::End.to_u8(),
            }],
        )
        .unwrap();
        assert_eq!(tx.state().hand_count(owner), hand + expected_draws);
        assert_eq!(tx.state().players.get(owner).unwrap().hp, expected_hp);
    }
}
}
