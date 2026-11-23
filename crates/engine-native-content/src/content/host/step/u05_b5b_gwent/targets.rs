//! U05-5b 共享用牌可用性、出杀次数限制（`getCardUsable("sha")`）与目标过滤原语。
//!
//! 独立实现于 `u05_b5b_gwent/` 内，覆盖 `lanquan` / `sqlongwu` / `jielue` / `junchi`，
//! 与 `u05_b5a_gwent/` 保持零文件交叉以免合流冲突。

use super::*;
use crate::catalog::{CardKind, Catalog, EquipSlot};
use crate::event::DamageNature;
use crate::state::GameStateCore;

fn is_upstream_trick_card(name: &str, kind: CardKind) -> bool {
    kind.is_trick_kind()
        && !name.starts_with("gw_")
        && !name.starts_with("gwmaoxian_")
        && !name.starts_with("tuteng")
}

fn card_nature(kind: CardKind) -> DamageNature {
    match kind {
        CardKind::FireSha => DamageNature::Fire,
        CardKind::LeiSha => DamageNature::Thunder,
        _ => DamageNature::Normal,
    }
}

fn owner_has_snatch_or_bing_distance_free(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    card: CardId,
    kind: CardKind,
) -> bool {
    let has = |sk: &str| catalog.skill_by_name(sk).is_some_and(|d| core.has_skill(owner, d));
    if has(crate::content::standard::QI_CAI)
        || has(crate::content::standard::BOSS_SHOUYI)
        || (catalog
            .skill_by_name(crate::content::standard::BOSS_SHENEN)
            .is_some_and(|d| core.skills.keys().any(|(_, dd)| *dd == d))
            && core.is_boss_side(owner))
        || (is_red_card(catalog, card) && has(crate::content::standard::BOSS_FENTIAN))
    {
        return true;
    }
    crate::rvm_bridge::modifiers::eval_card_distance_unlimited(
        core,
        catalog,
        owner,
        card,
        kind,
        card_nature(kind),
    )
}

/// 评估 `actor` 对 `target` 使用 `card`（声明 `kind`）是否因帷幕/VM `CardTargetCancel`/潜行/禁目标等失效。
pub(super) fn b5b_target_canceled(
    core: &GameStateCore,
    catalog: &Catalog,
    actor: PlayerId,
    target: PlayerId,
    card: CardId,
    kind: CardKind,
) -> bool {
    if !core.player(target).alive || (target != actor && is_target_hidden(core, target)) {
        return true;
    }
    let name = clean_gwent_name(catalog.cards[card as usize].name);
    let nature = card_nature(kind);
    if crate::rvm_bridge::modifiers::eval_card_target_canceled(
        core, catalog, actor, target, card, kind, nature,
    ) {
        return true;
    }
    let is_black = !is_red_card(catalog, card);
    if is_upstream_trick_card(name, kind) && is_black {
        let weimu = [crate::content::standard::BOSS_WEIMU, "weimu", "reweimu"]
            .iter()
            .any(|&sk| catalog.skill_by_name(sk).is_some_and(|d| core.has_skill(target, d)));
        if weimu {
            return true;
        }
    }
    if kind == CardKind::Chain
        && crate::rvm_bridge::modifiers::d34_l28_mods::rewrite_tengjia_blocks_link(
            core, catalog, target,
        )
    {
        return true;
    }
    if matches!(kind, CardKind::Le | CardKind::Bing | CardKind::ShanDian) {
        if catalog.blocks_delayed_trick(core, target) {
            return true;
        }
        if matches!(kind, CardKind::Le | CardKind::ShanDian)
            && catalog
                .skill_by_name(crate::content::standard::BOSS_PANGUAN)
                .is_some_and(|d| core.has_skill(target, d))
        {
            return true;
        }
        if target != actor
            && catalog
                .skill_by_name("swd_wuxie")
                .is_some_and(|d| core.has_skill(actor, d))
        {
            return true;
        }
    }
    if kind.is_sha_like() {
        if crate::kernel::sha_user_or_target_blocked(core, catalog, actor, target) {
            return true;
        }
        let molicha = catalog
            .skill_by_name(crate::content::host::GJ_FOOD_MOLICHA_KEY)
            .is_some_and(|d| core.has_skill(target, d));
        if molicha && is_black {
            return true;
        }
        let has_armor = core
            .player(target)
            .equips
            .iter()
            .any(|ec| catalog.card_slot_of(core, *ec) == Some(EquipSlot::Armor));
        let remen = catalog
            .skill_by_name(crate::content::standard::BOSS_REMEN)
            .is_some_and(|d| core.has_skill(target, d));
        if kind == CardKind::Sha && nature == DamageNature::Normal && remen && !has_armor {
            return true;
        }
    }
    false
}

