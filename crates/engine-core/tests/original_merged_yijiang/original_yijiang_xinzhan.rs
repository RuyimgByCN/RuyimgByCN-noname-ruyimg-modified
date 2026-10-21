//! Golden coverage for YIJIANG「心战」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    ObserverKind, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::{canonical_state_hash, project_observer};
use rule_api::EffectRequest;

const XINZHAN: u32 = 3_275_354_148;
const HUILEI: u32 = 2_889_506_960;
const MASU: u32 = 3_141_577_158;
const YIJIANG: &[u8] =
    include_bytes!("../../../../content/original/ported/yijiang/yijiang.npack.json");
const OLD: &[u8] = include_bytes!("../../../../content/original/ported/old/old.npack.json");
const SOURCE: &str = include_str!("../../../../content/original/ported/yijiang/xinzhan.skill");

fn started(seed: u64) -> (GameState, PlayerId, PlayerId, SkillId) {
    let package = content_schema::ContentPackage::from_json_bytes(YIJIANG).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    let players = tx.state().players.alive_by_seat();
    let owner = tx.state().turn.current;
    let other = players.into_iter().find(|player| *player != owner).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(XINZHAN),
        }],
    )
    .unwrap();
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == XINZHAN)
        .unwrap();
    (tx.state().clone(), owner, other, skill)
}

fn inject_top(state: &mut GameState, cards: &[(Suit, u32)]) -> Vec<CardId> {
    let draw = state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    cards
        .iter()
        .map(|(suit, def)| {
            let card = state.cards.alloc(Card {
                id: CardId::NONE,
                def: CardDefId::new(*def),
                kind: CardKind::Other,
                suit: *suit,
                rank: 7,
                weapon_range: 0,
                zone: draw,
                owner: PlayerId::NONE,
                face_up: false,
            });
            state.zones.get_mut(draw).unwrap().cards.push(card);
            card
        })
        .collect()
}

fn activate(
    state: GameState,
    owner: PlayerId,
    skill: SkillId,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: Vec::new(),
                card_targets: Vec::new(),
            },
        },
    )
}

fn resolve(
    state: GameState,
    owner: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = engine_api::ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

#[test]
fn packages_publish_exact_skill_character_and_dependency() {
    let yijiang = content_schema::ContentPackage::from_json_bytes(YIJIANG).unwrap();
    assert_eq!(yijiang.manifest.rule_vm_abi_version, 637);
    let compiled = content_compiler::compile_source_named("xinzhan", SOURCE).unwrap();
    let expected = &compiled.modules[0];
    let actual = yijiang
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);
    let active = yijiang
        .active_index
        .iter()
        .find(|active| active.skill_def_id == XINZHAN)
        .unwrap();
    assert_eq!(active.uses_per_turn, Some(1));
    assert!(active.hand_gt_max_hp);
    assert!(!active.hand_gt_hp);

    let old = content_schema::ContentPackage::from_json_bytes(OLD).unwrap();
    assert!(old.manifest.dependencies.iter().any(|dependency| {
        dependency.namespace == "official"
            && dependency.package_id == "yijiang"
            && dependency.version == "0.1.0"
    }));
    let masu = old
        .character_index
        .iter()
        .find(|character| character.id == MASU)
        .unwrap();
    assert_eq!((masu.name_key.as_str(), masu.max_hp), ("masu", 3));
    assert_eq!(masu.skills, [XINZHAN, HUILEI]);
}

#[test]
fn activation_requires_hand_above_max_hp_and_is_limited_once() {
    let (mut state, owner, _, skill) = started(2_026_071_801);
    let hand = state.hand_count(owner);
    state.players.get_mut(owner).unwrap().max_hp = hand;
    let before = canonical_state_hash(&state);
    let (same, rejected) = activate(state, owner, skill, 1);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);

    let mut state = same;
    state.players.get_mut(owner).unwrap().max_hp = hand - 1;
    inject_top(&mut state, &[(Suit::Heart, 98_000)]);
    let (state, accepted) = activate(state, owner, skill, 2);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert!(accepted.choice.is_some());
    let (state, cancelled) = resolve(state, owner, Vec::new(), true, 3);
    assert!(matches!(cancelled.result, CommandResult::Accepted { .. }));
    assert_eq!(state.skills.get(skill).unwrap().use_count, 1);
    let (same, rejected) = activate(state.clone(), owner, skill, 4);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), canonical_state_hash(&state));
}

