//! O2 全量合并第二批（2026-10-14）：家族 dcsbjianwei 的 1 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_dcsbjianwei_draw_topup {
//! 第二百二十三批：dcsbjianwei 坚卫摸牌补面晋升行为测试（v1.11.6 增量层散批）。
//! 主面：摸牌阶段手牌<2 → 摸 4（locked forced，tunchu 同款建模）。
//! 债：起始手牌+4 面（gameDrawBegin 数值改写无原语）。
//! 同批定界：shiji 6 件全复合；jsrg 抽样全复合；v1.11.6 增量 68 件头部扫过
//! ——offline pe/lx/ps 系列与 collab 质数/牌名档位系列均为复合。

use engine_api::{CardId, DomainEvent, PlayerId, SkillDefId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const JIANWEI: u32 = 2_772_977_525;
const JIANWEI_SRC: &str =
    include_str!("../../../content/original/ported/xianding/dcsbjianwei.skill");
const XIANDING: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("jianwei-b223", JIANWEI_SRC).unwrap();
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
        skill_def: SkillDefId::new(JIANWEI),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, other)
}

fn fire_draw(state: GameState, current: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current,
            phase: engine_api::TurnPhaseTag::Draw,
        }],
        &mut TriggerFireRecord::new(),
    )
    .expect("forced dispatch");
    engine_core::effect::run_effects(&mut tx, effects).expect("forced effects");
    tx.commit().state
}

#[test]
fn jianwei_contract_and_draw_topup_gate() {
    // 契约：包内触发存在（phase_draw、locked 非可选）。
    let package = content_schema::ContentPackage::from_json_bytes(XIANDING).unwrap();
    let trigger = package
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == JIANWEI)
        .expect("jianwei trigger");
    assert_eq!(trigger.event, "phase_draw");
    assert!(!trigger.optional, "locked forced");

    // 手牌 ≥2（bootstrap）→ 门控拦截，不摸。
    let (state, owner, other) = setup(20_260_928_01);
    let hand_full = state.hand_count(owner);
    let state = fire_draw(state, owner);
    assert_eq!(state.hand_count(owner), hand_full, "gate blocks at >=2");

    // 压手牌到 1 张 → 摸 4。
    let state = shrink_hand_to(state, owner, 1);
    let hand_low = state.hand_count(owner);
    let state = fire_draw(state, owner);
    assert_eq!(
        state.hand_count(owner),
        hand_low + 4,
        "hand<2 gates top-up of 4"
    );
}

/// 压低手牌至 target_count 张（逐张移入弃牌堆；同步卡区状态）。
fn shrink_hand_to(mut state: GameState, owner: PlayerId, target_count: u32) -> GameState {
    use engine_core::state::ZoneKind;
    while state.hand_count(owner) > target_count {
        let mut tx = Transaction::begin(state);
        let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
        let discard = tx
            .state()
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .expect("discard pile");
        let card = tx.state().zones.get(hand).unwrap().cards[0];
        tx.state_mut()
            .zones
            .get_mut(hand)
            .unwrap()
            .cards
            .retain(|c| *c != card);
        tx.state_mut().zones.get_mut(discard).unwrap().cards.push(card);
        if let Some(card_state) = tx.state_mut().cards.get_mut(card) {
            card_state.zone = discard;
            card_state.owner = PlayerId::NONE;
        }
        state = tx.commit().state;
    }
    state
}
}
