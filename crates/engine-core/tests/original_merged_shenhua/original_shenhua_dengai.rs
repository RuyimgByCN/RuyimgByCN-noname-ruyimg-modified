//! Golden coverage for 屯田/凿险/急袭（shenhua 邓艾，ABI 533-534）.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::query::{query, QueryContext};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const TUNTIAN: u32 = 2_366_746_565;
const ZAOXIAN: u32 = 853_475_869;
const JIXI: u32 = 3_780_186_838;
const SOURCE: &str = include_str!("../../../../content/original/ported/shenhua/dengai.skill");

struct Setup {
    tx: Transaction,
    owner: PlayerId,
    other: PlayerId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("dengai-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 3, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let other = seats[1];
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
        skill_def: SkillDefId::new(TUNTIAN),
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(ZAOXIAN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = other;
    tx.state_mut().turn.phase = Phase::Play;
    Setup { tx, owner, other }
}

fn alloc_card(
    tx: &mut Transaction,
    owner: PlayerId,
    zone: engine_api::ZoneId,
    kind: CardKind,
    def: u32,
    suit: Suit,
    rank: u8,
) -> CardId {
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
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

fn resolve_choice(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "choice must be accepted: {:?}",
        result.result
    );
    next
}

#[test]
fn package_exposes_tuntian_zaoxian_contract() {
    let compiled = content_compiler::compile_source_named("dengai", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 534);
    let tuntian = compiled
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == TUNTIAN)
        .unwrap();
    assert_eq!(tuntian.event, "cards_moved");
    assert_eq!(tuntian.filter, "owner_lost_cards_outside_turn");
    assert!(tuntian.optional);
    let zaoxian = compiled
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == ZAOXIAN)
        .unwrap();
    assert_eq!(zaoxian.event, "phase_prepare");
    assert_eq!(
        zaoxian.filter,
        "owner_phase_begin_skill_pile_gte_three_2366746565"
    );
    assert!(!zaoxian.optional);
    let jixi = compiled
        .view_as_index
        .iter()
        .find(|v| v.skill_def_id == JIXI)
        .unwrap();
    assert_eq!(jixi.as_kind, "shunshou");
    assert_eq!(jixi.source_zone.as_deref(), Some("hand_expansion"));
    assert_eq!(compiled.skill_index.len(), 3);
    assert_eq!(compiled.trigger_index.len(), 2);
    assert_eq!(compiled.modifier_index.len(), 1);
}

#[test]
fn tuntian_stores_non_heart_judge_card_in_pile_and_reduces_distance() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    // Place a known non-heart card on the draw pile top (club 7).
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let judge_card = alloc_card(
        &mut tx,
        PlayerId::NONE,
        draw,
        CardKind::Sha,
        96_330,
        Suit::Club,
        7,
    );
    // The owner loses a hand card during another player's turn.
    let hand = tx.state().zones.find(ZoneKind::Hand, setup.owner).unwrap();
    let lost = alloc_card(
        &mut tx,
        setup.owner,
        hand,
        CardKind::Tao,
        96_331,
        Suit::Heart,
        1,
    );
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::MoveCards {
                cards: vec![lost],
                to: discard,
                from: Some(hand),
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    // The trigger opened the optional tuntian choice.
    let mut state = tx.commit().state;
    assert!(
        state.has_pending_choice(),
        "optional tuntian confirm pending"
    );
    state = resolve_choice(state, setup.owner, vec![1], 1);
    // Judge card (club 7) went into the 田 pile (face-up expansion of the owner).
    let pile = state
        .skill_piles
        .iter()
        .find(|pile| pile.owner == setup.owner && pile.skill_def == TUNTIAN)
        .map(|pile| pile.zone)
        .unwrap();
    let pile_cards: Vec<_> = state
        .zones
        .get(pile)
        .map(|zone| zone.cards.clone())
        .unwrap_or_default();
    assert_eq!(pile_cards, vec![judge_card], "judge card stored as 田");
    assert!(!state.has_pending_choice());
    // Distance from the owner to the other player is reduced by the 田 count (base 1 → 0).
    let mut ctx = QueryContext::new();
    let dist = match query(
        &state,
        &QueryRequest::Distance {
            from: setup.owner,
            to: setup.other,
        },
        &mut ctx,
    ) {
        QueryResponse::Int(d) => d,
        other => panic!("distance query failed: {other:?}"),
    };
    // 相邻距离基准 1，被田数减至 0 后按引擎不变量钳制回 1（全引擎距离下限为 1，
    // 与旧 JS 距离可为 0 的差异为文档化近似）。
    assert_eq!(
        dist, 1,
        "adjacent distance clamped at the engine floor of 1"
    );
    let dist_back = match query(
        &state,
        &QueryRequest::Distance {
            from: setup.other,
            to: setup.owner,
        },
        &mut ctx,
    ) {
        QueryResponse::Int(d) => d,
        other => panic!("distance query failed: {other:?}"),
    };
    assert_eq!(dist_back, 1, "other-to-owner distance unaffected");
    // 隔座基准距离 2，减 1 田后为 1 —— 证明计数修正器生效。
    let seats = state.players.alive_by_seat();
    let far = seats[2];
    let dist_far = match query(
        &state,
        &QueryRequest::Distance {
            from: setup.owner,
            to: far,
        },
        &mut ctx,
    ) {
        QueryResponse::Int(d) => d,
        other => panic!("distance query failed: {other:?}"),
    };
    assert_eq!(dist_far, 1, "far distance reduced from 2 by the 田 count");
}

