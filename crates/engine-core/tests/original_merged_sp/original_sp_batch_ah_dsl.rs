//! Golden for sp batch-AH DSL rewrites:
//! 困奋 (kunfen) end-phase lose 1 HP then draw 2;
//! 礼下 (lixia) other's end phase outside owner's range → self/event-player draw 1
//!   + rebiluan2 distance mark -1;
//! 妆梳·锦囊 (zhuangshu_trick) judge phase → skip judge or discard phase;
//! 骗南 (olpianan) game start / discard-phase end → discard all non-Shan hand, draw to HP.

use engine_api::{
    CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const KUNFEN: u32 = 2_673_970_052;
const LIXIA: u32 = 2_791_840_336;
const REBILUAN2: u32 = 3_075_740_841;
const ZHUANGSHU_TRICK: u32 = 2_226_439_241;
const OLPIANAN: u32 = 3_870_010_283;
const OLLINJIE: u32 = 4_117_878_839;
const ZHUANGSHU_BASIC: u32 = 3_108_569_468;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/sp/sp.npack.json");

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

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
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
                cancelled,
            },
        },
    )
}

fn inject_hand(state: &mut GameState, player: PlayerId, kind: CardKind, tag: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(91_700 + tag),
        kind,
        suit: engine_api::Suit::Club,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: true,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn equip_mount(state: &mut GameState, player: PlayerId, kind: CardKind) {
    let zone = state.zones.find(ZoneKind::Equip, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(9_999),
        kind,
        suit: engine_api::Suit::Spade,
        rank: 1,
        weapon_range: 0,
        zone,
        owner: player,
        face_up: true,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
}

fn mark_of(state: &GameState, player: PlayerId, mark: u32) -> Option<i32> {
    state
        .players
        .get(player)
        .unwrap()
        .marks
        .iter()
        .find(|(id, _)| *id == mark)
        .map(|(_, count)| *count)
}

fn has_skill(state: &GameState, owner: PlayerId, def: u32) -> bool {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .any(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == def)
        })
}

#[test]
fn package_publishes_batch_ah_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    let mut seen = std::collections::BTreeSet::new();
    for trigger in &package.trigger_index {
        if matches!(
            trigger.skill_def_id,
            KUNFEN | LIXIA | ZHUANGSHU_TRICK | OLPIANAN | OLLINJIE | ZHUANGSHU_BASIC
        ) {
            seen.insert((
                trigger.skill_def_id,
                trigger.event.as_str(),
                trigger.filter.as_str(),
                trigger.optional,
            ));
        }
    }
    assert!(seen.contains(&(KUNFEN, "phase_end", "owner_is_target", false)));
    assert!(seen.contains(&(
        LIXIA,
        "phase_end",
        "owner_alive_other_phase_end_player_outside_owner_range",
        true
    )));
    assert!(seen.contains(&(ZHUANGSHU_TRICK, "phase_judge", "owner_is_target", true)));
    assert!(seen.contains(&(OLPIANAN, "game_started", "owner_alive", false)));
    assert!(seen.contains(&(OLPIANAN, "phase_discard_end", "owner_is_target", false)));
    assert!(seen.contains(&(OLLINJIE, "damage", "owner_is_target", false)));
    assert!(seen.contains(&(ZHUANGSHU_BASIC, "damage_before", "owner_is_target", true)));
}

#[test]
fn kunfen_end_phase_loses_one_hp_then_draws_two() {
    let (state, owner) = started(2_026_081_500);
    let hp_before = state.players.get(owner).unwrap().hp;
    let hand_before = state.hand_count(owner);
    let state = with_skill(state, owner, KUNFEN);

    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    let state = tx.commit().state;
    assert_eq!(state.players.get(owner).unwrap().hp, hp_before - 1);
    assert_eq!(state.hand_count(owner), hand_before + 2);
}

#[test]
fn lixia_outside_range_fires_and_marks_distance_minus_one() {
    let (mut state, owner) = started(2_026_081_501);
    let other = state
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    // A defensive horse on the owner makes distance(other→owner) = 2 > other's
    // attack range 1 → the event player is not in the owner's range.
    equip_mount(&mut state, owner, CardKind::HorseDef);
    let state = with_skill(state, owner, LIXIA);
    let before = state.hand_count(owner);

    let mut tx = Transaction::begin(state);
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: other,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();

    // Confirm the optional trigger, then branch 1: self draws 1.
    let (state, confirmed) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    let (state, resolved) = resolve(state, owner, vec![1], false, 2);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), before + 1);
    assert!(has_skill(&state, owner, REBILUAN2));
    assert_eq!(mark_of(&state, owner, REBILUAN2), Some(-1));
}

#[test]
fn lixia_branch_two_draws_event_player_one() {
    let (mut state, owner) = started(2_026_081_502);
    let other = state
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    equip_mount(&mut state, owner, CardKind::HorseDef);
    let state = with_skill(state, owner, LIXIA);
    let other_before = state.hand_count(other);
    let owner_before = state.hand_count(owner);

    let mut tx = Transaction::begin(state);
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: other,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    let tx = tx.commit();
    let (state, confirmed) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    // Branch 2: the event player draws 1.
    let (state, resolved) = resolve(state, owner, vec![2], false, 2);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(other), other_before + 1);
    assert_eq!(state.hand_count(owner), owner_before);
    assert_eq!(mark_of(&state, owner, REBILUAN2), Some(-1));
}

