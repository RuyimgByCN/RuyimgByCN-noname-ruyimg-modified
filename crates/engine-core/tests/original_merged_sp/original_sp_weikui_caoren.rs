//! Golden coverage for SP 曹仁「伪溃」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::{query, QueryContext};
use engine_core::state::{turn_flag, Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::TriggerFireRecord;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const WEIKUI: u32 = 3_890_756_007;
const WEIKUI2: u32 = 2_980_539_775;
const LIZHAN: u32 = 2_790_547_004;
const SP_CAOREN: u32 = 4_080_002_414;
const SOURCE: &str = include_str!("../../../../content/original/ported/sp/weikui.skill");
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/sp/sp.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
}

fn setup(seed: u64) -> Setup {
    let package = content_compiler::compile_source_named("weikui-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 5, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[2];
    for player in &players {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 4;
        participant.max_hp = 4;
    }
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
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(WEIKUI),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    let skill = skill_id(tx.state(), owner, WEIKUI).unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        target,
        skill,
    }
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> Option<SkillId> {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| state.skills.get(*skill).unwrap().def.raw() == def)
}

fn inject(state: &mut GameState, owner: PlayerId, def: u32, kind: CardKind) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Club,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn command(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn activate(setup: Setup, key: u64) -> (GameState, PipelineResult) {
    command(
        setup.state,
        setup.owner,
        CommandKind::ActivateSkill {
            skill: setup.skill,
            targets: vec![setup.target],
            card_targets: Vec::new(),
        },
        key,
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    command(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled,
        },
        key,
    )
}

fn distance(state: &GameState, from: PlayerId, to: PlayerId) -> i32 {
    match query(
        state,
        &QueryRequest::Distance { from, to },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("unexpected distance response: {other:?}"),
    }
}

fn bound_target_state(target: PlayerId) -> Vec<u8> {
    let mut state = b"BND1".to_vec();
    state.extend_from_slice(&target.raw().to_le_bytes());
    state
}

fn end_owner_turn(state: GameState, owner: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    let effects = engine_core::trigger::dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current: owner,
            phase: TurnPhaseTag::End,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other: false,
            current_dealt_damage_other: false,
            phase_targeted_players: 0,
            phase_hp_changed_players: 0,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

#[test]
fn package_publishes_abi196_and_complete_sp_caoren() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == WEIKUI)
        .unwrap();
    assert_eq!(active.window, "play_phase");
    assert_eq!(active.uses_per_turn, Some(1));
    let targets = active.targets.as_ref().unwrap();
    assert_eq!(
        (targets.min, targets.max, targets.allow_self),
        (1, 1, false)
    );
    assert!(targets.hand_nonempty);
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == WEIKUI2
            && modifier.target == "distance_from_skill_bound_target"
            && modifier.mode == "override"
            && modifier.value == 1
    }));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == SP_CAOREN)
        .unwrap();
    assert_eq!(character.name_key, "sp_caoren");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [WEIKUI, LIZHAN]);

    let compiled = content_compiler::compile_source_named("weikui", SOURCE).unwrap();
    for expected in &compiled.modules {
        let actual = package
            .modules
            .iter()
            .find(|module| module.name == expected.name)
            .unwrap();
        assert_eq!(actual.logic_hash, expected.logic_hash);
    }
}

#[test]
fn no_shan_loses_hp_then_forces_one_target_hand_discard() {
    let mut setup = setup(2_026_072_601);
    let card = inject(
        &mut setup.state,
        setup.target,
        196_001,
        CardKind::TrickWuzhong,
    );
    let hp = setup.state.players.get(setup.owner).unwrap().hp;
    let owner = setup.owner;
    let target = setup.target;
    let (state, offered) = activate(setup, 1);
    assert_eq!(state.players.get(owner).unwrap().hp, hp - 1);
    let choice = offered.choice.unwrap();
    assert_eq!(choice.observer, owner);
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseDiscardTargetCard
    );
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [card.raw()]
    );
    assert!(!offered
        .events
        .iter()
        .any(|event| matches!(event, DomainEvent::CardsRevealed { .. })));

    let (state, resolved) = resolve(state, owner, vec![card.raw()], false, 2);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.cards.get(card).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
    assert!(skill_id(&state, owner, WEIKUI2).is_none());
    assert_eq!(state.hand_count(target), 0);
}