/// 对应上游 `player.getCardUsable("sha")`：同回合内（含 `lanquan` / 同回合 `phaseEnd` `sqlongwu`）检查剩余出杀额度。
pub(super) fn b5b_sha_within_usable_limit(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    c: CardId,
) -> bool {
    let raw_kind = catalog.card_kind(c);
    let kind = if is_sha_or_wuyao(catalog, c) {
        CardKind::Sha
    } else {
        raw_kind
    };
    let nature = card_nature(raw_kind);
    let (vm_sha_limit, vm_sha_unlimited) = crate::rvm_bridge::modifiers::eval_sha_use_limit(
        core,
        catalog,
        owner,
        Some(c),
        kind,
        nature,
        1,
    );
    let has_zhuge_clean = !crate::kernel::mbcmfangzhu_baiban(core, owner)
        && core.has_active_equip_kind(catalog, owner, CardKind::Zhuge);
    let unlimited_host = has_zhuge_clean
        || (!catalog.is_rechanyuan_sealed_name(core, owner, crate::vocabulary::names::PAO_XIAO)
            && catalog
                .skill_by_name(crate::vocabulary::names::PAO_XIAO)
                .is_some_and(|d| core.has_skill(owner, d)))
        || (!catalog.is_rechanyuan_sealed_name(core, owner, "paoxiao")
            && catalog
                .skill_by_name("paoxiao")
                .is_some_and(|d| core.has_skill(owner, d)))
        || (!catalog.is_rechanyuan_sealed_name(core, owner, crate::vocabulary::host::WUDIAN_BAOLEI_KEY)
            && catalog
                .skill_by_name(crate::vocabulary::host::WUDIAN_BAOLEI_KEY)
                .is_some_and(|d| core.has_skill(owner, d) && core.player(owner).baolei_mode == 1))
        || (!catalog.is_rechanyuan_sealed_name(core, owner, crate::vocabulary::host::L27_SBZHAXIANG_KEY)
            && catalog
                .skill_by_name(crate::vocabulary::host::L27_SBZHAXIANG_KEY)
                .is_some_and(|d| core.has_skill(owner, d))
            && sbzhaxiang_active(core, owner))
        || catalog
            .skill_by_name(crate::vocabulary::host::GJ_FOOD_YUANBAOROU_KEY)
            .is_some_and(|d| core.skills.get(&(owner, d)).is_some_and(|i| i.enabled && !i.once_used))
        || (!catalog.is_rechanyuan_sealed_name(core, owner, crate::vocabulary::names::BOSS_HUIHUO)
            && catalog
                .skill_by_name(crate::vocabulary::names::BOSS_HUIHUO)
                .is_some_and(|d| core.skills.get(&(owner, d)).is_some_and(|i| i.enabled && !i.once_used)))
        || (!catalog.is_rechanyuan_sealed_name(core, owner, "lingshi")
            && catalog
                .skill_by_name("lingshi")
                .is_some_and(|d| core.skills.get(&(owner, d)).is_some_and(|i| i.enabled && !i.once_used))
            && core
                .player(owner)
                .equips
                .iter()
                .any(|c| catalog.card_slot_of(core, *c) == Some(EquipSlot::Treasure)));
    let land_sha_bonus =
        core.current_land == Some(CardKind::Longlushanfeng) && !core.turn.land_sha_used;
    let poyue_red_free = catalog
        .skill_by_name("poyue")
        .is_some_and(|d| core.has_skill(owner, d))
        && is_red_card(catalog, c);
    let fentian_red = catalog
        .skill_by_name(crate::content::standard::BOSS_FENTIAN)
        .is_some_and(|d| core.has_skill(owner, d))
        && is_red_card(catalog, c);
    if vm_sha_unlimited || unlimited_host || fentian_red || poyue_red_free {
        return true;
    }
    if vm_sha_limit == 0 {
        return false;
    }
    if core.turn.current != owner || !core.turn.sha_used_play_phase {
        return true;
    }
    if land_sha_bonus {
        return true;
    }
    let dcdichou_eff_extra = {
        let has_eff = catalog
            .skill_by_name("dcdichou_eff")
            .or_else(|| crate::rvm_ops::resolve_skill_def(catalog, 2_754_298_115))
            .is_some_and(|d| core.has_skill(owner, d))
            || core
                .vm_aux
                .flags
                .get(&(owner, "skill:2754298115".to_string()))
                .copied()
                .unwrap_or(0)
                != 0;
        let bonus = core
            .vm_aux
            .marks
            .get(&(owner, 2_754_298_115))
            .copied()
            .unwrap_or(0);
        let used = core
            .vm_aux
            .flags
            .get(&(owner, "dcdichou_eff_sha_used".to_string()))
            .copied()
            .unwrap_or(0) as i32;
        has_eff && bonus > used
    };
    let vm_sha_extra_used = core
        .vm_aux
        .flags
        .get(&(owner, "vm_sha_extra_used".to_string()))
        .copied()
        .unwrap_or(0);
    dcdichou_eff_extra || vm_sha_limit > 1 + vm_sha_extra_used
}

