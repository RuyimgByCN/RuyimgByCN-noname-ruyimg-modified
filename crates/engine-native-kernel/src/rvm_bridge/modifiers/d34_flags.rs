//! Authoritative kernel evaluation for Decision 34 (`U01`, `U05`, `U07`, `U08`, `L10`, `L11`,
//! `L19`, `L20`, `L25`, `L27`, `L28`, `L29`) passive skills, equipment/servant modifiers,
//! explicit per-skill buff/debuff flags, and `VmViewAsSpec` conversions.
//!
//! Zero blanket `has_flag_prefix` catch-alls: every modifier rule checks an explicit
//! upstream skill or per-skill flag backed by upstream `mod.*` / `trigger.*` semantics.

use crate::catalog::{CardKind, Catalog, EquipSlot, Suit};
use crate::event::DamageNature;
use crate::ids::{CardId, PlayerId};
use crate::state::GameStateCore;

#[path = "d34_view_as.rs"]
mod d34_view_as;
#[path = "d34_u05_mods.rs"]
mod d34_u05_mods;
#[path = "d34_l11_mods.rs"]
pub(super) mod d34_l11_mods;
#[path = "d34_u01_mods.rs"]
mod d34_u01_mods;
#[path = "d34_unrespondable.rs"]
mod d34_unrespondable;
pub use d34_l11_mods::{
    apply_draw_count, has_yaya_protected, is_card_ignored_in_discard_phase,
    tanbei_unlimited_target, txlianbi_blocks_unlink, txminguangkai_blocks_link,
};
pub use d34_u01_mods::{
    has_ddddanggu_negative, is_dddfusi_hand_locked, is_dddtongyu_hand_locked, is_u01_hand_locked,
    is_u01_real_lord, is_u01_shown_card,
};
pub use d34_unrespondable::{apply_card_unrespondable, apply_card_unrespondable_with_nature};
pub use d34_view_as::find_d34_flag_view_as;

#[inline]
pub(super) fn flag_val(state: &GameStateCore, player: PlayerId, key: &str) -> u32 {
    if state.vm_aux.flags.is_empty() {
        return 0;
    }
    state
        .vm_aux
        .flags
        .get(&(player, key.to_string()))
        .copied()
        .unwrap_or(0)
}

#[inline]
pub(super) fn has_ext_skill(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    name: &str,
) -> bool {
    if catalog.skills.len() <= 24 {
        return false;
    }
    catalog
        .skill_by_name(name)
        .is_some_and(|def| state.has_skill(player, def))
}

#[inline]
pub(super) fn has_slot_equipped(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    pred: impl Fn(EquipSlot) -> bool,
) -> bool {
    state.player(player).equips.iter().any(|&c| {
        catalog
            .cards
            .get(c as usize)
            .and_then(|d| d.kind.equip_slot())
            .is_some_and(&pred)
    })
}

#[inline]
pub(super) fn has_weapon(state: &GameStateCore, catalog: &Catalog, player: PlayerId) -> bool {
    has_slot_equipped(state, catalog, player, |s| s == EquipSlot::Weapon)
}

#[inline]
pub(super) fn card_suit(catalog: &Catalog, card: Option<CardId>) -> Option<Suit> {
    card.and_then(|c| catalog.cards.get(c as usize).map(|d| d.suit))
}

#[inline]
fn suit_code(suit: Suit) -> u32 {
    match suit {
        Suit::Spade => 1,
        Suit::Heart => 2,
        Suit::Club => 3,
        Suit::Diamond => 4,
    }
}

pub fn is_card_ignored_hand_limit(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    card: CardId,
    kind: CardKind,
) -> bool {
    if flag_val(state, player, "dddfusi_active") > 0 {
        return true;
    }
    if catalog
        .skill_by_name("dddfusi")
        .is_some_and(|def| state.has_skill(player, def))
        && is_u01_shown_card(state, player, card)
    {
        return true;
    }
    if flag_val(state, player, "u05_mod:qingzun") > 0 && kind == CardKind::Jiu {
        return true;
    }
    if flag_val(state, player, "u05_mod:hsbaowu_huangjinyuanhou") > 0 && kind.is_basic_kind() {
        return true;
    }
    d34_u05_mods::is_card_ignored_hand_limit(state, catalog, player, card)
        || d34_l11_mods::is_card_ignored_hand_limit(state, catalog, player, card, kind)
}

