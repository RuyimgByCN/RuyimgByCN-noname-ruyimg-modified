//! Explicit per-skill `U05 杀海拾遗` modifier evaluations (`d34_u05_mods.rs`), extracted from
//! `archive/upstream/noname/apps/core/extension/杀海拾遗/{card,character}/*/skill.js`.
//! Zero blanket prefix catch-alls: every rule checks its exact skill/flag and upstream condition.

use crate::catalog::{CardKind, Catalog, EquipSlot, Suit};
use crate::event::DamageNature;
use crate::ids::{CardId, PlayerId};
use crate::state::GameStateCore;

fn flag_val(state: &GameStateCore, seat: PlayerId, key: &str) -> u32 {
    state
        .vm_aux
        .flags
        .get(&(seat, key.to_string()))
        .copied()
        .unwrap_or(0)
}

fn has_armor(state: &GameStateCore, catalog: &Catalog, seat: PlayerId) -> bool {
    state
        .player(seat)
        .equips
        .iter()
        .any(|&cid| catalog.card_slot_of(state, cid) == Some(EquipSlot::Armor))
}

pub(super) fn is_card_ignored_hand_limit(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    card: CardId,
) -> bool {
    let kind = catalog.card_kind(card);
    // card/gujian/skill.js:151-177 (`chunbing`) & card/mtg/skill.js:32-43 (`mtg_shamolvzhou_skill`)
    if kind.is_basic_kind()
        && (flag_val(state, player, "u05_mod:chunbing") > 0
            || flag_val(state, player, "u05_mod:mtg_shamolvzhou_skill") > 0)
    {
        return true;
    }
    // card/gujian/skill.js:343-371 (`yuanbaorou`): Sha ignored in hand limit
    if kind.is_sha_like() && flag_val(state, player, "u05_mod:yuanbaorou") > 0 {
        return true;
    }
    false
}

pub(super) fn apply_hand_limit(state: &GameStateCore, player: PlayerId, mut limit: i32) -> i32 {
    let mut delta: i32 = 0;
    // card/hearth/skill.js:14-20 (`jihuocard2`)
    if flag_val(state, player, "u05_mod:jihuocard2") > 0 {
        delta += 2;
    }
    // character/hearth/skill.js:5312-5321 (`xjumo`)
    if flag_val(state, player, "u05_mod:xjumo") > 0 {
        if state.player(player).hp < state.player(player).max_hp as i32 {
            delta += 5;
        } else {
            delta += 3;
        }
    }
    // character/hearth/skill.js:7104-7110 (`tuteng_h`)
    if flag_val(state, player, "u05_mod:tuteng_h") > 0 {
        delta -= 1;
    }
    // card/swd/skill.js:5-12 (`qiankundai`)
    if flag_val(state, player, "u05_mod:qiankundai") > 0 {
        delta += 1;
    }
    // card/swd/skill.js:1738-1749 (`hslingjian_xingtigaizao_equip5`)
    if flag_val(state, player, "u05_mod:hslingjian_xingtigaizao_equip5") > 0 {
        delta -= 1;
    }
    // card/swd/skill.js:2191-2212 (`hslingjian_xingtigaizao`)
    let xtg = flag_val(state, player, "u05_mod:hslingjian_xingtigaizao");
    if xtg > 0 {
        delta -= xtg as i32;
    }
    // character/swd/skill.js:7727-7739 (`mailun6`)
    if flag_val(state, player, "u05_mod:mailun6") > 0 {
        delta -= 1;
    }
    // character/ow/skill.js:1975-2016 (`jijia`: +4 mech HP hand limit bonus)
    let jijia = flag_val(state, player, "u05_mod:jijia");
    if jijia > 0 {
        delta += jijia.max(4) as i32;
    }
    limit += delta;
    limit
}

