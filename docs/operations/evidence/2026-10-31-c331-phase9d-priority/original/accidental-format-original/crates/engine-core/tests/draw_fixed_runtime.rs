//! Runtime regression coverage for fixed Draw-phase counts and queued replacements.

use content_compiler::compile_source_named;
use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const FIRST_REPLACEMENT: u32 = 4_100_000_001;
const SECOND_REPLACEMENT: u32 = 4_100_000_002;
const STANDARD: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");
const SOURCE: &str = r#"
skill first_replacement(4100000001) {
  trigger phase_draw owner_is_target_draw_not_fixed_hp_one_or_hand_empty optional {
    effect replace_draw_gain_top_two_add_skills_if_different_colors(self, 16, 23);
  }
}

skill second_replacement(4100000002) {
  trigger phase_draw owner_is_target_draw_not_fixed_hp_one_or_hand_empty optional {
    effect replace_draw_gain_top_two_add_skills_if_different_colors(self, 16, 23);
  }
}
"#;

#[test]
fn first_queued_draw_replacement_suppresses_second_optional_prompt() {
    let standard = content_schema::ContentPackage::from_json_bytes(STANDARD).unwrap();
    let replacements = compile_source_named("queued-draw-replacements", SOURCE).unwrap();
    assert_eq!(replacements.manifest.rule_vm_abi_version, 300);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&standard)
        .unwrap();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&replacements)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 30_000, "identity-standard").unwrap();
    let owner = tx.state().players.alive_by_seat()[0];
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
            skill_def: SkillDefId::new(FIRST_REPLACEMENT),
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(SECOND_REPLACEMENT),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let first_instance = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.def == SkillDefId::new(FIRST_REPLACEMENT))
        .unwrap()
        .id;
    let second_instance = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.def == SkillDefId::new(SECOND_REPLACEMENT))
        .unwrap()
        .id;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Draw;
    tx.state_mut().players.get_mut(owner).unwrap().hp = 1;
    tx.state_mut().revision = 1;

    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let draw_cards = tx.state().zones.get(draw).unwrap().cards.clone();
    for card in &draw_cards[draw_cards.len() - 2..] {
        tx.state_mut().cards.get_mut(*card).unwrap().suit = Suit::Heart;
    }
    let hand_before = tx.state().hand_count(owner);

    let event = DomainEvent::TurnStateChanged {
        current: owner,
        phase: TurnPhaseTag::Draw,
    };
    assert_eq!(
        dispatch_for_events(&mut tx, &[event], &mut TriggerFireRecord::new()),
        Err(EffectError::ChoiceRequested)
    );
    let state = tx.commit().state;
    let (state, result) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );

    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.turn.draw_phase_fixed_count(), Some(0));
    assert_eq!(state.hand_count(owner), hand_before + 2);
    assert!(result.events.iter().any(
        |event| matches!(event, DomainEvent::RuleNote { code: 100, payload } if payload.first() == Some(&first_instance.raw()))
    ));
    assert!(!result.events.iter().any(
        |event| matches!(event, DomainEvent::RuleNote { code: 100, payload } if payload.first() == Some(&second_instance.raw()))
    ));
}