fn b5b_sha_targets(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    sha_card: CardId,
) -> Vec<ChoiceOption> {
    let raw_kind = catalog.card_kind(sha_card);
    let kind = if raw_kind.is_sha_like() {
        raw_kind
    } else {
        CardKind::Sha
    };
    let nature = card_nature(raw_kind);
    let unlimited =
        owner_has_snatch_or_bing_distance_free(core, catalog, owner, sha_card, kind)
            || crate::rvm_bridge::modifiers::eval_card_distance_unlimited(
                core, catalog, owner, sha_card, kind, nature,
            );
    let range = core.attack_range(catalog, owner);
    core.seat_order_from(owner)
        .into_iter()
        .filter(|&t| {
            t != owner
                && (unlimited
                    || core.computed_distance(catalog, owner, t) <= range
                    || core.turn.zhufangshenshi_target == Some(t))
                && !b5b_target_canceled(core, catalog, owner, t, sha_card, kind)
        })
        .map(ChoiceOption::Player)
        .collect()
}

pub(super) fn b5b_jiedao_victim_options(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    holder: PlayerId,
) -> Vec<ChoiceOption> {
    let range = core.attack_range(catalog, holder);
    core.seat_order_from(owner)
        .into_iter()
        .filter(|&v| {
            v != owner
                && v != holder
                && core.player(v).alive
                && !is_target_hidden(core, v)
                && !crate::kernel::sha_user_or_target_blocked(core, catalog, holder, v)
                && (core.computed_distance(catalog, holder, v) <= range
                    || core.turn.zhufangshenshi_target == Some(v))
        })
        .map(ChoiceOption::Player)
        .collect()
}

fn b5b_jiedao_holder_options(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    jiedao_card: CardId,
) -> Vec<ChoiceOption> {
    core.seat_order_from(owner)
        .into_iter()
        .filter(|&h| {
            h != owner
                && !b5b_target_canceled(core, catalog, owner, h, jiedao_card, CardKind::Jiedao)
                && core
                    .player(h)
                    .equips
                    .iter()
                    .any(|c| catalog.card_slot_of(core, *c) == Some(EquipSlot::Weapon))
                && !b5b_jiedao_victim_options(core, catalog, owner, h).is_empty()
        })
        .map(ChoiceOption::Player)
        .collect()
}

