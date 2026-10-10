//! Golden coverage for 桃廖化「当先」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{turn_flag, Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TYDANGXIAN: u32 = 3_099_116_192;
const TYFULI: u32 = 3_185_405_820;
const TY_LIAOHUA: u32 = 2_402_057_315;
const SOURCE: &str = include_str!("../../../content/original/ported/offline/tydangxian.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    previous: PlayerId,
}

fn inject_discard(state: &mut GameState, def: u32, kind: CardKind, suit: Suit) -> CardId {
    let zone = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone,
        owner: PlayerId::NONE,
        face_up: true,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("tydangxian-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, TYDANGXIAN as u64, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let previous = seats[0];
    let owner = seats[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(TYDANGXIAN),
        },
        EffectRequest::SetTurnState {
            current: previous,
            phase: Phase::End.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        previous,
    }
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

#[test]
fn package_publishes_exact_skill_and_complete_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let compiled = content_compiler::compile_source_named("tydangxian", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 77);
    assert_eq!(package.manifest.rule_vm_abi_version, 691);
    assert!(package.skill_index.iter().any(|skill| {
        skill.skill_def_id == TYDANGXIAN && skill.name == "tydangxian" && skill.locked
    }));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == TY_LIAOHUA)
        .unwrap();
    assert_eq!(character.skills, [TYDANGXIAN, TYFULI]);
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .unwrap();
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn turn_begin_forces_sha_choice_then_opens_extra_play_phase() {
    let mut setup = setup();
    let first = inject_discard(&mut setup.state, 97_001, CardKind::Sha, Suit::Spade);
    let selected = inject_discard(&mut setup.state, 97_002, CardKind::Sha, Suit::Heart);
    let other = inject_discard(&mut setup.state, 97_003, CardKind::Shan, Suit::Club);

    let (state, result) = submit(setup.state, setup.previous, CommandKind::EndPhase, 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let choice = result.choice.unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    assert_eq!((choice.min_count, choice.max_count), (1, 1));
    assert!(!choice.allow_cancel);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [first.raw(), selected.raw()]
    );
    assert_ne!(state.turn.phase, Phase::Play);

    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, resolved) = submit(
        state,
        setup.owner,
        CommandKind::ResolveChoice {
            choice_id,
            selections: vec![selected.raw()],
            cancelled: false,
        },
        2,
    );
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.turn.current, setup.owner);
    assert_eq!(state.turn.phase, Phase::Play);
    assert_eq!(state.turn.flag(turn_flag::OPENING_EXTRA_PLAY_ACTIVE), 1);
    assert!(state.hand_cards(setup.owner).contains(&selected));
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(state.zones.get(discard).unwrap().cards.contains(&first));
    assert!(state.zones.get(discard).unwrap().cards.contains(&other));
    assert!(!state.zones.get(discard).unwrap().cards.contains(&selected));
}

#[test]
fn no_discard_sha_skips_choice_and_still_opens_extra_play_phase() {
    let mut setup = setup();
    let other = inject_discard(&mut setup.state, 97_011, CardKind::Shan, Suit::Club);
    let (state, result) = submit(setup.state, setup.previous, CommandKind::EndPhase, 10);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert_eq!(state.turn.current, setup.owner);
    assert_eq!(state.turn.phase, Phase::Play);
    assert_eq!(state.turn.flag(turn_flag::OPENING_EXTRA_PLAY_ACTIVE), 1);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(state.zones.get(discard).unwrap().cards.contains(&other));
}
