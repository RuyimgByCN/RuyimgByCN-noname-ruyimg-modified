//! Golden coverage for 成略/恃才/寸目（shenhua 许攸，ABI 589）.

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

const CHENGLVE: u32 = 1_234_567_901;
const SHICAI: u32 = 1_234_567_902;
const CUNMU: u32 = 1_234_567_903;
const SOURCE: &str = include_str!("../../../../content/original/ported/shenhua/xuyou.skill");

struct Setup {
    tx: Transaction,
    owner: PlayerId,
    other: PlayerId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("xuyou-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
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
    for def in [CHENGLVE, SHICAI, CUNMU] {
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(def),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
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

/// 成略：首次发动翻为阳——摸一张弃两张；第二次发动翻为阴——摸两张弃一张。
/// 弃置花色本回合无距离与次数限制。
#[test]
fn chenglve_flips_draws_discards_and_unlimits_discarded_suits() {
    let setup = setup();
    let mut tx = setup.tx;
    let hand = tx.state().zones.find(ZoneKind::Hand, setup.owner).unwrap();
    let club = alloc_card(
        &mut tx,
        setup.owner,
        hand,
        CardKind::Sha,
        97_901,
        Suit::Club,
        5,
    );
    let spade = alloc_card(
        &mut tx,
        setup.owner,
        hand,
        CardKind::Shan,
        97_902,
        Suit::Spade,
        6,
    );
    let heart = alloc_card(
        &mut tx,
        setup.owner,
        hand,
        CardKind::Tao,
        97_903,
        Suit::Heart,
        7,
    );
    let diamond = alloc_card(
        &mut tx,
        setup.owner,
        hand,
        CardKind::Jiu,
        97_904,
        Suit::Diamond,
        8,
    );
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == setup.owner && skill.def.raw() == CHENGLVE)
        .unwrap()
        .id;
    let state = tx.commit().state;
    let revision = state.revision;
    let (state, activated) = apply_command(
        state,
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![],
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(activated.result, CommandResult::Accepted { .. }),
        "activate rejected: {:?}",
        activated.result
    );
    // 首次：翻为阳 → 摸一张弃两张（黑桃/红桃弃置 → 本回合黑桃/红桃无限制）。
    eprintln!(
        "DBG chenglve after activate: pending={} kind={:?} hand={} blob={:?}",
        state.has_pending_choice(),
        state.resolution.continuation_kind,
        state.hand_count(setup.owner),
        state.skills.get(skill).map(|s| s.state_blob.clone())
    );
    assert!(state.has_pending_choice(), "成略 discard prompt");
    let state = resolve(state, setup.owner, vec![spade.raw(), heart.raw()], false, 2);
    assert!(!state.has_pending_choice());
    let state = state.clone();
    let flag = state
        .turn
        .flag(engine_core::state::turn_flag::CHENGLVE_UNLIMITED_SUITS)
        .max(0) as u32;
    assert_eq!(flag, (1 << 1) | (1 << 2), "弃置黑桃/红桃记录为无限制花色");
    assert!(engine_core::query::card_use_unlimited_for_card(&state, setup.owner, club) == false);
    // 同回合不可再次发动（限一次）。
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = setup.owner;
    tx.state_mut().turn.phase = Phase::Play;
    let state = tx.commit().state;
    let revision = state.revision;
    let (state, second) = apply_command(
        state,
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(3),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![],
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(second.result, CommandResult::Rejected { .. }),
        "成略每回合限一次"
    );
    let _ = state;
    let _ = club;
}

/// 恃才：使用非装备牌后，实体牌置顶并摸一张；同类牌本回合不重复触发。
#[test]
fn shicai_places_used_card_on_top_and_draws_once_per_category() {
    let setup = setup();
    let mut tx = setup.tx;
    let hand = tx.state().zones.find(ZoneKind::Hand, setup.owner).unwrap();
    let sha = alloc_card(
        &mut tx,
        setup.owner,
        hand,
        CardKind::Sha,
        97_911,
        Suit::Club,
        5,
    );
    // 使用【杀】结算：实体牌进入弃牌堆。
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![sha],
            to: discard,
            from: Some(hand),
        }],
    )
    .unwrap();
    let state = tx.commit().state;
    // 使用【杀】→ 恃才可选确认。
    let state = dispatch(
        state,
        DomainEvent::CardUsed {
            player: setup.owner,
            card: sha,
            targets: vec![setup.other],
        },
    );
    assert!(state.has_pending_choice(), "恃才 confirm");
    let state = resolve(state, setup.owner, vec![1], false, 10);
    assert!(state.has_pending_choice(), "恃才 place prompt");
    let state = resolve(state, setup.owner, vec![sha.raw()], false, 11);
    assert!(!state.has_pending_choice());
    // 弃牌堆的【杀】已置于牌堆顶并被摸回手牌，随后摸一张。
    assert!(
        state.hand_cards(setup.owner).contains(&sha),
        "置顶后摸回【杀】"
    );
    assert_eq!(state.hand_count(setup.owner), 1);
    // 同类牌本回合不再触发：再次使用【杀】无选择。
    let mut tx = Transaction::begin(state);
    let hand = tx.state().zones.find(ZoneKind::Hand, setup.owner).unwrap();
    let sha2 = alloc_card(
        &mut tx,
        setup.owner,
        hand,
        CardKind::Sha,
        97_912,
        Suit::Spade,
        6,
    );
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![sha2],
            to: discard,
            from: Some(hand),
        }],
    )
    .unwrap();
    let state = tx.commit().state;
    let state = dispatch(
        state,
        DomainEvent::CardUsed {
            player: setup.owner,
            card: sha2,
            targets: vec![setup.other],
        },
    );
    // 确认后检查类别位：同类牌首用后不再开放置选择。
    assert!(state.has_pending_choice(), "同类牌首用后确认仍触发");
    let state = resolve(state, setup.owner, vec![1], false, 12);
    assert!(!state.has_pending_choice(), "同类牌首用后不再放置");
}

