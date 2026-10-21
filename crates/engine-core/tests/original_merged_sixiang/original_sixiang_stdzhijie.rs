//! Golden coverage for 标甘夫人「智诫」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const STD_GANFUREN: u32 = 2_343_280_510;
const STDZHIJIE: u32 = 2_635_655_868;
const STDSHUSHENX: u32 = 3_633_500_345;
const SOURCE: &str = include_str!("../../../../content/original/ported/sixiang/stdzhijie.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/sixiang/sixiang.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

struct Setup {
    state: GameState,
    owner: PlayerId,
    phase_player: PlayerId,
    other: PlayerId,
}

fn setup() -> Setup {
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 2_026_072_413, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package())
        .unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let phase_player = players[1];
    let other = players[2];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(STDZHIJIE),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = phase_player;
    tx.state_mut().turn.phase = Phase::Play;
    Setup {
        state: tx.commit().state,
        owner,
        phase_player,
        other,
    }
}

fn inject(state: &mut GameState, owner: PlayerId, kind: CardKind, def: u32) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn skill_id(state: &GameState, owner: PlayerId) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == STDZHIJIE)
        .unwrap()
        .id
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

fn dispatch_card_used(
    mut state: GameState,
    player: PlayerId,
    card: CardId,
    target: PlayerId,
) -> GameState {
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player,
            card,
            targets: vec![target],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    state = tx.commit().state;
    state
}

#[test]
fn package_publishes_abi140_and_complete_ganfuren() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert_eq!(package.skill_index.len(), 181);
    assert_eq!(package.character_index.len(), 101);
    assert!(package.skill_index.iter().any(|skill| {
        skill.skill_def_id == STDZHIJIE && skill.name == "stdzhijie" && !skill.locked
    }));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == STD_GANFUREN)
        .unwrap();
    assert_eq!(character.skills, [STDZHIJIE, STDSHUSHENX]);

    let compiled = content_compiler::compile_source_named("stdzhijie", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 140);
    for expected in compiled.modules {
        let actual = package
            .modules
            .iter()
            .find(|module| module.name == expected.name)
            .unwrap();
        assert_eq!(actual.logic_hash, expected.logic_hash);
    }
}

#[test]
fn bound_category_draws_only_for_first_three_matching_uses_in_that_phase() {
    let mut setup = setup();
    let revealed = inject(
        &mut setup.state,
        setup.phase_player,
        CardKind::TrickWuzhong,
        99_401,
    );
    let matching: Vec<_> = (0..4)
        .map(|offset| {
            inject(
                &mut setup.state,
                setup.phase_player,
                CardKind::TrickWuxie,
                99_410 + offset,
            )
        })
        .collect();
    let basic = inject(&mut setup.state, setup.phase_player, CardKind::Sha, 99_420);

    let mut tx = Transaction::begin(setup.state);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::TurnStateChanged {
                current: setup.phase_player,
                phase: TurnPhaseTag::Play,
            }],
            &mut TriggerFireRecord::new(),
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let committed = tx.commit();
    assert_eq!(committed.output.choice.unwrap().kind, ChoiceKind::Confirm);
    let (state, confirmed) = resolve(committed.state, setup.owner, vec![1], 1);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseRevealOtherHandCardRememberCategory
    );
    assert_eq!(confirmed.choice.unwrap().kind, ChoiceKind::SelectCards);
    let (mut state, selected) = resolve(state, setup.owner, vec![revealed.raw()], 2);
    assert!(matches!(selected.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state
            .skills
            .get(skill_id(&state, setup.owner))
            .unwrap()
            .state_blob,
        [
            b"BCT1".as_slice(),
            &setup.phase_player.raw().to_le_bytes(),
            &[2]
        ]
        .concat()
    );

    let before = state.hand_count(setup.owner);
    state = dispatch_card_used(state, setup.phase_player, basic, setup.other);
    assert_eq!(state.hand_count(setup.owner), before);
    for (index, card) in matching.into_iter().enumerate() {
        state = dispatch_card_used(state, setup.phase_player, card, setup.other);
        assert_eq!(
            state.hand_count(setup.owner),
            before + u32::try_from((index + 1).min(3)).unwrap()
        );
    }

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: setup.phase_player,
            phase: Phase::Discard.to_u8(),
        }],
    )
    .unwrap();
    assert!(tx
        .state()
        .skills
        .get(skill_id(tx.state(), setup.owner))
        .unwrap()
        .state_blob
        .is_empty());
    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: setup.phase_player,
            phase: Phase::Play.to_u8(),
        }],
    )
    .unwrap();
    assert!(dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: setup.phase_player,
            phase: TurnPhaseTag::Play,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap()
    .is_empty());
}
