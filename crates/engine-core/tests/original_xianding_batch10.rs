//! Golden for XIANDING（限定包）批次十：
//! 敏善（dcminshan）、受责（dcshouze）。

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCMINSHAN: u32 = 2_275_391_703;
const DCSHOUZE: u32 = 3_027_710_363;
const DCLINGFANG: u32 = 3_884_646_990;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, first, second) = (players[0], players[1], players[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, first, second)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

fn deal_damage(
    state: GameState,
    source: PlayerId,
    target: PlayerId,
) -> (GameState, Result<Vec<engine_api::DomainEvent>, EffectError>) {
    let mut tx = Transaction::begin(state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
            card: CardId::NONE,
            card_kind: 0,
            card_flags: 0,
        }],
    );
    (tx.commit().state, result)
}

#[test]
fn package_exposes_batch10_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let minshan = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCMINSHAN)
        .expect("dcminshan trigger");
    assert_eq!(minshan.event, "damage");
    assert_eq!(minshan.filter, "owner_is_target_per_damage_point");
    assert!(minshan.optional);

    let shouze = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCSHOUZE)
        .expect("dcshouze trigger");
    assert_eq!(shouze.event, "phase_end");
    assert_eq!(shouze.filter, "owner_is_target_mark_positive_3884646990");
    assert!(!shouze.optional);
}

#[test]
fn dcminshan_offers_other_player_two_card_draw_after_damage() {
    let (mut tx, owner, first, second) = setup(20_260_824);
    add_skill(&mut tx, owner, DCMINSHAN);
    tx.state_mut().players.get_mut(owner).unwrap().hp -= 1;
    let before = tx.state().hand_count(second);
    let (state, result) = deal_damage(tx.state().clone(), first, owner);
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let state = resolve(state, owner, vec![1], false, 1);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseTargetDraw
    );
    let state = resolve(state, owner, vec![second.raw()], false, 2);
    assert_eq!(state.hand_count(second), before + 2);
}

#[test]
fn dcshouze_consumes_mark_loses_hp_and_gains_sha_at_phase_end() {
    let (mut tx, owner, _first, _second) = setup(20_260_825);
    add_skill(&mut tx, owner, DCSHOUZE);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddMark {
            player: owner,
            mark: engine_api::MarkDefId::new(DCLINGFANG),
            delta: 1,
        }],
    )
    .unwrap();
    let hp_before = tx.state().players.get(owner).unwrap().hp;
    let discard = tx
        .state()
        .zones
        .find(
            engine_core::state::ZoneKind::DiscardPile,
            engine_api::PlayerId::NONE,
        )
        .unwrap();
    let sha = tx.state_mut().cards.alloc(engine_core::state::Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(1),
        kind: engine_core::state::CardKind::Sha,
        suit: engine_api::Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: discard,
        owner: engine_api::PlayerId::NONE,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(discard)
        .unwrap()
        .cards
        .push(sha);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::End;
    let effects = dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    let err = run_effects(&mut tx, effects);
    assert!(matches!(err, Err(EffectError::ChoiceRequested)));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::ChooseGainDiscardCardByKind
    );
    let state = resolve(tx.state().clone(), owner, vec![sha.raw()], false, 1);
    assert_eq!(state.players.get(owner).unwrap().hp, hp_before - 1);
    assert_eq!(state.hand_count(owner), 1);
}
