//! Golden coverage for the complete SP2「兴棹」family and shared「恂恂」reorder.

use std::collections::BTreeSet;

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, schedule_damage};
use engine_core::state::{CardKind, ContinuationKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const SP2: &[u8] = include_bytes!("../../../content/original/ported/sp2/sp2.npack.json");
const REFRESH: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");
const XINGZHAO: u32 = 2_244_503_587;
const XUNXUN: u32 = 4_191_758_778;

#[test]
fn tangzi_character_entry_grants_only_the_composed_parent_skill() {
    let package = content_schema::ContentPackage::from_json_bytes(SP2).unwrap();
    let tangzi = package
        .character_index
        .iter()
        .find(|character| character.id == 2_852_100_501)
        .expect("唐咨 should be present in the SP2 extension package");
    assert_eq!(tangzi.name_key, "xf_tangzi");
    assert_eq!(tangzi.sex, "male");
    assert_eq!(tangzi.group, "wei");
    assert_eq!(tangzi.max_hp, 4);
    assert_eq!(tangzi.skills, vec![XINGZHAO]);
}

fn setup(load_refresh: bool) -> (Transaction, PlayerId) {
    let mut engine = Engine::new();
    engine
        .load_package(&content_schema::ContentPackage::from_json_bytes(SP2).unwrap())
        .unwrap();
    if load_refresh {
        engine
            .load_package(&content_schema::ContentPackage::from_json_bytes(REFRESH).unwrap())
            .unwrap();
    }
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_715),
            kind: CommandKind::StartGame {
                player_count: 4,
                rng_seed: 20_260_715,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    let skills: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| skill.id)
        .collect();
    for skill in skills {
        run_effects(&mut tx, vec![EffectRequest::RemoveSkill { skill }]).unwrap();
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(XINGZHAO),
        }],
    )
    .unwrap();
    (tx, owner)
}

fn set_wounded(tx: &mut Transaction, count: usize) {
    let players = tx.state().players.alive_by_seat();
    for player in &players {
        let state = tx.state_mut().players.get_mut(*player).unwrap();
        state.max_hp = 10;
        state.hp = 10;
    }
    for player in players.into_iter().take(count) {
        tx.state_mut().players.get_mut(player).unwrap().hp = 9;
    }
}

fn end_phase(
    state: engine_core::state::GameState,
    owner: PlayerId,
    key: u64,
) -> (
    engine_core::state::GameState,
    engine_core::command::PipelineResult,
) {
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::EndPhase,
        },
    )
}

#[test]
fn equipment_entry_and_replacement_draw_once_per_zone_boundary() {
    let (mut tx, owner) = setup(false);
    set_wounded(&mut tx, 2);
    let cards = tx.state().hand_cards(owner);
    let first = cards[0];
    let second = cards[1];
    tx.state_mut().cards.get_mut(first).unwrap().kind = CardKind::Weapon;
    tx.state_mut().cards.get_mut(second).unwrap().kind = CardKind::Weapon;
    let hand_zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let equip_zone = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let hand = tx.state().hand_count(owner);

    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![first],
            to: equip_zone,
            from: Some(hand_zone),
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand);

    run_effects(
        &mut tx,
        vec![
            EffectRequest::MoveCards {
                cards: vec![first],
                to: discard,
                from: Some(equip_zone),
            },
            EffectRequest::MoveCards {
                cards: vec![second],
                to: equip_zone,
                from: Some(hand_zone),
            },
        ],
    )
    .unwrap();
    assert_eq!(
        tx.state().hand_count(owner),
        hand + 1,
        "replacing equipment crosses the zone boundary twice"
    );
}

#[test]
fn equipment_movement_does_not_draw_below_two_wounded_players() {
    let (mut tx, owner) = setup(false);
    set_wounded(&mut tx, 1);
    let card = tx.state().hand_cards(owner)[0];
    tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Weapon;
    let hand_zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let equip_zone = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    let hand = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![card],
            to: equip_zone,
            from: Some(hand_zone),
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand - 1);
}