#[test]
fn zhuangshu_trick_branch_one_skips_next_judge_phase() {
    let (state, owner) = started(2_026_081_503);
    let state = with_skill(state, owner, ZHUANGSHU_TRICK);
    assert!(state.pending_judge_skips.is_empty());

    let mut tx = Transaction::begin(state);
    let opened = run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Judge.to_u8(),
        }],
    );
    // Optional trigger opens during the Judge transition.
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    let tx = tx.commit();
    let (state, confirmed) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    // OptionalTrigger → Custom choice (branch 1 = skip judge).
    let (state, resolved) = resolve(state, owner, vec![1], false, 2);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert_eq!(state.pending_judge_skips, [owner]);
}

#[test]
fn zhuangshu_trick_branch_two_skips_next_discard_phase() {
    let (state, owner) = started(2_026_081_504);
    let state = with_skill(state, owner, ZHUANGSHU_TRICK);

    let mut tx = Transaction::begin(state);
    let opened = run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Judge.to_u8(),
        }],
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    let tx = tx.commit();
    let (state, confirmed) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    // Branch 2 = skip discard phase.
    let (state, resolved) = resolve(state, owner, vec![2], false, 2);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert!(state.pending_judge_skips.is_empty());
    assert_eq!(state.turn.skip_discard(), true);
}

#[test]
fn olpianan_game_started_discards_all_then_draws_to_hp() {
    let (state, owner) = started(2_026_081_505);
    let hp = state.players.get(owner).unwrap().hp;
    assert!(state.hand_count(owner) as i32 >= hp);
    let state = with_skill(state, owner, OLPIANAN);

    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::GameStarted { player_count: 2 }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    let state = tx.commit().state;
    // All hand discarded, then drawn back up to hp.
    assert_eq!(state.hand_count(owner), hp as u32);
}

#[test]
fn olpianan_discard_phase_end_discards_and_draws_to_hp() {
    let (state, owner) = started(2_026_081_506);
    let hp = state.players.get(owner).unwrap().hp;
    let state = with_skill(state, owner, OLPIANAN);
    // Add extra hand cards so the discard is observable.
    let mut state = state;
    inject_hand(&mut state, owner, CardKind::Tao, 1);
    inject_hand(&mut state, owner, CardKind::Shan, 2);
    let hand_before = state.hand_count(owner);
    assert!(hand_before > hp as u32);

    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current: owner,
            phase: TurnPhaseTag::Discard,
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
    let state = tx.commit().state;
    assert_eq!(state.hand_count(owner), hp as u32);
}

#[test]
fn ollinjie_source_discards_sha_then_owner_draws() {
    let (state, owner) = started(2_026_081_510);
    let source = state
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let state = with_skill(state, owner, OLLINJIE);
    let owner_before = state.hand_count(owner);
    // Source is forced to discard a hand card; inject a Sha into its hand.
    let mut state = state;
    let sha = inject_hand(&mut state, source, CardKind::Sha, 1);
    let source_before = state.hand_count(source);

    let mut tx = Transaction::begin(state);
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::DamageDealt {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    // Forced trigger returns the effect requests; running them opens the source's
    // hand-card discard choice.
    let opened = run_effects(&mut tx, opened);
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::ResolveOllinjie
    );
    let tx = tx.commit();
    let (state, resolved) = resolve(tx.state, source, vec![sha.raw()], false, 1);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(source), source_before - 1);
    // Discarded card was a Sha → owner draws 1.
    assert_eq!(state.hand_count(owner), owner_before + 1);
}

#[test]
fn ollinjie_source_discards_non_sha_no_draw() {
    let (state, owner) = started(2_026_081_511);
    let source = state
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let state = with_skill(state, owner, OLLINJIE);
    let owner_before = state.hand_count(owner);
    let mut state = state;
    let tao = inject_hand(&mut state, source, CardKind::Tao, 2);
    let source_before = state.hand_count(source);

    let mut tx = Transaction::begin(state);
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::DamageDealt {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    let opened = run_effects(&mut tx, opened);
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    let tx = tx.commit();
    let (state, resolved) = resolve(tx.state, source, vec![tao.raw()], false, 1);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(source), source_before - 1);
    // Discarded card is not a Sha → owner does not draw.
    assert_eq!(state.hand_count(owner), owner_before);
}

#[test]
fn zhuangshu_basic_discards_one_and_prevents_damage() {
    let (state, owner) = started(2_026_081_512);
    let state = with_skill(state, owner, ZHUANGSHU_BASIC);
    let hp_before = state.players.get(owner).unwrap().hp;
    let hand_before = state.hand_count(owner);
    let discardable = state.hand_cards(owner);
    assert!(!discardable.is_empty(), "owner has hand cards");
    let cost = discardable[0];

    let mut tx = Transaction::begin(state);
    let opened = run_effects(
        &mut tx,
        vec![EffectRequest::CommitDamage {
            damage_id: 0,
            source: PlayerId::NONE,
            target: owner,
            amount: 1,
            base_amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
            card: engine_api::CardId::NONE,
            card_kind: CardKind::Sha.tag(),
            card_flags: 0,
        }],
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let (state, confirmed) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    // Confirm the discard choice: pick the first discardable hand card.
    let (state, discarded) = resolve(state, owner, vec![cost.raw()], false, 2);
    assert!(matches!(discarded.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand_before - 1);
    assert_eq!(state.players.get(owner).unwrap().hp, hp_before);
}
