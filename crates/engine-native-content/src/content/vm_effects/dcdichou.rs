//! UP37-R01 敌仇（`dcdichou` `3304378096` + `dcdichou_eff` `2754298115`，tag 121）：
//! 完整复刻 `engine-core` `ResolveDcdichou` 的多步状态机：
//! - `action == 0`（准备阶段开始）：选择废除一个未废除的装备栏（1..=5），弃置该栏已有装备，
//!   随后衔接 `action == 1`。
//! - `action == 1`：收集场上有 `dcdichou` 标记（`3304378096 > 0`）且有 `he` 牌的存活角色，
//!   令 `owner` 选一名目标，随后衔接 `action == 2`。
//! - `action == 2`：从目标的 `he` 牌中选 1 张弃置，本回合获得临时技 `dcdichou_eff`（`2754298115`），
//!   并将标记 `2754298115` 置为所弃置牌的牌名字数（攻击范围与出牌阶段使用【杀】次数上限各 +字数）。
//! - `action == 3`：直接按所弃置 `card` 授予本回合临时技 `dcdichou_eff` 并置字数标记。
//! - `action == 10`（结束阶段）：收集场上有 `dcdichou` 标记或本回合曾移去过 `dcdichou` 标记的存活角色，
//!   清空所有角色的 `dcdichou` 标记后，令 `owner` 选择其中任意名角色（`1..=k`），按座位序各造成 1 点火焰伤害。
use super::batch13_targets::seats_from;
use super::*;
use crate::catalog::{CardKind, EquipSlot};

pub(super) const DCDICHOU_MARK: u32 = 3_304_378_096;
pub(super) const DCDICHOU_EFF_DEF: u32 = 2_754_298_115;

fn enabled_equip_slots(core: &GameStateCore, owner: u8) -> Vec<ChoiceOption> {
    let disabled_mask = core
        .vm_aux
        .flags
        .get(&(owner, "disabled_equip_slots".to_string()))
        .copied()
        .unwrap_or(0);
    (1u8..=5)
        .filter(|&slot| (disabled_mask & (1u32 << slot)) == 0)
        .map(ChoiceOption::Tag)
        .collect()
}

fn marked_targets_with_cards(core: &GameStateCore, owner: u8) -> Vec<ChoiceOption> {
    seats_from(core, owner)
        .into_iter()
        .filter(|&p| {
            core.player(p).alive
                && core.vm_aux.marks.get(&(p, DCDICHOU_MARK)).copied().unwrap_or(0) > 0
                && !cards(core, p).is_empty()
        })
        .map(ChoiceOption::Player)
        .collect()
}

fn phase_end_candidates(core: &GameStateCore, owner: u8) -> Vec<u8> {
    seats_from(core, owner)
        .into_iter()
        .filter(|&p| {
            core.player(p).alive
                && (core.vm_aux.marks.get(&(p, DCDICHOU_MARK)).copied().unwrap_or(0) > 0
                    || core
                        .vm_aux
                        .flags
                        .get(&(p, "dcdichou_mark_removed_this_turn".to_string()))
                        .copied()
                        .unwrap_or(0)
                        != 0)
        })
        .collect()
}

