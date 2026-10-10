//! Golden coverage for 荆扬庞统/连环's HP cost and virtual Tiesuo use.

use engine_api::{
    Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, SkillId, RULE_VM_ABI_VERSION,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::hash::canonical_state_hash;
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const JYLIANHUAN: u32 = 3_727_355_012;
const OFFLINE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

fn setup() -> (GameState, PlayerId, PlayerId, SkillId) {
    let package = content_schema::ContentPackage::from_json_bytes(OFFLINE).unwrap();
    assert!(
        (343..=RULE_VM_ABI_VERSION).contains(&package.manifest.rule_vm_abi_version),
        "unexpected package ABI {}",
        package.manifest.rule_vm_abi_version
    );
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == JYLIANHUAN && skill.name == "jylianhuan"));
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == JYLIANHUAN)
        .unwrap();
    assert_eq!(active.uses_per_play_phase, Some(1));
    let targets = active.targets.as_ref().unwrap();
    assert_eq!((targets.min, targets.max, targets.allow_self), (1, 2, true));

    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 2_026_073_101, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();

    let natural_skills = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, natural_skills).unwrap();
    for player in [owner, other] {
        run_effects(
            &mut tx,
            vec![EffectRequest::DiscardFromHand {
                player,
                count: u32::MAX,
            }],
        )
        .unwrap();
        let state = tx.state_mut().players.get_mut(player).unwrap();
        state.hp = 4;
        state.max_hp = 4;
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(JYLIANHUAN),
        }],
    )
    .unwrap();
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == JYLIANHUAN)
        .unwrap();
    (tx.state().clone(), owner, other, skill)
}

fn is_linked(state: &GameState, player: PlayerId) -> bool {
    state
        .players
        .get(player)
        .unwrap()
        .marks
        .iter()
        .any(|(mark, count)| *mark == engine_core::content::MARK_CHAIN && *count > 0)
}

fn activate(
    state: GameState,
    owner: PlayerId,
    other: PlayerId,
    skill: SkillId,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![owner, other],
                card_targets: vec![],
            },
        },
    )
}

#[test]
fn jylianhuan_loses_hp_then_links_up_to_two_targets_once_per_play_phase() {
    let (state, owner, other, skill) = setup();
    let (state, result) = activate(state, owner, other, skill, 1);
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{:?}",
        result.result
    );
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(owner).unwrap().hp, 3);
    assert!(is_linked(&state, owner));
    assert!(is_linked(&state, other));
    assert!(result.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardUsed {
            player,
            targets,
            ..
        } if *player == owner && targets == &[owner, other]
    )));

    let before = canonical_state_hash(&state);
    let (same, second) = activate(state, owner, other, skill, 2);
    assert!(matches!(second.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
}
