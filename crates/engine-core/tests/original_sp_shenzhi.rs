//! Golden for SP 甘夫人「神智」rewritten from the legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const SHENZHI: u32 = 2_944_037_719;
const STDSHUSHEN: u32 = 3_504_783_479;
const GANFUREN: u32 = 3_981_632_700;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn isolated(hand: u32, hp: i32) -> (Transaction, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(SHENZHI as u64),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: SHENZHI as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));

    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([
        EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        },
        EffectRequest::DrawCards {
            player: owner,
            count: hand,
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(SHENZHI),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().hp = hp;
    assert_eq!(tx.state().hand_count(owner), hand);
    (tx, owner)
}

fn offer(tx: &mut Transaction, owner: PlayerId) -> Result<Vec<EffectRequest>, EffectError> {
    dispatch_for_events(
        tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    )
}

fn resolve(
    tx: &Transaction,
    owner: PlayerId,
    accept: bool,
    key: u64,
) -> engine_core::state::GameState {
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![u32::from(accept)],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn committed_package_exposes_optional_nonempty_prepare_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.dependencies.iter().any(|dependency| {
        dependency.namespace == "official"
            && dependency.package_id == "standard"
            && dependency.version == "0.1.0"
    }));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == GANFUREN)
        .unwrap();
    assert_eq!(character.name_key, "ganfuren");
    assert_eq!(character.skills, vec![STDSHUSHEN, SHENZHI]);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == SHENZHI)
        .unwrap();
    assert_eq!(
        (
            trigger.event.as_str(),
            trigger.filter.as_str(),
            trigger.optional,
        ),
        ("phase_prepare", "owner_is_target_hand_nonempty", true)
    );
}

#[test]
fn accepting_discards_every_hand_card_and_recovers_when_count_reaches_hp() {
    let (mut tx, owner) = isolated(3, 2);
    assert_eq!(offer(&mut tx, owner), Err(EffectError::ChoiceRequested));
    let state = resolve(&tx, owner, true, 1);
    assert_eq!(state.hand_count(owner), 0);
    assert_eq!(state.players.get(owner).unwrap().hp, 3);

    let (mut no_recover, owner) = isolated(1, 2);
    assert_eq!(
        offer(&mut no_recover, owner),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(&no_recover, owner, true, 2);
    assert_eq!(state.hand_count(owner), 0);
    assert_eq!(state.players.get(owner).unwrap().hp, 2);
}

#[test]
fn declining_preserves_cards_and_empty_hand_creates_no_choice() {
    let (mut declined, owner) = isolated(2, 2);
    assert_eq!(
        offer(&mut declined, owner),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(&declined, owner, false, 3);
    assert_eq!(state.hand_count(owner), 2);
    assert_eq!(state.players.get(owner).unwrap().hp, 2);

    let (mut empty, owner) = isolated(0, 2);
    assert!(offer(&mut empty, owner).unwrap().is_empty());
    assert!(!empty.state().has_pending_choice());
}