pub(super) fn apply_attack_range(state: &GameStateCore, player: PlayerId, mut range: i32) -> i32 {
    // character/ow/skill.js:941-963 (`bfengshi`: attackFrom - countUsed())
    if flag_val(state, player, "u05_mod:bfengshi") > 0 {
        range += state.turn.cards_used_play_phase.max(1) as i32;
    }
    range
}

pub(super) fn apply_distance_total(
    state: &GameStateCore,
    catalog: &Catalog,
    from: PlayerId,
    to: PlayerId,
    mut dist: i32,
) -> i32 {
    // character/hearth/skill.js:7251-7262 (`tuteng8`) & card/swd/skill.js:1418-1424
    if flag_val(state, from, "u05_mod:tuteng8") > 0 {
        dist -= 1;
    }
    if flag_val(state, from, "u05_mod:hslingjian_xuanfengzhiren_equip4_dist") > 0 {
        dist -= 1;
    }
    if flag_val(state, to, "u05_mod:hslingjian_xuanfengzhiren_equip3_dist") > 0 {
        dist += 1;
    }
    // character/ow/skill.js:287-300, 562-578, 1610-1626, 1961-1974 (-Infinity distance)
    if flag_val(state, from, "u05_mod:feitiao2") > 0
        || flag_val(state, from, "u05_mod:tuijin2") > 0
        || flag_val(state, from, "u05_mod:juji3") > 0
        || flag_val(state, to, "u05_mod:liangou2") > 0
    {
        dist -= 99;
    }
    // character/ow/skill.js:1805-1856 (`bingqiang2..5`)
    dist += flag_val(state, to, "u05_mod:bingqiang2") as i32;
    dist -= flag_val(state, to, "u05_mod:bingqiang3") as i32;
    dist -= flag_val(state, from, "u05_mod:bingqiang4") as i32;
    dist += flag_val(state, from, "u05_mod:bingqiang5") as i32;
    // character/ow/skill.js:2163-2171 (`tuji`: globalFrom - countUsed() on own turn)
    if state.turn.current == from && flag_val(state, from, "u05_mod:tuji") > 0 {
        dist -= state.turn.cards_used_play_phase.max(1) as i32;
    }
    // card/swd/skill.js:1483-1500 (`hslingjian_zhongxinghujia_equip3/4`: armor check)
    if flag_val(state, to, "u05_mod:hslingjian_zhongxinghujia_equip3") > 0
        && has_armor(state, catalog, to)
    {
        dist += 1;
    }
    if flag_val(state, from, "u05_mod:hslingjian_zhongxinghujia_equip4") > 0
        && has_armor(state, catalog, from)
    {
        dist -= 1;
    }
    // card/swd/skill.js:1576-1593 (`hslingjian_jinjilengdong_equip3/4`: turned-over check)
    if flag_val(state, to, "u05_mod:hslingjian_jinjilengdong_equip3") > 0
        && state.player(to).face_down
    {
        dist += 2;
    }
    if flag_val(state, from, "u05_mod:hslingjian_jinjilengdong_equip4") > 0
        && state.player(from).face_down
    {
        dist -= 2;
    }
    // card/swd/skill.js:1659-1676 (`hslingjian_yinmilichang_equip3/4`: hp == 1 check)
    if flag_val(state, to, "u05_mod:hslingjian_yinmilichang_equip3") > 0
        && state.player(to).hp == 1
    {
        dist += 1;
    }
    if flag_val(state, from, "u05_mod:hslingjian_yinmilichang_equip4") > 0
        && state.player(from).hp == 1
    {
        dist -= 1;
    }
    // card/swd/skill.js:1718-1737 (`hslingjian_xingtigaizao_equip3/4`)
    if flag_val(state, from, "u05_mod:hslingjian_xingtigaizao_equip3") > 0 {
        dist += 1;
    }
    if flag_val(state, to, "u05_mod:hslingjian_xingtigaizao_equip3") > 0 {
        dist += 1;
    }
    if flag_val(state, from, "u05_mod:hslingjian_xingtigaizao_equip4") > 0 {
        dist -= 1;
    }
    if flag_val(state, to, "u05_mod:hslingjian_xingtigaizao_equip4") > 0 {
        dist -= 1;
    }
    // card/swd/skill.js:1769-1786 (`hslingjian_shengxiuhaojiao_equip3/4`: hp < hand.len())
    if flag_val(state, to, "u05_mod:hslingjian_shengxiuhaojiao_equip3") > 0
        && state.player(to).hp < state.player(to).hand.len() as i32
    {
        dist += 1;
    }
    if flag_val(state, from, "u05_mod:hslingjian_shengxiuhaojiao_equip4") > 0
        && state.player(from).hp < state.player(from).hand.len() as i32
    {
        dist -= 1;
    }
    // card/swd/skill.js:1836-1853 (`hslingjian_shijianhuisu_equip3/4`: equips.len() == 1)
    if flag_val(state, to, "u05_mod:hslingjian_shijianhuisu_equip3") > 0
        && state.player(to).equips.len() == 1
    {
        dist += 1;
    }
    if flag_val(state, from, "u05_mod:hslingjian_shijianhuisu_equip4") > 0
        && state.player(from).equips.len() == 1
    {
        dist -= 1;
    }
    // character/swd/skill.js:7740-7759 (`mailun7` / `mailun8`) & card/zhenfa/card.js:123 (`fengyangzhen`)
    if flag_val(state, from, "u05_mod:mailun7") > 0 || flag_val(state, to, "u05_mod:mailun7") > 0 {
        dist -= 1;
    }
    if flag_val(state, from, "u05_mod:mailun8") > 0 || flag_val(state, to, "u05_mod:mailun8") > 0 {
        dist += 1;
    }
    if flag_val(state, to, "u05_mod:fengyangzhen") > 0 {
        dist += 1;
    }
    dist
}

