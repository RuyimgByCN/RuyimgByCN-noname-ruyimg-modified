//! Golden coverage for 良姻/箜声（shenhua 周妃，ABI 579）.

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

const LIANGYIN: u32 = 4_194_727_596;
const KONGSHENG: u32 = 3_654_831_984;
const SOURCE: &str = include_str!("../../../../content/original/ported/shenhua/zhoufei.skill");

struct Setup {
    tx: Transaction,
    owner: PlayerId,
    other: PlayerId,
    far: PlayerId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("zf-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
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
        skill_def: SkillDefId::new(LIANGYIN),
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(KONGSHENG),
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

/// 箜声：准备阶段将一张牌置为「箜」。
fn seed_kong(state: GameState, owner: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.phase = Phase::Prepare;
    let owner_hand = hand_zone(&tx, owner);
    alloc_card(
        &mut tx,
        owner,
        owner_hand,
        CardKind::Sha,
        97_399,
        Suit::Club,
        9,
    );
    let state = tx.commit().state;
    let state = dispatch(
        state,
        DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::Prepare,
        },
    );
    assert!(state.has_pending_choice(), "箜声 confirm");
    let state = resolve(state, owner, vec![1], false, 9_000);
    assert!(state.has_pending_choice(), "箜声 place prompt");
    let hand: Vec<_> = state.hand_cards(owner);
    resolve(state, owner, vec![hand[0].raw()], false, 9_001)
}

#[test]
fn package_exposes_zhoufei_contract() {
    let compiled = content_compiler::compile_source_named("zf", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 579);
    assert_eq!(compiled.skill_index.len(), 2);
    let triggers: Vec<_> = compiled
        .trigger_index
        .iter()
        .filter(|t| t.skill_def_id == LIANGYIN || t.skill_def_id == KONGSHENG)
        .collect();
    assert_eq!(triggers.len(), 5);
    let kongsheng_end = triggers
        .iter()
        .find(|t| t.skill_def_id == KONGSHENG && t.event == "phase_end")
        .unwrap();
    assert!(!kongsheng_end.optional);
}

/// 良姻（失牌分支）：双方各摸一张；手牌等于箜数且受伤时回复。
#[test]
fn liangyin_draw_and_recover() {
    let setup = setup();
    // 先置 1 张「箜」。
    let state = setup.tx.commit().state;
    let state = seed_kong(state, setup.owner);
    // 持有者受伤。
    let mut tx = Transaction::begin(state);
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
    // 持有者回合外失去一张牌 → 良姻失牌分支。
    tx.state_mut().turn.current = setup.other;
    tx.state_mut().turn.phase = Phase::Play;
    let owner_hand = hand_zone(&tx, setup.owner);
    let lost = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Sha,
        97_400,
        Suit::Club,
        2,
    );
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    // 真实移牌（合成事件不改变牌位）；移牌中良姻触发将开确认。
    match run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![lost],
            to: discard,
            from: Some(owner_hand),
        }],
    ) {
        Ok(_) => {}
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("move error: {e:?}"),
    }
    let state = tx.commit().state;
    assert!(state.has_pending_choice(), "良姻 confirm");
    let state = resolve(state, setup.owner, vec![1], false, 9_010);
    assert!(state.has_pending_choice(), "良姻 draw target");
    let state = resolve(state, setup.owner, vec![setup.far.raw()], false, 9_011);
    // 双方各摸一张；箜数 1：far 摸后手牌 1 == 箜数且未受伤 → 不回复；
    // 持有者摸后手牌 1 == 箜数且受伤 → 开回复选择。
    assert!(state.has_pending_choice(), "良姻 recover prompt");
    let hp_before = state.players.get(setup.owner).unwrap().hp;
    // 摸牌中途的良姻获牌分支确认（拒绝），随后摸牌分支恢复并开回复选择。
    let state = resolve(state, setup.owner, vec![0], false, 9_012);
    assert!(state.has_pending_choice(), "良姻 recover prompt");
    let state = resolve(state, setup.owner, vec![0], false, 9_013);
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(setup.owner).unwrap().hp, hp_before + 1);
}

