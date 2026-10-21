//! Golden coverage for 标鲍信「募讨」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const STDMUTAO: u32 = 3_513_095_743;
const SOURCE: &str = include_str!("../../../../content/original/ported/sixiang/stdmutao.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/sixiang/sixiang.npack.json");

struct Setup {
    tx: Transaction,
    owner: PlayerId,
    sha_target: PlayerId,
    other_target: PlayerId,
    sha: CardId,
    other: CardId,
}

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn inject_hand(state: &mut GameState, owner: PlayerId, kind: CardKind, def: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Heart,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn setup() -> Setup {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package())
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 2_026_072_410, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let sha_target = players[1];
    let other_target = players[2];
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(STDMUTAO),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    let sha = inject_hand(tx.state_mut(), sha_target, CardKind::Sha, 137_001);
    let other = inject_hand(tx.state_mut(), other_target, CardKind::Shan, 137_002);
    Setup {
        tx,
        owner,
        sha_target,
        other_target,
        sha,
        other,
    }
}

fn phase_prepare(player: PlayerId) -> DomainEvent {
    DomainEvent::TurnStateChanged {
        current: player,
        phase: TurnPhaseTag::Prepare,
    }
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    )
}

fn trigger(setup: Setup) -> (GameState, PipelineResult) {
    let mut tx = setup.tx;
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[phase_prepare(setup.owner)],
            &mut TriggerFireRecord::new(),
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let committed = tx.commit();
    assert_eq!(
        committed.output.choice.as_ref().unwrap().kind,
        ChoiceKind::Confirm
    );
    resolve(committed.state, setup.owner, vec![1], 1)
}

#[test]
fn package_publishes_abi138_with_stdmutao_and_baoxin() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert_eq!(package.skill_index.len(), 181);
    assert_eq!(package.character_index.len(), 101);
    assert!(package.skill_index.iter().any(|skill| {
        skill.skill_def_id == STDMUTAO && skill.name == "stdmutao" && !skill.locked
    }));
    assert!(package
        .character_index
        .iter()
        .any(|character| character.name_key == "std_baoxin"));

    let compiled = content_compiler::compile_source_named("stdmutao", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 137);
    let expected = &compiled.modules[0];
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);
}

#[test]
fn selected_sha_hand_is_publicly_revealed_before_owner_damage() {
    let setup = setup();
    let owner = setup.owner;
    let target = setup.sha_target;
    let other_target = setup.other_target;
    let sha = setup.sha;
    let before_hp = setup.tx.state().players.get(target).unwrap().hp;
    let (selecting, choice_output) = trigger(setup);
    assert!(matches!(
        choice_output.result,
        CommandResult::Accepted { .. }
    ));
    let choice = choice_output.choice.as_ref().unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectTargets);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [target.raw(), other_target.raw()]
    );

    let (done, output) = resolve(selecting, owner, vec![target.raw()], 2);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(done.players.get(target).unwrap().hp, before_hp - 1);
    let revealed = output
        .events
        .iter()
        .position(|event| {
            matches!(event, DomainEvent::CardsRevealed { cards, to } if cards == &[sha] && to.is_empty())
        })
        .unwrap();
    let damaged = output
        .events
        .iter()
        .position(|event| {
            matches!(
                event,
                DomainEvent::DamageDealt {
                    source,
                    target: damaged,
                    amount: 1,
                    ..
                } if *source == owner && *damaged == target
            )
        })
        .unwrap();
    assert!(revealed < damaged);
}

#[test]
fn non_sha_hand_is_revealed_without_damage_and_empty_table_does_not_trigger() {
    let setup = setup();
    let owner = setup.owner;
    let target = setup.other_target;
    let other = setup.other;
    let before_hp = setup.tx.state().players.get(target).unwrap().hp;
    let (selecting, _) = trigger(setup);
    let (done, output) = resolve(selecting, owner, vec![target.raw()], 3);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(done.players.get(target).unwrap().hp, before_hp);
    assert!(output.events.iter().any(
        |event| matches!(event, DomainEvent::CardsRevealed { cards, to } if cards == &[other] && to.is_empty())
    ));
    assert!(!output.events.iter().any(
        |event| matches!(event, DomainEvent::DamageDealt { target: damaged, .. } if *damaged == target)
    ));

    let mut tx = Transaction::begin(done);
    let hands: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .map(|player| EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        })
        .collect();
    run_effects(&mut tx, hands).unwrap();
    let effects = dispatch_for_events(
        &mut tx,
        &[phase_prepare(owner)],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.is_empty());
}
