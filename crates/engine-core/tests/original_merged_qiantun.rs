//! O2 全量合并第二批（2026-10-14）：家族 qiantun 的 1 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_qiantun_machine {
//! 第二百五十二批：jsrgqiantun 谦吞拼点族状态机行为测试（ABI 703 / tag 726 /
//! Cont 308）。链路：ActivateSkill → 选其他有手牌角色 → 目标展示任意张手牌
//! （强制 ≥1）→ owner 选拼点牌（不受限）→ 目选拼点牌（手中有展示牌则限展示
//! 牌）→ rank 比较：赢则获得剩余展示牌，没赢（含平）则获得其余手牌。拼点牌
//! 均置入弃牌堆。

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const QIANTUN: u32 = 4_020_601_661;
const QIANTUN_SRC: &str =
    include_str!("../../../content/original/ported/jsrg/jsrgqiantun.skill");

fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("qiantun-b252", QIANTUN_SRC).unwrap();
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
        skill_def: SkillDefId::new(QIANTUN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.phase = engine_core::state::Phase::Play;
    (tx.commit().state, owner, target)
}

fn hand_of(state: &GameState, player: PlayerId) -> Vec<CardId> {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    state.zones.get(hand).unwrap().cards.clone()
}

fn rank_of(state: &GameState, card: CardId) -> u8 {
    state.cards.get(card).map(|c| c.rank).unwrap_or(0)
}

fn discard_has(state: &GameState, card: CardId) -> bool {
    let discard = state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE).unwrap();
    state.zones.get(discard).unwrap().cards.contains(&card)
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

fn activate(state: GameState, owner: PlayerId) -> GameState {
    let sid = state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.def.raw() == QIANTUN)
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
fn qiantun_win_path_takes_remaining_shown_cards() {
    // 目标展示 3 张 → 目选拼点牌限于展示牌（弃置 1 张）→ owner 出最大点数
    // 赢 → 获得剩余 2 张展示牌。
    let (state, owner, target) = setup(20_260_912_81);
    let state = activate(state, owner);
    assert!(state.has_pending_choice(), "target selection opens");
    let state = resolve(state, owner, vec![target.raw()], false, 11);

    // 目标展示 3 张（保留 1 张不展示）。
    let t_hand = hand_of(&state, target);
    let shown: Vec<u32> = t_hand[..3].iter().map(|c| c.raw()).collect();
    let kept_unshown = t_hand[3];
    let state = resolve(state, target, shown.clone(), false, 12);

    // owner 选自己最大的拼点牌。
    let o_hand = hand_of(&state, owner);
    let p_pick = o_hand
        .iter()
        .copied()
        .max_by_key(|card| rank_of(&state, *card))
        .unwrap();
    let state = resolve(state, owner, vec![p_pick.raw()], false, 13);

    // 目标从展示牌中选拼点牌：选其中最大的。
    let shown_ids: Vec<CardId> = shown.iter().map(|raw| CardId::new(*raw)).collect();
    let t_pick = shown_ids
        .iter()
        .copied()
        .max_by_key(|card| rank_of(&state, *card))
        .unwrap();
    println!("PRE14 t_pick={} p_pick={} shown={:?} t_hand={:?} o_hand={:?}",
        t_pick.raw(), p_pick.raw(), shown,
        hand_of(&state, target).iter().map(|c| c.raw()).collect::<Vec<_>>(),
        hand_of(&state, owner).iter().map(|c| c.raw()).collect::<Vec<_>>());
    let state = resolve(state, target, vec![t_pick.raw()], false, 14);
    assert!(!state.has_pending_choice());

    // 拼点牌置入弃牌堆（两侧）。
    assert!(discard_has(&state, p_pick));
    assert!(discard_has(&state, t_pick));

    let owner_won = rank_of(&state, p_pick) > rank_of(&state, t_pick);
    if owner_won {
        // 赢：剩余 2 张展示牌归 owner。
        let remaining: Vec<CardId> = shown_ids
            .iter()
            .copied()
            .filter(|card| *card != t_pick)
            .collect();
        let o_hand_now = hand_of(&state, owner);
        for card in &remaining {
            assert!(o_hand_now.contains(card), "shown card taken by owner");
        }
        assert!(!hand_of(&state, target).contains(&remaining[0]));
    } else {
        // 没赢：owner 获得非展示牌（那张 kept_unshown）。
        assert!(hand_of(&state, owner).contains(&kept_unshown));
    }
    let _ = kept_unshown;
}

#[test]
fn qiantun_lose_path_takes_non_shown_cards() {
    // owner 出最小点数且目标展示牌中有更大者 → owner 没赢 → 获得目标未展示
    // 的手牌。
    let (state, owner, target) = setup(20_260_912_82);
    let state = activate(state, owner);
    let state = resolve(state, owner, vec![target.raw()], false, 21);

    let t_hand = hand_of(&state, target);
    // 目标展示全部 4 张 → 没赢时无可获得的非展示牌，改用展示 3 张留 1 张。
    let shown: Vec<u32> = t_hand[..3].iter().map(|c| c.raw()).collect();
    let kept = t_hand[3];
    let state = resolve(state, target, shown.clone(), false, 22);

    // owner 出最小点数。
    let o_hand = hand_of(&state, owner);
    let p_pick = o_hand
        .iter()
        .copied()
        .min_by_key(|card| rank_of(&state, *card))
        .unwrap();
    let state = resolve(state, owner, vec![p_pick.raw()], false, 23);

    // 目标从展示牌中选出大于 p_pick 的牌（若存在 → owner 没赢）。
    let shown_ids: Vec<CardId> = shown.iter().map(|raw| CardId::new(*raw)).collect();
    let bigger = shown_ids
        .iter()
        .copied()
        .find(|card| rank_of(&state, *card) > rank_of(&state, p_pick));
    let t_pick = match bigger {
        Some(card) => card,
        None => shown_ids.iter().copied().max_by_key(|card| rank_of(&state, *card)).unwrap(),
    };
    let state = resolve(state, target, vec![t_pick.raw()], false, 24);
    assert!(!state.has_pending_choice());

    let owner_won = rank_of(&state, p_pick) > rank_of(&state, t_pick);
    if !owner_won {
        // 没赢（含平）：owner 获得未展示牌。
        assert!(
            hand_of(&state, owner).contains(&kept),
            "non-shown card taken on loss"
        );
        assert!(!hand_of(&state, target).contains(&kept));
    } else {
        // 罕见：owner 最小仍大于所有展示牌 → 赢臂，展示牌归 owner。
        let remaining: Vec<CardId> = shown_ids
            .iter()
            .copied()
            .filter(|card| *card != t_pick)
            .collect();
        let o_hand_now = hand_of(&state, owner);
        assert!(o_hand_now.contains(&remaining[0]));
    }
    assert!(discard_has(&state, p_pick));
    assert!(discard_has(&state, t_pick));
}

#[test]
fn qiantun_cancel_places_nothing() {
    let (state, owner, target) = setup(20_260_912_83);
    let state = activate(state, owner);
    assert!(state.has_pending_choice());
    let state = resolve(state, owner, vec![], true, 31);
    assert!(!state.has_pending_choice());
    assert_eq!(hand_of(&state, target).len(), 4);
}
}