pub(super) fn is_single_target_legal_no_dist(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    c: CardId,
    t: PlayerId,
) -> bool {
    if t == owner || !core.player(t).alive || catalog.blocks_all_cards(core, owner) {
        return false;
    }
    let name = clean_gwent_name(catalog.cards[c as usize].name);
    let kind = if is_sha_or_wuyao(catalog, c) {
        CardKind::Sha
    } else {
        catalog.card_kind(c)
    };
    let use_kind = if is_sha_or_wuyao(catalog, c) && !catalog.card_kind(c).is_sha_like() {
        CardKind::Sha
    } else {
        catalog.card_kind(c)
    };
    if crate::rvm_bridge::modifiers::eval_card_use_canceled(
        core,
        catalog,
        owner,
        c,
        use_kind,
        card_nature(use_kind),
    ) {
        return false;
    }
    if b5b_target_canceled(core, catalog, owner, t, c, use_kind) {
        return false;
    }
    if is_sha_or_wuyao(catalog, c) {
        return true;
    }
    let qianxun = catalog
        .skill_by_name(crate::content::standard::QIAN_XUN)
        .is_some_and(|d| core.has_skill(t, d));
    let kongcheng = catalog
        .skill_by_name(crate::content::standard::KONG_CHENG)
        .is_some_and(|d| core.has_skill(t, d));
    let wuxin = catalog
        .skill_by_name(crate::content::standard::BOSS_WUXIN)
        .is_some_and(|d| core.has_skill(owner, d));
    if name.starts_with("gw_") || name.starts_with("gwmaoxian_") {
        return b5b_card_targets(core, catalog, owner, c)
            .is_some_and(|(opts, min_t, _)| min_t <= 1 && opts.contains(&ChoiceOption::Player(t)));
    }
    match kind {
        CardKind::Tao => core.player(t).hp < core.player(t).max_hp as i32,
        CardKind::Duel => !(kongcheng && core.player(t).hand.is_empty()),
        CardKind::Dismantle => {
            !core.player(t).hand.is_empty()
                || !core.player(t).equips.is_empty()
                || !core.player(t).judge.is_empty()
        }
        CardKind::Snatch => {
            (!core.player(t).hand.is_empty() || !core.player(t).equips.is_empty()) && !qianxun
        }
        CardKind::Le => {
            !wuxin
                && !qianxun
                && !core
                    .player(t)
                    .judge
                    .iter()
                    .any(|jc| catalog.card_kind(*jc) == CardKind::Le)
        }
        CardKind::Bing => {
            !wuxin
                && !core
                    .player(t)
                    .judge
                    .iter()
                    .any(|jc| catalog.card_kind(*jc) == CardKind::Bing)
        }
        _ => true,
    }
}

