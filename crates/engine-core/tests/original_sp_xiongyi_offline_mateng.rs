//! Golden coverage for SP `xiongyi` and the dependent OFFLINE Mateng closure.

use engine_api::{
    Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const XIONGYI: u32 = 2_476_017_492;
const MATENG: u32 = 3_979_368_086;
const STANDARD: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");
const SHENHUA: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");
const SP: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");
const OFFLINE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

fn package(bytes: &[u8]) -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(bytes).unwrap()
}

fn started(seed: u64) -> (GameState, PlayerId, PlayerId, PlayerId) {
    let standard = package(STANDARD);
    let shenhua = package(SHENHUA);
    let sp = package(SP);
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&standard).unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&shenhua).unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&sp).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let others: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(XIONGYI),
        }],
    )
    .unwrap();
    (tx.state().clone(), owner, others[0], others[1])
}

fn activate(
    state: GameState,
    owner: PlayerId,
    targets: Vec<PlayerId>,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let skill = state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| state.skills.get(*skill).unwrap().def.raw() == XIONGYI)
        .unwrap();
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets,
                card_targets: Vec::new(),
            },
        },
    )
}

#[test]
fn packages_publish_xiongyi_and_close_offline_mateng() {
    let sp = package(SP);
    assert!(sp.manifest.rule_vm_abi_version >= 647);
    let active = sp
        .active_index
        .iter()
        .find(|active| active.skill_def_id == XIONGYI)
        .unwrap();
    assert_eq!(active.window, "play_phase");
    assert!(active.once_per_game);
    let targets = active.targets.as_ref().unwrap();
    assert_eq!((targets.min, targets.max), (1, 255));
    assert!(targets.allow_self);

    let offline = package(OFFLINE);
    assert!(offline.manifest.dependencies.iter().any(|dependency| {
        dependency.namespace == "official"
            && dependency.package_id == "standard"
            && dependency.version == "0.1.0"
    }));
    assert!(offline.manifest.dependencies.iter().any(|dependency| {
        dependency.namespace == "official"
            && dependency.package_id == "sp"
            && dependency.version == "0.1.0"
    }));
    let mateng = offline
        .character_index
        .iter()
        .find(|character| character.id == MATENG)
        .unwrap();
    assert_eq!(mateng.name_key, "mateng");
    assert_eq!(mateng.sex, "male");
    assert_eq!(mateng.group, "qun");
    assert_eq!(mateng.max_hp, 4);
    assert_eq!(mateng.skills, [2, XIONGYI]);
}

#[test]
fn xiongyi_draws_selected_targets_in_seat_order_recovers_at_minimum_and_is_limited() {
    let (mut state, owner, first_other, second_other) = started(2_026_072_301);
    state.players.get_mut(owner).unwrap().hp = 2;
    state.players.get_mut(first_other).unwrap().hp = 2;
    state.players.get_mut(second_other).unwrap().hp = 3;
    let owner_hand = state.hand_count(owner);
    let target_hand = state.hand_count(first_other);
    let expected_order: Vec<_> = state
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player == owner || *player == first_other)
        .collect();

    let (state, pipe) = activate(state, owner, vec![first_other, owner], 1);
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), owner_hand + 3);
    assert_eq!(state.hand_count(first_other), target_hand + 3);
    assert_eq!(state.players.get(owner).unwrap().hp, 3);
    let draw_order: Vec<_> = pipe
        .events
        .iter()
        .filter_map(|event| match event {
            DomainEvent::CardsDrawn { player, cards } if cards.len() == 3 => Some(*player),
            _ => None,
        })
        .collect();
    assert_eq!(draw_order, expected_order);
    assert!(pipe.events.iter().any(|event| matches!(
        event,
        DomainEvent::HpRecovered {
            player,
            amount: 1,
            ..
        } if *player == owner
    )));

    let owner_hand = state.hand_count(owner);
    let (state, pipe) = activate(state, owner, vec![second_other], 2);
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));
    assert_eq!(state.hand_count(owner), owner_hand);
}

#[test]
fn xiongyi_does_not_recover_when_owner_is_not_at_minimum_hp() {
    let (mut state, owner, target, other) = started(2_026_072_302);
    state.players.get_mut(owner).unwrap().hp = 3;
    state.players.get_mut(other).unwrap().hp = 1;
    let target_hand = state.hand_count(target);

    let (state, pipe) = activate(state, owner, vec![target], 3);
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(target), target_hand + 3);
    assert_eq!(state.players.get(owner).unwrap().hp, 3);
    assert!(!pipe
        .events
        .iter()
        .any(|event| matches!(event, DomainEvent::HpRecovered { player, .. } if *player == owner)));
}
