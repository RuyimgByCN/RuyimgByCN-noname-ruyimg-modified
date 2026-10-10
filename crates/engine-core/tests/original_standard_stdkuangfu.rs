//! Golden for standard 潘凤「狂斧」rewritten from the legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{turn_flag, Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const STANDARD: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");
const SOURCE: &str = include_str!("../../../content/standard/standard_m1.skill");
const STDKUANGFU: u32 = 4_213_747_779;

fn setup(seed: u64, owner_hp: i32, target_hp: i32) -> (GameState, PlayerId, PlayerId) {
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
    let owner = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
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
    for player in [owner, target] {
        effects.push(EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        });
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(STDKUANGFU),
    });
    run_effects(&mut tx, effects).unwrap();
    for (player, hp) in [(owner, owner_hp), (target, target_hp)] {
        let state = tx.state_mut().players.get_mut(player).unwrap();
        state.max_hp = state.max_hp.max(hp.max(1) as u32);
        state.hp = hp;
    }
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx.state().clone(), owner, target)
}

fn play_sha(state: GameState, owner: PlayerId, target: PlayerId, key: u64) -> GameState {
    let mut tx = Transaction::begin(state);
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(80_000 + key as u32),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    let (state, pipeline) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card,
                targets: vec![target],
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(pipeline.result, CommandResult::Accepted { .. }),
        "play Sha rejected: {:?}",
        pipeline.result
    );
    assert!(pipeline.choice.is_none());
    state
}

fn allow_another_sha(state: GameState) -> GameState {
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.set_flag(turn_flag::SHA_PLAYED, 0);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::SHA_PLAYED_IN_PLAY_PHASE, 0);
    tx.state().clone()
}

fn enter_another_play_phase(state: GameState, owner: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Discard.to_u8(),
        }],
    )
    .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        }],
    )
    .unwrap();
    tx.state_mut().turn.set_flag(turn_flag::SHA_PLAYED, 0);
    tx.state().clone()
}

#[test]
fn committed_package_contains_locked_stdkuangfu_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(STANDARD).unwrap();
    let panfeng = package
        .character_index
        .iter()
        .find(|character| character.name_key == "std_panfeng")
        .expect("standard package must publish legacy std_panfeng");
    assert_eq!(panfeng.id, 4_123_573_742);
    assert_eq!(panfeng.sex, "male");
    assert_eq!(panfeng.group, "qun");
    assert_eq!(panfeng.max_hp, 4);
    assert_eq!(panfeng.skills, [STDKUANGFU]);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDKUANGFU && skill.locked));
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == STDKUANGFU)
        .unwrap();
    assert_eq!(trigger.event, "damage");
    assert_eq!(
        trigger.filter,
        "owner_source_play_sha_other_alive_once_per_play_phase"
    );
    assert!(!trigger.optional);
}

#[test]
fn lower_hp_target_draws_two_only_once_in_each_play_phase() {
    let (state, owner, target) = setup(20_260_717_1, 4, 4);
    let state = play_sha(state, owner, target, 1);
    assert_eq!(state.hand_count(owner), 2);
    assert_eq!(state.players.get(owner).unwrap().hp, 4);

    let state = play_sha(allow_another_sha(state), owner, target, 2);
    assert_eq!(state.hand_count(owner), 2);
    assert_eq!(state.players.get(owner).unwrap().hp, 4);

    let state = play_sha(enter_another_play_phase(state, owner), owner, target, 3);
    assert_eq!(state.hand_count(owner), 4);
    assert_eq!(state.players.get(owner).unwrap().hp, 4);
}

#[test]
fn target_hp_not_lower_after_damage_makes_owner_lose_hp() {
    let (state, owner, target) = setup(20_260_717_2, 2, 4);
    let state = play_sha(state, owner, target, 4);
    assert_eq!(state.players.get(target).unwrap().hp, 3);
    assert_eq!(state.players.get(owner).unwrap().hp, 1);
    assert_eq!(state.hand_count(owner), 0);
}
