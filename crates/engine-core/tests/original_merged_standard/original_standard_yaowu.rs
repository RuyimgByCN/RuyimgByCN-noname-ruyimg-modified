//! Golden tests for standard 华雄「耀武」rewritten from legacy JavaScript.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const STANDARD: &[u8] = include_bytes!("../../../../content/standard/standard.npack.json");
const SOURCE: &str = include_str!("../../../../content/standard/standard_m1.skill");
const HUAXIONG: u32 = 4_212_795_236;
const YAOWU: u32 = 3_872_761_084;

fn loaded(seed: u64) -> (Transaction, PlayerId, PlayerId) {
    let package = content_compiler::compile_source(SOURCE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let source = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != source)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    let remove = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, remove).unwrap();
    for player in [source, target] {
        run_effects(
            &mut tx,
            vec![EffectRequest::DiscardFromHand {
                player,
                count: u32::MAX,
            }],
        )
        .unwrap();
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: target,
            skill_def: SkillDefId::new(YAOWU),
        }],
    )
    .unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    (tx, source, target)
}

fn inject_sha(tx: &mut Transaction, source: PlayerId, suit: Suit) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, source).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(70_000 + suit as u32),
        kind: CardKind::Sha,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: source,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn play(source: PlayerId, target: PlayerId, card: CardId, key: u64) -> Command {
    Command {
        actor: source,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(key),
        kind: CommandKind::PlayCard {
            card,
            targets: vec![target],
            card_targets: vec![],
        },
    }
}

#[test]
fn committed_standard_package_contains_huaxiong_and_locked_yaowu() {
    let package = content_schema::ContentPackage::from_json_bytes(STANDARD).unwrap();
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == HUAXIONG)
        .unwrap();
    assert_eq!((character.initial_hp, character.max_hp), (0, 6));
    assert_eq!(character.skills, [YAOWU]);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == YAOWU && skill.locked));
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == YAOWU)
        .unwrap();
    assert_eq!(trigger.event, "damage_before");
    assert_eq!(trigger.filter, "owner_is_target_red_sha_source_alive");
}

#[test]
fn red_sha_forces_the_living_source_to_choose_before_damage() {
    let (mut tx, source, target) = loaded(20_260_716_4);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: source,
            amount: 1,
            is_lose_hp: true,
            nature: 0,
        }],
    )
    .unwrap();
    let target_hp = tx.state().players.get(target).unwrap().hp;
    let sha = inject_sha(&mut tx, source, Suit::Diamond);
    let (state, pipeline) = apply_command(tx.state().clone(), play(source, target, sha, 1));
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    let choice = pipeline.choice.unwrap();
    assert_eq!(choice.observer, source);
    assert!(!choice.allow_cancel);
    assert_eq!(state.players.get(target).unwrap().hp, target_hp);
}

#[test]
fn nonred_sha_does_not_trigger_standard_yaowu() {
    let (mut tx, source, target) = loaded(20_260_716_5);
    let target_hp = tx.state().players.get(target).unwrap().hp;
    let target_hand = tx.state().hand_count(target);
    let sha = inject_sha(&mut tx, source, Suit::Club);
    let (state, pipeline) = apply_command(tx.state().clone(), play(source, target, sha, 2));
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert!(pipeline.choice.is_none());
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 1);
    assert_eq!(state.hand_count(target), target_hand);
}
