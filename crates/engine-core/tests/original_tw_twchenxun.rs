//! Golden coverage for TW 幻刘封「陈勋」.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{CardKind, GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const TWCHENXUN: u32 = 4_060_298_774;
const HUAN_LIUFENG: u32 = 3_034_906_798;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twchenxun.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/tw/tw.npack.json");

fn setup(seed: u64, responder_has_sha: bool) -> (Transaction, [PlayerId; 3], Option<u32>) {
    let package = content_compiler::compile_source_named("twchenxun-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 397);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    let players: [PlayerId; 3] = tx.state().players.alive_by_seat().try_into().unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(TWCHENXUN),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in players {
        for card in tx.state().hand_cards(player) {
            tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Other;
        }
    }
    let response = responder_has_sha.then(|| {
        let card = tx.state().hand_cards(players[1])[0];
        tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Sha;
        card.raw()
    });
    tx.state_mut().turn.current = players[0];
    tx.state_mut().turn.phase = Phase::Prepare;
    tx.state_mut().turn.round = 1;
    (tx, players, response)
}

fn fire(tx: &mut Transaction, owner: PlayerId) -> Result<(), EffectError> {
    let effects = dispatch_for_events(
        tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    )?;
    run_effects(tx, effects).map(|_| ())
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

#[test]
fn chains_only_after_target_damage_and_can_be_cancelled() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(
        committed
            .character_index
            .iter()
            .find(|character| character.id == HUAN_LIUFENG)
            .unwrap()
            .skills,
        vec![TWCHENXUN]
    );

    let (mut tx, [owner, first, second], _) = setup(2_026_080_201, false);
    let owner_hand = tx.state().hand_count(owner);
    assert_eq!(fire(&mut tx, owner), Err(EffectError::ChoiceRequested));
    let (state, result) = resolve(tx.commit().state, owner, vec![first.raw()], false, 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), owner_hand + 1);
    assert_eq!(state.resolution.continuation_payload[4..], [second.raw()]);
    let (state, result) = resolve(state, owner, vec![second.raw()], false, 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), owner_hand + 2);
    assert!(!state.has_pending_choice());

    let (mut tx, [owner, target, _], response) = setup(2_026_080_202, true);
    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    assert_eq!(fire(&mut tx, owner), Err(EffectError::ChoiceRequested));
    let (state, _) = resolve(tx.commit().state, owner, vec![target.raw()], false, 3);
    let (state, result) = resolve(state, target, vec![response.unwrap()], false, 4);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp - 2);
    assert!(!state.has_pending_choice());

    let (mut tx, [owner, _, _], _) = setup(2_026_080_203, false);
    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    assert_eq!(fire(&mut tx, owner), Err(EffectError::ChoiceRequested));
    let (state, result) = resolve(tx.commit().state, owner, Vec::new(), true, 5);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp);
    assert!(!state.has_pending_choice());
}
