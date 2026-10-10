//! Golden coverage for 征荣/鸿举/清俭（shenhua 毌丘俭，ABI 545-547）.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const ZHENGRONG: u32 = 3_942_454_790;
const HONGJU: u32 = 1_729_861_379;
const QINGJIAN: u32 = 3_345_707_863;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/guanqiujian.skill");

struct Setup {
    tx: Transaction,
    owner: PlayerId,
    other: PlayerId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("gqj-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let other = seats[1];
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
        skill_def: SkillDefId::new(ZHENGRONG),
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(HONGJU),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    Setup { tx, owner, other }
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

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
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
    )
}

fn skill_instance(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == def)
        .map(|skill| skill.id)
        .unwrap()
}

fn activate(
    state: GameState,
    owner: PlayerId,
    def: u32,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let skill = skill_instance(&state, owner, def);
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: Vec::new(),
                card_targets: Vec::new(),
            },
        },
    )
}

fn option_tokens(result: &engine_core::command::PipelineResult) -> Vec<u32> {
    result
        .choice
        .as_ref()
        .map(|choice| choice.options.iter().map(|option| option.token).collect())
        .unwrap_or_default()
}

/// Run one full 征荣 flow: the owner uses a Sha on `other` (eligible when the target's
/// hand count is at least the owner's), confirms, picks the target and one card.
/// Returns the committed state and the taken card.
fn run_zhengrong(
    state: GameState,
    owner: PlayerId,
    other: PlayerId,
    card: CardId,
    key_base: u64,
) -> (GameState, CardId) {
    let mut tx = Transaction::begin(state);
    let fired = dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::CardUsed {
            player: owner,
            card,
            targets: vec![other],
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => {
            run_effects(&mut tx, effects).unwrap();
        }
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("dispatch error: {e:?}"),
    }
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice(), "optional 征荣 confirm");
    let (state, confirmed) = resolve(state, owner, vec![1], key_base);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    let target_tokens = option_tokens(&confirmed);
    assert_eq!(target_tokens, [other.raw()]);
    let (state, picked) = resolve(state, owner, vec![other.raw()], key_base + 1);
    assert!(matches!(picked.result, CommandResult::Accepted { .. }));
    let card_tokens = option_tokens(&picked);
    let taken = CardId::new(card_tokens[0]);
    let (state, done) = resolve(state, owner, vec![taken.raw()], key_base + 2);
    assert!(matches!(done.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    (state, taken)
}

fn rong_pile_cards(state: &GameState, owner: PlayerId) -> Vec<CardId> {
    state
        .skill_piles
        .iter()
        .find(|pile| pile.owner == owner && pile.skill_def == ZHENGRONG)
        .map(|pile| {
            state
                .zones
                .get(pile.zone)
                .map(|zone| zone.cards.clone())
                .unwrap_or_default()
        })
        .unwrap_or_default()
}

#[test]
fn package_exposes_guanqiujian_contract() {
    let compiled = content_compiler::compile_source_named("gqj", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 547);
    let zhengrong = compiled
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == ZHENGRONG)
        .unwrap();
    assert_eq!(zhengrong.event, "card_targeted");
    assert_eq!(
        zhengrong.filter,
        "owner_source_first_damage_card_target_hand_gte_owner"
    );
    assert!(zhengrong.optional);
    let hongju = compiled
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == HONGJU)
        .unwrap();
    assert_eq!(hongju.event, "phase_prepare");
    assert_eq!(
        hongju.filter,
        "owner_phase_begin_skill_pile_gte_three_3942454790"
    );
    let qingjian = compiled
        .active_index
        .iter()
        .find(|a| a.skill_def_id == QINGJIAN)
        .unwrap();
    assert_eq!(qingjian.window, "play_phase");
    assert_eq!(qingjian.uses_per_turn, Some(1));
    assert_eq!(compiled.skill_index.len(), 3);
    assert_eq!(compiled.trigger_index.len(), 2);
    assert_eq!(compiled.active_index.len(), 1);
}

#[test]
fn zhengrong_stores_one_target_card_as_rong() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    let owner_hand = tx.state().zones.find(ZoneKind::Hand, setup.owner).unwrap();
    let sha = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Sha,
        96_400,
        Suit::Spade,
        8,
    );
    let other_hand = tx.state().zones.find(ZoneKind::Hand, setup.other).unwrap();
    // Target has 2 hand cards >= owner's 1.
    let victim = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Tao,
        96_401,
        Suit::Heart,
        1,
    );
    let _spare = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Shan,
        96_402,
        Suit::Heart,
        2,
    );
    let (state, taken) = run_zhengrong(tx.commit().state, setup.owner, setup.other, sha, 1);
    assert_eq!(taken, victim, "owner took the victim card");
    let pile = rong_pile_cards(&state, setup.owner);
    assert_eq!(pile, vec![victim], "taken card stored as 荣");
}

