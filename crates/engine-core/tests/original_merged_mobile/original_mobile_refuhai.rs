//! Golden coverage for MOBILE 卫温诸葛直「浮海」.

use engine_api::{
    ChoiceId, Command, CommandKind, DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const REFUHAI: u32 = 2_797_180_006;
const CHARACTER: u32 = 3_812_335_636;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/mobile/mobile.npack.json");

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn choose(state: GameState, actor: PlayerId, choice: u32, key: u64) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections: vec![choice],
            cancelled: false,
        },
        key,
    )
}

#[test]
fn package_publishes_character_closure() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 493);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.id == CHARACTER)
            .unwrap()
            .skills,
        [REFUHAI]
    );
}

#[test]
fn private_choices_reveal_together_and_draw_initial_equal_run() {
    let package = content_compiler::compile_source_named(
        "refuhai-test",
        include_str!("../../../../content/original/ported/mobile/refuhai.skill"),
    )
    .unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, 2_026_080_201, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(REFUHAI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = tx.state().players.get(owner).unwrap().skills[0];
    let before = tx.state().hand_count(owner);
    let (state, activated) = submit(
        tx.commit().state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: Vec::new(),
            card_targets: Vec::new(),
        },
        1,
    );
    assert_eq!(activated.choice.unwrap().observer, players[1]);
    let (state, first) = choose(state, players[1], 1, 2);
    assert_eq!(first.choice.unwrap().observer, players[2]);
    assert!(!first.events.iter().any(|event| matches!(
        event,
        DomainEvent::RuleNote {
            code: 0x4930_0001,
            ..
        }
    )));
    let (state, second) = choose(state, players[2], 1, 3);
    assert_eq!(second.choice.unwrap().observer, players[3]);
    let (state, final_result) = choose(state, players[3], 2, 4);
    assert!(final_result.events.iter().any(|event| matches!(
        event,
        DomainEvent::RuleNote {
            code: 0x4930_0001,
            ..
        }
    )));
    assert_eq!(state.hand_count(owner), before + 2);
}
