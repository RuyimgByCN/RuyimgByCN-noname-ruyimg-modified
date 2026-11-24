//! Phase 3 P3-C1a `L11` 跨包 NPC Part A 出牌阶段主动技资格校验（`activate.rs`）。

use super::helpers::*;
use super::l11_common::*;
use super::*;
use crate::catalog::{CardKind, EquipSlot, QueryCtx, Suit};
use crate::event::DamageNature;
use crate::ids::PlayerId;

pub(in crate::content::host) fn activate_l11_npc_a(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<Box<dyn crate::catalog::SkillMachine>> {
    if !is_unblocked_npc_a(ctx.state, ctx.catalog, owner) {
        return None;
    }
    let st = ctx.state;
    let p = st.player(owner);
    let he = he_cards(st, owner);
    let others = alive_others(st, owner);
    let ok = match skill.name.as_str() {
        "baiyi" => {
            flag(st, owner, "l11_baiyi_used") == 0
                && p.hp < p.max_hp as i32
                && st.alive_players().count() > 2
        }
        "duwu" => {
            flag(st, owner, "l11_duwu_banned_turn") == 0
                && others.iter().any(|&t| {
                    st.in_attack_range(ctx.catalog, owner, t)
                        && st.player(t).hp > 0
                        && he.len() >= st.player(t).hp as usize
                })
        }
        "fenxun" => !he.is_empty() && !others.is_empty(),
        "guanzong" => others.len() >= 2,
        "gushe" => {
            !p.hand.is_empty()
                && flag(st, owner, "l11_gushe_marks") < 7
                && others.iter().any(|&t| !st.player(t).hand.is_empty())
        }
        "jinglve" => {
            flag(st, owner, "l11_jinglve_active_target") == 0
                && others.iter().any(|&t| !st.player(t).hand.is_empty())
        }
        "jujian" => !he.is_empty() && !others.is_empty(),
        "lianzhu" => !he.is_empty() && !others.is_empty(),
        "lihun" => {
            flag(st, owner, "l11_lihun_used_play") == 0
                && !he.is_empty()
                && others.iter().any(|&t| {
                    st.matches_gender(ctx.catalog, t, crate::content::standard::Gender::Male)
                })
        }
        "mingjian" => !p.hand.is_empty() && !others.is_empty(),
        "reguose" => {
            he.iter()
                .any(|&c| card_suit(ctx.catalog, st, owner, c) == Suit::Diamond)
                && st.alive_players().any(|t| {
                    st.player(t)
                        .judge
                        .iter()
                        .any(|&j| ctx.catalog.resolved_card_kind(st, j) == CardKind::Le)
                        || (t != owner
                            && !ctx.catalog.blocks_delayed_trick(st, t)
                            && !ctx
                                .catalog
                                .skill_by_name(crate::vocabulary::names::QIAN_XUN)
                                .is_some_and(|d| st.has_skill(t, d))
                            && !ctx
                                .catalog
                                .skill_by_name("qianxun")
                                .is_some_and(|d| st.has_skill(t, d))
                            && ctx.catalog.ow_target_enabled(
                                st,
                                owner,
                                t,
                                None,
                                CardKind::Le,
                                DamageNature::Normal,
                            ))
                })
        }
        "mizhao" => {
            !p.hand.is_empty()
                && others.iter().any(|&t| !st.player(t).hand.is_empty())
                && others.len() >= 2
        }
        "mouzhu" => others.iter().any(|&t| !st.player(t).hand.is_empty()),
        "new_reqingnang" => {
            flag(st, owner, "l11_new_reqingnang_ban") == 0
                && !p.hand.is_empty()
                && st.alive_players().any(|t| {
                    st.player(t).hp < st.player(t).max_hp as i32
                        && flag(st, t, "l11_new_reqingnang_t") == 0
                })
        }
        "new_yijue" => !he.is_empty() && others.iter().any(|&t| !st.player(t).hand.is_empty()),
        "nzry_chenglve" => true,
        "nzry_feijun" => {
            !he.is_empty()
                && others.iter().any(|&t| {
                    st.player(t).hand.len() >= p.hand.len()
                        || st.player(t).equips.len() >= p.equips.len()
                })
        }
        "quji" => {
            let x = (p.max_hp as i32 - p.hp).max(0) as usize;
            x > 0 && he.len() >= x
        }
        "refenxun" => flag(st, owner, "l11_refenxun_used_play") == 0 && !others.is_empty(),
        "reganlu" => {
            let lost = (p.max_hp as i32 - p.hp).max(0) as usize;
            let all: Vec<PlayerId> = st.alive_players().collect();
            (0..all.len()).any(|i| {
                ((i + 1)..all.len()).any(|j| {
                    let (a, b) = (all[i], all[j]);
                    let ea = st.player(a).equips.len();
                    let eb = st.player(b).equips.len();
                    (ea > 0 || eb > 0) && (ea.abs_diff(eb) <= lost || a == owner || b == owner)
                })
            })
        }
        "rejunxing" => !p.hand.is_empty() && !others.is_empty(),
        "reluanji" => {
            let used = flag(st, owner, "l11_reluanji_suits_turn");
            p.hand
                .iter()
                .filter(|&&c| (used & suit_bit(card_suit(ctx.catalog, st, owner, c))) == 0)
                .count()
                >= 2
        }
        "rewurong" => !p.hand.is_empty() && others.iter().any(|&t| !st.player(t).hand.is_empty()),
        "rezhijian" => p.hand.iter().any(|&c| {
            ctx.catalog.card_slot_of(st, c).is_some_and(|sl| {
                others.iter().any(|&t| {
                    ctx.catalog.is_equip_slot_enabled(st, t, sl)
                        && !st
                            .player(t)
                            .equips
                            .iter()
                            .any(|&e| ctx.catalog.card_slot_of(st, e) == Some(sl))
                })
            })
        }),
        "rongbei" => {
            flag(st, owner, "l11_rongbei_used") == 0
                && st.alive_players().any(|t| {
                    [
                        EquipSlot::Weapon,
                        EquipSlot::Armor,
                        EquipSlot::HorsePlus,
                        EquipSlot::HorseMinus,
                        EquipSlot::Treasure,
                    ]
                    .into_iter()
                    .any(|sl| {
                        ctx.catalog.is_equip_slot_enabled(st, t, sl)
                            && !st
                                .player(t)
                                .equips
                                .iter()
                                .any(|&e| ctx.catalog.card_slot_of(st, e) == Some(sl))
                    })
                })
        }
        "kyou_zhidian" => {
            let (lim, unlim) = crate::rvm_bridge::modifiers::eval_sha_use_limit(
                st,
                ctx.catalog,
                owner,
                None,
                CardKind::Sha,
                DamageNature::Normal,
                1,
            );
            let used_extra = flag(st, owner, "vm_sha_extra_used");
            let can_use_sha = !st.turn.sha_used_play_phase || unlim || (1 + used_extra) < lim;
            can_use_sha
                && p.hand
                    .iter()
                    .any(|&c| !ctx.catalog.card_kind(c).is_basic_kind() && !ctx.catalog.card_kind(c).is_equip())
                && others.iter().any(|&t| {
                    ctx.catalog
                        .ow_target_enabled(st, owner, t, None, CardKind::Sha, DamageNature::Normal)
                })
        }
        "dclbjiuxian" => p.hand.iter().any(|&c| {
            crate::rvm_bridge::modifiers::d34_l11_mods_npc_a::is_multi_target_trick_for_jiuxian(
                ctx.catalog.card_kind(c),
                ctx.catalog.cards[c as usize].name,
            )
        }),
        "olhuoji" => {
            he.iter().any(|&c| is_red_card(ctx.catalog, st, owner, c))
                && st.alive_players().any(|t| !st.player(t).hand.is_empty())
        }
        "reshuangxiong" => {
            let col = flag(st, owner, "shuangxiong_color");
            col > 0
                && p.hand.iter().any(|&c| {
                    let red = is_red_card(ctx.catalog, st, owner, c);
                    (col == 1 && !red) || (col == 2 && red)
                })
                && !others.is_empty()
        }
        _ => false,
    };
    ok.then(|| active_skill_machine(&skill.name, HostBehavior::L11CrossNpcA, owner))
        .flatten()
}
