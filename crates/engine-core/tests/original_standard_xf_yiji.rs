//! Golden coverage for Standard 伊籍 rewritten from the legacy JavaScript semantics.

use content_compiler::compile_source_named;
use engine_api::{
    ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const XF_YIJI: u32 = 4_192_370_611;
const JIJIE: u32 = 4_028_496_128;
const JIYUAN: u32 = 2_449_404_379;
const SOURCE: &str = include_str!("../../../content/standard/xinfu_jijie.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    other: PlayerId,
    jijie: SkillId,
}

fn setup() -> Setup {
    let package = compile_source_named("standard-xf-yiji-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_725_181, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let other = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(JIJIE),
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(JIYUAN),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let jijie = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == JIJIE)
        .unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        other,
        jijie,
    }
}

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

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled: false,
        },
        key,
    )
}

#[test]
fn package_closes_xf_yiji_with_both_exact_skills() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == XF_YIJI)
        .unwrap();
    assert_eq!(character.skills, [JIJIE, JIYUAN]);
    let jiyuan: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == JIYUAN)
        .collect();
    assert_eq!(jiyuan.len(), 2);
    assert!(jiyuan.iter().any(|trigger| {
        trigger.event == "cards_given"
            && trigger.filter == "owner_alive_gave_cards_to_other_alive"
            && trigger.optional
    }));
}

#[test]
fn jijie_reveals_exact_bottom_card_and_jiyuan_draws_for_recipient() {
    let setup = setup();
    let draw = setup
        .state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let bottom = setup.state.zones.get(draw).unwrap().cards[0];
    let before = setup.state.hand_count(setup.other);
    let (state, offered) = submit(
        setup.state,
        setup.owner,
        CommandKind::ActivateSkill {
            skill: setup.jijie,
            targets: Vec::new(),
            card_targets: Vec::new(),
        },
        1,
    );
    assert!(offered.choice.is_some(), "{offered:?}");
    let choice = offered.choice.unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectTargets);
    assert!(state.cards.get(bottom).is_some_and(|card| card.zone
        == state
            .zones
            .find(ZoneKind::Processing, PlayerId::NONE)
            .unwrap()));

    let (state, optional) = resolve(state, setup.owner, vec![setup.other.raw()], 2);
    assert_eq!(optional.choice.as_ref().unwrap().kind, ChoiceKind::Confirm);
    assert!(state.hand_cards(setup.other).contains(&bottom));
    let (state, done) = resolve(state, setup.owner, vec![1], 3);
    assert!(matches!(done.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.other), before + 2);
}

#[test]
fn jiyuan_may_draw_for_any_other_dying_player() {
    let setup = setup();
    let before = setup.state.hand_count(setup.other);
    let mut tx = Transaction::begin(setup.state);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::EnteredDying {
                player: setup.other,
                source: PlayerId::NONE,
            }],
            &mut TriggerFireRecord::new(),
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = tx.commit().state;
    let (state, done) = resolve(state, setup.owner, vec![1], 4);
    assert!(matches!(done.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.other), before + 1);
}