pub(super) fn b5b_card_targets(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    c: CardId,
) -> Option<(Vec<ChoiceOption>, usize, usize)> {
    let name = clean_gwent_name(catalog.cards[c as usize].name);
    let kind = catalog.card_kind(c);
    if is_sha_or_wuyao(catalog, c) {
        return Some((b5b_sha_targets(core, catalog, owner, c), 1, 1));
    }
    if kind == CardKind::Jiedao {
        return Some((b5b_jiedao_holder_options(core, catalog, owner, c), 1, 1));
    }
    if kind.aoe_need().is_some()
        || kind.equip_slot().is_some()
        || (matches!(
            kind,
            CardKind::Wuzhong
                | CardKind::Taoyuan
                | CardKind::Wugu
                | CardKind::ShanDian
                | CardKind::Poxiao
                | CardKind::Wenyi
                | CardKind::Shanbengshu
        ) && !(kind == CardKind::Wuzhong && name.starts_with("gw")))
    {
        return None;
    }
    if matches!(
        name,
        "gw_hudiewu"
            | "gw_yigeniyin"
            | "gw_leizhoushu"
            | "gw_aerdeyin"
            | "gw_ansha"
            | "gw_xinsheng"
            | "gw_niuquzhijing"
            | "gw_zhongmozhizhan"
            | "gw_ganhan"
            | "gw_huangjiashenpan"
            | "gw_tunshi"
            | "gw_chongci"
            | "gw_shizizhaohuan"
            | "gw_zuihouyuanwang"
            | "gw_zirankuizeng"
            | "gw_guaiwuchaoxue"
            | "gw_xianzumaijiu"
            | "gwmaoxian_yenaifa"
            | "gwmaoxian_telisi"
            | "gwmaoxian_hengsaite"
            | "gwmaoxian_bulanwang"
            | "gwmaoxian_haluo"
            | "gwmaoxian_airuiting"
            | "gwmaoxian_aisinie"
            | "gwmaoxian_falanxisika"
            | "gwmaoxian_huoge"
    ) {
        return None;
    }
    if kind == CardKind::Chain {
        let opts: Vec<ChoiceOption> = core
            .seat_order_from(owner)
            .into_iter()
            .filter(|&p| !b5b_target_canceled(core, catalog, owner, p, c, CardKind::Chain))
            .map(ChoiceOption::Player)
            .collect();
        let max = opts.len().min(2);
        return Some((opts, 1, max));
    }
    let kongcheng = catalog.skill_by_name(crate::content::standard::KONG_CHENG);
    let qianxun = catalog.skill_by_name(crate::content::standard::QIAN_XUN);
    let wuxin = catalog
        .skill_by_name(crate::content::standard::BOSS_WUXIN)
        .is_some_and(|d| core.has_skill(owner, d));
    match kind {
        CardKind::Duel => {
            let opts = core
                .seat_order_from(owner)
                .into_iter()
                .filter(|&p| {
                    p != owner
                        && !b5b_target_canceled(core, catalog, owner, p, c, CardKind::Duel)
                        && !(kongcheng.is_some_and(|d| core.has_skill(p, d))
                            && core.player(p).hand.is_empty())
                })
                .map(ChoiceOption::Player)
                .collect();
            return Some((opts, 1, 1));
        }
        CardKind::Dismantle => {
            let opts = core
                .seat_order_from(owner)
                .into_iter()
                .filter(|&p| {
                    p != owner
                        && !b5b_target_canceled(core, catalog, owner, p, c, CardKind::Dismantle)
                        && (!core.player(p).hand.is_empty()
                            || !core.player(p).equips.is_empty()
                            || !core.player(p).judge.is_empty())
                })
                .map(ChoiceOption::Player)
                .collect();
            return Some((opts, 1, 1));
        }
        CardKind::Snatch => {
            let dist_free = owner_has_snatch_or_bing_distance_free(
                core,
                catalog,
                owner,
                c,
                CardKind::Snatch,
            );
            let opts = core
                .seat_order_from(owner)
                .into_iter()
                .filter(|&p| {
                    p != owner
                        && !b5b_target_canceled(core, catalog, owner, p, c, CardKind::Snatch)
                        && !qianxun.is_some_and(|d| core.has_skill(p, d))
                        && (!core.player(p).hand.is_empty() || !core.player(p).equips.is_empty())
                        && (dist_free || core.computed_distance(catalog, owner, p) <= 1)
                })
                .map(ChoiceOption::Player)
                .collect();
            return Some((opts, 1, 1));
        }
        CardKind::Le => {
            if wuxin {
                return Some((Vec::new(), 1, 1));
            }
            let opts = core
                .seat_order_from(owner)
                .into_iter()
                .filter(|&p| {
                    p != owner
                        && !b5b_target_canceled(core, catalog, owner, p, c, CardKind::Le)
                        && !qianxun.is_some_and(|d| core.has_skill(p, d))
                        && !core
                            .player(p)
                            .judge
                            .iter()
                            .any(|jc| catalog.card_kind(*jc) == CardKind::Le)
                })
                .map(ChoiceOption::Player)
                .collect();
            return Some((opts, 1, 1));
        }
        CardKind::Bing => {
            if wuxin {
                return Some((Vec::new(), 1, 1));
            }
            let dist_free =
                owner_has_snatch_or_bing_distance_free(core, catalog, owner, c, CardKind::Bing);
            let opts = core
                .seat_order_from(owner)
                .into_iter()
                .filter(|&p| {
                    p != owner
                        && !b5b_target_canceled(core, catalog, owner, p, c, CardKind::Bing)
                        && !core
                            .player(p)
                            .judge
                            .iter()
                            .any(|jc| catalog.card_kind(*jc) == CardKind::Bing)
                        && (dist_free || core.computed_distance(catalog, owner, p) <= 1)
                })
                .map(ChoiceOption::Player)
                .collect();
            return Some((opts, 1, 1));
        }
        CardKind::Qinpendayu
        | CardKind::Birinongwu
        | CardKind::Ciguhanshuang
        | CardKind::Baoxueyaoshui
        | CardKind::Zhihuanjun
        | CardKind::Zumoshoukao
        | CardKind::Aozuzhilei
        | CardKind::Kunenfayin
        | CardKind::Yanziyaoshui => {
            let opts = eligible_bronze_targets(core, catalog, owner, kind)
                .into_iter()
                .filter(|opt| {
                    matches!(
                        opt,
                        ChoiceOption::Player(p)
                            if !b5b_target_canceled(core, catalog, owner, *p, c, kind)
                    )
                })
                .collect();
            return Some((opts, 1, 1));
        }
        _ => {}
    }
    let others_with_cards = |need_hand: bool| -> Vec<ChoiceOption> {
        core.seat_order_from(owner)
            .into_iter()
            .filter(|&p| {
                p != owner
                    && !b5b_target_canceled(core, catalog, owner, p, c, kind)
                    && (!need_hand || !core.player(p).hand.is_empty())
            })
            .map(ChoiceOption::Player)
            .collect()
    };
    match name {
        "gwmaoxian_yioufeisi" => Some((others_with_cards(false), 2, 2)),
        "gwmaoxian_fulisi" => {
            let opts = others_with_cards(true);
            let max = opts.len().min(3);
            Some((opts, 1, max))
        }
        "gwmaoxian_kaerweite" => {
            let opts = others_with_cards(true);
            let max = opts.len().min(2);
            Some((opts, 1, max))
        }
        "gwmaoxian_fuertaisite" => {
            let opts: Vec<ChoiceOption> = core
                .seat_order_from(owner)
                .into_iter()
                .filter(|&p| !b5b_target_canceled(core, catalog, owner, p, c, kind))
                .map(ChoiceOption::Player)
                .collect();
            let max = opts.len().min(2);
            Some((opts, 1, max))
        }
        "gw_zhuoshao" => {
            let mhp = core.alive_players().map(|p| core.player(p).hp).max().unwrap_or(0);
            let opts: Vec<ChoiceOption> = core
                .seat_order_from(owner)
                .into_iter()
                .filter(|&p| {
                    core.player(p).hp == mhp
                        && !b5b_target_canceled(core, catalog, owner, p, c, kind)
                })
                .map(ChoiceOption::Player)
                .collect();
            let max = opts.len();
            Some((opts, 1, max))
        }
        "gw_baishuang" => {
            let opts: Vec<ChoiceOption> =
                eligible_bronze_targets(core, catalog, owner, CardKind::Ciguhanshuang)
                    .into_iter()
                    .filter(|opt| {
                        matches!(
                            opt,
                            ChoiceOption::Player(p)
                                if !b5b_target_canceled(core, catalog, owner, *p, c, kind)
                        )
                    })
                    .collect();
            let max = opts.len().min(3);
            Some((opts, 1, max))
        }
        "gw_baobaoshu" => {
            let opts: Vec<ChoiceOption> = core
                .seat_order_from(owner)
                .into_iter()
                .filter(|&p| {
                    !has_named_skill(core, catalog, p, "gw_baobaoshu")
                        && !b5b_target_canceled(core, catalog, owner, p, c, kind)
                })
                .map(ChoiceOption::Player)
                .collect();
            let max = opts.len().min(2);
            Some((opts, 1, max))
        }
        "gw_youer" | "gw_tongdi" => Some((others_with_cards(true), 1, 1)),
        "gw_nuhaifengbao" => {
            let opts = core
                .seat_order_from(owner)
                .into_iter()
                .filter(|&p| {
                    !has_named_skill(core, catalog, p, "gw_nuhaifengbao")
                        && !b5b_target_canceled(core, catalog, owner, p, c, kind)
                })
                .map(ChoiceOption::Player)
                .collect();
            Some((opts, 1, 1))
        }
        "gwmaoxian_enxier" => {
            let h0 = core.player(owner).hand.len() as i32;
            let opts = core
                .seat_order_from(owner)
                .into_iter()
                .filter(|&p| {
                    (core.player(p).hand.len() as i32 - h0).abs() <= 1
                        && !b5b_target_canceled(core, catalog, owner, p, c, kind)
                })
                .map(ChoiceOption::Player)
                .collect();
            Some((opts, 1, 1))
        }
        "gwmaoxian_kuite" => {
            let h0 = core.player(owner).hand.len();
            let opts = core
                .seat_order_from(owner)
                .into_iter()
                .filter(|&p| {
                    p != owner
                        && core.player(p).hand.len() >= h0
                        && !b5b_target_canceled(core, catalog, owner, p, c, kind)
                })
                .map(ChoiceOption::Player)
                .collect();
            Some((opts, 1, 1))
        }
        "gwmaoxian_dagong" => {
            let opts = core
                .seat_order_from(owner)
                .into_iter()
                .filter(|&p| {
                    (!has_named_skill(core, catalog, p, GW_CIGU_STATE_KEY)
                        || !has_named_skill(core, catalog, p, GW_QINPEN_STATE_KEY)
                        || !has_named_skill(core, catalog, p, GW_BIRIN_STATE_KEY))
                        && !b5b_target_canceled(core, catalog, owner, p, c, kind)
                })
                .map(ChoiceOption::Player)
                .collect();
            Some((opts, 1, 1))
        }
        "gwmaoxian_luoqi" => Some((others_with_cards(false), 1, 1)),
        "gw_butianshu" | "gwmaoxian_jieluote" | "gwmaoxian_laduoweide" | "gwmaoxian_gaier" => {
            let opts = core
                .seat_order_from(owner)
                .into_iter()
                .filter(|&p| !b5b_target_canceled(core, catalog, owner, p, c, kind))
                .map(ChoiceOption::Player)
                .collect();
            Some((opts, 1, 1))
        }
        _ => None,
    }
}

