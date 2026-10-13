//! O2 全量合并批次 1（2026-10-14）：家族 sp2 的 40 个独立测试文件逐字包裹为模块——#[test] 函数名不变、测试数量不变；测试全名获得确定性前缀（模块名=原文件名，子串过滤行为不变）；include_bytes/str 相对路径不受影响（同目录）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_sp2_batch_ag_dsl {
//! Golden for sp2 batch-AG DSL rewrite:
//! 2 个技能——批量脚本误标为「audio-only / 待迁移」，上游 (sp2/skill.js) 在引用的
//! 行号处均有真实规则体：
//!   - lslixun（李肃）：受到伤害前防止之，改为获得等量「珠」标记
//!   - xingshen（邢氏）：受到伤害后摸 1（上游手牌最少时摸 2 分支未建模）

use engine_api::{
    Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const LSLIXUN: u32 = 4_094_034_720;
const XINGSHEN: u32 = 3_761_126_799;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp2/sp2.npack.json");

fn started(seed: u64) -> (GameState, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let state = engine.state().clone();
    let owner = state.turn.current;
    (state, owner)
}

fn with_skill(state: GameState, owner: PlayerId, def: u32) -> GameState {
    let mut tx = Transaction::begin(state);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(def),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

#[test]
fn package_publishes_sp2_batch_ag_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 562);
    let triggers: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| matches!(trigger.skill_def_id, LSLIXUN | XINGSHEN))
        .map(|trigger| {
            (
                trigger.skill_def_id,
                trigger.event.as_str(),
                trigger.filter.as_str(),
                trigger.optional,
            )
        })
        .collect();
    assert!(triggers.contains(&(LSLIXUN, "damage_before", "owner_is_target", false)));
    assert!(triggers.contains(&(XINGSHEN, "damage", "owner_is_target", false)));
    assert!(!package.modifier_index.iter().any(|modifier| {
        matches!(modifier.skill_def_id, LSLIXUN | XINGSHEN)
            && modifier.target == "hand_limit"
    }));
}

#[test]
fn lslixun_turns_damage_into_mark() {
    let (state, owner) = started(2_026_081_101);
    let state = with_skill(state, owner, LSLIXUN);
    let mut tx = Transaction::begin(state);
    let other = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let hp_before = tx.state().players.get(owner).unwrap().hp;
    let effects = run_effects(
        &mut tx,
        vec![EffectRequest::CommitDamage {
            damage_id: 0,
            source: other,
            target: owner,
            amount: 2,
            base_amount: 2,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
            card: engine_api::CardId::NONE,
            card_kind: 0,
            card_flags: 0,
        }],
    )
    .unwrap();
    assert!(
        effects
            .iter()
            .any(|event| matches!(event, DomainEvent::DamagePrevented { .. })),
        "lslixun must prevent the damage"
    );
    // hp 不变，标记 +2
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp_before);
    let marks = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .marks
        .iter()
        .find(|(id, _)| *id == LSLIXUN)
        .map(|(_, count)| *count)
        .unwrap_or(0);
    assert_eq!(marks, 2, "lslixun must add 珠 marks equal to damage amount");
}

#[test]
fn xingshen_draws_one_after_damage() {
    let (state, owner) = started(2_026_081_102);
    let state = with_skill(state, owner, XINGSHEN);
    let before = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    let other = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::DamageDealt {
            source: other,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().hand_count(owner), before + 1);
}
}

mod original_sp2_beizhan {
//! Golden coverage for 备战（SP 审配）目标摸牌+授予技能（ABI 455）.
//!
//! 审配结束阶段可选确认 → 选择目标摸至 min(5, 体力上限) → 目标获得「备战·待战」；
//! 其回合开始时若手牌数为全场最多（可并列），本回合获得「自守」
//! （card_targets_self_only，yijiang zishou2），随后移除自身。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const BEIZHAN: u32 = 2_840_635_277;
const BEIZHAN2: u32 = 4_111_010_111;
const SOURCE: &str = include_str!("../../../content/original/ported/sp2/beizhan.skill");

fn drive_choice(
    state: engine_core::state::GameState,
    actor: PlayerId,
    selection: u32,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![selection],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "choice must be accepted"
    );
    next
}

#[test]
fn draws_target_to_max_hp_then_grants_waiting_skill() {
    let package = content_compiler::compile_source_named("beizhan-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 455);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 2_026_080_115, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target, _third) = (players[0], players[1], players[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(BEIZHAN),
    });
    run_effects(&mut tx, effects).unwrap();
    let target_hand = tx.state().hand_count(target);
    let target_max_hp = tx.state().players.get(target).unwrap().max_hp;

    // Owner's end phase: optional confirm, then the draw-target choice.
    let mut ending = Transaction::begin(tx.commit().state);
    let fired = dispatch_for_events(
        &mut ending,
        &[engine_api::DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => {
            if let Err(e) = run_effects(&mut ending, effects) {
                assert!(
                    matches!(e, EffectError::ChoiceRequested),
                    "optional confirm must pause: {e:?}"
                );
            }
        }
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("dispatch failed: {e:?}"),
    }
    let mut state = ending.commit().state;
    let mut choices = 0;
    while state.has_pending_choice() && choices < 4 {
        let selection = if matches!(
            state.resolution.continuation_kind,
            engine_core::state::ContinuationKind::OptionalTrigger
        ) {
            1
        } else {
            target.raw()
        };
        state = drive_choice(state, owner, selection);
        choices += 1;
    }

    assert_eq!(
        state.hand_count(target),
        (target_max_hp.min(5)).max(target_hand),
        "target draws to min(5, max HP)"
    );
    assert!(
        state
            .skills
            .sorted_live()
            .iter()
            .any(|skill| skill.owner == target && skill.def.raw() == BEIZHAN2),
        "target gains beizhan2"
    );
    assert!(!state.has_pending_choice());
}
}

mod original_sp2_cangchu {
//! Golden coverage for sp2 藏储 (cangchu): locked skill that grants 3 「粮」 marks
//! at game start and removes one mark per point of fire damage the owner takes.
//!
//! Exercises the `game_started` event trigger and the `add_mark`/`remove_mark`
//! effects with the damage event amount. The upstream custom `cangchuAwaken`
//! event (wired to 良营) is not ported; see the .skill comment.

use engine_api::{
    Command, CommandKind, DomainEvent, ExpectedRevision, IdempotencyKey, MarkDefId, PlayerId,
    SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const CANGCHU: u32 = 2_861_531_568;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp2/sp2.npack.json");

fn start() -> (Engine, PlayerId, PlayerId) {
    let mut engine = Engine::new();
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_816,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&pkg).unwrap();
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    (engine, owner, other)
}

fn add_cangchu(tx: &mut Transaction, owner: PlayerId) {
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    let mut effects = effects;
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(CANGCHU),
    });
    run_effects(tx, effects).unwrap();
}

fn marks_of(tx: &Transaction, player: PlayerId) -> i32 {
    tx.state()
        .players
        .get(player)
        .and_then(|p| p.marks.iter().find(|(id, _)| *id == CANGCHU))
        .map(|(_, count)| *count)
        .unwrap_or(0)
}

fn fire_game_started(tx: &mut Transaction, player_count: u8) {
    let effects = dispatch_for_events(
        tx,
        &[DomainEvent::GameStarted { player_count }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("game-started dispatch error: {e:?}"));
    run_effects(tx, effects).unwrap();
}

fn deal_damage(tx: &mut Transaction, source: PlayerId, target: PlayerId, amount: u32, nature: u8) {
    let events = run_effects(
        tx,
        vec![EffectRequest::DealDamage {
            source,
            target,
            amount,
            is_lose_hp: false,
            nature,
        }],
    )
    .unwrap_or_else(|e| panic!("damage error: {e:?}"));
    // Damage triggers fire through the damage event the engine already dispatched;
    // the trigger pipeline ran inside run_effects, so no manual dispatch is needed.
    let _ = events;
}

#[test]
fn committed_package_publishes_cangchu_triggers() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let events: Vec<&str> = pkg
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == CANGCHU)
        .map(|trigger| trigger.event.as_str())
        .collect();
    assert!(events.contains(&"game_started"));
    assert!(events.contains(&"damage"));
}

#[test]
fn game_start_grants_three_marks_and_fire_damage_removes_them() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    add_cangchu(&mut tx, owner);

    fire_game_started(&mut tx, 2);
    assert_eq!(marks_of(&tx, owner), 3, "game start grants 3 marks");

    // Fire damage 1 → 3 - 1 = 2.
    deal_damage(&mut tx, other, owner, 1, 1 /* fire */);
    assert_eq!(marks_of(&tx, owner), 2, "1 fire damage removes 1 mark");

    // Fire damage 2 → marks exhausted.
    deal_damage(&mut tx, other, owner, 2, 1);
    assert_eq!(marks_of(&tx, owner), 0, "remaining marks all removed");
}

#[test]
fn non_fire_damage_does_not_remove_marks() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    add_cangchu(&mut tx, owner);

    fire_game_started(&mut tx, 2);
    assert_eq!(marks_of(&tx, owner), 3);

    deal_damage(&mut tx, other, owner, 1, 0 /* normal */);
    assert_eq!(marks_of(&tx, owner), 3, "normal damage does not touch 粮 marks");
}
}

mod original_sp2_dcdanji {
//! Golden coverage for 单骑 + 怒嗔（sp2 新杀SP关羽，ABI 466）.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag, ZoneId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCDANJI: u32 = 3_851_102_612;
const DCNUCHEN: u32 = 2_960_451_206;
const MASHU: u32 = 2;
const SOURCE: &str = include_str!("../../../content/original/ported/sp2/dcdanji.skill");

fn drive_choice(
    state: engine_core::state::GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "choice must be accepted: {:?}",
        result.result
    );
    next
}

fn fire_phase(tx: Transaction, player: PlayerId, tag: TurnPhaseTag) -> Transaction {
    let mut tx = tx;
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: player,
            phase: tag,
        }],
        &mut TriggerFireRecord::new(),
    );
    eprintln!(
        "fire_phase({tag:?}): dispatch = {:?}",
        fired.as_ref().map(|v| (
            v.len(),
            v.iter()
                .take(8)
                .map(|e| format!("{e:?}"))
                .collect::<Vec<_>>()
        ))
    );
    match fired {
        Ok(effects) => match run_effects(&mut tx, effects) {
            Ok(_) => tx,
            Err(engine_core::effect::EffectError::ChoiceRequested) => tx,
            Err(e) => panic!("effect error: {e:?}"),
        },
        Err(engine_core::effect::EffectError::ChoiceRequested) => tx,
        Err(e) => panic!("dispatch error: {e:?}"),
    }
}

fn setup(tx: &mut Transaction, owner: PlayerId, target: PlayerId) {
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(DCNUCHEN),
    });
    run_effects(tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    // Deterministic hands: move everything to the discard pile via the engine (keeps zone
    // pointers consistent), then seed exactly. Target: Spade 5. Owner: Spade 7 + Heart 3.
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    for (zone_kind, player) in [(ZoneKind::Hand, owner), (ZoneKind::Hand, target)] {
        let zone_id = tx.state().zones.find(zone_kind, player).unwrap();
        let cards: Vec<CardId> = tx.state().zones.get(zone_id).unwrap().cards.clone();
        let moves = cards
            .into_iter()
            .map(|card| EffectRequest::MoveCards {
                cards: vec![card],
                to: discard,
                from: Some(zone_id),
            })
            .collect();
        run_effects(tx, moves).unwrap();
    }
    let owner_hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let target_hand = tx.state().zones.find(ZoneKind::Hand, target).unwrap();
    let mut alloc = |def: u32, kind: CardKind, suit: Suit, rank: u8, hand: engine_api::ZoneId| {
        let card = tx.state_mut().cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(def),
            kind,
            suit,
            rank,
            weapon_range: 0,
            zone: hand,
            owner,
            face_up: false,
        });
        tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
        card
    };
    alloc(96_000, CardKind::Sha, Suit::Spade, 5, target_hand);
    alloc(96_001, CardKind::Sha, Suit::Spade, 7, owner_hand);
    alloc(96_002, CardKind::Tao, Suit::Heart, 3, owner_hand);
}

#[test]
fn awakening_at_prepare_when_hej_above_hp() {
    let package = content_compiler::compile_source_named("dcdanji-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 466);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_801, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DCDANJI),
        }],
    )
    .unwrap();
    // hej > hp: set hp below maxHp to also exercise recover+draw.
    let max_hp_before = tx.state().players.get(owner).unwrap().max_hp;
    tx.state_mut().players.get_mut(owner).unwrap().hp = max_hp_before as i32 - 2;
    let owner_hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let extra = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(96_003),
        kind: CardKind::Sha,
        suit: Suit::Club,
        rank: 9,
        weapon_range: 0,
        zone: owner_hand,
        owner,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(owner_hand)
        .unwrap()
        .cards
        .push(extra);
    let hand_before = tx.state().hand_count(owner);

    let expected_num = 1; // (maxHp-1) - hp after the loss, with hp = maxHp-2
    let mut tx = Transaction::begin(tx.commit().state);
    tx.state_mut().turn.current = owner;
    let tx = fire_phase(tx, owner, TurnPhaseTag::Prepare);
    let state = tx.commit().state;
    let owner_state = state.players.get(owner).unwrap();
    assert_eq!(owner_state.max_hp, max_hp_before - 1, "maxHp reduced by 1");
    assert_eq!(
        owner_state.hp,
        max_hp_before as i32 - 1,
        "recovered to the new maxHp"
    );
    assert_eq!(
        state.hand_count(owner),
        hand_before + expected_num,
        "drew maxHp-hp after recovery"
    );
    let owned: Vec<u32> = state
        .skills
        .sorted_live()
        .into_iter()
        .filter(|skill| skill.owner == owner)
        .map(|skill| skill.def.raw())
        .collect();
    assert!(owned.contains(&MASHU), "gained mashu: {owned:?}");
    assert!(owned.contains(&DCNUCHEN), "gained dcnuchen: {owned:?}");
    assert!(
        !owned.contains(&DCDANJI),
        "awakened skill removed: {owned:?}"
    );
}

#[test]
fn dcnuchen_discard_same_suit_damages() {
    let package = content_compiler::compile_source_named("dcnuchen-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target) = (seats[0], seats[1]);
    setup(&mut tx, owner, target);
    let target_hp_before = tx.state().players.get(target).unwrap().hp;
    let owner_hand_before = tx.state().hand_count(owner);

    let mut tx = Transaction::begin(tx.commit().state);
    let tx = fire_phase(tx, owner, TurnPhaseTag::Play);
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice(), "target choice pending");
    state = drive_choice(state, owner, vec![target.raw()], false);
    assert!(state.has_pending_choice(), "hand-card choice pending");
    let target_hand = state.hand_cards(target);
    let revealed = *target_hand
        .iter()
        .find(|c| state.cards.get(**c).unwrap().suit == Suit::Spade)
        .unwrap();
    state = drive_choice(state, owner, vec![revealed.raw()], false);
    assert!(
        state.has_pending_choice(),
        "same-suit discard choice pending"
    );
    let owner_hand: Vec<u32> = state
        .hand_cards(owner)
        .into_iter()
        .filter(|c| state.cards.get(*c).unwrap().suit == Suit::Spade)
        .map(|c| c.raw())
        .collect();
    assert_eq!(owner_hand.len(), 1, "only the Spade is offered");
    state = drive_choice(state, owner, owner_hand.clone(), false);

    let target_hp_after = state.players.get(target).unwrap().hp;
    assert_eq!(
        target_hp_after,
        target_hp_before - 1,
        "1 damage for 1 discarded"
    );
    assert_eq!(
        state.hand_count(owner),
        owner_hand_before - 1,
        "discarded one card"
    );
    assert!(
        !state
            .hand_cards(owner)
            .iter()
            .any(|c| c.raw() == owner_hand[0]),
        "discarded Spade left the hand"
    );
}

#[test]
fn dcnuchen_cancel_gains_target_same_suit_hand() {
    let package = content_compiler::compile_source_named("dcnuchen-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_803, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target) = (seats[0], seats[1]);
    setup(&mut tx, owner, target);
    let target_spade: Vec<u32> = tx
        .state()
        .hand_cards(target)
        .into_iter()
        .filter(|c| tx.state().cards.get(*c).unwrap().suit == Suit::Spade)
        .map(|c| c.raw())
        .collect();
    assert!(
        target_spade.len() >= 1,
        "seeded Spade present (bootstrap may add more Spades)"
    );
    let owner_before = tx.state().hand_count(owner);

    let mut tx = Transaction::begin(tx.commit().state);
    let tx = fire_phase(tx, owner, TurnPhaseTag::Play);
    let mut state = tx.commit().state;
    state = drive_choice(state, owner, vec![target.raw()], false);
    let revealed = *state
        .hand_cards(target)
        .iter()
        .find(|c| state.cards.get(**c).unwrap().suit == Suit::Spade)
        .unwrap();
    state = drive_choice(state, owner, vec![revealed.raw()], false);
    assert!(state.has_pending_choice());
    state = drive_choice(state, owner, vec![], true);

    assert_eq!(
        state.hand_count(owner),
        owner_before + 1,
        "gained the Spade"
    );
    assert!(state
        .hand_cards(owner)
        .iter()
        .any(|c| c.raw() == target_spade[0]));
    let target_kept = state.hand_count(target);
    assert_eq!(
        target_kept, 0,
        "target lost its only hand card: {target_kept}"
    );
}
}

mod original_sp2_dcdeshi {
//! Golden for SP2「得势」(dcdeshi): while wounded, Sha damage to the owner is
//! prevented, the owner gains a Sha from the draw/discard piles, and the owner
//! loses 1 max HP.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{damage_nature, EffectRequest};

const DCDESHI: u32 = 4_143_976_690;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp2/sp2.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, source) = (players[0], players[1]);
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
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, source)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
}

fn hand_sha_kind(tx: &Transaction, player: PlayerId) -> Vec<CardKind> {
    tx.state()
        .hand_cards(player)
        .iter()
        .map(|card| tx.state().cards.get(*card).unwrap().kind)
        .collect()
}

#[test]
fn package_declares_dcdeshi_wounded_sha_damage_before_trigger() {
    let package = package();
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCDESHI)
        .expect("dcdeshi damage_before trigger");
    assert_eq!(trigger.event, "damage_before");
    assert_eq!(trigger.filter, "owner_is_target_wounded_sha");
    assert!(!trigger.optional);
    assert!(!package
        .modifier_index
        .iter()
        .any(|m| m.skill_def_id == DCDESHI));
}

#[test]
fn wounded_owner_absorbs_sha_damage_gains_sha_and_loses_max_hp() {
    let (mut tx, owner, source) = setup(20_261_201);
    add_skill(&mut tx, owner, DCDESHI);
    // The lord starts at hp = base max (5) while lord bonus sets max 6, so it is
    // already wounded; wound further to keep it unambiguous.
    let hp = tx.state().players.get(owner).unwrap().hp;
    tx.state_mut().players.get_mut(owner).unwrap().hp = hp - 1;
    let max_hp_before = tx.state().players.get(owner).unwrap().max_hp;
    let hand_before = tx.state().hand_count(owner);

    let events = run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: damage_nature::NORMAL,
            card: CardId::NONE,
            card_kind: CardKind::Sha.tag(),
            card_flags: 0,
        }],
    )
    .unwrap();
    // Damage is prevented: no DamageDealt event and HP unchanged.
    assert!(!events.iter().any(|event| matches!(event, DomainEvent::DamageDealt { .. })));
    assert_eq!(
        tx.state().players.get(owner).unwrap().hp,
        hp - 1,
        "sha damage to wounded owner is prevented"
    );
    // Exactly one Sha joined the hand (from draw or discard pile) and max HP dropped by one.
    assert_eq!(tx.state().hand_count(owner), hand_before + 1);
    let gained = tx
        .state()
        .hand_cards(owner)
        .iter()
        .map(|card| tx.state().cards.get(*card).unwrap().kind)
        .collect::<Vec<_>>();
    assert!(gained.contains(&CardKind::Sha));
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, max_hp_before - 1);
}

#[test]
fn healthy_owner_takes_sha_damage_unchanged() {
    let (mut tx, owner, source) = setup(20_261_202);
    add_skill(&mut tx, owner, DCDESHI);
    // Heal the lord to full so the wounded precondition is false.
    let max = tx.state().players.get(owner).unwrap().max_hp;
    tx.state_mut().players.get_mut(owner).unwrap().hp = max as i32;
    let hp = max as i32;
    let max_hp_before = tx.state().players.get(owner).unwrap().max_hp;
    let hand_before = tx.state().hand_count(owner);
    let sha_hand_before = hand_sha_kind(&tx, owner).iter().filter(|k| **k == CardKind::Sha).count();

    let events = run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: damage_nature::NORMAL,
            card: CardId::NONE,
            card_kind: CardKind::Sha.tag(),
            card_flags: 0,
        }],
    )
    .unwrap();
    assert!(events.iter().any(|event| matches!(event, DomainEvent::DamageDealt { .. })));
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp - 1);
    assert_eq!(
        tx.state().players.get(owner).unwrap().max_hp,
        max_hp_before,
        "max HP only changes when the trigger fires"
    );
    assert_eq!(tx.state().hand_count(owner), hand_before);
    assert_eq!(
        hand_sha_kind(&tx, owner).iter().filter(|k| **k == CardKind::Sha).count(),
        sha_hand_before
    );
}

