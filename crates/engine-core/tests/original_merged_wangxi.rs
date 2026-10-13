//! O2 全量合并第二批（2026-10-14）：家族 wangxi 的 1 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_wangxi_family {
//! Golden coverage for the three legacy「忘隙」implementations.

use content_compiler::compile_source_named;
use engine_api::{
    CardMoveReason, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/refresh/wangxi.skill"),
    include_str!("../../../content/original/ported/refresh/xinwangxi.skill"),
    include_str!("../../../content/original/ported/diy/junkwangxi.skill"),
    r#"
skill draw_followup(999) locked {
  trigger cards_drawn owner_gains_multiple_outside_own_draw {
    effect draw_cards(self, 1);
  }
}
"#,
);
const WANGXI: u32 = 3_936_329_432;
const XINWANGXI: u32 = 4_161_732_730;
const JUNKWANGXI: u32 = 2_370_802_609;
const REFRESH: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

fn setup(skill: u32) -> (GameState, PlayerId, PlayerId) {
    let package = compile_source_named("wangxi-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 20_260_715, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let other = players[1];
    let old_skills: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| skill.id)
        .collect();
    let mut effects: Vec<_> = old_skills
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, other)
}

fn damage_choice(
    state: GameState,
    source: PlayerId,
    target: PlayerId,
    amount: u32,
) -> (GameState, engine_api::ChoiceRequest) {
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source,
                target,
                amount,
                is_lose_hp: false,
                nature: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let committed = tx.commit();
    (committed.state, committed.output.choice.unwrap())
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: choice,
                selections,
                cancelled: false,
            },
        },
    )
}

#[test]
fn wangxi_asks_once_per_damage_point_and_each_side_draws_once_when_accepted() {
    let (state, owner, other) = setup(WANGXI);
    let owner_hand = state.hand_count(owner);
    let other_hand = state.hand_count(other);
    let (state, _) = damage_choice(state, other, owner, 2);

    let (state, first) = resolve(state, owner, vec![0], 1);
    assert!(matches!(first.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let (state, second) = resolve(state, owner, vec![1], 2);
    assert!(matches!(second.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(owner), owner_hand + 1);
    assert_eq!(state.hand_count(other), other_hand + 1);
}

#[test]
fn xinwangxi_can_give_any_existing_hand_or_equipment_card_after_drawing_two() {
    let (state, owner, other) = setup(XINWANGXI);
    let old_card = state.hand_cards(owner)[0];
    let owner_hand = state.hand_count(owner);
    let other_hand = state.hand_count(other);
    let (state, _) = damage_choice(state, owner, other, 1);
    let (state, accepted) = resolve(state, owner, vec![1], 3);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::GiveOneHandOrEquip
    );
    assert!(accepted
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .any(|option| option.token == old_card.raw()));
    assert_eq!(state.hand_count(owner), owner_hand + 2);

    let (state, given) = resolve(state, owner, vec![old_card.raw()], 4);
    assert!(matches!(given.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(owner), owner_hand + 1);
    assert_eq!(state.hand_count(other), other_hand + 1);
    assert_eq!(state.cards.get(old_card).unwrap().owner, other);
    assert!(given.events.iter().any(|event| matches!(
        event,
        engine_api::DomainEvent::CardsMoved {
            cards,
            reason: CardMoveReason::Give,
            ..
        } if cards == &vec![old_card]
    )));
}

#[test]
fn junkwangxi_only_allows_cards_drawn_by_that_activation() {
    let (state, owner, other) = setup(JUNKWANGXI);
    let old_card = state.hand_cards(owner)[0];
    let owner_hand = state.hand_count(owner);
    let other_hand = state.hand_count(other);
    let (state, _) = damage_choice(state, other, owner, 1);
    let (state, accepted) = resolve(state, owner, vec![1], 5);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    let offered: Vec<_> = accepted
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .map(|option| option.token)
        .collect();
    assert_eq!(offered.len(), 2);
    assert!(!offered.contains(&old_card.raw()));

    let (state, given) = resolve(state, owner, vec![offered[0]], 6);
    assert!(matches!(given.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), owner_hand + 1);
    assert_eq!(state.hand_count(other), other_hand + 1);
}

#[test]
fn drawn_only_scope_excludes_cards_from_triggers_caused_by_the_draw() {
    let (state, owner, other) = setup(JUNKWANGXI);
    let owner_hand = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(999),
            },
            EffectRequest::SetTurnState {
                current: owner,
                phase: engine_core::state::Phase::Play.to_u8(),
            },
        ],
    )
    .unwrap();
    let (state, _) = damage_choice(tx.commit().state, owner, other, 1);
    let (state, accepted) = resolve(state, owner, vec![1], 7);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.hand_count(owner),
        owner_hand + 3,
        "the draw trigger resolves before the give choice"
    );
    assert_eq!(accepted.choice.as_ref().unwrap().options.len(), 2);
}

#[test]
fn self_damage_never_offers_wangxi() {
    let (state, owner, _) = setup(WANGXI);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());
}
}