/// 良姻（获牌分支）：弃一张牌并令一名其他角色弃一张牌。
#[test]
fn liangyin_discard_branch() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    tx.state_mut().turn.current = setup.owner;
    tx.state_mut().turn.phase = Phase::Play;
    // 持有者摸一张（真实摸牌记录获牌历史并触发良姻获牌分支）。
    let draw_pile = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let paid = alloc_card(
        &mut tx,
        setup.owner,
        draw_pile,
        CardKind::Sha,
        97_401,
        Suit::Spade,
        3,
    );
    let far_hand = hand_zone(&tx, setup.far);
    let victim = alloc_card(
        &mut tx,
        setup.far,
        far_hand,
        CardKind::Tao,
        97_402,
        Suit::Heart,
        4,
    );
    match run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: setup.owner,
            count: 1,
        }],
    ) {
        Ok(_) => {}
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("gain error: {e:?}"),
    }
    let state = tx.commit().state;
    assert!(state.has_pending_choice(), "良姻 gain confirm");
    let state = resolve(state, setup.owner, vec![1], false, 9_020);
    assert!(state.has_pending_choice(), "良姻 discard prompt");
    let state = resolve(state, setup.owner, vec![paid.raw()], false, 9_021);
    assert!(state.has_pending_choice(), "良姻 target prompt");
    let state = resolve(state, setup.owner, vec![setup.far.raw()], false, 9_022);
    assert!(state.has_pending_choice(), "良姻 target card prompt");
    let state = resolve(state, setup.owner, vec![victim.raw()], false, 9_023);
    assert!(!state.has_pending_choice());
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(victim).unwrap().zone, discard);
}

/// 箜声：结束阶段非装备箜回手、装备箜令一名角色使用。
#[test]
fn kongsheng_end_returns_and_equips() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    // 置 2 张「箜」：一张桃（非装备）、一张武器。
    tx.state_mut().turn.phase = Phase::Prepare;
    // 先给两张手牌（一张桃、一件武器），再走准备阶段置箜流程。
    let owner_hand = hand_zone(&tx, setup.owner);
    let peach = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Tao,
        97_403,
        Suit::Heart,
        2,
    );
    let weapon = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Weapon,
        97_404,
        Suit::Diamond,
        6,
    );
    let state = tx.commit().state;
    let state = dispatch(
        state,
        DomainEvent::TurnStateChanged {
            current: setup.owner,
            phase: engine_api::TurnPhaseTag::Prepare,
        },
    );
    assert!(state.has_pending_choice(), "箜声 confirm");
    let state = resolve(state, setup.owner, vec![1], false, 9_030);
    assert!(state.has_pending_choice(), "箜声 place prompt");
    let state = resolve(
        state,
        setup.owner,
        vec![peach.raw(), weapon.raw()],
        false,
        9_031,
    );
    assert!(!state.has_pending_choice());
    // 结束阶段：非装备（手牌里的桃）回手、武器令目标使用。
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.phase = Phase::End;
    let state = tx.commit().state;
    let state = dispatch(
        state,
        DomainEvent::TurnStateChanged {
            current: setup.owner,
            phase: engine_api::TurnPhaseTag::End,
        },
    );
    // 装备箜目标选择（回手无触发时直接到达）。
    assert!(state.has_pending_choice(), "箜声 equip target");
    let state = resolve(state, setup.owner, vec![setup.other.raw()], false, 9_031);
    // 装备移入/使用中，其他角色获得/失去牌触发良姻确认（全部拒绝）。
    let mut seq = 9_033u64;
    let mut state = state;
    while state.has_pending_choice() {
        state = resolve(state, setup.owner, vec![0], false, seq);
        seq += 1;
    }
    // 武器装备到 other。
    let other_equip = state.zones.find(ZoneKind::Equip, setup.other).unwrap();
    assert!(
        state
            .zones
            .get(other_equip)
            .unwrap()
            .cards
            .contains(&weapon),
        "weapon equipped to the target"
    );
    // 非装备箜（桃）回到持有者手牌。
    assert_eq!(state.hand_count(setup.owner), 1, "non-equip 箜 returned");
}