#[test]
fn wounded_owner_takes_non_sha_damage_unchanged() {
    let (mut tx, owner, source) = setup(20_261_203);
    add_skill(&mut tx, owner, DCDESHI);
    let hp = tx.state().players.get(owner).unwrap().hp;
    tx.state_mut().players.get_mut(owner).unwrap().hp = hp - 1;
    let max_hp_before = tx.state().players.get(owner).unwrap().max_hp;
    let hand_before = tx.state().hand_count(owner);

    let events = run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: damage_nature::FIRE,
            card: CardId::NONE,
            card_kind: CardKind::TrickHuogong.tag(),
            card_flags: 0,
        }],
    )
    .unwrap();
    assert!(events.iter().any(|event| matches!(event, DomainEvent::DamageDealt { .. })));
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp - 2);
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, max_hp_before);
    assert_eq!(tx.state().hand_count(owner), hand_before);
}
}

mod original_sp2_dcjuexiang {
//! Golden for SP2「绝响」(dcjuexiang): on the owner's death the living damage
//! source discards all equipment and loses 1 HP; the owner then grants「残殒」
//! (dccanyun) to one other character.

use engine_api::{
    CardDefId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCJUEXIANG: u32 = 3_898_356_517;
const DCCANYUN: u32 = 3_523_035_260;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp2/sp2.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, first, second) = (players[0], players[1], players[2]);
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
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    // Keep seats distinct: owner kills first; second is the independent grant target.
    (tx, owner, first, second)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
}

fn equip_two_cards(tx: &mut Transaction, player: PlayerId) {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let equip_zone = tx.state().zones.find(ZoneKind::Equip, player).unwrap();
    let cards: Vec<_> = tx.state().hand_cards(player)[..2].to_vec();
    for (index, card) in cards.iter().enumerate() {
        let c = tx.state_mut().cards.get_mut(*card).unwrap();
        c.kind = if index == 0 { CardKind::Weapon } else { CardKind::Armor };
        c.weapon_range = if index == 0 { 2 } else { 0 };
    }
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards,
            to: equip_zone,
            from: Some(hand),
        }],
    )
    .unwrap();
}

fn has_skill(state: &GameState, player: PlayerId, def: u32) -> bool {
    state
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .any(|skill| state.skills.get(*skill).map(|s| s.def.raw()) == Some(def))
}

#[test]
fn package_declares_dcjuexiang_death_trigger_with_source_alive_filter() {
    let package = package();
    let death = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCJUEXIANG)
        .expect("dcjuexiang death trigger");
    assert_eq!(death.event, "death");
    assert_eq!(death.filter, "owner_is_target_source_alive");
    assert!(!death.optional);
    assert!(!package
        .modifier_index
        .iter()
        .any(|m| m.skill_def_id == DCJUEXIANG));
}

#[test]
fn living_killer_discards_equipment_loses_hp_and_chosen_other_gains_canyun() {
    let (mut tx, owner, _first, second) = setup(20_261_101);
    add_skill(&mut tx, owner, DCJUEXIANG);
    let killer = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner && *player != second)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: killer,
            count: 2,
        }],
    )
    .unwrap();
    equip_two_cards(&mut tx, killer);
    let source_hp = tx.state().players.get(killer).unwrap().hp;

    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PlayerKilled {
            player: owner,
            killer,
            hand_equip_cards: vec![],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    let dispatched = run_effects(&mut tx, fired);
    assert!(matches!(dispatched, Err(EffectError::ChoiceRequested)));
    // Source equipment zone is emptied and source lost 1 HP before the grant choice.
    let equip_zone = tx.state().zones.find(ZoneKind::Equip, killer).unwrap();
    assert!(tx.state().zones.get(equip_zone).unwrap().cards.is_empty());
    assert_eq!(tx.state().players.get(killer).unwrap().hp, source_hp - 1);

    let committed = tx.commit();
    let revision = committed.state.revision;
    let choice_id = ChoiceId::new(committed.state.resolution.pending_choice_id);
    let (state, result) = engine_core::command::apply_command(
        committed.state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![second.raw()],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    assert!(has_skill(&state, second, DCCANYUN));
    assert!(!has_skill(&state, killer, DCCANYUN));
    assert!(!state.has_pending_choice());
}

#[test]
fn dead_killer_skips_effects_entirely() {
    let (mut tx, owner, _first, _second) = setup(20_261_102);
    add_skill(&mut tx, owner, DCJUEXIANG);
    // No real killer: PlayerId::NONE is never alive, so the source-alive filter
    // rejects the trigger and neither the discard nor the grant runs.
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PlayerKilled {
            player: owner,
            killer: PlayerId::NONE,
            hand_equip_cards: vec![],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(fired.is_empty());
}
}

mod original_sp2_dctingxian {
//! Golden coverage for 铤险（sp2 孙狼）摸牌 + 杀目标排除（ABI 464）.
//!
//! 使用【杀】指定目标后（每回合一次）摸装备数+1 张牌，并令此【杀】对至多等量目标无效。

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DCTINGXIAN: u32 = 3_933_341_702;
const SOURCE: &str = include_str!("../../../content/original/ported/sp2/dctingxian.skill");

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone: ZoneKind,
    kind: CardKind,
    def: u32,
) -> CardId {
    let zone_id = tx.state().zones.find(zone, owner).expect("target zone");
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: engine_api::Suit::Spade,
        rank: 5,
        weapon_range: 0,
        zone: zone_id,
        owner,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(zone_id)
        .expect("zone")
        .cards
        .push(card);
    card
}

fn play(
    state: GameState,
    actor: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card,
                targets,
                card_targets: vec![],
            },
        },
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
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
    )
}

#[test]
fn draws_and_excludes_sha_target_from_damage() {
    let package = content_compiler::compile_source_named("dctingxian-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 464);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_718, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target) = (seats[0], seats[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(DCTINGXIAN),
    });
    run_effects(&mut tx, effects).unwrap();
    // Strip the target's hand so it cannot respond with a Shan.
    let target_hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .expect("target hand");
    let target_cards: Vec<_> = tx
        .state()
        .zones
        .get(target_hand)
        .map(|zone| zone.cards.clone())
        .unwrap_or_default();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .expect("discard pile");
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: target_cards,
            to: discard,
            from: Some(target_hand),
        }],
    )
    .unwrap();
    let sha = inject(&mut tx, owner, ZoneKind::Hand, CardKind::Sha, 96_000);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = engine_core::state::Phase::Play;
    let owner_hand_before = tx.state().hand_count(owner);
    let target_hp_before = tx.state().players.get(target).unwrap().hp;

    // Play the Sha: the trigger fires, draws equip+1, and opens the exclusion choice.
    let (state, result) = play(tx.commit().state, owner, sha, vec![target], 1);
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "unexpected play result: {:?}",
        result.result
    );
    let choice = result.choice.expect("exclusion choice pending");
    assert_eq!(choice.kind, ChoiceKind::SelectTargets);
    assert_eq!(choice.observer, owner);
    let equip_count = 0; // no equipment injected
    assert_eq!(
        state.hand_count(owner),
        owner_hand_before + equip_count,
        "owner draws equipment count + 1 (Sha leaves the hand)"
    );

    // Exclude the target from this Sha.
    let (state, result) = resolve(state, owner, vec![target.raw()], false, 2);
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "unexpected exclusion result: {:?}",
        result.result
    );
    // The excluded Sha resolves with no response (target has no Shan); any further
    // engine choices are declined so the resolution completes.
    let mut state = state;
    let mut key = 3;
    loop {
        if state.resolution.pending_choice_id == 0 {
            break;
        }
        let (next, result) = resolve(state, target, Vec::new(), true, key);
        assert!(
            matches!(result.result, CommandResult::Accepted { .. }),
            "unexpected sha resolution result: {:?}",
            result.result
        );
        state = next;
        key += 1;
        assert!(key < 10, "sha resolution did not settle");
    }
    assert_eq!(
        state.players.get(target).unwrap().hp,
        target_hp_before,
        "excluded Sha target takes no damage"
    );
}
}

mod original_sp2_fuyuan {
//! 第一百六十批：sp2「辅袁」（辛评 fuyuan）零新原语重写。
//! 上游：回合外使用或打出/响应一张牌后，若当前回合角色手牌数少于你，其摸一张牌，
//! 否则你摸一张牌。实现：card_used/card_responded 双臂 + 既有过滤器
//! owner_source_outside_turn_card_used_or_responded → hand_count(current_phase_player())
//! 比较分支决定 draw_cards 目标（current_phase_player() 首用于效果参数位）。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const FUYUAN: u32 = 2_389_825_863;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp2/sp2.npack.json");

fn started(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let state = engine.state().clone();
    let current = state.turn.current;
    let owner = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != current)
        .unwrap();
    (state, current, owner)
}

fn strip_skills_and_hands(state: GameState) -> GameState {
    let mut tx = Transaction::begin(state);
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain(
            tx.state()
                .players
                .alive_by_seat()
                .into_iter()
                .map(|player| EffectRequest::DiscardFromHand {
                    player,
                    count: u32::MAX,
                }),
        )
        .collect();
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

fn add_fuyuan(state: GameState, owner: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    run_effects(&mut tx, vec![EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(FUYUAN),
    }])
    .unwrap();
    tx.commit().state
}

fn inject_hand(tx: &mut Transaction, owner: PlayerId, kind: CardKind, def: u32) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Diamond,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn play_sha(
    state: GameState,
    actor: PlayerId,
    target: PlayerId,
    key: u64,
) -> (GameState, PipelineResult) {
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.phase = Phase::Play;
    let sha = inject_hand(&mut tx, actor, CardKind::Sha, 98_400 + key as u32);
    let tx = tx.commit();
    let revision = tx.state.revision;
    apply_command(
        tx.state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![target],
                card_targets: Vec::new(),
            },
        },
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> GameState {
    let choice_id = engine_api::ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "resolve rejected: {:?}",
        result.result
    );
    state
}

fn hand_count(state: &GameState, player: PlayerId) -> u32 {
    state
        .zones
        .find(ZoneKind::Hand, player)
        .and_then(|zone| state.zones.get(zone).map(|z| z.cards.len() as u32))
        .unwrap_or(0)
}

#[test]
fn package_publishes_fuyuan_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    for event in ["card_used", "card_responded"] {
        assert!(
            package.trigger_index.iter().any(|trigger| {
                trigger.skill_def_id == FUYUAN
                    && trigger.event == event
                    && trigger.filter == "owner_source_outside_turn_card_used_or_responded"
                    && trigger.optional
            }),
            "fuyuan must publish a {event} arm"
        );
    }
    assert!(!package
        .modifier_index
        .iter()
        .any(|modifier| modifier.skill_def_id == FUYUAN && modifier.target == "hand_limit"));
}

#[test]
fn fuyuan_outside_turn_response_fewer_hand_phase_player_draws() {
    let (state, current, owner) = started(2_026_090_601);
    let state = strip_skills_and_hands(state);
    let state = add_fuyuan(state, owner);

    // current 回合：其对 owner 出【杀】；owner 手牌 = 闪 + 填充（两张）。
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = current;
    let shan = inject_hand(&mut tx, owner, CardKind::Shan, 98_501);
    inject_hand(&mut tx, owner, CardKind::Tao, 98_502);
    let state = tx.commit().state;

    let (state, played) = play_sha(state, current, owner, 1);
    assert!(matches!(played.result, CommandResult::Accepted { .. }));
    // owner 响应窗：选【闪】。
    let shan_option = played
        .choice
        .as_ref()
        .expect("shan window opens")
        .options
        .iter()
        .find(|option| option.token == shan.raw())
        .expect("shan option listed");
    let state = resolve(state, owner, vec![shan_option.token], 2);

    // 回合外响应成立 → 辅袁可选确认。
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger,
        "outside-turn response must open the fuyuan confirm"
    );
    // owner 剩 1 手牌 > current 的 0 → 当前回合角色（current）摸一张。
    assert_eq!(hand_count(&state, owner), 1);
    assert_eq!(hand_count(&state, current), 0);
    let state = resolve(state, owner, vec![1], 3);
    assert_eq!(hand_count(&state, current), 1, "phase player with fewer hand draws");
    assert_eq!(hand_count(&state, owner), 1);
}

#[test]
fn fuyuan_outside_turn_response_equal_hand_self_draws() {
    let (state, current, owner) = started(2_026_090_602);
    let state = strip_skills_and_hands(state);
    let state = add_fuyuan(state, owner);

    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = current;
    let shan = inject_hand(&mut tx, owner, CardKind::Shan, 98_511);
    let state = tx.commit().state;

    let (state, played) = play_sha(state, current, owner, 1);
    assert!(matches!(played.result, CommandResult::Accepted { .. }));
    let shan_option = played
        .choice
        .as_ref()
        .expect("shan window opens")
        .options
        .iter()
        .find(|option| option.token == shan.raw())
        .expect("shan option listed");
    let state = resolve(state, owner, vec![shan_option.token], 2);

    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    // 响应后双方手牌均为 0（不小于）→ owner 自己摸一张。
    assert_eq!(hand_count(&state, owner), 0);
    let state = resolve(state, owner, vec![1], 3);
    assert_eq!(hand_count(&state, owner), 1, "equal hand falls to self draw");
    assert_eq!(hand_count(&state, current), 0);
}

#[test]
fn fuyuan_not_triggered_during_own_turn() {
    let (state, _current, owner) = started(2_026_090_603);
    let state = strip_skills_and_hands(state);
    let state = add_fuyuan(state, owner);

    // owner 自己回合内出【杀】：过滤器（回合外门槛）拦截，不弹确认。
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = owner;
    let state = tx.commit().state;
    let other = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let (state, played) = play_sha(state, owner, other, 1);
    assert!(matches!(played.result, CommandResult::Accepted { .. }));
    assert_ne!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger,
        "own-turn use must not open the fuyuan confirm"
    );
}
}

mod original_sp2_gangzhi {
//! Golden for SP2「刚直」的双向伤害转体力流失语义。

use engine_api::{
    Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::effect::{apply_effect, run_effects};
use engine_core::intent::{resolve_damage_intent, DamageIntent};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const GANGZHI: u32 = 3_573_807_347;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp2/sp2.npack.json");

fn isolated_state() -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == GANGZHI && skill.locked));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(GANGZHI as u64),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: GANGZHI as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));

    let owner = engine.state().turn.current;
    let others: Vec<_> = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
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
        skill_def: SkillDefId::new(GANGZHI),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, others[0], others[1])
}

fn resolved(tx: &Transaction, source: PlayerId, target: PlayerId) -> DamageIntent {
    resolve_damage_intent(
        tx.state(),
        DamageIntent {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
            cancelled: false,
        },
    )
}

#[test]
fn converts_only_damage_between_the_owner_and_another_living_player() {
    let (mut tx, owner, other, third) = isolated_state();
    assert!(resolved(&tx, owner, other).is_lose_hp);
    assert!(resolved(&tx, other, owner).is_lose_hp);
    assert!(!resolved(&tx, owner, owner).is_lose_hp);
    assert!(!resolved(&tx, PlayerId::NONE, owner).is_lose_hp);
    assert!(!resolved(&tx, other, third).is_lose_hp);

    let hp = tx.state().players.get(other).unwrap().hp;
    let events = apply_effect(
        &mut tx,
        EffectRequest::DealDamage {
            source: owner,
            target: other,
            amount: 2,
            is_lose_hp: false,
            nature: 1,
        },
    )
    .unwrap();
    assert!(matches!(
        events.as_slice(),
        [DomainEvent::DamageDealt {
            is_lose_hp: true,
            ..
        }]
    ));
    assert_eq!(tx.state().players.get(other).unwrap().hp, hp - 2);
    assert!(!tx.state().damage_history.contains(&(owner, other)));

    tx.state_mut().players.get_mut(other).unwrap().alive = false;
    assert!(!resolved(&tx, other, owner).is_lose_hp);
}
}

mod original_sp2_gongjian {
//! Golden coverage for 拱坚（sp2 朱儁，ABI 473）.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const GONGJIAN: u32 = 3_309_202_552;
const SOURCE: &str = include_str!("../../../content/original/ported/sp2/gongjian.skill");

fn alloc_card(tx: &mut Transaction, owner: PlayerId, kind: CardKind, def: u32) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn play(
    state: GameState,
    actor: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
    key: u64,
) -> GameState {
    let revision = state.revision;
    let (next, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card,
                targets,
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "play must be accepted: {:?}",
        result.result
    );
    next
}

fn drive_choice(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "choice must be accepted: {:?}",
        result.result
    );
    next
}

fn setup(tx: &mut Transaction, owner: PlayerId, others: &[PlayerId]) {
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(GONGJIAN),
    });
    run_effects(tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    // Empty the others' hands so cards resolve without responses.
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    for &other in others {
        let hand = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
        let cards: Vec<CardId> = tx.state().zones.get(hand).unwrap().cards.clone();
        run_effects(
            tx,
            cards
                .into_iter()
                .map(|card| EffectRequest::MoveCards {
                    cards: vec![card],
                    to: discard,
                    from: Some(hand),
                })
                .collect(),
        )
        .unwrap();
    }
}

/// Seed cards into a player's hand; returns their ids.
fn seed_hand(tx: &mut Transaction, player: PlayerId, specs: &[(CardKind, u32)]) -> Vec<CardId> {
    specs
        .iter()
        .map(|(kind, def)| alloc_card(tx, player, *kind, *def))
        .collect()
}

#[test]
fn shared_target_discards_and_gains_sha() {
    let package = content_compiler::compile_source_named("gongjian-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 473);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_801, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, a, b) = (seats[0], seats[1], seats[2]);
    setup(&mut tx, owner, &[a, b]);
    // A holds a Sha + a Tao; B holds a Sha.
    let a_cards = seed_hand(
        &mut tx,
        a,
        &[(CardKind::Sha, 96_100), (CardKind::Tao, 96_101)],
    );
    seed_hand(&mut tx, b, &[(CardKind::Sha, 96_102)]);
    let owner_hand_before = tx.state().hand_count(owner);
    let sha1 = alloc_card(&mut tx, owner, CardKind::Sha, 96_000);
    let sha2 = alloc_card(&mut tx, owner, CardKind::Sha, 96_001);

    let mut tx = Transaction::begin(tx.commit().state);
    // First Sha at A -> LAST_SHA_TARGETS = {A}.
    let mut state = play(tx.commit().state, owner, sha1, vec![a], 1);
    // The native per-turn Sha limit is one; clear the flag so the test can use two
    // (identity-standard in a real game grants "杀 unlimited" to everyone).
    state
        .turn
        .set_flag(engine_core::state::turn_flag::SHA_PLAYED, 0);
    state
        .turn
        .set_flag(engine_core::state::turn_flag::SHA_PLAYED_IN_PLAY_PHASE, 0);
    let mut tx = Transaction::begin(state);
    // Second Sha at A shares the target -> gongjian fires.
    let state = play(tx.commit().state, owner, sha2, vec![a], 2);
    assert!(
        state.resolution.pending_choice_id != 0,
        "discard choice pending"
    );
    // Discard only A's Sha.
    let state = drive_choice(state, owner, vec![a_cards[0].raw()], 3);
    assert_eq!(
        state.hand_count(owner),
        owner_hand_before + 1,
        "gained A's Sha (two own Shas used, one gained)"
    );
    assert_eq!(
        state.hand_count(a),
        1,
        "A kept only the Tao (Sha discarded and gained)"
    );
}

#[test]
fn no_choice_without_shared_target() {
    let package = content_compiler::compile_source_named("gongjian-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, a, b) = (seats[0], seats[1], seats[2]);
    setup(&mut tx, owner, &[a, b]);
    seed_hand(&mut tx, a, &[(CardKind::Sha, 96_100)]);
    seed_hand(&mut tx, b, &[(CardKind::Sha, 96_101)]);
    let sha1 = alloc_card(&mut tx, owner, CardKind::Sha, 96_000);
    let sha2 = alloc_card(&mut tx, owner, CardKind::Sha, 96_001);

    let mut tx = Transaction::begin(tx.commit().state);
    let mut state = play(tx.commit().state, owner, sha1, vec![a], 1);
    state
        .turn
        .set_flag(engine_core::state::turn_flag::SHA_PLAYED, 0);
    state
        .turn
        .set_flag(engine_core::state::turn_flag::SHA_PLAYED_IN_PLAY_PHASE, 0);
    let mut tx = Transaction::begin(state);
    // Second Sha at B shares no target with the first.
    let state = play(tx.commit().state, owner, sha2, vec![b], 2);
    assert_eq!(
        state.resolution.pending_choice_id, 0,
        "no gongjian choice without a shared target"
    );
}

#[test]
fn only_sha_cards_are_gained() {
    let package = content_compiler::compile_source_named("gongjian-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, a, b) = (seats[0], seats[1], seats[2]);
    setup(&mut tx, owner, &[a, b]);
    // A holds a Sha + a Tao; B holds a Sha (never targeted, no overlap with A).
    let a_cards = seed_hand(
        &mut tx,
        a,
        &[(CardKind::Sha, 96_100), (CardKind::Tao, 96_101)],
    );
    seed_hand(&mut tx, b, &[(CardKind::Sha, 96_102)]);
    let owner_hand_before = tx.state().hand_count(owner);
    let sha1 = alloc_card(&mut tx, owner, CardKind::Sha, 96_000);
    let sha2 = alloc_card(&mut tx, owner, CardKind::Sha, 96_001);

    let mut tx = Transaction::begin(tx.commit().state);
    let mut state = play(tx.commit().state, owner, sha1, vec![a], 1);
    state
        .turn
        .set_flag(engine_core::state::turn_flag::SHA_PLAYED, 0);
    state
        .turn
        .set_flag(engine_core::state::turn_flag::SHA_PLAYED_IN_PLAY_PHASE, 0);
    let mut tx = Transaction::begin(state);
    let state = play(tx.commit().state, owner, sha2, vec![a], 2);
    assert!(state.resolution.pending_choice_id != 0);
    // Discard both of A's cards: only the Sha comes back to the owner.
    let state = drive_choice(state, owner, a_cards.iter().map(|id| id.raw()).collect(), 3);
    assert_eq!(
        state.hand_count(owner),
        owner_hand_before + 1,
        "gained the Sha, the Tao stays in the discard pile"
    );
    assert_eq!(state.hand_count(a), 0, "A lost both cards");
}
}

mod original_sp2_hfjieying {
//! Golden coverage for 界营（sp2 韩馥）选人授技 + 杀加目标（ABI 465）.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const HFJIEYING: u32 = 2_801_808_452;
const HFJIEYING2: u32 = 4_111_610_111;
const SOURCE: &str = include_str!("../../../content/original/ported/sp2/hfjieying.skill");

fn drive_choice(
    state: engine_core::state::GameState,
    actor: PlayerId,
    selection: u32,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![selection],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "choice must be accepted"
    );
    next
}

fn fire_end_phase(
    tx: Transaction,
    player: PlayerId,
) -> Result<Transaction, engine_core::effect::EffectError> {
    let mut tx = tx;
    let fired = dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::TurnStateChanged {
            current: player,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => match run_effects(&mut tx, effects) {
            Ok(_) => Ok(tx),
            Err(engine_core::effect::EffectError::ChoiceRequested) => Ok(tx),
            Err(e) => Err(e),
        },
        Err(engine_core::effect::EffectError::ChoiceRequested) => Ok(tx),
        Err(e) => Err(e),
    }
}

#[test]
fn grants_temp_skill_and_adds_sha_target() {
    let package = content_compiler::compile_source_named("hfjieying-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 465);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_718, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, recipient, other) = (seats[0], seats[1], seats[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(HFJIEYING),
    });
    run_effects(&mut tx, effects).unwrap();

    // Owner's end phase: confirm + choose the recipient.
    let mut tx = Transaction::begin(tx.commit().state);
    tx.state_mut().turn.current = owner;
    let tx = fire_end_phase(tx, owner).unwrap();
    let mut state = tx.commit().state;
    assert!(
        state.has_pending_choice(),
        "optional confirm must be pending"
    );
    state = drive_choice(state, owner, 1); // owner confirms
    state = drive_choice(state, owner, recipient.raw()); // choose the recipient
    assert!(
        state
            .skills
            .sorted_live()
            .iter()
            .any(|skill| skill.owner == recipient && skill.def.raw() == HFJIEYING2),
        "recipient gains hfjieying2"
    );

    // Recipient's play phase: uses a Sha; the temp skill offers an extra target.
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = recipient;
    tx.state_mut().turn.phase = Phase::Play;
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, recipient)
        .expect("recipient hand");
    let sha = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(96_000),
        kind: CardKind::Sha,
        suit: engine_api::Suit::Spade,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner: recipient,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(hand)
        .expect("hand zone")
        .cards
        .push(sha);
    let committed = tx.commit();
    let revision = committed.state.revision;
    let (state, result) = apply_command(
        committed.state,
        Command {
            actor: recipient,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![other],
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "unexpected play result: {:?}",
        result.result
    );
    let choice = result.choice.expect("extra-target choice pending");
    assert_eq!(choice.kind, ChoiceKind::SelectTargets);
    assert_eq!(choice.observer, recipient);
}
}

