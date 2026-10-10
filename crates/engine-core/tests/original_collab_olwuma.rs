//! Golden coverage for COLLAB「午马」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{turn_flag, Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const OLWUMA: u32 = 3_608_420_255;
const OL_WUMA: u32 = 3_167_375_132;
const FORCE_SKIP: u32 = 4_294_967_000;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/collab/collab.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    others: Vec<PlayerId>,
}

fn setup(seed: u64, force_skip: bool) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    if force_skip {
        let helper = content_compiler::compile_source_named(
            "olwuma-test-force-skip",
            r#"
skill force_skip(4294967000) locked {
  modifier skip_judge_phase override 1;
  modifier skip_discard_phase override 1;
}
"#,
        )
        .unwrap();
        std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&helper).unwrap();
    }
    let owner = tx.state().turn.current;
    let others = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLWUMA),
    });
    if force_skip {
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(FORCE_SKIP),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        others,
    }
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    def: u32,
    kind: CardKind,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn submit(state: GameState, actor: PlayerId, key: u64) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::EndPhase,
        },
    )
}

fn dispatch_card_used(tx: &mut Transaction, user: PlayerId, card: CardId, targets: Vec<PlayerId>) {
    let effects = dispatch_for_events(
        tx,
        &[DomainEvent::CardUsed {
            player: user,
            card,
            targets,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(tx, effects).unwrap();
}

#[test]
fn package_retains_olwuma_skill_and_character_closure() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 694);
    assert_eq!(package.skill_index.len(), 152);
    assert_eq!(package.character_index.len(), 88);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == OLWUMA)
        .unwrap();
    assert_eq!(trigger.event, "card_targeted");
    assert_eq!(trigger.filter, "owner_alive_targeted_by_other_trick");
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == OLWUMA && modifier.target == "turn_face_down_cancel"
    }));
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == OLWUMA && modifier.target == "restore_skipped_phase"
    }));
    assert!(package.character_index.iter().any(|character| {
        character.id == OL_WUMA
            && character.name_key == "ol_wuma"
            && character.sex == "male"
            && character.group == "qun"
            && character.max_hp == 4
            && character.skills == [OLWUMA]
    }));
}

#[test]
fn another_players_trick_draws_once_even_when_it_has_multiple_targets() {
    let mut setup = setup(2_026_072_306, false);
    let source = setup.others[0];
    let other = setup.others[1];
    let trick = inject(
        &mut setup.state,
        source,
        ZoneKind::Hand,
        99_101,
        CardKind::TrickWuzhong,
    );
    let before = setup.state.hand_count(setup.owner);
    let mut tx = Transaction::begin(setup.state);
    dispatch_card_used(&mut tx, source, trick, vec![setup.owner, source, other]);
    assert_eq!(tx.state().hand_count(setup.owner), before + 1);

    let sha = inject(
        tx.state_mut(),
        source,
        ZoneKind::Hand,
        99_102,
        CardKind::Sha,
    );
    dispatch_card_used(&mut tx, source, sha, vec![setup.owner]);
    assert_eq!(tx.state().hand_count(setup.owner), before + 1);

    let own_trick = inject(
        tx.state_mut(),
        setup.owner,
        ZoneKind::Hand,
        99_103,
        CardKind::TrickJuedou,
    );
    let before_own_use = tx.state().hand_count(setup.owner);
    dispatch_card_used(&mut tx, setup.owner, own_trick, vec![other]);
    assert_eq!(tx.state().hand_count(setup.owner), before_own_use);
}

#[test]
fn face_up_cannot_turn_face_down_but_existing_face_down_state_can_clear() {
    let setup = setup(2_026_072_307, false);
    let mut tx = Transaction::begin(setup.state);
    let events = run_effects(
        &mut tx,
        vec![EffectRequest::TurnOver {
            player: setup.owner,
        }],
    )
    .unwrap();
    assert!(events.is_empty());
    assert!(!tx.state().players.get(setup.owner).unwrap().turned_over);

    tx.state_mut()
        .players
        .get_mut(setup.owner)
        .unwrap()
        .turned_over = true;
    let events = run_effects(
        &mut tx,
        vec![EffectRequest::TurnOver {
            player: setup.owner,
        }],
    )
    .unwrap();
    assert!(!tx.state().players.get(setup.owner).unwrap().turned_over);
    assert!(events.iter().any(|event| matches!(
        event,
        DomainEvent::PlayerTurnedOver {
            player,
            turned_over: false,
        } if *player == setup.owner
    )));
}

