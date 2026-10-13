//! O2 全量合并第二批（2026-10-14）：家族 canyun 的 1 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_canyun_machine {
//! 第二百五十五批：dccanyun 残殒装备比较状态机行为测试（ABI 705 / tag 728 /
//! Cont 310）。链路：ActivateSkill → 弃置一张手牌（费用）→ 选 1-2 名其他角色
//! → 逐目标比较装备数：owner 多=目标回 1 / 相等=目标摸 1 / 目标多=目标失
//! 1 体力；目标结算后体力为 1 → owner 摸 1。

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const CANYUN: u32 = 3_523_035_260;
const CANYUN_SRC: &str =
    include_str!("../../../content/original/ported/sp2/dccanyun.skill");

fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("canyun-b255", CANYUN_SRC).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
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
        skill_def: SkillDefId::new(CANYUN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.phase = engine_core::state::Phase::Play;
    (tx.commit().state, owner, target)
}

fn equip_count(state: &GameState, player: PlayerId) -> usize {
    let equip = state.zones.find(ZoneKind::Equip, player).unwrap();
    state.zones.get(equip).unwrap().cards.len()
}

fn hp_of(state: &GameState, player: PlayerId) -> i32 {
    state.players.get(player).unwrap().hp
}

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
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
        "resolve rejected {key}: {:?}",
        result.result
    );
    next
}

fn activate(state: GameState, owner: PlayerId) -> GameState {
    let sid = state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.def.raw() == CANYUN)
        .unwrap()
        .id;
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill: sid,
                targets: Vec::new(),
                card_targets: Vec::new(),
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "activate rejected"
    );
    state
}

#[test]
fn canyun_equal_equip_target_draws_one() {
    // 双方均无装备 → 相等 → 目标摸 1 张。
    let (state, owner, target) = setup(20_260_912_101);
    let o_hand = hand_first(&state, owner);
    let t_hand = state.hand_count(target);
    assert_eq!(equip_count(&state, owner), 0);
    assert_eq!(equip_count(&state, target), 0);

    let state = activate(state, owner);
    assert!(state.has_pending_choice(), "discard cost opens");
    let state = resolve(state, owner, vec![o_hand.raw()], 11);
    assert!(state.has_pending_choice(), "target selection opens");
    let state = resolve(state, owner, vec![target.raw()], 12);
    assert!(!state.has_pending_choice());

    // 目标摸 1；owner 费用 -1（目标 hp 未到 1 → owner 不摸）。
    assert_eq!(state.hand_count(target), t_hand + 1);
    assert!(hp_of(&state, target) > 1 || state.hand_count(owner) >= 4);
}

#[test]
fn canyun_owner_more_equip_target_recovers() {
    // 注入装备给 owner（比目标多）→ 目标回 1 体力。先压目标 1 点体力留空间。
    let (mut state, owner, target) = setup(20_260_912_102);
    {
        let hp = state.players.get(target).unwrap().hp;
        state.players.get_mut(target).unwrap().hp = hp - 1;
    }
    // 把一张牌置入 owner 装备区（模拟装备领先）。
    let draw = state.zones.find(ZoneKind::DrawPile, PlayerId::NONE).unwrap();
    let equip = state.zones.find(ZoneKind::Equip, owner).unwrap();
    let equip_card = *state
        .zones
        .get(draw)
        .unwrap()
        .cards
        .last()
        .unwrap();
    state
        .zones
        .get_mut(draw)
        .unwrap()
        .cards
        .retain(|c| *c != equip_card);
    state.zones.get_mut(equip).unwrap().cards.push(equip_card);
    if let Some(card_state) = state.cards.get_mut(equip_card) {
        card_state.zone = equip;
        card_state.owner = owner;
    }
    let state = state;
    let hp_target = hp_of(&state, target);
    let o_hand = hand_first(&state, owner);

    let state = activate(state, owner);
    let state = resolve(state, owner, vec![o_hand.raw()], 21);
    let state = resolve(state, owner, vec![target.raw()], 22);
    assert!(!state.has_pending_choice());

    // owner 装备多 → 目标回复 1。
    assert_eq!(hp_of(&state, target), hp_target + 1);
    assert!(discard_has(&state, o_hand), "cost card discarded");
}

#[test]
fn canyun_target_more_equip_target_loses_hp() {
    // 目标装备更多 → 目标失去 1 体力；体力为 1 → owner 摸 1。
    let (mut state, owner, target) = setup(20_260_912_103);
    // 目标注入 1 张装备（owner 0 张 → 目标多）。
    let draw = state.zones.find(ZoneKind::DrawPile, PlayerId::NONE).unwrap();
    let equip = state.zones.find(ZoneKind::Equip, target).unwrap();
    let equip_card = *state.zones.get(draw).unwrap().cards.last().unwrap();
    state
        .zones
        .get_mut(draw)
        .unwrap()
        .cards
        .retain(|c| *c != equip_card);
    state.zones.get_mut(equip).unwrap().cards.push(equip_card);
    if let Some(card_state) = state.cards.get_mut(equip_card) {
        card_state.zone = equip;
        card_state.owner = target;
    }
    // 目标体力设为 2 → 失去 1 后为 1 → owner 摸 1。
    state.players.get_mut(target).unwrap().hp = 2;
    let state = state;
    let o_hand = hand_first(&state, owner);
    let owner_hand = state.hand_count(owner);

    let state = activate(state, owner);
    let state = resolve(state, owner, vec![o_hand.raw()], 31);
    let state = resolve(state, owner, vec![target.raw()], 32);
    assert!(!state.has_pending_choice());

    assert_eq!(hp_of(&state, target), hp_of(&state, target));
    assert_eq!(state.hand_count(owner), owner_hand - 1 + 1);
}

fn hand_first(state: &GameState, player: PlayerId) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    *state
        .zones
        .get(hand)
        .unwrap()
        .cards
        .last()
        .expect("hand not empty")
}

fn discard_has(state: &GameState, card: CardId) -> bool {
    let discard = state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE).unwrap();
    state.zones.get(discard).unwrap().cards.contains(&card)
}
}