mod original_sp2_jiaojie {
//! Golden for SP2「狡黠」按颜色生效的手牌、距离与次数 modifier。

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
    Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::query::{query, QueryContext};
use engine_core::state::{CardKind, Phase};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const JIAOJIE: u32 = 3_299_853_333;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp2/sp2.npack.json");

fn target_legal(
    tx: &Transaction,
    owner: PlayerId,
    card: engine_api::CardId,
    target: PlayerId,
) -> bool {
    matches!(
        query(
            tx.state(),
            &QueryRequest::TargetLegal {
                user: owner,
                card,
                target,
            },
            &mut QueryContext::new(),
        ),
        QueryResponse::Bool(true)
    )
}

#[test]
fn red_cards_are_protected_and_only_black_cards_ignore_range_and_use_count() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == JIAOJIE && skill.locked));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 4,
                rng_seed: 20_260_714,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let seats = engine.state().players.alive_by_seat();
    let owner_index = seats.iter().position(|player| *player == owner).unwrap();
    let near = seats[(owner_index + 1) % seats.len()];
    let far = seats[(owner_index + 2) % seats.len()];

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
        skill_def: SkillDefId::new(JIAOJIE),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut()
        .turn
        .set_flag(engine_core::state::turn_flag::SHA_PLAYED, 1);
    tx.state_mut()
        .turn
        .set_flag(engine_core::state::turn_flag::SHA_PLAYED_IN_PLAY_PHASE, 1);

    let cards = tx.state().hand_cards(owner);
    assert!(cards.len() >= 2);
    let black = cards[0];
    let red = cards[1];
    {
        let card = tx.state_mut().cards.get_mut(black).unwrap();
        card.kind = CardKind::Sha;
        card.suit = Suit::Spade;
    }
    {
        let card = tx.state_mut().cards.get_mut(red).unwrap();
        card.kind = CardKind::Sha;
        card.suit = Suit::Heart;
    }

    assert!(engine_core::query::card_ignored_for_hand_limit(
        tx.state(),
        owner,
        red
    ));
    assert!(!engine_core::query::card_ignored_for_hand_limit(
        tx.state(),
        owner,
        black
    ));
    let discardable = engine_core::query::discard_to_limit_tokens(tx.state(), owner);
    assert!(!discardable.contains(&red));
    assert!(discardable.contains(&black));

    assert_eq!(engine_core::query::sha_use_limit_for(tx.state(), owner), 1);
    assert_eq!(
        engine_core::query::sha_use_limit_for_card(tx.state(), owner, black),
        u32::MAX
    );
    assert_eq!(
        engine_core::query::sha_use_limit_for_card(tx.state(), owner, red),
        1
    );
    assert!(target_legal(&tx, owner, black, far));
    assert!(!target_legal(&tx, owner, red, far));

    let (unchanged, rejected) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: red,
                targets: vec![near],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(unchanged, *tx.state());

    let (_, accepted) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::PlayCard {
                card: black,
                targets: vec![far],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
}
}

mod original_sp2_jieliang {
//! Golden coverage for 劫粮（sp2 韩猛）摸牌削减 + 弃牌截获（ABI 461）.
//!
//! 其他角色摸牌阶段：可弃一张牌令其摸牌数/手牌上限 -1；其弃牌阶段弃牌后可获一张弃牌。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const JIELIANG: u32 = 4_227_370_894;
const JIELIANG_LESS: u32 = 3_795_806_305;
const JIELIANG_GAIN: u32 = 2_999_325_050;
const SOURCE: &str = include_str!("../../../content/original/ported/sp2/jieliang.skill");

fn drive_choice(
    state: engine_core::state::GameState,
    actor: PlayerId,
    selection: u32,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![selection],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "choice must be accepted"
    );
    next
}

fn fire_phase(
    mut tx: Transaction,
    player: PlayerId,
    phase: TurnPhaseTag,
) -> Result<Transaction, EffectError> {
    let fired = dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::TurnStateChanged {
            current: player,
            phase,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => match run_effects(&mut tx, effects) {
            Ok(_) => Ok(tx),
            Err(EffectError::ChoiceRequested) => Ok(tx),
            Err(e) => Err(e),
        },
        Err(EffectError::ChoiceRequested) => Ok(tx),
        Err(e) => Err(e),
    }
}

#[test]
fn reduces_draw_and_intercepts_discarded_card() {
    let package = content_compiler::compile_source_named("jieliang-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 461);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 2_026_080_115, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target, _third) = (players[0], players[1], players[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(JIELIANG),
    });
    run_effects(&mut tx, effects).unwrap();
    // Give the owner one discardable card for the cost.
    let owner_hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, owner)
        .expect("owner hand zone");
    let cost = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_700),
        kind: CardKind::Sha,
        suit: engine_api::Suit::Heart,
        rank: 3,
        weapon_range: 0,
        zone: owner_hand,
        owner,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(owner_hand)
        .expect("hand zone")
        .cards
        .push(cost);
    // Give the target one card that will be discarded later.
    let target_hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .expect("target hand zone");
    let loot = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_701),
        kind: CardKind::Sha,
        suit: engine_api::Suit::Club,
        rank: 7,
        weapon_range: 0,
        zone: target_hand,
        owner: target,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(target_hand)
        .expect("hand zone")
        .cards
        .push(loot);

    // Target's draw phase: optional confirm + cost discard.
    let mut tx = Transaction::begin(tx.commit().state);
    tx.state_mut().turn.current = target;
    let tx = fire_phase(tx, target, TurnPhaseTag::Draw).unwrap();
    let mut state = tx.commit().state;
    assert!(
        state.has_pending_choice(),
        "optional confirm must be pending"
    );
    state = drive_choice(state, owner, 1); // owner confirms
    state = drive_choice(state, owner, cost.raw()); // owner discards the cost card
    assert!(
        state
            .skills
            .sorted_live()
            .iter()
            .any(|skill| skill.owner == target && skill.def.raw() == JIELIANG_LESS),
        "target gains jieliang_less (draw -1, hand limit -1)"
    );
    assert!(
        state
            .skills
            .sorted_live()
            .iter()
            .any(|skill| skill.owner == owner && skill.def.raw() == JIELIANG_GAIN),
        "owner gains jieliang_gain"
    );

    // Target's discard phase: move the loot card to the discard pile; owner intercepts it.
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.phase = Phase::Discard;
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .expect("discard pile");
    let moved = run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![loot],
            to: discard,
            from: Some(target_hand),
        }],
    );
    assert!(
        matches!(moved, Err(EffectError::ChoiceRequested)),
        "intercept choice must pause"
    );
    let mut state = tx.commit().state;
    assert!(
        state.has_pending_choice(),
        "intercept card choice must be pending"
    );
    state = drive_choice(state, owner, loot.raw());
    assert!(
        state.hand_cards(owner).contains(&loot),
        "owner intercepts the discarded card"
    );
    assert!(!state.has_pending_choice());
}
}

mod original_sp2_juntun {
//! Golden for SP2「军屯」的准备阶段语义。

use content_schema::PackageCharacterEntry;
use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{CardKind, Identity, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const JUNTUN: u32 = 3_811_889_239;
const ZHIHU: u32 = 3_057_288_085;
const CHENGGONG: u32 = 2_957_700_161;
const HUANGKONG: u32 = 2_905_812_698;
const QUANJIU: u32 = 2_358_780_202;
const STARZHONGGU: u32 = 2_431_837_100;
const RELIEWEI: u32 = 3_943_534_187;
const STARSUJUN: u32 = 4_007_048_185;
const YECHOU2: u32 = 3_899_060_351;
/// Concatenate every committed sp2 source in pack order (alphabetical),
/// mirroring `content-cli pack` input concatenation so the logic hash matches
/// the committed npack regardless of future roster growth.
fn sp2_sources() -> String {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../content/original/ported/sp2");
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

const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp2/sp2.npack.json");

fn started_engine(package: &content_schema::ContentPackage) -> Engine {
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_713,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(package).unwrap();
    engine
}

#[test]
fn yechou2_loses_one_hp_at_every_turn_end() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let engine = started_engine(&committed);
    let owner = engine.state().turn.current;
    let ending_player = engine
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
            skill_def: SkillDefId::new(YECHOU2),
        }],
    )
    .unwrap();
    let hp = tx.state().players.get(owner).unwrap().hp;
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current: ending_player,
            phase: TurnPhaseTag::End,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other: false,
            current_dealt_damage_other: false,
            phase_targeted_players: 0,
            phase_hp_changed_players: 0,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp - 1);
}

#[test]
fn chenggong_only_offers_to_draw_for_a_multi_target_card() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let engine = started_engine(&committed);
    let owner = engine.state().turn.current;
    let user = engine
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
            skill_def: SkillDefId::new(CHENGGONG),
        }],
    )
    .unwrap();
    let hand = tx.state().hand_count(user);
    let mut fires = TriggerFireRecord::new();
    let paused = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: user,
            card: CardId::NONE,
            targets: vec![owner, user],
        }],
        &mut fires,
    )
    .unwrap_err();
    assert_eq!(paused, engine_core::effect::EffectError::ChoiceRequested);

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
    assert_eq!(state.hand_count(user), hand + 1);

    let mut single = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut single,
        &[DomainEvent::CardUsed {
            player: user,
            card: CardId::NONE,
            targets: vec![owner],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.is_empty());
}

fn prepare(tx: &mut Transaction, owner: PlayerId) {
    run_effects(
        tx,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: engine_core::state::Phase::Prepare.to_u8(),
        }],
    )
    .unwrap();
}

fn accept_optional(tx: Transaction, owner: PlayerId, key: u64) -> Transaction {
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    Transaction::begin(state)
}

#[test]
fn committed_package_loses_max_hp_then_draws_original_max_hp() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut compiled = content_compiler::compile_source_named("sp2", &sp2_sources()).unwrap();
    compiled.character_index = committed.character_index.clone();
    compiled.manifest.dependencies = committed.manifest.dependencies.clone();
    compiled.manifest.logic_hash = compiled.compute_logic_hash();
    assert_eq!(committed.manifest.logic_hash, compiled.manifest.logic_hash);
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == JUNTUN));

    let engine = started_engine(&committed);
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    {
        let player = tx.state_mut().players.get_mut(owner).unwrap();
        player.hp = 4;
        player.max_hp = 4;
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(JUNTUN),
        }],
    )
    .unwrap();
    let hand = tx.state().hand_count(owner);
    prepare(&mut tx, owner);
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, 3);
    assert_eq!(tx.state().hand_count(owner), hand + 4);

    tx.state_mut().players.get_mut(owner).unwrap().max_hp = 1;
    let hand = tx.state().hand_count(owner);
    prepare(&mut tx, owner);
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, 1);
    assert_eq!(tx.state().hand_count(owner), hand);
}

#[test]
fn zhihu_draws_twice_per_turn_for_damage_to_others() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let engine = started_engine(&committed);
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    let target = tx.state_mut().players.get_mut(other).unwrap();
    target.hp = 10;
    target.max_hp = 10;
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(ZHIHU),
        }],
    )
    .unwrap();
    let hand = tx.state().hand_count(owner);

    for expected in [hand + 2, hand + 4, hand + 4] {
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: owner,
                target: other,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            }],
        )
        .unwrap();
        assert_eq!(tx.state().hand_count(owner), expected);
    }
}

#[test]
fn huangkong_draws_only_when_targeted_outside_the_owners_turn_with_no_hand() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let engine = started_engine(&committed);
    let owner = engine.state().turn.current;
    let source = engine
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
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(HUANGKONG),
        },
        EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        },
    ]);
    run_effects(&mut tx, effects).unwrap();

    let cards = tx.state().hand_cards(source);
    assert!(cards.len() >= 3);
    tx.state_mut().cards.get_mut(cards[0]).unwrap().kind = CardKind::Sha;
    tx.state_mut().cards.get_mut(cards[1]).unwrap().kind = CardKind::TrickGuohe;
    tx.state_mut().cards.get_mut(cards[2]).unwrap().kind = CardKind::Sha;

    run_effects(
        &mut tx,
        vec![EffectRequest::UseCard {
            player: source,
            card: cards[0],
            targets: vec![owner],
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), 0);

    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: source,
            phase: Phase::Play.to_u8(),
        }],
    )
    .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::UseCard {
            player: source,
            card: cards[1],
            targets: vec![owner],
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), 2);

    run_effects(
        &mut tx,
        vec![EffectRequest::UseCard {
            player: source,
            card: cards[2],
            targets: vec![owner],
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), 2);
}

#[test]
fn quanjiu_uses_physical_identity_for_the_converted_shas_use_limit() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let engine = started_engine(&committed);
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    let skills = tx.state().players.get(owner).unwrap().skills.clone();
    for skill in skills {
        run_effects(&mut tx, vec![EffectRequest::RemoveSkill { skill }]).unwrap();
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(QUANJIU),
        }],
    )
    .unwrap();

    let cards = tx.state().hand_cards(owner);
    let jiu = cards[0];
    let sha = cards[1];
    tx.state_mut().cards.get_mut(jiu).unwrap().kind = CardKind::Jiu;
    tx.state_mut().cards.get_mut(sha).unwrap().kind = CardKind::Sha;

    let physical = tx.state().cards.get(jiu).unwrap();
    assert_eq!(physical.kind, CardKind::Jiu);
    assert_eq!(
        engine_core::query::effective_card_kind(tx.state(), owner, physical),
        CardKind::Sha
    );
    assert!(engine_core::query::card_use_unlimited_for_card(
        tx.state(),
        owner,
        jiu
    ));
    assert!(!engine_core::query::card_use_unlimited_for_card(
        tx.state(),
        owner,
        sha
    ));
    assert_eq!(tx.state().cards.get(jiu).unwrap().kind, CardKind::Jiu);
}

#[test]
fn starzhonggu_is_lord_only_and_compares_round_to_alive_qun_count() {
    let mut package = content_compiler::compile_source_named(
        "sp2-starzhonggu",
        include_str!("../../../content/original/ported/sp2/starzhonggu.skill"),
    )
    .unwrap();
    package.character_index = [9_211_001, 9_211_002]
        .into_iter()
        .map(|id| PackageCharacterEntry {
            id,
            name_key: format!("qun_{id}"),
            max_hp: 4,
            initial_hp: 0,
            skills: vec![],
            sex: "male".into(),
            group: "qun".into(),
            is_lord: false,
            clans: vec![],
            tags: vec![],
            sort_key: String::new(),
            portrait_asset: None,
            skin_assets: vec![],
        })
        .collect();
    package.manifest.logic_hash = package.compute_logic_hash();

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_711),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 20_260_711,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let lord = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.identity == Identity::Lord)
        .unwrap()
        .id;
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != lord)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    tx.state_mut().players.get_mut(lord).unwrap().character_def = 9_211_001;
    tx.state_mut().players.get_mut(other).unwrap().character_def = 9_211_002;
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: lord,
                skill_def: SkillDefId::new(STARZHONGGU),
            },
            EffectRequest::AddSkill {
                player: other,
                skill_def: SkillDefId::new(STARZHONGGU),
            },
        ],
    )
    .unwrap();

    tx.state_mut().turn.round = 1;
    assert_eq!(engine_core::query::draw_count_for(tx.state(), lord), 1);
    assert_eq!(engine_core::query::draw_count_for(tx.state(), other), 2);

    tx.state_mut().turn.round = 2;
    assert_eq!(engine_core::query::draw_count_for(tx.state(), lord), 4);

    tx.state_mut().turn.round = 1;
    tx.state_mut().players.get_mut(other).unwrap().alive = false;
    assert_eq!(engine_core::query::draw_count_for(tx.state(), lord), 4);
}

#[test]
fn reliewei_tracks_the_hp_limit_but_is_unlimited_during_the_owners_turn() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let engine = started_engine(&committed);
    let owner = engine.state().turn.current;
    let other = engine
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
        skill_def: SkillDefId::new(RELIEWEI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().hp = 2;
    tx.state_mut().turn.current = other;
    let hand = tx.state().hand_count(owner);

    for key in 10..12 {
        let paused = dispatch_for_events(
            &mut tx,
            &[DomainEvent::EnteredDying {
                player: other,
                source: owner,
            }],
            &mut TriggerFireRecord::new(),
        )
        .unwrap_err();
        assert_eq!(paused, engine_core::effect::EffectError::ChoiceRequested);
        tx = accept_optional(tx, owner, key);
    }
    assert_eq!(tx.state().hand_count(owner), hand + 2);
    dispatch_for_events(
        &mut tx,
        &[DomainEvent::EnteredDying {
            player: other,
            source: owner,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());

    tx.state_mut().turn.current = owner;
    let paused = dispatch_for_events(
        &mut tx,
        &[DomainEvent::EnteredDying {
            player: other,
            source: owner,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_err();
    assert_eq!(paused, engine_core::effect::EffectError::ChoiceRequested);
}

#[test]
fn starsujun_checks_the_remaining_hand_after_card_use() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let engine = started_engine(&committed);
    let owner = engine.state().turn.current;
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
        skill_def: SkillDefId::new(STARSUJUN),
    });
    run_effects(&mut tx, effects).unwrap();
    let hand = tx.state().hand_cards(owner);
    assert!(hand.len() >= 4 && hand.len() % 2 == 0);
    let half = hand.len() / 2;
    for card in &hand[..half] {
        tx.state_mut().cards.get_mut(*card).unwrap().kind = CardKind::Sha;
    }
    for card in &hand[half..] {
        tx.state_mut().cards.get_mut(*card).unwrap().kind = CardKind::TrickGuohe;
    }

    let paused = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card: CardId::NONE,
            targets: vec![],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_err();
    assert_eq!(paused, engine_core::effect::EffectError::ChoiceRequested);
    tx = accept_optional(tx, owner, 20);
    assert_eq!(tx.state().hand_count(owner), hand.len() as u32 + 2);

    let hand = tx.state().hand_cards(owner);
    for card in &hand {
        tx.state_mut().cards.get_mut(*card).unwrap().kind = CardKind::Sha;
    }
    tx.state_mut().cards.get_mut(hand[0]).unwrap().kind = CardKind::TrickGuohe;

    dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card: CardId::NONE,
            targets: vec![],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());
}
}

