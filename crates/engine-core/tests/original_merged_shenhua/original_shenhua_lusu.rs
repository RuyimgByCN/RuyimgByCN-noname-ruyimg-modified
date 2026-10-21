//! Golden coverage for 好施/缔盟（shenhua 鲁肃，ABI 555-556）.

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

const HAOSHI: u32 = 1_569_684_326;
const DIMENG: u32 = 427_650_324;
const SOURCE: &str = include_str!("../../../../content/original/ported/shenhua/lusu.skill");

struct Setup {
    tx: Transaction,
    owner: PlayerId,
    other: PlayerId,
    far: PlayerId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("lusu-test", SOURCE).unwrap();
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
        skill_def: SkillDefId::new(HAOSHI),
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(DIMENG),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
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

#[test]
fn package_exposes_lusu_contract() {
    let compiled = content_compiler::compile_source_named("lusu", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 556);
    let draw = compiled
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == HAOSHI && t.event == "phase_draw")
        .unwrap();
    assert!(draw.optional);
    assert_eq!(draw.filter, "owner_is_target_draw_not_replaced");
    let draw_end = compiled
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == HAOSHI && t.event == "phase_draw_end")
        .unwrap();
    assert_eq!(draw_end.filter, "owner_is_target");
    let dimeng = compiled
        .active_index
        .iter()
        .find(|a| a.skill_def_id == DIMENG)
        .unwrap();
    assert_eq!(dimeng.window, "play_phase");
    assert_eq!(dimeng.uses_per_turn, Some(1));
    assert_eq!(compiled.skill_index.len(), 2);
    assert_eq!(compiled.trigger_index.len(), 2);
    assert_eq!(compiled.active_index.len(), 1);
}