pub fn apply_hand_limit(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    mut limit: i32,
) -> i32 {
    if flag_val(state, player, "l11_xuwang_hand_minus1") > 0
        || has_ext_skill(state, catalog, player, "txhj_xuwangzhimian")
    {
        limit -= 1;
    }
    let l11_plus3 = flag_val(state, player, "l11_buff_handlimit_plus3");
    if l11_plus3 > 0 {
        limit += l11_plus3 as i32;
    } else if has_ext_skill(state, catalog, player, "buff_txhj_wuxiangdexinxue")
        || has_ext_skill(state, catalog, player, "_buff_txhj_wuxiangdexinxue")
        || has_ext_skill(state, catalog, player, "txhj_wuxiangdexinxue")
    {
        limit += 3;
    }
    limit += if has_ext_skill(state, catalog, player, "txhj_aHaoSkill3") {
        state.player(player).equips.len() as i32
    } else {
        flag_val(state, player, "l11_ahao_equip_bonus") as i32
    };
    if flag_val(state, player, "l25_yuxi_treasure_active") > 0 {
        limit += 1;
    }
    limit -= flag_val(state, player, "l25_chanyuan_crazy") as i32;
    limit += flag_val(state, player, "l25_level_maxhand_bonus") as i32;
    limit += flag_val(state, player, "l19_wujin_hand_limit_bonus") as i32;
    limit -= flag_val(state, player, "l27_jishe_limit_minus") as i32;
    let has_tpys_card = state.player(player).equips.iter().any(|&c| {
        catalog
            .cards
            .get(c as usize)
            .is_some_and(|cd| cd.name == "th_taipingyaoshu")
    });
    if !is_target_unequip2(state, player)
        && (flag_val(state, player, "th_taipingyaoshu_prevent_elemental") > 0
            || flag_val(state, player, "l28_taipingyaoshu_equipped") > 0
            || has_tpys_card
            || has_ext_skill(state, catalog, player, "th_taipingyaoshu"))
    {
        let mut faction_mask = 0u16;
        for (pid, p) in state.players.iter().enumerate() {
            if !p.alive {
                continue;
            }
            if let Some(g) = p.general_slots[0] {
                if let Some((_, faction, _, _, _)) = catalog.general_meta(g) {
                    faction_mask |= 1u16 << (faction as u8);
                    continue;
                }
            }
            faction_mask |= 1u16 << ((pid % 3) as u8);
        }
        let distinct = faction_mask.count_ones() as i32;
        limit += (distinct - 1).max(0);
    }
    let zyhufu_flag = flag_val(state, player, "yxs_zyhufu_hand_limit_bonus");
    if zyhufu_flag > 0 {
        limit += zyhufu_flag as i32;
    } else if has_ext_skill(state, catalog, player, "zyhufu")
        && has_slot_equipped(state, catalog, player, |s| s == EquipSlot::Armor)
    {
        limit += 5;
    }
    if has_ext_skill(state, catalog, player, "taolue")
        || flag_val(state, player, "yxs_taolue_hand_limit") > 0
    {
        limit += 1;
    }
    if has_ext_skill(state, catalog, player, "kongju")
        || flag_val(state, player, "yxs_kongju_ward") > 0
    {
        let lost = (state.player(player).max_hp as i32 - state.player(player).hp).max(0);
        limit += lost;
    }
    if flag_val(state, player, "yxs_kuangchan_no_lord_hp") > 0
        && state.player(player).hp > state.player(player).max_hp as i32
    {
        limit -= 1;
    }
    if flag_val(state, player, "u05_mod:gw_qinpendayu") > 0 {
        limit -= 1;
    }
    if flag_val(state, player, "u05_mod:chunbing") > 0 {
        limit += 1;
    }
    if flag_val(state, player, "u05_mod:hlongyi") > 0 {
        limit += 1;
    }
    if flag_val(state, player, "u05_mod:moyao") > 0 {
        limit += 2;
    }
    if flag_val(state, player, "u05_mod:lingshi") > 0 {
        limit += 2;
    }
    if flag_val(state, player, "dddqiahua_has_xunxun") > 0 {
        limit += 1;
    }
    if flag_val(state, player, "th_yingshi_hermit") > 0 {
        limit += 2;
    }
    if flag_val(state, player, "l10_scsyaozhuo_skip_draw") > 0 {
        limit -= 1;
    }
    limit += d34_u01_mods::u01_xiaoxing_bonus(state, catalog, player);
    limit += flag_val(state, player, "u05_mod:tanlin3") as i32;
    let limit = super::d34_l28_mods::apply_shengong_hand_limit(state, catalog, player, limit);
    let limit = d34_u05_mods::apply_hand_limit(state, player, limit);
    d34_l11_mods::apply_hand_limit(state, catalog, player, limit)
}

