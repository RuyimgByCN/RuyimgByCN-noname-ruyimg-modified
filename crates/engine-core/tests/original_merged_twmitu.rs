//! O2 全量合并第二批（2026-10-14）：家族 twmitu 的 1 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_twmitu_machine {
//! 第二百四十九批：twmitu 迷途多段拼点链状态机 slice 1 行为测试（ABI 702 /
//! tag 725 / Cont 307）。链路：准备阶段确认 → SelectTargets [1,3]（任意存活
//! 角色，可取消）→ 各目标按座次摸 2 张、摸得牌打「迷途」content tag 并公开
//! 亮出。拼点轮与崩溃面留 slice 2+。

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const TWMITU: u32 = 3_791_467_368;
const TWMITU_TAG: u32 = 3_195_907_057;
const TWMITU_SRC: &str =
    include_str!("../../../content/original/ported/tw/twmitu.skill");

fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("twmitu-b249", TWMITU_SRC).unwrap();
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
        skill_def: SkillDefId::new(TWMITU),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, other)
}

fn tagged_count(state: &GameState, player: PlayerId) -> usize {
    let hand = state.zones.find(engine_core::state::ZoneKind::Hand, player).unwrap();
    state
        .zones
        .get(hand)
        .unwrap()
        .cards
        .iter()
        .filter(|card| state.card_has_content_tag(**card, TWMITU_TAG))
        .count()
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
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
                cancelled,
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

fn fire_prepare(state: GameState, owner: PlayerId) -> (GameState, bool) {
    let mut tx = Transaction::begin(state);
    let outcome = match dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    ) {
        Err(EffectError::ChoiceRequested) => Err(EffectError::ChoiceRequested),
        Ok(effects) => engine_core::effect::run_effects(&mut tx, effects).map(|_| ()),
        Err(other) => panic!("unexpected: {other:?}"),
    };
    let opened = matches!(outcome, Err(EffectError::ChoiceRequested));
    (tx.commit().state, opened)
}

#[test]
fn twmitu_slice1_two_targets_draw_tag_show() {
    // 选两目标：各摸 2 张，4 张摸得牌全部打「迷途」标记。
    let (state, owner, other) = setup(20_260_912_51);
    let owner_hand = state.hand_count(owner);
    let other_hand = state.hand_count(other);

    let (state, opened) = fire_prepare(state, owner);
    assert!(opened, "confirm opens");
    let state = resolve(state, owner, vec![1], false, 111);
    assert!(state.has_pending_choice(), "target selection opens");
    let state = resolve(state, owner, vec![owner.raw(), other.raw()], false, 112);

    // 摸 2 + 全部打标（公开亮出为表现面，此处验证标记与手牌数）。
    assert_eq!(state.hand_count(owner), owner_hand + 2);
    assert_eq!(state.hand_count(other), other_hand + 2);
    assert_eq!(tagged_count(&state, owner), 2);
    assert_eq!(tagged_count(&state, other), 2);

    // slice 2 拼点轮随之打开：两轮拼点均 declined 收口。
    // 第一轮目标 = 座 0（owner 本人）：owner 为其指定 other 为对手后 owner 拒绝。
    let state = resolve(state, owner, vec![other.raw()], false, 113);
    let state = resolve(state, owner, vec![2], false, 114);
    // 第二轮目标 = other：owner 指定自己为对手后 other 拒绝。
    let state = resolve(state, owner, vec![owner.raw()], false, 115);
    let state = resolve(state, other, vec![2], false, 116);
    assert!(!state.has_pending_choice());
}

#[test]
fn twmitu_slice1_cancel_places_no_tags() {
    let (state, owner, other) = setup(20_260_912_52);
    let owner_hand = state.hand_count(owner);
    let (state, opened) = fire_prepare(state, owner);
    assert!(opened);
    let state = resolve(state, owner, vec![], true, 121);
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(owner), owner_hand);
    assert_eq!(tagged_count(&state, owner), 0);
    assert_eq!(tagged_count(&state, other), 0);
}


// ---- 批 250：slice 2 拼点轮（指定对手→知情→双方选牌→虚拟杀）----

fn rank_of(state: &GameState, card: CardId) -> u8 {
    state.cards.get(card).map(|c| c.rank).unwrap_or(0)
}

fn hp_of(state: &GameState, player: PlayerId) -> i32 {
    state.players.get(player).unwrap().hp
}