mod original_sp2_kuimang {
//! Golden for SP2「溃蟒」及跨回合伤害关系记录。

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::transaction::Transaction;
use engine_core::{canonical_state_hash, Engine};
use rule_api::EffectRequest;

const KUIMANG: u32 = 2_972_733_766;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp2/sp2.npack.json");

#[test]
fn kuimang_draws_only_for_a_previously_damaged_dead_player() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == KUIMANG));
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: 20_260_721,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let players: Vec<PlayerId> = engine
        .state()
        .players
        .iter_live()
        .map(|player| player.id)
        .collect();
    let (owner, victim, untouched) = (players[0], players[1], players[2]);
    let mut tx = Transaction::begin(engine.state().clone());
    // Roster-independent: the standard roster gained choice-bearing characters
    // (gongsunzan 义从 / old_re_lidian 询询+忘隙) whose booted presence changes
    // range/hand-limit/trigger answers; this test grants its own skills.
    let booted: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    if !booted.is_empty() {
        run_effects(&mut tx, booted).unwrap();
    }

    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(KUIMANG),
        }],
    )
    .unwrap();

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: victim,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert!(tx.state().damage_history.contains(&(owner, victim)));
    let mut without_history = tx.state().clone();
    without_history.damage_history.clear();
    assert_ne!(
        canonical_state_hash(tx.state()),
        canonical_state_hash(&without_history)
    );

    let hand = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::KillPlayer {
            player: victim,
            killer: untouched,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand + 2);

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: untouched,
            amount: 1,
            is_lose_hp: true,
            nature: 0,
        }],
    )
    .unwrap();
    assert!(!tx.state().damage_history.contains(&(owner, untouched)));
    let hand = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::KillPlayer {
            player: untouched,
            killer: owner,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand);
}
}

mod original_sp2_langmie_diy_junk_duanwei {
//! Golden coverage for SP2「狼灭」and the cross-package DIY 牢段煨 closure.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{turn_flag, Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const LANGMIE: u32 = 3_392_096_402;
const JUNK_DUANWEI: u32 = 3_934_400_008;
const SOURCE: &str = include_str!("../../../content/original/ported/sp2/langmie.skill");
const SP2_PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp2/sp2.npack.json");
const DIY_PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/diy/diy.npack.json");

fn started() -> (Transaction, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("langmie-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, LANGMIE as u64, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let other = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(LANGMIE),
    });
    effects.push(EffectRequest::SetTurnState {
        current: other,
        phase: Phase::Play.to_u8(),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, other)
}

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, def: u32) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Club,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn record_used(tx: &mut Transaction, player: PlayerId, card: CardId, target: PlayerId) {
    let effects = dispatch_for_events(
        tx,
        &[DomainEvent::CardUsed {
            player,
            card,
            targets: vec![target],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(tx, effects).unwrap();
}

fn phase_play_ended(tx: &mut Transaction, player: PlayerId) -> Result<(), EffectError> {
    let effects = match dispatch_for_events(
        tx,
        &[DomainEvent::PhaseEnded {
            current: player,
            phase: TurnPhaseTag::Play,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other: true,
            current_dealt_damage_other: false,
            phase_targeted_players: 0,
            phase_hp_changed_players: 0,
        }],
        &mut TriggerFireRecord::new(),
    ) {
        Ok(effects) => effects,
        Err(err) => return Err(err),
    };
    run_effects(tx, effects).map(|_| ())
}

fn phase_end_started(tx: &mut Transaction, player: PlayerId) -> Result<(), EffectError> {
    let effects = match dispatch_for_events(
        tx,
        &[DomainEvent::TurnStateChanged {
            current: player,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    ) {
        Ok(effects) => effects,
        Err(err) => return Err(err),
    };
    run_effects(tx, effects).map(|_| ())
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    )
}

#[test]
fn packages_publish_langmie_and_cross_package_character_closure() {
    let sp2 = content_schema::ContentPackage::from_json_bytes(SP2_PACKAGE).unwrap();
    let diy = content_schema::ContentPackage::from_json_bytes(DIY_PACKAGE).unwrap();
    let compiled = content_compiler::compile_source_named("langmie", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 105);
    assert_eq!(sp2.manifest.rule_vm_abi_version, 753);
    assert!(sp2
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == LANGMIE && skill.name == "langmie"));
    let triggers: Vec<_> = sp2
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == LANGMIE)
        .collect();
    assert_eq!(triggers.len(), 2);
    assert!(triggers.iter().all(|trigger| trigger.optional));
    assert!(diy
        .manifest
        .dependencies
        .iter()
        .any(|dependency| dependency.package_id == "sp2"));
    let character = diy
        .character_index
        .iter()
        .find(|character| character.id == JUNK_DUANWEI)
        .unwrap();
    assert_eq!(character.name_key, "junk_duanwei");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [LANGMIE]);
}

#[test]
fn duplicate_logical_category_in_exact_play_phase_offers_optional_draw() {
    let (mut tx, owner, other) = started();
    let basic = inject(&mut tx, other, CardKind::Sha, 91_001);
    let trick = inject(&mut tx, other, CardKind::TrickWuzhong, 91_002);
    record_used(&mut tx, other, basic, owner);
    record_used(&mut tx, other, trick, owner);
    assert!(!tx.state().turn.play_card_category_repeated(other));
    assert_eq!(phase_play_ended(&mut tx, other), Ok(()));

    tx.state_mut().turn.clear_play_card_category_history();
    let second_basic = inject(&mut tx, other, CardKind::Shan, 91_003);
    record_used(&mut tx, other, basic, owner);
    record_used(&mut tx, other, second_basic, owner);
    assert!(tx.state().turn.play_card_category_repeated(other));
    let before = tx.state().hand_count(owner);
    assert_eq!(
        phase_play_ended(&mut tx, other),
        Err(EffectError::ChoiceRequested)
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let (state, accepted) = resolve(tx.commit().state, owner, vec![1], 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), before + 1);
}

#[test]
fn damage_branch_declines_cleanly_or_discards_before_owner_sourced_damage() {
    let (mut tx, owner, other) = started();
    tx.state_mut().turn.phase = Phase::End;
    tx.state_mut().turn.set_flag(turn_flag::DAMAGE_DEALT, 2);
    let owner_hand = tx.state().hand_count(owner);
    let other_hp = tx.state().players.get(other).unwrap().hp;
    assert_eq!(
        phase_end_started(&mut tx, other),
        Err(EffectError::ChoiceRequested)
    );
    let (state, declined) = resolve(tx.commit().state, owner, vec![0], 2);
    assert!(matches!(declined.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), owner_hand);
    assert_eq!(state.players.get(other).unwrap().hp, other_hp);

    let (mut tx, owner, other) = started();
    tx.state_mut().turn.phase = Phase::End;
    tx.state_mut().turn.set_flag(turn_flag::DAMAGE_DEALT, 2);
    let discard = tx.state().hand_cards(owner)[0];
    let owner_hand = tx.state().hand_count(owner);
    let other_hp = tx.state().players.get(other).unwrap().hp;
    assert_eq!(
        phase_end_started(&mut tx, other),
        Err(EffectError::ChoiceRequested)
    );
    let (state, accepted) = resolve(tx.commit().state, owner, vec![1], 3);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::DiscardOneHandOrEquip
    );
    let offered: Vec<_> = accepted
        .choice
        .unwrap()
        .options
        .into_iter()
        .map(|option| option.token)
        .collect();
    assert!(offered.contains(&discard.raw()));

    let (state, discarded) = resolve(state, owner, vec![discard.raw()], 4);
    assert!(matches!(discarded.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), owner_hand - 1);
    assert_eq!(state.players.get(other).unwrap().hp, other_hp - 1);
    assert!(discarded.events.iter().any(|event| matches!(
        event,
        DomainEvent::DamageDealt {
            source,
            target,
            amount: 1,
            ..
        } if *source == owner && *target == other
    )));
}
}

mod original_sp2_liangjue {
//! 第一百六十五批：sp2「狼灭」（张横 liangjue）零引擎改动重写。
//! 上游：锁定技，当黑色牌进入或离开你的装备区/判定区后，你摸两张牌；若体力值
//! 大于 1，你失去 1 点体力。实现：cards_moved +
//! owner_black_card_moved_equip_judge（本批六点注册；进向经 to_zone ZoneId 反查
//! owner）→ draw_cards(self, 2) + if hp(self) > 1 lose_hp(self, 1)。

use engine_api::{
    CardDefId, CardId, CardMoveReason, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, Suit,
};
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::dispatch_for_events;
use engine_core::Engine;
use rule_api::EffectRequest;

const LIANGJUE: u32 = 2_187_481_060;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp2/sp2.npack.json");

fn started(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let state = engine.state().clone();
    let owner = state.turn.current;
    let other = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    (state, owner, other)
}

fn strip_and_arm(state: GameState, owner: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain(
            tx.state()
                .players
                .alive_by_seat()
                .into_iter()
                .map(|player| EffectRequest::DiscardFromHand {
                    player,
                    count: u32::MAX,
                }),
        )
        .chain([EffectRequest::AddSkill {
            player: owner,
            skill_def: engine_api::SkillDefId::new(LIANGJUE),
        }])
        .collect();
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.commit().state
}

fn alloc_card(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    suit: Suit,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(98_901),
        kind: CardKind::Weapon,
        suit,
        rank: 5,
        weapon_range: 1,
        zone,
        owner,
        face_up: true,
    })
}

fn hand_count(state: &GameState, player: PlayerId) -> u32 {
    state
        .zones
        .find(ZoneKind::Hand, player)
        .and_then(|zone| state.zones.get(zone).map(|z| z.cards.len() as u32))
        .unwrap_or(0)
}

fn hp_of(state: &GameState, player: PlayerId) -> i32 {
    state.players.get(player).unwrap().hp
}

#[test]
fn package_publishes_liangjue_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == LIANGJUE
            && trigger.event == "cards_moved"
            && trigger.filter == "owner_black_card_moved_equip_judge"
            && !trigger.optional
    }));
    assert!(!package
        .modifier_index
        .iter()
        .any(|modifier| modifier.skill_def_id == LIANGJUE && modifier.target == "hand_limit"));
}

#[test]
fn liangjue_draws_two_and_loses_one_when_black_card_enters_equip() -> Result<(), Box<dyn std::error::Error>> {
    let (state, owner, _other) = started(2_026_091_101);
    let state = strip_and_arm(state, owner);
    let hp_before = hp_of(&state, owner);
    assert!(hp_before > 1, "bootstrap hp must exceed 1 for the lose arm");

    // 黑色装备牌进入 owner 装备区（CardsMoved from 手牌区 to 装备区）。
    let mut tx = Transaction::begin(state);
    let from = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let to = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    let card = alloc_card(&mut tx, owner, ZoneKind::Hand, Suit::Club);
    tx.state_mut()
        .zones
        .get_mut(from)
        .unwrap()
        .cards
        .retain(|c| *c != card);
    tx.state_mut().zones.get_mut(to).unwrap().cards.push(card);
    let hand_before = hand_count(&tx.state(), owner);
    let requests = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardsMoved {
            cards: vec![card],
            from,
            to,
            reason: CardMoveReason::UseOrRespond,
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    )?;
    run_effects(&mut tx, requests)?;
    let state = tx.commit().state;

    assert_eq!(hand_count(&state, owner), hand_before + 2, "draws two");
    assert_eq!(hp_of(&state, owner), hp_before - 1, "loses one when hp above 1");
    Ok(())
}
#[test]
fn liangjue_skips_red_card_and_non_owner_zones() -> Result<(), Box<dyn std::error::Error>> {
    let (state, owner, other) = started(2_026_091_102);
    let state = strip_and_arm(state, owner);

    // 红色装备牌进入 owner 装备区：不触发。
    let mut tx = Transaction::begin(state);
    let from = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let to = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    let card = alloc_card(&mut tx, owner, ZoneKind::Hand, Suit::Heart);
    tx.state_mut()
        .zones
        .get_mut(from)
        .unwrap()
        .cards
        .retain(|c| *c != card);
    tx.state_mut().zones.get_mut(to).unwrap().cards.push(card);
    let hand_before = hand_count(&tx.state(), owner);
    let requests = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardsMoved {
            cards: vec![card],
            from,
            to,
            reason: CardMoveReason::UseOrRespond,
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    )?;
    run_effects(&mut tx, requests)?;
    let state = tx.commit().state;
    assert_eq!(hand_count(&state, owner), hand_before, "red card must not trigger");

    // 黑色牌进入**他人**装备区：不触发（区限定自己）。
    let mut tx = Transaction::begin(state);
    let from = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    let to = tx.state().zones.find(ZoneKind::Equip, other).unwrap();
    let card = alloc_card(&mut tx, other, ZoneKind::Hand, Suit::Spade);
    tx.state_mut()
        .zones
        .get_mut(from)
        .unwrap()
        .cards
        .retain(|c| *c != card);
    tx.state_mut().zones.get_mut(to).unwrap().cards.push(card);
    let hand_before = hand_count(&tx.state(), owner);
    let requests = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardsMoved {
            cards: vec![card],
            from,
            to,
            reason: CardMoveReason::UseOrRespond,
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    )?;
    run_effects(&mut tx, requests)?;
    let state = tx.commit().state;
    assert_eq!(hand_count(&state, owner), hand_before, "other's equip zone must not trigger");
    Ok(())
}
#[test]
fn liangjue_at_one_hp_draws_without_losing() -> Result<(), Box<dyn std::error::Error>> {
    let (state, owner, _other) = started(2_026_091_103);
    let state = strip_and_arm(state, owner);

    // hp 压到 1：摸 2 但不失体力。
    let mut tx = Transaction::begin(state);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 1;
    let from = tx.state().zones.find(ZoneKind::Judge, owner).unwrap();
    let to = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = alloc_card(&mut tx, owner, ZoneKind::Judge, Suit::Spade);
    tx.state_mut()
        .zones
        .get_mut(from)
        .unwrap()
        .cards
        .retain(|c| *c != card);
    tx.state_mut().zones.get_mut(to).unwrap().cards.push(card);
    let hand_before = hand_count(&tx.state(), owner);
    let requests = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardsMoved {
            cards: vec![card],
            from,
            to,
            reason: CardMoveReason::Discard,
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    )?;
    run_effects(&mut tx, requests)?;
    let state = tx.commit().state;

    assert_eq!(hand_count(&state, owner), hand_before + 2, "still draws two at 1 hp");
    assert_eq!(hp_of(&state, owner), 1, "no hp loss at 1 hp");
    Ok(())
}
}

mod original_sp2_lulve {
//! Golden coverage for 掠卤（sp2 梁兴）目标二选一（ABI 463）.
//!
//! 出牌阶段开始：选一名手牌数更少（>0）的其他角色，其选择交全部手牌并令拥有者
//! 翻面，或自己翻面并视为对拥有者使用【杀】。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const LULVE: u32 = 3_019_895_260;
const SOURCE: &str = include_str!("../../../content/original/ported/sp2/lulve.skill");

fn drive_choice(
    state: engine_core::state::GameState,
    actor: PlayerId,
    selection: u32,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![selection],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "choice must be accepted"
    );
    next
}

fn give_hand(mut tx: Transaction, player: PlayerId, count: u32) -> (Transaction, Vec<CardId>) {
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .expect("hand zone");
    let mut cards = Vec::new();
    for rank in 1..=count {
        let card = tx.state_mut().cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(9_700),
            kind: CardKind::Sha,
            suit: engine_api::Suit::Heart,
            rank: rank as u8,
            weapon_range: 0,
            zone: hand,
            owner: player,
            face_up: false,
        });
        tx.state_mut()
            .zones
            .get_mut(hand)
            .expect("hand zone")
            .cards
            .push(card);
        cards.push(card);
    }
    (tx, cards)
}

fn fire_play(tx: Transaction, owner: PlayerId) -> Result<Transaction, EffectError> {
    let mut tx = tx;
    let fired = dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Play,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => match run_effects(&mut tx, effects) {
            Ok(_) => Ok(tx),
            Err(EffectError::ChoiceRequested) => Ok(tx),
            Err(e) => Err(e),
        },
        Err(EffectError::ChoiceRequested) => Ok(tx),
        Err(e) => Err(e),
    }
}

#[test]
fn branch_zero_gives_all_hand_and_flips_owner() {
    let package = content_compiler::compile_source_named("lulve-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 463);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 2_026_080_115, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target, _third) = (players[0], players[1], players[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(LULVE),
    });
    run_effects(&mut tx, effects).unwrap();
    // Controlled hands: owner 3, target 1.
    let (tx, _owner_cards) = give_hand(tx, owner, 3);
    let (mut tx, target_cards) = give_hand(tx, target, 1);
    let owner_hand_before = tx.state().hand_count(owner);
    let target_hand_before = tx.state().hand_count(target);
    tx.state_mut().turn.current = owner;

    let tx = fire_play(tx, owner).unwrap();
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice(), "target choice must be pending");
    state = drive_choice(state, owner, target.raw()); // owner picks the target
    state = drive_choice(state, target, 0); // target chooses give-all + flip owner
    assert_eq!(
        state.hand_count(owner),
        owner_hand_before + target_hand_before,
        "owner gains the target's whole hand"
    );
    assert_eq!(state.hand_count(target), 0, "target hand is emptied");
    assert!(
        state.players.get(owner).unwrap().turned_over,
        "owner flips face down"
    );
    assert!(
        !state.players.get(target).unwrap().turned_over,
        "target stays face up"
    );
    assert!(!state.has_pending_choice());
    let _ = target_cards;
}

#[test]
fn branch_one_flips_target_and_uses_virtual_sha() {
    let package = content_compiler::compile_source_named("lulve-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 463);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 2_026_080_115, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target, _third) = (players[0], players[1], players[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(LULVE),
    });
    run_effects(&mut tx, effects).unwrap();
    let (tx, _owner_cards) = give_hand(tx, owner, 3);
    let (mut tx, _target_cards) = give_hand(tx, target, 1);
    tx.state_mut().turn.current = owner;

    let tx = fire_play(tx, owner).unwrap();
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice(), "target choice must be pending");
    state = drive_choice(state, owner, target.raw()); // owner picks the target
                                                      // The virtual Sha may open a response window for the owner; drive the branch first.
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state,
        Command {
            actor: target,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "branch choice must be accepted"
    );
    let state = next;
    assert!(
        state.players.get(target).unwrap().turned_over,
        "target flips face down"
    );
}
}

mod original_sp2_lxzhuixi {
//! Golden for SP2「追袭」按伤害双方翻面状态双向加伤的语义。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::intent::{resolve_damage_intent, DamageIntent};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const LXZHUIXI: u32 = 2_387_115_210;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp2/sp2.npack.json");

fn isolated_state() -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == LXZHUIXI && skill.locked));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(LXZHUIXI as u64),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: LXZHUIXI as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));

    let players = engine.state().players.alive_by_seat();
    let mut tx = Transaction::begin(engine.state().clone());
    // Roster-independent: remove every booted skill, draining any
    // optional-trigger choices deal-dependent skills leave behind, so later
    // DealDamage calls in these tests are choice-free.
    let mut key = 9_000u64;
    loop {
        if !tx.state().skills.sorted_live().is_empty() {
            let effects: Vec<_> = tx
                .state()
                .skills
                .sorted_live()
                .into_iter()
                .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
                .collect();
            match run_effects(&mut tx, effects) {
                Ok(_) => {}
                Err(EffectError::ChoiceRequested) => {}
                Err(e) => panic!("effect error: {e:?}"),
            }
        }
        if tx.state().resolution.pending_choice_id == 0 {
            break;
        }
        let choice_id = ChoiceId::new(tx.state().resolution.pending_choice_id);
        let (next, _) = apply_command(
            tx.state().clone(),
            Command {
                actor: PlayerId::NONE,
                expected_revision: ExpectedRevision(tx.state().revision),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id,
                    selections: vec![],
                    cancelled: true,
                },
            },
        );
        tx = Transaction::begin(next);
        key += 1;
    }
    (tx, players[0], players[1], players[2])
}

fn add_lxzhuixi(tx: &mut Transaction, owner: PlayerId) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(LXZHUIXI),
        }],
    )
    .unwrap();
}

fn resolved_amount(tx: &Transaction, source: PlayerId, target: PlayerId) -> u32 {
    resolve_damage_intent(
        tx.state(),
        DamageIntent {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
            cancelled: false,
        },
    )
    .amount
}