#[test]
fn private_peek_offers_only_hearts_and_selected_subset_is_publicly_gained() {
    let (mut state, owner, other, skill) = started(2_026_071_802);
    state.players.get_mut(owner).unwrap().max_hp = 3;
    let top = inject_top(
        &mut state,
        &[
            (Suit::Heart, 98_010),
            (Suit::Spade, 98_011),
            (Suit::Heart, 98_012),
        ],
    );
    let draw = state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let (state, pipeline) = activate(state, owner, skill, 1);
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseGainTopCardsBySuit
    );
    assert!(pipeline.events.iter().any(|event| matches!(
        event,
        engine_api::DomainEvent::CardsRevealed { cards, to }
            if cards == &top && to == &vec![owner]
    )));
    let choice = pipeline.choice.unwrap();
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        vec![top[0].raw(), top[2].raw()]
    );
    let owner_view = project_observer(&state, ObserverKind::Player(owner));
    let other_view = project_observer(&state, ObserverKind::Player(other));
    assert!(top.iter().all(|card| owner_view
        .known_cards
        .iter()
        .any(|known| known.id == *card && known.def.is_some())));
    assert!(top
        .iter()
        .all(|card| other_view.known_cards.iter().all(|known| known.id != *card)));

    let (state, pipeline) = resolve(state, owner, vec![top[0].raw()], false, 2);
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    assert_eq!(state.cards.get(top[0]).unwrap().zone, hand);
    assert_eq!(state.cards.get(top[0]).unwrap().owner, owner);
    let draw_cards = &state.zones.get(draw).unwrap().cards;
    assert_eq!(&draw_cards[draw_cards.len() - 2..], &[top[1], top[2]]);
    assert!(pipeline.events.iter().any(|event| matches!(
        event,
        engine_api::DomainEvent::CardsRevealed { cards, to }
            if cards == &vec![top[0]] && to.is_empty()
    )));
}

#[test]
fn cancellation_and_no_matching_suit_leave_the_draw_top_unchanged() {
    let (mut state, owner, _, skill) = started(2_026_071_803);
    state.players.get_mut(owner).unwrap().max_hp = 3;
    let top = inject_top(
        &mut state,
        &[
            (Suit::Heart, 98_020),
            (Suit::Club, 98_021),
            (Suit::Diamond, 98_022),
        ],
    );
    let draw = state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let (state, pipeline) = activate(state, owner, skill, 1);
    assert!(pipeline.choice.is_some());
    let (state, pipeline) = resolve(state, owner, Vec::new(), true, 2);
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    let cards = &state.zones.get(draw).unwrap().cards;
    assert_eq!(&cards[cards.len() - 3..], top.as_slice());

    let (mut state, owner, _, skill) = started(2_026_071_804);
    state.players.get_mut(owner).unwrap().max_hp = 3;
    let top = inject_top(
        &mut state,
        &[
            (Suit::Spade, 98_030),
            (Suit::Club, 98_031),
            (Suit::Diamond, 98_032),
        ],
    );
    let draw = state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let (state, pipeline) = activate(state, owner, skill, 3);
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert!(pipeline.choice.is_none());
    let cards = &state.zones.get(draw).unwrap().cards;
    assert_eq!(&cards[cards.len() - 3..], top.as_slice());
}

#[test]
fn forged_nonheart_selection_is_rejected_without_mutation() {
    let (mut state, owner, _, skill) = started(2_026_071_805);
    state.players.get_mut(owner).unwrap().max_hp = 3;
    let top = inject_top(
        &mut state,
        &[
            (Suit::Heart, 98_040),
            (Suit::Spade, 98_041),
            (Suit::Heart, 98_042),
        ],
    );
    let (state, pipeline) = activate(state, owner, skill, 1);
    assert!(pipeline.choice.is_some());
    let before = canonical_state_hash(&state);
    let (same, rejected) = resolve(state, owner, vec![top[1].raw()], false, 2);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
}
