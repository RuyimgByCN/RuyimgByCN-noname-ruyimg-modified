//! Golden coverage for BINGSHI 势夏侯霸「砺兵」「破袭」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::query::{query, QueryContext};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const POTLIBING: u32 = 2_358_132_416;
const POTPOXI: u32 = 4_156_018_278;
const SOURCE: &str = include_str!("../../../../content/original/ported/bingshi/pot_xiahouba.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/bingshi/bingshi.npack.json");

fn setup(seed: u64, skill_def: u32) -> (GameState, PlayerId, SkillId) {
    let package = content_compiler::compile_source_named("pot_xiahouba_test", SOURCE).unwrap();
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
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == skill_def)
        .unwrap()
        .id;
    (tx.commit().state, owner, skill)
}

fn clear_hand(tx: &mut Transaction, player: PlayerId) {
    let cards = tx.state().hand_cards(player);
    if cards.is_empty() {
        return;
    }
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards,
            to: discard,
            from: Some(hand),
        }],
    )
    .unwrap();
}

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, suit: Suit, rank: u8) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(522_000 + u32::from(rank)),
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

fn choose(state: GameState, selections: Vec<u32>, key: u64) -> GameState {
    let actor = PlayerId::new(state.resolution.pending_observer);
    let (state, output) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled: false,
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    state
}

fn attack_range(state: &GameState, player: PlayerId) -> i32 {
    match query(
        state,
        &QueryRequest::AttackRange { player },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("unexpected attack-range response: {other:?}"),
    }
}

#[test]
fn package_publishes_complete_pot_xiahouba_at_abi522() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 677);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.name_key == "pot_xiahouba")
            .unwrap()
            .skills,
        [POTLIBING, POTPOXI]
    );
}

#[test]
fn libing_cycles_range_draw_and_next_sha_damage_then_resets() {
    let (state, owner, skill) = setup(5221, POTLIBING);
    let target = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(state);
    clear_hand(&mut tx, target);
    let non_damage = inject(&mut tx, owner, CardKind::Tao, Suit::Heart, 2);
    let sha = inject(&mut tx, owner, CardKind::Sha, Suit::Spade, 3);
    let base_range = attack_range(tx.state(), owner);
    for _ in 0..3 {
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePotlibing {
                player: owner,
                card: non_damage,
                stage: 0,
                skill,
            }],
        )
        .unwrap();
    }
    assert_eq!(attack_range(tx.state(), owner), base_range + 1);
    let hp = tx.state().players.get(target).unwrap().hp;
    let state = tx.commit().state;
    let (state, output) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![target],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 2);
    assert_eq!(attack_range(&state, owner), base_range);
}

fn discard_sha_for_poxi(mut tx: Transaction, owner: PlayerId, sha: CardId) -> GameState {
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DiscardCards {
                player: owner,
                cards: vec![sha],
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    tx.commit().state
}

#[test]
fn poxi_direct_hits_targets_with_no_more_hand_cards() {
    let (state, owner, _) = setup(5222, POTPOXI);
    let target = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(state);
    clear_hand(&mut tx, owner);
    clear_hand(&mut tx, target);
    let sha = inject(&mut tx, owner, CardKind::Sha, Suit::Club, 4);
    let hp = tx.state().players.get(target).unwrap().hp;
    let state = discard_sha_for_poxi(tx, owner, sha);
    let state = choose(state, vec![target.raw()], 2);
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
}

#[test]
fn poxi_allows_a_target_with_more_hand_cards_to_respond() {
    let (state, owner, _) = setup(5223, POTPOXI);
    let target = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(state);
    clear_hand(&mut tx, owner);
    clear_hand(&mut tx, target);
    let sha = inject(&mut tx, owner, CardKind::Sha, Suit::Diamond, 5);
    let shan = inject(&mut tx, target, CardKind::Shan, Suit::Heart, 6);
    let hp = tx.state().players.get(target).unwrap().hp;
    let state = discard_sha_for_poxi(tx, owner, sha);
    let state = choose(state, vec![target.raw()], 3);
    assert!(state.has_pending_choice());
    let state = choose(state, vec![shan.raw()], 4);
    assert_eq!(state.players.get(target).unwrap().hp, hp);
}