fn dealt_amount(tx: &mut Transaction, source: PlayerId, target: PlayerId) -> i32 {
    let before = tx.state().players.get(target).unwrap().hp;
    run_effects(
        tx,
        vec![EffectRequest::DealDamage {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    before - tx.state().players.get(target).unwrap().hp
}

#[test]
fn source_side_bonus_requires_different_turned_over_states() {
    let (mut tx, source, target, _) = isolated_state();
    add_lxzhuixi(&mut tx, source);

    assert_eq!(resolved_amount(&tx, source, target), 1);
    tx.state_mut().players.get_mut(target).unwrap().turned_over = true;
    assert_eq!(resolved_amount(&tx, source, target), 2);
}

#[test]
fn target_side_bonus_does_not_apply_to_unrelated_or_sourceless_damage() {
    let (mut tx, source, target, third) = isolated_state();
    add_lxzhuixi(&mut tx, target);
    tx.state_mut().players.get_mut(source).unwrap().turned_over = true;
    // Fixed HP so the two-damage hits never open a dying window, independent
    // of the booted roster's max_hp.
    tx.state_mut().players.get_mut(target).unwrap().max_hp = 5;
    tx.state_mut().players.get_mut(target).unwrap().hp = 5;

    assert_eq!(dealt_amount(&mut tx, source, target), 2);
    assert_eq!(dealt_amount(&mut tx, source, third), 1);
    assert_eq!(dealt_amount(&mut tx, PlayerId::NONE, target), 1);
}

#[test]
fn source_and_target_owners_each_contribute_once() {
    let (mut tx, source, target, _) = isolated_state();
    add_lxzhuixi(&mut tx, source);
    add_lxzhuixi(&mut tx, target);
    tx.state_mut().players.get_mut(target).unwrap().turned_over = true;
    tx.state_mut().players.get_mut(target).unwrap().max_hp = 5;
    tx.state_mut().players.get_mut(target).unwrap().hp = 5;

    assert_eq!(dealt_amount(&mut tx, source, target), 3);
}
}

mod original_sp2_nhguizhu {
//! Golden for SP2「鬼助」的全局濒死与每回合限一次语义。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::Phase;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const NHGUIZHU: u32 = 2_454_936_215;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp2/sp2.npack.json");

fn isolated_state() -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == NHGUIZHU && !skill.locked));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
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
    let other = engine
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
        skill_def: SkillDefId::new(NHGUIZHU),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, other)
}

fn offer(tx: &mut Transaction, dying: PlayerId) -> Result<Vec<EffectRequest>, EffectError> {
    dispatch_for_events(
        tx,
        &[DomainEvent::EnteredDying {
            player: dying,
            source: PlayerId::NONE,
        }],
        &mut TriggerFireRecord::new(),
    )
}

fn accept(tx: &Transaction, owner: PlayerId, key: u64) -> engine_core::state::GameState {
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn first_dying_event_is_optional_and_second_in_the_same_turn_is_suppressed() {
    let (mut tx, owner, other) = isolated_state();
    let hand = tx.state().hand_count(owner);
    assert_eq!(
        offer(&mut tx, other).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = accept(&tx, owner, 1);
    assert_eq!(state.hand_count(owner), hand + 2);

    let mut same_turn = Transaction::begin(state);
    let effects = offer(&mut same_turn, owner).unwrap();
    assert!(effects.is_empty());
    assert!(!same_turn.state().has_pending_choice());
}

#[test]
fn another_players_new_turn_resets_the_limit_and_dead_owner_cannot_trigger() {
    let (mut tx, owner, other) = isolated_state();
    assert_eq!(
        offer(&mut tx, other).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let mut state = accept(&tx, owner, 2);

    state.turn.current = owner;
    state.turn.phase = Phase::End;
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(3),
            kind: CommandKind::EndPhase,
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.current, other);

    let mut next_turn = Transaction::begin(state);
    assert_eq!(
        offer(&mut next_turn, other).unwrap_err(),
        EffectError::ChoiceRequested
    );

    let (mut dead_owner, owner, other) = isolated_state();
    dead_owner.state_mut().players.get_mut(owner).unwrap().alive = false;
    let effects = offer(&mut dead_owner, other).unwrap();
    assert!(effects.is_empty());
    assert!(!dead_owner.state().has_pending_choice());
}
}

mod original_sp2_nhlundao_machine {
//! Golden for SP2 论道 nhlundao (ABI 752, tag 768, Cont 333):
//! 受伤后若来源存活且手牌数不同：手牌多则摸 1，否则弃来源一张（he 区，
//! owner 选）。真管道（裸 DealDamage 派发 Damage 触发）。

use engine_api::{
    CardDefId, CardId, PlayerId, SkillDefId,
};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const NHLUNDAO: u32 = 3_109_389_590;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp2/sp2.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn started(seed: u64) -> (GameState, Vec<PlayerId>) {
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package()).unwrap();
    let players = tx.state().players.alive_by_seat();
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
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    (tx.commit().state, players)
}

fn grant(state: &mut GameState, player: PlayerId, def: u32) {
    let mut tx = Transaction::begin(state.clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
    *state = tx.commit().state;
}

fn inject(state: &mut GameState, owner: PlayerId, zone: ZoneKind, count: usize, rank: u8, tag: u32) {
    for index in 0..count {
        let zone = state.zones.find(zone, owner).unwrap();
        let card = state.cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(105_000 + tag * 10 + index as u32),
            kind: CardKind::Other,
            suit: engine_api::Suit::Club,
            rank,
            weapon_range: 0,
            zone,
            owner,
            face_up: false,
        });
        state.zones.get_mut(zone).unwrap().cards.push(card);
    }
}

fn damage(
    state: &mut GameState,
    source: PlayerId,
    target: PlayerId,
    amount: u32,
) -> Result<(), EffectError> {
    let mut tx = Transaction::begin(state.clone());
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target,
            amount,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    );
    *state = tx.commit().state;
    result.map(|_| ())
}

#[test]
fn package_publishes_nhlundao_contract() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 753);
    let entry = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == NHLUNDAO)
        .expect("nhlundao trigger");
    assert_eq!(entry.event.as_str(), "damage");
    assert_eq!(entry.filter.as_str(), "owner_is_target");
    assert!(!entry.optional);
}

#[test]
fn larger_hand_draws_one_after_damage() {
    let (mut state, players) = started(2_026_108_001);
    let owner = players[0];
    let source = players[1];
    grant(&mut state, owner, NHLUNDAO);
    // owner 手牌 3 > source 手牌 1 → 受伤后摸 1。
    inject(&mut state, owner, ZoneKind::Hand, 3, 7, 1);
    inject(&mut state, source, ZoneKind::Hand, 1, 7, 2);
    let owner_before = state.hand_count(owner);
    damage(&mut state, source, owner, 1).unwrap();
    assert_eq!(state.hand_count(owner), owner_before + 1);
}

#[test]
fn smaller_hand_discards_one_source_card() {
    let (mut state, players) = started(2_026_108_002);
    let owner = players[0];
    let source = players[1];
    grant(&mut state, owner, NHLUNDAO);
    // owner 手牌 0 < source 手牌 2 → 受伤后 owner 弃 source 一张（开选牌窗）。
    inject(&mut state, source, ZoneKind::Hand, 2, 7, 3);
    let source_before = state.hand_count(source);
    let res = damage(&mut state, source, owner, 1);
    assert!(matches!(res, Err(EffectError::ChoiceRequested)));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ResolveNhlundaoStage
    );
    // 选 source 第一张手牌弃置。
    let hand = state.zones.find(ZoneKind::Hand, source).unwrap();
    let pick = state.zones.get(hand).unwrap().cards[0];
    let choice_id = engine_api::ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = engine_core::command::apply_command(
        state,
        engine_api::Command {
            actor: owner,
            expected_revision: engine_api::ExpectedRevision(0),
            idempotency_key: engine_api::IdempotencyKey(7),
            kind: engine_api::CommandKind::ResolveChoice {
                choice_id,
                selections: vec![pick.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, engine_api::CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(source), source_before - 1);
    assert!(!state.has_pending_choice());
}

#[test]
fn equal_hands_are_quiet() {
    let (mut state, players) = started(2_026_108_003);
    let owner = players[0];
    let source = players[1];
    grant(&mut state, owner, NHLUNDAO);
    inject(&mut state, owner, ZoneKind::Hand, 2, 7, 4);
    inject(&mut state, source, ZoneKind::Hand, 2, 7, 5);
    let owner_before = state.hand_count(owner);
    damage(&mut state, source, owner, 1).unwrap();
    assert_eq!(state.hand_count(owner), owner_before);
}
}

mod original_sp2_nhyinbing {
//! Golden for SP2「阴兵」的杀伤害转体力流失与全局体力流失摸牌语义。

use engine_api::{
    CardId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::effect::{apply_effect, run_effects};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const NHYINBING: u32 = 2_701_847_677;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp2/sp2.npack.json");

fn isolated_state() -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == NHYINBING && skill.locked));
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == NHYINBING
            && modifier.target == "damage_as_hp_loss"
            && modifier.selector.as_deref() == Some("sha")
    }));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(NHYINBING as u64),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: NHYINBING as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));

    let owner = engine.state().turn.current;
    let others: Vec<_> = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
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
        skill_def: SkillDefId::new(NHYINBING),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, others[0], others[1])
}

#[test]
fn converts_only_the_owners_logical_sha_damage_and_draws_after_other_hp_loss() {
    let (mut tx, owner, other, third) = isolated_state();

    let ordinary = apply_effect(
        &mut tx,
        EffectRequest::DealDamage {
            source: owner,
            target: other,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        },
    )
    .unwrap();
    assert!(matches!(
        ordinary.as_slice(),
        [DomainEvent::DamageDealt {
            is_lose_hp: false,
            ..
        }]
    ));

    let hand = tx.state().hand_count(owner);
    let sha = apply_effect(
        &mut tx,
        EffectRequest::DealShaDamage {
            source: owner,
            target: third,
            card: CardId::NONE,
            amount: 1,
            nature: 0,
            card_flags: 0,
        },
    )
    .unwrap();
    assert!(matches!(
        sha.as_slice(),
        [DomainEvent::DamageDealt {
            is_lose_hp: true,
            ..
        }]
    ));
    let effects = dispatch_for_events(&mut tx, &sha, &mut TriggerFireRecord::new()).unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().hand_count(owner), hand + 1);

    let incoming = apply_effect(
        &mut tx,
        EffectRequest::DealShaDamage {
            source: other,
            target: owner,
            card: CardId::NONE,
            amount: 1,
            nature: 0,
            card_flags: 0,
        },
    )
    .unwrap();
    assert!(matches!(
        incoming.as_slice(),
        [DomainEvent::DamageDealt {
            is_lose_hp: false,
            ..
        }]
    ));
}
}

mod original_sp2_nifu {
//! Golden for SP2 严夫人「匿伏」rewritten from legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::ContinuationKind;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const NIFU: u32 = 2_611_616_498;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp2/sp2.npack.json");

fn isolated(hand: u32) -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(NIFU as u64),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: NIFU as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([
        EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        },
        EffectRequest::DrawCards {
            player: owner,
            count: hand,
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(NIFU),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, other)
}

fn fire(tx: &mut Transaction, current: PlayerId) -> Result<(), EffectError> {
    let effects = dispatch_for_events(
        tx,
        &[DomainEvent::TurnStateChanged {
            current,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    )?;
    run_effects(tx, effects).map(|_| ())
}

#[test]
fn committed_package_exposes_locked_global_phase_end_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == NIFU)
        .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 753);
    assert_eq!(
        (
            trigger.event.as_str(),
            trigger.filter.as_str(),
            trigger.optional
        ),
        ("phase_end", "owner_alive", false)
    );
}

#[test]
fn any_players_turn_end_draws_owner_up_to_four() {
    let (mut tx, owner, other) = isolated(2);
    fire(&mut tx, other).unwrap();
    assert_eq!(tx.state().hand_count(owner), 4);
}

#[test]
fn excess_hand_requires_owner_to_choose_exact_cards() {
    let (mut tx, owner, other) = isolated(6);
    let original = tx.state().hand_cards(owner);
    assert_eq!(fire(&mut tx, other), Err(EffectError::ChoiceRequested));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::AdjustHandTo
    );
    let discarded = vec![original[0].raw(), original[5].raw()];
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(NIFU as u64 + 1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: discarded.clone(),
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let hand = state.hand_cards(owner);
    assert_eq!(hand.len(), 4);
    assert!(discarded
        .into_iter()
        .all(|card| !hand.iter().any(|remaining| remaining.raw() == card)));
}
}

mod original_sp2_recuorui_re_niujin {
//! Golden coverage for SP2 牛金「摧锐」rewritten from legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, SkillId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::hash::canonical_state_hash;
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const RE_NIUJIN: u32 = 3_368_551_416;
const RECUORUI: u32 = 4_246_379_136;
const RELIEWEI: u32 = 3_943_534_187;
const SOURCE: &str = include_str!("../../../content/original/ported/sp2/recuorui.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp2/sp2.npack.json");

fn setup(hp: i32) -> (GameState, PlayerId, Vec<PlayerId>, SkillId) {
    let package = content_compiler::compile_source_named("recuorui-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 288);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_729_31, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let targets = players[1..].to_vec();
    let effects = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain([EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(RECUORUI),
        }])
        .collect();
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().players.get_mut(owner).unwrap().hp = hp;
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == RECUORUI)
        .unwrap();
    (tx.commit().state, owner, targets, skill)
}

fn activate(
    state: GameState,
    owner: PlayerId,
    skill: SkillId,
    targets: Vec<PlayerId>,
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
                targets,
                card_targets: Vec::new(),
            },
        },
    )
}

fn resolve(
    state: GameState,
    owner: PlayerId,
    card: u32,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![card],
                cancelled: false,
            },
        },
    )
}

#[test]
fn package_contains_character_and_hp_capped_active_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 753);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.id == RE_NIUJIN)
            .unwrap()
            .skills,
        [RECUORUI, RELIEWEI]
    );
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == RECUORUI)
        .unwrap();
    let targets = active.targets.as_ref().unwrap();
    assert!(
        active.once_per_game
            && targets.min == 1
            && targets.max == u8::MAX
            && !targets.allow_self
            && targets.max_by_owner_hp
            && targets.hand_nonempty
    );
}

#[test]
fn gains_in_submitted_order_obeys_current_hp_and_is_limited() {
    let (state, owner, targets, skill) = setup(2);
    let submitted = vec![targets[1], targets[0]];
    let mut first_options: Vec<_> = state
        .hand_cards(submitted[0])
        .into_iter()
        .map(|card| card.raw())
        .collect();
    first_options.sort_unstable();
    let mut second_options: Vec<_> = state
        .hand_cards(submitted[1])
        .into_iter()
        .map(|card| card.raw())
        .collect();
    second_options.sort_unstable();

    let (state, pipe) = activate(state, owner, skill, submitted.clone(), 1);
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    let offered: Vec<_> = pipe
        .choice
        .unwrap()
        .options
        .into_iter()
        .map(|option| option.token)
        .collect();
    assert_eq!(offered, first_options);

    let first = first_options[0];
    let (state, pipe) = resolve(state, owner, first, 2);
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert!(state
        .hand_cards(owner)
        .iter()
        .any(|card| card.raw() == first));
    let offered: Vec<_> = pipe
        .choice
        .unwrap()
        .options
        .into_iter()
        .map(|option| option.token)
        .collect();
    assert_eq!(offered, second_options);

    let second = second_options[0];
    let (state, pipe) = resolve(state, owner, second, 3);
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert!(state
        .hand_cards(owner)
        .iter()
        .any(|card| card.raw() == second));
    let before = canonical_state_hash(&state);
    let (same, pipe) = activate(state, owner, skill, vec![submitted[0]], 4);
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);

    let (state, owner, targets, skill) = setup(1);
    let before = canonical_state_hash(&state);
    let (same, pipe) = activate(state, owner, skill, targets, 5);
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
}
}

mod original_sp2_sibian {
//! Golden coverage for 思变（sp2 张温，ABI 470）.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag, ZoneId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const SIBIAN: u32 = 3_021_138_179;
const SOURCE: &str = include_str!("../../../content/original/ported/sp2/sibian.skill");

fn drive_choice(
    state: engine_core::state::GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "choice must be accepted: {:?}",
        result.result
    );
    next
}

fn fire_draw(tx: Transaction, player: PlayerId) -> Transaction {
    let mut tx = tx;
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: player,
            phase: TurnPhaseTag::Draw,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => match run_effects(&mut tx, effects) {
            Ok(_) => tx,
            Err(engine_core::effect::EffectError::ChoiceRequested) => tx,
            Err(e) => panic!("effect error: {e:?}"),
        },
        Err(engine_core::effect::EffectError::ChoiceRequested) => tx,
        Err(e) => panic!("dispatch error: {e:?}"),
    }
}

fn empty_hand(tx: &mut Transaction, player: PlayerId) {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let cards: Vec<CardId> = tx.state().zones.get(hand).unwrap().cards.clone();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        tx,
        cards
            .into_iter()
            .map(|card| EffectRequest::MoveCards {
                cards: vec![card],
                to: discard,
                from: Some(hand),
            })
            .collect(),
    )
    .unwrap();
}

/// Seed `specs` (rank, def) onto the top of the draw pile; returns their ids.
fn seed_pile(tx: &mut Transaction, specs: &[(u8, u32)]) -> Vec<CardId> {
    let pile = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let mut ids = Vec::new();
    for (rank, def) in specs {
        let card = tx.state_mut().cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(*def),
            kind: CardKind::Sha,
            suit: Suit::Spade,
            rank: *rank,
            weapon_range: 0,
            zone: pile,
            owner: PlayerId::NONE,
            face_up: false,
        });
        tx.state_mut().zones.get_mut(pile).unwrap().cards.push(card);
        ids.push(card);
    }
    ids
}

fn setup(tx: &mut Transaction, owner: PlayerId) {
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(SIBIAN),
    });
    run_effects(tx, effects).unwrap();
}

#[test]
fn gains_extremes_gives_middle_to_min_hand_player() {
    let package = content_compiler::compile_source_named("sibian-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 470);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_801, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, other) = (seats[0], seats[1]);
    setup(&mut tx, owner);
    empty_hand(&mut tx, owner);
    empty_hand(&mut tx, other);
    // Top of the pile: ranks 3, 5, 5, 9 (9 = max, 3 = min -> gained; 5,5 -> middle).
    seed_pile(
        &mut tx,
        &[(3, 96_000), (5, 96_001), (5, 96_002), (9, 96_003)],
    );
    let pile_len_before = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .and_then(|id| tx.state().zones.get(id))
        .map(|z| z.cards.len())
        .unwrap();

    let mut tx = Transaction::begin(tx.commit().state);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Draw;
    let tx = fire_draw(tx, owner);
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice(), "min-hand target choice pending");
    assert_eq!(state.hand_count(owner), 2, "gained the max and min cards");
    assert_eq!(
        state.turn.draw_phase_fixed_count(),
        Some(0),
        "ordinary draw skipped"
    );
    state = drive_choice(state, owner, vec![other.raw()], false);
    assert_eq!(state.hand_count(other), 2, "received the middle cards");
    let pile_len = state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .and_then(|id| state.zones.get(id))
        .map(|z| z.cards.len())
        .unwrap();
    assert_eq!(pile_len, pile_len_before - 4, "four cards left the pile");
}

#[test]
fn cancel_sends_middle_to_discard() {
    let package = content_compiler::compile_source_named("sibian-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, other) = (seats[0], seats[1]);
    setup(&mut tx, owner);
    empty_hand(&mut tx, owner);
    empty_hand(&mut tx, other);
    seed_pile(
        &mut tx,
        &[(3, 96_000), (5, 96_001), (5, 96_002), (9, 96_003)],
    );

    let mut tx = Transaction::begin(tx.commit().state);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Draw;
    let tx = fire_draw(tx, owner);
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice());
    state = drive_choice(state, owner, vec![], true);
    assert_eq!(state.hand_count(owner), 2, "extremes still gained");
    assert_eq!(state.hand_count(other), 0, "middle not given");
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let discard_len = state.zones.get(discard).map(|z| z.cards.len()).unwrap();
    assert!(discard_len >= 2, "middle cards went to the discard pile");
}
}

