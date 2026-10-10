//! Golden coverage for BINGSHI 势鲁肃「好施/缔盟」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{run_effects, EffectError};
use engine_core::projection::project_player;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const POTDIMENG: u32 = 2_916_721_654;
const POTHAOSHI: u32 = 3_050_076_876;
const SOURCE: &str = include_str!("../../../content/original/ported/bingshi/pot_lusu.skill");

fn setup(seed: u64) -> (GameState, PlayerId, Vec<PlayerId>) {
    let package = content_compiler::compile_source(SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 536);
    assert!(package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == POTDIMENG)
        .unwrap()
        .targets
        .as_ref()
        .is_some_and(|targets| targets.pair_hand_difference_lte_three));
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
    let owner = engine.state().turn.current;
    let order = engine.state().players.alive_by_seat();
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
        skill_def: SkillDefId::new(POTDIMENG),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let owner_state = tx.state_mut().players.get_mut(owner).unwrap();
    owner_state.max_hp = 4;
    owner_state.hp = 2;
    clear_hands(&mut tx);
    (tx.commit().state, owner, order)
}

fn clear_hands(tx: &mut Transaction) {
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let mut effects = Vec::new();
    for player in tx.state().players.alive_by_seat() {
        let cards = tx.state().hand_cards(player);
        if !cards.is_empty() {
            effects.push(EffectRequest::MoveCards {
                cards,
                to: discard,
                from: tx.state().zones.find(ZoneKind::Hand, player),
            });
        }
    }
    run_effects(tx, effects).unwrap();
}

fn inject_hand(tx: &mut Transaction, player: PlayerId, serial: u32) -> CardId {
    inject_hand_kind(tx, player, serial, CardKind::Other)
}

fn inject_hand_kind(tx: &mut Transaction, player: PlayerId, serial: u32, kind: CardKind) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(535_000 + serial),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn skill(state: &GameState, owner: PlayerId) -> engine_api::SkillId {
    skill_by_def(state, owner, POTDIMENG)
}

fn skill_by_def(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == def)
        .unwrap()
        .id
}

fn resolve(
    state: GameState,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let actor = PlayerId::new(state.resolution.pending_observer);
    let revision = state.revision;
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: choice,
                selections,
                cancelled,
            },
        },
    )
}

fn grant_pothaoshi(state: GameState, owner: PlayerId, borrower: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    let skill = skill_by_def(tx.state(), owner, POTHAOSHI);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::ResolvePothaoshi {
            player: owner,
            target: PlayerId::NONE,
            action: 0,
            skill,
        }],
    );
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    let (state, output) = resolve(tx.commit().state, vec![borrower.raw()], false, 5360);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    state
}

fn setup_pothaoshi(seed: u64) -> (GameState, PlayerId, PlayerId, PlayerId) {
    let (state, owner, order) = setup(seed);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(POTHAOSHI),
        }],
    )
    .unwrap();
    let others: Vec<_> = order
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let borrower = others[0];
    let third = others[1];
    (
        grant_pothaoshi(tx.commit().state, owner, borrower),
        owner,
        borrower,
        third,
    )
}

#[test]
fn potdimeng_revalidates_pair_swaps_hands_and_settles_both_choices() {
    let (state, owner, order) = setup(5350);
    let target = order.into_iter().find(|player| *player != owner).unwrap();
    let mut tx = Transaction::begin(state);
    for serial in 20..=23 {
        inject_hand(&mut tx, target, serial);
    }
    let state = tx.commit().state;
    let (_, output) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::ActivateSkill {
                skill: skill(&state, owner),
                targets: vec![owner, target],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(!matches!(output.result, CommandResult::Accepted { .. }));

    let (state, owner, order) = setup(5351);
    let target = order.into_iter().find(|player| *player != owner).unwrap();
    let mut tx = Transaction::begin(state);
    inject_hand(&mut tx, owner, 1);
    for serial in 2..=5 {
        inject_hand(&mut tx, target, serial);
    }
    let state = tx.commit().state;
    let (state, output) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill: skill(&state, owner),
                targets: vec![owner, target],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), 4);
    assert_eq!(state.hand_count(target), 1);
    let (state, output) = resolve(state, Vec::new(), true, 2);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), 4);
    assert_eq!(state.hand_count(target), 3);

    let (state, owner, order) = setup(5352);
    let target = order.into_iter().find(|player| *player != owner).unwrap();
    let mut tx = Transaction::begin(state);
    inject_hand(&mut tx, owner, 10);
    for serial in 11..=14 {
        inject_hand(&mut tx, target, serial);
    }
    let state = tx.commit().state;
    let (state, _) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(3),
            kind: CommandKind::ActivateSkill {
                skill: skill(&state, owner),
                targets: vec![owner, target],
                card_targets: Vec::new(),
            },
        },
    );
    let cards: Vec<_> = state.hand_cards(owner)[..2]
        .iter()
        .map(|card| card.raw())
        .collect();
    let (state, output) = resolve(state, cards, false, 4);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), 2);
    assert_eq!(state.hand_count(target), 1);
}

