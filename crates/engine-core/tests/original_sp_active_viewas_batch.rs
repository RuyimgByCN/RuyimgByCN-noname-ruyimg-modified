//! Golden coverage for the active/view_as rewrite batch in sp:
//!   - 断发 xinfu_duanfa: active, discard any number of black hand cards, draw the same count.
//!   - 宠慎 olchongshen: red hand cards may be used as 【闪】.
//!   - 拒关 juguan: a hand card may be used as 【杀】 or 【决斗】 once per play phase.
//!
//! Exercises the `card_black` active selection filter (new primitive), the
//! `discard_selected`/`draw_selected` pair, and the `view_as` shan/sha/juedou
//! declarations.

use engine_api::{
    CardDefId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const XINFU_DUANFA: u32 = 3_504_403_856;
const OLCHONGSHEN: u32 = 4_123_829_304;
const JUGUAN: u32 = 4_273_039_202;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn start() -> (Engine, PlayerId) {
    let mut engine = Engine::new();
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_901,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&pkg).unwrap();
    let owner = engine.state().turn.current;
    (engine, owner)
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

fn strip_skills(tx: &mut Transaction, player: PlayerId) {
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(tx, effects).unwrap();
}

fn clear_hand(tx: &mut Transaction, player: PlayerId) {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let cards = tx.state().hand_cards(player);
    if cards.is_empty() {
        return;
    }
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

fn inject_hand(
    tx: &mut Transaction,
    player: PlayerId,
    kind: CardKind,
    suit: Suit,
) -> engine_api::CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: CardDefId::new(170_100 + kind.tag() as u32),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn activate(
    actor: PlayerId,
    skill: engine_api::SkillId,
    targets: Vec<PlayerId>,
    cards: Vec<engine_api::CardId>,
    key: u64,
) -> Command {
    Command {
        actor,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(key),
        kind: CommandKind::ActivateSkill {
            skill,
            targets,
            card_targets: cards,
        },
    }
}

#[test]
fn package_declares_the_new_view_as_and_active_entries() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let chongshen = package
        .view_as_index
        .iter()
        .find(|view| view.skill_def_id == OLCHONGSHEN)
        .unwrap();
    assert_eq!(chongshen.as_kind, "shan");
    assert_eq!(chongshen.window, "any");
    assert_eq!(chongshen.filter, "red");
    for (kind, window) in [("sha", "play_phase"), ("juedou", "play_phase")] {
        assert!(package.view_as_index.iter().any(|view| {
            view.skill_def_id == JUGUAN
                && view.as_kind == kind
                && view.window == window
                && view.uses_per_turn == Some(1)
        }));
    }
    assert!(package
        .active_index
        .iter()
        .any(|active| active.skill_def_id == XINFU_DUANFA));
}

#[test]
fn duanfa_discards_black_cards_and_draws_equal_count() {
    let (engine, owner) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    strip_skills(&mut tx, owner);
    clear_hand(&mut tx, owner);
    tx.state_mut().turn.phase = Phase::Play;
    let skill = add_skill(&mut tx, owner, XINFU_DUANFA);
    let black1 = inject_hand(&mut tx, owner, CardKind::Sha, Suit::Spade);
    let black2 = inject_hand(&mut tx, owner, CardKind::Shan, Suit::Club);
    let red = inject_hand(&mut tx, owner, CardKind::Tao, Suit::Heart);
    let hand_before = tx.state().hand_count(owner);
    let draw_pile_before = {
        let draw = tx
            .state()
            .zones
            .find(ZoneKind::DrawPile, PlayerId::NONE)
            .unwrap();
        tx.state().zones.get(draw).unwrap().cards.len()
    };

    // Red card is not selectable: activating with only the red card is rejected.
    let (_, rejected) = apply_command(
        tx.state().clone(),
        activate(owner, skill, vec![], vec![red], 1),
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));

    // Two black cards: discard both, draw two.
    let (state, accepted) = apply_command(
        tx.state().clone(),
        activate(owner, skill, vec![], vec![black1, black2], 2),
    );
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert!(!state.hand_cards(owner).contains(&black1));
    assert!(!state.hand_cards(owner).contains(&black2));
    assert!(
        state.hand_cards(owner).contains(&red),
        "red card untouched"
    );
    assert_eq!(
        state.hand_count(owner),
        hand_before,
        "discard 2 + draw 2 keeps hand size"
    );
    assert_eq!(
        {
            let draw = state
                .zones
                .find(ZoneKind::DrawPile, PlayerId::NONE)
                .unwrap();
            state.zones.get(draw).unwrap().cards.len()
        },
        draw_pile_before - 2,
        "two cards drawn from the pile"
    );
}