#[test]
fn twmitu_slice2_compare_round_winner_virtual_sha() {
    // 目标=other：owner 指定自己为对手 → other 同意 → 双方选牌 →
    // owner 出更大点数 → owner 视为对 other 使用虚拟杀（other -1 体力）。
    let (state, owner, other) = setup(20_260_912_61);
    let (state, opened) = fire_prepare(state, owner);
    assert!(opened);
    let state = resolve(state, owner, vec![1], false, 131);
    let state = resolve(state, owner, vec![other.raw()], false, 132);
    assert!(state.has_pending_choice(), "stage 3 opponent pick opens");

    // owner 指定自己为拼点对手（2p 局唯一候选）。
    let state = resolve(state, owner, vec![owner.raw()], false, 133);

    // stage 4：other 知情确认（Custom 1=同意）。
    assert!(state.has_pending_choice());
    let state = resolve(state, other, vec![1], false, 134);

    // stage 5：other 选拼点牌（取其最小点数，确保必输）。
    let other_hand = state.hand_cards(other);
    let o_pick = *other_hand
        .iter()
        .min_by_key(|card| rank_of(&state, **card))
        .unwrap();
    let state = resolve(state, other, vec![o_pick.raw()], false, 135);

    // stage 6：owner 选拼点牌（取大于 o_pick 的最大点数，确保必赢）。
    let owner_hand = state.hand_cards(owner);
    let winner_pick = owner_hand
        .iter()
        .copied()
        .filter(|card| rank_of(&state, *card) > rank_of(&state, o_pick))
        .max_by_key(|card| rank_of(&state, *card));
    let winner_pick = match winner_pick {
        Some(card) => card,
        None => *owner_hand.iter().max_by_key(|card| rank_of(&state, **card)).unwrap(),
    };
    let hp_other = hp_of(&state, other);
    // 直伤结算（登记债：目标无应闪机会），链路直接收口。
    let state = resolve(state, owner, vec![winner_pick.raw()], false, 136);
    assert!(!state.has_pending_choice());

    if rank_of(&state, winner_pick) > rank_of(&state, o_pick) {
        assert_eq!(
            hp_of(&state, other),
            hp_other - 1,
            "winner virtual sha deals 1 damage to loser"
        );
    } else {
        // 兜底：无法构造必赢时至少收口。
        assert!(!state.has_pending_choice());
    }
}

#[test]
fn twmitu_slice2_decline_closes_chain_without_damage() {
    let (state, owner, other) = setup(20_260_912_62);
    let (state, opened) = fire_prepare(state, owner);
    assert!(opened);
    let state = resolve(state, owner, vec![1], false, 141);
    let state = resolve(state, owner, vec![other.raw()], false, 142);
    let state = resolve(state, owner, vec![owner.raw()], false, 143);
    let hp_other = hp_of(&state, other);
    let hp_owner = hp_of(&state, owner);
    // other 拒绝拼点（Custom 2）。
    let state = resolve(state, other, vec![2], false, 144);
    assert!(!state.has_pending_choice());
    assert_eq!(hp_of(&state, other), hp_other);
    assert_eq!(hp_of(&state, owner), hp_owner);
}


// ---- 批 251：slice 3 崩溃面（拼点用牌 ≠ 展示牌 → owner maxHp-1）----

fn max_hp_of(state: &GameState, player: PlayerId) -> i32 {
    state.players.get(player).unwrap().max_hp as i32
}

#[test]
fn twmitu_slice3_collapse_penalty_on_mismatched_compare_card() {
    // 目标 other 的拼点用牌选未打标的原手牌（≠ 其展示牌）→ owner maxHp-1。
    let (state, owner, other) = setup(20_260_912_71);
    let owner_max = max_hp_of(&state, owner);

    let (state, opened) = fire_prepare(state, owner);
    assert!(opened);
    let state = resolve(state, owner, vec![1], false, 151);
    let state = resolve(state, owner, vec![other.raw()], false, 152);
    // stage 3：owner 指定自己为 other 的拼点对手。
    let state = resolve(state, owner, vec![owner.raw()], false, 153);
    // stage 4：other 同意拼点。
    let state = resolve(state, other, vec![1], false, 154);
    // stage 5：other 选一张未打标的原手牌（≠ 展示牌）。
    let untagged = state
        .hand_cards(other)
        .into_iter()
        .find(|card| !state.card_has_content_tag(*card, TWMITU_TAG))
        .expect("bootstrap hand card present");
    let state = resolve(state, other, vec![untagged.raw()], false, 155);
    // stage 6：owner 任选一张拼点牌。
    let owner_card = state.hand_cards(owner).first().copied().unwrap();
    let state = resolve(state, owner, vec![owner_card.raw()], false, 156);

    // 崩溃面：owner 体力上限 -1；拼点虚拟杀若开启闪响应则取消之。
    if state.has_pending_choice() {
        let state = resolve(state, other, vec![], true, 157);
        assert!(!state.has_pending_choice());
        assert_eq!(max_hp_of(&state, owner), owner_max - 1);
    } else {
        assert_eq!(max_hp_of(&state, owner), owner_max - 1);
    }
}

#[test]
fn twmitu_slice3_no_penalty_on_decline() {
    // 拒绝拼点 → 无 compare 事件 → 无崩溃惩罚。
    let (state, owner, other) = setup(20_260_912_72);
    let owner_max = max_hp_of(&state, owner);
    let (state, opened) = fire_prepare(state, owner);
    assert!(opened);
    let state = resolve(state, owner, vec![1], false, 161);
    let state = resolve(state, owner, vec![other.raw()], false, 162);
    let state = resolve(state, owner, vec![owner.raw()], false, 163);
    let state = resolve(state, other, vec![2], false, 164);
    assert!(!state.has_pending_choice());
    assert_eq!(max_hp_of(&state, owner), owner_max);
}
}
