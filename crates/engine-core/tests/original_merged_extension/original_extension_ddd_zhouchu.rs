//! Golden for 3D精选 周处【侠横】（ABI 618）。

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DDDXIAHENG: u32 = 3_492_764_267;
const STATE_MAGIC: u32 = 0xDDD0_6180;
const SOURCE: &str = include_str!("../../../../content/original/ported/ddd/zhouchu.skill");

fn setup(seed: u64) -> (GameState, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("ddd-zhouchu", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(DDDXIAHENG),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = players[0];
    tx.state_mut().turn.phase = Phase::Play;
    (tx.commit().state, players)
}

fn inject(state: &mut GameState, owner: PlayerId, serial: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_100 + serial),
        kind: CardKind::Shan,
        suit: Suit::Club,
        rank: 8,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn skill(state: &GameState, owner: PlayerId) -> engine_api::SkillId {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|id| {
            state
                .skills
                .get(*id)
                .is_some_and(|skill| skill.def.raw() == DDDXIAHENG)
        })
        .unwrap()
}

fn choose(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    )
}

fn start(state: GameState, owner: PlayerId) -> GameState {
    let instance = skill(&state, owner);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddXiaheng {
                player: owner,
                discard_target: PlayerId::NONE,
                damage_target: PlayerId::NONE,
                action: 0,
                skill: instance,
            }],
        ),
        Err(engine_core::effect::EffectError::ChoiceRequested)
    );
    tx.commit().state
}

#[test]
fn package_exposes_exact_abi_and_locked_trigger() {
    let package = content_compiler::compile_source_named("ddd-zhouchu", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 618);
    assert_eq!(package.skill_index.len(), 1);
    assert_eq!(package.trigger_index.len(), 1);
    assert!(!package.trigger_index[0].optional);
}

#[test]
fn xiaheng_forces_target_discard_then_damage_and_max_hp_loss() {
    let (mut state, players) = setup(20_260_806);
    let owner = players[0];
    let discard_target = players[1];
    let damage_target = players[2];
    let cards = [
        inject(&mut state, discard_target, 1),
        inject(&mut state, discard_target, 2),
        inject(&mut state, discard_target, 3),
    ];
    let old_hp = state.players.get(damage_target).unwrap().hp;
    let old_max_hp = state.players.get(owner).unwrap().max_hp;

    let state = start(state, owner);
    let (state, result) = choose(state, owner, vec![discard_target.raw()], 10);
    let choice = result.choice.unwrap();
    assert_eq!(choice.observer, discard_target);
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    let (state, result) = choose(
        state,
        discard_target,
        vec![cards[0].raw(), cards[1].raw()],
        11,
    );
    let choice = result.choice.unwrap();
    assert_eq!(choice.observer, owner);
    assert_eq!(choice.kind, ChoiceKind::SelectTargets);
    let (state, result) = choose(state, owner, vec![damage_target.raw()], 12);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(discard_target), 1);
    assert_eq!(state.players.get(damage_target).unwrap().hp, old_hp - 1);
    assert_eq!(state.players.get(owner).unwrap().max_hp, old_max_hp - 1);
}

#[test]
fn xiaheng_same_target_removes_the_skill_after_damage() {
    let (mut state, players) = setup(20_260_807);
    let owner = players[0];
    let target = players[1];
    let cards = [inject(&mut state, target, 4), inject(&mut state, target, 5)];
    let old_hp = state.players.get(target).unwrap().hp;

    let state = start(state, owner);
    let (state, _) = choose(state, owner, vec![target.raw()], 20);
    let (state, _) = choose(state, target, vec![cards[0].raw(), cards[1].raw()], 21);
    let (state, result) = choose(state, owner, vec![target.raw()], 22);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, old_hp - 1);
    assert!(state.players.get(owner).unwrap().skills.iter().all(|id| {
        state
            .skills
            .get(*id)
            .is_none_or(|instance| instance.def.raw() != DDDXIAHENG)
    }));
}

#[test]
fn third_distinct_damage_target_deletes_future_penalties() {
    let (mut state, players) = setup(20_260_808);
    let owner = players[0];
    let instance = skill(&state, owner);
    let blob = &mut state.skills.get_mut(instance).unwrap().state_blob;
    blob.extend_from_slice(&STATE_MAGIC.to_le_bytes());
    blob.push(0);
    blob.extend_from_slice(&2u32.to_le_bytes());
    blob.extend_from_slice(&players[1].raw().to_le_bytes());
    blob.extend_from_slice(&players[2].raw().to_le_bytes());
    let old_max_hp = state.players.get(owner).unwrap().max_hp;

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDddXiaheng {
            player: owner,
            discard_target: players[1],
            damage_target: players[3],
            action: 2,
            skill: instance,
        }],
    )
    .unwrap();
    let state = tx.commit().state;
    assert_eq!(state.players.get(owner).unwrap().max_hp, old_max_hp - 1);
    assert_eq!(state.skills.get(instance).unwrap().state_blob[4], 1);

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDddXiaheng {
            player: owner,
            discard_target: players[2],
            damage_target: players[2],
            action: 2,
            skill: instance,
        }],
    )
    .unwrap();
    let state = tx.commit().state;
    assert_eq!(state.players.get(owner).unwrap().max_hp, old_max_hp - 1);
    assert!(state.skills.get(instance).is_some());
}