#[test]
fn duanfa_is_limited_to_once_per_turn() {
    let (engine, owner) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    strip_skills(&mut tx, owner);
    clear_hand(&mut tx, owner);
    tx.state_mut().turn.phase = Phase::Play;
    let skill = add_skill(&mut tx, owner, XINFU_DUANFA);
    let black = inject_hand(&mut tx, owner, CardKind::Sha, Suit::Spade);

    let (state, first) = apply_command(
        tx.state().clone(),
        activate(owner, skill, vec![], vec![black], 1),
    );
    assert!(matches!(first.result, CommandResult::Accepted { .. }));

    let second_black = {
        let mut tx = Transaction::begin(state.clone());
        inject_hand(&mut tx, owner, CardKind::Shan, Suit::Club)
    };
    let (_, second) = apply_command(
        state,
        activate(owner, skill, vec![], vec![second_black], 2),
    );
    assert!(matches!(second.result, CommandResult::Rejected { .. }));
}

#[test]
fn chongshen_offers_only_red_hand_cards_in_respond_shan_window() {
    let (engine, owner, other) = start2();
    let mut tx = Transaction::begin(engine.state().clone());
    strip_skills(&mut tx, owner);
    clear_hand(&mut tx, owner);
    strip_skills(&mut tx, other);
    clear_hand(&mut tx, other);
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().turn.current = other;
    add_skill(&mut tx, owner, OLCHONGSHEN);
    let red = inject_hand(&mut tx, owner, CardKind::Tao, Suit::Heart);
    let black = inject_hand(&mut tx, owner, CardKind::Sha, Suit::Spade);
    let sha = inject_hand(&mut tx, other, CardKind::Sha, Suit::Club);

    // The other player plays 杀 at the owner → a respond-shan window opens for the owner.
    let (state, result) = apply_command(
        tx.commit().state,
        Command {
            actor: other,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![owner],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }), "unexpected play result: {:?}", result.result);
    let choice = result
        .choice
        .expect("杀 targeting the owner must open a respond-shan window");
    assert_eq!(choice.observer, owner);
    assert!(
        choice
            .options
            .iter()
            .any(|option| option.token == red.raw()),
        "red hand card offered as 闪 via 宠慎"
    );
    assert!(
        !choice
            .options
            .iter()
            .any(|option| option.token == black.raw()),
        "black hand card not offered"
    );

    // Resolving with the red card dodges the 杀.
    let (state, dodged) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![red.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(dodged.result, CommandResult::Accepted { .. }));
    assert!(
        !state.hand_cards(owner).contains(&red),
        "the red card was consumed as the 闪"
    );
}

#[test]
fn juguan_offers_sha_and_juedou_once_each() {
    let (engine, owner, other) = start2();
    let mut tx = Transaction::begin(engine.state().clone());
    strip_skills(&mut tx, owner);
    clear_hand(&mut tx, owner);
    tx.state_mut().turn.phase = Phase::Play;
    let skill = add_skill(&mut tx, owner, JUGUAN);
    let card1 = inject_hand(&mut tx, owner, CardKind::Tao, Suit::Heart);
    let card2 = inject_hand(&mut tx, owner, CardKind::Shan, Suit::Diamond);

    let (_, as_sha) = apply_command(
        tx.state().clone(),
        activate(owner, skill, vec![other], vec![card1], 1),
    );
    assert!(
        matches!(as_sha.result, CommandResult::Accepted { .. }),
        "hand card usable as 杀"
    );

    let (_, as_juedou) = apply_command(
        tx.state().clone(),
        activate(owner, skill, vec![other], vec![card2], 2),
    );
    assert!(
        matches!(as_juedou.result, CommandResult::Accepted { .. }),
        "hand card usable as 决斗 (separate once-per-transform limit)"
    );
}

fn start2() -> (Engine, PlayerId, PlayerId) {
    let (engine, owner) = start();
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    (engine, owner, other)
}
