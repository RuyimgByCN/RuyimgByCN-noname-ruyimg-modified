//! Golden coverage for 荐相/审时（shenhua 蒯良蒯越，ABI 574）.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const JIANXIANG: u32 = 300_846_397;
const SHENSHI: u32 = 3_785_166_504;
const SOURCE: &str =
    include_str!("../../../../content/original/ported/shenhua/kuailiangkuaiyue.skill");

struct Setup {
    tx: Transaction,
    owner: PlayerId,
    other: PlayerId,
    far: PlayerId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("kl-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 3, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let other = seats[1];
    let far = seats[2];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(seats.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(JIANXIANG),
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(SHENSHI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    Setup {
        tx,
        owner,
        other,
        far,
    }
}

fn alloc_card(
    tx: &mut Transaction,
    owner: PlayerId,
    zone: engine_api::ZoneId,
    kind: CardKind,
    def: u32,
    suit: Suit,
    rank: u8,
) -> CardId {
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn hand_zone(tx: &Transaction, player: PlayerId) -> engine_api::ZoneId {
    tx.state().zones.find(ZoneKind::Hand, player).unwrap()
}

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
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "unexpected result: {:?}",
        result.result
    );
    state
}

fn dispatch(state: GameState, event: DomainEvent) -> GameState {
    let mut tx = Transaction::begin(state);
    let fired = dispatch_for_events(&mut tx, &[event], &mut TriggerFireRecord::new());
    match fired {
        Ok(effects) => match run_effects(&mut tx, effects) {
            Err(EffectError::ChoiceRequested) => {}
            Ok(_) => {}
            Err(e) => panic!("dispatch error: {e:?}"),
        },
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("dispatch error: {e:?}"),
    }
    tx.commit().state
}

#[test]
fn package_exposes_kl_contract() {
    let compiled = content_compiler::compile_source_named("kl", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 574);
    assert_eq!(compiled.skill_index.len(), 2);
    let jianxiang = compiled
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == JIANXIANG)
        .unwrap();
    assert_eq!(jianxiang.event, "card_targeted");
    assert!(jianxiang.optional);
    let shenshi_active = compiled
        .active_index
        .iter()
        .find(|a| a.skill_def_id == SHENSHI)
        .unwrap();
    assert_eq!(shenshi_active.uses_per_turn, Some(1));
    let shenshi_triggers: Vec<_> = compiled
        .trigger_index
        .iter()
        .filter(|t| t.skill_def_id == SHENSHI)
        .collect();
    assert_eq!(shenshi_triggers.len(), 2);
}

/// 荐相：受指目标后选手牌最少者摸一张。
#[test]
fn jianxiang_min_hand_draws() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    tx.state_mut().turn.current = setup.other;
    tx.state_mut().turn.phase = Phase::Play;
    let other_hand = hand_zone(&tx, setup.other);
    let used = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Sha,
        97_100,
        Suit::Spade,
        5,
    );
    let far_hand = hand_zone(&tx, setup.far);
    let f1 = alloc_card(
        &mut tx,
        setup.far,
        far_hand,
        CardKind::Tao,
        97_101,
        Suit::Heart,
        6,
    );
    let state = tx.commit().state;
    let state = dispatch(
        state,
        DomainEvent::CardUsed {
            player: setup.other,
            card: used,
            targets: vec![setup.owner],
        },
    );
    assert!(state.has_pending_choice(), "荐相 confirm");
    let state = resolve(state, setup.owner, vec![1], false, 9_001);
    assert!(state.has_pending_choice(), "荐相 target prompt");
    // 手牌最少者：owner(0) 与 far(1) 中最少者——owner 0, far 1 → 最少者 owner 自己。
    // 选择最少的其它可选者（测试断言 far 不是最少；这里 owner 手牌 0 为最少）。
    let state = resolve(state, setup.owner, vec![setup.owner.raw()], false, 9_002);
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(setup.owner), 1, "owner drew one");
    let _ = (used, f1);
}