pub fn apply_attack_range(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    mut cur: i32,
) -> i32 {
    cur += flag_val(state, player, "l27_qimou_range_bonus") as i32;
    if has_ext_skill(state, catalog, player, "ssr_huchi") {
        cur += 1;
    }
    let l11_range = flag_val(state, player, "l11_buff_attack_range_plus3");
    if l11_range > 0 {
        cur += l11_range as i32;
    } else if has_ext_skill(state, catalog, player, "buff_txhj_changbingguangren")
        || has_ext_skill(state, catalog, player, "_buff_txhj_changbingguangren")
        || has_ext_skill(state, catalog, player, "txhj_changbingguangren")
    {
        cur += 3;
    }
    if (has_ext_skill(state, catalog, player, "hanqiang")
        || flag_val(state, player, "yxs_hanqiang_range") > 0)
        && !has_weapon(state, catalog, player)
    {
        cur += 1;
    }
    if flag_val(state, player, "u05_mod:shihuawuqi") > 0 {
        cur += 2;
    }
    cur += d34_u01_mods::u01_xiaoxing_bonus(state, catalog, player);
    if flag_val(state, player, "u05_mod:mailun5") > 0 {
        cur += 1;
    }
    let cur = super::d34_l28_mods::apply_shengong_attack_range(state, catalog, player, cur);
    let cur = d34_u05_mods::apply_attack_range(state, player, cur);
    d34_l11_mods::apply_attack_range(state, catalog, player, cur)
}

pub fn apply_distance_total(
    state: &GameStateCore,
    catalog: &Catalog,
    a: PlayerId,
    b: PlayerId,
    mut cur: i32,
) -> i32 {
    if a == b {
        return cur;
    }
    if has_ext_skill(state, catalog, b, "ssr_biyue") {
        let pb = state.player(b);
        if pb.hand.is_empty() {
            cur += 1;
        }
        if pb.equips.is_empty() {
            cur += 1;
        }
        if pb.judge.is_empty() {
            cur += 1;
        }
    }
    if has_ext_skill(state, catalog, a, "ssr_newyicong") {
        let target_has_mount = has_slot_equipped(state, catalog, b, |s| {
            matches!(s, EquipSlot::HorsePlus | EquipSlot::HorseMinus)
        });
        cur -= if target_has_mount { 99 } else { 1 };
    }
    cur -= flag_val(state, a, "kangyin2_bonus_applied") as i32;
    cur -= flag_val(state, a, "l27_xiefang_distance_mod") as i32;
    if (has_ext_skill(state, catalog, a, "xiangma")
        || flag_val(state, a, "yxs_xiangma_mounted") > 0)
        && state.player(a).hp > 2
    {
        cur -= 1;
    }
    if (has_ext_skill(state, catalog, b, "xiangma")
        || flag_val(state, b, "yxs_xiangma_mounted") > 0)
        && state.player(b).hp <= 2
    {
        cur += 1;
    }
    if flag_val(state, a, "th_zhongzhuang_mounted") > 0
        || flag_val(state, a, "u05_cross_mod:mashu") > 0
        || flag_val(state, a, "u05_cross_mod:yicong") > 0
        || flag_val(state, a, "u05_mod:jifeng") > 0
        || flag_val(state, a, "u05_mod:ljifeng") > 0
    {
        cur -= 1;
    }
    if flag_val(state, b, "th_zhongzhuang_mounted") > 0
        || flag_val(state, b, "u05_cross_mod:feiying") > 0
        || flag_val(state, b, "u05_mod:heilonglinpian") > 0
        || flag_val(state, b, "u05_mod:sliufeng") > 0
    {
        cur += 1;
    }
    let cur = d34_u05_mods::apply_distance_total(state, catalog, a, b, cur);
    d34_l11_mods::apply_distance_total(state, catalog, a, b, cur)
}