#[test]
fn draw_and_play_skip_flags_record_events_but_restore_the_exact_phases() {
    let mut setup = setup(2_026_072_308, false);
    setup.state.turn.current = setup.owner;
    setup.state.turn.phase = Phase::Judge;
    setup.state.turn.set_flag(turn_flag::SKIP_DRAW, 1);
    let before = setup.state.hand_count(setup.owner);
    let (state, draw) = submit(setup.state, setup.owner, 1);
    assert!(matches!(draw.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.phase, Phase::Draw);
    assert_eq!(state.hand_count(setup.owner), before + 2);
    assert!(draw.events.iter().any(|event| matches!(
        event,
        DomainEvent::PhaseSkipped {
            current,
            phase: TurnPhaseTag::Draw,
        } if *current == setup.owner
    )));

    let mut state = state;
    state.turn.set_flag(turn_flag::SKIP_PLAY, 1);
    let (state, play) = submit(state, setup.owner, 2);
    assert!(matches!(play.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.phase, Phase::Play);
    assert!(play.events.iter().any(|event| matches!(
        event,
        DomainEvent::PhaseSkipped {
            current,
            phase: TurnPhaseTag::Play,
        } if *current == setup.owner
    )));
}

#[test]
fn judgment_and_discard_skip_modifiers_still_execute_the_owner_phases() {
    let mut judge = setup(2_026_072_309, true);
    let delayed = inject(
        &mut judge.state,
        judge.owner,
        ZoneKind::Judge,
        99_111,
        CardKind::TrickBingliang,
    );
    judge.state.turn.current = judge.owner;
    judge.state.turn.phase = Phase::Judge;
    let (state, result) = submit(judge.state, judge.owner, 10);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let judge_zone = state.zones.find(ZoneKind::Judge, judge.owner).unwrap();
    assert!(!state
        .zones
        .get(judge_zone)
        .unwrap()
        .cards
        .contains(&delayed));
    assert!(result.events.iter().any(|event| matches!(
        event,
        DomainEvent::PhaseSkipped {
            current,
            phase: TurnPhaseTag::Judge,
        } if *current == judge.owner
    )));

    let mut discard = setup(2_026_072_310, true);
    inject(
        &mut discard.state,
        discard.owner,
        ZoneKind::Hand,
        99_112,
        CardKind::Sha,
    );
    discard.state.turn.current = discard.owner;
    discard.state.turn.phase = Phase::Discard;
    let (state, result) = submit(discard.state, discard.owner, 11);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.phase, Phase::Discard);
    assert_eq!(
        result.choice.as_ref().unwrap().kind,
        ChoiceKind::DiscardToLimit
    );
    assert!(result.events.iter().any(|event| matches!(
        event,
        DomainEvent::PhaseSkipped {
            current,
            phase: TurnPhaseTag::Discard,
        } if *current == discard.owner
    )));
}

#[test]
fn explicit_skip_current_phase_is_audited_without_advancing() {
    let mut setup = setup(2_026_072_311, false);
    for phase in [Phase::Draw, Phase::Play, Phase::Discard] {
        setup.state.turn.current = setup.owner;
        setup.state.turn.phase = phase;
        let mut tx = Transaction::begin(setup.state);
        let events = run_effects(
            &mut tx,
            vec![EffectRequest::SkipCurrentPhase {
                player: setup.owner,
            }],
        )
        .unwrap();
        assert_eq!(tx.state().turn.phase, phase);
        assert!(events.iter().any(|event| matches!(
            event,
            DomainEvent::PhaseSkipped { current, phase: skipped }
                if *current == setup.owner && *skipped == phase.to_tag()
        )));
        setup.state = tx.commit().state;
    }
}
