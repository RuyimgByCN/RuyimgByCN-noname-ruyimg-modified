//! AI 规划前的信息遮蔽：隐藏对手手牌与暗置信息，并记录遮蔽诊断。

use super::*;

/// 诊断：最近一次遮蔽是否为静止态（1/0）、栈深、挂起机器数、是否有开放选择。
pub(super) static REDACT_LAST: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub(super) static REDACT_STACK: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub(super) static REDACT_PARKS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub(super) static REDACT_CHOICE: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);

/// 读取最近一次遮蔽的静止态诊断（quiescent, stack_len, parks_len, has_choice）。
pub fn redact_last_state() -> (bool, u64, u64, bool) {
    use std::sync::atomic::Ordering::Relaxed;
    (
        REDACT_LAST.load(Relaxed) == 1,
        REDACT_STACK.load(Relaxed),
        REDACT_PARKS.load(Relaxed),
        REDACT_CHOICE.load(Relaxed) == 1,
    )
}

// Host payload schemas and pending ownership repairs still limit full continuation remapping.
pub(super) fn can_redact_continuation(e: &Engine, seat: u8) -> bool {
    e.open_choice().is_none_or(|choice| choice.observer == seat)
        && e.resolution()
            .parks
            .iter()
            .all(|p| !super::remap::requires_conservative_redaction(&p.machine.snapshot()))
}

