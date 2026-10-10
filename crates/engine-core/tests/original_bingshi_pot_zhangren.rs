//! Golden coverage for BINGSHI 势张任「伏暗」「引弦」.

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

const POTFUAN: u32 = 2_955_537_231;
const POTFUAN_RANGE: u32 = 2_955_537_232;
const POTYINXIAN: u32 = 2_595_028_433;
const SOURCE: &str = include_str!("../../../content/original/ported/bingshi/pot_zhangren.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/bingshi/bingshi.npack.json");

fn setup(seed: u64) -> (GameState, PlayerId, SkillId, SkillId) {
    let package = content_compiler::compile_source_named("pot_zhangren_test", SOURCE).unwrap();
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
    effects.extend(
        [POTFUAN, POTYINXIAN].map(|skill_def| EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill_def),
        }),
    );
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let find = |tx: &Transaction, def| {
        tx.state()
            .skills
            .sorted_live()
            .into_iter()
            .find(|skill| skill.owner == owner && skill.def.raw() == def)
            .unwrap()
            .id
    };
    let fuan = find(&tx, POTFUAN);
    let yinxian = find(&tx, POTYINXIAN);
    (tx.commit().state, owner, fuan, yinxian)
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
        def: CardDefId::new(525_000 + u32::from(rank)),
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
fn package_publishes_complete_pot_zhangren_at_abi525() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 677);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.name_key == "pot_zhangren")
            .unwrap()
            .skills,
        [POTFUAN, POTYINXIAN]
    );
}

#[test]
fn yinxian_boosts_farthest_sha_and_grows_range_outside_turn() {
    let (state, owner, _, yinxian) = setup(5251);
    let target = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(state);
    clear_hand(&mut tx, target);
    let sha = inject(&mut tx, owner, CardKind::Sha, Suit::Spade, 1);
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
    let base = attack_range(&state, owner);
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = target;
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolvePotyinxian {
            player: owner,
            card: sha,
            stage: 1,
            skill: yinxian,
        }],
    )
    .unwrap();
    assert_eq!(attack_range(tx.state(), owner), base + 1);
}

#[test]
fn fuan_records_suit_uses_virtual_sha_and_can_silence() {
    let (state, owner, fuan, yinxian) = setup(5252);
    let target = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(state);
    clear_hand(&mut tx, owner);
    clear_hand(&mut tx, target);
    let heart = inject(&mut tx, owner, CardKind::Tao, Suit::Heart, 2);
    tx.state_mut().turn.current = target;
    for _ in 0..3 {
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePotyinxian {
                player: owner,
                card: heart,
                stage: 1,
                skill: yinxian,
            }],
        )
        .unwrap();
    }
    tx.state_mut().turn.current = owner;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePotfuan {
                player: owner,
                card: CardId::NONE,
                stage: 0,
                range_helper_def: SkillDefId::new(POTFUAN_RANGE),
                yinxian_def: SkillDefId::new(POTYINXIAN),
                skill: fuan,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = choose(tx.commit().state, vec![2], 2);
    assert_eq!(attack_range(&state, owner), 0);

    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = target;
    let target_heart = inject(&mut tx, target, CardKind::Shan, Suit::Heart, 3);
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardCards {
            player: target,
            cards: vec![target_heart],
        }],
    )
    .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: target,
            skill_def: SkillDefId::new(POTFUAN),
        }],
    )
    .unwrap();
    let hp = tx.state().players.get(target).unwrap().hp;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePotfuan {
                player: owner,
                card: CardId::NONE,
                stage: 2,
                range_helper_def: SkillDefId::new(POTFUAN_RANGE),
                yinxian_def: SkillDefId::new(POTYINXIAN),
                skill: fuan,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = choose(tx.commit().state, vec![target.raw()], 3);
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
    assert!(state.has_pending_choice());
    let state = choose(state, vec![2], 4);
    let target_skill = state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == target && skill.def.raw() == POTFUAN)
        .unwrap();
    assert!(!state.skill_effectively_enabled(target_skill));
}