#[test]
fn pothaoshi_borrows_visible_hand_refills_twice_and_clears() {
    let (state, owner, borrower, third) = setup_pothaoshi(5361);
    let mut tx = Transaction::begin(state);
    let first = inject_hand_kind(&mut tx, owner, 101, CardKind::Jiu);
    tx.state_mut().turn.current = borrower;
    tx.state_mut().turn.phase = Phase::Play;
    let state = tx.commit().state;

    assert!(state.playable_hand_cards(borrower).contains(&first));
    assert!(project_player(&state, borrower)
        .known_cards
        .iter()
        .any(|card| card.id == first && card.def.is_some()));
    assert!(!project_player(&state, third)
        .known_cards
        .iter()
        .any(|card| card.id == first && card.def.is_some()));

    let mut state = state;
    for (index, serial) in [101, 102, 103].into_iter().enumerate() {
        if index > 0 {
            let mut tx = Transaction::begin(state);
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .unwrap();
            let hand = tx.state().zones.find(ZoneKind::Hand, owner);
            let cards = tx.state().hand_cards(owner);
            if !cards.is_empty() {
                run_effects(
                    &mut tx,
                    vec![EffectRequest::MoveCards {
                        cards,
                        to: discard,
                        from: hand,
                    }],
                )
                .unwrap();
            }
            inject_hand_kind(&mut tx, owner, serial, CardKind::Jiu);
            state = tx.commit().state;
        }
        let card = state.hand_cards(owner)[0];
        let (next, output) = apply_command(
            state.clone(),
            Command {
                actor: borrower,
                expected_revision: ExpectedRevision(state.revision),
                idempotency_key: IdempotencyKey(5362 + index as u64),
                kind: CommandKind::PlayCard {
                    card,
                    targets: Vec::new(),
                    card_targets: Vec::new(),
                },
            },
        );
        assert!(matches!(output.result, CommandResult::Accepted { .. }));
        assert_eq!(next.hand_count(owner), if index < 2 { 3 } else { 0 });
        state = next;
    }

    let mut tx = Transaction::begin(state);
    let skill = skill_by_def(tx.state(), owner, POTHAOSHI);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolvePothaoshi {
            player: owner,
            target: PlayerId::NONE,
            action: 1,
            skill,
        }],
    )
    .unwrap();
    let after_clear = inject_hand_kind(&mut tx, owner, 104, CardKind::Jiu);
    assert!(!tx
        .state()
        .playable_hand_cards(borrower)
        .contains(&after_clear));
}

#[test]
fn pothaoshi_borrowed_shan_uses_the_response_pipeline() {
    let (state, owner, borrower, attacker) = setup_pothaoshi(5365);
    let mut tx = Transaction::begin(state);
    let shan = inject_hand_kind(&mut tx, owner, 201, CardKind::Shan);
    let sha = inject_hand_kind(&mut tx, attacker, 202, CardKind::Sha);
    tx.state_mut().turn.current = attacker;
    tx.state_mut().turn.phase = Phase::Play;
    let state = tx.commit().state;
    let (state, output) = apply_command(
        state.clone(),
        Command {
            actor: attacker,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(5366),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![borrower],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(PlayerId::new(state.resolution.pending_observer), borrower);
    let (state, output) = resolve(state, vec![shan.raw()], false, 5367);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), 3);
    assert!(state
        .players
        .get(borrower)
        .is_some_and(|player| player.hp == player.max_hp as i32));
}