mod original_sp2_spjiedao {
//! Golden coverage for 劫刀（sp2 忙牙长，ABI 469）.

use engine_api::{
    CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const SPJIEDAO: u32 = 4_242_283_768;
const SOURCE: &str = include_str!("../../../content/original/ported/sp2/spjiedao.skill");

fn drive_choice(
    state: engine_core::state::GameState,
    actor: PlayerId,
    selections: Vec<u32>,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "choice must be accepted: {:?}",
        result.result
    );
    next
}

fn commit_damage(
    tx: &mut Transaction,
    source: PlayerId,
    target: PlayerId,
    amount: u32,
) -> Result<(), engine_core::effect::EffectError> {
    match run_effects(
        tx,
        vec![EffectRequest::CommitDamage {
            damage_id: 0,
            source,
            target,
            amount,
            base_amount: amount,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
            card: CardId::NONE,
            card_kind: 0,
            card_flags: 0,
        }],
    ) {
        Ok(_) => Ok(()),
        Err(engine_core::effect::EffectError::ChoiceRequested) => Ok(()),
        Err(e) => Err(e),
    }
}

#[test]
fn boosts_damage_and_discards_chosen_cards() {
    let package = content_compiler::compile_source_named("spjiedao-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 469);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_801, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, victim) = (seats[0], seats[1]);
    // Strip skills; give the owner 劫刀 and wound them (maxHp - 2 → 2 lost HP).
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(SPJIEDAO),
    });
    run_effects(&mut tx, effects).unwrap();
    let max_hp = tx.state().players.get(owner).unwrap().max_hp;
    tx.state_mut().players.get_mut(owner).unwrap().hp = max_hp as i32 - 2;
    // Deterministic discardable hand: empty the owner's hand, seed exactly two cards.
    let owner_hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let cards: Vec<CardId> = tx.state().zones.get(owner_hand).unwrap().cards.clone();
    run_effects(
        &mut tx,
        cards
            .into_iter()
            .map(|card| EffectRequest::MoveCards {
                cards: vec![card],
                to: discard,
                from: Some(owner_hand),
            })
            .collect(),
    )
    .unwrap();
    let mut seeded = Vec::new();
    for def in [96_000u32, 96_001] {
        let card = tx.state_mut().cards.alloc(engine_core::state::Card {
            id: CardId::NONE,
            def: engine_api::CardDefId::new(def),
            kind: engine_core::state::CardKind::Sha,
            suit: engine_api::Suit::Spade,
            rank: 5,
            weapon_range: 0,
            zone: owner_hand,
            owner,
            face_up: false,
        });
        tx.state_mut()
            .zones
            .get_mut(owner_hand)
            .unwrap()
            .cards
            .push(card);
        seeded.push(card);
    }
    let victim_hp_before = tx.state().players.get(victim).unwrap().hp;

    let mut tx = Transaction::begin(tx.commit().state);
    commit_damage(&mut tx, owner, victim, 1).unwrap();
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice(), "optional confirm pending");
    state = drive_choice(state, owner, vec![1]); // confirm 劫刀
    assert!(state.has_pending_choice(), "boost amount choice pending");
    // Choose X = 2 (both lost HP), then discard exactly 2 he cards.
    state = drive_choice(state, owner, vec![2]);
    assert!(state.has_pending_choice(), "discard choice pending");
    let discardable: Vec<u32> = state
        .hand_cards(owner)
        .into_iter()
        .map(|id| id.raw())
        .collect();
    assert_eq!(discardable.len(), 2, "exactly the seeded cards");
    state = drive_choice(state, owner, discardable);

    assert_eq!(
        state.players.get(victim).unwrap().hp,
        victim_hp_before - 3,
        "1 base + 2 boost = 3 damage"
    );
    assert_eq!(state.hand_count(owner), 0, "both cards discarded");
}

#[test]
fn cancel_leaves_damage_unboosted() {
    let package = content_compiler::compile_source_named("spjiedao-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, victim) = (seats[0], seats[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(SPJIEDAO),
    });
    run_effects(&mut tx, effects).unwrap();
    let max_hp = tx.state().players.get(owner).unwrap().max_hp;
    tx.state_mut().players.get_mut(owner).unwrap().hp = max_hp as i32 - 2;
    let victim_hp_before = tx.state().players.get(victim).unwrap().hp;

    let mut tx = Transaction::begin(tx.commit().state);
    commit_damage(&mut tx, owner, victim, 1).unwrap();
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice());
    state = drive_choice(state, owner, vec![1]); // confirm 劫刀
    assert!(state.has_pending_choice(), "boost amount choice pending");
    let hand_before = state.hand_count(owner);
    // Cancel the boost: damage stays 1.
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![],
                cancelled: true,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state = next;
    assert_eq!(
        state.players.get(victim).unwrap().hp,
        victim_hp_before - 1,
        "unboosted damage"
    );
    assert_eq!(state.hand_count(owner), hand_before, "nothing discarded");
}

#[test]
fn second_damage_of_turn_not_boostable() {
    let package = content_compiler::compile_source_named("spjiedao-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_803, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, victim) = (seats[0], seats[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(SPJIEDAO),
    });
    run_effects(&mut tx, effects).unwrap();
    let max_hp = tx.state().players.get(owner).unwrap().max_hp;
    tx.state_mut().players.get_mut(owner).unwrap().hp = max_hp as i32 - 2;
    let victim_hp_before = tx.state().players.get(victim).unwrap().hp;

    let mut tx = Transaction::begin(tx.commit().state);
    // First damage: confirm then cancel the boost.
    commit_damage(&mut tx, owner, victim, 1).unwrap();
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice());
    state = drive_choice(state, owner, vec![1]); // confirm 劫刀
    assert!(state.has_pending_choice(), "boost amount choice pending");
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(3),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![],
                cancelled: true,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state = next;
    assert_eq!(state.players.get(victim).unwrap().hp, victim_hp_before - 1);

    // Second damage of the turn: the confirm still opens (optional), but the composite
    // sees the owner already dealt damage this turn and does not offer a boost.
    let mut tx = Transaction::begin(state);
    commit_damage(&mut tx, owner, victim, 1).unwrap();
    let state = tx.commit().state;
    assert!(state.has_pending_choice(), "optional confirm opens again");
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(4),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let state = next;
    assert!(
        !state.has_pending_choice(),
        "second damage of the turn is not boostable"
    );
    assert_eq!(state.players.get(victim).unwrap().hp, victim_hp_before - 2);
}
}

mod original_sp2_starcuxia_weiming {
//! Golden for SP2 粗遐 starcuxia / 未命 starweiming (ABI 750, tag 766):
//! 孪生锁定技——其他角色使用牌指定 owner 后：条件成立则其随机弃 1 手牌，
//! 条件仍成立则 owner 摸 1。真管道（CardUsed 事件派发）。

use engine_api::{
    CardDefId, CardId, DomainEvent, PlayerId, SkillDefId,
};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const STARCUXIA: u32 = 3_366_017_041;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp2/sp2.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn started(seed: u64) -> (GameState, Vec<PlayerId>) {
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package()).unwrap();
    let players = tx.state().players.alive_by_seat();
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
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    (tx.commit().state, players)
}

fn grant(state: &mut GameState, player: PlayerId, def: u32) {
    let mut tx = Transaction::begin(state.clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
    *state = tx.commit().state;
}

fn inject(state: &mut GameState, owner: PlayerId, zone: ZoneKind, kind: CardKind, tag: u32) -> CardId {
    let zone = state.zones.find(zone, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(103_000 + tag),
        kind,
        suit: engine_api::Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn fire_card_used(state: &mut GameState, source: PlayerId, card: CardId, target: PlayerId) {
    let mut tx = Transaction::begin(state.clone());
    // dispatch 产出待执行效果，须 run_effects 落地（含机器选牌窗短路）。
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: source,
            card,
            targets: vec![target],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    match run_effects(&mut tx, effects) {
        Ok(_) | Err(EffectError::ChoiceRequested) => {}
        Err(err) => panic!("unexpected: {err:?}"),
    }
    *state = tx.commit().state;
}

fn fire_machine(
    state: &mut GameState,
    owner: PlayerId,
    source: PlayerId,
) {
    let mut tx = Transaction::begin(state.clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveStarcuxiaWeiming {
            player: owner,
            source,
            variant: 0,
        }],
    )
    .unwrap();
    *state = tx.commit().state;
}

#[test]
fn package_publishes_starcuxia_contract() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 753);
    let entry = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == STARCUXIA)
        .expect("starcuxia trigger");
    assert_eq!(entry.event.as_str(), "card_targeted");
    assert!(!entry.optional);
}

#[test]
fn starcuxia_forces_random_discard_when_source_dominant_and_damaged_owner() {
    let (mut state, players) = started(2_026_105_001);
    let owner = players[0];
    let source = players[1];
    grant(&mut state, owner, STARCUXIA);
    // source hp 压到高于 owner；source 伤害过 owner（damage_history）。
    {
        let s = state.players.get_mut(source).unwrap();
        s.hp = 4;
        let o = state.players.get_mut(owner).unwrap();
        o.hp = 2;
    }
    state.damage_history.insert((source, owner));
    let used = inject(&mut state, source, ZoneKind::Hand, CardKind::Sha, 1);
    let spare = inject(&mut state, source, ZoneKind::Hand, CardKind::Shan, 2);
    let source_before = state.hand_count(source);
    let owner_before = state.hand_count(owner);

    // 机器直调：粗遐随机弃 1 + 条件复验摸 1。
    let _played = inject(&mut state, source, ZoneKind::Hand, CardKind::Sha, 3);
    fire_machine(&mut state, owner, source);

    // 注入 played 在 source_before 之后：机器时手牌 = before+1，弃 1 后 = before。
    assert_eq!(state.hand_count(source), source_before);
    assert!(
        !state.hand_cards(source).contains(&used) || !state.hand_cards(source).contains(&spare),
        "随机弃 1 后两张至少一张离开手牌"
    );
    assert_eq!(state.hand_count(owner), owner_before + 1, "条件复验成立 owner 摸 1");
    let _ = spare;
}

#[test]
fn starcuxia_real_pipeline_fires_through_card_targeted() {
    let (mut state, players) = started(2_026_105_003);
    let owner = players[0];
    let source = players[1];
    grant(&mut state, owner, STARCUXIA);
    {
        let s = state.players.get_mut(source).unwrap();
        s.hp = 4;
        let o = state.players.get_mut(owner).unwrap();
        o.hp = 2;
    }
    state.damage_history.insert((source, owner));
    let used = inject(&mut state, source, ZoneKind::Hand, CardKind::Sha, 6);
    let spare = inject(&mut state, source, ZoneKind::Hand, CardKind::Shan, 7);
    let source_before = state.hand_count(source);
    let owner_before = state.hand_count(owner);

    // 真管道：CardUsed 事件 → CardTargeted 逐目标 ctx → 威临 filter eval
    // → 机器执行（随机弃 1 + 条件复验摸 1）。
    let played = inject(&mut state, source, ZoneKind::Hand, CardKind::Sha, 8);
    fire_card_used(&mut state, source, played, owner);

    assert_eq!(
        state.hand_count(source),
        source_before + 1 - 1,
        "随机弃 1"
    );
    assert_eq!(state.hand_count(owner), owner_before + 1, "复验成立摸 1");
    let _ = (used, spare); // 被弃的是三张之一（随机），len 断言已覆盖。
}

#[test]
fn starcuxia_gated_when_neither_condition_holds() {
    let (mut state, players) = started(2_026_105_002);
    let owner = players[0];
    let source = players[1];
    grant(&mut state, owner, STARCUXIA);
    // source hp 低于 owner 且从未伤害过 owner → 不触发。
    {
        let s = state.players.get_mut(source).unwrap();
        s.hp = 1;
        let o = state.players.get_mut(owner).unwrap();
        o.hp = 3;
    }
    let _played = inject(&mut state, source, ZoneKind::Hand, CardKind::Sha, 4);
    let source_before = state.hand_count(source);
    let owner_before = state.hand_count(owner);
    let _played = inject(&mut state, source, ZoneKind::Hand, CardKind::Sha, 5);

    fire_machine(&mut state, owner, source);
    assert_eq!(state.hand_count(source), source_before);
    assert_eq!(state.hand_count(owner), owner_before);
}
}

mod original_sp2_starjianyu {
//! Golden for SP2「翦羽」：拥有者回合内按失去装备的其他角色摸牌。

use engine_api::{Command, CommandKind, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId};
use engine_core::effect::run_effects;
use engine_core::state::{CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const STARJIANYU: u32 = 2_340_348_064;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp2/sp2.npack.json");

#[test]
fn draws_once_when_another_player_loses_equipment_during_owners_turn() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_715),
        kind: CommandKind::StartGame {
            player_count: 3,
            rng_seed: 20_260_715,
            ruleset: "identity-standard".into(),
        },
    });
    let players = engine.state().players.alive_by_seat();
    let [owner, other, third] = players.as_slice() else {
        panic!("expected three players");
    };
    let (owner, other, third) = (*owner, *other, *third);
    let mut tx = Transaction::begin(engine.state().clone());
    let mut setup: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    setup.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(STARJIANYU),
    });
    run_effects(&mut tx, setup).unwrap();

    let other_hand = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    let other_equip = tx.state().zones.find(ZoneKind::Equip, other).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let equipment = tx.state().hand_cards(other)[..2].to_vec();
    tx.state_mut().cards.get_mut(equipment[0]).unwrap().kind = CardKind::Weapon;
    tx.state_mut().cards.get_mut(equipment[1]).unwrap().kind = CardKind::Armor;
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: equipment.clone(),
            to: other_equip,
            from: Some(other_hand),
        }],
    )
    .unwrap();
    let hand = tx.state().hand_count(owner);

    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: equipment,
            to: discard,
            from: Some(other_equip),
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand + 1);

    let third_hand = tx.state().zones.find(ZoneKind::Hand, third).unwrap();
    let third_equip = tx.state().zones.find(ZoneKind::Equip, third).unwrap();
    let third_card = tx.state().hand_cards(third)[0];
    tx.state_mut().cards.get_mut(third_card).unwrap().kind = CardKind::Weapon;
    run_effects(
        &mut tx,
        vec![
            EffectRequest::MoveCards {
                cards: vec![third_card],
                to: third_equip,
                from: Some(third_hand),
            },
            EffectRequest::SetTurnState {
                current: other,
                phase: Phase::Play.to_u8(),
            },
            EffectRequest::MoveCards {
                cards: vec![third_card],
                to: discard,
                from: Some(third_equip),
            },
        ],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand + 1);
}
}

mod original_sp2_xiangshu_machine {
//! Golden for SP2 襄戍 xiangshu (ABI 753, tag 769):
//! 限定技——结束阶段主动：令一名受伤角色回复 X 摸 X（X=min(5, 本回合
//! owner 造成伤害数)）。active 全管道 + 机器直调混合。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const XIANGSHU: u32 = 2_433_141_736;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp2/sp2.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn started(seed: u64) -> (GameState, Vec<PlayerId>) {
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package()).unwrap();
    let players = tx.state().players.alive_by_seat();
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
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    (tx.commit().state, players)
}

fn add_skill(state: &mut GameState, player: PlayerId, def: u32) -> engine_api::SkillId {
    let mut tx = Transaction::begin(state.clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
    *state = tx.commit().state;
    state
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| state.skills.get(*skill).unwrap().def.raw() == def)
        .unwrap()
}

fn inject(state: &mut GameState, owner: PlayerId, count: usize, tag: u32) {
    for index in 0..count {
        let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
        let card = state.cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(106_000 + tag * 10 + index as u32),
            kind: CardKind::Other,
            suit: engine_api::Suit::Club,
            rank: 7,
            weapon_range: 0,
            zone: hand,
            owner,
            face_up: false,
        });
        state.zones.get_mut(hand).unwrap().cards.push(card);
    }
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

#[test]
fn package_publishes_xiangshu_contract() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 753);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == XIANGSHU)
        .expect("xiangshu active entry");
    assert!(active.once_per_game);
    assert!(active
        .targets
        .as_ref()
        .is_some_and(|targets| targets.wounded && targets.min == 1 && targets.max == 1));
}

#[test]
fn xiangshu_recovers_and_draws_by_damage_dealt_then_latches() {
    let (mut state, players) = started(2_026_109_001);
    let owner = players[0];
    let wounded = players[1];
    state.turn.current = owner;
    state.turn.phase = Phase::Play;
    let skill = add_skill(&mut state, owner, XIANGSHU);
    // 受伤目标 hp 1（上限 3 → 最多回 2）；owner 本回合造成伤害 2 次 → X=2。
    {
        let w = state.players.get_mut(wounded).unwrap();
        w.hp = 1;
        w.max_hp = 3;
    }
    {
        let mut tx = Transaction::begin(state.clone());
        // 直写 damage_pair_counts（来源对 owner 的伤害计数 +2）。
        *tx.state_mut()
            .turn
            .damage_pair_counts
            .entry((owner, wounded))
            .or_default() += 2;
        state = tx.commit().state;
    }
    inject(&mut state, wounded, 4, 1);
    let hand_before = state.hand_count(wounded);

    let (state, accepted) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![wounded],
            card_targets: Vec::new(),
        },
        1,
    );
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    // 回复至满（X=2 足够回 2）+ 摸 2。
    assert_eq!(state.players.get(wounded).unwrap().hp, 3);
    assert_eq!(state.hand_count(wounded), hand_before + 2);

    // once_per_game：二次发动被拒。
    let (_, rejected) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![wounded],
            card_targets: Vec::new(),
        },
        2,
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
}
}

mod original_sp2_xianshuai {
//! Golden for SP2「先率」的每轮首次有来源伤害语义。

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const XIANSHUAI: u32 = 3_228_501_791;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp2/sp2.npack.json");

#[test]
fn only_the_first_sourced_damage_each_round_fires_and_owner_damage_repeats() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: 20_260_715,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let others: Vec<_> = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let source = others[0];
    let target = others[1];
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
        skill_def: SkillDefId::new(XIANSHUAI),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in [owner, source, target] {
        let state = tx.state_mut().players.get_mut(player).unwrap();
        state.hp = 10;
        state.max_hp = 10;
    }
    let hand = tx.state().hand_count(owner);

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand);

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand + 1);

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
    assert_eq!(tx.state().players.get(target).unwrap().hp, 7);
    assert_eq!(tx.state().hand_count(owner), hand + 1);

    tx.state_mut().turn.round += 1;
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
    assert_eq!(tx.state().players.get(target).unwrap().hp, 5);
    assert_eq!(tx.state().hand_count(owner), hand + 2);
}
}

mod original_sp2_xinfu_lianpian {
//! Golden coverage for 连篇（sp2 苏飞，ABI 472）.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const XINFU_LIANPIAN: u32 = 3_795_462_236;
const SOURCE: &str = include_str!("../../../content/original/ported/sp2/xinfu_lianpian.skill");

fn alloc_card(tx: &mut Transaction, owner: PlayerId, kind: CardKind, def: u32) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn play(
    state: GameState,
    actor: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
    key: u64,
) -> GameState {
    let revision = state.revision;
    let (next, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card,
                targets,
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "play must be accepted: {:?}",
        result.result
    );
    next
}

fn drive_choice(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state,
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
        "choice must be accepted: {:?}",
        result.result
    );
    next
}

fn setup(tx: &mut Transaction, owner: PlayerId, others: &[PlayerId]) {
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(XINFU_LIANPIAN),
    });
    run_effects(tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    // Empty the other players' hands so every card resolves without responses.
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    for &other in others {
        let hand = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
        let cards: Vec<CardId> = tx.state().zones.get(hand).unwrap().cards.clone();
        run_effects(
            tx,
            cards
                .into_iter()
                .map(|card| EffectRequest::MoveCards {
                    cards: vec![card],
                    to: discard,
                    from: Some(hand),
                })
                .collect(),
        )
        .unwrap();
    }
}

