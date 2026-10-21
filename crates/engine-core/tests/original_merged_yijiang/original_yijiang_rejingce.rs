//! Golden coverage for 郭淮「精策」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::{query, QueryContext};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const REJINGCE: u32 = 2_158_663_152;
const GUOHUAI: u32 = 3_179_439_469;
const SOURCE: &str = include_str!("../../../../content/original/ported/yijiang/rejingce.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/yijiang/yijiang.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    other: PlayerId,
    jiu: [CardId; 2],
    trick: CardId,
    weapon: CardId,
}

fn inject(state: &mut GameState, owner: PlayerId, def: u32, kind: CardKind, suit: Suit) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank: 7,
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
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
    tx.state_mut().revision = 1;
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    {
        let owner_state = tx.state_mut().players.get_mut(owner).unwrap();
        owner_state.character_def = GUOHUAI;
        owner_state.max_hp = 3;
        owner_state.hp = 3;
    }

    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in tx.state().players.alive_by_seat() {
        effects.push(EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        });
    }
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(REJINGCE),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let jiu = [
        inject(tx.state_mut(), owner, 92_001, CardKind::Jiu, Suit::Spade),
        inject(tx.state_mut(), owner, 92_002, CardKind::Jiu, Suit::Spade),
    ];
    let trick = inject(
        tx.state_mut(),
        owner,
        92_003,
        CardKind::TrickWuzhong,
        Suit::Heart,
    );
    let weapon = inject(tx.state_mut(), owner, 92_004, CardKind::Weapon, Suit::Club);
    Setup {
        state: tx.commit().state,
        owner,
        other,
        jiu,
        trick,
        weapon,
    }
}

fn play(state: GameState, actor: PlayerId, card: CardId, key: u64) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card,
                targets: Vec::new(),
                card_targets: Vec::new(),
            },
        },
    )
}

fn end_play(state: GameState, actor: PlayerId, key: u64) -> (GameState, PipelineResult) {
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

fn accept_optional(state: GameState, actor: PlayerId, key: u64) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    )
}

fn hand_limit(state: &GameState, player: PlayerId) -> i32 {
    match query(
        state,
        &QueryRequest::HandLimit { player },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("unexpected hand-limit response: {other:?}"),
    }
}

#[test]
fn package_publishes_complete_guohuai_in_abi53_package() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 637);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == GUOHUAI)
        .unwrap();
    assert_eq!(character.skills, [REJINGCE]);
    let compiled = content_compiler::compile_source_named("rejingce", SOURCE).unwrap();
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .unwrap();
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn repeated_suit_is_bounded_while_logical_categories_accumulate() {
    let setup = setup(2_026_071_901);
    assert_eq!(hand_limit(&setup.state, setup.owner), 3);

    let (state, first) = play(setup.state, setup.owner, setup.jiu[0], 1);
    assert!(matches!(first.result, CommandResult::Accepted { .. }));
    assert_eq!(hand_limit(&state, setup.owner), 4);
    assert_eq!(state.turn.play_card_category_count(setup.owner), 1);

    let (state, repeated) = play(state, setup.owner, setup.jiu[1], 2);
    assert!(matches!(repeated.result, CommandResult::Accepted { .. }));
    assert_eq!(hand_limit(&state, setup.owner), 4);
    assert_eq!(state.turn.play_card_category_count(setup.owner), 1);

    let (state, trick) = play(state, setup.owner, setup.trick, 3);
    assert!(matches!(trick.result, CommandResult::Accepted { .. }));
    assert_eq!(hand_limit(&state, setup.owner), 5);
    assert_eq!(state.turn.play_card_category_count(setup.owner), 2);

    let (state, weapon) = play(state, setup.owner, setup.weapon, 4);
    assert!(matches!(weapon.result, CommandResult::Accepted { .. }));
    assert_eq!(hand_limit(&state, setup.owner), 6);
    assert_eq!(state.turn.play_card_category_count(setup.owner), 3);
}

#[test]
fn phase_end_draws_category_count_and_resets_histories_at_the_right_boundaries() {
    let setup = setup(2_026_071_902);
    let (state, _) = play(setup.state, setup.owner, setup.jiu[0], 10);
    let (state, _) = play(state, setup.owner, setup.trick, 11);
    let (state, _) = play(state, setup.owner, setup.weapon, 12);
    let hand_before = state.hand_count(setup.owner);

    let (state, optional) = end_play(state, setup.owner, 13);
    assert!(matches!(optional.result, CommandResult::Accepted { .. }));
    assert!(optional.choice.is_some());
    let (state, accepted) = accept_optional(state, setup.owner, 14);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.owner), hand_before + 3);
    assert!(!state.has_pending_choice());

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: setup.owner,
            phase: Phase::Play.to_u8(),
        }],
    )
    .unwrap();
    assert_eq!(tx.state().turn.play_card_category_count(setup.owner), 0);
    assert_eq!(
        tx.state().turn.distinct_hand_card_suits_used(setup.owner),
        3
    );
    assert_eq!(hand_limit(tx.state(), setup.owner), 6);

    let (state, empty_end) = end_play(tx.commit().state, setup.owner, 15);
    assert!(matches!(empty_end.result, CommandResult::Accepted { .. }));
    assert!(empty_end.choice.is_none());

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: setup.other,
            phase: Phase::Prepare.to_u8(),
        }],
    )
    .unwrap();
    assert_eq!(
        tx.state().turn.distinct_hand_card_suits_used(setup.owner),
        0
    );
    assert_eq!(hand_limit(tx.state(), setup.owner), 3);
}
