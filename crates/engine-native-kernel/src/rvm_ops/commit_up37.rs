//! Batch 18 (`UP37` Complete Closure): authoritative host-side skill resolvers for
//! `UP37-R02..R09` and `UP37-N02..N07`.

use super::commit::pop_deck;
use super::*;
pub mod decision34;
pub mod decision34_ext;
pub mod decision34_pack9;
pub mod decision34_u08_l19_l20_l27;
mod generals;
use generals::*;

pub const UP37_STORAGE_DISPATCH_KEY: u32 = 0x5550_3337;

pub const OLJIANGWU_ID: u32 = 3_718_591_883;
pub const STARBISHI_ID: u32 = 2_784_805_231;
pub const JSRGZONGHAI_ID: u32 = 3_103_978_481;
pub const JIGE_ID: u32 = 3_859_369_025;
pub const DCSBJUZHAN_ID: u32 = 3_220_030_848;
pub const JISHE_ID: u32 = 3_011_223_344;
pub const STARDANGCHEN_ID: u32 = 2_784_819_597;
pub const DCSBQINGSHI_ID: u32 = 2_271_027_796;
pub const DCSBZAOJI_ID: u32 = 3_421_915_330;
pub const SHENFU_ID: u32 = 2_862_962_594;
pub const OLSBGUQU_ID: u32 = 4_204_242_468;
pub const DCSBTAOZHOU_ID: u32 = 3_591_321_970;
pub const DCXIANGCHEN_ID: u32 = 2_749_573_634;
pub const POTCUIFENG_ID: u32 = 2_365_008_257;
pub const ZEXING_ID: u32 = 2_331_836_028;
pub const TT_ZHIYI_ID: u32 = 3_644_472_252;
pub const TWRENFANG_ID: u32 = 3_456_624_936;
pub const TWJUEMIE_ID: u32 = 2_465_023_183;
pub const TWJIANMING_ID: u32 = 2_495_679_541;
pub const RELONGHUN_ID: u32 = 3_894_575_372;
pub const OLHUAGUANG_ID: u32 = 3_010_477_673;
pub const OLXUANBAI_ID: u32 = 3_762_297_239;
pub const OLFANGMA_ID: u32 = 3_715_058_816;
pub const OLJINGXIAN_ID: u32 = 2_801_773_264;
pub const OLJUNZHU_ID: u32 = 2_795_607_434;
pub const OLREHUANSHI_ID: u32 = 3_274_133_709;
pub const OLSHILUN_ID: u32 = 4_097_066_031;
pub const OLZHUOMING_ID: u32 = 3_643_715_420;

fn first_other_alive(core: &GameStateCore, p: PlayerId) -> Option<PlayerId> {
    core.seat_order_from(p)
        .into_iter()
        .find(|&q| q != p && core.player(q).alive)
}

fn flag(core: &GameStateCore, p: PlayerId, key: &str) -> u32 {
    core.vm_aux
        .flags
        .get(&(p, key.to_string()))
        .copied()
        .unwrap_or(0)
}

fn set_flag(core: &mut GameStateCore, p: PlayerId, key: &str, val: u32) {
    if val == 0 {
        core.vm_aux.flags.remove(&(p, key.to_string()));
    } else {
        core.vm_aux.flags.insert((p, key.to_string()), val);
    }
}

fn turn_var(core: &GameStateCore, p: PlayerId, key: &str) -> u32 {
    core.vm_aux
        .flags
        .get(&(p, key.to_string()))
        .copied()
        .unwrap_or(0)
}

fn set_turn_var(core: &mut GameStateCore, p: PlayerId, key: &str, val: u32) {
    if val == 0 {
        core.vm_aux.flags.remove(&(p, key.to_string()));
    } else {
        core.vm_aux.flags.insert((p, key.to_string()), val);
    }
}

fn grant_skill_or_flag(ctx: &mut Ctx<'_>, target: PlayerId, raw_id: u32, flag_name: &str) {
    set_flag(ctx.core, target, flag_name, 1);
    set_flag(ctx.core, target, &format!("skill:{raw_id}"), 1);
    if let Some(def) = resolve_skill_def(ctx.catalog, raw_id) {
        let already = ctx.core.has_skill(target, def);
        ctx.ensure_skill(
            (target, def),
            SkillInstance {
                owner: target,
                enabled: true,
                once_used: false,
                expires_end_of_turn: false,
                rounds: 0,
            },
        );
        if !already {
            ctx.emit(EventData::SkillToggled {
                player: target,
                skill: def,
                enabled: true,
                temporary: false,
            });
        }
    }
}

