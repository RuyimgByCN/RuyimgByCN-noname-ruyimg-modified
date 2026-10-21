//! Golden coverage for mobile 吴珂「助国」and the completed character entry.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const WUKE: u32 = 3_285_217_552;
const MBANDA: u32 = 2_849_838_127;
const MBZHUGUO: u32 = 2_303_782_932;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/mobile/mobile.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup() -> (Transaction, PlayerId, Vec<PlayerId>) {
    let mut engine = Engine::new();
    engine.load_package(&package()).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 4,
                rng_seed: 2_026_073_102,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    // 批 269 起，孛明（文钦）game_started 触发会在开局打开选择；本测无关，
    // 由选择观察者取消以继续。
    if let Some(choice) = engine.last_choice() {
        let observer = choice.observer;
        let choice_id = choice.choice_id;
        assert!(matches!(
            engine.submit(Command {
                actor: observer,
                expected_revision: ExpectedRevision(engine.state().revision),
                idempotency_key: IdempotencyKey(999_001),
                kind: CommandKind::ResolveChoice {
                    choice_id,
                    selections: vec![],
                    cancelled: true,
                },
            }),
            CommandResult::Accepted { .. }
        ));
    }
    assert!(!engine.state().has_pending_choice());
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
        skill_def: SkillDefId::new(MBZHUGUO),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in tx.state().players.alive_by_seat() {
        clear_hand(&mut tx, player);
    }
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, others)
}

fn clear_hand(tx: &mut Transaction, player: PlayerId) {
    let cards = tx.state().hand_cards(player);
    if !cards.is_empty() {
        run_effects(tx, vec![EffectRequest::DiscardCards { player, cards }]).unwrap();
    }
}

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, offset: u32) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(331_001 + offset),
        kind,
        suit: if offset % 2 == 0 {
            Suit::Heart
        } else {
            Suit::Spade
        },
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn skill_instance(state: &GameState, owner: PlayerId) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == MBZHUGUO)
        .unwrap()
        .id
}

fn begin(tx: &mut Transaction, owner: PlayerId, target: PlayerId) -> Result<(), EffectError> {
    let skill = skill_instance(tx.state(), owner);
    run_effects(
        tx,
        vec![EffectRequest::ResolveMbzhuguo {
            player: owner,
            target,
            stage: 0,
            skill,
        }],
    )
    .map(|_| ())
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, Vec<DomainEvent>) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (state, output) = apply_command(
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
        matches!(output.result, CommandResult::Accepted { .. }),
        "{:?}",
        output.result
    );
    (state, output.events)
}

#[test]
fn package_publishes_current_abi_and_complete_wuke() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 734);
    assert_eq!(package.skill_index.len(), 441);
    assert_eq!(package.character_index.len(), 205);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == WUKE)
        .unwrap();
    assert_eq!(character.name_key, "wuke");
    assert_eq!(character.sex, "female");
    assert_eq!(character.group, "wu");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, [MBANDA, MBZHUGUO]);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == MBZHUGUO)
        .unwrap();
    assert_eq!(active.window, "play_phase");
    assert_eq!(active.uses_per_turn, Some(1));
    assert_eq!(
        active
            .targets
            .as_ref()
            .map(|targets| (targets.min, targets.max, targets.allow_self)),
        Some((1, 1, true))
    );
}

#[test]
fn draw_to_cap_then_use_physical_sha_on_owner_selected_target() {
    let (mut tx, owner, others) = setup();
    let adjusted = others[0];
    let victim = others[2];
    let sha = inject(&mut tx, adjusted, CardKind::Sha, 0);
    let limit = tx.state().players.get(adjusted).unwrap().max_hp.min(5);
    assert_eq!(
        begin(&mut tx, owner, adjusted).unwrap_err(),
        EffectError::ChoiceRequested
    );
    assert_eq!(tx.state().hand_count(adjusted), limit);

    let (target_chosen, _) = resolve(tx.state().clone(), owner, vec![victim.raw()], false, 1);
    assert_eq!(target_chosen.resolution.pending_observer, adjusted.raw());
    let (sha_chosen, _) = resolve(target_chosen, adjusted, vec![sha.raw()], false, 2);
    let (used, events) = resolve(sha_chosen, adjusted, vec![victim.raw()], false, 3);
    assert!(!used.hand_cards(adjusted).contains(&sha));
    assert!(events.iter().any(
        |event| matches!(event, DomainEvent::CardUsed { player, card, targets } if *player == adjusted && *card == sha && targets.contains(&victim))
    ));
}

#[test]
fn excess_hand_discards_to_cap_then_recovers() {
    let (mut tx, owner, others) = setup();
    let target = others[0];
    let limit = tx.state().players.get(target).unwrap().max_hp.min(5);
    for offset in 0..=limit {
        inject(&mut tx, target, CardKind::Sha, offset);
    }
    tx.state_mut().players.get_mut(target).unwrap().hp = 2;
    assert_eq!(
        begin(&mut tx, owner, target).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let discard = tx.state().hand_cards(target)[0];
    let (resolved, events) = resolve(tx.state().clone(), target, vec![discard.raw()], false, 4);
    assert_eq!(resolved.hand_count(target), limit);
    assert_eq!(resolved.players.get(target).unwrap().hp, 3);
    assert!(events.iter().any(
        |event| matches!(event, DomainEvent::HpRecovered { player, amount: 1, .. } if *player == target)
    ));
}

#[test]
fn equal_hand_count_recovers_without_follow_up() {
    let (mut tx, owner, others) = setup();
    let target = others[0];
    let limit = tx.state().players.get(target).unwrap().max_hp.min(5);
    for offset in 0..limit {
        inject(&mut tx, target, CardKind::Sha, offset);
    }
    tx.state_mut().players.get_mut(target).unwrap().hp = 2;
    let skill = skill_instance(tx.state(), owner);
    let events = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveMbzhuguo {
            player: owner,
            target,
            stage: 0,
            skill,
        }],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());
    assert_eq!(tx.state().players.get(target).unwrap().hp, 3);
    assert!(events.iter().any(
        |event| matches!(event, DomainEvent::HpRecovered { player, amount: 1, .. } if *player == target)
    ));
}
