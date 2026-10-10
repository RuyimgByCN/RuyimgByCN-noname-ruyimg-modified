//! Golden coverage for 飞军/兵略（shenhua 王平，ABI 526）.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const NZRY_FEIJUN: u32 = 2_410_819_466;
const NZRY_BINGLVE: u32 = 3_525_767_842;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/nzry_feijun.skill");

struct Setup {
    tx: Transaction,
    owner: PlayerId,
    hand_more: PlayerId,
    equip_more: PlayerId,
    cost_card: CardId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("feijun-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let hand_more = seats[1];
    let equip_more = seats[2];
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
        skill_def: SkillDefId::new(NZRY_FEIJUN),
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(NZRY_BINGLVE),
    });
    run_effects(&mut tx, effects).unwrap();
    // The owner keeps one cost card; the hand-more target gets three hand cards.
    let owner_hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let cost = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(96_140),
        kind: CardKind::Sha,
        suit: Suit::Spade,
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
        .push(cost);
    let target_hand = tx.state().zones.find(ZoneKind::Hand, hand_more).unwrap();
    let give_cards: Vec<CardId> = (0..3)
        .map(|i| {
            let card = tx.state_mut().cards.alloc(Card {
                id: CardId::NONE,
                def: CardDefId::new(96_150 + i),
                kind: CardKind::Tao,
                suit: Suit::Heart,
                rank: 6,
                weapon_range: 0,
                zone: target_hand,
                owner: hand_more,
                face_up: false,
            });
            tx.state_mut()
                .zones
                .get_mut(target_hand)
                .unwrap()
                .cards
                .push(card);
            card
        })
        .collect();
    // The equip-more target gets two weapons equipped.
    let equip_zone = tx.state().zones.find(ZoneKind::Equip, equip_more).unwrap();
    let mut equips = Vec::new();
    for i in 0..2 {
        let card = tx.state_mut().cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(96_160 + i),
            kind: CardKind::Weapon,
            suit: Suit::Club,
            rank: 7,
            weapon_range: 2,
            zone: equip_zone,
            owner: equip_more,
            face_up: false,
        });
        tx.state_mut()
            .zones
            .get_mut(equip_zone)
            .unwrap()
            .cards
            .push(card);
        equips.push(card);
    }
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    Setup {
        tx,
        owner,
        hand_more,
        equip_more,
        cost_card: cost,
    }
}

fn activate(
    state: GameState,
    owner: PlayerId,
    skill: SkillId,
    cards: Vec<CardId>,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![],
                card_targets: cards,
            },
        },
    )
}

fn resolve_choice(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
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

fn fire_play_end(tx: Transaction, current: PlayerId) -> Transaction {
    let mut tx = tx;
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current,
            phase: TurnPhaseTag::Play,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other: false,
            current_dealt_damage_other: false,
            phase_hp_changed_players: 0,
            phase_targeted_players: 0,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => match run_effects(&mut tx, effects) {
            Ok(_) => tx,
            Err(EffectError::ChoiceRequested) => tx,
            Err(e) => panic!("effect error: {e:?}"),
        },
        Err(EffectError::ChoiceRequested) => tx,
        Err(e) => panic!("dispatch error: {e:?}"),
    }
}

#[test]
fn package_exposes_feijun_binglve_contract() {
    let compiled = content_compiler::compile_source_named("feijun", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 526);
    let active = compiled
        .active_index
        .iter()
        .find(|active| active.skill_def_id == NZRY_FEIJUN)
        .unwrap();
    assert_eq!(active.window, "play_phase");
    assert_eq!(active.uses_per_turn, Some(1));
    assert_eq!(
        active.selection.as_ref().map(|s| s.zone.as_str()),
        Some("hand_equip")
    );
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == NZRY_BINGLVE)
        .unwrap();
    assert_eq!(trigger.event, "phase_play_end");
    assert_eq!(trigger.filter, "owner_is_target_feijun_new_target");
}

#[test]
fn hand_branch_gives_card_then_binglve_draws_two_at_phase_end() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    // Find the owner's active skill instance.
    let skill = tx
        .state()
        .players
        .get(setup.owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == NZRY_FEIJUN)
        .unwrap();
    let (state, activated) = activate(
        tx.commit().state,
        setup.owner,
        skill,
        vec![setup.cost_card],
        1,
    );
    assert!(
        matches!(activated.result, CommandResult::Accepted { .. }),
        "activation must be accepted: {:?}",
        activated.result
    );
    let mut state = resolve_choice(state, setup.owner, vec![0], 2);
    state = resolve_choice(state, setup.owner, vec![setup.hand_more.raw()], 3);
    let given = state.hand_cards(setup.hand_more)[0];
    state = resolve_choice(state, setup.hand_more, vec![given.raw()], 4);

    assert_eq!(
        state.hand_count(setup.owner),
        1,
        "cost discarded, one card gained from the target"
    );
    assert!(
        state.hand_cards(setup.owner).contains(&given),
        "the hand-more target gave a card to the owner"
    );
    // The play phase ends: 兵略 draws two for the first-time target.
    let mut tx = Transaction::begin(state);
    let tx = fire_play_end(tx, setup.owner);
    let state = tx.commit().state;
    assert_eq!(
        state.hand_count(setup.owner),
        3,
        "given card + two from 兵略"
    );
}

