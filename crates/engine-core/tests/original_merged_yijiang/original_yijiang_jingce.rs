//! Golden for 一将成名「精策」用牌历史语义。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const JINGCE: u32 = 2_443_825_940;
const XINJINGCE: u32 = 4_267_063_918;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/yijiang/yijiang.npack.json");

fn at_phase_with_used_cards(skill: u32, used: usize) -> (Transaction, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|entry| entry.skill_def_id == skill && !entry.locked));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(skill as u64 + used as u64),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_714 + used as u64,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    let mut setup: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|instance| EffectRequest::RemoveSkill { skill: instance.id })
        .collect();
    setup.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill),
    });
    setup.push(EffectRequest::SetTurnState {
        current: owner,
        phase: Phase::Play.to_u8(),
    });
    run_effects(&mut tx, setup).unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().hp = 2;

    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, owner)
        .and_then(|zone| tx.state().zones.get(zone))
        .unwrap()
        .cards
        .clone();
    assert!(hand.len() >= used);
    for card in hand.into_iter().take(used) {
        run_effects(
            &mut tx,
            vec![EffectRequest::UseCard {
                player: owner,
                card,
                targets: vec![],
            }],
        )
        .unwrap();
    }
    (tx, owner)
}

fn assert_threshold(skill: u32, trigger_phase: Phase) {
    let (mut below, owner) = at_phase_with_used_cards(skill, 1);
    let hand = below.state().hand_count(owner);
    run_effects(
        &mut below,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: trigger_phase.to_u8(),
        }],
    )
    .unwrap();
    assert_eq!(below.state().hand_count(owner), hand);
    assert!(!below.state().has_pending_choice());

    let (mut enough, owner) = at_phase_with_used_cards(skill, 2);
    let hand = enough.state().hand_count(owner);
    let paused = run_effects(
        &mut enough,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: trigger_phase.to_u8(),
        }],
    )
    .unwrap_err();
    assert_eq!(paused, EffectError::ChoiceRequested);

    let (state, result) = apply_command(
        enough.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(skill as u64),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(enough.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand + 2);
}

#[test]
fn jingce_checks_total_cards_used_when_play_phase_ends() {
    assert_threshold(JINGCE, Phase::Discard);
}

#[test]
fn xinjingce_checks_the_same_turn_history_when_end_phase_begins() {
    assert_threshold(XINJINGCE, Phase::End);
}