#[test]
fn tuntian_heart_judge_card_goes_nowhere() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let judge_card = alloc_card(
        &mut tx,
        PlayerId::NONE,
        draw,
        CardKind::Sha,
        96_340,
        Suit::Heart,
        7,
    );
    let hand = tx.state().zones.find(ZoneKind::Hand, setup.owner).unwrap();
    let lost = alloc_card(
        &mut tx,
        setup.owner,
        hand,
        CardKind::Tao,
        96_341,
        Suit::Heart,
        1,
    );
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::MoveCards {
                cards: vec![lost],
                to: discard,
                from: Some(hand),
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let mut state = tx.commit().state;
    state = resolve_choice(state, setup.owner, vec![1], 1);
    // Heart judge card: no 田 pile is created.
    assert!(
        !state
            .skill_piles
            .iter()
            .any(|pile| pile.owner == setup.owner && pile.skill_def == TUNTIAN),
        "no 田 pile for a heart judge"
    );
    let discard_cards: Vec<_> = state
        .zones
        .get(discard)
        .map(|zone| zone.cards.clone())
        .unwrap_or_default();
    assert!(
        discard_cards.contains(&judge_card),
        "heart judge card to discard"
    );
}

#[test]
fn zaoxian_awakens_with_three_tian() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    // Stack three non-heart cards: first three judgments build 田.
    let c1 = alloc_card(
        &mut tx,
        PlayerId::NONE,
        draw,
        CardKind::Sha,
        96_350,
        Suit::Club,
        2,
    );
    let c2 = alloc_card(
        &mut tx,
        PlayerId::NONE,
        draw,
        CardKind::Sha,
        96_351,
        Suit::Spade,
        3,
    );
    let c3 = alloc_card(
        &mut tx,
        PlayerId::NONE,
        draw,
        CardKind::Sha,
        96_352,
        Suit::Club,
        4,
    );
    for _ in 0..3 {
        let hand = tx.state().zones.find(ZoneKind::Hand, setup.owner).unwrap();
        let lost = alloc_card(
            &mut tx,
            setup.owner,
            hand,
            CardKind::Tao,
            96_360,
            Suit::Heart,
            1,
        );
        let discard = tx
            .state()
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap();
        assert_eq!(
            run_effects(
                &mut tx,
                vec![EffectRequest::MoveCards {
                    cards: vec![lost],
                    to: discard,
                    from: Some(hand),
                }],
            ),
            Err(EffectError::ChoiceRequested)
        );
        let mut state = tx.commit().state;
        state = resolve_choice(state, setup.owner, vec![1], 1);
        tx = Transaction::begin(state);
    }
    // Now the owner's prepare phase: 凿险 awakens (max hp 5 → 4, gains 急袭).
    let mut tx = tx;
    tx.state_mut().turn.current = setup.owner;
    let state = tx.commit().state;
    let fired = dispatch_for_events(
        &mut Transaction::begin(state.clone()),
        &[DomainEvent::TurnStateChanged {
            current: setup.owner,
            phase: engine_api::TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    );
    let mut tx = match fired {
        Ok(effects) => {
            let mut next = Transaction::begin(state);
            run_effects(&mut next, effects).unwrap();
            next
        }
        Err(EffectError::ChoiceRequested) => Transaction::begin(state),
        Err(e) => panic!("dispatch error: {e:?}"),
    };
    let state = tx.commit().state;
    assert_eq!(
        state.players.get(setup.owner).unwrap().max_hp,
        4,
        "max hp reduced from lord 5 to 4"
    );
    assert!(
        state
            .skills
            .sorted_live()
            .iter()
            .any(|skill| skill.owner == setup.owner && skill.def.raw() == JIXI),
        "急袭 granted"
    );
    // 田 stay in the pile (usable by 急袭).
    let pile = state
        .skill_piles
        .iter()
        .find(|pile| pile.owner == setup.owner && pile.skill_def == TUNTIAN)
        .map(|pile| pile.zone)
        .unwrap();
    let pile_cards: Vec<_> = state
        .zones
        .get(pile)
        .map(|zone| zone.cards.clone())
        .unwrap_or_default();
    assert_eq!(pile_cards.len(), 3);
    let _ = (c1, c2, c3);
}

#[test]
fn zaoxian_does_not_fire_below_three_tian() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let _ = alloc_card(
        &mut tx,
        PlayerId::NONE,
        draw,
        CardKind::Sha,
        96_370,
        Suit::Club,
        2,
    );
    let hand = tx.state().zones.find(ZoneKind::Hand, setup.owner).unwrap();
    let lost = alloc_card(
        &mut tx,
        setup.owner,
        hand,
        CardKind::Tao,
        96_371,
        Suit::Heart,
        1,
    );
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::MoveCards {
                cards: vec![lost],
                to: discard,
                from: Some(hand),
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let mut state = tx.commit().state;
    state = resolve_choice(state, setup.owner, vec![1], 1);
    // One 田 only — prepare phase does not awaken.
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = setup.owner;
    let state = tx.commit().state;
    let fired = dispatch_for_events(
        &mut Transaction::begin(state.clone()),
        &[DomainEvent::TurnStateChanged {
            current: setup.owner,
            phase: engine_api::TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    );
    let effects = fired.unwrap_or_else(|_| Vec::new());
    let mut tx = Transaction::begin(state);
    run_effects(&mut tx, effects).unwrap();
    let state = tx.commit().state;
    assert_eq!(
        state.players.get(setup.owner).unwrap().max_hp,
        5,
        "no awakening below three 田 (lord bonus)"
    );
    assert!(!state
        .skills
        .sorted_live()
        .iter()
        .any(|skill| skill.owner == setup.owner && skill.def.raw() == JIXI));
}

#[test]
fn jixi_uses_tian_as_shunshou() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    // Grant 急袭, then build one 田 through the real tuntian flow.
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: setup.owner,
            skill_def: SkillDefId::new(JIXI),
        }],
    )
    .unwrap();
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let _ = alloc_card(
        &mut tx,
        PlayerId::NONE,
        draw,
        CardKind::Sha,
        96_380,
        Suit::Club,
        5,
    );
    let hand = tx.state().zones.find(ZoneKind::Hand, setup.owner).unwrap();
    let lost = alloc_card(
        &mut tx,
        setup.owner,
        hand,
        CardKind::Tao,
        96_381,
        Suit::Heart,
        1,
    );
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::MoveCards {
                cards: vec![lost],
                to: discard,
                from: Some(hand),
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let mut state = tx.commit().state;
    state = resolve_choice(state, setup.owner, vec![1], 1);
    // The 田 pile is an expansion zone owned by the owner — the hand_expansion
    // view-as source for 顺手牵羊.
    let pile = state
        .skill_piles
        .iter()
        .find(|pile| pile.owner == setup.owner && pile.skill_def == TUNTIAN)
        .map(|pile| pile.zone)
        .unwrap();
    let pile_zone = state.zones.get(pile).map(|zone| zone.kind).unwrap();
    assert_eq!(
        pile_zone,
        engine_core::state::ZoneKind::Expansion,
        "田 pile is an expansion zone"
    );
    let pile_cards: Vec<_> = state
        .zones
        .get(pile)
        .map(|zone| zone.cards.clone())
        .unwrap_or_default();
    assert_eq!(pile_cards.len(), 1, "one 田 stored");
    assert!(
        state
            .cards
            .get(pile_cards[0])
            .is_some_and(|card| card.owner == setup.owner),
        "田 owned by the owner"
    );
    assert!(
        state
            .skills
            .sorted_live()
            .iter()
            .any(|skill| skill.owner == setup.owner && skill.def.raw() == JIXI),
        "急袭 active"
    );
}