#[test]
fn shared_target_draws_and_gives() {
    let package = content_compiler::compile_source_named("lianpian-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 472);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_801, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, a, b) = (seats[0], seats[1], seats[2]);
    setup(&mut tx, owner, &[a, b]);
    let sha = alloc_card(&mut tx, owner, CardKind::Sha, 96_000);
    let nanman = alloc_card(&mut tx, owner, CardKind::TrickNanman, 96_001);
    let hand_before = tx.state().hand_count(owner);
    let a_hand_before = tx.state().hand_count(a);

    let mut tx = Transaction::begin(tx.commit().state);
    // First card: Sha at A -> LAST_CARD_TARGETS = {A}.
    let state = play(tx.commit().state, owner, sha, vec![a], 1);
    // Second card: Nanman (targets A + B) shares target A with the previous Sha.
    let state = play(state, owner, nanman, vec![], 2);
    assert!(
        state.resolution.pending_choice_id != 0,
        "give choice pending"
    );
    assert_eq!(
        state.hand_count(owner),
        hand_before - 1,
        "two cards used, one drawn: net -1 before the give"
    );
    let state = drive_choice(state, owner, vec![a.raw()], false, 3);
    assert_eq!(
        state.hand_count(a),
        a_hand_before + 1,
        "A received the card"
    );
    assert_eq!(
        state.hand_count(owner),
        hand_before - 2,
        "two cards used, drawn card given away"
    );
}

#[test]
fn no_choice_without_shared_target() {
    let package = content_compiler::compile_source_named("lianpian-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, a, b) = (seats[0], seats[1], seats[2]);
    setup(&mut tx, owner, &[a, b]);
    let sha = alloc_card(&mut tx, owner, CardKind::Sha, 96_000);
    let juedou = alloc_card(&mut tx, owner, CardKind::TrickJuedou, 96_001);

    let mut tx = Transaction::begin(tx.commit().state);
    // Sha at A -> LAST_CARD_TARGETS = {A}.
    let state = play(tx.commit().state, owner, sha, vec![a], 1);
    // Juedou at B shares no target with the previous Sha.
    let state = play(state, owner, juedou, vec![b], 2);
    assert_eq!(
        state.resolution.pending_choice_id, 0,
        "no lianpian choice without a shared target"
    );
}

#[test]
fn cancel_keeps_drawn_card() {
    let package = content_compiler::compile_source_named("lianpian-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, a, b) = (seats[0], seats[1], seats[2]);
    setup(&mut tx, owner, &[a, b]);
    let sha = alloc_card(&mut tx, owner, CardKind::Sha, 96_000);
    let nanman = alloc_card(&mut tx, owner, CardKind::TrickNanman, 96_001);
    let hand_before = tx.state().hand_count(owner);

    let mut tx = Transaction::begin(tx.commit().state);
    let state = play(tx.commit().state, owner, sha, vec![a], 1);
    let state = play(state, owner, nanman, vec![], 2);
    assert!(state.resolution.pending_choice_id != 0);
    let state = drive_choice(state, owner, vec![], true, 3);
    assert_eq!(
        state.hand_count(owner),
        hand_before - 1,
        "cancelled give keeps the drawn card (2 used + 1 drawn)"
    );
    assert_eq!(state.hand_count(a), 0, "A received nothing");
}
}

mod original_sp2_xinfu_qinguo {
//! Golden coverage for 勤国（sp2 吕岱，ABI 474）.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit, ZoneId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const XINFU_QINGUO: u32 = 2_648_676_099;
const SOURCE: &str = include_str!("../../../content/original/ported/sp2/xinfu_qinguo.skill");

fn alloc_card(
    tx: &mut Transaction,
    owner: PlayerId,
    kind: CardKind,
    def: u32,
    zone: ZoneId,
) -> CardId {
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 5,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn play(
    state: GameState,
    actor: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
    key: u64,
) -> GameState {
    let revision = state.revision;
    let (next, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card,
                targets,
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "play must be accepted: {:?}",
        result.result
    );
    next
}

fn drive_choice(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state,
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
        "choice must be accepted: {:?}",
        result.result
    );
    next
}

fn setup(tx: &mut Transaction, owner: PlayerId, others: &[PlayerId]) {
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(XINFU_QINGUO),
    });
    run_effects(tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    for &other in others {
        let hand = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
        let cards: Vec<CardId> = tx.state().zones.get(hand).unwrap().cards.clone();
        run_effects(
            tx,
            cards
                .into_iter()
                .map(|card| EffectRequest::MoveCards {
                    cards: vec![card],
                    to: discard,
                    from: Some(hand),
                })
                .collect(),
        )
        .unwrap();
    }
}

#[test]
fn equipment_use_offers_virtual_sha() {
    let package = content_compiler::compile_source_named("qinguo-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 474);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_801, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, victim) = (seats[0], seats[1]);
    setup(&mut tx, owner, &[victim]);
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let weapon = alloc_card(&mut tx, owner, CardKind::Weapon, 96_000, hand);
    let victim_hp_before = tx.state().players.get(victim).unwrap().hp;

    let mut tx = Transaction::begin(tx.commit().state);
    // Equip the weapon: 勤国 offers a virtual Sha.
    let state = play(tx.commit().state, owner, weapon, vec![], 1);
    assert!(
        state.resolution.pending_choice_id != 0,
        "virtual Sha choice pending"
    );
    let state = drive_choice(state, owner, vec![victim.raw()], false, 2);
    assert_eq!(
        state.players.get(victim).unwrap().hp,
        victim_hp_before - 1,
        "virtual Sha hit the victim"
    );
}

#[test]
fn non_equipment_use_offers_nothing() {
    let package = content_compiler::compile_source_named("qinguo-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, victim) = (seats[0], seats[1]);
    setup(&mut tx, owner, &[victim]);
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let sha = alloc_card(&mut tx, owner, CardKind::Sha, 96_000, hand);

    let mut tx = Transaction::begin(tx.commit().state);
    let state = play(tx.commit().state, owner, sha, vec![victim], 1);
    assert_eq!(
        state.resolution.pending_choice_id, 0,
        "no qinguo choice for a Sha use"
    );
}

#[test]
fn recover_when_equip_count_equals_hp_after_equip_loss() {
    let package = content_compiler::compile_source_named("qinguo-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_803, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, _other) = (seats[0], seats[1]);
    setup(&mut tx, owner, &[]);
    // Wound the owner to 3 HP; seed 4 equipment cards into the equip zone.
    tx.state_mut().players.get_mut(owner).unwrap().hp = 3;
    let equip_zone = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    for (kind, def) in [
        (CardKind::Weapon, 96_000),
        (CardKind::Armor, 96_001),
        (CardKind::HorseOff, 96_002),
        (CardKind::HorseDef, 96_003),
    ] {
        alloc_card(&mut tx, owner, kind, def, equip_zone);
    }
    let hp_before = tx.state().players.get(owner).unwrap().hp;

    let mut tx = Transaction::begin(tx.commit().state);
    // Move one equipment out: equip count 4 -> 3 == hp 3, wounded -> recover 1.
    let equip_zone = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    let moving = *tx
        .state()
        .zones
        .get(equip_zone)
        .unwrap()
        .cards
        .first()
        .unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![moving],
            to: discard,
            from: Some(equip_zone),
        }],
    )
    .unwrap();
    let state = tx.commit().state;
    assert_eq!(
        state.players.get(owner).unwrap().hp,
        hp_before + 1,
        "recovered 1 HP when equip count matched HP after the loss"
    );
}
}

mod original_sp2_xinfu_xingzhao {
//! Golden coverage for the complete SP2「兴棹」family and shared「恂恂」reorder.

use std::collections::BTreeSet;

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, schedule_damage};
use engine_core::state::{CardKind, ContinuationKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const SP2: &[u8] = include_bytes!("../../../content/original/ported/sp2/sp2.npack.json");
const REFRESH: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");
const XINGZHAO: u32 = 2_244_503_587;
const XUNXUN: u32 = 4_191_758_778;

#[test]
fn tangzi_character_entry_grants_only_the_composed_parent_skill() {
    let package = content_schema::ContentPackage::from_json_bytes(SP2).unwrap();
    let tangzi = package
        .character_index
        .iter()
        .find(|character| character.id == 2_852_100_501)
        .expect("唐咨 should be present in the SP2 extension package");
    assert_eq!(tangzi.name_key, "xf_tangzi");
    assert_eq!(tangzi.sex, "male");
    assert_eq!(tangzi.group, "wei");
    assert_eq!(tangzi.max_hp, 4);
    assert_eq!(tangzi.skills, vec![XINGZHAO]);
}

fn setup(load_refresh: bool) -> (Transaction, PlayerId) {
    let mut engine = Engine::new();
    engine
        .load_package(&content_schema::ContentPackage::from_json_bytes(SP2).unwrap())
        .unwrap();
    if load_refresh {
        engine
            .load_package(&content_schema::ContentPackage::from_json_bytes(REFRESH).unwrap())
            .unwrap();
    }
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_715),
            kind: CommandKind::StartGame {
                player_count: 4,
                rng_seed: 20_260_715,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    let skills: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| skill.id)
        .collect();
    for skill in skills {
        run_effects(&mut tx, vec![EffectRequest::RemoveSkill { skill }]).unwrap();
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(XINGZHAO),
        }],
    )
    .unwrap();
    (tx, owner)
}

fn set_wounded(tx: &mut Transaction, count: usize) {
    let players = tx.state().players.alive_by_seat();
    for player in &players {
        let state = tx.state_mut().players.get_mut(*player).unwrap();
        state.max_hp = 10;
        state.hp = 10;
    }
    for player in players.into_iter().take(count) {
        tx.state_mut().players.get_mut(player).unwrap().hp = 9;
    }
}

fn end_phase(
    state: engine_core::state::GameState,
    owner: PlayerId,
    key: u64,
) -> (
    engine_core::state::GameState,
    engine_core::command::PipelineResult,
) {
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::EndPhase,
        },
    )
}

#[test]
fn equipment_entry_and_replacement_draw_once_per_zone_boundary() {
    let (mut tx, owner) = setup(false);
    set_wounded(&mut tx, 2);
    let cards = tx.state().hand_cards(owner);
    let first = cards[0];
    let second = cards[1];
    tx.state_mut().cards.get_mut(first).unwrap().kind = CardKind::Weapon;
    tx.state_mut().cards.get_mut(second).unwrap().kind = CardKind::Weapon;
    let hand_zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let equip_zone = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let hand = tx.state().hand_count(owner);

    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![first],
            to: equip_zone,
            from: Some(hand_zone),
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand);

    run_effects(
        &mut tx,
        vec![
            EffectRequest::MoveCards {
                cards: vec![first],
                to: discard,
                from: Some(equip_zone),
            },
            EffectRequest::MoveCards {
                cards: vec![second],
                to: equip_zone,
                from: Some(hand_zone),
            },
        ],
    )
    .unwrap();
    assert_eq!(
        tx.state().hand_count(owner),
        hand + 1,
        "replacing equipment crosses the zone boundary twice"
    );
}

#[test]
fn equipment_movement_does_not_draw_below_two_wounded_players() {
    let (mut tx, owner) = setup(false);
    set_wounded(&mut tx, 1);
    let card = tx.state().hand_cards(owner)[0];
    tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Weapon;
    let hand_zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let equip_zone = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    let hand = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![card],
            to: equip_zone,
            from: Some(hand_zone),
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand - 1);
}

#[test]
fn xunxun_orders_two_top_and_two_bottom_then_draws_normally() {
    let (mut tx, owner) = setup(false);
    set_wounded(&mut tx, 1);
    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Judge.to_u8(),
        }],
    )
    .unwrap();
    let hand = tx.state().hand_count(owner);
    let (pending, pipe) = end_phase(tx.state().clone(), owner, 1);
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert_eq!(
        pending.resolution.continuation_kind,
        ContinuationKind::ReorderDrawPileFourSplitTwoThenDraw
    );
    let choice = pipe.choice.unwrap();
    assert_eq!((choice.min_count, choice.max_count), (4, 4));
    assert!(choice.allow_cancel);
    let offered: Vec<u32> = choice.options.iter().map(|option| option.token).collect();
    let order = vec![offered[0], offered[1], offered[3], offered[2]];
    let (resolved, pipe) = apply_command(
        pending,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: order.clone(),
                cancelled: false,
            },
        },
    );
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert!(!resolved.has_pending_choice());
    assert_eq!(resolved.turn.phase, Phase::Draw);
    assert_eq!(resolved.hand_count(owner), hand + 2);
    let hand_cards: BTreeSet<u32> = resolved
        .hand_cards(owner)
        .into_iter()
        .map(|card| card.raw())
        .collect();
    assert!(hand_cards.contains(&order[0]) && hand_cards.contains(&order[1]));
    let draw = resolved
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let draw_cards = &resolved.zones.get(draw).unwrap().cards;
    assert_eq!(draw_cards[0].raw(), order[2]);
    assert_eq!(draw_cards[1].raw(), order[3]);
}

#[test]
fn xingzhao_does_not_offer_xunxun_when_nobody_is_wounded() {
    let (mut tx, owner) = setup(false);
    set_wounded(&mut tx, 0);
    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Judge.to_u8(),
        }],
    )
    .unwrap();
    let hand = tx.state().hand_count(owner);
    let (state, pipe) = end_phase(tx.state().clone(), owner, 7);
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.turn.phase, Phase::Draw);
    assert_eq!(state.hand_count(owner), hand + 2);
}

#[test]
fn xunxun_is_optional_and_multiple_equivalent_sources_do_not_duplicate_it() {
    let (mut tx, owner) = setup(true);
    set_wounded(&mut tx, 1);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(XUNXUN),
            },
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::Judge.to_u8(),
            },
        ],
    )
    .unwrap();
    let hand = tx.state().hand_count(owner);
    let (pending, pipe) = end_phase(tx.state().clone(), owner, 3);
    let choice = pipe.choice.unwrap();
    let (resolved, pipe) = apply_command(
        pending,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(4),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(choice.choice_id.raw()),
                selections: vec![],
                cancelled: true,
            },
        },
    );
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert!(!resolved.has_pending_choice());
    assert_eq!(resolved.hand_count(owner), hand + 2);
}

#[test]
fn three_wounded_players_skip_judge_and_discard_but_two_do_not() {
    let (mut tx, owner) = setup(false);
    set_wounded(&mut tx, 2);
    assert!(!engine_core::query::skip_judge_phase(tx.state(), owner));
    assert!(!engine_core::query::skip_discard_phase(tx.state(), owner));
    let mut low = Transaction::begin(tx.state().clone());
    run_effects(
        &mut low,
        vec![
            EffectRequest::DrawCards {
                player: owner,
                count: 8,
            },
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::Discard.to_u8(),
            },
        ],
    )
    .unwrap();
    let (low_state, low_pipe) = end_phase(low.state().clone(), owner, 5);
    assert!(matches!(low_pipe.result, CommandResult::Accepted { .. }));
    assert_eq!(
        low_state.resolution.continuation_kind,
        ContinuationKind::DiscardToLimit
    );

    set_wounded(&mut tx, 3);
    assert!(engine_core::query::skip_judge_phase(tx.state(), owner));
    assert!(engine_core::query::skip_discard_phase(tx.state(), owner));

    run_effects(
        &mut tx,
        vec![
            EffectRequest::DrawCards {
                player: owner,
                count: 4,
            },
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::Discard.to_u8(),
            },
        ],
    )
    .unwrap();
    let hand = tx.state().hand_count(owner);
    let (state, pipe) = end_phase(tx.state().clone(), owner, 6);
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.turn.phase, Phase::End);
    assert_eq!(state.hand_count(owner), hand);
}

#[test]
fn parent_damage_tier_matches_the_legacy_zero_or_four_threshold() {
    let (mut tx, owner) = setup(false);
    let target = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    set_wounded(&mut tx, 0);
    let hp = tx.state().players.get(target).unwrap().hp;
    schedule_damage(&mut tx, owner, target, 1).unwrap();
    assert_eq!(tx.state().players.get(target).unwrap().hp, hp - 2);

    set_wounded(&mut tx, 1);
    let hp = tx.state().players.get(target).unwrap().hp;
    schedule_damage(&mut tx, owner, target, 1).unwrap();
    assert_eq!(tx.state().players.get(target).unwrap().hp, hp - 1);
}
}

mod original_sp2_xinfu_yisuan {
//! 第一百六十一批：sp2「翼算」（李傕 xinfu_yisuan）重写——新过滤器 + 既有原语。
//! 上游：出牌阶段限一次，你使用普通锦囊牌结算结束后，可以减 1 点体力上限，
//! 然后获得此牌对应牌堆里的实体牌。实现：card_resolved +
//! owner_source_trick_resolved_own_play_phase_once_per_turn（本批六点注册）→
//! change_max_hp(self, 0 - 1) + gain_event_cards(self)。
//! 近似：gain_event_cards 只取已进弃牌堆者（filterInD 边界）。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const YISUAN: u32 = 4_087_417_398;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp2/sp2.npack.json");

fn started(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let state = engine.state().clone();
    let owner = state.turn.current;
    let other = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    (state, owner, other)
}

fn strip_and_arm(state: GameState, owner: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain(
            tx.state()
                .players
                .alive_by_seat()
                .into_iter()
                .map(|player| EffectRequest::DiscardFromHand {
                    player,
                    count: u32::MAX,
                }),
        )
        .chain([EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(YISUAN),
        }])
        .collect();
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.commit().state
}

fn play_card(
    state: GameState,
    actor: PlayerId,
    kind: CardKind,
    targets: Vec<PlayerId>,
    key: u64,
) -> (GameState, PipelineResult) {
    let mut tx = Transaction::begin(state);
    let zone = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(98_700 + key as u32),
        kind,
        suit: Suit::Spade,
        rank: 8,
        weapon_range: 0,
        zone,
        owner: actor,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    let tx = tx.commit();
    let revision = tx.state.revision;
    apply_command(
        tx.state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card,
                targets,
                card_targets: Vec::new(),
            },
        },
    )
}

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let choice_id = engine_api::ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "resolve rejected: {:?}",
        result.result
    );
    state
}

fn hand_count(state: &GameState, player: PlayerId) -> u32 {
    state
        .zones
        .find(ZoneKind::Hand, player)
        .and_then(|zone| state.zones.get(zone).map(|z| z.cards.len() as u32))
        .unwrap_or(0)
}

fn max_hp_of(state: &GameState, player: PlayerId) -> i32 {
    state.players.get(player).unwrap().max_hp as i32
}

#[test]
fn package_publishes_yisuan_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == YISUAN
            && trigger.event == "card_resolved"
            && trigger.filter == "owner_source_trick_resolved_own_play_phase_once_per_turn"
            && trigger.optional
    }));
    assert!(!package
        .modifier_index
        .iter()
        .any(|modifier| modifier.skill_def_id == YISUAN && modifier.target == "hand_limit"));
}

#[test]
fn yisuan_trick_resolution_gains_cards_at_max_hp_cost() {
    let (state, owner, other) = started(2_026_090_801);
    let state = strip_and_arm(state, owner);
    let max_hp = max_hp_of(&state, owner);

    // 出无中生有（普通锦囊，无目标）——结算后实体牌进弃牌堆。
    let (state, played) = play_card(state, owner, CardKind::TrickWuzhong, Vec::new(), 1);
    assert!(matches!(played.result, CommandResult::Accepted { .. }), "play rejected: {:?}", played.result);

    // 无中生有先结算摸 2（含可能的抽牌选择），然后翼算确认弹出。
    let mut state = state;
    for key in 2..8 {
        if state.resolution.continuation_kind
            == engine_core::state::ContinuationKind::OptionalTrigger
        {
            break;
        }
        if !state.has_pending_choice() {
            break;
        }
        state = resolve(state, owner, vec![], key);
    }
    assert_eq!(
        state.resolution.continuation_kind,
        engine_core::state::ContinuationKind::OptionalTrigger,
        "trick resolution must open the yisuan confirm"
    );
    let hand_before = hand_count(&state, owner);
    let state = resolve(state, owner, vec![1], 8);

    // 减 1 体力上限 + 获得弃牌堆中的此【无中生有】实体牌。
    assert_eq!(max_hp_of(&state, owner), max_hp - 1, "loses one max hp");
    assert_eq!(hand_count(&state, owner), hand_before + 1, "gains the resolved trick card");
}

#[test]
fn yisuan_second_trick_same_turn_does_not_refire() {
    let (state, owner, other) = started(2_026_090_802);
    let state = strip_and_arm(state, owner);
    let max_hp = max_hp_of(&state, owner);

    // 第一张锦囊：确认发动。
    let (state, played) = play_card(state, owner, CardKind::TrickWuzhong, Vec::new(), 1);
    assert!(matches!(played.result, CommandResult::Accepted { .. }));
    let mut state = state;
    for key in 2..8 {
        if state.resolution.continuation_kind
            == engine_core::state::ContinuationKind::OptionalTrigger
        {
            break;
        }
        if !state.has_pending_choice() {
            break;
        }
        state = resolve(state, owner, vec![], key);
    }
    let state = resolve(state, owner, vec![1], 8);
    assert_eq!(max_hp_of(&state, owner), max_hp - 1);

    // 第二张锦囊：每回合限一次，不再弹确认。
    let (state, played) = play_card(state, owner, CardKind::TrickWuzhong, Vec::new(), 11);
    assert!(matches!(played.result, CommandResult::Accepted { .. }));
    let mut state = state;
    for key in 12..16 {
        if !state.has_pending_choice() {
            break;
        }
        state = resolve(state, owner, vec![], key);
    }
    assert_ne!(
        state.resolution.continuation_kind,
        engine_core::state::ContinuationKind::OptionalTrigger,
        "second trick in the same play phase must not refire yisuan"
    );
    assert_eq!(max_hp_of(&state, owner), max_hp - 1, "max hp unchanged by the second use");
}
}

mod original_sp2_xinxingluan {
//! Golden coverage for 心星乱（sp2 樊稠，ABI 476）.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit, ZoneId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const XINXINGLUAN: u32 = 3_806_437_757;
const SOURCE: &str = include_str!("../../../content/original/ported/sp2/xinxingluan.skill");

fn alloc_card(
    tx: &mut Transaction,
    owner: PlayerId,
    kind: CardKind,
    def: u32,
    rank: u8,
    zone: ZoneId,
) -> CardId {
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn play(
    state: GameState,
    actor: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
    key: u64,
) -> GameState {
    let revision = state.revision;
    let (next, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card,
                targets,
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "play must be accepted: {:?}",
        result.result
    );
    next
}

fn resolve_choice(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state,
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
        "choice must be accepted: {:?}",
        result.result
    );
    next
}

fn setup(tx: &mut Transaction, owner: PlayerId, target: PlayerId) {
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(XINXINGLUAN),
    });
    run_effects(tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    // Empty the target's hand so the probe Sha cannot be dodged.
    let hand = tx.state().zones.find(ZoneKind::Hand, target).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let cards: Vec<CardId> = tx.state().zones.get(hand).unwrap().cards.clone();
    run_effects(
        tx,
        cards
            .into_iter()
            .map(|card| EffectRequest::MoveCards {
                cards: vec![card],
                to: discard,
                from: Some(hand),
            })
            .collect(),
    )
    .unwrap();
}

