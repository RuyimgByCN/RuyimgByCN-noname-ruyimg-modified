//! Golden tests for KEY「求略」rewritten from the legacy JavaScript semantics.

use engine_api::{
    CardDefId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, SkillId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{turn_flag, Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const HINATA_QIULVE: u32 = 3_761_656_212;
const HINATA_QIULVE_CLEAR: u32 = 3_988_736_644;
const SOURCE: &str = include_str!("../../../content/original/ported/key/hinata_qiulve.skill");
const KEY: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

fn engine(seed: u64, players: u8) -> Engine {
    let package = content_compiler::compile_source_named("hinata_qiulve_test", SOURCE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: players,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    engine
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

fn isolate_and_add(tx: &mut Transaction, owner: PlayerId) -> SkillId {
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(HINATA_QIULVE),
    });
    run_effects(tx, effects).unwrap();
    tx.state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == HINATA_QIULVE)
        .unwrap()
}

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    suit: Suit,
    rank: u8,
) -> engine_api::CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: CardDefId::new(99_000 + kind.tag() as u32 * 20 + u32::from(rank)),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn activate(
    actor: PlayerId,
    skill: SkillId,
    target: PlayerId,
    card: engine_api::CardId,
    key: u64,
) -> Command {
    Command {
        actor,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(key),
        kind: CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: vec![card],
        },
    }
}

#[test]
fn committed_package_contains_typed_view_as_contract_and_folded_helper() {
    let committed = content_schema::ContentPackage::from_json_bytes(KEY).unwrap();
    let compiled = content_compiler::compile_source_named("hinata_qiulve", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 14);
    let play = compiled
        .view_as_index
        .iter()
        .find(|view| view.skill_def_id == HINATA_QIULVE && view.window == "play_phase")
        .unwrap();
    assert_eq!(play.filter, "non_basic");
    assert_eq!(play.source_zone.as_deref(), Some("hand_equip"));
    assert!(play.ignore_distance && play.red_unrespondable && play.black_no_use_count);
    assert!(compiled
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == HINATA_QIULVE_CLEAR));
    for expected in &compiled.view_as_index {
        let actual = committed
            .view_as_index
            .iter()
            .find(|view| {
                view.skill_def_id == expected.skill_def_id && view.window == expected.window
            })
            .unwrap();
        assert_eq!(actual, expected);
    }
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == HINATA_QIULVE_CLEAR));
    for expected in &compiled.modules {
        let actual = committed
            .modules
            .iter()
            .find(|module| module.name == expected.name)
            .unwrap();
        assert_eq!(actual.logic_hash, expected.logic_hash);
    }
}

#[test]
fn red_non_basic_sha_reaches_distance_two_and_cannot_be_answered() {
    let engine = engine(20_260_718, 4);
    let actor = engine.state().turn.current;
    let seats = engine.state().players.alive_by_seat();
    let actor_index = seats.iter().position(|player| *player == actor).unwrap();
    let far = seats[(actor_index + 2) % seats.len()];
    let mut tx = Transaction::begin(engine.state().clone());
    tx.state_mut().turn.phase = Phase::Play;
    clear_hand(&mut tx, actor);
    clear_hand(&mut tx, far);
    let skill = isolate_and_add(&mut tx, actor);
    let source = inject(
        &mut tx,
        actor,
        ZoneKind::Hand,
        CardKind::Weapon,
        Suit::Heart,
        7,
    );
    inject(&mut tx, far, ZoneKind::Hand, CardKind::Shan, Suit::Club, 8);
    let hp = tx.state().players.get(far).unwrap().hp;

    let (state, pipe) = apply_command(tx.state().clone(), activate(actor, skill, far, source, 1));
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert!(pipe.choice.is_none());
    assert_eq!(state.players.get(far).unwrap().hp, hp - 1);
}

#[test]
fn black_non_basic_sha_does_not_consume_or_require_a_sha_use() {
    let engine = engine(20_260_719, 2);
    let actor = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != actor)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    tx.state_mut().turn.phase = Phase::Play;
    clear_hand(&mut tx, actor);
    clear_hand(&mut tx, target);
    let skill = isolate_and_add(&mut tx, actor);
    let source = inject(
        &mut tx,
        actor,
        ZoneKind::Hand,
        CardKind::TrickWuzhong,
        Suit::Spade,
        9,
    );
    tx.state_mut().turn.set_flag(turn_flag::SHA_PLAYED, 1);
    let hp = tx.state().players.get(target).unwrap().hp;

    let (state, pipe) = apply_command(
        tx.state().clone(),
        activate(actor, skill, target, source, 2),
    );
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED), 1);
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);

    let mut basic = Transaction::begin(tx.state().clone());
    let rejected = inject(
        &mut basic,
        actor,
        ZoneKind::Hand,
        CardKind::Shan,
        Suit::Spade,
        10,
    );
    let (_, pipe) = apply_command(
        basic.state().clone(),
        activate(actor, skill, target, rejected, 3),
    );
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));
}

#[test]
fn non_basic_hand_or_equipment_card_can_answer_duel_as_sha() {
    let engine = engine(20_260_720, 2);
    let source_player = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != source_player)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    tx.state_mut().turn.phase = Phase::Play;
    clear_hand(&mut tx, source_player);
    clear_hand(&mut tx, target);
    isolate_and_add(&mut tx, target);
    let duel = inject(
        &mut tx,
        source_player,
        ZoneKind::Hand,
        CardKind::TrickJuedou,
        Suit::Diamond,
        11,
    );
    let equipment = inject(
        &mut tx,
        target,
        ZoneKind::Equip,
        CardKind::Armor,
        Suit::Club,
        12,
    );

    let (_, pipe) = apply_command(
        tx.state().clone(),
        Command {
            actor: source_player,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(4),
            kind: CommandKind::PlayCard {
                card: duel,
                targets: vec![target],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    let choice = pipe.choice.expect("求略应提供非基本牌作为响应杀");
    assert_eq!(choice.observer, target);
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == equipment.raw()));
}