#[test]
fn hongju_awakens_with_three_rong() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    let owner_hand = tx.state().zones.find(ZoneKind::Hand, setup.owner).unwrap();
    let other_hand = tx.state().zones.find(ZoneKind::Hand, setup.other).unwrap();
    // Build three 荣 through three 征荣 flows; the target keeps at least two hand cards.
    let mut state = tx.commit().state;
    for i in 0..3u64 {
        let mut tx = Transaction::begin(state);
        let sha = alloc_card(
            &mut tx,
            setup.owner,
            owner_hand,
            CardKind::Sha,
            96_410 + i as u32,
            Suit::Spade,
            8,
        );
        // Grant the target enough cards to keep hand >= owner each round.
        let _ = alloc_card(
            &mut tx,
            setup.other,
            other_hand,
            CardKind::Tao,
            96_420 + i as u32,
            Suit::Heart,
            1,
        );
        let _ = alloc_card(
            &mut tx,
            setup.other,
            other_hand,
            CardKind::Shan,
            96_421 + i as u32,
            Suit::Heart,
            2,
        );
        state = run_zhengrong(tx.commit().state, setup.owner, setup.other, sha, 10 + i * 4).0;
    }
    assert_eq!(rong_pile_cards(&state, setup.owner).len(), 3);
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = setup.owner;
    let mut dispatch_tx = Transaction::begin(tx.commit().state);
    let fired = dispatch_for_events(
        &mut dispatch_tx,
        &[engine_api::DomainEvent::TurnStateChanged {
            current: setup.owner,
            phase: engine_api::TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => {
            assert_eq!(
                run_effects(&mut dispatch_tx, effects),
                Err(EffectError::ChoiceRequested),
                "鸿举 swap-or-skip choice"
            );
        }
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("dispatch error: {e:?}"),
    }
    let mut state = dispatch_tx.commit().state;
    // Skip the swap: awaken directly (max hp 5 -> 4, gain 清俭).
    let (state, skipped) = resolve(state, setup.owner, vec![0], 1);
    assert!(matches!(skipped.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.players.get(setup.owner).unwrap().max_hp,
        4,
        "max hp reduced"
    );
    assert!(
        state
            .skills
            .sorted_live()
            .iter()
            .any(|skill| skill.owner == setup.owner && skill.def.raw() == QINGJIAN),
        "清俭 granted"
    );
    assert_eq!(
        rong_pile_cards(&state, setup.owner).len(),
        3,
        "荣 stay in the pile"
    );
    assert!(!state.has_pending_choice());
}

#[test]
fn qingjian_exchanges_rong_and_discards_target_equipment() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    let owner_hand = tx.state().zones.find(ZoneKind::Hand, setup.owner).unwrap();
    let other_hand = tx.state().zones.find(ZoneKind::Hand, setup.other).unwrap();
    // Seed one 荣 via the real flow.
    let sha = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Sha,
        96_430,
        Suit::Spade,
        8,
    );
    let _ = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Tao,
        96_431,
        Suit::Heart,
        1,
    );
    let (state, _) = run_zhengrong(tx.commit().state, setup.owner, setup.other, sha, 1);
    assert_eq!(rong_pile_cards(&state, setup.owner).len(), 1);
    // Grant 清俭 for the active test.
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: setup.owner,
            skill_def: SkillDefId::new(QINGJIAN),
        }],
    )
    .unwrap();
    // Give the owner a hand card and the target an equipped weapon.
    let owner_hand = tx.state().zones.find(ZoneKind::Hand, setup.owner).unwrap();
    let hand_card = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Tao,
        96_432,
        Suit::Heart,
        2,
    );
    let other_equip = tx.state().zones.find(ZoneKind::Equip, setup.other).unwrap();
    let weapon = alloc_card(
        &mut tx,
        setup.other,
        other_equip,
        CardKind::Weapon,
        96_433,
        Suit::Spade,
        6,
    );
    let state = tx.commit().state;
    let (state, activated) = activate(state, setup.owner, QINGJIAN, 2);
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    let rong = rong_pile_cards(&state, setup.owner)[0];
    assert_eq!(option_tokens(&activated), [rong.raw()]);
    let (state, r) = resolve(state, setup.owner, vec![rong.raw()], 3);
    assert!(matches!(r.result, CommandResult::Accepted { .. }));
    let (state, r) = resolve(state, setup.owner, vec![hand_card.raw()], 4);
    assert!(matches!(r.result, CommandResult::Accepted { .. }));
    let (state, r) = resolve(state, setup.owner, vec![setup.other.raw()], 5);
    assert!(matches!(r.result, CommandResult::Accepted { .. }));
    let (state, r) = resolve(state, setup.owner, vec![weapon.raw()], 6);
    assert!(matches!(r.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    // 荣 moved to the owner's hand; the hand card and the weapon are discarded.
    assert!(state.hand_cards(setup.owner).contains(&rong), "荣 gained");
    let discard_zone = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(hand_card).unwrap().zone, discard_zone);
    assert_eq!(state.cards.get(weapon).unwrap().zone, discard_zone);
}

#[test]
fn zhengrong_filter_skips_non_damage_cards() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    let owner_hand = tx.state().zones.find(ZoneKind::Hand, setup.owner).unwrap();
    // A Tao is not a damage-tagged card: the filter must not fire.
    let tao = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Tao,
        96_440,
        Suit::Heart,
        8,
    );
    let other_hand = tx.state().zones.find(ZoneKind::Hand, setup.other).unwrap();
    let _ = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Tao,
        96_441,
        Suit::Heart,
        1,
    );
    let _ = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Shan,
        96_442,
        Suit::Heart,
        2,
    );
    let mut tx = tx;
    let fired = dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::CardUsed {
            player: setup.owner,
            card: tao,
            targets: vec![setup.other],
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => assert!(effects.is_empty(), "征荣 must not fire for a Tao"),
        Err(EffectError::ChoiceRequested) => panic!("征荣 must not open a choice here"),
        Err(_) => {}
    }
    let state = tx.commit().state;
    assert!(!state.has_pending_choice());
}