#[test]
fn equip_branch_discards_one_equipment() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    let skill = tx
        .state()
        .players
        .get(setup.owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == NZRY_FEIJUN)
        .unwrap();
    let (state, activated) = activate(
        tx.commit().state,
        setup.owner,
        skill,
        vec![setup.cost_card],
        1,
    );
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    // Both branches are available; choose the equipment branch.
    let mut state = resolve_choice(state, setup.owner, vec![1], 2);
    state = resolve_choice(state, setup.owner, vec![setup.equip_more.raw()], 3);
    let equip = state
        .zones
        .find(ZoneKind::Equip, setup.equip_more)
        .and_then(|zone| state.zones.get(zone))
        .map(|zone| zone.cards[0])
        .unwrap();
    let equip_zone = state.zones.find(ZoneKind::Equip, setup.equip_more).unwrap();
    let equip_count_before = state.zones.get(equip_zone).unwrap().cards.len();
    state = resolve_choice(state, setup.equip_more, vec![equip.raw()], 4);
    assert_eq!(
        state.zones.get(equip_zone).unwrap().cards.len(),
        equip_count_before - 1,
        "the equip-more target discarded one equipment"
    );
    assert_eq!(state.hand_count(setup.owner), 0, "cost discarded");
}

#[test]
fn repeated_target_does_not_rearm_binglve() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    let skill = tx
        .state()
        .players
        .get(setup.owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == NZRY_FEIJUN)
        .unwrap();
    // First use against the hand-more target.
    let (state, _) = activate(
        tx.commit().state,
        setup.owner,
        skill,
        vec![setup.cost_card],
        1,
    );
    let mut state = resolve_choice(state, setup.owner, vec![0], 2);
    state = resolve_choice(state, setup.owner, vec![setup.hand_more.raw()], 3);
    let given = state.hand_cards(setup.hand_more)[0];
    state = resolve_choice(state, setup.hand_more, vec![given.raw()], 4);
    let mut tx = Transaction::begin(state);
    let tx = fire_play_end(tx, setup.owner);
    let mut state = tx.commit().state;
    assert_eq!(state.hand_count(setup.owner), 3);
    // Next turn: the per-turn flag resets but the permanent target mark stays.
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.clear_per_player_card_counts();
    tx.state_mut().skills.get_mut(skill).unwrap().use_count = 0;
    let hand = tx.state().zones.find(ZoneKind::Hand, setup.owner).unwrap();
    let cost = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(96_170),
        kind: CardKind::Shan,
        suit: Suit::Heart,
        rank: 8,
        weapon_range: 0,
        zone: hand,
        owner: setup.owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(cost);
    // The hand-more target gets refilled so the hand branch is available again.
    let target_hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, setup.hand_more)
        .unwrap();
    for i in 0..3 {
        let card = tx.state_mut().cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(96_180 + i),
            kind: CardKind::Tao,
            suit: Suit::Heart,
            rank: 6,
            weapon_range: 0,
            zone: target_hand,
            owner: setup.hand_more,
            face_up: false,
        });
        tx.state_mut()
            .zones
            .get_mut(target_hand)
            .unwrap()
            .cards
            .push(card);
    }
    tx.state_mut().turn.phase = Phase::Play;
    state = tx.commit().state;
    // Same target again: the mark exists, so no new 兵略 flag.
    let (state, _) = activate(state, setup.owner, skill, vec![cost], 5);
    let mut state = resolve_choice(state, setup.owner, vec![0], 6);
    state = resolve_choice(state, setup.owner, vec![setup.hand_more.raw()], 7);
    let given2 = state.hand_cards(setup.hand_more)[0];
    state = resolve_choice(state, setup.hand_more, vec![given2.raw()], 8);
    assert_eq!(
        state.hand_count(setup.owner),
        4,
        "cost spent, one card gained"
    );
    // Play phase end: 兵略 must NOT fire again (same target, already marked).
    let mut tx = Transaction::begin(state);
    let tx = fire_play_end(tx, setup.owner);
    let state = tx.commit().state;
    assert_eq!(state.hand_count(setup.owner), 4, "no second 兵略 draw");
}