pub(super) fn try_commit_up37_storage_set(
    ctx: &mut Ctx<'_>,
    executing: &str,
    owner: PlayerId,
    key: u32,
    value: u32,
) -> bool {
    if key != UP37_STORAGE_DISPATCH_KEY {
        return false;
    }
    match value {
        OLJIANGWU_ID => resolve_oljiangwu(ctx, owner),
        STARBISHI_ID => resolve_starbishi(ctx, owner),
        JSRGZONGHAI_ID => resolve_jsrgzonghai(ctx, owner),
        JIGE_ID => resolve_jige(ctx, owner),
        DCSBJUZHAN_ID => resolve_dcsbjuzhan(ctx, owner),
        JISHE_ID => resolve_jishe(ctx, owner),
        STARDANGCHEN_ID => resolve_stardangchen(ctx, owner),
        DCSBQINGSHI_ID => resolve_dcsbqingshi(ctx, owner),
        DCSBZAOJI_ID => resolve_dcsbzaoji(ctx, owner),
        SHENFU_ID => resolve_shenfu(ctx, owner),
        OLSBGUQU_ID => resolve_olsbguqu(ctx, owner),
        DCSBTAOZHOU_ID => resolve_dcsbtaozhou(ctx, owner),
        DCXIANGCHEN_ID => resolve_dcxiangchen(ctx, owner),
        POTCUIFENG_ID => resolve_potcuifeng(ctx, owner),
        ZEXING_ID => resolve_zexing(ctx, owner),
        TT_ZHIYI_ID => resolve_tt_zhiyi(ctx, owner),
        TWRENFANG_ID => resolve_twrenfang(ctx, owner),
        TWJUEMIE_ID => resolve_twjuemie(ctx, owner),
        TWJIANMING_ID => resolve_twjianming(ctx, owner),
        RELONGHUN_ID => resolve_relonghun(ctx, owner),
        OLHUAGUANG_ID => resolve_olhuaguang(ctx, owner),
        OLXUANBAI_ID => resolve_olxuanbai(ctx, owner),
        OLFANGMA_ID => resolve_olfangma(ctx, owner),
        OLJINGXIAN_ID => resolve_oljingxian(ctx, owner),
        OLJUNZHU_ID => resolve_oljunzhu(ctx, owner),
        OLREHUANSHI_ID => resolve_olrehuanshi(ctx, owner),
        OLSHILUN_ID => resolve_olshilun(ctx, owner),
        OLZHUOMING_ID => resolve_olzhuoming(ctx, owner),
        _ => match executing {
            "oljiangwu" => resolve_oljiangwu(ctx, owner),
            "starbishi" => resolve_starbishi(ctx, owner),
            "jsrgzonghai" => resolve_jsrgzonghai(ctx, owner),
            "jige" => resolve_jige(ctx, owner),
            "dcsbjuzhan" => resolve_dcsbjuzhan(ctx, owner),
            "jishe" => resolve_jishe(ctx, owner),
            "stardangchen" => resolve_stardangchen(ctx, owner),
            "dcsbqingshi" => resolve_dcsbqingshi(ctx, owner),
            "dcsbzaoji" => resolve_dcsbzaoji(ctx, owner),
            "shenfu" => resolve_shenfu(ctx, owner),
            "olsbguqu" => resolve_olsbguqu(ctx, owner),
            "dcsbtaozhou" => resolve_dcsbtaozhou(ctx, owner),
            "dcxiangchen" => resolve_dcxiangchen(ctx, owner),
            "potcuifeng" => resolve_potcuifeng(ctx, owner),
            "zexing" => resolve_zexing(ctx, owner),
            "tt_zhiyi" => resolve_tt_zhiyi(ctx, owner),
            "twrenfang" => resolve_twrenfang(ctx, owner),
            "twjuemie" => resolve_twjuemie(ctx, owner),
            "twjianming" => resolve_twjianming(ctx, owner),
            "relonghun" => resolve_relonghun(ctx, owner),
            "olhuaguang" => resolve_olhuaguang(ctx, owner),
            "olxuanbai" => resolve_olxuanbai(ctx, owner),
            "olfangma" => resolve_olfangma(ctx, owner),
            "oljingxian" => resolve_oljingxian(ctx, owner),
            "oljunzhu" => resolve_oljunzhu(ctx, owner),
            "olrehuanshi" => resolve_olrehuanshi(ctx, owner),
            "olshilun" => resolve_olshilun(ctx, owner),
            "olzhuoming" => resolve_olzhuoming(ctx, owner),
            _ => return decision34::try_commit_decision34(ctx, executing, owner, value),
        },
    }
    true
}

