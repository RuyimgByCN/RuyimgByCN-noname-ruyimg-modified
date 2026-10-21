//! Golden for MOBILE「精策」的实体用牌/响应弃置历史。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit, ZoneId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{turn_flag, Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const MOBILEJINGCE: u32 = 3_410_757_233;
const MOBILEJINGCE_COUNT: u32 = 3_753_058_142;
const XIN_GUOHUAI: u32 = 2_296_836_656;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/mobile/mobile.npack.json");

fn start(seed: u64) -> Command {
    Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(seed),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    }
}

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone: ZoneId,
    kind: CardKind,
    suit: Suit,
    rank: u8,
) -> CardId {
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(84_000 + kind.tag() as u32 * 20 + rank as u32),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn remove_all_skills(tx: &mut Transaction) {
    let effects = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(tx, effects).unwrap();
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.submit(start(seed));
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    remove_all_skills(&mut tx);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(MOBILEJINGCE),
            },
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::Play.to_u8(),
            },
        ],
    )
    .unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().hp = 2;
    (tx, owner, other)
}

fn clear_hand(tx: &mut Transaction, player: PlayerId) {
    let cards = tx.state().hand_cards(player);
    if cards.is_empty() {
        return;
    }
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards,
            to: discard,
            from: Some(hand),
        }],
    )
    .unwrap();
}

fn pass_wuxie(
    mut state: GameState,
    mut pipeline: PipelineResult,
    mut key: u64,
) -> (GameState, PipelineResult) {
    while state.has_pending_choice()
        && state.resolution.continuation_kind == ContinuationKind::WuxieWindow
    {
        let choice = pipeline.choice.clone().expect("wuxie choice");
        key += 1;
        (state, pipeline) = apply_command(
            state,
            Command {
                actor: choice.observer,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id: choice.choice_id,
                    selections: vec![],
                    cancelled: true,
                },
            },
        );
        assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    }
    (state, pipeline)
}

#[test]
fn package_declares_rewritten_skill_helper_and_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 252);
    assert!(package.skill_index.iter().any(|skill| {
        skill.skill_def_id == MOBILEJINGCE && skill.name == "mobilejingce" && !skill.locked
    }));
    assert!(package.skill_index.iter().any(|skill| {
        skill.skill_def_id == MOBILEJINGCE_COUNT
            && skill.name == "mobilejingce_count"
            && skill.locked
    }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == MOBILEJINGCE
            && trigger.event == "phase_end"
            && trigger.filter == "owner_is_target_use_response_discarded_at_least_hp"
            && trigger.optional
    }));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == XIN_GUOHUAI)
        .unwrap();
    assert_eq!(character.name_key, "xin_guohuai");
    assert_eq!(character.group, "wei");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, vec![MOBILEJINGCE]);
}

#[test]
fn used_and_responded_physical_cards_share_the_turn_threshold() {
    let (mut tx, owner, other) = setup(20_260_718);
    clear_hand(&mut tx, owner);
    clear_hand(&mut tx, other);
    let owner_hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let other_hand = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    let used = inject(&mut tx, owner, owner_hand, CardKind::Other, Suit::Heart, 3);
    let responded = inject(&mut tx, other, other_hand, CardKind::Shan, Suit::Club, 4);
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();

    run_effects(
        &mut tx,
        vec![EffectRequest::UseCard {
            player: owner,
            card: used,
            targets: vec![],
        }],
    )
    .unwrap();
    run_effects(
        &mut tx,
        vec![
            EffectRequest::MoveCards {
                cards: vec![responded],
                to: discard,
                from: Some(other_hand),
            },
            EffectRequest::CardResponded {
                player: other,
                respond_to: owner,
                card: responded,
                response_kind: CardKind::Shan.tag(),
            },
        ],
    )
    .unwrap();
    assert_eq!(tx.state().turn.flag(turn_flag::USE_RESPONSE_DISCARDED), 2);

    let hand_before = tx.state().hand_count(owner);
    let paused = run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::End.to_u8(),
        }],
    )
    .unwrap_err();
    assert_eq!(paused, EffectError::ChoiceRequested);
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(MOBILEJINGCE as u64),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand_before + 2);
}

#[test]
fn successful_delayed_trick_placement_does_not_count_as_discarded() {
    let (mut tx, owner, other) = setup(20_260_719);
    clear_hand(&mut tx, owner);
    clear_hand(&mut tx, other);
    tx.state_mut().revision = 1;
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let le = inject(&mut tx, owner, hand, CardKind::TrickLe, Suit::Heart, 6);
    let (state, pipeline) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: le,
                targets: vec![other],
                card_targets: vec![],
            },
        },
    );
    let (state, _) = pass_wuxie(state, pipeline, 1);
    assert_eq!(state.turn.flag(turn_flag::USE_RESPONSE_DISCARDED), 0);
    let judge = state.zones.find(ZoneKind::Judge, other).unwrap();
    assert!(state.zones.get(judge).unwrap().cards.contains(&le));
}

#[test]
fn nullified_delayed_trick_and_the_wuxie_both_count() {
    let (mut tx, owner, other) = setup(20_260_720);
    clear_hand(&mut tx, owner);
    clear_hand(&mut tx, other);
    tx.state_mut().revision = 1;
    let owner_hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let other_hand = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    let le = inject(
        &mut tx,
        owner,
        owner_hand,
        CardKind::TrickLe,
        Suit::Spade,
        6,
    );
    let wuxie = inject(
        &mut tx,
        other,
        other_hand,
        CardKind::TrickWuxie,
        Suit::Club,
        12,
    );
    let (state, pipeline) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: le,
                targets: vec![other],
                card_targets: vec![],
            },
        },
    );
    let choice = pipeline.choice.unwrap();
    assert_eq!(choice.observer, other);
    let (state, pipeline) = apply_command(
        state,
        Command {
            actor: other,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![wuxie.raw()],
                cancelled: false,
            },
        },
    );
    let (state, _) = pass_wuxie(state, pipeline, 2);
    assert_eq!(state.turn.flag(turn_flag::USE_RESPONSE_DISCARDED), 2);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let cards = &state.zones.get(discard).unwrap().cards;
    assert!(cards.contains(&le));
    assert!(cards.contains(&wuxie));
}
