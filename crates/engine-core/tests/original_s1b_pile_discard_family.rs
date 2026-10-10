//! 第一百九十五批：S1-B 消费池 dormant 契约行为测试（S1-A 直构 SkillPileRef 模式）。
//! tuifeng2（sp 退锋·终段）/ nsdiemou（diy 谋谋）/ dcpijian（xianding 被坚）：
//! 技能堆计数门控 + discard_skill_pile 清堆 + 简单效果链。
//! 供给技（tuifeng/nsjiquan/dcyanzuo 置入面）落地前为 dormant 契约——本测试以
//! 直构堆验证门控与效果链真实可执行。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Zone, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

fn setup(seed: u64, source: &str, tag: &str, skill_def: u32, pile_def: u32) -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named(tag, source).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let other = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, other)
}

fn inject_hand_card(state: GameState, owner: PlayerId, def: u32) -> (GameState, CardId) {
    let mut tx = Transaction::begin(state);
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Other,
        suit: Suit::Spade,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    (tx.commit().state, card)
}

/// 直接构造 (owner, pile_def) 技能堆并移入 n 张牌。
fn fill_pile(state: GameState, owner: PlayerId, pile_def: u32, n: usize) -> GameState {
    let mut state = state;
    for i in 0..n {
        let (s2, card) = inject_hand_card(state, owner, 9_500 + i as u32);
        state = s2;
        let mut tx = Transaction::begin(state);
        let existing = tx
            .state()
            .skill_piles
            .iter()
            .find(|p| p.owner == owner && p.skill_def == pile_def)
            .map(|p| p.zone);
        let zone = match existing {
            Some(zone) => zone,
            None => {
                let zone = tx.state_mut().zones.alloc(Zone {
                    id: engine_api::ZoneId::NONE,
                    kind: ZoneKind::Expansion,
                    owner,
                    cards: Vec::new(),
                });
                tx.state_mut()
                    .skill_piles
                    .push(engine_core::state::SkillPileRef {
                        owner,
                        skill_def: pile_def,
                        zone,
                        usable_as_hand: false,
                        wants_lost_count: false,
                    });
                zone
            }
        };
        let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
        tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
        tx.state_mut()
            .zones
            .get_mut(hand)
            .unwrap()
            .cards
            .retain(|c| *c != card);
        if let Some(card_state) = tx.state_mut().cards.get_mut(card) {
            card_state.zone = zone;
        }
        state = tx.commit().state;
    }
    state
}

fn fire_phase(
    state: GameState,
    owner: PlayerId,
    phase: TurnPhaseTag,
) -> (GameState, Result<(), EffectError>) {
    let mut tx = Transaction::begin(state);
    let outcome = match dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase,
        }],
        &mut TriggerFireRecord::new(),
    ) {
        Err(EffectError::ChoiceRequested) => Err(EffectError::ChoiceRequested),
        Ok(effects) => engine_core::effect::run_effects(&mut tx, effects).map(|_| ()),
        Err(other) => panic!("unexpected dispatch error: {other:?}"),
    };
    (tx.commit().state, outcome)
}

fn resolve(state: GameState, owner: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (next, result) = apply_command(
        state,
        Command {
            actor: owner,
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
        "resolve rejected: {:?}",
        result.result
    );
    next
}

fn has_skill(state: &GameState, owner: PlayerId, skill_def: u32) -> bool {
    state
        .skills
        .sorted_live()
        .iter()
        .any(|skill| skill.owner == owner && skill.def.raw() == skill_def)
}

const TUIFENG2: u32 = 2_717_343_011;
const TUIFENG_PILE: u32 = 2_854_026_704;
const TUIFENG3: u32 = 3_847_747_290;
const TUIFENG2_SRC: &str = include_str!("../../../content/original/ported/sp/tuifeng2.skill");
const SP: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

#[test]
fn tuifeng2_contract_and_effects() {
    // 契约：包内触发存在 + ABI 随 discard_skill_pile 抬升。
    let package = content_schema::ContentPackage::from_json_bytes(SP).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 684);
    let trigger = package
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == TUIFENG2)
        .expect("tuifeng2 trigger");
    assert_eq!(trigger.event, "phase_prepare");

    // 行为：堆 2 张 → 摸 4 + 获退锋3（等量标记）+ 堆清空。
    let source = include_str!("../../../content/original/ported/sp/tuifeng2.skill");
    let (state, owner, _other) = setup(20_260_916_01, source, "tuifeng2", TUIFENG2, TUIFENG_PILE);
    let state = fill_pile(state, owner, TUIFENG_PILE, 2);
    let hand_before = state.hand_count(owner);

    let (state, outcome) = fire_phase(state, owner, TurnPhaseTag::Prepare);
    assert!(outcome.is_ok(), "forced effects, got {outcome:?}");
    assert_eq!(
        state.hand_count(owner),
        hand_before + 4,
        "draw 2×N with N=2"
    );
    assert!(has_skill(&state, owner, TUIFENG3), "退锋3 granted");
    assert_eq!(state.skill_pile_cards(owner, TUIFENG_PILE).len(), 0, "pile drained");
}