/// 审时（阳分支）：出牌阶段限一次交牌给手牌最多者并造成伤害。
#[test]
fn shenshi_yang_gives_and_damages() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    tx.state_mut().turn.current = setup.owner;
    tx.state_mut().turn.phase = Phase::Play;
    let owner_hand = hand_zone(&tx, setup.owner);
    let gift = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Sha,
        97_102,
        Suit::Club,
        7,
    );
    let far_hand = hand_zone(&tx, setup.far);
    let f1 = alloc_card(
        &mut tx,
        setup.far,
        far_hand,
        CardKind::Tao,
        97_103,
        Suit::Heart,
        8,
    );
    let state = tx.commit().state;
    let skill = state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.def.raw() == SHENSHI)
        .map(|skill| skill.id)
        .unwrap();
    let mut tx = Transaction::begin(state);
    tx.state_mut().skills.get_mut(skill).unwrap().state_blob = b"SS1\x00".to_vec();
    let state = tx.commit().state;
    let (state, result) = {
        let revision = state.revision;
        let (state, result) = apply_command(
            state,
            Command {
                actor: setup.owner,
                expected_revision: ExpectedRevision(revision),
                idempotency_key: IdempotencyKey(9_010),
                kind: CommandKind::ActivateSkill {
                    skill,
                    targets: Vec::new(),
                    card_targets: Vec::new(),
                },
            },
        );
        (state, result)
    };
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "activate rejected: {:?}",
        result.result
    );
    assert!(state.has_pending_choice(), "审时 give prompt");
    let state = resolve(state, setup.owner, vec![gift.raw()], false, 9_011);
    assert!(state.has_pending_choice(), "审时 target prompt");
    // 手牌最多者：far(1) 唯一。
    let hp_before = state.players.get(setup.far).unwrap().hp;
    let state = resolve(state, setup.owner, vec![setup.far.raw()], false, 9_012);
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(setup.far).unwrap().hp, hp_before - 1);
    assert!(
        state.hand_cards(setup.far).contains(&gift),
        "gift moved to the target"
    );
    let _ = f1;
}

/// 审时（阴分支）：受伤后交来源一张牌，来源回合结束时追踪牌在手则摸至四张。
#[test]
fn shenshi_yin_tracks_gift_until_source_turn_end() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    tx.state_mut().turn.current = setup.other;
    tx.state_mut().turn.phase = Phase::Play;
    let owner_hand = hand_zone(&tx, setup.owner);
    let gift = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Sha,
        97_104,
        Suit::Club,
        9,
    );
    let state = tx.commit().state;
    let skill = state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.def.raw() == SHENSHI)
        .map(|skill| skill.id)
        .unwrap();
    let mut tx = Transaction::begin(state);
    tx.state_mut().skills.get_mut(skill).unwrap().state_blob = b"SS1\x01".to_vec();
    let state = tx.commit().state;
    // 受伤事件（来源 other 对 owner 造成伤害）。
    let state = dispatch(
        state,
        DomainEvent::DamageDealt {
            source: setup.other,
            target: setup.owner,
            amount: 1,
            nature: 0,
            is_lose_hp: false,
        },
    );
    assert!(state.has_pending_choice(), "审时 confirm");
    let state = resolve(state, setup.owner, vec![1], false, 9_020);
    assert!(state.has_pending_choice(), "审时 give prompt");
    let state = resolve(state, setup.owner, vec![gift.raw()], false, 9_021);
    assert!(!state.has_pending_choice());
    assert!(
        state.hand_cards(setup.other).contains(&gift),
        "gift moved to the source"
    );
    // 来源回合结束：追踪牌仍在来源手牌 → 持有者摸至四张。
    let state = dispatch(
        state,
        DomainEvent::PhaseEnded {
            current: setup.other,
            phase: engine_api::TurnPhaseTag::End,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other: false,
            current_dealt_damage_other: false,
            phase_targeted_players: 0,
            phase_hp_changed_players: 0,
        },
    );
    assert!(state.has_pending_choice(), "审时 turn-end confirm");
    let state = resolve(state, setup.owner, vec![1], false, 9_022);
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(setup.owner), 4, "drew to four");
}
