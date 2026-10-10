//! Golden tests for 旧华雄「耀武」rewritten from the legacy JavaScript semantics.

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

const OLD: &[u8] = include_bytes!("../../../content/original/ported/old/old.npack.json");
const REFRESH: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");
const SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/refresh/new_reyaowu.skill"),
    include_str!("../../../content/original/ported/refresh/reyaowu.skill")
);
const OL_HUAXIONG: u32 = 2_161_198_899;
const NEW_REYAOWU: u32 = 2_159_889_222;
const REYAOWU: u32 = 2_805_206_003;

fn loaded(seed: u64, skill_def: u32) -> (Transaction, PlayerId, PlayerId) {
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
            skill_def: SkillDefId::new(skill_def),
        }],
    )
    .unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    (tx, source, target)
}

fn inject_card(tx: &mut Transaction, source: PlayerId, kind: CardKind, suit: Suit) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, source).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(69_000 + suit as u32),
        kind,
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

fn inject_sha(tx: &mut Transaction, source: PlayerId, suit: Suit) -> CardId {
    inject_card(tx, source, CardKind::Sha, suit)
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
fn committed_package_contains_old_huaxiong_and_both_damage_branches() {
    let package = content_schema::ContentPackage::from_json_bytes(OLD).unwrap();
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == OL_HUAXIONG)
        .unwrap();
    assert_eq!((character.initial_hp, character.max_hp), (0, 6));
    assert_eq!(character.skills, vec![NEW_REYAOWU]);
    let skills = content_schema::ContentPackage::from_json_bytes(REFRESH).unwrap();
    assert!(skills
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == NEW_REYAOWU && skill.locked));
    let filters: Vec<_> = skills
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == NEW_REYAOWU)
        .map(|trigger| trigger.filter.as_str())
        .collect();
    assert_eq!(
        filters,
        [
            "owner_is_target_red_sha_source_alive",
            "owner_is_target_nonred_sha"
        ]
    );
    let filters: Vec<_> = skills
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == REYAOWU)
        .map(|trigger| trigger.filter.as_str())
        .collect();
    assert_eq!(
        filters,
        [
            "owner_is_target_red_card_source_alive",
            "owner_is_target_nonred_card"
        ]
    );
}

#[test]
fn red_sha_pauses_before_damage_for_the_source_to_recover_or_draw() {
    let (mut tx, source, target) = loaded(20_260_716_1, NEW_REYAOWU);
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
    let source_hp = tx.state().players.get(source).unwrap().hp;
    let target_hp = tx.state().players.get(target).unwrap().hp;
    let sha = inject_sha(&mut tx, source, Suit::Heart);

    let (state, pipeline) = apply_command(tx.state().clone(), play(source, target, sha, 1));
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    let choice = pipeline.choice.unwrap();
    assert_eq!(choice.observer, source);
    assert!(!choice.allow_cancel);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [1, 2]
    );
    assert_eq!(state.players.get(target).unwrap().hp, target_hp);

    let (state, pipeline) = apply_command(
        state,
        Command {
            actor: source,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![2],
                cancelled: false,
            },
        },
    );
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(source).unwrap().hp, source_hp + 1);
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 1);
}

#[test]
fn nonred_sha_draws_for_the_target_without_opening_a_choice() {
    let (mut tx, source, target) = loaded(20_260_716_2, NEW_REYAOWU);
    let hp = tx.state().players.get(target).unwrap().hp;
    let hand = tx.state().hand_count(target);
    let sha = inject_sha(&mut tx, source, Suit::Spade);
    let (state, pipeline) = apply_command(tx.state().clone(), play(source, target, sha, 3));
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(target), hand + 1);
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
}

#[test]
fn old_reyaowu_uses_any_linked_card_and_ignores_unlinked_damage() {
    let (mut red, source, target) = loaded(20_260_716_4, REYAOWU);
    let card = inject_card(&mut red, source, CardKind::TrickHuogong, Suit::Heart);
    let source_hand = red.state().hand_count(source);
    let target_hp = red.state().players.get(target).unwrap().hp;
    run_effects(
        &mut red,
        vec![EffectRequest::DealCardDamage {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
            card,
            card_kind: CardKind::TrickHuogong.tag(),
            card_flags: 0,
        }],
    )
    .unwrap();
    assert_eq!(red.state().hand_count(source), source_hand + 1);
    assert_eq!(red.state().players.get(target).unwrap().hp, target_hp - 1);

    let (mut black, source, target) = loaded(20_260_716_5, REYAOWU);
    let card = inject_card(&mut black, source, CardKind::TrickHuogong, Suit::Club);
    let target_hand = black.state().hand_count(target);
    run_effects(
        &mut black,
        vec![EffectRequest::DealCardDamage {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
            card,
            card_kind: CardKind::TrickHuogong.tag(),
            card_flags: 0,
        }],
    )
    .unwrap();
    assert_eq!(black.state().hand_count(target), target_hand + 1);

    let (mut unlinked, source, target) = loaded(20_260_716_6, REYAOWU);
    let source_hand = unlinked.state().hand_count(source);
    let target_hand = unlinked.state().hand_count(target);
    run_effects(
        &mut unlinked,
        vec![EffectRequest::DealDamage {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(unlinked.state().hand_count(source), source_hand);
    assert_eq!(unlinked.state().hand_count(target), target_hand);
}

#[test]
fn elemental_chain_keeps_the_sha_card_provenance() {
    let package = content_compiler::compile_source(SOURCE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: 20_260_716_3,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let source = engine.state().turn.current;
    let seats = engine.state().players.alive_by_seat();
    let source_index = seats.iter().position(|player| *player == source).unwrap();
    let first = seats[(source_index + 1) % 3];
    let owner = seats[(source_index + 2) % 3];
    let mut tx = Transaction::begin(engine.state().clone());
    let remove = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, remove).unwrap();
    for player in [source, first, owner] {
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
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(NEW_REYAOWU),
            },
            EffectRequest::AddMark {
                player: first,
                mark: MarkDefId::new(engine_core::content::MARK_CHAIN),
                delta: 1,
            },
            EffectRequest::AddMark {
                player: owner,
                mark: MarkDefId::new(engine_core::content::MARK_CHAIN),
                delta: 1,
            },
            EffectRequest::DealDamage {
                source: PlayerId::NONE,
                target: source,
                amount: 1,
                is_lose_hp: true,
                nature: 0,
            },
        ],
    )
    .unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    let sha = inject_sha(&mut tx, source, Suit::Diamond);
    tx.state_mut().cards.get_mut(sha).unwrap().kind = CardKind::ShaFire;
    let first_hp = tx.state().players.get(first).unwrap().hp;
    let owner_hp = tx.state().players.get(owner).unwrap().hp;

    let (state, pipeline) = apply_command(tx.state().clone(), play(source, first, sha, 4));
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    let choice = pipeline.choice.unwrap();
    assert_eq!(choice.observer, source);
    assert_eq!(state.players.get(first).unwrap().hp, first_hp - 1);
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp);

    let (state, pipeline) = apply_command(
        state,
        Command {
            actor: source,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(5),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp - 1);
}