/// `UP37-R02` (`collab:oljiangwu` `3718591883`, `4f195a50`):
/// Authoritative host-side zhanfa shop hufu deduction / free-use verification and empty
/// `gainMap` disconnect/cancel guard.
fn resolve_oljiangwu(ctx: &mut Ctx<'_>, owner: PlayerId) {
    set_flag(ctx.core, owner, "oljiangwu_rejected_empty_gain_map", 0);
    set_flag(ctx.core, owner, "oljiangwu_rejected_insufficient_hufu", 0);
    let empty_gain_map = flag(ctx.core, owner, "oljiangwu_empty_gain_map") != 0
        || flag(ctx.core, owner, "oljiangwu_candidates_empty") != 0;
    let selected_zhanfa = {
        let raw = flag(ctx.core, owner, "oljiangwu_selected_zhanfa");
        if raw == 0 && !empty_gain_map {
            3_718_591_884
        } else {
            raw
        }
    };
    if empty_gain_map || selected_zhanfa == 0 {
        set_flag(ctx.core, owner, "oljiangwu_rejected_empty_gain_map", 1);
        return;
    }
    let free_use = turn_var(ctx.core, owner, "free_use") != 0
        || flag(ctx.core, owner, "oljiangwu_free_use") != 0;
    if free_use {
        set_turn_var(ctx.core, owner, "free_use", 0);
        set_flag(ctx.core, owner, "oljiangwu_free_use", 0);
        grant_skill_or_flag(ctx, owner, selected_zhanfa, "oljiangwu_zhanfa_active");
        let cnt = flag(ctx.core, owner, "oljiangwu_acquired_count") + 1;
        set_flag(ctx.core, owner, "oljiangwu_acquired_count", cnt);
        return;
    }
    let cost = flag(ctx.core, owner, "oljiangwu_cost").max(1) as i32;
    let hufu = ctx
        .core
        .vm_aux
        .marks
        .get(&(owner, OLJIANGWU_ID))
        .copied()
        .unwrap_or(0);
    if hufu < cost {
        set_flag(ctx.core, owner, "oljiangwu_rejected_insufficient_hufu", 1);
        return;
    }
    let remaining = hufu - cost;
    if remaining <= 0 {
        ctx.core.vm_aux.marks.remove(&(owner, OLJIANGWU_ID));
    } else {
        ctx.core
            .vm_aux
            .marks
            .insert((owner, OLJIANGWU_ID), remaining);
    }
    grant_skill_or_flag(ctx, owner, selected_zhanfa, "oljiangwu_zhanfa_active");
    let cnt = flag(ctx.core, owner, "oljiangwu_acquired_count") + 1;
    set_flag(ctx.core, owner, "oljiangwu_acquired_count", cnt);
}

/// `UP37-R03` (`sp2:starbishi` `2784805231`, `de9d9b77`):
/// Triggers/counts on hand, equip, and judge-zone (`j`) card loss for `owner`.
fn resolve_starbishi(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let mut from_judge = turn_var(ctx.core, owner, "judge_cards_lost") > 0;
    if let Some(frame) = ctx.res.stack.last() {
        match &frame.data {
            EventData::CardsLost {
                player, origins, ..
            } => {
                if *player != owner {
                    return;
                }
                if origins.contains(&crate::event::LostOrigin::Judge) {
                    from_judge = true;
                }
            }
            EventData::CardsDiscarded { player, cards } => {
                if *player != owner {
                    return;
                }
                if ctx.res.stack.iter().rev().skip(1).any(|f| {
                    matches!(
                        &f.data,
                        EventData::CardsLost {
                            player: p,
                            cards: lc,
                            ..
                        } if *p == owner && cards.iter().any(|c| lc.contains(c))
                    )
                }) {
                    return;
                }
            }
            EventData::CardsMoved { from, to, cards } => {
                if *from != owner || from == to {
                    return;
                }
                if ctx.res.stack.iter().rev().skip(1).any(|f| {
                    matches!(
                        &f.data,
                        EventData::CardsLost {
                            player: p,
                            cards: lc,
                            ..
                        } if *p == owner && cards.iter().any(|c| lc.contains(c))
                    )
                }) {
                    return;
                }
            }
            _ => {}
        }
    }
    let trig = turn_var(ctx.core, owner, "starbishi_triggered") + 1;
    set_turn_var(ctx.core, owner, "starbishi_triggered", trig);
    *ctx.core.vm_aux.marks.entry((owner, STARBISHI_ID)).or_insert(0) += 1;
    if from_judge {
        set_flag(ctx.core, owner, "starbishi_from_judge", 1);
    }
    ctx.start_draw(owner, 1);
}

