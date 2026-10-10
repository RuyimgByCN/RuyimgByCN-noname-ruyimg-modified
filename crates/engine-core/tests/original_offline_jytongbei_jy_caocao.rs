//! Golden coverage for 荆扬曹操「统北」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const JY_CAOCAO: u32 = 4_130_532_255;
const JYLIJUN: u32 = 2_928_426_897;
const JYT_ONGBEI: u32 = 2_908_493_496;
const SOURCE: &str = include_str!("../../../content/original/ported/offline/jytongbei.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

fn submit(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (state, result) = apply_command(
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
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

fn setup(seed: u64) -> (GameState, PlayerId, PlayerId, CardId) {
    let package = content_compiler::compile_source_named("jytongbei-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 498);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target) = (players[0], players[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::DiscardFromHand {
        player: target,
        count: u32::MAX,
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(JYT_ONGBEI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().group_overrides.insert(target, "qun".into());
    let hand = tx.state().zones.find(ZoneKind::Hand, target).unwrap();
    let tao = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_498),
        kind: CardKind::Tao,
        suit: Suit::Heart,
        rank: 9,
        weapon_range: 0,
        zone: hand,
        owner: target,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(tao);
    (tx.commit().state, owner, target, tao)
}

fn begin_damage(state: GameState, source: PlayerId, target: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::CommitDamage {
            damage_id: 0,
            source,
            target,
            amount: 1,
            base_amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
            card: CardId::NONE,
            card_kind: 0,
            card_flags: 0,
        }],
    );
    assert!(matches!(result, Err(EffectError::ChoiceRequested)));
    tx.commit().state
}

#[test]
fn package_and_runtime_cover_gift_and_damage_branches() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 691);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.id == JY_CAOCAO)
            .unwrap()
            .skills,
        [JYLIJUN, JYT_ONGBEI]
    );

    let (state, owner, target, tao) = setup(2_026_080_203);
    let hp = state.players.get(target).unwrap().hp;
    let state = begin_damage(state, owner, target);
    assert!(state.has_pending_choice());
    let state = submit(state, owner, vec![1], false, 1);
    assert!(state.has_pending_choice());
    let state = submit(state, owner, vec![1], false, 2);
    assert!(state.has_pending_choice());
    let state = submit(state, target, vec![tao.raw()], false, 3);
    assert!(state.hand_cards(owner).contains(&tao));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);

    let (state, owner, target, _tao) = setup(2_026_080_204);
    let hp = state.players.get(target).unwrap().hp;
    let state = begin_damage(state, owner, target);
    let state = submit(state, owner, vec![1], false, 4);
    let state = submit(state, owner, vec![1], false, 5);
    let state = submit(state, target, Vec::new(), true, 6);
    assert_eq!(state.players.get(target).unwrap().hp, hp - 2);
}
