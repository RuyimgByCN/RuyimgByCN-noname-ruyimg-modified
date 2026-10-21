//! Golden coverage for 界郭淮「精策」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DECADEJINGCE: u32 = 2_705_593_131;
const RE_GUOHUAI: u32 = 3_763_685_284;
const SOURCE: &str = include_str!("../../../../content/original/ported/refresh/decadejingce.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/refresh/refresh.npack.json");

fn inject(state: &mut GameState, owner: PlayerId, def: u32, suit: Suit) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Jiu,
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

fn setup(seed: u64, suits: &[Suit]) -> (GameState, PlayerId, PlayerId, Vec<CardId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    tx.state_mut().revision = 1;
    let owner = tx.state().turn.current;
    let next = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    {
        let player = tx.state_mut().players.get_mut(owner).unwrap();
        player.character_def = RE_GUOHUAI;
        player.max_hp = 2;
        player.hp = 2;
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
            skill_def: SkillDefId::new(DECADEJINGCE),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let cards = suits
        .iter()
        .enumerate()
        .map(|(index, suit)| inject(tx.state_mut(), owner, 232_000 + index as u32, *suit))
        .collect();
    (tx.commit().state, owner, next, cards)
}

fn command(
    state: GameState,
    actor: PlayerId,
    key: u64,
    kind: CommandKind,
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

fn play(state: GameState, actor: PlayerId, card: CardId, key: u64) -> GameState {
    let (state, result) = command(
        state,
        actor,
        key,
        CommandKind::PlayCard {
            card,
            targets: Vec::new(),
            card_targets: Vec::new(),
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

fn end(state: GameState, actor: PlayerId, key: u64) -> (GameState, PipelineResult) {
    command(state, actor, key, CommandKind::EndPhase)
}

fn choose(state: GameState, actor: PlayerId, token: u32, key: u64) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = command(
        state,
        actor,
        key,
        CommandKind::ResolveChoice {
            choice_id,
            selections: vec![token],
            cancelled: false,
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

fn reach_end(state: GameState, owner: PlayerId, key: u64) -> (GameState, PipelineResult) {
    let (state, play_end) = end(state, owner, key);
    assert!(matches!(play_end.result, CommandResult::Accepted { .. }));
    let (state, discard_end) = end(state, owner, key + 1);
    assert!(matches!(discard_end.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.phase, Phase::End);
    (state, discard_end)
}

#[test]
fn package_publishes_rewritten_skill_and_complete_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 251);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == RE_GUOHUAI)
        .unwrap();
    assert_eq!(character.skills, [DECADEJINGCE]);
    let compiled = content_compiler::compile_source_named("decadejingce", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 232);
}

#[test]
fn insufficient_suits_choose_one_complete_extra_phase() {
    let (mut state, owner, next, cards) = setup(2_026_072_801, &[Suit::Spade, Suit::Spade]);
    for (index, card) in cards.into_iter().enumerate() {
        state = play(state, owner, card, 10 + index as u64);
    }
    let (state_after_end, optional) = reach_end(state, owner, 20);
    assert!(optional.choice.is_some());
    let state = choose(state_after_end, owner, 1, 22);
    assert!(state.has_pending_choice());
    let hand_before = state.hand_count(owner);
    let state = choose(state, owner, 1, 23);
    assert_eq!(state.turn.phase, Phase::End);
    let (state, _) = end(state, owner, 24);
    assert_eq!(state.turn.current, owner);
    assert_eq!(state.turn.phase, Phase::Draw);
    assert_eq!(state.hand_count(owner), hand_before + 2);

    let (state, result) = end(state, owner, 25);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.current, next);
    assert_ne!(state.turn.phase, Phase::Play);
}

#[test]
fn enough_suits_automatically_runs_draw_then_play() {
    let (mut state, owner, next, cards) = setup(2_026_072_802, &[Suit::Spade, Suit::Heart]);
    for (index, card) in cards.into_iter().enumerate() {
        state = play(state, owner, card, 30 + index as u64);
    }
    let (state, optional) = reach_end(state, owner, 40);
    assert!(optional.choice.is_some());
    let hand_before = state.hand_count(owner);
    let state = choose(state, owner, 1, 42);
    assert!(!state.has_pending_choice());
    assert_eq!(state.turn.phase, Phase::End);
    let (state, _) = end(state, owner, 43);
    assert_eq!(state.turn.phase, Phase::Draw);
    assert_eq!(state.hand_count(owner), hand_before + 2);

    let (state, _) = end(state, owner, 44);
    assert_eq!(state.turn.current, owner);
    assert_eq!(state.turn.phase, Phase::Play);
    let (state, _) = end(state, owner, 45);
    assert_eq!(state.turn.current, next);
}

#[test]
fn card_count_below_hp_does_not_offer_jingce() {
    let (state, owner, next, cards) = setup(2_026_072_803, &[Suit::Diamond]);
    let state = play(state, owner, cards[0], 50);
    let (state, result) = reach_end(state, owner, 51);
    assert!(result.choice.is_none());
    let (state, result) = end(state, owner, 53);
    assert!(result.choice.is_none());
    assert_eq!(state.turn.current, next);
}