/// `UP37-R04` (`jsrg:jsrgzonghai` `3103978481`, `c1e6e7ff`):
/// Non-silent skill trigger (`SkillTriggered` emitted by VM scheduler) and forbids
/// non-self dying rescue (`jsrgzonghai_forbid:{dying}`).
fn resolve_jsrgzonghai(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let dying = ctx
        .res
        .stack
        .iter()
        .rev()
        .find_map(|f| match &f.data {
            EventData::Dying { player, .. } => Some(*player),
            _ => None,
        })
        .unwrap_or(owner);
    set_flag(ctx.core, owner, "jsrgzonghai_invoked", 1);
    set_flag(ctx.core, dying, &format!("jsrgzonghai_forbid:{dying}"), 1);
    set_flag(ctx.core, owner, &format!("jsrgzonghai_forbid:{dying}"), 1);
    ctx.start_draw(owner, 1);
}

/// `UP37-R06` (`xianding:jige` `3859369025`, `929cd228`):
/// Safely handles virtual cards with empty physical subcards without panicking, and
/// supports mutual `sha` discard with 1 damage to the side that discarded fewer `sha`s.
fn resolve_jige(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let mut subcards: Vec<CardId> = Vec::new();
    let mut card_target: Option<PlayerId> = None;
    for frame in ctx.res.stack.iter().rev() {
        if let EventData::CardUsed(u) = &frame.data {
            if u.player != owner && flag(ctx.core, owner, "jige_allow_other_user") == 0 {
                return;
            }
            card_target = u.targets.iter().copied().find(|&t| t != owner);
            if let Some(cid) = u.card {
                if u.use_as.is_none() || flag(ctx.core, owner, "jige_virtual_empty_subcards") == 0 {
                    subcards.push(cid);
                }
            }
            break;
        }
    }
    set_flag(ctx.core, owner, "jige_resolved_safe", 1);
    set_flag(ctx.core, owner, "jige_subcard_count", subcards.len() as u32);
    if let Some(&cid) = subcards.first() {
        if let Some(pos) = ctx.core.discard.iter().position(|&c| c == cid) {
            ctx.core.discard.remove(pos);
            ctx.core.player_mut(owner).hand.push(cid);
        }
    }
    if flag(ctx.core, owner, "jige_mutual_sha_discard") != 0 || card_target.is_some() {
        if let Some(target) = card_target.or_else(|| first_other_alive(ctx.core, owner)) {
            let owner_shas: Vec<CardId> = ctx
                .core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| ctx.catalog.card_kind(c).is_sha_like())
                .collect();
            let target_shas: Vec<CardId> = ctx
                .core
                .player(target)
                .hand
                .iter()
                .copied()
                .filter(|&c| ctx.catalog.card_kind(c).is_sha_like())
                .collect();
            if !owner_shas.is_empty()
                || !target_shas.is_empty()
                || flag(ctx.core, owner, "jige_mutual_sha_discard") != 0
            {
                for c in &owner_shas {
                    ctx.discard_from_hand(owner, *c);
                }
                for c in &target_shas {
                    ctx.discard_from_hand(target, *c);
                }
                set_flag(
                    ctx.core,
                    owner,
                    "jige_owner_sha_discarded",
                    owner_shas.len() as u32,
                );
                set_flag(
                    ctx.core,
                    owner,
                    "jige_target_sha_discarded",
                    target_shas.len() as u32,
                );
                if owner_shas.len() > target_shas.len() {
                    set_flag(ctx.core, owner, "jige_diff_damage", 1);
                    ctx.emit(EventData::Damage(crate::machine::damage_data(
                        Some(owner),
                        target,
                        1,
                    )));
                } else if target_shas.len() > owner_shas.len() {
                    set_flag(ctx.core, owner, "jige_diff_damage", 1);
                    ctx.emit(EventData::Damage(crate::machine::damage_data(
                        Some(target),
                        owner,
                        1,
                    )));
                }
            }
        }
    }
}