fn b5b_no_target_usable(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    c: CardId,
    name: &str,
    kind: CardKind,
) -> bool {
    if kind.equip_slot().is_some() {
        return true;
    }
    if kind.aoe_need().is_some() {
        return core
            .alive_players()
            .any(|p| p != owner && !b5b_target_canceled(core, catalog, owner, p, c, kind));
    }
    if kind == CardKind::Tao {
        return core.player(owner).hp < core.player(owner).max_hp as i32
            && !b5b_target_canceled(core, catalog, owner, owner, c, kind);
    }
    if kind == CardKind::Jiu || name == "gw_lang" {
        return !core.turn.jiu_used;
    }
    if name == "gw_xianzumaijiu" {
        return !has_named_skill(core, catalog, owner, "gw_xianzumaijiu")
            && flag(core, owner, "gw_xianzumaijiu_used_play") == 0;
    }
    if kind == CardKind::ShanDian {
        return !b5b_target_canceled(core, catalog, owner, owner, c, kind)
            && !core
                .player(owner)
                .judge
                .iter()
                .any(|jc| catalog.card_kind(*jc) == CardKind::ShanDian);
    }
    if matches!(
        kind,
        CardKind::Taoyuan | CardKind::Wugu | CardKind::Poxiao | CardKind::Wenyi
    ) {
        return core
            .alive_players()
            .any(|p| !b5b_target_canceled(core, catalog, owner, p, c, kind));
    }
    if kind == CardKind::Shanbengshu {
        return core.alive_players().any(|p| {
            p != owner
                && !core.is_friend_of(catalog, owner, p)
                && !core.player(p).equips.is_empty()
                && !b5b_target_canceled(core, catalog, owner, p, c, kind)
        });
    }
    if kind == CardKind::Wuzhong
        && (matches!(name, "无中生有" | "wuzhong") || name.starts_with("wuzhong_"))
    {
        return !b5b_target_canceled(core, catalog, owner, owner, c, kind);
    }
    match name {
        "gw_guaiwuchaoxue" => flag(core, owner, "gw_guaiwuchaoxue_used_play") == 0,
        "gw_hudiewu" => core.seat_order_from(owner).into_iter().any(|p| {
            p != owner
                && core.player(p).alive
                && (!core.player(p).equips.is_empty() || !core.player(p).judge.is_empty())
        }),
        "gw_aerdeyin" => core
            .seat_order_from(owner)
            .into_iter()
            .any(|p| p != owner && core.player(p).alive && !core.is_friend_of(catalog, owner, p)),
        "gw_ansha" => core.seat_order_from(owner).into_iter().any(|p| {
            p != owner
                && core.player(p).alive
                && !core.is_friend_of(catalog, owner, p)
                && core.player(p).hp == 1
        }),
        "gw_xinsheng" => core.seat_order_from(owner).into_iter().any(|p| {
            core.player(p).alive
                && (!matches!(core.mode, crate::state::GameMode::Kingdoms)
                    || core.player(p).revealed_generals[0]
                    || core.player(p).revealed_generals[1])
        }),
        "gw_niuquzhijing" => {
            let cur_hp = core.player(owner).hp;
            core.seat_order_from(owner)
                .into_iter()
                .any(|p| core.player(p).alive && core.player(p).hp != cur_hp)
        }
        "gw_tunshi" => {
            if core.player(owner).max_hp <= 1 {
                return false;
            }
            core.seat_order_from(owner).into_iter().any(|p| {
                p != owner
                    && core.player(p).alive
                    && !core.is_friend_of(catalog, owner, p)
                    && core.skills.iter().any(|((o, def), _)| {
                        *o == p
                            && !catalog.is_charlotte_skill(*def)
                            && !core.player(p).equips.iter().any(|ec| {
                                catalog.cards[*ec as usize]
                                    .equip_skills
                                    .contains(&catalog.skills[*def as usize].name)
                            })
                    })
            })
        }
        "gw_yigeniyin"
        | "gw_leizhoushu"
        | "gw_zhongmozhizhan"
        | "gw_ganhan"
        | "gw_huangjiashenpan"
        | "gw_chongci"
        | "gw_shizizhaohuan"
        | "gw_zuihouyuanwang"
        | "gw_zirankuizeng"
        | "gwmaoxian_yenaifa"
        | "gwmaoxian_telisi"
        | "gwmaoxian_hengsaite"
        | "gwmaoxian_bulanwang"
        | "gwmaoxian_haluo"
        | "gwmaoxian_airuiting"
        | "gwmaoxian_aisinie"
        | "gwmaoxian_falanxisika"
        | "gwmaoxian_huoge" => true,
        _ => false,
    }
}

