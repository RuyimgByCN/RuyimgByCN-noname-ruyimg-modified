//! Golden tests for yijiang 将华雄「恃勇」rewritten from legacy JavaScript.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    MarkDefId, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");
const SOURCE: &str = include_str!("../../../content/original/ported/yijiang/shiyong.skill");
const OLD_HUAXIONG: u32 = 3_259_572_192;
const SHIYONG: u32 = 2_676_469_853;

fn loaded(seed: u64) -> (Transaction, PlayerId, PlayerId) {
    let package = content_compiler::compile_source(SOURCE).unwrap();
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
    let source = engine.state().turn.current;
    let owner = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != source)
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
    for player in [source, owner] {
        effects.push(EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        });
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(SHIYONG),
    });
    run_effects(&mut tx, effects).unwrap();
    let player = tx.state_mut().players.get_mut(owner).unwrap();
    player.max_hp = 6;
    player.hp = 6;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, source, owner)
}

fn inject_sha(tx: &mut Transaction, source: PlayerId, suit: Suit) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, source).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(80_000 + suit as u32),
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

fn deal_sha(
    tx: &mut Transaction,
    source: PlayerId,
    owner: PlayerId,
    card: CardId,
    amount: u32,
    card_flags: u8,
) {
    run_effects(
        tx,
        vec![EffectRequest::DealShaDamage {
            source,
            target: owner,
            card,
            amount,
            nature: rule_api::damage_nature::NORMAL,
            card_flags,
        }],
    )
    .unwrap();
}

#[test]
fn committed_yijiang_package_contains_old_huaxiong_and_locked_shiyong() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == OLD_HUAXIONG)
        .unwrap();
    assert_eq!((character.initial_hp, character.max_hp), (0, 6));
    assert_eq!(character.skills, [SHIYONG]);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == SHIYONG && skill.locked));
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == SHIYONG)
        .unwrap();
    assert_eq!(trigger.event, "damage");
    assert_eq!(trigger.filter, "owner_is_target_red_or_jiu_sha");
}

#[test]
fn red_sha_damage_reduces_max_hp_after_damage() {
    let (mut tx, source, owner) = loaded(20_260_716_6);
    let card = inject_sha(&mut tx, source, Suit::Heart);
    deal_sha(&mut tx, source, owner, card, 1, 0);
    let player = tx.state().players.get(owner).unwrap();
    assert_eq!((player.hp, player.max_hp), (5, 5));
}

#[test]
fn black_sha_without_jiu_does_not_trigger_even_at_two_damage() {
    let (mut tx, source, owner) = loaded(20_260_716_7);
    let card = inject_sha(&mut tx, source, Suit::Club);
    deal_sha(&mut tx, source, owner, card, 2, 0);
    let player = tx.state().players.get(owner).unwrap();
    assert_eq!((player.hp, player.max_hp), (4, 6));
}

#[test]
fn black_sha_boosted_by_jiu_reduces_max_hp() {
    let (mut tx, source, owner) = loaded(20_260_716_8);
    let card = inject_sha(&mut tx, source, Suit::Spade);
    deal_sha(
        &mut tx,
        source,
        owner,
        card,
        2,
        rule_api::damage_card_flag::JIU_BOOSTED,
    );
    let player = tx.state().players.get(owner).unwrap();
    assert_eq!((player.hp, player.max_hp), (4, 5));
}

#[test]
fn playing_black_sha_with_jiu_mark_propagates_the_provenance_flag() {
    let (mut tx, source, owner) = loaded(20_260_716_9);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddMark {
            player: source,
            mark: MarkDefId::new(engine_core::content::MARK_JIU),
            delta: 1,
        }],
    )
    .unwrap();
    let card = inject_sha(&mut tx, source, Suit::Club);
    let (state, pipeline) = apply_command(
        tx.state().clone(),
        Command {
            actor: source,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card,
                targets: vec![owner],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert!(pipeline.choice.is_none());
    let player = state.players.get(owner).unwrap();
    assert_eq!((player.hp, player.max_hp), (4, 5));
    assert_eq!(
        state
            .players
            .get(source)
            .unwrap()
            .marks
            .iter()
            .find(|(mark, _)| *mark == engine_core::content::MARK_JIU)
            .map(|(_, count)| *count)
            .unwrap_or(0),
        0
    );
}