pub(super) fn card_chinese_name_len(catalog: &Catalog, card: u32) -> i32 {
    let Some(def) = catalog.cards.get(card as usize) else {
        return 1;
    };
    match def.kind {
        CardKind::Sha
        | CardKind::FireSha
        | CardKind::LeiSha
        | CardKind::Shan
        | CardKind::Tao
        | CardKind::Jiu => 1,
        CardKind::Duel
        | CardKind::ShanDian
        | CardKind::HorsePlus
        | CardKind::HorseMinus
        | CardKind::Shuigong
        | CardKind::HongShui
        | CardKind::Feibiao
        | CardKind::Yougeng
        | CardKind::Qingtuan
        | CardKind::Chunbing
        | CardKind::Molicha
        | CardKind::Xiajiao
        | CardKind::Hufu
        | CardKind::Caoyao => 2,
        CardKind::WuxingPan
        | CardKind::ShouLiJian
        | CardKind::Sanjian
        | CardKind::Suolianjia
        | CardKind::Zhaomingdan
        | CardKind::Jihuocard
        | CardKind::Shujinsan
        | CardKind::Jinlianzhu
        | CardKind::Tanhuadong
        | CardKind::Yuanbaorou
        | CardKind::Gudonggeng
        | CardKind::Liyutang
        | CardKind::Luyugeng
        | CardKind::Qinggang
        | CardKind::Qilin
        | CardKind::Qinglong
        | CardKind::Hanbing
        | CardKind::Renwang
        | CardKind::Guanshi
        | CardKind::Armor => 3,
        CardKind::Cixiong => 5,
        _ => 4,
    }
}

pub(super) fn active_ask(
    effect: &EffectRequest,
    state: &[u32],
    core: &GameStateCore,
) -> Option<ExpectedAsk> {
    let EffectRequest::ResolveDcdichou { player, .. } = effect else {
        return None;
    };
    let owner = seat(*player, core)?;
    match state {
        [1] => {
            let options = enabled_equip_slots(core, owner);
            if options.is_empty() {
                return None;
            }
            Some(ask(
                owner,
                "rvm_choice",
                ChoiceKind::PickTag,
                options,
                1,
                1,
                false,
            ))
        }
        [2] => {
            let options = marked_targets_with_cards(core, owner);
            if options.is_empty() {
                return None;
            }
            Some(ask(
                owner,
                "rvm_choice",
                ChoiceKind::SelectPlayers,
                options,
                1,
                1,
                false,
            ))
        }
        [3, target] if (*target as usize) < core.players.len() => {
            let options = cards(core, *target as u8);
            if options.is_empty() {
                return None;
            }
            Some(ask(
                owner,
                "rvm_discard_pick",
                ChoiceKind::SelectCards,
                options,
                1,
                1,
                false,
            ))
        }
        [10, count, rest @ ..] if *count as usize == rest.len() && !rest.is_empty() => {
            let options: Vec<ChoiceOption> = rest
                .iter()
                .filter(|&&p| (p as usize) < core.players.len() && core.player(p as u8).alive)
                .map(|&p| ChoiceOption::Player(p as u8))
                .collect();
            let max = options.len();
            if max == 0 {
                return None;
            }
            Some(ask(
                owner,
                "rvm_choice",
                ChoiceKind::SelectPlayers,
                options,
                1,
                max,
                false,
            ))
        }
        _ => None,
    }
}