pub(super) fn b5b_card_can_use(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    c: CardId,
    enforce_sha_limit: bool,
) -> bool {
    if !core.player(owner).alive || catalog.blocks_all_cards(core, owner) {
        return false;
    }
    let name = clean_gwent_name(catalog.cards[c as usize].name);
    let kind = catalog.card_kind(c);
    if matches!(
        name,
        "gw_dieyi" | "gwjinli_jiu" | "gw_fuyuan" | "gw_dudayuanshuai1" | "gw_dudayuanshuai2"
    ) {
        return false;
    }
    if kind == CardKind::Shan || kind == CardKind::Nullify {
        return false;
    }
    let eff_kind = if is_sha_or_wuyao(catalog, c) && !kind.is_sha_like() {
        CardKind::Sha
    } else if name == "gw_lang" {
        CardKind::Jiu
    } else {
        kind
    };
    if crate::rvm_bridge::modifiers::eval_card_use_canceled(
        core,
        catalog,
        owner,
        c,
        eff_kind,
        card_nature(eff_kind),
    ) {
        return false;
    }
    if is_sha_or_wuyao(catalog, c) {
        return !crate::kernel::sha_blocked_for(core, catalog, owner)
            && (!enforce_sha_limit || b5b_sha_within_usable_limit(core, catalog, owner, c))
            && !b5b_sha_targets(core, catalog, owner, c).is_empty();
    }
    if let Some((opts, min_t, _)) = b5b_card_targets(core, catalog, owner, c) {
        return opts.len() >= min_t;
    }
    b5b_no_target_usable(core, catalog, owner, c, name, kind)
}
