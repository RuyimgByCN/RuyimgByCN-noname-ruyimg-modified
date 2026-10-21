//! Golden tests for 神话再临「怀柔」rewritten from the legacy JavaScript semantics.

use engine_api::{
    CardMoveReason, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::run_effects;
use engine_core::hash::canonical_state_hash;
use engine_core::state::{CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const DRLT_HUAIROU: u32 = 3_699_262_389;
const SOURCE: &str = include_str!("../../../../content/original/ported/shenhua/drlt_huairou.skill");
const SHENHUA: &[u8] =
    include_bytes!("../../../../content/original/ported/shenhua/shenhua.npack.json");

fn started() -> (GameState, PlayerId, SkillId) {
    let package = content_compiler::compile_source_named("drlt_huairou_test", SOURCE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_718),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 20_260_718,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    tx.state_mut().turn.phase = Phase::Play;
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(DRLT_HUAIROU),
    });
    run_effects(&mut tx, effects).unwrap();
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == DRLT_HUAIROU)
        .unwrap();
    (tx.commit().state, owner, skill)
}

fn activate(
    state: GameState,
    owner: PlayerId,
    skill: SkillId,
    cards: Vec<engine_api::CardId>,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![],
                card_targets: cards,
            },
        },
    )
}

#[test]
fn committed_package_exposes_the_typed_equipment_recast() {
    let committed = content_schema::ContentPackage::from_json_bytes(SHENHUA).unwrap();
    let compiled = content_compiler::compile_source_named("drlt_huairou", SOURCE).unwrap();
    let active = compiled
        .active_index
        .iter()
        .find(|active| active.skill_def_id == DRLT_HUAIROU)
        .unwrap();
    let selection = active.selection.as_ref().unwrap();
    assert_eq!(selection.zone, "hand_equip");
    assert_eq!((selection.min, selection.max), (1, 1));
    assert_eq!(selection.filter.as_deref(), Some("card_equip"));
    let expected = &compiled.modules[0];
    let actual = committed
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);
}

#[test]
fn equipment_recast_rejects_other_cards_then_draws_one_replacement() {
    let (mut state, owner, skill) = started();
    let hand = state.hand_cards(owner);
    assert!(hand.len() >= 2);
    let equipment = hand[0];
    let ordinary = hand[1];
    state.cards.get_mut(equipment).unwrap().kind = CardKind::Weapon;
    state.cards.get_mut(ordinary).unwrap().kind = CardKind::Other;

    let before = canonical_state_hash(&state);
    let (same, rejected) = activate(state.clone(), owner, skill, vec![ordinary], 1);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);

    let hand_before = state.hand_count(owner);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let (state, result) = activate(state, owner, skill, vec![equipment], 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand_before);
    assert_eq!(state.cards.get(equipment).unwrap().zone, discard);
    assert_eq!(state.skills.get(skill).unwrap().use_count, 1);
    assert!(result.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardsMoved {
            cards,
            to,
            reason: CardMoveReason::Recast,
            ..
        } if cards == &vec![equipment] && *to == discard
    )));
    assert!(result.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardsDrawn { player, cards } if *player == owner && cards.len() == 1
    )));
}
