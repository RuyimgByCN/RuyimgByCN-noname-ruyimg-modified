//! Batch U05-6a (`shahai-hearth` Part 1): Batch A 尾段 (`nsaiqi` / `nsbeiming` /
//! `hsnitai` / `hspuzhao` / `hstianqi*`)。

use super::u05_hearth::{
    alive_others, base_card_name, discard_card_from_zone, equip_card_to_player, exact_base_name,
    flag, gain_card_to_hand, has_any_skill, has_skill_hearth, in_attack_range_hearth,
    is_hearth_friend, refund_once_used, remove_skill, rng_pick, set_flag, spawn_card_by_base,
};
use super::u05_hearth_asks::{
    set_active_ask, wait_hs, ASK_HS_HSPUZHAO_CARD, ASK_HS_HSTIANQI_CARD, ASK_HS_NSBEIMING_ORDER,
};
use super::*;
use crate::catalog::{CardKind, Catalog};
use crate::choice::ChoiceOption;
use crate::event::EventData;
use crate::ids::{CardId, PlayerId, SkillDefId};
use crate::machine::{Ctx, Flow, LogEvent};
use crate::state::GameStateCore;

pub(super) fn step_a_tail(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.def_key.as_str() {
        "nsaiqi" => step_nsaiqi_nsbeiming(m, ctx),
        "hsnitai" => step_hsnitai(m, ctx),
        "hspuzhao" => step_hspuzhao(m, ctx),
        "hstianqi" => step_hstianqi(m, ctx),
        "hstianqi_dalian" => {
            let dmg = ctx
                .res
                .stack
                .iter()
                .rev()
                .find_map(|f| match &f.data {
                    EventData::Damage(d) if d.source == Some(m.owner) => Some(d.amount.max(1) as u32),
                    _ => None,
                })
                .unwrap_or(1);
            ctx.recover(m.owner, dmg, None);
            Flow::Done
        }
        "hstianqi_shali" => {
            let amt = ctx
                .res
                .stack
                .iter()
                .rev()
                .find_map(|f| match &f.data {
                    EventData::Recover { player, amount, .. } if *player == m.owner => {
                        Some(*amount as i32)
                    }
                    _ => None,
                })
                .unwrap_or(1);
            ctx.change_hujia(m.owner, amt);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_nsaiqi_nsbeiming(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => {
            if let Some(&top) = ctx.core.deck.last() {
                if flag(ctx.core, 0, &format!("hs_nsaiqi_vanish:{top}")) > 0 {
                    return Flow::Done;
                }
            }
            for _ in 0..3 {
                if let Some(cid) = ctx.core.deck.pop() {
                    set_flag(ctx.core, 0, &format!("hs_nsaiqi_vanish:{cid}"), 1);
                    ctx.core
                        .vm_aux
                        .piles
                        .entry((m.owner, "hs_nsaiqi".to_string()))
                        .or_default()
                        .push(cid);
                }
            }
            let total = ctx
                .core
                .vm_aux
                .piles
                .get(&(m.owner, "hs_nsaiqi".to_string()))
                .map_or(0, |v| v.len());
            if total >= 9
                && has_skill_hearth(ctx.core, ctx.catalog, m.owner, "nsbeiming")
                && !ctx.catalog.is_tieji_sealed(ctx.core, m.owner)
            {
                ctx.start_draw(m.owner, 1);
                return wait_hs(m, ctx, HostStage::SelectCard, ASK_HS_NSBEIMING_ORDER);
            }
            Flow::Done
        }
        HostStage::SelectCard => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::SelectCard, 0);
            let mut pile = ctx
                .core
                .vm_aux
                .piles
                .remove(&(m.owner, "hs_nsaiqi".to_string()))
                .unwrap_or_default();
            let chosen: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Card(c) => Some(*c),
                    _ => None,
                })
                .collect();
            for &c in chosen.iter().rev() {
                if let Some(pos) = pile.iter().position(|&x| x == c) {
                    pile.remove(pos);
                    ctx.core.deck.push(c);
                }
            }
            for c in pile {
                ctx.core.deck.push(c);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_hsnitai(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let left = flag(ctx.core, m.owner, "hs_nitai_turns_left");
    let active_def = flag(ctx.core, m.owner, "hs_nitai_active_def");
    if active_def > 0 {
        if left > 0 {
            set_flag(ctx.core, m.owner, "hs_nitai_turns_left", left - 1);
        } else {
            let name = ctx.catalog.skill_name((active_def - 1) as u16).to_string();
            remove_skill(ctx, m.owner, &name);
            set_flag(ctx.core, m.owner, "hs_nitai_active_def", 0);
        }
    }
    let Some(token) = spawn_card_by_base(ctx, "hsnitai_card") else {
        return Flow::Done;
    };
    let mut cand_generals: Vec<(u16, Vec<SkillDefId>)> = Vec::new();
    for g in 0..ctx.catalog.general_count() as u16 {
        if let Some((gname, _, sks, _, _)) = ctx.catalog.general_meta(g) {
            if !gname.starts_with("hs_") {
                continue;
            }
            let unheld: Vec<SkillDefId> = sks
                .iter()
                .filter_map(|&sk| {
                    if !has_any_skill(ctx.core, ctx.catalog, m.owner, sk) {
                        ctx.catalog.skill_by_name(sk)
                    } else {
                        None
                    }
                })
                .collect();
            if !unheld.is_empty() {
                cand_generals.push((g, unheld));
            }
        }
    }
    let Some(idx) = (!cand_generals.is_empty())
        .then(|| ctx.core.rng.gen_range(cand_generals.len() as u32) as usize)
    else {
        ctx.core
            .detached_cards
            .push((token, crate::event::DetachedKind::Unclaimed));
        return Flow::Done;
    };
    let (_, skills) = &cand_generals[idx];
    let cnt = skills.len().min(3);
    set_flag(ctx.core, 0, &format!("hs_nitai_cnt:{token}"), cnt as u32);
    for (i, &def) in skills.iter().take(cnt).enumerate() {
        set_flag(ctx.core, 0, &format!("hs_nitai_sk_{i}:{token}"), u32::from(def) + 1);
    }
    gain_card_to_hand(ctx, m.owner, token);
    Flow::Done
}

fn step_hspuzhao(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => wait_hs(m, ctx, HostStage::SelectCard, ASK_HS_HSPUZHAO_CARD),
        HostStage::SelectCard => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::SelectCard, 0);
            let Some(ChoiceOption::Card(picked)) = ctx.answer().first().copied() else {
                refund_once_used(ctx, m.owner, "hspuzhao");
                return Flow::Done;
            };
            discard_card_from_zone(ctx, m.owner, picked);
            ctx.log(LogEvent::CardDiscarded {
                player: m.owner,
                cards: vec![picked],
            });
            let mut friends: Vec<PlayerId> = ctx
                .core
                .seat_order_from(m.owner)
                .into_iter()
                .filter(|&p| p != m.owner && is_hearth_friend(ctx.core, ctx.catalog, m.owner, p))
                .collect();
            if friends.is_empty() {
                ctx.start_draw(m.owner, 2);
            } else {
                friends.push(m.owner);
                while friends.len() > 3 {
                    let drop_i = ctx.core.rng.gen_range(friends.len() as u32) as usize;
                    friends.remove(drop_i);
                }
                friends.sort_unstable();
                for t in friends {
                    ctx.start_draw(t, 1);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_hstianqi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let in_phase_begin = ctx
        .res
        .stack
        .last()
        .is_some_and(|f| matches!(f.data, EventData::Phase { .. }));
    if in_phase_begin {
        let winner = match &ctx.core.mode {
            crate::state::GameMode::Identity { roles } => {
                match roles.get(m.owner as usize) {
                    Some(crate::state::Role::Lord | crate::state::Role::Loyalist) => {
                        crate::state::Winner::LordSide
                    }
                    Some(crate::state::Role::Traitor) => crate::state::Winner::Traitor,
                    _ => crate::state::Winner::Rebels,
                }
            }
            crate::state::GameMode::Kingdoms => ctx
                .core
                .effective_faction(ctx.catalog, m.owner)
                .map(crate::state::Winner::Faction)
                .unwrap_or(crate::state::Winner::Player(m.owner)),
            crate::state::GameMode::Versus { .. } => {
                crate::state::Winner::Versus(ctx.core.versus_team_of(m.owner).unwrap_or(0))
            }
            crate::state::GameMode::Doudizhu { lord } => crate::state::Winner::Doudizhu {
                lord_won: *lord == m.owner,
            },
            crate::state::GameMode::Boss { boss, .. } => crate::state::Winner::Boss {
                boss_won: *boss == m.owner,
            },
            _ => crate::state::Winner::Player(m.owner),
        };
        ctx.core.outcome = Some(crate::state::Outcome { winner });
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => wait_hs(m, ctx, HostStage::SelectCard, ASK_HS_HSTIANQI_CARD),
        HostStage::SelectCard => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::SelectCard, 0);
            let Some(ChoiceOption::Card(picked)) = ctx.answer().first().copied() else {
                refund_once_used(ctx, m.owner, "hstianqi");
                return Flow::Done;
            };
            let from_equip = ctx.core.player(m.owner).equips.contains(&picked);
            let cname = exact_base_name(ctx.catalog.cards[picked as usize].name).to_string();
            discard_card_from_zone(ctx, m.owner, picked);
            ctx.log(LogEvent::CardDiscarded {
                player: m.owner,
                cards: vec![picked],
            });
            if from_equip {
                let bit = match cname.as_str() {
                    "hstianqi_dalian" => 1 << 0,
                    "hstianqi_shali" => 1 << 1,
                    "hstianqi_suolasi" => 1 << 2,
                    "hstianqi_nazigelin" => 1 << 3,
                    _ => 0,
                };
                if bit != 0 {
                    let old_mask = flag(ctx.core, m.owner, "hs_hstianqi_mask");
                    if (old_mask & bit) == 0 {
                        set_flag(ctx.core, m.owner, "hs_hstianqi_mask", old_mask | bit);
                        if let Some(def) = ctx.catalog.skill_by_name("hstianqi") {
                            ctx.core.vm_aux.marks.insert(
                                (m.owner, u32::from(def)),
                                (old_mask | bit).count_ones() as i32,
                            );
                        }
                    }
                }
                ctx.start_draw(m.owner, 2);
            } else {
                let pl = ctx.core.player(m.owner);
                let has_slot = |slot| {
                    pl.equips
                        .iter()
                        .any(|&e| ctx.catalog.card_kind(e).equip_slot() == Some(slot))
                };
                let mut candidates = Vec::new();
                if !has_slot(crate::catalog::EquipSlot::Weapon) {
                    candidates.push("hstianqi_dalian");
                }
                if !has_slot(crate::catalog::EquipSlot::Armor) {
                    candidates.push("hstianqi_shali");
                }
                if !has_slot(crate::catalog::EquipSlot::HorsePlus) {
                    candidates.push("hstianqi_suolasi");
                }
                if !has_slot(crate::catalog::EquipSlot::HorseMinus) {
                    candidates.push("hstianqi_nazigelin");
                }
                if let Some(pick_name) = rng_pick(ctx, &candidates) {
                    if let Some(eq) = spawn_card_by_base(ctx, pick_name) {
                        equip_card_to_player(ctx, m.owner, eq);
                    }
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn effective_hearth_card(
    core: &GameStateCore,
    catalog: &Catalog,
    actor: PlayerId,
    cid: CardId,
) -> (CardId, CardKind) {
    let kind = core
        .declared_kinds
        .get(&cid)
        .copied()
        .unwrap_or_else(|| catalog.card_kind(cid));
    let eff_cid = if core.declared_kinds.contains_key(&cid) {
        super::u05_hearth_hooks::wxuying_effective_src(core, catalog, actor, cid)
    } else {
        cid
    };
    (eff_cid, kind)
}

pub(super) fn hearth_target_choice_max(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    cid: CardId,
    opts_len: usize,
) -> usize {
    let (eff_cid, _) = effective_hearth_card(core, catalog, owner, cid);
    if exact_base_name(catalog.cards[eff_cid as usize].name) == "hsqizhou_tu" {
        opts_len.max(1)
    } else {
        1
    }
}

pub(super) fn collect_player_answers(answer: &[ChoiceOption]) -> Vec<PlayerId> {
    answer
        .iter()
        .filter_map(|o| match o {
            ChoiceOption::Player(p) => Some(*p),
            _ => None,
        })
        .collect()
}

pub(super) fn is_hshuanling_named_trick(exact: &str) -> bool {
    matches!(
        exact,
        "shandianjian"
            | "ht_shandianjian"
            | "huogong"
            | "hsfashu_buwendingyibian"
            | "hsqizhou_huo"
            | "hsqizhou_tu"
            | "hsqingyu_feibiao"
            | "hsqingyu_zhanfang"
            | "hsqingyu_hufu"
            | "hsqingyu_shandian"
            | "hsdusu_xueji"
            | "hsdusu_kuyecao"
            | "hsdusu_huoyanhua"
            | "hsdusu_shinancao"
            | "hsshenqi_nengliangzhiguang"
            | "hsmengjing_feicuiyoulong"
            | "hsmengjing_mengye"
            | "hsmengjing_mengjing"
            | "hsmengjing_huanxiaojiemei"
            | "hszuzhou_nvwudeganguo"
            | "hszuzhou_nvwudepingguo"
            | "hszuzhou_nvwudexuetu"
            | "hszuzhou_guhuo"
    )
}

pub(super) fn is_hshuanling_beneficial(kind: CardKind, exact: &str) -> bool {
    matches!(kind, CardKind::Zhiliaobo | CardKind::Shijieshu)
        || matches!(
            exact,
            "hsmengjing_huanxiaojiemei"
                | "hsqizhou_tu"
                | "hsqingyu_zhanfang"
                | "hsqingyu_hufu"
                | "hsdusu_kuyecao"
                | "hsdusu_shinancao"
                | "hsshenqi_nengliangzhiguang"
                | "hszuzhou_nvwudepingguo"
        )
}

fn judge_card_kind(core: &GameStateCore, catalog: &Catalog, cid: CardId) -> CardKind {
    core.declared_kinds
        .get(&cid)
        .copied()
        .unwrap_or_else(|| catalog.card_kind(cid))
}

pub(super) fn single_target_options(
    core: &GameStateCore,
    catalog: &Catalog,
    actor: PlayerId,
    cid: CardId,
) -> Vec<ChoiceOption> {
    let (eff_cid, kind) = effective_hearth_card(core, catalog, actor, cid);
    let raw = catalog.cards[eff_cid as usize].name;
    let exact = exact_base_name(raw);
    let base = base_card_name(raw);
    let is_huogong = exact == "huogong";
    if exact == "hsfashu_buwendingyibian"
        && flag(core, actor, "hs_buwendingyibian_used_turn") as i32 >= core.player(actor).hp
    {
        return Vec::new();
    }
    if exact == "hsshenqi_nengliangzhiguang" && core.player(actor).face_down {
        return Vec::new();
    }
    core.alive_players()
        .filter(|&t| {
            let need_other = is_huogong
                || matches!(exact, "hsqizhou_tu" | "hszuzhou_guhuo")
                || matches!(
                    kind,
                    CardKind::Duel
                        | CardKind::Dismantle
                        | CardKind::Snatch
                        | CardKind::Jiedao
                        | CardKind::Le
                        | CardKind::Bing
                        | CardKind::Zhaomingdan
                );
            if need_other && t == actor {
                return false;
            }
            if kind.is_sha_like() {
                return in_attack_range_hearth(core, catalog, actor, t);
            }
            if kind == CardKind::Jiedao {
                let has_weapon = core.player(t).equips.iter().any(|&ec| {
                    catalog.card_kind(ec).equip_slot() == Some(crate::catalog::EquipSlot::Weapon)
                });
                return has_weapon
                    && core.alive_players().any(|v| {
                        v != actor
                            && v != t
                            && core.computed_distance(catalog, t, v)
                                <= core.attack_range(catalog, t)
                    });
            }
            let pl = core.player(t);
            if matches!(kind, CardKind::Snatch | CardKind::Bing) {
                let in_dist = core.computed_distance(catalog, actor, t) <= 1;
                if !in_dist {
                    return false;
                }
                return if kind == CardKind::Snatch {
                    !pl.hand.is_empty() || !pl.equips.is_empty()
                } else {
                    !pl.judge
                        .iter()
                        .any(|&jc| judge_card_kind(core, catalog, jc) == CardKind::Bing)
                };
            }
            if kind == CardKind::Le {
                return !pl
                    .judge
                    .iter()
                    .any(|&jc| judge_card_kind(core, catalog, jc) == CardKind::Le);
            }
            if is_huogong || base == "shandianjian" {
                return !pl.hand.is_empty();
            }
            if exact == "hsdusu_xueji" {
                return !pl.equips.is_empty();
            }
            if exact == "hsdusu_huoyanhua" {
                return in_attack_range_hearth(core, catalog, actor, t);
            }
            if exact == "hsdusu_kuyecao" {
                return !pl.yinshen_active;
            }
            if exact == "hsdusu_shinancao" {
                return !has_skill_hearth(core, catalog, t, "hsdusu_shinancao");
            }
            if matches!(
                exact,
                "hsqingyu_feibiao" | "hszuzhou_nvwudeganguo" | "hszuzhou_guhuo"
            ) {
                return !pl.hand.is_empty() || !pl.equips.is_empty();
            }
            if exact == "hsmengjing_huanxiaojiemei" || kind == CardKind::Zhiliaobo {
                return pl.hp < pl.max_hp as i32;
            }
            if exact == "hsmengjing_mengjing" {
                let has_le = pl
                    .judge
                    .iter()
                    .any(|&jc| judge_card_kind(core, catalog, jc) == CardKind::Le);
                return !has_le || !pl.equips.is_empty();
            }
            if exact == "hszuzhou_nvwudexuetu" {
                return !has_any_skill(core, catalog, t, "zhoujiang");
            }
            if matches!(kind, CardKind::Dismantle | CardKind::Zhaomingdan) {
                return !pl.hand.is_empty() || !pl.equips.is_empty() || !pl.judge.is_empty();
            }
            true
        })
        .map(ChoiceOption::Player)
        .collect()
}

pub(super) fn hearth_card_needs_target(
    core: &GameStateCore,
    catalog: &Catalog,
    actor: PlayerId,
    cid: CardId,
) -> bool {
    let (eff_cid, kind) = effective_hearth_card(core, catalog, actor, cid);
    let raw = catalog.cards[eff_cid as usize].name;
    let exact = exact_base_name(raw);
    let base = base_card_name(raw);
    kind.is_sha_like()
        || base == "shandianjian"
        || is_hshuanling_named_trick(exact)
        || matches!(
            kind,
            CardKind::Duel
                | CardKind::Dismantle
                | CardKind::Snatch
                | CardKind::Jiedao
                | CardKind::Le
                | CardKind::Bing
                | CardKind::Zhaomingdan
                | CardKind::Zhiliaobo
                | CardKind::Linghunzhihuo
                | CardKind::Shijieshu
        )
}

pub(super) fn is_tanmi_card_usable(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    cid: CardId,
) -> bool {
    if exact_base_name(catalog.cards[cid as usize].name) == "hsfashu_anyingjingxiang"
        && !core.declared_kinds.contains_key(&cid)
    {
        return false;
    }
    let (eff_cid, kind) = effective_hearth_card(core, catalog, owner, cid);
    let raw = catalog.cards[eff_cid as usize].name;
    let exact = exact_base_name(raw);
    let base = base_card_name(raw);
    if exact.starts_with("tuteng")
        || catalog.blocks_all_cards(core, owner)
        || core.player(owner).fengyin_active
    {
        return false;
    }
    let nature = match kind {
        CardKind::FireSha => crate::event::DamageNature::Fire,
        CardKind::LeiSha => crate::event::DamageNature::Thunder,
        _ => crate::event::DamageNature::Normal,
    };
    if crate::rvm_bridge::modifiers::eval_card_use_canceled(
        core, catalog, owner, eff_cid, kind, nature,
    ) {
        return false;
    }
    if base == "shihuawuqi" {
        return super::u05_hearth::flag(core, owner, "hs_shihuawuqi_used_turn") == 0;
    }
    if kind.is_equip() {
        return true;
    }
    if matches!(kind, CardKind::Shan | CardKind::Nullify | CardKind::Hufu) {
        return false;
    }
    let suit = catalog.effective_card_suit(core, owner, eff_cid);
    let is_red = matches!(suit, crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond);
    let fentian_red = is_red
        && catalog
            .skill_by_name(crate::vocabulary::names::BOSS_FENTIAN)
            .is_some_and(|d| core.has_skill(owner, d));
    if kind == CardKind::Jiu {
        let bushi = crate::rvm_bridge::modifiers::d34_l28_mods::bushi_unlimited_card(
            core, catalog, owner, Some(eff_cid),
        );
        return !core.turn.jiu_used || fentian_red || bushi;
    }
    if kind.is_sha_like() {
        if core.player(owner).wangshi_ban {
            return false;
        }
        if core.turn.current == owner && core.turn.phase == crate::event::Phase::Play {
            let (vm_sha_limit, vm_sha_unlimited) =
                crate::rvm_bridge::modifiers::eval_sha_use_limit(
                    core, catalog, owner, Some(eff_cid), kind, nature, 1,
                );
            let poyue = is_red
                && catalog
                    .skill_by_name("poyue")
                    .is_some_and(|d| core.has_skill(owner, d));
            let land_bonus =
                core.current_land == Some(CardKind::Longlushanfeng) && !core.turn.land_sha_used;
            let vm_used = core
                .vm_aux
                .flags
                .get(&(owner, "vm_sha_extra_used".to_string()))
                .copied()
                .unwrap_or(0);
            let unlimited = vm_sha_unlimited
                || fentian_red
                || poyue
                || land_bonus
                || super::l10_reguhuo::has_unlimited_sha(core, catalog, owner);
            if !unlimited
                && (vm_sha_limit == 0
                    || (core.turn.sha_used_play_phase && vm_sha_limit <= 1 + vm_used))
            {
                return false;
            }
        }
    }
    if kind == CardKind::Tao || exact == "hsqizhou_shui" {
        if kind == CardKind::Tao && crate::kernel::tao_futai2_banned(core, catalog, owner) {
            return false;
        }
        return core.player(owner).hp < core.player(owner).max_hp as i32;
    }
    if matches!(exact, "hsshenqi_kongbusangzhong" | "hsshenqi_morijingxiang") {
        if core.player(owner).face_down {
            return false;
        }
        let others = alive_others(core, owner);
        return if exact == "hsshenqi_morijingxiang" {
            others.into_iter().any(|p| {
                let pl = core.player(p);
                !pl.hand.is_empty() || !pl.equips.is_empty() || !pl.judge.is_empty()
            })
        } else {
            !others.is_empty()
        };
    }
    if exact == "hsmengjing_suxing" {
        return !alive_others(core, owner).is_empty();
    }
    if hearth_card_needs_target(core, catalog, owner, cid) {
        return !single_target_options(core, catalog, owner, cid).is_empty();
    }
    true
}
