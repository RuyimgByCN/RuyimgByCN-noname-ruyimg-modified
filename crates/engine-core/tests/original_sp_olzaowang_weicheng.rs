//! Golden for sp 早亡 (olzaowang) + 微城 (weicheng) rewrites.
//!
//! olzaowang: 限定技 — play phase, once per game: a chosen player gains 1 max HP,
//! heals 1, and draws 3.
//! weicheng: when cards move from the owner's hand to another player's hand, if the
//! owner's hand size is below their HP, they may draw 1.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const OLZAOWANG: u32 = 3_649_255_847;
const WEICHENG: u32 = 4_091_204_654;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn start() -> (Engine, PlayerId, PlayerId) {
    let mut engine = Engine::new();
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    engine.load_package(&pkg).unwrap();
    let result = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_261_701,
            ruleset: "identity-standard".into(),
        },
    });
    assert!(matches!(result, CommandResult::Accepted { .. }));
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

fn add_skill(tx: &mut Transaction, owner: PlayerId, def: u32) -> engine_api::SkillId {
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
        skill_def: SkillDefId::new(def),
    });
    run_effects(tx, effects).unwrap();
    tx.state()
        .skills
        .sorted_live()
        .iter()
        .find(|skill| skill.def.raw() == def)
        .unwrap()
        .id
}

fn inject_hand(
    tx: &mut Transaction,
    player: PlayerId,
    kind: CardKind,
) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
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
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn package_declares_olzaowang_active_and_weicheng_trigger() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pkg.active_index.iter().any(|a| a.skill_def_id == OLZAOWANG));
    assert!(pkg.trigger_index.iter().any(|t| {
        t.skill_def_id == WEICHENG
            && t.event == "cards_given"
            && t.optional
    }));
}

#[test]
fn olzaowang_grants_max_hp_heal_and_draw_once() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    let skill = add_skill(&mut tx, owner, OLZAOWANG);
    tx.state_mut().turn.phase = Phase::Play;
    let other_max_before = tx.state().players.get(other).unwrap().max_hp;
    let other_hp_before = tx.state().players.get(other).unwrap().hp;
    let other_hand_before = tx.state().hand_count(other);

    let (state, first) = apply_command(
        tx.commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![other],
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(first.result, CommandResult::Accepted { .. }),
        "早亡 activation accepted: {:?}",
        first.result
    );
    assert_eq!(state.players.get(other).unwrap().max_hp, other_max_before + 1);
    assert_eq!(state.players.get(other).unwrap().hp, other_hp_before + 1);
    assert_eq!(
        state.hand_count(other),
        other_hand_before + 3,
        "目标摸三张"
    );

    // 限定技 — second activation rejected.
    let (_, second) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![owner],
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(second.result, CommandResult::Rejected { .. }),
        "限定技 only fires once"
    );
}

#[test]
fn weicheng_draws_when_hand_below_hp_after_giving() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    let _skill = add_skill(&mut tx, owner, WEICHENG);
    // Owner is the lord: hp 5, hand 4 → hand < hp holds.
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let other_hand = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    let give = tx.state().hand_cards(owner)[0];
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![give],
            to: other_hand,
            from: Some(hand),
        }],
    )
    .unwrap();
    // Set the owner's HP above their hand so the draw condition holds.
    tx.state_mut().players.get_mut(owner).unwrap().hp = 6;
    let owner_hand_before = tx.state().hand_count(owner);
    assert!((owner_hand_before as i32) < 6);

    // Dispatch the cards-given event → optional trigger opens a confirm choice.
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardsGiven {
            giver: owner,
            recipient: other,
            cards: vec![give],
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => {
            run_effects(&mut tx, effects).unwrap();
        }
        Err(engine_core::effect::EffectError::ChoiceRequested) => {}
        Err(e) => panic!("dispatch error: {e:?}"),
    }
    assert!(
        tx.state().has_pending_choice(),
        "optional 微城 opens a confirm choice"
    );

    // Confirm → owner draws 1 (hand below HP).
    let choice_id = engine_api::ChoiceId::new(tx.state().resolution.pending_choice_id);
    let (state, confirmed) = apply_command(
        tx.clone().commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(confirmed.result, CommandResult::Accepted { .. }),
        "confirm accepted: {:?}",
        confirmed.result
    );
    assert_eq!(
        state.hand_count(owner),
        owner_hand_before + 1,
        "微城 draws 1"
    );
}

#[test]
fn weicheng_confirm_without_hand_below_hp_draws_nothing() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    let _skill = add_skill(&mut tx, owner, WEICHENG);
    // Owner hp = 1 → hand (4) is not below HP → confirming draws nothing.
    tx.state_mut().players.get_mut(owner).unwrap().hp = 1;
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let other_hand = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    let give = tx.state().hand_cards(owner)[0];
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![give],
            to: other_hand,
            from: Some(hand),
        }],
    )
    .unwrap();
    let owner_hand_before = tx.state().hand_count(owner);

    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardsGiven {
            giver: owner,
            recipient: other,
            cards: vec![give],
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => {
            run_effects(&mut tx, effects).unwrap();
        }
        Err(engine_core::effect::EffectError::ChoiceRequested) => {}
        Err(e) => panic!("dispatch error: {e:?}"),
    }
    assert!(tx.state().has_pending_choice(), "trigger still offers");

    let choice_id = engine_api::ChoiceId::new(tx.state().resolution.pending_choice_id);
    let (state, confirmed) = apply_command(
        tx.clone().commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.hand_count(owner),
        owner_hand_before,
        "hand not below HP → no draw"
    );
}
