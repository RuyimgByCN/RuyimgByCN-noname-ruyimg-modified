use super::super::{flag_val, has_ext_skill, is_target_unequip2};
use super::{d34_l25_mods, has_armor};
use crate::rvm_bridge::modifiers::{d34_l11_mods_npc_a, d34_l28_mods, eval_ignore_armor};
use crate::catalog::{CardKind, Catalog};
use crate::event::DamageNature;
use crate::ids::{CardId, PlayerId};
use crate::state::GameStateCore;

pub(in crate::rvm_bridge::modifiers) fn apply_damage_amount(
    state: &GameStateCore,
    catalog: &Catalog,
    source: Option<PlayerId>,
    target: PlayerId,
    card: Option<CardId>,
    nature: DamageNature,
    mut cur: i32,
) -> i32 {
    if cur <= 0 || catalog.has_diaohulishan(state, target) {
        return 0;
    }
    let ckind = card.map(|c| catalog.card_kind(c));
    let is_sha = ckind.is_some_and(CardKind::is_sha_like);
    let is_trick = ckind.is_some_and(|k| !k.is_basic_kind() && k.equip_slot().is_none());

    // extension.js:3723 (`txhuangfu`: immune to thunder damage)
    if nature == DamageNature::Thunder
        && (flag_val(state, target, "l11_mod:txhuangfu") > 0
            || has_ext_skill(state, catalog, target, "txhuangfu"))
    {
        return 0;
    }
    // extension.js:5851 (`txyongguan`), `shenhua.js:7913` (`retianxiang`), `refresh/skill.js:10339` (`wulie`),
    // `mobile/skill.js:30818` (`xinfu_daigong`), `shiji.js:4013` (`mjmouzhi`)
    if flag_val(state, target, "l11_mod:txyongguan_ward") > 0
        || flag_val(state, target, "l11_mod:wulie_lie") > 0
        || flag_val(state, target, "l11_mod:xinfu_daigong_ward") > 0
        || (flag_val(state, target, "l11_jiuwei_misi_active") > 0 && state.player(target).hp <= 1)
    {
        return 0;
    }
    if let Some(src) = source {
        // `yijiang.js:9766` (`zhiman`: prevent damage to steal field card)
        if flag_val(state, src, "l11_mod:zhiman_prevent") > 0 {
            return 0;
        }
        let pot_y = flag_val(state, src, "potzhuangshi_damage_y");
        if pot_y > 0 && card.is_some() && state.turn.cards_used_play_phase <= pot_y {
            cur += 1;
        }
        if src != target
            && (flag_val(state, target, "xinfu_xionghuo_marked") > 0
                || flag_val(state, target, "xionghuo") > 0)
            && (has_ext_skill(state, catalog, src, "xinfu_xionghuo")
                || flag_val(state, src, "xinfu_xionghuo_init") > 0
                || flag_val(state, src, "xinfu_xionghuo_marks") > 0)
        {
            cur += 1;
        }
        if card.is_some() && flag_val(state, src, "mbwuling_spirit") == 1 {
            cur += 1;
        }
        let vuln_from = flag_val(state, target, "xinfu_lingren_vuln_from");
        let vuln_card = flag_val(state, target, "xinfu_lingren_vuln_card");
        if vuln_from == u32::from(src) + 1
            && (is_sha || is_trick)
            && card.is_some_and(|c| c + 1 == vuln_card)
        {
            cur += 1;
        }
    }
    if flag_val(state, target, "twzuhuo_shields") > 0
        || flag_val(state, target, "l27_twzuhuo_shield") > 0
    {
        return 0;
    }
    if flag_val(state, target, "mbwuling_spirit") == 3 {
        cur = (cur - 1).max(0);
    }
    if is_sha && flag_val(state, target, "l27_shouxi_nullified") > 0 {
        return 0;
    }
    if is_trick && flag_val(state, target, "l27_dinghan_negated") > 0 {
        return 0;
    }
    // extension.js:5657-5681 (`txmanjia` = `txmanjia1` + `txmanjia2`: unarmored Vine Armor)
    let has_txmanjia1 = flag_val(state, target, "l11_mod:txmanjia") > 0
        || has_ext_skill(state, catalog, target, "txmanjia")
        || has_ext_skill(state, catalog, target, "txmanjia1");
    let has_txmanjia2 = flag_val(state, target, "l11_mod:txmanjia") > 0
        || has_ext_skill(state, catalog, target, "txmanjia")
        || has_ext_skill(state, catalog, target, "txmanjia2");
    if !has_armor(state, catalog, target) {
        let armor_ignored = is_target_unequip2(state, target)
            || source.is_some_and(|src| {
                d34_l28_mods::source_ignores_armor(
                    state, catalog, src, target, card, ckind, nature,
                )
            });
        if has_txmanjia1
            && !armor_ignored
            && (matches!(ckind, Some(CardKind::SavageAttack | CardKind::ArcheryAttack))
                || (matches!(ckind, Some(CardKind::Sha)) && nature == DamageNature::Normal))
        {
            return 0;
        }
        if has_txmanjia2 && nature == DamageNature::Fire {
            cur += 1;
        }
    }

    if let Some(src) = source {
        // extension.js:554, 572, 699, 1344, 3355, 5809 & cross-package damage boosts
        if flag_val(state, src, "l11_mod:damage_plus1") > 0
            || (flag_val(state, src, "l11_mod:kyou_zhidian_dmg") > 0
                && (is_sha || card.is_none_or(|c| flag_val(state, src, &format!("l11_kyou_dmg_card:{c}")) > 0)))
            || (is_sha
                && card.is_some_and(|c| {
                    matches!(catalog.effective_card_suit(state, src, c), crate::catalog::Suit::Heart)
                })
                && flag_val(state, src, &format!("l11_yijue_heart_target:{target}")) > 0)
            || ((is_sha || matches!(ckind, Some(CardKind::Duel)))
                && flag_val(state, src, "l11_mod:reluoyi_dmg_plus1") > 0)
            || flag_val(state, src, "l11_mod:sbliegong_dmg") > 0
            || flag_val(state, src, "l11_mod:xinfu_zengdao_dao") > 0
        {
            cur += 1;
        }
        if flag_val(state, src, "l11_mod:twxuhe_plus2") > 0 {
            cur += 2;
        }
        if is_sha
            && (flag_val(state, src, "l11_mod:txfusha") > 0
                || flag_val(state, src, "l11_leijimu_thunder_sha") > 0)
        {
            cur += 1;
        }
        // extension_buff.js:321 (`buff_txhj_taipingzhoufa`) & extension_card.js:1013 (`txhj_chiyanzhenhunqin`)
        if is_sha
            && (matches!(nature, DamageNature::Fire | DamageNature::Thunder)
                || flag_val(state, src, "l11_chiyan_all_fire") > 0)
            && (flag_val(state, src, "l11_taipingzhoufa") > 0
                || flag_val(state, src, "l11_chiyan_all_fire") > 0)
        {
            cur += 1;
        }
        // `newjiang/skill.js:7123` (`spyishi`: damage -1 to steal equip)
        if flag_val(state, src, "l11_mod:spyishi_reduce") > 0 {
            cur = (cur - 1).max(0);
        }
        // L19 末日乱斗 (`_wujinmoshiDamage`, `_wujinmoshiDamageE`)
        if has_ext_skill(state, catalog, src, "_wujinmoshiDamage")
            || has_ext_skill(state, catalog, src, "_wujinmoshiDamageE")
        {
            cur += flag_val(state, src, "wujin_damage_bonus")
                .max(flag_val(state, src, "l19_wujin_damage_bonus"))
                .max(1) as i32;
        } else if flag_val(state, src, "wujin_damage_bonus") > 0
            || flag_val(state, src, "l19_wujin_damage_bonus") > 0
        {
            cur += flag_val(state, src, "wujin_damage_bonus")
                .max(flag_val(state, src, "l19_wujin_damage_bonus")) as i32;
        }
    }

    // `newjiang/skill.js:6760` (`weifeng`: `惧` mark +1 damage) & `sp2/skill.js:4647` (`yise`: black card +1 Sha damage)
    if flag_val(state, target, "l11_mod:weifeng_ju") > 0
        || (is_sha && flag_val(state, target, "l11_mod:yise_black_vuln") > 0)
    {
        cur += 1;
    }
    // `sp.js:18564` (`jieyuan_less`: damage -1)
    if flag_val(state, target, "l11_mod:jieyuan_less") > 0 {
        cur = (cur - 1).max(0);
    }
    // extension.js:3760 (`txxiebi`: damage > 1 -> clamp to 1; real `txxiebi` runs at DamageBegin4)
    if cur > 1 && flag_val(state, target, "l11_mod:txxiebi") > 0 {
        cur = 1;
    }
    // extension_buff.js:288, 299 (`xihuozhikai`, `dingleizhikai`)
    if has_armor(state, catalog, target) {
        if nature == DamageNature::Fire && flag_val(state, target, "l11_xihuozhikai") > 0 {
            cur = (cur - 1).max(0);
        }
        if nature == DamageNature::Thunder && flag_val(state, target, "l11_dingleizhikai") > 0 {
            cur = (cur - 1).max(0);
        }
    }
    d34_l25_mods::apply_damage_amount(state, catalog, source, target, card, nature, cur)
}