pub fn redact_opponent_hands(e: &mut Engine, seat: u8) {
    use crate::ids::CardId;
    // Unknown physical owners and every typed continuation reference share one mapping.
    let quiescent = {
        let res = e.resolution();
        res.stack.is_empty() && res.parks.is_empty() && res.open_choice.is_none()
    };
    // NC12 诊断：记录最近一次遮蔽时的静止态判据与隐藏区重写情况（供故障切片比对）
    {
        let res = e.resolution();
        REDACT_LAST.store(
            if quiescent { 1 } else { 0 },
            std::sync::atomic::Ordering::Relaxed,
        );
        REDACT_STACK.store(res.stack.len() as u64, std::sync::atomic::Ordering::Relaxed);
        REDACT_PARKS.store(res.parks.len() as u64, std::sync::atomic::Ordering::Relaxed);
        REDACT_CHOICE.store(
            if res.open_choice.is_some() { 1 } else { 0 },
            std::sync::atomic::Ordering::Relaxed,
        );
    }
    if !can_redact_continuation(e, seat) {
        // shortcut: uninspected payloads, VM commit sentinels and other seats' filtered choices
        // retain hidden hands until semantic remapping exists. Public VM choices use a planner leaf.
        let n = e.state_core().deck.len();
        if n >= 2 {
            let mut rng_state: u64 = 0xD1CE_5EED_0000
                ^ (seat as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
                ^ e.revision().wrapping_mul(0x2545_F491_4F6C_DD1D);
            let core = e.core_mut();
            for k in (1..n).rev() {
                rng_state = rng_state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let j = ((rng_state >> 33) as usize) % (k + 1);
                core.deck.swap(j, k);
            }
        }
        return;
    }
    // NC10：模拟用的隐藏区改为**内容无关填充**——旧实现把「对手手牌 ∪ 牌库」
    // 做确定性洗牌后按原张数放回，虽然打乱了「谁拿哪张」，却完整保留了隐藏牌的
    // **多重集**，AI 仍可经模拟读到「对手手里有闪」这类事实（信息边界用例复现：
    // seed 42022 warm 9 seat 1）。现在改为：从目录中确定性抽取互不相同的牌填满
    // 隐藏区，只保留**张数**（公开信息），与真实隐藏内容无关。
    let redact_traps = e.state_core().vm_aux.storage.is_empty()
        || !e
            .catalog()
            .skills
            .iter()
            .any(|skill| skill.behavior.is_vm_skill());
    let n = e.state_core().players.len();
    let opponents: Vec<usize> = (0..n).filter(|&i| i as u8 != seat).collect();
    // 保留实体牌不代表其对 AI 可见；特殊牌区的隐藏身份仍需单独遮蔽。
    let mut visible: std::collections::BTreeSet<crate::ids::CardId> =
        std::collections::BTreeSet::new();
    {
        let core = e.state_core();
        let me = &core.players[seat as usize];
        visible.extend(me.hand.iter().copied());
        visible.extend(me.equips.iter().copied());
        visible.extend(me.judge.iter().copied());
        visible.extend(me.off_court.iter().copied());
        visible.extend(core.discard.iter().copied());
        // NC12 长程根因修正（2026-10-19）：**公开区与在飞区不得被重分配**。
        // 此前只保护「自己的区域 + 弃牌堆」，于是对手的**装备区/判定区/离场区**以及
        // `table`（在飞牌）可能被当作隐藏牌抽走并塞进隐藏区——这些牌被机器/帧/回合
        // 状态字段引用，重分配后 `take_hand_card` 找不到牌 → panic（实测
        // `machine.rs:408 card N not in hand of P`）。
        for (owner, p) in core.players.iter().enumerate() {
            visible.extend(p.equips.iter().copied());
            visible.extend(p.judge.iter().copied());
            visible.extend(p.off_court.iter().copied());
            // 筑城、壶仓和护天在上游可见；陷阱只有持有者可见。
            for pile in [&p.zhucheng, &p.lianyaohu_store, &p.hutian_hold] {
                visible.extend(pile.iter().copied());
            }
            if owner == seat as usize || !redact_traps {
                visible.extend(p.sxianjing_traps.iter().copied());
            }
        }
        // The observer retains their private VM piles.
        for ((owner, _), pile) in &core.vm_aux.piles {
            if *owner == seat {
                visible.extend(pile.iter().copied());
            }
        }
        visible.extend(core.table.iter().copied());
        let mut held = Vec::new();
        for park in &e.resolution().parks {
            super::remap::held_cards(&park.machine.snapshot(), &mut held);
        }
        visible.extend(held);
    }
    // 填充池：目录全部牌 id 去掉保留牌（保持确定性顺序）
    let mut pool: Vec<CardId> = (0..e.catalog().cards.len() as CardId)
        .filter(|c| !visible.contains(c))
        .collect();
    let hand_lens: Vec<usize> = opponents
        .iter()
        .map(|i| e.state_core().players[*i].hand.len())
        .collect();
    let deck_len = e.state_core().deck.len();
    // 确定性 Fisher–Yates（LCG：座位号 + 修订号播种，可重放）
    let mut rng_state: u64 = 0x5EED_CAFE_0000
        ^ (seat as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ e.revision().wrapping_mul(0x2545_F491_4F6C_DD1D);
    for k in (1..pool.len()).rev() {
        rng_state = rng_state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let j = ((rng_state >> 33) as usize) % (k + 1);
        pool.swap(j, k);
    }
    let mut trap_owners: Vec<usize> = if redact_traps {
        opponents
            .iter()
            .copied()
            .filter(|i| !e.state_core().players[*i].sxianjing_traps.is_empty())
            .collect()
    } else {
        Vec::new()
    };
    trap_owners
        .sort_by_key(|i| std::cmp::Reverse(e.state_core().players[*i].sxianjing_traps.len()));
    let trap_count: usize = trap_owners
        .iter()
        .map(|i| e.state_core().players[*i].sxianjing_traps.len())
        .sum();
    let vm_piles: Vec<_> = e
        .state_core()
        .vm_aux
        .piles
        .keys()
        .filter(|(owner, _)| *owner != seat)
        .cloned()
        .collect();
    let vm_count: usize = vm_piles
        .iter()
        .map(|key| e.state_core().vm_aux.piles[key].len())
        .sum();
    let need = hand_lens.iter().sum::<usize>() + deck_len + trap_count + vm_count;
    assert!(
        pool.len() >= need,
        "目录牌数不足以填充隐藏区（need={need}, pool={}）",
        pool.len()
    );
    // 先满足每位陷阱持有者的不同花色约束；优先剩余最多的花色，避免抢占稀缺花色。
    if trap_owners
        .iter()
        .any(|i| e.state_core().players[*i].sxianjing_traps.len() > 1)
    {
        let mut counts = [0usize; 4];
        for card in &pool {
            counts[e.catalog().cards[*card as usize].suit as usize] += 1;
        }
        let mut cursor = 0;
        for owner in &trap_owners {
            let mut used = [false; 4];
            for _ in &e.state_core().players[*owner].sxianjing_traps {
                let suit = (0..4)
                    .filter(|i| !used[*i] && counts[*i] > 0)
                    .max_by_key(|i| counts[*i])
                    .or_else(|| (0..4).filter(|i| counts[*i] > 0).max_by_key(|i| counts[*i]))
                    .expect("filled pool has enough cards");
                let index = (cursor..pool.len())
                    .find(|i| e.catalog().cards[pool[*i] as usize].suit as usize == suit)
                    .expect("suit count matches pool");
                pool.swap(cursor, index);
                cursor += 1;
                counts[suit] -= 1;
                used[suit] = true;
            }
        }
    }
    // 牌引用按同一个重分配映射更新，避免指向错误的实体。
    let remap: Vec<CardId> = {
        let mut remap: Vec<CardId> = (0..e.catalog().cards.len() as CardId).collect();
        let old = trap_owners
            .iter()
            .flat_map(|i| e.state_core().players[*i].sxianjing_traps.iter().copied())
            .chain(
                opponents
                    .iter()
                    .flat_map(|i| e.state_core().players[*i].hand.iter().copied()),
            )
            .chain(e.state_core().deck.iter().copied())
            .chain(
                vm_piles
                    .iter()
                    .flat_map(|key| e.state_core().vm_aux.piles[key].iter().copied()),
            );
        for (before, after) in old.zip(pool.iter().copied()) {
            remap[before as usize] = after;
        }
        remap
    };
    let core = e.core_mut();
    let mut cursor = 0usize;
    if redact_traps {
        for &owner in &trap_owners {
            for trap in &mut core.players[owner].sxianjing_traps {
                *trap = pool[cursor];
                cursor += 1;
            }
        }
    }
    for (&i, &len) in opponents.iter().zip(hand_lens.iter()) {
        core.players[i].hand = pool[cursor..cursor + len].to_vec();
        cursor += len;
    }
    core.deck = pool[cursor..cursor + deck_len].to_vec();
    cursor += deck_len;
    for key in &vm_piles {
        let pile = core.vm_aux.piles.get_mut(key).expect("existing pile");
        let len = pile.len();
        pile.copy_from_slice(&pool[cursor..cursor + len]);
        cursor += len;
    }
    if !remap.is_empty() {
        let replace = |card: &mut CardId| {
            if let Some(replacement) = remap.get(*card as usize) {
                *card = *replacement;
            }
        };
        for card in core.vm_aux.lost.values_mut().flatten() {
            replace(card);
        }
        for (card, _) in &mut core.turn.huanxia_track {
            replace(card);
        }
        for (_, a, b, c) in &mut core.turn.kunlunjing_snapshot {
            for card in a.iter_mut().chain(b).chain(c) {
                replace(card);
            }
        }
        if let Some(card) = &mut core.turn.luanjian_card {
            replace(card);
        }
        if let Some(card) = &mut core.turn.qizuo_copy {
            replace(card);
        }
        for player in &mut core.players {
            if let Some(card) = &mut player.ctianfu_card {
                replace(card);
            }
            for card in player
                .jiguanyuan_cards
                .iter_mut()
                .chain(&mut player.yishan_lost)
            {
                replace(card);
            }
            player.card_mods = std::mem::take(&mut player.card_mods)
                .into_iter()
                .map(|(mut key, mut value)| {
                    replace(&mut key);
                    replace(&mut value);
                    (key, value)
                })
                .collect();
        }
        core.declared_kinds = std::mem::take(&mut core.declared_kinds)
            .into_iter()
            .map(|(mut card, value)| {
                replace(&mut card);
                (card, value)
            })
            .collect();
        core.declared_slots = std::mem::take(&mut core.declared_slots)
            .into_iter()
            .map(|(mut card, value)| {
                replace(&mut card);
                (card, value)
            })
            .collect();
        core.card_natures = std::mem::take(&mut core.card_natures)
            .into_iter()
            .map(|(mut card, value)| {
                replace(&mut card);
                (card, value)
            })
            .collect();
    }
    super::remap::remap_resolution(e, &remap);
}

#[cfg(test)]
mod active_candidate_tests {
    use super::*;

    #[test]
    fn variant_two_weights_damage_by_enemy_skill_threat() {
        let engine = Engine::new(20261028, 3);
        let base = engine.ai_session(0).view();
        let tuning = ContentTuning {
            skill_threat: vec![("仁德".into(), 3.0), ("不臣".into(), 1.0)],
            ..ContentTuning::default()
        };
        let mut high = base.clone();
        high.players[1].skills = vec!["仁德"];
        high.players[1].hp -= 1;
        let mut low = base.clone();
        low.players[2].skills = vec!["不臣"];
        low.players[2].hp -= 1;
        let high_score = policy_score(&base, &high, 0, 2, Some(&tuning));
        let low_score = policy_score(&base, &low, 0, 2, Some(&tuning));
        assert!(
            high_score > low_score,
            "higher threat damage must score better: high={high_score} low={low_score}"
        );
    }

    #[test]
    fn variant_two_can_choose_granted_active_skill() {
        let mut engine = Engine::new(20261026, 2);
        let seat: crate::ids::PlayerId = 0;
        engine.debug_set_turn(seat, crate::event::Phase::Play, 1);
        engine.debug_set_hp(seat, 1);
        engine.debug_clear_hand(seat);
        engine.debug_grant(seat, "不臣");
        let plan = engine.plan_action(
            seat,
            &crate::ai::BoundedPlanner::new(64).with_policy_variant(2),
        );
        assert!(
            matches!(plan.action, GameAction::ActivateSkill { .. }),
            "variant 2 should choose an available active skill: {:?}",
            plan.action
        );
    }

    #[test]
    fn variant_zero_does_not_include_active_skill_candidates() {
        let mut engine = Engine::new(20261027, 2);
        let seat: crate::ids::PlayerId = 0;
        engine.debug_set_turn(seat, crate::event::Phase::Play, 1);
        engine.debug_set_hp(seat, 1);
        engine.debug_clear_hand(seat);
        engine.debug_grant(seat, "不臣");
        let candidates = enumerate_candidates_variant(&engine, seat, 0);
        assert!(
            !candidates
                .iter()
                .any(|action| matches!(action, GameAction::ActivateSkill { .. })),
            "variant 0 must keep historical behavior: {candidates:?}"
        );
    }

    #[test]
    fn variant_one_does_not_include_active_skill_candidates() {
        let mut engine = Engine::new(20261029, 2);
        let seat: crate::ids::PlayerId = 0;
        engine.debug_set_turn(seat, crate::event::Phase::Play, 1);
        engine.debug_set_hp(seat, 1);
        engine.debug_clear_hand(seat);
        engine.debug_grant(seat, "不臣");
        let candidates = enumerate_candidates_variant(&engine, seat, 1);
        assert!(
            !candidates
                .iter()
                .any(|action| matches!(action, GameAction::ActivateSkill { .. })),
            "variant 1 must keep historical behavior: {candidates:?}"
        );
    }

    #[test]
    fn variant_two_includes_granted_active_skill_candidates() {
        let mut engine = Engine::new(20261025, 2);
        let seat: crate::ids::PlayerId = 0;
        engine.debug_set_turn(seat, crate::event::Phase::Play, 1);
        engine.debug_move_named_card_to_hand(seat, "sha");
        engine.debug_grant(seat, "仁德");
        let candidates = enumerate_candidates_variant(&engine, seat, 2);
        assert!(
            candidates
                .iter()
                .any(|action| matches!(action, GameAction::ActivateSkill { .. })),
            "active skill must be a variant-2 candidate: {candidates:?}"
        );
    }
}