pub(super) fn apply_sha_use_limit(
    state: &GameStateCore,
    player: PlayerId,
    mut cur: i32,
    mut unlimited: bool,
) -> (i32, bool) {
    // character/ow/skill.js:346-422 (`bshaowei`) & 2322-2330 (`zhanlong2`): unlimited Sha
    if flag_val(state, player, "u05_mod:bshaowei") > 0
        || flag_val(state, player, "u05_mod:zhanlong2") > 0
    {
        unlimited = true;
    }
    // character/xianjian/skill.js:1850-1858 (`yanzhan4`) & card/mtg/skill.js:211-222 (`mtg_longlushanfeng_skill`)
    if flag_val(state, player, "u05_mod:yanzhan4") > 0 {
        cur += 1;
    }
    if flag_val(state, player, "u05_mod:mtg_longlushanfeng_skill") > 0 {
        cur += 1;
    }
    (cur, unlimited)
}

pub(super) fn apply_sha_extra_targets(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    kind: CardKind,
    mut extra: i32,
) -> i32 {
    // character/ow/skill.js:2331-2355 (`feiren`: Club Sha +1 target)
    if kind.is_sha_like()
        && flag_val(state, player, "u05_mod:feiren") > 0
        && state.player(player).hand.iter().any(|&cid| {
            catalog
                .cards
                .get(cid as usize)
                .is_some_and(|c| c.kind.is_sha_like() && c.suit == Suit::Club)
        })
    {
        extra += 1;
    }
    extra
}

pub(super) fn apply_card_distance_unlimited(
    state: &GameStateCore,
    player: PlayerId,
    kind: CardKind,
    mut unlimited: bool,
) -> bool {
    if kind.is_sha_like()
        && (flag_val(state, player, "u05_mod:bshaowei") > 0
            || flag_val(state, player, "u05_mod:feiren") > 0)
    {
        unlimited = true;
    }
    if flag_val(state, player, "u05_mod:zhufangshenshi") > 0
        || flag_val(state, player, "u05_mod:tanlin4") > 0
    {
        unlimited = true;
    }
    unlimited
}