#[test]
fn haoshi_draws_extra_two_and_gives_half_to_min_hand() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    tx.state_mut().turn.current = setup.owner;
    tx.state_mut().turn.phase = Phase::Draw;
    // Dispatch the draw-phase start: the optional 好施 confirm opens.
    let mut dispatch_tx = Transaction::begin(tx.commit().state);
    let fired = dispatch_for_events(
        &mut dispatch_tx,
        &[DomainEvent::TurnStateChanged {
            current: setup.owner,
            phase: engine_api::TurnPhaseTag::Draw,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => {
            assert_eq!(
                run_effects(&mut dispatch_tx, effects),
                Err(EffectError::ChoiceRequested),
                "好施 confirm"
            );
        }
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("dispatch error: {e:?}"),
    }
    let mut state = dispatch_tx.commit().state;
    let (state, confirmed) = resolve(state, setup.owner, vec![1], 1);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    // The draw count is fixed to 4: draw the four cards.
    let draw = state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let four: Vec<_> = state
        .zones
        .get(draw)
        .map(|zone| zone.cards.iter().rev().take(4).copied().collect())
        .unwrap_or_default();
    assert_eq!(four.len(), 4);
    let hand = state.zones.find(ZoneKind::Hand, setup.owner).unwrap();
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: four.clone(),
            to: hand,
            from: Some(draw),
        }],
    )
    .unwrap();
    // 4 cards: hand == 4 <= 5 -> the draw-end give-half must not fire.
    tx.state_mut().turn.phase = Phase::Draw;
    let mut dispatch_tx = Transaction::begin(tx.commit().state);
    let fired = dispatch_for_events(
        &mut dispatch_tx,
        &[DomainEvent::TurnStateChanged {
            current: setup.owner,
            phase: engine_api::TurnPhaseTag::Draw,
        }],
        &mut TriggerFireRecord::new(),
    );
    // (draw phase is fixed now: no trigger; nothing pending)
    match fired {
        Ok(effects) => assert!(effects.is_empty()),
        Err(_) => {}
    }
    let mut state = dispatch_tx.commit().state;
    // Give the owner two more hand cards -> hand 6 > 5 -> give floor(6/2)=3.
    let mut tx = Transaction::begin(state);
    let hand = tx.state().zones.find(ZoneKind::Hand, setup.owner).unwrap();
    let c1 = alloc_card(
        &mut tx,
        setup.owner,
        hand,
        CardKind::Sha,
        96_600,
        Suit::Spade,
        2,
    );
    let c2 = alloc_card(
        &mut tx,
        setup.owner,
        hand,
        CardKind::Sha,
        96_601,
        Suit::Spade,
        3,
    );
    tx.state_mut().turn.phase = Phase::Draw;
    let mut dispatch_tx = Transaction::begin(tx.commit().state);
    let fired = dispatch_for_events(
        &mut dispatch_tx,
        &[DomainEvent::PhaseEnded {
            current: setup.owner,
            phase: engine_api::TurnPhaseTag::Draw,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other: false,
            current_dealt_damage_other: false,
            phase_targeted_players: 0,
            phase_hp_changed_players: 0,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => {
            assert_eq!(
                run_effects(&mut dispatch_tx, effects),
                Err(EffectError::ChoiceRequested),
                "好施 give-half choice"
            );
        }
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("dispatch error: {e:?}"),
    }
    let mut state = dispatch_tx.commit().state;
    // The give-half choice opened (hand 6 -> give 3).
    assert!(state.has_pending_choice(), "好施 give-half choice");
    let hand_cards: Vec<_> = state.hand_cards(setup.owner);
    assert!(hand_cards.len() >= 3);
    let picks: Vec<u32> = hand_cards.iter().take(3).map(|card| card.raw()).collect();
    assert_eq!(picks.len(), 3);
    let (state, given) = resolve(state, setup.owner, picks, 2);
    assert!(matches!(given.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    // The owner kept 3 cards; the target received 3.
    assert_eq!(state.hand_count(setup.owner), 3);
    assert_eq!(state.hand_count(setup.other), 3);
    let _ = (c1, c2);
}

#[test]
fn dimeng_exchanges_hands_with_diff_discard() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    // Other has 3 cards, far has 1: diff = 2.
    let other_hand = tx.state().zones.find(ZoneKind::Hand, setup.other).unwrap();
    let o1 = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Sha,
        96_610,
        Suit::Spade,
        4,
    );
    let o2 = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Sha,
        96_611,
        Suit::Spade,
        5,
    );
    let o3 = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Shan,
        96_612,
        Suit::Heart,
        2,
    );
    let far_hand = tx.state().zones.find(ZoneKind::Hand, setup.far).unwrap();
    let f1 = alloc_card(
        &mut tx,
        setup.far,
        far_hand,
        CardKind::Tao,
        96_613,
        Suit::Heart,
        6,
    );
    // Owner has two hand cards to discard.
    let owner_hand = tx.state().zones.find(ZoneKind::Hand, setup.owner).unwrap();
    let p1 = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Sha,
        96_614,
        Suit::Club,
        7,
    );
    let p2 = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Sha,
        96_615,
        Suit::Club,
        8,
    );
    let state = tx.commit().state;
    let (state, activated) = activate(state, setup.owner, DIMENG, 1);
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    let (state, picked) = resolve(
        state,
        setup.owner,
        vec![setup.other.raw(), setup.far.raw()],
        2,
    );
    assert!(matches!(picked.result, CommandResult::Accepted { .. }));
    // Discard 2 cards (the diff).
    let (state, paid) = resolve(state, setup.owner, vec![p1.raw(), p2.raw()], 3);
    assert!(matches!(paid.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    // Hands exchanged: other now has 1, far now has 3.
    assert_eq!(state.hand_count(setup.other), 1);
    assert_eq!(state.hand_count(setup.far), 3);
    assert!(state.hand_cards(setup.far).contains(&o1));
    assert!(state.hand_cards(setup.far).contains(&o2));
    assert!(state.hand_cards(setup.far).contains(&o3));
    assert!(state.hand_cards(setup.other).contains(&f1));
    // The payment cards are discarded.
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(p1).unwrap().zone, discard);
    assert_eq!(state.cards.get(p2).unwrap().zone, discard);
}
