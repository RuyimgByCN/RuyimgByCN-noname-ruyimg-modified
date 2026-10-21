//! Golden coverage for 廖化「当先」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{turn_flag, Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DANGXIAN: u32 = 2_393_638_375;
const FULI: u32 = 3_748_306_630;
const LIAOHUA: u32 = 3_588_461_279;
const SOURCE: &str = include_str!("../../../../content/original/ported/yijiang/dangxian.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/yijiang/yijiang.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    first_sha: CardId,
    second_sha: CardId,
}

fn inject_sha(state: &mut GameState, owner: PlayerId, def: u32, suit: Suit) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Sha,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn setup(seed: u64) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();

    let seats = tx.state().players.alive_by_seat();
    let current = tx.state().turn.current;
    let current_index = seats.iter().position(|player| *player == current).unwrap();
    let owner = seats[(current_index + 1) % seats.len()];
    let target = current;

    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in seats {
        effects.push(EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        });
    }
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DANGXIAN),
        },
        EffectRequest::SetTurnState {
            current,
            phase: Phase::End.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();

    let first_sha = inject_sha(tx.state_mut(), owner, 94_001, Suit::Spade);
    let second_sha = inject_sha(tx.state_mut(), owner, 94_002, Suit::Heart);
    Setup {
        state: tx.commit().state,
        owner,
        target,
        first_sha,
        second_sha,
    }
}

fn end_phase(state: GameState, actor: PlayerId, key: u64) -> (GameState, PipelineResult) {
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

fn play_sha(
    state: GameState,
    actor: PlayerId,
    card: CardId,
    target: PlayerId,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card,
                targets: vec![target],
                card_targets: Vec::new(),
            },
        },
    )
}

#[test]
fn package_publishes_complete_liaohua_and_exact_abi53_package() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 637);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == LIAOHUA)
        .unwrap();
    assert_eq!(character.skills, [DANGXIAN, FULI]);
    let compiled = content_compiler::compile_source_named("dangxian", SOURCE).unwrap();
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .unwrap();
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn extra_play_precedes_prepare_and_normal_play_has_an_independent_sha_allowance() {
    let setup = setup(2_026_071_951);

    let (state, began) = end_phase(setup.state, setup.target, 1);
    assert!(matches!(began.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.current, setup.owner);
    assert_eq!(state.turn.phase, Phase::Play);
    assert_eq!(state.turn.flag(turn_flag::OPENING_EXTRA_PLAY_ACTIVE), 1);
    assert_eq!(state.turn.flag(turn_flag::OPENING_EXTRA_PLAY_PENDING), 0);
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED_IN_PLAY_PHASE), 0);

    let (state, first) = play_sha(state, setup.owner, setup.first_sha, setup.target, 2);
    assert!(matches!(first.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED), 1);
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED_IN_PLAY_PHASE), 1);

    let (state, prepare) = end_phase(state, setup.owner, 3);
    assert!(matches!(prepare.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.phase, Phase::Prepare);
    assert_eq!(state.turn.flag(turn_flag::OPENING_EXTRA_PLAY_ACTIVE), 0);

    let (state, judge) = end_phase(state, setup.owner, 4);
    assert!(matches!(judge.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.phase, Phase::Judge);

    let (state, draw) = end_phase(state, setup.owner, 5);
    assert!(matches!(draw.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.phase, Phase::Draw);

    let (state, play) = end_phase(state, setup.owner, 6);
    assert!(matches!(play.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.phase, Phase::Play);
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED), 1);
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED_IN_PLAY_PHASE), 0);

    let (state, second) = play_sha(state, setup.owner, setup.second_sha, setup.target, 7);
    assert!(matches!(second.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED), 2);
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED_IN_PLAY_PHASE), 1);
}
