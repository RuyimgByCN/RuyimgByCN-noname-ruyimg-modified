//! Golden coverage for 用间陈到「将烈」rewritten from the legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, CardMoveReason, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult,
    DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::hash::canonical_state_hash;
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const NSJIANGLIE: u32 = 3_311_281_717;
const NS_CHENDAO: u32 = 3_933_499_072;
const SOURCE: &str = include_str!("../../../../content/original/ported/offline/nsjianglie.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/offline/offline.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package())
        .unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let target = seats[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(seats.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(NSJIANGLIE),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().revision = 1;
    (tx.commit().state, owner, target)
}

fn inject_hand(
    state: &mut GameState,
    player: PlayerId,
    kind: CardKind,
    suit: Suit,
    def: u32,
) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn play_sha(
    mut state: GameState,
    owner: PlayerId,
    target: PlayerId,
    key: u64,
) -> (GameState, PipelineResult) {
    let sha = inject_hand(
        &mut state,
        owner,
        CardKind::Sha,
        Suit::Spade,
        154_000 + key as u32,
    );
    submit(
        state,
        owner,
        CommandKind::PlayCard {
            card: sha,
            targets: vec![target],
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
    submit(
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

#[test]
fn package_publishes_abi155_with_exact_jianglie_contract_and_character() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 691);
    assert_eq!(package.skill_index.len(), 851);
    assert_eq!(package.character_index.len(), 385);
    assert_eq!(package.trigger_index.len(), 587);
    assert_eq!(package.modules.len(), 724);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == NSJIANGLIE)
        .unwrap();
    assert_eq!(trigger.event, "card_targeted");
    assert_eq!(
        trigger.filter,
        "owner_source_sha_target_alive_hand_nonempty"
    );
    assert!(trigger.optional);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == NS_CHENDAO)
        .unwrap();
    assert_eq!(character.name_key, "ns_chendao");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [NSJIANGLIE]);

    let compiled = content_compiler::compile_source_named("nsjianglie", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 154);
    let expected = &compiled.modules[0];
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);
}

#[test]
fn mixed_hand_is_public_and_target_chooses_the_discarded_color() {
    let (mut state, owner, target) = setup(2_026_072_504);
    let red_a = inject_hand(&mut state, target, CardKind::Other, Suit::Heart, 154_101);
    let black = inject_hand(&mut state, target, CardKind::Other, Suit::Club, 154_102);
    let red_b = inject_hand(&mut state, target, CardKind::Other, Suit::Diamond, 154_103);
    let (state, offered) = play_sha(state, owner, target, 1);
    let confirmation = offered.choice.unwrap();
    assert_eq!(confirmation.observer, owner);
    assert_eq!(confirmation.kind, ChoiceKind::Confirm);

    let (state, revealed) = resolve(state, owner, vec![1], false, 2);
    let color = revealed.choice.unwrap();
    assert_eq!(color.observer, target);
    assert_eq!(color.kind, ChoiceKind::Custom);
    assert_eq!(
        color
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [0b01, 0b10]
    );
    assert!(revealed.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardsRevealed { cards, to }
            if cards == &[red_a, black, red_b] && to.is_empty()
    )));

    let (state, discarded) = resolve(state, target, vec![0b01], false, 3);
    assert!(matches!(discarded.result, CommandResult::Accepted { .. }));
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(red_a).unwrap().zone, discard);
    assert_eq!(state.cards.get(red_b).unwrap().zone, discard);
    assert_eq!(state.cards.get(black).unwrap().owner, target);
    assert_eq!(
        state.cards.get(black).unwrap().zone,
        state.zones.find(ZoneKind::Hand, target).unwrap()
    );
    assert!(discarded.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardsMoved {
            cards,
            reason: CardMoveReason::Discard,
            ..
        } if cards == &[red_a, red_b]
    )));
}

#[test]
fn single_color_discards_without_a_color_choice_and_empty_hand_skips_the_trigger() {
    let (mut state, owner, target) = setup(2_026_072_505);
    let red_a = inject_hand(&mut state, target, CardKind::Other, Suit::Heart, 154_201);
    let red_b = inject_hand(&mut state, target, CardKind::Other, Suit::Diamond, 154_202);
    let (state, offered) = play_sha(state, owner, target, 10);
    assert_eq!(offered.choice.unwrap().observer, owner);
    let (state, resolved) = resolve(state, owner, vec![1], false, 11);
    assert_ne!(
        state.resolution.continuation_kind,
        ContinuationKind::RevealHandChooseDiscardColor
    );
    assert!(resolved.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardsRevealed { cards, to }
            if cards == &[red_a, red_b] && to.is_empty()
    )));
    assert_eq!(state.hand_count(target), 0);

    let (state, owner, target) = setup(2_026_072_506);
    let before = canonical_state_hash(&state);
    let (state, output) = play_sha(state, owner, target, 20);
    assert!(output
        .choice
        .as_ref()
        .is_none_or(|choice| choice.observer != owner || choice.kind != ChoiceKind::Confirm));
    assert!(!output
        .events
        .iter()
        .any(|event| matches!(event, DomainEvent::CardsRevealed { .. })));
    assert_ne!(canonical_state_hash(&state), before);
}

#[test]
fn unoffered_color_is_rejected_without_mutating_the_choice_state() {
    let (mut state, owner, target) = setup(2_026_072_507);
    inject_hand(&mut state, target, CardKind::Other, Suit::Heart, 154_301);
    inject_hand(&mut state, target, CardKind::Other, Suit::Spade, 154_302);
    let (state, _) = play_sha(state, owner, target, 30);
    let (state, _) = resolve(state, owner, vec![1], false, 31);
    let before = canonical_state_hash(&state);
    let (same, rejected) = resolve(state, target, vec![4], false, 32);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
}
