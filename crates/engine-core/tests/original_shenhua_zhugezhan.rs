//! Golden coverage for 罪论/父荫（shenhua 诸葛瞻，ABI 563）.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const ZUILUN: u32 = 3_588_795_290;
const FUYIN: u32 = 3_383_028_272;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/zhugezhan.skill");

struct Setup {
    tx: Transaction,
    owner: PlayerId,
    other: PlayerId,
    far: PlayerId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("zhugezhan-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    // Enable the turn card-loss history via a helper filter (ABI 405 family).
    let helper = content_compiler::compile_source_named(
        "loss-tracking-helper",
        "skill loss_helper(4000000001) { trigger turn_ended owner_alive_lost_hand_this_turn optional { effect skip_current_phase(self); } }",
    )
    .unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&helper).unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let other = seats[1];
    let far = seats[2];
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
        skill_def: SkillDefId::new(ZUILUN),
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(FUYIN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    Setup {
        tx,
        owner,
        other,
        far,
    }
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

fn hand_zone(tx: &Transaction, player: PlayerId) -> engine_api::ZoneId {
    tx.state().zones.find(ZoneKind::Hand, player).unwrap()
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "unexpected result: {:?}",
        result.result
    );
    state
}

fn dispatch(state: GameState, event: DomainEvent) -> GameState {
    let mut tx = Transaction::begin(state);
    let fired = dispatch_for_events(&mut tx, &[event], &mut TriggerFireRecord::new());
    match fired {
        Ok(effects) => match run_effects(&mut tx, effects) {
            Err(EffectError::ChoiceRequested) => {}
            Ok(_) => {}
            Err(e) => panic!("dispatch error: {e:?}"),
        },
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("dispatch error: {e:?}"),
    }
    tx.commit().state
}

/// Prepare the draw pile with three known top cards and the owner at the End phase.
fn prep_end(state: GameState, owner: PlayerId) -> (GameState, Vec<CardId>) {
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.lost_cards.clear();
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let top1 = alloc_card(
        &mut tx,
        PlayerId::NONE,
        draw,
        CardKind::Tao,
        96_800,
        Suit::Heart,
        2,
    );
    let top2 = alloc_card(
        &mut tx,
        PlayerId::NONE,
        draw,
        CardKind::Sha,
        96_801,
        Suit::Spade,
        3,
    );
    let top3 = alloc_card(
        &mut tx,
        PlayerId::NONE,
        draw,
        CardKind::Shan,
        96_802,
        Suit::Heart,
        4,
    );
    tx.state_mut().turn.phase = Phase::End;
    (tx.commit().state, vec![top1, top2, top3])
}

#[test]
fn package_exposes_zhugezhan_contract() {
    let compiled = content_compiler::compile_source_named("zhugezhan", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 563);
    assert_eq!(compiled.skill_index.len(), 2);
    let zuilun = compiled
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == ZUILUN)
        .unwrap();
    assert_eq!(zuilun.event, "phase_end");
    assert!(zuilun.optional);
    let fuyin = compiled
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == FUYIN)
        .unwrap();
    assert_eq!(fuyin.event, "card_targeted");
    assert!(!fuyin.optional);
}

/// 罪论 count==0：获得全部三张。
#[test]
fn zuilun_zero_conditions_gains_all_three() {
    let setup = setup();
    let state = setup.tx.commit().state;
    let (state, revealed) = prep_end(state, setup.owner);
    // Count 0 needs: no loss, NOT minimum hand, damaged. Give the owner two hand
    // cards (others stay at zero -> not minimum) and damage the owner.
    let mut tx = Transaction::begin(state);
    let owner_hand = hand_zone(&tx, setup.owner);
    let h1 = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Sha,
        96_807,
        Suit::Club,
        9,
    );
    let h2 = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Shan,
        96_808,
        Suit::Heart,
        5,
    );
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: setup.owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    let state = tx.commit().state;
    let state = dispatch(
        state,
        DomainEvent::TurnStateChanged {
            current: setup.owner,
            phase: engine_api::TurnPhaseTag::End,
        },
    );
    assert!(state.has_pending_choice(), "罪论 confirm");
    let state = resolve(state, setup.owner, vec![1], false, 9_000);
    // No choice: the three cards were gained directly.
    assert!(!state.has_pending_choice(), "count 0 gains directly");
    for card in &revealed {
        assert!(
            state.hand_cards(setup.owner).contains(card),
            "gained {card:?}"
        );
    }
    assert_eq!(state.hand_count(setup.owner), 5, "2 held + 3 gained");
}

/// 罪论 count==2：选两张置顶，获其余一张。
#[test]
fn zuilun_two_conditions_puts_two_on_top() {
    let setup = setup();
    let state = setup.tx.commit().state;
    let (state, revealed) = prep_end(state, setup.owner);
    let state = dispatch(
        state,
        DomainEvent::TurnStateChanged {
            current: setup.owner,
            phase: engine_api::TurnPhaseTag::End,
        },
    );
    assert!(state.has_pending_choice(), "罪论 confirm");
    let state = resolve(state, setup.owner, vec![1], false, 9_000);
    assert!(state.has_pending_choice(), "罪论 pick prompt");
    // count: lost=0, min_hand=1 (tie at 0), undamaged=1 -> count=2.
    let state = resolve(
        state,
        setup.owner,
        vec![revealed[0].raw(), revealed[1].raw()],
        false,
        9_001,
    );
    assert!(!state.has_pending_choice());
    // revealed[0]/[1] back on the draw pile top; revealed[2] gained.
    assert!(state.hand_cards(setup.owner).contains(&revealed[2]));
    assert!(!state.hand_cards(setup.owner).contains(&revealed[0]));
    let draw = state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let top: Vec<_> = state
        .zones
        .get(draw)
        .map(|zone| zone.cards.iter().rev().take(2).copied().collect())
        .unwrap_or_default();
    assert!(top.contains(&revealed[0]) && top.contains(&revealed[1]));
}