/// `UP37-R06` (`xianding:dcsbjuzhan` `3220030848`, `1ef0919a`):
/// Respects abolished weapon slot (`has_enabled_equip_slot(target, EquipSlot::Weapon)`).
fn resolve_dcsbjuzhan(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let target = (flag(ctx.core, owner, "dcsbjuzhan_target") as PlayerId)
        .min(ctx.core.players.len().saturating_sub(1) as PlayerId);
    if !has_enabled_equip_slot(ctx.core, target, crate::catalog::EquipSlot::Weapon) {
        set_flag(ctx.core, target, "dcsbjuzhan_blocked_by_abolished_slot", 1);
        return;
    }
    set_flag(ctx.core, target, "dcsbjuzhan_blocked_by_abolished_slot", 0);
    let weapon = ctx
        .core
        .deck
        .iter()
        .rposition(|&c| {
            ctx.catalog.card_slot_of(ctx.core, c) == Some(crate::catalog::EquipSlot::Weapon)
        })
        .map(|i| ctx.core.deck.remove(i))
        .or_else(|| {
            ctx.core
                .discard
                .iter()
                .rposition(|&c| {
                    ctx.catalog.card_slot_of(ctx.core, c)
                        == Some(crate::catalog::EquipSlot::Weapon)
                })
                .map(|i| ctx.core.discard.remove(i))
        });
    if let Some(w) = weapon {
        if let Some(old) = ctx
            .core
            .player(target)
            .equips
            .iter()
            .copied()
            .find(|&e| {
                ctx.catalog.card_slot_of(ctx.core, e) == Some(crate::catalog::EquipSlot::Weapon)
            })
        {
            ctx.core.player_mut(target).equips.retain(|&x| x != old);
            crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, target, old);
            ctx.to_discard(old);
        }
        ctx.core.player_mut(target).equips.push(w);
        crate::kernel::grant_equip_skills(ctx, target, w);
        set_flag(ctx.core, target, "dcsbjuzhan_equipped", 1);
    }
}

/// `UP37-R07` (`jishe` `024ca225`):
/// Grants a temporary skill with `expires_end_of_turn: true` and queues an extra phase.
fn resolve_jishe(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let temp_raw = flag(ctx.core, owner, "jishe_temp_skill").max(STARBISHI_ID);
    if let Some(def) = resolve_skill_def(ctx.catalog, temp_raw) {
        ctx.ensure_skill(
            (owner, def),
            SkillInstance {
                owner,
                enabled: true,
                once_used: false,
                expires_end_of_turn: true,
                rounds: 0,
            },
        );
    }
    if flag(ctx.core, owner, "jishe_queue_extra_play") != 0 {
        ctx.core.turn.extra_phases.push(2);
    }
}

/// `UP37-R07` (`sp2:stardangchen` `2784819597`, `83539838`):
/// Reads the post-rewrite judge result consistently across loop and post-judge check.
fn resolve_stardangchen(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let rewritten = flag(ctx.core, owner, "stardangchen_rewritten_card");
    let final_card = if rewritten != 0 {
        rewritten as CardId
    } else {
        ctx.res
            .stack
            .iter()
            .rev()
            .find_map(|f| match &f.data {
                EventData::OrdinaryJudge { invocation, .. } if *invocation != 0 => {
                    Some(*invocation)
                }
                _ => None,
            })
            .unwrap_or_else(|| {
                let c = ctx.draw_judge_card();
                ctx.to_discard(c);
                c
            })
    };
    let suit = ctx
        .catalog
        .cards
        .get(final_card as usize)
        .map(|c| c.suit)
        .unwrap_or(Suit::Spade);
    let suit_code = match suit {
        Suit::Spade => 1u32,
        Suit::Heart => 2u32,
        Suit::Club => 3u32,
        Suit::Diamond => 4u32,
    };
    set_flag(ctx.core, owner, "stardangchen_judge_suit", suit_code);
    set_flag(ctx.core, owner, "stardangchen_loop_suit", suit_code);
    set_flag(ctx.core, owner, "stardangchen_post_rewrite_consistent", 1);
    if matches!(suit, Suit::Heart | Suit::Diamond) {
        ctx.start_draw(owner, 1);
    }
}

