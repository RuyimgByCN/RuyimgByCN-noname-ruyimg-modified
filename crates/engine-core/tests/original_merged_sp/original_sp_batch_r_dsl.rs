//! Golden for sp batch-R DSL rewrites:
//! 邢神 (olxingshen) damage → draw 1 + 「严教」mark cap 6;
//! 避乱·OL (olbiluan) own end phase → discard 1 → rebiluan2 + alive-group-count mark;
//! 避乱·新 (rebiluan) own end phase → discard 1 → rebiluan2 + min(4, alive) mark.

use engine_api::{
    ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const OLXINGSHEN: u32 = 2_814_304_936;
const OLBILUAN: u32 = 3_868_553_904;
const REBILUAN: u32 = 3_478_138_593;
const REBILUAN2: u32 = 3_075_740_841;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/sp/sp.npack.json");

fn started(seed: u64) -> (GameState, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let state = engine.state().clone();
    let owner = state.turn.current;
    (state, owner)
}

fn with_skill(state: GameState, owner: PlayerId, def: u32) -> GameState {
    let mut tx = Transaction::begin(state);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(def),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

fn mark_of(state: &GameState, player: PlayerId, mark: u32) -> Option<i32> {
    state
        .players
        .get(player)
        .unwrap()
        .marks
        .iter()
        .find(|(id, _)| *id == mark)
        .map(|(_, count)| *count)
}

fn has_skill(state: &GameState, owner: PlayerId, def: u32) -> bool {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .any(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == def)
        })
}

#[test]
fn package_publishes_batch_r_trigger_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    let triggers: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| matches!(trigger.skill_def_id, OLXINGSHEN | OLBILUAN | REBILUAN))
        .map(|trigger| {
            (
                trigger.skill_def_id,
                trigger.event.as_str(),
                trigger.filter.as_str(),
                trigger.optional,
            )
        })
        .collect();
    assert!(triggers.contains(&(OLXINGSHEN, "damage", "owner_is_target", false)));
    assert!(triggers.contains(&(
        OLBILUAN,
        "phase_end",
        "owner_is_target_phase_end_hand_equip_nonempty_other_alive",
        true
    )));
    assert!(triggers.contains(&(
        REBILUAN,
        "phase_end",
        "owner_is_target_phase_end_hand_equip_nonempty_other_alive",
        true
    )));
    assert!(!package
        .modifier_index
        .iter()
        .any(|modifier| { matches!(modifier.skill_def_id, OLXINGSHEN | OLBILUAN | REBILUAN) }));
}

#[test]
fn olxingshen_draws_one_and_marks_lost_hp_capped_at_six() {
    let (state, owner) = started(2_026_081_201);
    let source = state
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let state = with_skill(state, owner, OLXINGSHEN);
    let before = state.hand_count(owner);
    let max_hp = state.players.get(owner).unwrap().max_hp;
    let hp = state.players.get(owner).unwrap().hp;
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::DamageDealt {
            source,
            target: owner,
            amount: 1,
            nature: 0,
            is_lose_hp: false,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    let state = tx.commit().state;
    assert_eq!(state.hand_count(owner), before + 1);
    let lost = max_hp as i32 - hp;
    assert_eq!(mark_of(&state, owner, OLXINGSHEN), Some(lost.min(6)));
    // The synthetic DamageDealt dispatch only runs the trigger chain; the real
    // damage application is a separate engine step, so hp is untouched here.
    assert_eq!(state.players.get(owner).unwrap().hp, hp);
}

#[test]
fn olbiluan_optionally_discards_and_marks_alive_group_count() {
    let (state, owner) = started(2_026_081_202);
    let state = with_skill(state, owner, OLBILUAN);
    let before = state.hand_count(owner);
    let hand_card = state.hand_cards(owner)[0];

    let mut tx = Transaction::begin(state);
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();

    let (confirmed, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(
        confirmed.resolution.continuation_kind,
        ContinuationKind::Custom
    );
    assert_eq!(
        accepted.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );

    let (state, discarded) = resolve(confirmed, owner, vec![hand_card.raw()], false, 2);
    assert!(matches!(discarded.result, CommandResult::Accepted { .. }));
    assert!(has_skill(&state, owner, REBILUAN2));
    let groups = engine_core::query::alive_group_count(&state);
    assert_eq!(mark_of(&state, owner, REBILUAN2), Some(groups));
    assert_eq!(state.hand_count(owner), before - 1);
    assert_eq!(
        state.cards.get(hand_card).unwrap().zone,
        state
            .zones
            .find(engine_core::state::ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
}

#[test]
fn rebiluan_optionally_discards_and_marks_min_four_alive() {
    let (state, owner) = started(2_026_081_203);
    let state = with_skill(state, owner, REBILUAN);
    let before = state.hand_count(owner);
    let hand_card = state.hand_cards(owner)[0];

    let mut tx = Transaction::begin(state);
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();

    let (confirmed, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(
        accepted.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );

    let (state, discarded) = resolve(confirmed, owner, vec![hand_card.raw()], false, 2);
    assert!(matches!(discarded.result, CommandResult::Accepted { .. }));
    assert!(has_skill(&state, owner, REBILUAN2));
    assert_eq!(mark_of(&state, owner, REBILUAN2), Some(2));
    assert_eq!(state.hand_count(owner), before - 1);
}

#[test]
fn olbiluan_can_be_cancelled_without_discard_or_mark() {
    let (state, owner) = started(2_026_081_204);
    let state = with_skill(state, owner, OLBILUAN);
    let before = state.hand_count(owner);

    let mut tx = Transaction::begin(state);
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    let tx = tx.commit();

    let (state, cancelled) = resolve(tx.state, owner, vec![], true, 1);
    assert!(matches!(cancelled.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), before);
    assert!(!has_skill(&state, owner, REBILUAN2));
    assert!(mark_of(&state, owner, REBILUAN2).is_none());
}