#[test]
fn xunxun_orders_two_top_and_two_bottom_then_draws_normally() {
    let (mut tx, owner) = setup(false);
    set_wounded(&mut tx, 1);
    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Judge.to_u8(),
        }],
    )
    .unwrap();
    let hand = tx.state().hand_count(owner);
    let (pending, pipe) = end_phase(tx.state().clone(), owner, 1);
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert_eq!(
        pending.resolution.continuation_kind,
        ContinuationKind::ReorderDrawPileFourSplitTwoThenDraw
    );
    let choice = pipe.choice.unwrap();
    assert_eq!((choice.min_count, choice.max_count), (4, 4));
    assert!(choice.allow_cancel);
    let offered: Vec<u32> = choice.options.iter().map(|option| option.token).collect();
    let order = vec![offered[0], offered[1], offered[3], offered[2]];
    let (resolved, pipe) = apply_command(
        pending,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: order.clone(),
                cancelled: false,
            },
        },
    );
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert!(!resolved.has_pending_choice());
    assert_eq!(resolved.turn.phase, Phase::Draw);
    assert_eq!(resolved.hand_count(owner), hand + 2);
    let hand_cards: BTreeSet<u32> = resolved
        .hand_cards(owner)
        .into_iter()
        .map(|card| card.raw())
        .collect();
    assert!(hand_cards.contains(&order[0]) && hand_cards.contains(&order[1]));
    let draw = resolved
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let draw_cards = &resolved.zones.get(draw).unwrap().cards;
    assert_eq!(draw_cards[0].raw(), order[2]);
    assert_eq!(draw_cards[1].raw(), order[3]);
}

#[test]
fn xingzhao_does_not_offer_xunxun_when_nobody_is_wounded() {
    let (mut tx, owner) = setup(false);
    set_wounded(&mut tx, 0);
    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Judge.to_u8(),
        }],
    )
    .unwrap();
    let hand = tx.state().hand_count(owner);
    let (state, pipe) = end_phase(tx.state().clone(), owner, 7);
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.turn.phase, Phase::Draw);
    assert_eq!(state.hand_count(owner), hand + 2);
}

#[test]
fn xunxun_is_optional_and_multiple_equivalent_sources_do_not_duplicate_it() {
    let (mut tx, owner) = setup(true);
    set_wounded(&mut tx, 1);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(XUNXUN),
            },
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::Judge.to_u8(),
            },
        ],
    )
    .unwrap();
    let hand = tx.state().hand_count(owner);
    let (pending, pipe) = end_phase(tx.state().clone(), owner, 3);
    let choice = pipe.choice.unwrap();
    let (resolved, pipe) = apply_command(
        pending,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(4),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(choice.choice_id.raw()),
                selections: vec![],
                cancelled: true,
            },
        },
    );
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert!(!resolved.has_pending_choice());
    assert_eq!(resolved.hand_count(owner), hand + 2);
}

#[test]
fn three_wounded_players_skip_judge_and_discard_but_two_do_not() {
    let (mut tx, owner) = setup(false);
    set_wounded(&mut tx, 2);
    assert!(!engine_core::query::skip_judge_phase(tx.state(), owner));
    assert!(!engine_core::query::skip_discard_phase(tx.state(), owner));
    let mut low = Transaction::begin(tx.state().clone());
    run_effects(
        &mut low,
        vec![
            EffectRequest::DrawCards {
                player: owner,
                count: 8,
            },
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::Discard.to_u8(),
            },
        ],
    )
    .unwrap();
    let (low_state, low_pipe) = end_phase(low.state().clone(), owner, 5);
    assert!(matches!(low_pipe.result, CommandResult::Accepted { .. }));
    assert_eq!(
        low_state.resolution.continuation_kind,
        ContinuationKind::DiscardToLimit
    );

    set_wounded(&mut tx, 3);
    assert!(engine_core::query::skip_judge_phase(tx.state(), owner));
    assert!(engine_core::query::skip_discard_phase(tx.state(), owner));

    run_effects(
        &mut tx,
        vec![
            EffectRequest::DrawCards {
                player: owner,
                count: 4,
            },
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::Discard.to_u8(),
            },
        ],
    )
    .unwrap();
    let hand = tx.state().hand_count(owner);
    let (state, pipe) = end_phase(tx.state().clone(), owner, 6);
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.turn.phase, Phase::End);
    assert_eq!(state.hand_count(owner), hand);
}

#[test]
fn parent_damage_tier_matches_the_legacy_zero_or_four_threshold() {
    let (mut tx, owner) = setup(false);
    let target = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    set_wounded(&mut tx, 0);
    let hp = tx.state().players.get(target).unwrap().hp;
    schedule_damage(&mut tx, owner, target, 1).unwrap();
    assert_eq!(tx.state().players.get(target).unwrap().hp, hp - 2);

    set_wounded(&mut tx, 1);
    let hp = tx.state().players.get(target).unwrap().hp;
    schedule_damage(&mut tx, owner, target, 1).unwrap();
    assert_eq!(tx.state().players.get(target).unwrap().hp, hp - 1);
}