pub fn apply_sha_use_limit(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    card: Option<CardId>,
    mut cur: i32,
    mut unlimited: bool,
) -> (i32, bool) {
    let zhuge_plus = flag_val(state, player, "l11_txhj_zhuge_sha_plus3");
    if zhuge_plus > 0 {
        cur += zhuge_plus as i32;
    } else if has_ext_skill(state, catalog, player, "txhj_zhuge") {
        cur += 3;
    }
    cur += if has_ext_skill(state, catalog, player, "hgkurou_effect") {
        1
    } else {
        flag_val(state, player, "l11_hgkurou_sha_plus1") as i32
    };
    cur += if has_ext_skill(state, catalog, player, "txhj_aHaoSkill3") {
        state.player(player).equips.len() as i32
    } else {
        flag_val(state, player, "l11_ahao_equip_bonus") as i32
    };
    cur += flag_val(state, player, "yxs_guimian_sha_limit_bonus") as i32;
    cur += flag_val(state, player, "dddfuyi_qun_sha_bonus") as i32;
    cur += d34_u01_mods::u01_fuyi_sha_bonus(state, catalog, player);
    cur += flag_val(state, player, "l27_qimou_range_bonus") as i32;
    cur += flag_val(state, player, "l27_potzhuangshi_unblockable") as i32;
    if flag_val(state, player, "l27_jinghe_shared") > 0
        || flag_val(state, player, "l27_bolan_temp_skill") > 0
        || flag_val(state, player, "l27_duorui_stolen_skill") > 0
    {
        cur += 1;
    }
    if has_ext_skill(state, catalog, player, "ssr_paoxiao")
        || flag_val(state, player, "l25_paoxiao_unlimited") > 0
        || flag_val(state, player, "l19_baozou_sha_unlimited") > 0
        || flag_val(state, player, "l19_paoxiao_unlimited_sha") > 0
        || flag_val(state, player, "l27_kuangcai_unlimited") > 0
    {
        unlimited = true;
    }
    let suit = card_suit(catalog, card);
    let is_red = matches!(suit, Some(Suit::Heart | Suit::Diamond));
    if is_red
        && (has_ext_skill(state, catalog, player, "wumu")
            || flag_val(state, player, "yxs_wumu_active") > 0
            || flag_val(state, player, "l27_sbzhaxiang_red_sha") > 0)
    {
        unlimited = true;
    }
    if suit == Some(Suit::Diamond) && flag_val(state, player, "l20_wusheng_diamond_unlimited") > 0 {
        unlimited = true;
    }
    if super::d34_l28_mods::bushi_unlimited_card(state, catalog, player, card) {
        unlimited = true;
    }
    if has_ext_skill(state, catalog, player, "ssr_jijiang")
        || flag_val(state, player, "l25_jijiang_sha_plus1") > 0
    {
        cur += 1;
    }
    if has_ext_skill(state, catalog, player, "ssr_weilong") {
        cur += 1;
    }
    if has_ext_skill(state, catalog, player, "ssr_huchi") {
        cur += 1;
    }
    for k in [
        "u05_mod:yuanbaorou",
        "u05_mod:fengnu",
        "u05_mod:feidan",
        "u05_mod:duijue2",
        "u05_buff:chilongya2",
        "u05_buff:chongzhuang",
        "u05_buff:aqianghua2",
    ] {
        if flag_val(state, player, k) > 0 {
            cur += 1;
        }
    }
    cur = super::d34_l28_mods::apply_shengong_sha_limit(state, catalog, player, cur);
    let (cur, unlimited) = d34_u05_mods::apply_sha_use_limit(state, player, cur, unlimited);
    d34_l11_mods::apply_sha_use_limit(state, catalog, player, cur, unlimited)
}

pub fn apply_sha_extra_targets(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    kind: CardKind,
    mut extra: i32,
) -> i32 {
    if kind.is_sha_like() {
        let fangtian = flag_val(state, player, "l11_fangtian_extra_targets");
        if fangtian > 0 {
            extra += fangtian as i32;
        } else if has_ext_skill(state, catalog, player, "txhj_fangtianhuaji")
            && state.turn.phase == crate::event::Phase::Play
            && flag_val(state, player, "l11_phase_sha_count") == 0
        {
            extra += 2;
        }
        extra += flag_val(state, player, "kangyin2_bonus_applied") as i32;
        extra += flag_val(state, player, "ddd_xianxi_target_added") as i32;
        if flag_val(state, player, "u05_mod:gwchuanxin") > 0
            || flag_val(state, player, "u05_mod:shewu") > 0
        {
            extra += 1;
        }
    }
    let extra = d34_u05_mods::apply_sha_extra_targets(state, catalog, player, kind, extra);
    d34_l11_mods::apply_sha_extra_targets(state, catalog, player, kind, extra)
}

pub fn apply_card_distance_unlimited(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    card: CardId,
    kind: CardKind,
    unlimited: bool,
) -> bool {
    apply_card_distance_unlimited_to(state, catalog, player, None, card, kind, unlimited)
}

pub fn apply_card_distance_unlimited_to(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    target: Option<PlayerId>,
    card: CardId,
    kind: CardKind,
    mut unlimited: bool,
) -> bool {
    let is_trick = !kind.is_basic_kind() && !kind.is_equip();
    if is_trick
        && (has_ext_skill(state, catalog, player, "ssr_qicai")
            || flag_val(state, player, "l11_buff_trick_no_distance") > 0
            || has_ext_skill(state, catalog, player, "buff_txhj_yueyingdezhihui")
            || has_ext_skill(state, catalog, player, "_buff_txhj_yueyingdezhihui")
            || has_ext_skill(state, catalog, player, "txhj_yueyingdezhihui"))
    {
        unlimited = true;
    }
    if flag_val(state, player, "l25_jiwu_boost") > 0
        || flag_val(state, player, "l27_kuangcai_unlimited") > 0
    {
        unlimited = true;
    }
    if kind.is_sha_like() {
        if has_ext_skill(state, catalog, player, "sheshu")
            || flag_val(state, player, "yxs_sheshu_pierce") > 0
        {
            unlimited = true;
        }
        let suit = card_suit(catalog, Some(card));
        if matches!(suit, Some(Suit::Spade | Suit::Club))
            && (has_ext_skill(state, catalog, player, "wumu")
                || flag_val(state, player, "yxs_wumu_active") > 0)
        {
            unlimited = true;
        }
        if suit == Some(Suit::Diamond)
            && (has_ext_skill(state, catalog, player, "fz_new_rewusheng")
                || flag_val(state, player, "l20_wusheng_diamond_unlimited") > 0
                || flag_val(state, player, "fz_new_rewusheng_unlimited") > 0)
        {
            unlimited = true;
        }
    }
    let unlimited = d34_u05_mods::apply_card_distance_unlimited(state, player, kind, unlimited);
    d34_l11_mods::apply_card_distance_unlimited_to(
        state, catalog, player, target, card, kind, unlimited,
    )
}

