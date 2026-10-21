//! Golden coverage for old「集军」rewritten from legacy JavaScript semantics.

use engine_api::{
    Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    MatchOutcome, PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const OLD_JIJUN: u32 = 2_612_231_702;
const OLD_FANGTONG: u32 = 2_714_288_558;
const SOURCE: &str = include_str!("../../../../content/original/ported/old/old_jijun.skill");
const PAIR_SOURCE: &str = concat!(
    include_str!("../../../../content/original/ported/old/old_jijun.skill"),
    include_str!("../../../../content/original/ported/old/old_fangtong.skill"),
);
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/old/old.npack.json");

#[test]
fn selected_hand_cards_accumulate_in_the_skill_pile() {
    let package = content_compiler::compile_source_named("old-jijun-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 2_026_072_805, "identity-standard").unwrap();
    let owner = tx.state().players.alive_by_seat()[0];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLD_JIJUN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = tx.state().players.get(owner).unwrap().skills[0];
    let cards = tx.state().hand_cards(owner);
    let state = tx.commit().state;

    let revision = state.revision;
    let (state, first) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![],
                card_targets: cards[..2].to_vec(),
            },
        },
    );
    assert!(matches!(first.result, CommandResult::Accepted { .. }));
    assert_eq!(state.skill_pile_cards(owner, OLD_JIJUN), cards[..2]);
    let pile = state
        .skill_piles
        .iter()
        .find(|pile| pile.owner == owner && pile.skill_def == OLD_JIJUN)
        .unwrap();
    assert_eq!(
        state.zones.get(pile.zone).unwrap().kind,
        ZoneKind::Expansion
    );

    let revision = state.revision;
    let (state, second) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![],
                card_targets: vec![cards[2]],
            },
        },
    );
    assert!(matches!(second.result, CommandResult::Accepted { .. }));
    assert_eq!(state.skill_pile_cards(owner, OLD_JIJUN).len(), 3);
}

#[test]
fn package_publishes_old_jijun_without_javascript() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 275);
    for (def, name) in [(OLD_JIJUN, "old_jijun"), (OLD_FANGTONG, "old_fangtong")] {
        assert!(package
            .skill_index
            .iter()
            .any(|skill| skill.skill_def_id == def && skill.name == name));
    }
    assert!(!serde_json::to_string(&package).unwrap().contains(".js"));
}

#[test]
fn thirty_six_fang_end_the_match_for_the_owners_camp() {
    let package = content_compiler::compile_source_named("old-fangtong-test", PAIR_SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 2_026_072_806, "identity-standard").unwrap();
    let owner = tx.state().players.alive_by_seat()[0];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for skill_def in [OLD_JIJUN, OLD_FANGTONG] {
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill_def),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let jijun = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == OLD_JIJUN)
        .unwrap();
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let need = 36usize.saturating_sub(tx.state().hand_count(owner) as usize);
    let cards = tx.state().zones.get(draw).unwrap().cards[..need].to_vec();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards,
            to: hand,
            from: Some(draw),
        }],
    )
    .unwrap();
    let cards = tx.state().hand_cards(owner);
    let state = tx.commit().state;
    let revision = state.revision;
    let (state, stored) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(3),
            kind: CommandKind::ActivateSkill {
                skill: jijun,
                targets: vec![],
                card_targets: cards,
            },
        },
    );
    assert!(matches!(stored.result, CommandResult::Accepted { .. }));
    assert_eq!(state.skill_pile_cards(owner, OLD_JIJUN).len(), 36);

    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    let events = run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().outcome, MatchOutcome::LoyalistWin);
    assert!(events.iter().any(|event| matches!(
        event,
        DomainEvent::MatchEnded {
            outcome: MatchOutcome::LoyalistWin
        }
    )));
}