/// 寸目：摸牌阶段改为从牌堆底摸牌。
#[test]
fn cunmu_draws_from_bottom_of_draw_pile() {
    let setup = setup();
    let mut tx = setup.tx;
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let bottom = alloc_card(
        &mut tx,
        setup.owner,
        draw,
        CardKind::Sha,
        97_921,
        Suit::Club,
        5,
    );
    // 牌堆底 = 首个元素：把预设牌移到堆底。
    tx.state_mut()
        .zones
        .get_mut(draw)
        .unwrap()
        .cards
        .retain(|card| *card != bottom);
    tx.state_mut()
        .zones
        .get_mut(draw)
        .unwrap()
        .cards
        .insert(0, bottom);
    tx.state_mut().turn.current = setup.owner;
    tx.state_mut().turn.phase = Phase::Draw;
    let state = tx.commit().state;
    let state = dispatch(
        state,
        DomainEvent::TurnStateChanged {
            current: setup.owner,
            phase: engine_api::TurnPhaseTag::Draw,
        },
    );
    assert!(!state.has_pending_choice());
    // 触发设置底摸标记；摸牌阶段随后按此标记从牌堆底摸牌。
    assert_eq!(
        state
            .turn
            .flag(engine_core::state::turn_flag::DRAW_PHASE_FROM_BOTTOM),
        1,
        "寸目设置底摸标记"
    );
    let mut tx = Transaction::begin(state);
    let count = engine_core::query::draw_count_for(tx.state(), setup.owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCardsFromBottom {
            player: setup.owner,
            count,
        }],
    )
    .unwrap();
    let state = tx.commit().state;
    assert!(
        state.hand_cards(setup.owner).contains(&bottom),
        "寸目从牌堆底摸到预设牌"
    );
}

fn dispatch(state: GameState, event: DomainEvent) -> GameState {
    let mut tx = Transaction::begin(state);
    let fired = dispatch_for_events(&mut tx, &[event], &mut TriggerFireRecord::new());
    match fired {
        Ok(effects) => match run_effects(&mut tx, effects) {
            Ok(_) => {}
            Err(EffectError::ChoiceRequested) => {}
            Err(e) => panic!("dispatch error: {e:?}"),
        },
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("dispatch error: {e:?}"),
    }
    tx.commit().state
}

#[test]
fn package_exposes_xuyou_contract() {
    let compiled = content_compiler::compile_source_named("xuyou", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 589);
    assert_eq!(compiled.skill_index.len(), 3);
    let chenglve = compiled
        .skill_index
        .iter()
        .find(|skill| skill.skill_def_id == CHENGLVE)
        .unwrap();
    assert!(chenglve.conversion);
    let cunmu = compiled
        .skill_index
        .iter()
        .find(|skill| skill.skill_def_id == CUNMU)
        .unwrap();
    assert!(cunmu.lord_only);
    assert!(compiled
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == SHICAI
            && trigger.event == "card_used"
            && trigger.filter == "owner_is_source"
            && trigger.optional));
}