pub fn apply_card_target_canceled(
    state: &GameStateCore,
    catalog: &Catalog,
    actor: PlayerId,
    target: PlayerId,
    card: CardId,
    kind: CardKind,
    mut canceled: bool,
) -> bool {
    let is_trick = !kind.is_basic_kind() && !kind.is_equip();
    if actor != target
        && (has_ext_skill(state, catalog, target, "ssr_newjiahe")
            || flag_val(state, target, "l25_jiahe_qianxing") > 0
            || flag_val(state, target, "th_tiansuan2_immune") > 0
            || flag_val(state, target, "l20_diaohulishan_exiled") > 0)
    {
        canceled = true;
    }
    if matches!(kind, CardKind::Snatch | CardKind::Le)
        && flag_val(state, target, "u05_cross_mod:qianxun") > 0
    {
        canceled = true;
    }
    if kind.is_sha_like()
        && (has_ext_skill(state, catalog, target, "xianqu")
            || flag_val(state, target, "yxs_xianqu_ward") > 0)
    {
        let rank = catalog.cards.get(card as usize).map_or(0, |d| d.rank);
        if rank > 0 && rank < 8 {
            canceled = true;
        }
    }
    if (kind.is_sha_like() || is_trick)
        && (has_ext_skill(state, catalog, target, "chajue2")
            || flag_val(state, target, "chajue2_active") > 0
            || flag_val(state, target, "yxs_chajue_immune") > 0)
    {
        canceled = true;
    }
    // 英雄杀/character/skill.js:2709-2717 (`nvquan3`: `player.sex === "male"`)
    if kind == CardKind::Duel
        && actor != target
        && (has_ext_skill(state, catalog, target, "nvquan3")
            || has_ext_skill(state, catalog, target, "nvquan")
            || flag_val(state, target, "nvquan3_active") > 0)
        && state.matches_gender(
            catalog,
            actor,
            crate::vocabulary::generals::Gender::Male,
        )
    {
        canceled = true;
    }
    if has_ext_skill(state, catalog, target, "kongju")
        || flag_val(state, target, "yxs_kongju_ward") > 0
    {
        let hlen = state.player(target).hand.len();
        let mhp = state.player(target).max_hp as usize;
        if hlen < mhp && matches!(kind, CardKind::Snatch | CardKind::Dismantle) {
            canceled = true;
        } else if hlen > mhp && kind == CardKind::Le {
            canceled = true;
        }
    }
    let is_black = matches!(card_suit(catalog, Some(card)), Some(Suit::Spade | Suit::Club));
    if is_black && flag_val(state, target, "dddanzhi_black_ward") > 0 {
        canceled = true;
    }
    if actor != target && is_black && flag_val(state, target, "u05_mod:molicha") > 0 {
        canceled = true;
    }
    if actor != target
        && (kind.is_sha_like() || is_trick)
        && flag_val(state, target, "xianjian_ward_active") > 0
    {
        canceled = true;
    }
    if matches!(
        kind,
        CardKind::Le | CardKind::Bing | CardKind::ShanDian | CardKind::HongShui | CardKind::Guiyoujie
    ) && (flag_val(state, target, "l11_buff_no_delayed_trick") > 0
        || has_ext_skill(state, catalog, target, "buff_txhj_wenhejibei")
        || has_ext_skill(state, catalog, target, "_buff_txhj_wenhejibei")
        || has_ext_skill(state, catalog, target, "txhj_wenhejibei")
        || has_ext_skill(state, catalog, target, "xinfu_pdgyingshi"))
    {
        canceled = true;
    }
    if actor != target
        && is_trick
        && (flag_val(state, target, "l11_jiuwei_trick_nullify") > 0
            || flag_val(state, target, "l11_armor_guard:txhj_guofengyupao") > 0
            || flag_val(state, target, "l11_armor_guard:guofengyupao") > 0
            || flag_val(state, target, "l11_trick_guard:txhj_ruiSkill3") > 0
            || flag_val(state, target, "l11_trick_guard:txhj_yanSkill1") > 0
            || flag_val(state, target, "l11_trick_guard:txhj_xuanwuSkill3") > 0
            || flag_val(state, target, "l27_dinghan_negated") > 0)
    {
        canceled = true;
    }
    if actor != target
        && kind.is_sha_like()
        && (flag_val(state, target, "l20_shouxi_nullified") > 0
            || flag_val(state, target, "l27_shouxi_nullified") > 0
            || flag_val(state, target, "yxs_menshen_active") > 0)
    {
        canceled = true;
    }
    if super::d34_l28_mods::guofengyupao_cancels_target(state, catalog, actor, target, card, kind)
    {
        canceled = true;
    }
    let canceled = d34_u05_mods::apply_card_target_canceled(state, actor, target, kind, canceled);
    d34_l11_mods::apply_card_target_canceled(state, catalog, actor, target, Some(card), kind, canceled)
}