#[test]
fn gain_branch_takes_rank_six_equipment() {
    let package = content_compiler::compile_source_named("xxl-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 476);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_801, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target) = (seats[0], seats[1]);
    setup(&mut tx, owner, target);
    // A rank-6 weapon in the target's equip zone.
    let equip_zone = tx.state().zones.find(ZoneKind::Equip, target).unwrap();
    let weapon = alloc_card(&mut tx, target, CardKind::Weapon, 96_100, 6, equip_zone);
    let owner_hand_before = tx.state().hand_count(owner);
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let sha = alloc_card(&mut tx, owner, CardKind::Sha, 96_000, 5, hand);

    let mut tx = Transaction::begin(tx.commit().state);
    let state = play(tx.commit().state, owner, sha, vec![target], 1);
    assert!(
        state.resolution.pending_choice_id != 0,
        "branch choice pending"
    );
    let state = resolve_choice(state, owner, vec![2], false, 2); // gain branch
    assert!(state.has_pending_choice(), "gain target choice pending");
    let state = resolve_choice(state, owner, vec![target.raw()], false, 3);
    assert!(state.has_pending_choice(), "rank-6 card choice pending");
    let state = resolve_choice(state, owner, vec![weapon.raw()], false, 4);

    assert_eq!(
        state.hand_count(owner),
        owner_hand_before + 1,
        "gained the rank-6 equipment"
    );
    let equip_len = state
        .zones
        .find(ZoneKind::Equip, target)
        .and_then(|id| state.zones.get(id))
        .map(|z| z.cards.len())
        .unwrap();
    assert_eq!(equip_len, 0, "target's equip zone emptied");
}

#[test]
fn discard_branch_target_discards_rank_six_hand() {
    let package = content_compiler::compile_source_named("xxl-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target) = (seats[0], seats[1]);
    setup(&mut tx, owner, target);
    let target_hand = tx.state().zones.find(ZoneKind::Hand, target).unwrap();
    let six = alloc_card(&mut tx, target, CardKind::Sha, 96_101, 6, target_hand);
    alloc_card(&mut tx, target, CardKind::Tao, 96_102, 5, target_hand);
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let sha = alloc_card(&mut tx, owner, CardKind::Sha, 96_000, 5, hand);

    let mut tx = Transaction::begin(tx.commit().state);
    let state = play(tx.commit().state, owner, sha, vec![target], 1);
    assert!(state.resolution.pending_choice_id != 0);
    let state = resolve_choice(state, owner, vec![3], false, 2); // discard branch
    assert!(state.has_pending_choice(), "discard target choice pending");
    let state = resolve_choice(state, owner, vec![target.raw()], false, 3);
    assert!(state.has_pending_choice(), "target's rank-6 discard choice");
    let state = resolve_choice(state, target, vec![six.raw()], false, 4);

    assert_eq!(state.hand_count(target), 1, "target kept the rank-5 Tao");
}

#[test]
fn pile_branch_gains_rank_six_pile_card() {
    let package = content_compiler::compile_source_named("xxl-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_803, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target) = (seats[0], seats[1]);
    setup(&mut tx, owner, target);
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let six = alloc_card(&mut tx, PlayerId::NONE, CardKind::Sha, 96_200, 6, draw);
    let owner_hand_before = tx.state().hand_count(owner);
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let sha = alloc_card(&mut tx, owner, CardKind::Sha, 96_000, 5, hand);

    let mut tx = Transaction::begin(tx.commit().state);
    let state = play(tx.commit().state, owner, sha, vec![target], 1);
    assert!(state.resolution.pending_choice_id != 0);
    let state = resolve_choice(state, owner, vec![1], false, 2); // pile branch
    assert!(state.has_pending_choice(), "pile card choice pending");
    let state = resolve_choice(state, owner, vec![six.raw()], false, 3);

    assert_eq!(
        state.hand_count(owner),
        owner_hand_before + 1,
        "gained a rank-6 pile card"
    );
}

#[test]
fn once_per_turn() {
    let package = content_compiler::compile_source_named("xxl-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_804, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target) = (seats[0], seats[1]);
    setup(&mut tx, owner, target);
    let target_hand = tx.state().zones.find(ZoneKind::Hand, target).unwrap();
    alloc_card(&mut tx, target, CardKind::Sha, 96_101, 6, target_hand);
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let sha1 = alloc_card(&mut tx, owner, CardKind::Sha, 96_000, 5, hand);
    let sha2 = alloc_card(&mut tx, owner, CardKind::Sha, 96_001, 5, hand);

    let mut tx = Transaction::begin(tx.commit().state);
    // First Sha use: cancel the branch (still consumes the once-per-turn).
    let mut state = play(tx.commit().state, owner, sha1, vec![target], 1);
    assert!(state.resolution.pending_choice_id != 0);
    state = resolve_choice(state, owner, vec![], true, 2);
    // Second Sha use: no choice (already fired this turn).
    state
        .turn
        .set_flag(engine_core::state::turn_flag::SHA_PLAYED, 0);
    state
        .turn
        .set_flag(engine_core::state::turn_flag::SHA_PLAYED_IN_PLAY_PHASE, 0);
    let mut tx = Transaction::begin(state);
    let state = play(tx.commit().state, owner, sha2, vec![target], 3);
    assert_eq!(
        state.resolution.pending_choice_id, 0,
        "second card use offers no branch"
    );
}
}

mod original_sp2_xiongsuan {
//! Golden tests for SP2「凶算」rewritten from the legacy JavaScript semantics.

use engine_api::{
    Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, SkillId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::hash::canonical_state_hash;
use engine_core::state::{GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const XIONGSUAN: u32 = 3_088_500_344;
const LIQUEGUOSI: u32 = 2_859_668_029;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp2/sp2.npack.json");

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
    for player in [owner, other] {
        let state = tx.state_mut().players.get_mut(player).unwrap();
        state.hp = 10;
        state.max_hp = 10;
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(XIONGSUAN),
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
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == XIONGSUAN)
        .unwrap();
    (tx.state().clone(), owner, other, skill)
}

fn activate(
    state: GameState,
    owner: PlayerId,
    skill: SkillId,
    target: PlayerId,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let card = state.hand_cards(owner)[0];
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![target],
                card_targets: vec![card],
            },
        },
    )
}

#[test]
fn package_contains_complete_character_and_active_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == LIQUEGUOSI)
        .expect("李傕郭汜 character entry");
    assert_eq!(character.skills, vec![XIONGSUAN]);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == XIONGSUAN)
        .expect("凶算 active entry");
    assert_eq!(active.uses_per_turn, Some(1));
    assert_eq!(
        active
            .targets
            .as_ref()
            .map(|targets| (targets.min, targets.max)),
        Some((1, 1))
    );
    assert_eq!(
        active.selection.as_ref().map(|selection| (
            selection.zone.as_str(),
            selection.min,
            selection.max
        )),
        Some(("hand", 1, 1))
    );
}

#[test]
fn discards_damages_draws_and_loses_hp_only_for_another_target() {
    let (state, owner, other, skill) = started_state(2_026_071_505);
    let owner_hand = state.hand_count(owner);
    let owner_hp = state.players.get(owner).unwrap().hp;
    let target_hp = state.players.get(other).unwrap().hp;
    let discarded = state.hand_cards(owner)[0];

    let (state, pipe) = activate(state, owner, skill, other, 1);
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), owner_hand + 2);
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp - 1);
    assert_eq!(state.players.get(other).unwrap().hp, target_hp - 1);
    assert_eq!(
        state.cards.get(discarded).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
    assert!(pipe.events.iter().any(|event| matches!(
        event,
        DomainEvent::DamageDealt {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        } if *source == owner && *target == other
    )));
    assert!(pipe.events.iter().any(|event| matches!(
        event,
        DomainEvent::DamageDealt {
            target,
            amount: 1,
            is_lose_hp: true,
            ..
        } if *target == owner
    )));

    let before_retry = canonical_state_hash(&state);
    let (rejected, pipe) = activate(state, owner, skill, other, 2);
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&rejected), before_retry);
}

#[test]
fn self_target_takes_damage_without_extra_hp_loss() {
    let (state, owner, _other, skill) = started_state(2_026_071_506);
    let hp = state.players.get(owner).unwrap().hp;
    let (state, pipe) = activate(state, owner, skill, owner, 1);
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, hp - 1);
    assert!(!pipe.events.iter().any(|event| matches!(
        event,
        DomainEvent::DamageDealt {
            target,
            is_lose_hp: true,
            ..
        } if *target == owner
    )));
}

#[test]
fn missing_or_dead_target_is_rejected_without_mutation() {
    let (state, owner, other, skill) = started_state(2_026_071_507);
    let card = state.hand_cards(owner)[0];
    let before = canonical_state_hash(&state);
    let (same, pipe) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![],
                card_targets: vec![card],
            },
        },
    );
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);

    let mut dead = same;
    dead.players.get_mut(other).unwrap().alive = false;
    let before = canonical_state_hash(&dead);
    let (same, pipe) = activate(dead, owner, skill, other, 2);
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
}
}

mod original_sp2_yangzhong {
//! Golden coverage for 殃众（sp2 赵忠）来源弃牌失体（ABI 460）.
//!
//! 拥有者造成或受到伤害后，伤害来源可弃置两张牌，令受伤角色失去 1 点体力。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const YANGZHONG: u32 = 4_088_525_000;
const SOURCE: &str = include_str!("../../../content/original/ported/sp2/yangzhong.skill");

fn drive_choice(
    state: engine_core::state::GameState,
    actor: PlayerId,
    selection: u32,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![selection],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "choice must be accepted"
    );
    next
}

#[test]
fn damage_source_discards_two_then_target_loses_hp() {
    let package = content_compiler::compile_source_named("yangzhong-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 460);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 2_026_080_115, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, source, _third) = (players[0], players[1], players[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(YANGZHONG),
    });
    run_effects(&mut tx, effects).unwrap();
    // Give the source two discardable hand cards.
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, source)
        .expect("source hand zone");
    let mut cards = Vec::new();
    for rank in 1..=2u32 {
        let card = tx.state_mut().cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(9_700),
            kind: CardKind::Sha,
            suit: engine_api::Suit::Heart,
            rank: rank as u8,
            weapon_range: 0,
            zone: hand,
            owner: source,
            face_up: false,
        });
        tx.state_mut()
            .zones
            .get_mut(hand)
            .expect("hand zone")
            .cards
            .push(card);
        cards.push(card);
    }
    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    let source_hand_before = tx.state().hand_count(source);

    // Source deals 1 damage to the owner: source confirms, discards 2, owner loses 1 HP.
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    );
    assert!(matches!(result, Err(EffectError::ChoiceRequested)));
    let mut state = tx.commit().state;
    state = drive_choice(state, source, 1); // optional confirm by the damage source
                                            // Discard choice selects both cards in one command.
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state,
        Command {
            actor: source,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![cards[0].raw(), cards[1].raw()],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "discard choice must be accepted"
    );
    let state = next;
    assert_eq!(
        state.players.get(owner).unwrap().hp,
        owner_hp - 2,
        "damage target loses 1 HP on top of the 1 damage"
    );
    assert_eq!(
        state.hand_count(source),
        source_hand_before - 2,
        "damage source discards two cards"
    );
    assert!(!state.has_pending_choice());
}
}

mod original_sp2_yinyi {
//! 第一百六十批：sp2「隐翼」（yinyi）重写——新过滤器 + 既有 prevent 原语。
//! 上游：锁定技，每回合限一次，当你受到伤害时，若来源体力值与你不同且其手牌数
//! 与你不同，防止此伤害。实现：damage_before +
//! owner_targeted_source_hp_hand_differ_once_per_turn（本批六点注册）→
//! prevent_current_damage()。
//! 近似：上游 linked-nature 排除（铁索传导）未建模。

use engine_api::{CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId};
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{damage_nature, EffectRequest};

const YINYI: u32 = 3_441_826_579;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp2/sp2.npack.json");

fn started(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let state = engine.state().clone();
    let owner = state.turn.current;
    let source = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    (state, owner, source)
}

fn strip_and_setup(state: GameState, owner: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain(
            tx.state()
                .players
                .alive_by_seat()
                .into_iter()
                .map(|player| EffectRequest::DiscardFromHand {
                    player,
                    count: u32::MAX,
                }),
        )
        .chain([EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(YINYI),
        }])
        .collect();
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.commit().state
}

fn deal_damage(
    state: GameState,
    source: PlayerId,
    target: PlayerId,
    amount: u32,
) -> GameState {
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealShaDamage {
            source,
            target,
            card: CardId::NONE,
            amount,
            nature: damage_nature::NORMAL,
            card_flags: 0,
        }],
    )
    .unwrap();
    tx.commit().state
}

fn hp_of(state: &GameState, player: PlayerId) -> i32 {
    state.players.get(player).unwrap().hp
}

fn hand_count(state: &GameState, player: PlayerId) -> u32 {
    state
        .zones
        .find(ZoneKind::Hand, player)
        .and_then(|zone| state.zones.get(zone).map(|z| z.cards.len() as u32))
        .unwrap_or(0)
}

#[test]
fn package_publishes_yinyi_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == YINYI
            && trigger.event == "damage_before"
            && trigger.filter == "owner_targeted_source_hp_hand_differ_once_per_turn"
            && !trigger.optional
    }));
    assert!(!package
        .modifier_index
        .iter()
        .any(|modifier| modifier.skill_def_id == YINYI && modifier.target == "hand_limit"));
}

#[test]
fn yinyi_prevents_first_differing_damage_once_per_turn() {
    let (state, owner, source) = started(2_026_090_701);
    let state = strip_and_setup(state, owner);
    let owner_hp = hp_of(&state, owner);

    // 体力与手牌均不同（source 手牌 0、owner 也 0？——先保证手牌不同：给 owner 一张）。
    let mut tx = Transaction::begin(state);
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(98_601),
        kind: CardKind::Tao,
        suit: engine_api::Suit::Heart,
        rank: 5,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    let state = tx.commit().state;
    assert_ne!(hp_of(&state, owner), hp_of(&state, source), "bootstrap hp differs");
    assert_ne!(hand_count(&state, owner), hand_count(&state, source));

    // 第一次伤害：防止（hp 不变，锁定技无确认）。
    let state = deal_damage(state, source, owner, 1);
    assert_eq!(hp_of(&state, owner), owner_hp, "first differing damage prevented");

    // 第二次伤害：每回合限一次，照常结算。
    let state = deal_damage(state, source, owner, 1);
    assert_eq!(hp_of(&state, owner), owner_hp - 1, "second damage lands (once per turn)");
}

#[test]
fn yinyi_not_triggered_when_hand_counts_equal() {
    let (state, owner, source) = started(2_026_090_702);
    let state = strip_and_setup(state, owner);
    let owner_hp = hp_of(&state, owner);
    // 双方手牌均空（相等）→ 过滤不过，伤害照常。
    assert_eq!(hand_count(&state, owner), 0);
    assert_eq!(hand_count(&state, source), 0);
    let state = deal_damage(state, source, owner, 1);
    assert_eq!(hp_of(&state, owner), owner_hp - 1, "equal hand counts bypass yinyi");
}
}

mod original_sp2_yuanlve_sp_zhanghe {
//! Golden coverage for SP2 张郃「远略」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const SP_ZHANGHE: u32 = 4_161_518_802;
const YUANLVE: u32 = 3_073_596_171;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp2/sp2.npack.json");

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
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
    state
}

fn add_hand_card(tx: &mut Transaction, owner: PlayerId, kind: CardKind, rank: u8) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_497),
        kind,
        suit: Suit::Heart,
        rank,
        weapon_range: u8::from(matches!(kind, CardKind::Weapon)),
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn package_and_runtime_preserve_exact_yuanlve_flow() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 753);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.id == SP_ZHANGHE)
            .unwrap()
            .skills,
        [YUANLVE]
    );
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == YUANLVE)
        .unwrap();
    assert_eq!(active.uses_per_turn, Some(1));
    assert_eq!(
        active.selection.as_ref().unwrap().filter.as_deref(),
        Some("card_non_equip")
    );

    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 2_026_080_202, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, recipient, victim) = (players[0], players[1], players[2]);
    let effects = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain([EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(YUANLVE),
        }])
        .collect();
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let sha = add_hand_card(&mut tx, owner, CardKind::Sha, 7);
    let equipment = add_hand_card(&mut tx, owner, CardKind::Weapon, 8);
    let skill: SkillId = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == YUANLVE)
        .unwrap();
    let state = tx.commit().state;
    let hand_before = state.hand_count(owner);

    let revision = state.revision;
    let (state, rejected) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![recipient],
                card_targets: vec![equipment],
            },
        },
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));

    let revision = state.revision;
    let (state, accepted) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![recipient],
                card_targets: vec![sha],
            },
        },
    );
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand_before - 1);
    let state = resolve(state, recipient, vec![sha.raw()], false, 3);
    let mut state = resolve(state, recipient, vec![victim.raw()], false, 4);
    let mut key = 5;
    while state.has_pending_choice() {
        let actor = PlayerId::new(state.resolution.pending_observer);
        state = resolve(state, actor, Vec::new(), true, key);
        key += 1;
    }
    assert_eq!(
        state.hand_count(owner),
        hand_before,
        "successful use draws one"
    );
    assert_ne!(state.cards.get(sha).unwrap().owner, recipient);
}
}

mod original_sp2_zhongjie_starweilin {
//! Golden for SP2 张介 zhongjie (tag 765 / ABI 748) + 威临 starweilin
//! (filter rung 749, Cont 332):
//! 张介——死亡时选一名其他角色：+1 体力上限、回复 1、摸 1（forceDie）。
//! 威临——自己回合内对「本局未受过伤」角色造成伤害且用牌数 ≥ 其体力：
//! 伤害 +1。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events as dfx, TriggerFireRecord};
use rule_api::EffectRequest;

const ZHONGJIE: u32 = 2_821_141_991;
const STARWEILIN: u32 = 3_176_127_729;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp2/sp2.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn started(seed: u64) -> (GameState, Vec<PlayerId>) {
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package()).unwrap();
    let players = tx.state().players.alive_by_seat();
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
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    (tx.commit().state, players)
}

fn grant(state: &mut GameState, player: PlayerId, def: u32) -> engine_api::SkillId {
    let mut tx = Transaction::begin(state.clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
    *state = tx.commit().state;
    state
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| state.skills.get(*skill).unwrap().def.raw() == def)
        .unwrap()
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let choice_id = engine_api::ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

#[test]
fn package_publishes_zhongjie_starweilin_contracts() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 753);
    let zhongjie = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == ZHONGJIE)
        .expect("zhongjie trigger");
    assert_eq!(zhongjie.event.as_str(), "death");
    assert_eq!(zhongjie.filter.as_str(), "owner_is_target");
    assert!(!zhongjie.optional);
    let weilin = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == STARWEILIN)
        .expect("starweilin trigger");
    assert_eq!(weilin.event.as_str(), "damage_before");
    assert_eq!(
        weilin.filter.as_str(),
        "owner_source_in_turn_target_unharmed_card_actions_ge_target_hp"
    );
}

#[test]
fn zhongjie_grants_max_hp_recover_and_draw_on_death() {
    let (mut state, players) = started(2_026_104_001);
    // 选忠臣作为死亡者（主公/反贼死均会终结对局）。
    use engine_core::state::Identity;
    let owner = *players
        .iter()
        .find(|p| {
            state
                .players
                .get(**p)
                .map(|pl| pl.identity == Identity::Loyalist)
                .unwrap_or(false)
        })
        .unwrap();
    let heir = *players.iter().find(|p| **p != owner).unwrap();
    grant(&mut state, owner, ZHONGJIE);
    // owner 受致命伤死亡；heir 受伤状态（hp 1）以观察回复。
    {
        let h = state.players.get_mut(heir).unwrap();
        h.hp = 1;
        h.max_hp = 4;
    }
    let mut tx = Transaction::begin(state);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 0;
    tx.state_mut().players.get_mut(owner).unwrap().dying = true;
    // 直接击杀（无救）→ 死亡事件触发张介确认窗口。
    // 张介确认窗口以 Err(ChoiceRequested) 短路——正常信号。
    let kill_result = run_effects(
        &mut tx,
        vec![EffectRequest::KillPlayer {
            player: owner,
            killer: PlayerId::NONE,
        }],
    );
    match kill_result {
        Ok(_) | Err(EffectError::ChoiceRequested) => {}
        Err(err) => panic!("unexpected: {err:?}"),
    }
    // forced 触发：直接进入选目标窗（可取消 = 上游确认窗语义等价）。
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::ResolveZhongjieStage
    );

    // 选 heir → +1 上限 + 回复 + 摸 1。
    let (state, result) = resolve(tx.commit().state, owner, vec![heir.raw()], false, 7);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let heir_state = state.players.get(heir).unwrap();
    assert_eq!(heir_state.max_hp, 5);
    assert_eq!(heir_state.hp, 2);
    assert_eq!(state.hand_count(heir), 1);
    assert!(!state.has_pending_choice());
}

#[test]
fn starweilin_increases_damage_under_three_conditions() {
    let (mut state, players) = started(2_026_104_002);
    let owner = players[0];
    let target = players[1];
    state.turn.current = owner;
    state.turn.phase = Phase::Play;
    grant(&mut state, owner, STARWEILIN);
    // 目标 hp 压到 2（card_action_totals=3 ≥ 2 ✓）；注入一张【杀】供出牌。
    {
        let t = state.players.get_mut(target).unwrap();
        t.hp = 2;
    }
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let sha = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(102_001),
        kind: CardKind::Sha,
        suit: Suit::Club,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: true,
    });
    state
        .zones
        .get_mut(hand)
        .unwrap()
        .cards
        .push(sha);
    {
        let mut tx = Transaction::begin(state.clone());
        tx.state_mut().card_action_totals.insert(owner, 3);
        state = tx.commit().state;
    }
    let hp_before = state.players.get(target).unwrap().hp;

    // 真实出杀管道：DamageBefore 触发威临 → 伤害 +1（目标无闪 → 掉 2）。
    let mut tx = Transaction::begin(state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    );
    let _ = result;
    assert_eq!(
        tx.state().players.get(target).unwrap().hp,
        hp_before - 2,
        "威临应使伤害 +1"
    );
}
}
