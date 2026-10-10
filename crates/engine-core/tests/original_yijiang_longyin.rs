//! Golden coverage for 关平「龙吟」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{turn_flag, Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const LONGYIN: u32 = 3_578_226_944;
const GUANPING: u32 = 2_957_024_910;
const SOURCE: &str = include_str!("../../../content/original/ported/yijiang/longyin.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

struct Setup {
    state: GameState,
    source: PlayerId,
    target: PlayerId,
    owner: PlayerId,
    sha: CardId,
    cost: CardId,
}

fn inject(state: &mut GameState, owner: PlayerId, def: u32, kind: CardKind, suit: Suit) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
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

fn setup(seed: u64, suit: Suit) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let source = tx.state().turn.current;
    let target = players
        .iter()
        .copied()
        .find(|player| *player != source)
        .unwrap();
    let owner = players
        .iter()
        .copied()
        .find(|player| *player != source && *player != target)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in players {
        effects.push(EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        });
    }
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(LONGYIN),
        },
        EffectRequest::SetTurnState {
            current: source,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let sha = inject(tx.state_mut(), source, 93_001, CardKind::Sha, suit);
    let cost = inject(tx.state_mut(), owner, 93_002, CardKind::Jiu, Suit::Club);
    Setup {
        state: tx.commit().state,
        source,
        target,
        owner,
        sha,
        cost,
    }
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
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
                cancelled: false,
            },
        },
    )
}

fn use_sha_and_pay(setup: Setup, key: u64) -> GameState {
    let (state, offered) = apply_command(
        setup.state,
        Command {
            actor: setup.source,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card: setup.sha,
                targets: vec![setup.target],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(offered.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED), 1);
    assert_eq!(state.resolution.pending_observer, setup.owner.raw());

    let (state, discard) = resolve(state, setup.owner, vec![1], key + 1);
    assert!(matches!(discard.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseDiscardOneForShaRefundAndDrawIfRed
    );
    assert_eq!(state.resolution.pending_observer, setup.owner.raw());

    let (state, paid) = resolve(state, setup.owner, vec![setup.cost.raw()], key + 2);
    assert!(matches!(paid.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    state
}

#[test]
fn package_publishes_complete_guanping_and_exact_abi53_package() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 637);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == GUANPING)
        .unwrap();
    assert_eq!(character.skills, [LONGYIN]);
    let compiled = content_compiler::compile_source_named("longyin", SOURCE).unwrap();
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .unwrap();
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn black_sha_refunds_its_count_after_cost_without_drawing() {
    let setup = setup(2_026_071_931, Suit::Spade);
    let owner = setup.owner;
    let state = use_sha_and_pay(setup, 10);
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED), 0);
    assert_eq!(state.hand_count(owner), 0);
}

#[test]
fn red_sha_refunds_its_count_after_cost_and_draws_one() {
    let setup = setup(2_026_071_932, Suit::Heart);
    let owner = setup.owner;
    let state = use_sha_and_pay(setup, 20);
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED), 0);
    assert_eq!(state.hand_count(owner), 1);
}

#[test]
fn refund_is_idempotent_and_never_decrements_an_uncounted_sha() {
    let mut counted = setup(2_026_071_933, Suit::Spade);
    counted.state.turn.set_flag(turn_flag::SHA_PLAYED, 1);
    counted.state.resolution.resume_kind = ContinuationKind::ResumePlayedSha;
    // saved-Sha flags: bit 0 respondable, bit 1 counted, bit 2 refunded.
    counted.state.resolution.resume_payload = vec![
        counted.source.raw(),
        counted.target.raw(),
        1,
        0,
        1 | 2,
        counted.sha.raw(),
    ];
    let refund = EffectRequest::RefundCurrentShaUseAndDrawIfRed {
        player: counted.owner,
        sha_user: counted.source,
        card: counted.sha,
        skill: SkillId::NONE,
    };
    let mut tx = Transaction::begin(counted.state);
    run_effects(&mut tx, vec![refund.clone(), refund]).unwrap();
    assert_eq!(tx.state().turn.flag(turn_flag::SHA_PLAYED), 0);
    assert_eq!(tx.state().resolution.resume_payload[4] & 4, 4);

    let mut uncounted = setup(2_026_071_934, Suit::Spade);
    uncounted.state.turn.set_flag(turn_flag::SHA_PLAYED, 1);
    uncounted.state.resolution.resume_kind = ContinuationKind::ResumePlayedSha;
    uncounted.state.resolution.resume_payload = vec![
        uncounted.source.raw(),
        uncounted.target.raw(),
        1,
        0,
        1,
        uncounted.sha.raw(),
    ];
    let mut tx = Transaction::begin(uncounted.state);
    run_effects(
        &mut tx,
        vec![EffectRequest::RefundCurrentShaUseAndDrawIfRed {
            player: uncounted.owner,
            sha_user: uncounted.source,
            card: uncounted.sha,
            skill: SkillId::NONE,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().turn.flag(turn_flag::SHA_PLAYED), 1);
    assert_eq!(tx.state().resolution.resume_payload[4], 1);
}

#[test]
fn missing_cost_at_effect_time_cannot_refund_or_draw() {
    let setup = setup(2_026_071_935, Suit::Heart);
    let mut tx = Transaction::begin(setup.state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: setup.owner,
            count: u32::MAX,
        }],
    )
    .unwrap();
    tx.state_mut().turn.set_flag(turn_flag::SHA_PLAYED, 1);
    tx.state_mut().resolution.resume_kind = ContinuationKind::ResumePlayedSha;
    tx.state_mut().resolution.resume_payload = vec![
        setup.source.raw(),
        setup.target.raw(),
        1,
        0,
        1 | 2,
        setup.sha.raw(),
    ];
    run_effects(
        &mut tx,
        vec![EffectRequest::ChooseDiscardOneForShaRefundAndDrawIfRed {
            player: setup.owner,
            sha_user: setup.source,
            card: setup.sha,
            skill: SkillId::NONE,
        }],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());
    assert_eq!(tx.state().turn.flag(turn_flag::SHA_PLAYED), 1);
    assert_eq!(tx.state().hand_count(setup.owner), 0);
}