#[inline]
pub fn is_ducai3_forbidden(
    state: &GameStateCore,
    catalog: &Catalog,
    actor: PlayerId,
    card: CardId,
) -> bool {
    if d34_u01_mods::is_dddlianer_forbidden(state, catalog, actor, card) {
        return true;
    }
    if (card as usize) < catalog.cards.len()
        && flag_val(state, actor, "l25_gongxin_red_sealed") > 0
        && matches!(
            catalog.effective_card_suit(state, actor, card),
            Suit::Heart | Suit::Diamond
        )
    {
        return true;
    }
    if (card as usize) >= catalog.cards.len() {
        return false;
    }
    if state.player(actor).hand.contains(&card)
        && has_ext_skill(state, catalog, actor, "mdqianxi72")
    {
        let color_flag = flag_val(state, actor, "l11_mdqianxi72_color");
        let is_red = matches!(
            catalog.effective_card_suit(state, actor, card),
            Suit::Heart | Suit::Diamond
        );
        if (color_flag == 1 && is_red) || (color_flag == 2 && !is_red) {
            return true;
        }
    }
    if has_ext_skill(state, catalog, actor, "ducai2")
        || flag_val(state, actor, "ducai2_active") > 0
    {
        return false;
    }
    let sc = suit_code(catalog.effective_card_suit(state, actor, card));
    state.alive_players().any(|p| {
        p != actor
            && (flag_val(state, p, "yxs_ducai_forbidden_suit") == sc
                || flag_val(state, p, "ducai_suit") == sc)
    })
}

pub fn apply_card_use_canceled(
    state: &GameStateCore,
    catalog: &Catalog,
    actor: PlayerId,
    card: CardId,
    kind: CardKind,
    in_hand: bool,
    mut canceled: bool,
) -> bool {
    if flag_val(state, actor, "l11_xingtian_card_locked") > 0
        || flag_val(state, actor, "l25_yijue_locked") > 0
        || flag_val(state, actor, "u05_mod:shihuifen") > 0
    {
        canceled = true;
    }
    if kind.is_sha_like()
        && (flag_val(state, actor, "l25_zhenhun_forbid_sha") > 0
            || flag_val(state, actor, "u05_mod:gw_birinongwu") > 0
            || (in_hand && flag_val(state, actor, "yxs_yaoji_lebu_placed") > 0))
    {
        canceled = true;
    }
    if in_hand
        && flag_val(state, actor, "l27_duorui_sealed") > 0
        && !kind.is_basic_kind()
    {
        canceled = true;
    }
    if flag_val(state, actor, "mbcmfangzhu_ban") > 0 && !kind.is_trick_kind() {
        canceled = true;
    }
    let zhixi_cap = flag_val(state, actor, "l27_new_zhixi_cap");
    if in_hand && zhixi_cap > 0 && state.player(actor).hand.len() as u32 > zhixi_cap {
        canceled = true;
    }
    if in_hand
        && matches!(kind, CardKind::Tao | CardKind::Jiu)
        && flag_val(state, actor, "th_tiansuan2_4_forbid_tao_jiu") > 0
    {
        canceled = true;
    }
    let suit = card_suit(catalog, Some(card));
    if in_hand && suit == Some(Suit::Heart) && flag_val(state, actor, "l25_gongxin_heart_sealed") > 0
    {
        canceled = true;
    }
    let liduan_suit = flag_val(state, actor, "wdb_liduan_suit_locked");
    if in_hand && liduan_suit > 0 {
        if suit.is_some_and(|s| suit_code(s) == liduan_suit || liduan_suit == 5) {
            canceled = true;
        }
    }
    if is_ducai3_forbidden(state, catalog, actor, card) {
        canceled = true;
    }
    let cur_turn = state.turn.current;
    if in_hand
        && ((cur_turn != actor
            && state.player(cur_turn).alive
            && !state.player(cur_turn).hand.is_empty()
            && flag_val(state, cur_turn, "dddfusi_all_shown") > 0)
            || is_dddfusi_hand_locked(state, catalog, actor))
    {
        canceled = true;
    }
    let canceled =
        d34_u05_mods::apply_card_use_canceled(state, catalog, actor, card, kind, in_hand, canceled);
    d34_l11_mods::apply_card_use_canceled(state, catalog, actor, kind, in_hand, canceled)
}

