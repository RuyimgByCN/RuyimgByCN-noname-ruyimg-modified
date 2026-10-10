//! Golden coverage for 朱桓「奋励」「平寇」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::canonical_state_hash;
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const ZHUHUAN: u32 = 3_434_288_625;
const FENLI: u32 = 2_848_697_859;
const PINGKOU: u32 = 3_823_456_872;
const FENLI_SOURCE: &str = include_str!("../../../content/original/ported/yijiang/fenli.skill");
const PINGKOU_SOURCE: &str = include_str!("../../../content/original/ported/yijiang/pingkou.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    others: [PlayerId; 3],
}

fn setup(seed: u64) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
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
    effects.extend([FENLI, PINGKOU].map(|skill_def| EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill_def),
    }));
    run_effects(&mut tx, effects).unwrap();
    for player in &players {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 8;
        participant.max_hp = 8;
    }
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Judge;
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        others: [players[1], players[2], players[3]],
    }
}

fn inject_equip(state: &mut GameState, owner: PlayerId, def: u32) -> CardId {
    let zone = state.zones.find(ZoneKind::Equip, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Weapon,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 2,
        zone,
        owner,
        face_up: true,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled: false,
        },
        key,
    )
}

fn open_event(state: GameState, event: DomainEvent) -> (GameState, PipelineResult) {
    let mut tx = Transaction::begin(state);
    match dispatch_for_events(&mut tx, &[event], &mut TriggerFireRecord::new()) {
        Ok(effects) => assert_eq!(
            run_effects(&mut tx, effects).unwrap_err(),
            EffectError::ChoiceRequested
        ),
        Err(error) => assert_eq!(error, EffectError::ChoiceRequested),
    }
    let committed = tx.commit();
    let new_revision = committed.state.revision;
    let state_hash = committed.state_hash;
    let events = committed.output.events;
    let choice = committed.output.choice;
    (
        committed.state,
        PipelineResult {
            result: CommandResult::Accepted {
                new_revision,
                state_hash,
            },
            events,
            choice,
        },
    )
}

fn assert_optional(result: &PipelineResult) {
    let choice = result.choice.as_ref().unwrap();
    assert_eq!(choice.kind, ChoiceKind::Confirm);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [0, 1]
    );
}

#[test]
fn package_publishes_complete_zhuhuan_and_exact_abi55_modules() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 637);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == ZHUHUAN)
        .unwrap();
    assert_eq!(character.skills, [FENLI, PINGKOU]);
    for (name, source) in [("fenli", FENLI_SOURCE), ("pingkou", PINGKOU_SOURCE)] {
        let compiled = content_compiler::compile_source_named(name, source).unwrap();
        assert_eq!(compiled.manifest.rule_vm_abi_version, 55);
        for expected in compiled.modules {
            let actual = package
                .modules
                .iter()
                .find(|module| module.name == expected.name)
                .unwrap();
            assert_eq!(actual.logic_hash, expected.logic_hash);
        }
    }
}

#[test]
fn declining_draw_fenli_resumes_the_original_draw_phase() {
    let setup = setup(2_026_073_501);
    let before = setup.state.hand_count(setup.owner);
    let (state, offered) = submit(setup.state, setup.owner, CommandKind::EndPhase, 1);
    assert_optional(&offered);
    assert_eq!(state.turn.phase, Phase::Draw);
    assert_eq!(
        state.resolution.resume_kind,
        ContinuationKind::ContinueEnteredDrawPhase
    );

    let (state, declined) = resolve(state, setup.owner, vec![0], 2);
    assert!(matches!(declined.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.turn.phase, Phase::Draw);
    assert_eq!(state.hand_count(setup.owner), before + 2);
    assert_eq!(state.turn.skipped_phase_count(), 0);
}

#[test]
fn draw_fenli_skip_does_not_draw_and_records_the_phase() {
    let mut setup = setup(2_026_073_502);
    setup.state.players.get_mut(setup.others[0]).unwrap().hp = 9;
    setup.state.players.get_mut(setup.others[0]).unwrap().max_hp = 9;
    let before = setup.state.hand_count(setup.owner);
    let (state, offered) = submit(setup.state, setup.owner, CommandKind::EndPhase, 10);
    assert_optional(&offered);
    let (state, accepted) = resolve(state, setup.owner, vec![1], 11);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.turn.phase, Phase::Play);
    assert_eq!(state.hand_count(setup.owner), before);
    assert_eq!(state.turn.skipped_phase_count(), 1);
    assert!(accepted.events.iter().any(|event| matches!(
        event,
        DomainEvent::PhaseSkipped {
            current,
            phase: TurnPhaseTag::Draw,
        } if *current == setup.owner
    )));
}

#[test]
fn play_and_discard_fenli_feed_ordered_pingkou_with_atomic_target_validation() {
    let mut setup = setup(2_026_073_503);
    setup.state.turn.phase = Phase::Play;
    inject_equip(&mut setup.state, setup.owner, 99_501);
    let (state, play_offer) = open_event(
        setup.state,
        DomainEvent::TurnStateChanged {
            current: setup.owner,
            phase: TurnPhaseTag::Play,
        },
    );
    assert_optional(&play_offer);

    let (state, discard_offer) = resolve(state, setup.owner, vec![1], 20);
    assert_optional(&discard_offer);
    assert_eq!(state.turn.phase, Phase::Discard);
    assert_eq!(state.turn.skipped_phase_count(), 1);

    let (state, pingkou_offer) = resolve(state, setup.owner, vec![1], 21);
    assert_optional(&pingkou_offer);
    assert_eq!(state.turn.phase, Phase::End);
    assert_eq!(state.turn.skipped_phase_count(), 2);

    let (state, targets_offer) = resolve(state, setup.owner, vec![1], 22);
    let targets = targets_offer.choice.as_ref().unwrap();
    assert_eq!(targets.kind, ChoiceKind::SelectTargets);
    assert_eq!(targets.min_count, 1);
    assert_eq!(targets.max_count, 2);
    assert!(!targets
        .options
        .iter()
        .any(|option| option.token == setup.owner.raw()));
    for target in setup.others {
        assert!(targets
            .options
            .iter()
            .any(|option| option.token == target.raw()));
    }

    let before_hash = canonical_state_hash(&state);
    let (same, forged) = resolve(state, setup.owner, vec![setup.owner.raw()], 23);
    assert!(matches!(forged.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before_hash);

    let first = setup.others[1];
    let second = setup.others[0];
    let first_hp = same.players.get(first).unwrap().hp;
    let second_hp = same.players.get(second).unwrap().hp;
    let (state, damaged) = resolve(same, setup.owner, vec![first.raw(), second.raw()], 24);
    assert!(matches!(damaged.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(first).unwrap().hp, first_hp - 1);
    assert_eq!(state.players.get(second).unwrap().hp, second_hp - 1);
    let order: Vec<_> = damaged
        .events
        .iter()
        .filter_map(|event| match event {
            DomainEvent::DamageDealt { source, target, .. } if *source == setup.owner => {
                Some(*target)
            }
            _ => None,
        })
        .collect();
    assert_eq!(order, [first, second]);
}
