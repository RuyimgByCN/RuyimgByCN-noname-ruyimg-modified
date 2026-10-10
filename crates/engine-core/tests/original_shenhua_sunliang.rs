//! Golden coverage for 溃诛/掣政/立军（shenhua 孙亮，ABI 577）.

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

const KUIZHU: u32 = 1_891_631_254;
const ZHIZHENG: u32 = 3_830_747_834;
const LIJUN: u32 = 1_640_782_530;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/sunliang.skill");

struct Setup {
    tx: Transaction,
    owner: PlayerId,
    other: PlayerId,
    far: PlayerId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("sl-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
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
        skill_def: SkillDefId::new(KUIZHU),
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(ZHIZHENG),
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(LIJUN),
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

fn phase_end_event(player: PlayerId, tag: engine_api::TurnPhaseTag) -> DomainEvent {
    DomainEvent::PhaseEnded {
        current: player,
        phase: tag,
        damage_sources: 0,
        damaged_players: 0,
        card_targeted_other: false,
        current_dealt_damage_other: false,
        phase_targeted_players: 0,
        phase_hp_changed_players: 0,
    }
}

#[test]
fn package_exposes_sunliang_contract() {
    let compiled = content_compiler::compile_source_named("sl", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 577);
    assert_eq!(compiled.skill_index.len(), 3);
    let events: Vec<_> = compiled
        .trigger_index
        .iter()
        .map(|t| (t.skill_def_id, t.event.as_str(), t.optional))
        .collect();
    assert!(events.contains(&(KUIZHU, "phase_discard_end", true)));
    assert!(events.contains(&(ZHIZHENG, "phase_play_end", true)));
    assert!(events.contains(&(LIJUN, "card_used", true)));
}

/// 溃诛：弃牌阶段结束后按弃牌数选择摸牌分支，目标各摸一张。
#[test]
fn kuizhu_draw_branch() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    tx.state_mut().turn.current = setup.owner;
    tx.state_mut().turn.phase = Phase::Discard;
    tx.state_mut()
        .turn
        .set_flag(engine_core::state::turn_flag::DISCARD_PHASE_DISCARDED, 2);
    let state = tx.commit().state;
    let state = dispatch(
        state,
        phase_end_event(setup.owner, engine_api::TurnPhaseTag::Discard),
    );
    assert!(state.has_pending_choice(), "溃诛 confirm");
    let state = resolve(state, setup.owner, vec![1], false, 9_001);
    assert!(state.has_pending_choice(), "溃诛 branch prompt");
    // 摸牌分支（0）。
    let state = resolve(state, setup.owner, vec![0], false, 9_002);
    assert!(state.has_pending_choice(), "溃诛 draw targets");
    let state = resolve(
        state,
        setup.owner,
        vec![setup.other.raw(), setup.far.raw()],
        false,
        9_003,
    );
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(setup.other), 1);
    assert_eq!(state.hand_count(setup.far), 1);
}

/// 溃诛：伤害分支——体力值之和为 X 的角色各受 1 点伤害。
#[test]
fn kuizhu_damage_branch() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    tx.state_mut().turn.current = setup.owner;
    tx.state_mut().turn.phase = Phase::Discard;
    tx.state_mut()
        .turn
        .set_flag(engine_core::state::turn_flag::DISCARD_PHASE_DISCARDED, 2);
    // 将 far 体力降到 2，使体力值之和为 2 的伤害目标成立。
    run_effects(
        &mut tx,
        vec![EffectRequest::ChangeMaxHp {
            player: setup.far,
            delta: -1,
        }],
    )
    .unwrap();
    let state = tx.commit().state;
    let hp_before = state.players.get(setup.far).unwrap().hp;
    assert_eq!(hp_before, 2);
    let state = dispatch(
        state,
        phase_end_event(setup.owner, engine_api::TurnPhaseTag::Discard),
    );
    assert!(state.has_pending_choice(), "溃诛 confirm");
    let state = resolve(state, setup.owner, vec![1], false, 9_010);
    let state = resolve(state, setup.owner, vec![1], false, 9_011);
    assert!(state.has_pending_choice(), "溃诛 damage targets");
    let state = resolve(state, setup.owner, vec![setup.far.raw()], false, 9_012);
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(setup.far).unwrap().hp, hp_before - 1);
}

/// 掣政：出牌阶段结束时弃置一名攻击范围不包含你的角色的一张牌。
#[test]
fn zhizheng_discards_out_of_range_card() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    tx.state_mut().turn.current = setup.owner;
    tx.state_mut().turn.phase = Phase::Play;
    let far_hand = hand_zone(&tx, setup.far);
    let victim = alloc_card(
        &mut tx,
        setup.far,
        far_hand,
        CardKind::Sha,
        97_300,
        Suit::Club,
        2,
    );
    // 持有者装备防御马：他人到你的距离 +1，使远位角色攻击范围不包含你。
    let equip_zone = tx.state().zones.find(ZoneKind::Equip, setup.owner).unwrap();
    alloc_card(
        &mut tx,
        setup.owner,
        equip_zone,
        CardKind::HorseDef,
        97_302,
        Suit::Diamond,
        1,
    );
    let state = tx.commit().state;
    let state = dispatch(
        state,
        phase_end_event(setup.owner, engine_api::TurnPhaseTag::Play),
    );
    // 本回合用牌数 0 < 攻击范围不包含你的角色数 → 触发。
    assert!(state.has_pending_choice(), "掣政 confirm");
    let state = resolve(state, setup.owner, vec![1], false, 9_020);
    assert!(state.has_pending_choice(), "掣政 target prompt");
    let state = resolve(state, setup.owner, vec![setup.far.raw()], false, 9_021);
    assert!(state.has_pending_choice(), "掣政 card prompt");
    let state = resolve(state, setup.owner, vec![victim.raw()], false, 9_022);
    assert!(!state.has_pending_choice());
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(victim).unwrap().zone, discard);
}

/// 立军：吴势力角色出牌阶段使用实体【杀】后，持有者获得该牌并令其摸一张。
#[test]
fn lijun_gains_wu_sha() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    tx.state_mut().turn.current = setup.other;
    tx.state_mut().turn.phase = Phase::Play;
    // 将 other 改为吴势力。
    tx.state_mut()
        .group_overrides
        .insert(setup.other, "wu".into());
    let other_hand = hand_zone(&tx, setup.other);
    let sha = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Sha,
        97_301,
        Suit::Spade,
        3,
    );
    let state = tx.commit().state;
    let state = dispatch(
        state,
        DomainEvent::CardUsed {
            player: setup.other,
            card: sha,
            targets: vec![setup.far],
        },
    );
    assert!(state.has_pending_choice(), "立军 confirm");
    let state = resolve(state, setup.owner, vec![1], false, 9_030);
    assert!(!state.has_pending_choice());
    assert!(
        state.hand_cards(setup.owner).contains(&sha),
        "lord gained the Sha"
    );
    assert_eq!(state.hand_count(setup.other), 1, "Wu user drew one");
}