pub fn apply_dying_peach_target_canceled(
    state: &GameStateCore,
    catalog: &Catalog,
    actor: PlayerId,
    dying_target: PlayerId,
    card: CardId,
    kind: CardKind,
    mut canceled: bool,
) -> bool {
    if catalog.has_diaohulishan(state, dying_target) || catalog.has_diaohulishan(state, actor) {
        canceled = true;
    }
    if kind == CardKind::Tao
        && (flag_val(state, actor, "aozhan_active") > 0
            || has_ext_skill(state, catalog, actor, "aozhan"))
    {
        canceled = true;
    }
    if actor != dying_target && kind == CardKind::Tao {
        for p in state.alive_players() {
            if p != actor
                && (flag_val(state, p, "l19_wansha_forbid_ally_tao") > 0
                    || flag_val(state, p, "l19_guixin_wansha") > 0)
            {
                canceled = true;
                break;
            }
        }
    }
    if is_ducai3_forbidden(state, catalog, actor, card) {
        canceled = true;
    }
    d34_u05_mods::apply_dying_peach_target_canceled(state, actor, dying_target, canceled)
}

pub fn apply_damage_amount(
    state: &GameStateCore,
    catalog: &Catalog,
    source: Option<PlayerId>,
    target: PlayerId,
    card: Option<CardId>,
    nature: DamageNature,
    mut cur: i32,
) -> i32 {
    if cur <= 0 {
        return 0;
    }
    if has_ext_skill(state, catalog, target, "wluoyan")
        || flag_val(state, target, "wluoyan_active") > 0
    {
        return 0;
    }
    let ckind = card.map(|c| catalog.card_kind(c));
    let is_sha_card = ckind.is_some_and(CardKind::is_sha_like);
    if let Some(src) = source {
        if flag_val(state, src, "l19_wujin_damage_amp") > 0 {
            cur += 1;
        }
        if flag_val(state, src, "l20_lingren_dmg_plus") > 0 {
            cur += 1;
        }
        if (is_sha_card || card.is_none()) && flag_val(state, src, "l19_xunmeng_damage_bonus") > 0 {
            cur += 1;
        }
        if has_ext_skill(state, catalog, src, "th_zhongzhuang") {
            let atk_range = state.attack_range(catalog, src);
            if atk_range < 3 {
                cur += 1;
            } else if atk_range > 3 && cur > 1 {
                cur = 1;
            }
        } else if is_sha_card && flag_val(state, src, "th_zhongzhuang_damage_bonus") > 0 {
            cur += 1;
        }
        if is_sha_card {
            if flag_val(state, src, "l11_lulu_huwei_boost") > 0 {
                cur += 1;
            }
            if flag_val(state, src, "l20_liegong_damage_boost") > 0 {
                cur += 1;
            }
            if flag_val(state, src, "l10_scschihe_sha_boost") > 0 {
                cur += 1;
            }
        }
    }

    if has_ext_skill(state, catalog, target, "ssr_manjia") {
        if matches!(ckind, Some(CardKind::SavageAttack | CardKind::ArcheryAttack))
            || (matches!(ckind, Some(CardKind::Sha)) && nature == DamageNature::Normal)
        {
            return 0;
        }
        if nature == DamageNature::Fire {
            cur += 1;
        }
    }
    if flag_val(state, target, "l20_qingyu_prevented") > 0
        || flag_val(state, target, "l20_kuanshi_ward") > 0
        || flag_val(state, target, "l11_yaya2_protected") > 0
        || flag_val(state, target, "th_tiansuan2_0_prevent_damage") > 0
    {
        return 0;
    }
    if cur > 1
        && (flag_val(state, target, "th_tiansuan2_1_cap_damage") > 0
            || flag_val(state, target, "th_tiansuan2_2_cap_damage") > 0)
    {
        cur = 1;
    }
    cur = super::d34_l28_mods::rewrite_armor_damage_mod(
        state, catalog, source, target, card, nature, cur,
    );
    if nature == DamageNature::Fire && flag_val(state, target, "l11_armor_guard:txminguangkai") > 0 {
        return 0;
    }
    if nature == DamageNature::Fire && flag_val(state, target, "l11_buff_fire_shield") > 0 {
        cur = (cur - 1).max(0);
    }
    if nature == DamageNature::Thunder && flag_val(state, target, "l11_buff_thunder_shield") > 0 {
        cur = (cur - 1).max(0);
    }
    if d34_l11_mods::tpys_damage_prevented(state, catalog, source, target, card, nature) {
        return 0;
    }
    if flag_val(state, target, "u05_cross_mod:xinwuyan") > 0 {
        if let Some(k) = ckind {
            if !k.is_basic_kind() && !k.is_equip() {
                return 0;
            }
        }
    }
    if flag_val(state, target, "u05_cross_mod:jueqing") > 0 {
        return cur;
    }
    let moyan_rec = flag_val(state, target, "dddmoyan_recorded_count");
    if moyan_rec > 0 {
        let moyan_t = flag_val(state, target, "dddmoyan_target") as u8;
        if (moyan_t as usize) < state.players.len()
            && state.player(moyan_t).hand.len() <= moyan_rec as usize
        {
            cur += 1;
        }
    }
    if flag_val(state, target, "l25_tianxiang_deflected") > 0 {
        cur = (cur - 1).max(0);
    }
    let zuhuo = flag_val(state, target, "l27_twzuhuo_shield");
    if zuhuo > 0 {
        cur = (cur - zuhuo as i32).max(0);
    }
    if flag_val(state, target, "l11_armor_guard:txhj_chongyingshenfu") > 0 {
        cur = (cur - 1).max(0);
    }
    if cur > 1
        && (flag_val(state, target, "l11_dundun_zhongzhi_reduce") > 0
            || flag_val(state, target, "u05_buff:bingjia") > 0
            || flag_val(state, target, "u05_buff:bingjia2") > 0
            || flag_val(state, target, "u05_buff:cyxianjiang") > 0
            || flag_val(state, target, "u05_buff:dangping") > 0
            || (is_sha_card && flag_val(state, target, "u05_buff:gudonggeng") > 0))
    {
        cur -= 1;
    }
    let cur = d34_u05_mods::apply_damage_amount(state, catalog, source, target, card, nature, cur);
    d34_l11_mods::apply_damage_amount(state, catalog, source, target, card, nature, cur)
}