/// 罪论 count==3：全部置顶并选择一名其他角色一起失去 1 点体力。
#[test]
fn zuilun_three_conditions_lose_hp_together() {
    let setup = setup();
    let state = setup.tx.commit().state;
    let (state, revealed) = prep_end(state, setup.owner);
    // All three conditions: lost (force a loss), min hand (owner strict minimum), undamaged.
    let mut tx = Transaction::begin(state);
    let owner_hand = hand_zone(&tx, setup.owner);
    let lost_card = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Sha,
        96_803,
        Suit::Club,
        5,
    );
    // Give the others hand cards so the owner is the strict minimum.
    let other_hand = hand_zone(&tx, setup.other);
    let far_hand = hand_zone(&tx, setup.far);
    let o1 = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Sha,
        96_804,
        Suit::Club,
        6,
    );
    let f1 = alloc_card(
        &mut tx,
        setup.far,
        far_hand,
        CardKind::Sha,
        96_805,
        Suit::Club,
        7,
    );
    // Force a card loss this turn via an explicit move to the discard pile.
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![lost_card],
            to: discard,
            from: Some(owner_hand),
        }],
    )
    .unwrap();
    let state = tx.commit().state;
    let hp_before = state.players.get(setup.owner).unwrap().hp;
    let far_hp_before = state.players.get(setup.far).unwrap().hp;
    let state = dispatch(
        state,
        DomainEvent::TurnStateChanged {
            current: setup.owner,
            phase: engine_api::TurnPhaseTag::End,
        },
    );
    assert!(state.has_pending_choice(), "罪论 confirm");
    let state = resolve(state, setup.owner, vec![1], false, 9_000);
    assert!(state.has_pending_choice(), "罪论 lose-hp target prompt");
    let state = resolve(state, setup.owner, vec![setup.far.raw()], false, 9_010);
    assert!(!state.has_pending_choice());
    // Both lost 1 HP.
    assert_eq!(state.players.get(setup.owner).unwrap().hp, hp_before - 1);
    assert_eq!(state.players.get(setup.far).unwrap().hp, far_hp_before - 1);
    // All three on the draw pile top.
    let draw = state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let top: Vec<_> = state
        .zones
        .get(draw)
        .map(|zone| zone.cards.iter().rev().take(3).copied().collect())
        .unwrap_or_default();
    for card in &revealed {
        assert!(top.contains(card), "on top {card:?}");
    }
    let _ = (o1, f1);
}

/// 父荫：本回合首次被手牌数不小于自己的其他角色以【杀】指定时，此杀无效。
#[test]
fn fuyin_excludes_first_sha_from_richer_source() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    tx.state_mut().turn.current = setup.other;
    tx.state_mut().turn.phase = Phase::Play;
    let other_hand = hand_zone(&tx, setup.other);
    let s1 = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Sha,
        96_805,
        Suit::Spade,
        7,
    );
    let s2 = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Sha,
        96_806,
        Suit::Spade,
        8,
    );
    // Owner has 0 hand cards; the source has 2: the source hand >= owner hand.
    let mut state = tx.commit().state;
    let mut resolution = &mut state.resolution;
    resolution.resume_kind = engine_core::state::ContinuationKind::ResumePlayedSha;
    resolution.resume_payload = vec![setup.other.raw(), setup.owner.raw(), 1, 0, 1, s1.raw()];
    resolution.register_sha_resolution(setup.other, s1, &[setup.owner], false);
    drop(resolution);
    // Probe: does the trigger fire? Dispatch and inspect the produced effects.
    let state = dispatch(
        state,
        DomainEvent::CardUsed {
            player: setup.other,
            card: s1,
            targets: vec![setup.owner],
        },
    );
    assert!(!state.has_pending_choice(), "父荫 is locked and silent");
    // The exclusion lands in the saved resume payload: the owner is appended to the
    // exclusion tail of the ResumePlayedSha payload.
    let payload = &state.resolution.resume_payload;
    let excluded: Vec<_> = payload.get(6..).unwrap_or(&[]).to_vec();
    assert!(
        excluded.contains(&setup.owner.raw()),
        "owner excluded in the saved Sha payload: {payload:?}"
    );
    // Second Sha on the same turn: 父荫 already fired -> no exclusion.
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.phase = Phase::Play;
    let state = tx.commit().state;
    let state = dispatch(
        state,
        DomainEvent::CardUsed {
            player: setup.other,
            card: s2,
            targets: vec![setup.owner],
        },
    );
    assert!(!state.has_pending_choice());
    // The resume payload still belongs to the first Sha (s2 never registered);
    // its exclusion tail must be unchanged: only s1's owner exclusion.
    assert_eq!(
        &state.resolution.resume_payload,
        &[
            setup.other.raw(),
            setup.owner.raw(),
            1,
            0,
            1,
            s1.raw(),
            setup.owner.raw()
        ],
        "second Sha not excluded (once per turn)"
    );
}