pub(in crate::rvm_bridge::modifiers) fn tpys_damage_prevented(
    state: &GameStateCore,
    catalog: &Catalog,
    source: Option<PlayerId>,
    target: PlayerId,
    card: Option<CardId>,
    nature: DamageNature,
) -> bool {
    if !matches!(nature, DamageNature::Fire | DamageNature::Thunder) {
        return false;
    }
    let has_tpys = flag_val(state, target, "th_taipingyaoshu_prevent_elemental") > 0
        || flag_val(state, target, "l28_taipingyaoshu_equipped") > 0
        || has_ext_skill(state, catalog, target, "th_taipingyaoshu")
        || state.player(target).equips.iter().any(|&c| {
            !state.is_gw_dieyi_masked(catalog, target, c)
                && catalog
                    .cards
                    .get(c as usize)
                    .is_some_and(|d| d.name == "th_taipingyaoshu")
        });
    if !has_tpys || is_target_unequip2(state, target) {
        return false;
    }
    let src_unequip = source.is_some_and(|src| {
        eval_ignore_armor(state, catalog, src, target, card, nature)
            || (card.is_some_and(|c| catalog.card_kind(c).is_sha_like())
                && (state.has_active_equip_kind(catalog, src, CardKind::Qinggang)
                    || catalog
                        .skill_by_name(crate::vocabulary::names::BOSS_ZHADAO)
                        .is_some_and(|def| state.has_skill(src, def))))
    });
    !src_unequip
}