pub(super) fn step(
    ctx: &mut Ctx<'_>,
    effect: &EffectRequest,
    state: &mut Vec<u32>,
) -> Option<Flow> {
    let EffectRequest::ResolveDcdichou {
        player,
        target,
        card,
        action,
        helper_skill_def,
        ..
    } = effect
    else {
        return None;
    };
    let owner = seat(*player, ctx.core)?;
    if !ctx.core.player(owner).alive {
        return Some(Flow::Done);
    }
    let helper_raw = if helper_skill_def.is_some() {
        helper_skill_def.raw()
    } else {
        DCDICHOU_EFF_DEF
    };
    if state.is_empty() {
        return Some(match *action {
            0 => {
                let slots = enabled_equip_slots(ctx.core, owner);
                if slots.is_empty() {
                    start_stage2(ctx, owner, state)
                } else {
                    *state = vec![1];
                    wait(ask(
                        owner,
                        "rvm_choice",
                        ChoiceKind::PickTag,
                        slots,
                        1,
                        1,
                        false,
                    ))
                }
            }
            1 => start_stage2(ctx, owner, state),
            2 => {
                let Some(t) = seat(*target, ctx.core) else {
                    return Some(Flow::Done);
                };
                start_stage3(ctx, owner, t, state)
            }
            3 => {
                apply_dcdichou_eff_buff(ctx, owner, helper_raw, card.raw());
                Flow::Done
            }
            10 => start_stage10(ctx, owner, state),
            _ => Flow::Done,
        });
    }
    Some(match state.as_slice() {
        [1] => {
            let Some(ChoiceOption::Tag(slot)) = ctx.answer().first().copied() else {
                return Some(Flow::Done);
            };
            if (1..=5).contains(&slot) {
                disable_equip_slot_and_discard(ctx, owner, slot);
            }
            start_stage2(ctx, owner, state)
        }
        [2] => {
            let Some(ChoiceOption::Player(chosen)) = ctx.answer().first().copied() else {
                return Some(Flow::Done);
            };
            if !marked_targets_with_cards(ctx.core, owner).contains(&ChoiceOption::Player(chosen))
            {
                return Some(Flow::Done);
            }
            start_stage3(ctx, owner, chosen, state)
        }
        [3, t] => {
            let target_seat = *t as u8;
            let Some(ChoiceOption::Card(picked)) = ctx.answer().first().copied() else {
                return Some(Flow::Done);
            };
            if ctx.core.player(target_seat).hand.contains(&picked)
                || ctx.core.player(target_seat).equips.contains(&picked)
            {
                discard(ctx, target_seat, &[picked]);
                apply_dcdichou_eff_buff(ctx, owner, helper_raw, picked);
            }
            Flow::Done
        }
        [10, count, rest @ ..] if *count as usize == rest.len() => {
            let allowed: Vec<u8> = rest.iter().map(|&p| p as u8).collect();
            let mut chosen: Vec<u8> = ctx
                .answer()
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Player(p)
                        if allowed.contains(p) && ctx.core.player(*p).alive =>
                    {
                        Some(*p)
                    }
                    _ => None,
                })
                .collect();
            chosen.sort_by_key(|&p| ctx.core.seat_steps_from(owner, p));
            chosen.dedup();
            if chosen.is_empty() {
                return Some(Flow::Done);
            }
            let mut next_state = vec![11, 0];
            next_state.extend(chosen.into_iter().map(u32::from));
            *state = next_state;
            pump_stage11(ctx, owner, state)
        }
        [11, ..] => pump_stage11(ctx, owner, state),
        _ => Flow::Done,
    })
}

fn start_stage2(ctx: &mut Ctx<'_>, owner: u8, state: &mut Vec<u32>) -> Flow {
    let options = marked_targets_with_cards(ctx.core, owner);
    if options.is_empty() {
        return Flow::Done;
    }
    *state = vec![2];
    wait(ask(
        owner,
        "rvm_choice",
        ChoiceKind::SelectPlayers,
        options,
        1,
        1,
        false,
    ))
}

fn start_stage3(ctx: &mut Ctx<'_>, owner: u8, target: u8, state: &mut Vec<u32>) -> Flow {
    let options = cards(ctx.core, target);
    if options.is_empty() {
        return Flow::Done;
    }
    *state = vec![3, target as u32];
    wait(ask(
        owner,
        "rvm_discard_pick",
        ChoiceKind::SelectCards,
        options,
        1,
        1,
        false,
    ))
}

fn start_stage10(ctx: &mut Ctx<'_>, owner: u8, state: &mut Vec<u32>) -> Flow {
    let candidates = phase_end_candidates(ctx.core, owner);
    for p in 0..ctx.core.players.len() as u8 {
        ctx.core.vm_aux.marks.remove(&(p, DCDICHOU_MARK));
        ctx.core
            .vm_aux
            .flags
            .remove(&(p, "dcdichou_mark_removed_this_turn".to_string()));
    }
    if candidates.is_empty() {
        return Flow::Done;
    }
    let options: Vec<ChoiceOption> = candidates.iter().copied().map(ChoiceOption::Player).collect();
    let max = options.len();
    let mut next_state = vec![10, max as u32];
    next_state.extend(candidates.into_iter().map(u32::from));
    *state = next_state;
    wait(ask(
        owner,
        "rvm_choice",
        ChoiceKind::SelectPlayers,
        options,
        1,
        max,
        false,
    ))
}