const NSDIEMOU: u32 = 3_934_300_047;
const NSJIQUAN_MARK: u32 = 3_934_300_045;
const NSDIEMOU_SRC: &str = include_str!("../../../content/original/ported/diy/nsdiemou.skill");
const DIY: &[u8] = include_bytes!("../../../content/original/ported/diy/diy.npack.json");

#[test]
fn nsdiemou_contract_and_awaken_effects() {
    let package = content_schema::ContentPackage::from_json_bytes(DIY).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 684);
    let trigger = package
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == NSDIEMOU)
        .expect("nsdiemou trigger");
    assert_eq!(trigger.event, "phase_play");

    // 行为：堆 2 张（2 人局存活数=2，门控 N+1>2 即 N≥2）→ 摸 N + 减上限 + 清堆 + once。
    let source = include_str!("../../../content/original/ported/diy/nsdiemou.skill");
    let (state, owner, _other) = setup(20_260_916_02, source, "nsdiemou", NSDIEMOU, NSJIQUAN_MARK);
    let state = fill_pile(state, owner, NSJIQUAN_MARK, 2);
    let hand_before = state.hand_count(owner);

    let (state, outcome) = fire_phase(state, owner, TurnPhaseTag::Play);
    assert!(outcome.is_ok(), "forced awakening, got {outcome:?}");
    assert_eq!(state.hand_count(owner), hand_before + 2, "draw N=2");
    assert_eq!(state.skill_pile_cards(owner, NSJIQUAN_MARK).len(), 0, "pile drained");
}

fn has_skill_state(state: &GameState, owner: PlayerId, skill_def: u32) -> bool {
    state
        .skills
        .sorted_live()
        .iter()
        .any(|skill| skill.owner == owner && skill.def.raw() == skill_def)
}

const DCPIJIAN: u32 = 2_217_810_417;
const DCYANZUO_PILE: u32 = 3_086_522_468;
const DCPIJIAN_SRC: &str = include_str!("../../../content/original/ported/xianding/dcpijian.skill");
const XIANDING: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

#[test]
fn dcpijian_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(XIANDING).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 684);
    let trigger = package
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == DCPIJIAN)
        .expect("dcpijian trigger");
    assert_eq!(trigger.event, "phase_end");
    assert_eq!(trigger.filter, "owner_skill_pile_nonempty_3086522468");
    assert!(!trigger.optional, "locked skill");
}

#[test]
fn dcpijian_discards_pile_and_damages_target() {
    // 3 人局：存活=3，堆 3 张达门。
    let compiled = content_compiler::compile_source_named("dcpijian-test", DCPIJIAN_SRC).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 20_260_916_03, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(DCPIJIAN),
    });
    run_effects(&mut tx, effects).unwrap();
    let state = tx.commit().state;
    let state = fill_pile(state, owner, DCYANZUO_PILE, 3);
    let target_hp_before = state.players.get(target).unwrap().hp;

    let (state, outcome) = fire_phase(state, owner, TurnPhaseTag::End);
    assert!(
        matches!(outcome, Err(EffectError::ChoiceRequested)),
        "choose_other_damage opens target selection: {outcome:?}"
    );
    // 选目标（choose_other_damage 的 SelectTargets）→ 伤害 2 结算。
    let state = resolve(state, owner, vec![target.raw()], 20_260_916_31);
    assert!(!state.has_pending_choice());
    assert_eq!(state.skill_pile_cards(owner, DCYANZUO_PILE).len(), 0, "pile drained");
    assert_eq!(
        state.players.get(target).unwrap().hp,
        target_hp_before - 2,
        "target takes 2 damage"
    );
}