#[test]
fn shan_is_privately_revealed_then_normal_sha_binds_distance_until_turn_end() {
    let mut setup = setup(2_026_072_602);
    let shan = inject(&mut setup.state, setup.target, 196_011, CardKind::Shan);
    let owner = setup.owner;
    let target = setup.target;
    let target_hp = setup.state.players.get(target).unwrap().hp;
    assert_eq!(distance(&setup.state, owner, target), 2);

    let (state, offered) = activate(setup, 10);
    let response = offered.choice.unwrap();
    assert_eq!(response.observer, target);
    assert!(response
        .options
        .iter()
        .any(|option| option.token == shan.raw()));
    assert!(offered.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardsRevealed { cards, to }
            if cards == &[shan] && to == &[owner]
    )));
    assert!(skill_id(&state, owner, WEIKUI2).is_none());

    let (state, resolved) = resolve(state, target, vec![shan.raw()], false, 11);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, target_hp);
    let helper = skill_id(&state, owner, WEIKUI2).unwrap();
    assert_eq!(
        state.skills.get(helper).unwrap().state_blob,
        bound_target_state(target)
    );
    assert_eq!(distance(&state, owner, target), 1);

    let expired = end_owner_turn(state, owner);
    assert!(skill_id(&expired, owner, WEIKUI2).is_none());
    assert_eq!(distance(&expired, owner, target), 2);
}

#[test]
fn exhausted_sha_count_skips_sha_but_still_binds_target() {
    let mut setup = setup(2_026_072_603);
    let shan = inject(&mut setup.state, setup.target, 196_021, CardKind::Shan);
    setup
        .state
        .turn
        .set_flag(turn_flag::SHA_PLAYED_IN_PLAY_PHASE, 1);
    let owner = setup.owner;
    let target = setup.target;
    let (state, result) = activate(setup, 20);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert!(result.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardsRevealed { cards, to }
            if cards == &[shan] && to == &[owner]
    )));
    assert!(!result
        .events
        .iter()
        .any(|event| matches!(event, DomainEvent::CardUsed { .. })));
    assert_eq!(
        state
            .skills
            .get(skill_id(&state, owner, WEIKUI2).unwrap())
            .unwrap()
            .state_blob,
        bound_target_state(target)
    );
}

#[test]
fn dying_from_the_hp_cost_stops_the_hand_branch() {
    let mut setup = setup(2_026_072_604);
    setup.state.players.get_mut(setup.owner).unwrap().hp = 1;
    let card = inject(
        &mut setup.state,
        setup.target,
        196_031,
        CardKind::TrickWuzhong,
    );
    let owner = setup.owner;
    let target = setup.target;
    assert_eq!(setup.state.hand_count(owner), 0);
    assert!(engine_core::query::dying_save_card_for(&setup.state, owner, owner).is_none());
    assert_eq!(setup.state.cards.get(card).unwrap().owner, target);
    let (state, result) = activate(setup, 30);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let participant = state.players.get(owner).unwrap();
    assert!(!participant.alive && !participant.dying && participant.hp == 0);
    assert!(state.hand_cards(target).contains(&card));
    assert!(skill_id(&state, owner, WEIKUI2).is_none());
    assert!(result.choice.is_none());
    assert_eq!(
        result
            .events
            .iter()
            .filter(|event| matches!(
                event,
                DomainEvent::DamageDealt {
                    target: damaged,
                    is_lose_hp: true,
                    ..
                } if *damaged == owner
            ))
            .count(),
        1
    );
    assert!(!result
        .events
        .iter()
        .any(|event| matches!(event, DomainEvent::CardsRevealed { .. })));
}