fn pump_stage11(ctx: &mut Ctx<'_>, owner: u8, state: &mut [u32]) -> Flow {
    if state.len() < 3 || !ctx.core.player(owner).alive {
        return Flow::Done;
    }
    while (state[1] as usize) < state.len() - 2 {
        let target = state[2 + state[1] as usize] as u8;
        state[1] += 1;
        if !ctx.core.player(target).alive {
            continue;
        }
        *ctx.core
            .vm_aux
            .flags
            .entry((owner, "lifetime".to_string()))
            .or_insert(0) += 1;
        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
            Some(owner),
            target,
            1,
            crate::event::DamageNature::Fire,
            None,
        )));
        let has_more_alive = state[2 + state[1] as usize..]
            .iter()
            .any(|&p| ctx.core.player(p as u8).alive);
        return if has_more_alive {
            Flow::Yield
        } else {
            Flow::Done
        };
    }
    Flow::Done
}

fn disable_equip_slot_and_discard(ctx: &mut Ctx<'_>, owner: u8, slot: u8) {
    let bit = 1u32 << slot;
    *ctx.core
        .vm_aux
        .flags
        .entry((owner, "disabled_equip_slots".to_string()))
        .or_insert(0) |= bit;
    ctx.core
        .vm_aux
        .flags
        .insert((owner, format!("equip_slot_disabled:{slot}")), 1);
    let want = match slot {
        1 => EquipSlot::Weapon,
        2 => EquipSlot::Armor,
        3 => EquipSlot::HorsePlus,
        4 => EquipSlot::HorseMinus,
        5 => EquipSlot::Treasure,
        _ => return,
    };
    let equipped: Vec<u32> = ctx
        .core
        .player(owner)
        .equips
        .iter()
        .copied()
        .filter(|&c| ctx.catalog.card_slot_of(ctx.core, c) == Some(want))
        .collect();
    if !equipped.is_empty() {
        discard(ctx, owner, &equipped);
    }
}

fn apply_dcdichou_eff_buff(ctx: &mut Ctx<'_>, owner: u8, helper_raw: u32, card: u32) {
    let name_len = card_chinese_name_len(ctx.catalog, card);
    if let Some(def) = crate::rvm_ops::resolve_skill_def(ctx.catalog, helper_raw)
        .or_else(|| ctx.catalog.skill_by_name("dcdichou_eff"))
    {
        if !ctx.core.has_skill(owner, def) {
            ctx.add_skill_until_turn_end(owner, def);
        }
    }
    ctx.core
        .vm_aux
        .flags
        .insert((owner, format!("skill:{helper_raw}")), 1);
    ctx.core
        .vm_aux
        .flags
        .insert((owner, format!("skill_turn:{helper_raw}")), 1);
    ctx.core
        .vm_aux
        .marks
        .insert((owner, DCDICHOU_EFF_DEF), name_len);
    ctx.core
        .vm_aux
        .marks
        .insert((owner, helper_raw), name_len);
}

pub(super) fn validate(effect: &EffectRequest, state: &[u32], core: &GameStateCore) -> bool {
    let EffectRequest::ResolveDcdichou { player, .. } = effect else {
        return false;
    };
    let seats = core.players.len();
    if player.raw() == 0 || (player.raw() as usize) > seats {
        return false;
    }
    match state {
        [1] | [2] => true,
        [3, target] => (*target as usize) < seats,
        [10, count, rest @ ..] => {
            !rest.is_empty()
                && rest.len() <= seats
                && *count as usize == rest.len()
                && rest.iter().all(|&p| (p as usize) < seats)
        }
        [11, cursor, rest @ ..] => {
            !rest.is_empty()
                && rest.len() <= seats
                && (*cursor as usize) <= rest.len()
                && rest.iter().all(|&p| (p as usize) < seats)
        }
        _ => false,
    }
}
