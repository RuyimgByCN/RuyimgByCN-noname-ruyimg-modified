//! Golden coverage for REFRESH「完杀」.

use engine_api::{DomainEvent, MarkDefId, PlayerId, SkillDefId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const REWANSHA: u32 = 3_382_466_194;
const JILUE_WANSHA: u32 = 3_522_766_701;
const PROBE: u32 = 1_001;
const SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/refresh/rewansha.skill"),
    "skill probe(1001) {}\n",
);
const JILUE_SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/extra/shen_simayi.skill"),
    "skill probe(1001) {}\n",
);

fn dispatch(tx: &mut Transaction, event: DomainEvent) {
    let effects = dispatch_for_events(tx, &[event], &mut TriggerFireRecord::new()).unwrap();
    run_effects(tx, effects).unwrap();
}

#[test]
fn suppresses_only_non_dying_others_during_the_dying_window() {
    let package = content_compiler::compile_source_named("rewansha", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 20_260_731, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let owner = tx.state().turn.current;
    let others: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
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
            skill_def: SkillDefId::new(REWANSHA),
        },
        EffectRequest::AddSkill {
            player: others[0],
            skill_def: SkillDefId::new(PROBE),
        },
        EffectRequest::AddSkill {
            player: others[1],
            skill_def: SkillDefId::new(PROBE),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().players.get_mut(others[1]).unwrap().dying = true;

    dispatch(
        &mut tx,
        DomainEvent::EnteredDying {
            player: others[1],
            source: owner,
        },
    );
    let probe = |tx: &Transaction, player: PlayerId| {
        let skill = tx
            .state()
            .players
            .get(player)
            .unwrap()
            .skills
            .iter()
            .find_map(|skill| {
                tx.state()
                    .skills
                    .get(*skill)
                    .filter(|skill| skill.def.raw() == PROBE)
            })
            .unwrap();
        tx.state().skill_effectively_enabled(skill)
    };
    assert!(!probe(&tx, others[0]));
    assert!(probe(&tx, others[1]));

    tx.state_mut().players.get_mut(others[1]).unwrap().dying = false;
    dispatch(&mut tx, DomainEvent::LeftDying { player: others[1] });
    assert!(probe(&tx, others[0]));
}

#[test]
fn jilue_wansha_suppresses_only_while_its_activation_mark_exists() {
    let package = content_compiler::compile_source_named("jilue-wansha", JILUE_SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 20_260_731, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let owner = tx.state().turn.current;
    let others: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let old: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, old).unwrap();
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(JILUE_WANSHA),
            },
            EffectRequest::AddSkill {
                player: others[0],
                skill_def: SkillDefId::new(PROBE),
            },
        ],
    )
    .unwrap();
    let probe_enabled = |tx: &Transaction| {
        let skill = tx
            .state()
            .players
            .get(others[0])
            .unwrap()
            .skills
            .iter()
            .find_map(|skill| {
                tx.state()
                    .skills
                    .get(*skill)
                    .filter(|skill| skill.def.raw() == PROBE)
            })
            .unwrap();
        tx.state().skill_effectively_enabled(skill)
    };
    tx.state_mut().players.get_mut(others[1]).unwrap().dying = true;
    dispatch(
        &mut tx,
        DomainEvent::EnteredDying {
            player: others[1],
            source: owner,
        },
    );
    assert!(probe_enabled(&tx));

    run_effects(
        &mut tx,
        vec![EffectRequest::AddMark {
            player: owner,
            mark: MarkDefId::new(JILUE_WANSHA),
            delta: 1,
        }],
    )
    .unwrap();
    dispatch(
        &mut tx,
        DomainEvent::EnteredDying {
            player: others[1],
            source: owner,
        },
    );
    assert!(!probe_enabled(&tx));

    tx.state_mut().players.get_mut(others[1]).unwrap().dying = false;
    dispatch(&mut tx, DomainEvent::LeftDying { player: others[1] });
    assert!(probe_enabled(&tx));
}