pub(super) fn apply_card_target_canceled(
    state: &GameStateCore,
    actor: PlayerId,
    target: PlayerId,
    kind: CardKind,
    mut canceled: bool,
) -> bool {
    // character/gujian/skill.js:1867-1875 (`gjqtwuxie`: immune to others' delayed tricks)
    if actor != target
        && matches!(
            kind,
            CardKind::Le | CardKind::Bing | CardKind::ShanDian | CardKind::HongShui
        )
        && flag_val(state, target, "u05_mod:gjqtwuxie") > 0
    {
        canceled = true;
    }
    // character/ow/skill.js:1880-1905 (`jidong2`) & 2643-2657 (`sheng2`): untargetable by others
    if actor != target
        && (flag_val(state, target, "u05_mod:jidong2") > 0
            || flag_val(state, target, "u05_mod:sheng2") > 0)
    {
        canceled = true;
    }
    // character/ow/skill.js:2702-2723 (`yihun2`) & card/swd/skill.js:2213-2228 (`hslingjian_jinjilengdong`)
    if flag_val(state, target, "u05_mod:yihun2") > 0
        || flag_val(state, target, "u05_mod:hslingjian_jinjilengdong") > 0
    {
        canceled = true;
    }
    // character/xianjian/skill.js:2560-2582 (`guijiang2`: cannot be targeted by Tao)
    if kind == CardKind::Tao && flag_val(state, target, "u05_mod:guijiang2") > 0 {
        canceled = true;
    }
    // card/swd/skill.js:1677-1697 (`hslingjian_yinmilichang_equip5`: empty hand immune to Sha/Duel)
    if state.player(target).hand.is_empty()
        && (kind.is_sha_like() || kind == CardKind::Duel)
        && flag_val(state, target, "u05_mod:hslingjian_yinmilichang_equip5") > 0
    {
        canceled = true;
    }
    // card/swd/skill.js:1760-1768 & 2157-2190 (`hslingjian_shengxiuhaojiao_equip2` / `hslingjian_chaofeng`)
    if flag_val(state, actor, "u05_mod:hslingjian_chaofeng") > 0
        && flag_val(state, target, "u05_mod:hslingjian_shengxiuhaojiao_equip2") > 0
    {
        canceled = true;
    }
    if kind.is_sha_like()
        && flag_val(state, actor, "u05_mod:hslingjian_chaofeng") == 0
        && flag_val(state, target, "u05_mod:hslingjian_chaofeng") == 0
        && state
            .alive_players()
            .any(|p| p != actor && flag_val(state, p, "u05_mod:hslingjian_chaofeng") > 0)
    {
        canceled = true;
    }
    canceled
}

pub(super) fn apply_card_use_canceled(
    state: &GameStateCore,
    catalog: &Catalog,
    actor: PlayerId,
    card: CardId,
    kind: CardKind,
    in_hand: bool,
    mut canceled: bool,
) -> bool {
    // character/gujian/skill.js:2459-2471 (`yuling4`: maxHp + 2 cards per turn)
    if state.turn.current == actor
        && flag_val(state, actor, "u05_mod:yuling4") > 0
        && state.turn.cards_used_play_phase >= state.player(actor).max_hp + 2
    {
        canceled = true;
    }
    // character/gwent/skill.js:2197-2234 (`hupeng2`: gw_dudayuanshuai2 unplayable)
    if flag_val(state, actor, "u05_mod:hupeng2") > 0
        && catalog
            .cards
            .get(card as usize)
            .is_some_and(|c| c.name.contains("gw_dudayuanshuai2"))
    {
        canceled = true;
    }
    // character/hearth/skill.js:1847-1855 (`lieyang2`) & 4601-4614 (`duzhang2`): trick cards locked
    let is_trick = !kind.is_basic_kind() && kind.equip_slot().is_none();
    if is_trick
        && (flag_val(state, actor, "u05_mod:lieyang2") > 0
            || flag_val(state, actor, "u05_mod:duzhang2") > 0)
    {
        canceled = true;
    }
    // character/hearth/skill.js:6779-6794 (`mengun2`) & character/ow/skill.js:1285-1311, 1415-1436, 2702-2723
    if flag_val(state, actor, "u05_mod:shanguang2") > 0
        || flag_val(state, actor, "u05_mod:yihun2") > 0
        || flag_val(state, actor, "u05_mod:hslingjian_jinjilengdong") > 0
    {
        canceled = true;
    }
    if in_hand
        && (flag_val(state, actor, "u05_mod:mianzhen2") > 0
            || flag_val(state, actor, "u05_mod:mengun2") > 0)
    {
        canceled = true;
    }
    canceled
}