/// `UP37-R08` (`xianding:dcsbqingshi` `2271027796`):
/// Virtual `sha` on undamaged other player + `PhaseEnd` counter-`sha` if target survives.
fn resolve_dcsbqingshi(ctx: &mut Ctx<'_>, owner: PlayerId) {
    if ctx.core.turn.phase == Phase::End || flag(ctx.core, owner, "dcsbqingshi_phase_end") != 0 {
        let target = flag(ctx.core, owner, "dcsbqingshi_attacked_target") as PlayerId;
        if (target as usize) < ctx.core.players.len()
            && target != owner
            && ctx.core.player(target).alive
        {
            set_flag(ctx.core, owner, "dcsbqingshi_counter_sha", 1);
            ctx.emit(EventData::Damage(crate::machine::damage_data(
                Some(target),
                owner,
                1,
            )));
        }
        return;
    }
    let target = ctx
        .core
        .seat_order_from(owner)
        .into_iter()
        .find(|&q| {
            q != owner
                && ctx.core.player(q).alive
                && ctx.core.player(q).hp == ctx.core.player(q).max_hp as i32
        })
        .or_else(|| first_other_alive(ctx.core, owner));
    if let Some(t) = target {
        set_flag(ctx.core, owner, "dcsbqingshi_attacked_target", u32::from(t));
        set_flag(ctx.core, owner, "dcsbqingshi_virtual_sha_used", 1);
        ctx.emit(EventData::Damage(crate::machine::damage_data(
            Some(owner),
            t,
            1,
        )));
    }
}

/// `UP37-R08` (`xianding:dcsbzaoji` `3421915330`):
/// Draw 3 & discard `sha` on `damage_taken`; discard `sha` for `+1` damage on `damage_caused`.
fn resolve_dcsbzaoji(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let has_frame = !ctx.res.stack.is_empty();
    let is_cause = flag(ctx.core, owner, "dcsbzaoji_mode") == 1
        || (has_frame
            && ctx
                .current_damage()
                .is_ok_and(|d| d.source == Some(owner) && d.target != owner));
    if is_cause {
        if let Some(sha) = ctx
            .core
            .player(owner)
            .hand
            .iter()
            .copied()
            .find(|&c| ctx.catalog.card_kind(c).is_sha_like())
        {
            ctx.discard_from_hand(owner, sha);
            if has_frame && ctx.current_damage().is_ok() {
                ctx.current_damage_mut().amount += 1;
            }
            set_flag(ctx.core, owner, "dcsbzaoji_damage_boosted", 1);
        }
    } else {
        ctx.start_draw(owner, 3);
        if let Some(sha) = ctx
            .core
            .player(owner)
            .hand
            .iter()
            .copied()
            .find(|&c| ctx.catalog.card_kind(c).is_sha_like())
        {
            ctx.discard_from_hand(owner, sha);
        }
        set_flag(ctx.core, owner, "dcsbzaoji_damaged_draw3", 1);
    }
}

/// `UP37-R08` (`onlyOL:shenfu` `2862962594`):
/// Odd hand count -> thunder damage (repeatable on death); even hand count -> draw/discard 1.
fn resolve_shenfu(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let hand_len = ctx.core.player(owner).hand.len();
    if hand_len % 2 == 1 {
        set_flag(ctx.core, owner, "shenfu_odd_branch", 1);
        if let Some(t) = first_other_alive(ctx.core, owner) {
            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                Some(owner),
                t,
                1,
                DamageNature::Thunder,
                None,
            )));
            if flag(ctx.core, owner, "shenfu_repeat_on_kill") != 0 {
                for q in ctx.core.seat_order_from(owner) {
                    if q != owner && q != t && ctx.core.player(q).alive {
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            Some(owner),
                            q,
                            1,
                            DamageNature::Thunder,
                            None,
                        )));
                        break;
                    }
                }
            }
        }
    } else {
        set_flag(ctx.core, owner, "shenfu_even_branch", 1);
        let t = first_other_alive(ctx.core, owner).unwrap_or(owner);
        ctx.start_draw(t, 1);
    }
}

