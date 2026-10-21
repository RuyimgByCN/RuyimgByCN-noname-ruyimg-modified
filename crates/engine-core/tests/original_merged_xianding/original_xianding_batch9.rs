//! Golden for XIANDING（限定包）批次九：
//! 返攻（dcsbfangong）、新膂力（xinlvli）。

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, SkillId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DCSBFANGONG: u32 = 4_167_621_774;
const XINLVLI: u32 = 3_655_535_745;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package())
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, first, second) = (players[0], players[1], players[2]);
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
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, first, second)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
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

fn activate(
    state: GameState,
    actor: PlayerId,
    skill: SkillId,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: Vec::new(),
                card_targets: Vec::new(),
            },
        },
    )
}

fn deal_damage(
    state: GameState,
    source: PlayerId,
    target: PlayerId,
) -> (GameState, Result<Vec<engine_api::DomainEvent>, EffectError>) {
    let mut tx = Transaction::begin(state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
            card: CardId::NONE,
            card_kind: 0,
            card_flags: 0,
        }],
    );
    (tx.commit().state, result)
}

#[test]
fn package_exposes_batch9_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let fangong = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == DCSBFANGONG)
        .expect("dcsbfangong active");
    assert_eq!(fangong.window, "play_phase");
    assert!(fangong.once_per_game);

    let lvli = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == XINLVLI)
        .expect("xinlvli trigger");
    assert_eq!(lvli.event, "damage");
    assert_eq!(lvli.filter, "owner_source_once_per_turn");
    assert!(lvli.optional);
}

#[test]
fn dcsbfangong_recovers_and_draws_up_to_five() {
    let (mut tx, owner, _first, _second) = setup(20_260_891);
    add_skill(&mut tx, owner, DCSBFANGONG);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 1;
    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: owner,
            count: 3,
        }],
    )
    .unwrap();
    let max_hp = tx.state().players.get(owner).unwrap().max_hp as i32;
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == DCSBFANGONG)
        .expect("dcsbfangong skill instance")
        .id;
    let (state, pipe) = activate(tx.state().clone(), owner, skill, 1);
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, max_hp);
    // 摸 min(5, 当前手牌数)：激活前 3 张，回复后无变化，故摸 3 张。
    assert_eq!(state.hand_count(owner), 6);
}

#[test]
fn xinlvli_draws_difference_when_hp_exceeds_hand() {
    let (mut tx, owner, first, _second) = setup(20_260_892);
    add_skill(&mut tx, owner, XINLVLI);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 3;
    let before = tx.state().hand_count(owner);
    let (state, result) = deal_damage(tx.state().clone(), owner, first);
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    let state = resolve(state, owner, vec![1], false, 2);
    let hp = state.players.get(owner).unwrap().hp;
    let hand = state.hand_count(owner) as i32;
    assert_eq!(hand, before as i32 + hp - before as i32);
}

#[test]
fn xinlvli_recovers_difference_when_hand_exceeds_hp() {
    let (mut tx, owner, first, _second) = setup(20_260_893);
    add_skill(&mut tx, owner, XINLVLI);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 1;
    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: owner,
            count: 4,
        }],
    )
    .unwrap();
    let (state, result) = deal_damage(tx.state().clone(), owner, first);
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    let state = resolve(state, owner, vec![1], false, 3);
    let hp = state.players.get(owner).unwrap().hp;
    let hand = state.hand_count(owner) as i32;
    assert_eq!(hp, hand);
}
