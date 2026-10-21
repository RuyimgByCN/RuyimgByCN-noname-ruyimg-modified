//! Golden for OFFLINE 小杀「淑丽」rewritten from the legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const VTBSHULI: u32 = 3_163_490_927;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/offline/offline.npack.json");

// 2026-08-15：offline roster +1（zh_shen_zhangjiao）平移了固定种子的身份分配，
// 初始武将出现 skill_added 类可选触发打断 setup 批次；种子 _01→_03、_02→_04 平移恢复。
fn setup(seed: u64) -> (GameState, PlayerId, PlayerId, PlayerId, CardId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
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
    let players = engine.state().players.alive_by_seat();
    let (owner, source, target) = (players[0], players[1], players[2]);
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
        skill_def: SkillDefId::new(VTBSHULI),
    });
    run_effects(&mut tx, effects).unwrap();
    let victim = tx.state_mut().players.get_mut(target).unwrap();
    victim.hp = 20;
    victim.max_hp = 20;
    let hand = tx.state().zones.find(ZoneKind::Hand, source).unwrap();
    let sha = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(93_459),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: source,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(sha);
    (tx.commit().state, owner, source, target, sha)
}

fn deal_sha(
    state: GameState,
    source: PlayerId,
    target: PlayerId,
    sha: CardId,
) -> (GameState, Result<Vec<engine_api::DomainEvent>, EffectError>) {
    let mut tx = Transaction::begin(state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealShaDamage {
            source,
            target,
            card: sha,
            amount: 1,
            nature: rule_api::damage_nature::NORMAL,
            card_flags: 0,
        }],
    );
    (tx.commit().state, result)
}

fn accept(state: GameState, owner: PlayerId, key: u64) -> GameState {
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: choice,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn committed_package_contains_optional_twice_per_turn_sha_filter() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == VTBSHULI && !skill.locked));
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == VTBSHULI)
        .unwrap();
    assert_eq!(
        (trigger.event.as_str(), trigger.filter.as_str()),
        ("damage", "owner_alive_other_source_sha_twice")
    );
    assert!(trigger.optional);
}

#[test]
fn other_players_sha_damage_can_draw_for_both_sides_only_twice_per_turn() {
    let (mut state, owner, source, target, sha) = setup(20_260_716_03);
    let owner_hand = state.hand_count(owner);
    let source_hand = state.hand_count(source);
    for key in 1..=2 {
        let (next, result) = deal_sha(state, source, target, sha);
        assert_eq!(result, Err(EffectError::ChoiceRequested));
        state = accept(next, owner, key);
    }
    assert_eq!(state.hand_count(owner), owner_hand + 2);
    assert_eq!(state.hand_count(source), source_hand + 2);

    let (state, result) = deal_sha(state, source, target, sha);
    assert!(result.is_ok());
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(owner), owner_hand + 2);
    assert_eq!(state.hand_count(source), source_hand + 2);
}

#[test]
fn own_or_non_sha_damage_does_not_offer_shuli() {
    let (state, owner, source, target, sha) = setup(20_260_716_04);
    let (state, result) = deal_sha(state, owner, target, sha);
    assert!(result.is_ok());
    assert!(!state.has_pending_choice());

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());
}