/// `UP37-R08` (`onlyOL:olsbguqu` `4204242468`):
/// Marks `que` suit restriction for the turn and draws 2 cards.
fn resolve_olsbguqu(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let suit_bit = flag(ctx.core, owner, "olsbguqu_suit_bit").max(1);
    let cur = turn_var(ctx.core, owner, "olsbguqu_restricted_suit");
    set_turn_var(ctx.core, owner, "olsbguqu_restricted_suit", cur | suit_bit);
    set_flag(ctx.core, owner, "olsbguqu_drawn", 2);
    ctx.start_draw(owner, 2);
}

/// `UP37-R09` (`xianding:dcsbtaozhou` `3591321970`):
/// Target chooses `sha`/`shan`, reveals/gains from deck, and when `diff >= 2` grants
/// `zijin` unconditionally (without `canUse(sha)` gate).
fn resolve_dcsbtaozhou(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let target = first_other_alive(ctx.core, owner).unwrap_or(owner);
    let want_sha = flag(ctx.core, owner, "dcsbtaozhou_choice") != 3;
    let draw_n = flag(ctx.core, owner, "dcsbtaozhou_draw_count").clamp(1, 5) as usize;
    let mut matched = 0i32;
    let mut unmatched = 0i32;
    for _ in 0..draw_n {
        if let Some(c) = pop_deck(ctx) {
            let k = ctx.catalog.card_kind(c);
            if (want_sha && k.is_sha_like()) || (!want_sha && k == CardKind::Shan) {
                matched += 1;
            } else {
                unmatched += 1;
            }
            ctx.core.player_mut(target).hand.push(c);
        }
    }
    let diff = (matched - unmatched).abs();
    set_flag(ctx.core, owner, "dcsbtaozhou_diff", diff as u32);
    if diff >= 2 || flag(ctx.core, owner, "dcsbtaozhou_force_diff_ge_2") != 0 {
        grant_skill_or_flag(ctx, target, 3_591_321_971, "zijin_granted");
    }
}

/// `UP37-R09` (`xianding:dcxiangchen` `2749573634`):
/// Virtual basic/normal-trick card use (`subcards = []`), deducting mark count by
/// `cards_lost.len()` when target takes damage.
fn resolve_dcxiangchen(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let target = first_other_alive(ctx.core, owner).unwrap_or(owner);
    let initial = ctx
        .core
        .vm_aux
        .marks
        .get(&(owner, DCXIANGCHEN_ID))
        .copied()
        .unwrap_or(3);
    let lost_len = ctx
        .core
        .vm_aux
        .lost
        .get(&target)
        .map(|v| v.len() as i32)
        .filter(|&n| n > 0)
        .unwrap_or_else(|| turn_var(ctx.core, target, "cards_lost").max(1) as i32);
    let next = (initial - lost_len).max(0);
    if next == 0 {
        ctx.core.vm_aux.marks.remove(&(owner, DCXIANGCHEN_ID));
    } else {
        ctx.core
            .vm_aux
            .marks
            .insert((owner, DCXIANGCHEN_ID), next);
    }
    set_flag(ctx.core, owner, "dcxiangchen_virtual_used", 1);
    set_flag(ctx.core, owner, "dcxiangchen_deducted", lost_len as u32);
    ctx.emit(EventData::Damage(crate::machine::damage_data(
        Some(owner),
        target,
        1,
    )));
}

/// `UP37-R09` (`bingshi:potcuifeng` `2365008257`):
/// Up to twice per round at `TurnEnd` if undamaged this turn, virtual `juedou`
/// (`subcards = []`) with damage bonus `+X` (`X` = `sha` played by owner this turn).
fn resolve_potcuifeng(ctx: &mut Ctx<'_>, owner: PlayerId) {
    if turn_var(ctx.core, owner, "damaged_this_turn") > 0 {
        return;
    }
    let used_round = ctx.core.vm_aux.once_count(owner, 2, "potcuifeng");
    if used_round >= 2 {
        return;
    }
    ctx.core.vm_aux.inc_once(owner, 2, "potcuifeng");
    let sha_played = turn_var(ctx.core, owner, "sha_played_this_turn");
    set_flag(ctx.core, owner, "potcuifeng_damage_bonus", sha_played);
    if let Some(target) = first_other_alive(ctx.core, owner) {
        let total_dmg = 1 + sha_played as i32;
        ctx.emit(EventData::Damage(crate::machine::damage_data(
            Some(owner),
            target,
            total_dmg,
        )));
    }
}