pub fn is_target_unequip2(state: &GameStateCore, target: PlayerId) -> bool {
    flag_val(state, target, "l11_xingtian_armor_null") > 0
        || flag_val(state, target, "l11_noda_axe2_turn") > 0
        || flag_val(state, target, "l25_yijue_locked") > 0
        || flag_val(state, target, "l25_pojia_broken") > 0
        || flag_val(state, target, "l25_hubu_unequip2") > 0
        || flag_val(state, target, "chixueqingfeng2_count") > 0
}

pub fn apply_ignore_armor(
    state: &GameStateCore,
    catalog: &Catalog,
    source: PlayerId,
    target: PlayerId,
    ignore: bool,
) -> bool {
    apply_ignore_armor_with_card(state, catalog, source, target, None, ignore)
}

pub fn apply_ignore_armor_with_card(
    state: &GameStateCore,
    catalog: &Catalog,
    source: PlayerId,
    target: PlayerId,
    card: Option<CardId>,
    mut ignore: bool,
) -> bool {
    if is_target_unequip2(state, target)
        || has_ext_skill(state, catalog, source, "ssr_pojia")
        || has_ext_skill(state, catalog, source, "ssr_hubu")
        || flag_val(state, source, "l25_pojia_ignore_armor") > 0
        || (has_ext_skill(state, catalog, source, "fz_gzpaoxiao")
            && (has_ext_skill(state, catalog, source, "shouyue")
                || flag_val(state, source, "shouyue_active") > 0))
        || flag_val(state, source, "l27_twxiafeng_spent") > 0
    {
        ignore = true;
    }
    if state.turn.current == source
        && state.turn.phase == crate::event::Phase::Play
        && !state.is_friend_of(catalog, source, target)
        && (has_weapon(state, catalog, source)
            || d34_l11_mods::has_equipped_named(state, catalog, source, "txhj_feilong"))
        && (flag_val(state, source, "l11_weapon_pozhenzhifeng") > 0
            || has_ext_skill(state, catalog, source, "txhj_pozhenzhifeng")
            || has_ext_skill(state, catalog, source, "buff_txhj_pozhenzhifeng")
            || has_ext_skill(state, catalog, source, "_buff_txhj_pozhenzhifeng"))
    {
        ignore = true;
    }
    let ignore = d34_u05_mods::apply_ignore_armor(state, source, ignore);
    let ignore = d34_l11_mods::apply_ignore_armor(state, catalog, source, target, card, ignore);
    super::d34_l28_mods::apply_ignore_armor(state, catalog, target, ignore)
}