pub(in crate::rvm_bridge::modifiers) fn apply_ignore_armor(
    state: &GameStateCore,
    catalog: &Catalog,
    source: PlayerId,
    target: PlayerId,
    card: Option<CardId>,
    mut ignore: bool,
) -> bool {
    // `mobile.js:16944` (`gnsheque`) & `diy.js:3461` (`kyou_zhidian`)
    if flag_val(state, source, "l11_mod:gnsheque_pierce") > 0
        || flag_val(state, source, "l11_mod:kyou_zhidian_pierce") > 0
        || d34_l11_mods_npc_a::card_ignore_armor(state, source, card)
    {
        ignore = true;
    }
    let ignore = super::d34_l11_mods_vm_b::apply_ignore_armor(state, catalog, source, target, ignore);
    d34_l25_mods::apply_ignore_armor(state, catalog, source, target, card, ignore)
}

pub(in crate::rvm_bridge::modifiers) fn apply_card_unrespondable(
    state: &GameStateCore,
    catalog: &Catalog,
    source: PlayerId,
    kind: CardKind,
    mut unrespondable: bool,
) -> bool {
    if kind.is_sha_like()
        && (flag_val(state, source, "l11_mod:kyou_zhidian_unresp") > 0
            || flag_val(state, source, "l11_mod:sbliegong_unresp") > 0
            || flag_val(state, source, "l11_mod:twxiangyu_unresp") > 0
            || has_ext_skill(state, catalog, source, "twxiangyu"))
    {
        unrespondable = true;
    }
    if flag_val(state, source, "l11_mod:spwenji_unresp") > 0
        || flag_val(state, source, "l11_mod:twqirang") > 0
    {
        unrespondable = true;
    }
    let pot_x = flag_val(state, source, "potzhuangshi_unblockable_x");
    if pot_x > 0 && state.turn.cards_used_play_phase <= pot_x {
        unrespondable = true;
    }
    let xiafeng_x = flag_val(state, source, "twxiafeng_x")
        .max(flag_val(state, source, "l27_twxiafeng_spent"));
    if xiafeng_x > 0 && state.turn.cards_used_play_phase <= xiafeng_x {
        unrespondable = true;
    }
    d34_l25_mods::apply_card_unrespondable(state, catalog, source, kind, unrespondable)
}

/// `txtanbei_effect3` + `txtanbei_effect1` (`extension.js:4729-4739`):
/// `actor` 对携带 `txtanbei_effect1` 的 `target` 使用牌没有次数与距离限制。
pub fn tanbei_unlimited_target(
    state: &GameStateCore,
    catalog: &Catalog,
    actor: PlayerId,
    target: PlayerId,
) -> bool {
    (has_ext_skill(state, catalog, actor, "txtanbei_effect3")
        && has_ext_skill(state, catalog, target, "txtanbei_effect1"))
        || flag_val(state, actor, &format!("skill_turn:xinfu_tanbei_unlimited:{target}")) > 0
        || (flag_val(state, actor, "l11_xiongluan1_active") > 0
            && flag_val(state, target, &format!("l11_xiongluan2_from:{actor}")) > 0)
}
