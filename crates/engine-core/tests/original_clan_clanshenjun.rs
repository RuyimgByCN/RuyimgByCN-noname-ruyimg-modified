//! Golden coverage for clan 神君 (clanshenjun): locked clan skill that reveals
//! and gaintags same-kind hand cards when someone uses 杀/锦囊, then lets the
//! owner play one tagged card at phase end.
//!
//! Exercises the primitives added for the clan skill family's second cluster:
//!   - `owner_alive_hand_same_kind_event_card` trigger filter (同名牌),
//!   - `clanshenjun_reveal` effect (CardsRevealed + gaintag by skill def id),
//!   - `owner_alive_has_skill_tagged_cards` phase-end gate,
//!   - `clanshenjun_use` choice continuation (tagged card -> target -> play).
//! 同名牌 is approximated by effective_card_kind equality; the phase-end window
//! is the owner's own play-phase end (see the .skill comment).

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const CLANSHENJUN: u32 = 2_483_457_953;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/clan/clan.npack.json");

fn start() -> (engine_core::Engine, PlayerId, PlayerId) {
    let mut engine = Engine::new();
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_810,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&pkg).unwrap();
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    (engine, owner, other)
}

fn strip_and_add_clanshenjun(tx: &mut Transaction, owner: PlayerId) -> engine_api::SkillId {
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    let mut effects = effects;
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(CLANSHENJUN),
    });
    run_effects(tx, effects).unwrap();
    tx.state()
        .skills
        .sorted_live()
        .iter()
        .find(|skill| skill.def.raw() == CLANSHENJUN)
        .unwrap()
        .id
}

fn clear_hands(tx: &mut Transaction) {
    let seats: Vec<_> = tx.state().players.alive_by_seat();
    for player in seats {
        let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
        let cards: Vec<_> = tx.state().hand_cards(player);
        if !cards.is_empty() {
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
    }
}

fn inject_hand(state: &mut engine_core::state::GameState, player: PlayerId, kind: CardKind) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(170_100 + kind.tag() as u32),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: true,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn dispatch_card_used(tx: &mut Transaction, user: PlayerId, card: CardId) {
    let fired = dispatch_for_events(
        tx,
        &[DomainEvent::CardUsed {
            player: user,
            card,
            targets: vec![],
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => {
            let _ = run_effects(tx, effects);
        }
        Err(engine_core::effect::EffectError::ChoiceRequested) => {}
        Err(e) => panic!("card_used dispatch error: {e:?}"),
    }
}

#[test]
fn committed_package_publishes_clanshenjun_triggers() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let triggers: Vec<_> = pkg
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == CLANSHENJUN)
        .collect();
    assert_eq!(triggers.len(), 2);
    assert!(triggers.iter().any(|trigger| {
        trigger.event == "card_used" && trigger.filter == "owner_alive_hand_same_kind_event_card"
    }));
    assert!(triggers.iter().any(|trigger| {
        trigger.event == "phase_end" && trigger.filter == "owner_alive_has_skill_tagged_cards"
    }));
}

#[test]
fn same_kind_use_reveals_and_tags_matching_hand_cards() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    strip_and_add_clanshenjun(&mut tx, owner);
    clear_hands(&mut tx);
    // The owner holds a 杀; the other player uses a 杀 → same-kind match.
    let owner_sha = inject_hand(tx.state_mut(), owner, CardKind::Sha);
    let owner_tao = inject_hand(tx.state_mut(), owner, CardKind::Tao);
    let used_sha = inject_hand(tx.state_mut(), other, CardKind::Sha);
    dispatch_card_used(&mut tx, other, used_sha);
    // The matching 杀 was tagged with the skill def; the 桃 was not.
    assert!(tx
        .state()
        .card_has_content_tag(owner_sha, CLANSHENJUN));
    assert!(!tx
        .state()
        .card_has_content_tag(owner_tao, CLANSHENJUN));
    // Both cards stay in hand (reveal, not discard).
    assert!(tx.state().hand_cards(owner).contains(&owner_sha));
    assert!(tx.state().hand_cards(owner).contains(&owner_tao));
}

#[test]
fn non_matching_kind_does_not_tag() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    strip_and_add_clanshenjun(&mut tx, owner);
    clear_hands(&mut tx);
    // Owner holds only 桃; the other uses a 杀 → no same-kind match.
    let owner_tao = inject_hand(tx.state_mut(), owner, CardKind::Tao);
    let used_sha = inject_hand(tx.state_mut(), other, CardKind::Sha);
    dispatch_card_used(&mut tx, other, used_sha);
    assert!(!tx
        .state()
        .card_has_content_tag(owner_tao, CLANSHENJUN));
}

#[test]
fn phase_end_use_plays_tagged_card_on_chosen_target() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    let skill = strip_and_add_clanshenjun(&mut tx, owner);
    clear_hands(&mut tx);
    // Tag a 杀 via the reveal path, then end the owner's play phase.
    let owner_sha = inject_hand(tx.state_mut(), owner, CardKind::Sha);
    let used_sha = inject_hand(tx.state_mut(), other, CardKind::Sha);
    dispatch_card_used(&mut tx, other, used_sha);
    assert!(tx
        .state()
        .card_has_content_tag(owner_sha, CLANSHENJUN));
    let other_hp = tx.state().players.get(other).unwrap().hp;

    // 出牌阶段结束：神君 opens the tagged-card use choice.
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => {
            let _ = run_effects(&mut tx, effects);
        }
        Err(engine_core::effect::EffectError::ChoiceRequested) => {}
        Err(e) => panic!("phase_end dispatch error: {e:?}"),
    }
    assert!(
        tx.state().has_pending_choice(),
        "神君 use choice must open at phase end"
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        engine_core::state::ContinuationKind::ClanshenjunUseCard
    );

    // Stage 0: pick the tagged 杀.
    let choice_id = engine_api::ChoiceId::new(tx.state().resolution.pending_choice_id);
    let (state, picked) = apply_command(
        tx.clone().commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![owner_sha.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(picked.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        engine_core::state::ContinuationKind::ClanshenjunUseCard
    );

    // Stage 1: pick the target; the 杀 plays and deals 1.
    let stage1_choice = engine_api::ChoiceId::new(state.resolution.pending_choice_id);
    let (state, used) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: stage1_choice,
                selections: vec![other.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(used.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(other).unwrap().hp, other_hp - 1);
    assert!(!state.hand_cards(owner).contains(&owner_sha));
}