pub(super) fn apply_dying_peach_target_canceled(
    state: &GameStateCore,
    actor: PlayerId,
    dying_target: PlayerId,
    mut canceled: bool,
) -> bool {
    // character/xianjian/skill.js:2583-2591 (`guijiang3` / `guijiang2`) & `shanguang2`
    if flag_val(state, actor, "u05_mod:shanguang2") > 0
        || flag_val(state, dying_target, "u05_mod:guijiang2") > 0
        || flag_val(state, dying_target, "u05_mod:guijiang3") > 0
    {
        canceled = true;
    }
    canceled
}

pub(super) fn apply_damage_amount(
    state: &GameStateCore,
    catalog: &Catalog,
    source: Option<PlayerId>,
    target: PlayerId,
    card: Option<CardId>,
    _nature: DamageNature,
    mut amount: i32,
) -> i32 {
    if amount <= 0 {
        return amount;
    }
    if let Some(src) = source {
        // character/hearth/skill.js:2523-2549 (`zhongji`: +1 damage)
        if flag_val(state, src, "u05_mod:zhongji_dmg") > 0 {
            amount += 1;
        }
        // character/ow/skill.js:2331-2355 (`feiren`: Spade Sha +1 damage)
        if flag_val(state, src, "u05_mod:feiren") > 0
            && card.and_then(|cid| catalog.cards.get(cid as usize)).is_some_and(|c| {
                c.kind.is_sha_like() && c.suit == Suit::Spade
            })
        {
            amount += 1;
        }
    }
    // character/ow/skill.js:562-578 (`liangou2`: +1 incoming damage)
    if flag_val(state, target, "u05_mod:liangou2") > 0 {
        amount += 1;
    }
    // card/gujian/skill.js:313-342 (`molicha`) & 456-467 (`heilonglinpian`): reduce damage > 1
    if amount > 1
        && (flag_val(state, target, "u05_mod:molicha") > 0
            || flag_val(state, target, "u05_mod:heilonglinpian") > 0)
    {
        amount -= 1;
    }
    amount
}

pub(super) fn apply_ignore_armor(
    state: &GameStateCore,
    source: PlayerId,
    mut ignore: bool,
) -> bool {
    if flag_val(state, source, "u05_mod:bshaowei") > 0 {
        ignore = true;
    }
    ignore
}

pub(super) fn apply_card_unrespondable(
    state: &GameStateCore,
    source: PlayerId,
    target: PlayerId,
    kind: CardKind,
    mut unresp: bool,
) -> bool {
    if kind.is_sha_like()
        && (flag_val(state, source, "u05_mod:bshaowei") > 0
            || flag_val(state, source, "u05_mod:juji3") > 0
            || flag_val(state, source, "u05_mod:yunchuizhen") > 0
            || flag_val(state, target, "u05_mod:xiaoyue2") > 0
            || flag_val(state, target, "u05_mod:yanzhan3") > 0
            || flag_val(state, target, "u05_mod:shanguang2") > 0)
    {
        unresp = true;
    }
    unresp
}
