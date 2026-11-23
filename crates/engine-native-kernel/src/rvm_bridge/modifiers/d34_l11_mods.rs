//! Explicit per-skill `L11 太虚幻境` modifier evaluations (`d34_l11_mods.rs`), extracted from
//! `archive/upstream/noname-liuli/noname/extension/太虚幻境/{extension.js,extension_skill.js,extension_buff.js}`
//! and the 164 cross-package NPC skills in `archive/upstream/noname-liuli/noname/character/*.js`
//! & `archive/upstream/noname/apps/core/character/**/*.js`.

use crate::catalog::{CardKind, Catalog, EquipSlot, Suit};
use crate::event::DamageNature;
use crate::ids::{CardId, PlayerId};
use crate::state::{GameMode, GameStateCore, Role};

use super::flag_val;
use super::has_ext_skill;

#[path = "d34_l25_mods.rs"]
mod d34_l25_mods;

fn has_armor(state: &GameStateCore, catalog: &Catalog, seat: PlayerId) -> bool {
    state
        .player(seat)
        .equips
        .iter()
        .any(|&cid| catalog.card_slot_of(state, cid) == Some(EquipSlot::Armor))
}

pub(super) fn has_equipped_named(state: &GameStateCore, catalog: &Catalog, seat: PlayerId, name: &str) -> bool {
    state
        .player(seat)
        .equips
        .iter()
        .any(|&cid| catalog.cards.get(cid as usize).is_some_and(|cd| cd.name == name))
}

pub(in crate::rvm_bridge::modifiers) fn is_ordinary_trick_kind(kind: CardKind) -> bool {
    !kind.is_basic_kind()
        && kind.equip_slot().is_none()
        && !matches!(
            kind,
            CardKind::Le
                | CardKind::Bing
                | CardKind::ShanDian
                | CardKind::HongShui
                | CardKind::Guiyoujie
        )
}

pub fn txminguangkai_blocks_link(state: &GameStateCore, catalog: &Catalog, player: PlayerId) -> bool {
    !state.player(player).linked
        && state.player(player).hp < state.player(player).max_hp as i32
        && !super::is_target_unequip2(state, player)
        && (has_equipped_named(state, catalog, player, "txminguangkai")
            || has_ext_skill(state, catalog, player, "txminguangkai")
            || has_ext_skill(state, catalog, player, "txminguangkai_link"))
}

pub fn txlianbi_blocks_unlink(state: &GameStateCore, catalog: &Catalog, player: PlayerId) -> bool {
    state.player(player).linked && has_ext_skill(state, catalog, player, "txlianbi")
}

pub fn has_yaya_protected(state: &GameStateCore, catalog: &Catalog, player: PlayerId) -> bool {
    flag_val(state, player, "l11_yaya_protected") > 0
        || has_ext_skill(state, catalog, player, "txhj_yayaSkill2_protected")
}

pub fn is_card_ignored_in_discard_phase(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    card: CardId,
) -> bool {
    if flag_val(state, player, &format!("l11_gaintag_paiku:{card}")) > 0
        && (has_ext_skill(state, catalog, player, "_txhj_skill_maxCards")
            || state
                .alive_players()
                .any(|h| has_ext_skill(state, catalog, h, "_txhj_skill_maxCards")))
    {
        return true;
    }
    let is_black = matches!(
        catalog.effective_card_suit(state, player, card),
        Suit::Spade | Suit::Club
    );
    is_black
        && state.alive_players().any(|h| {
            state.is_friend_of(catalog, player, h)
                && (has_ext_skill(state, catalog, h, "cxyJiaoXia")
                    || flag_val(state, h, "l11_mod:cxyJiaoXia") > 0)
        })
}

pub(super) fn is_card_ignored_hand_limit(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    card: CardId,
    kind: CardKind,
) -> bool {
    if is_card_ignored_in_discard_phase(state, catalog, player, card) {
        return true;
    }
    // `tw/skill.js:16171` (`twyuhua`: non-basic cards ignored for hand limit)
    if !kind.is_basic_kind()
        && (flag_val(state, player, "l11_mod:twyuhua") > 0
            || has_ext_skill(state, catalog, player, "twyuhua"))
    {
        return true;
    }
    // `sp.js:16213` (`xiehui`: black cards ignored for hand limit)
    let is_black = catalog
        .cards
        .get(card as usize)
        .is_some_and(|cd| matches!(cd.suit, Suit::Spade | Suit::Club));
    if is_black
        && (flag_val(state, player, "l11_mod:xiehui") > 0
            || has_ext_skill(state, catalog, player, "xiehui"))
    {
        return true;
    }
    false
}

pub(super) fn apply_hand_limit(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    mut limit: i32,
) -> i32 {
    // extension.js:3441 (`xinmingjian2`: unlimited maxHandcard)
    if flag_val(state, player, "l11_mod:xinmingjian2") > 0
        || has_ext_skill(state, catalog, player, "xinmingjian2")
    {
        limit += 99;
    }
    // extension.js:4988 (`txkeji_add`: +4 maxHandcard)
    if flag_val(state, player, "l11_mod:txkeji_add") > 0
        || has_ext_skill(state, catalog, player, "txkeji_add")
    {
        limit += 4;
    }
    // `shenhua.js:7047` (`buqu`: hand limit equals `创` count when `创` present)
    let buqu = flag_val(state, player, "l11_mod:buqu_chuang");
    if buqu > 0 {
        limit = limit.max(buqu as i32);
    }
    // `yijiang.js:7487` (`jigong`), `yijiang.js:8886` (`mingjian`), `mobile/skill.js:23519` (`requanji`),
    // `sb/skill.js:9647` (`sbyingzi`), `sp2/skill.js:9985` (`shunshi`)
    limit += flag_val(state, player, "l11_mod:jigong_hand") as i32;
    limit += flag_val(state, player, "l11_mod:mingjian_buff") as i32;
    limit += flag_val(state, player, "l11_mod:requanji_quan") as i32;
    limit += flag_val(state, player, "l11_mod:sbyingzi_hand") as i32;
    limit += flag_val(state, player, "l11_mod:shunshi_hand") as i32;
    // extension.js:1393 (`txhj_renwang`: +expansion count)
    limit += flag_val(state, player, "l11_mod:txhj_renwang") as i32;
    // extension_buff.js:365 (`buff_txhj_wuweizhile` & `wangzun`: target hand limit -1)
    limit -= flag_val(state, player, "l11_wuweizhile_minus1") as i32;
    // extension.js:5044 (`txyingyang`: pindian/hand bonus)
    if flag_val(state, player, "l11_yingyang_pindian_mod") > 0 {
        limit += 1;
    }
    // L27 选将美化 Batch A (`kuangcai`, `sbtongye`, `quanji`, `sbjianxiong`)
    limit += flag_val(state, player, "kuangcai_delta") as i32;
    limit += flag_val(state, player, "sbtongye_ye")
        .max(flag_val(state, player, "sbtongye")) as i32;
    let quanji_pile = state
        .vm_aux
        .piles
        .get(&(player, "quanji".to_string()))
        .map_or(0, |p| p.len() as u32);
    limit += flag_val(state, player, "quanji_count")
        .max(flag_val(state, player, "quanji"))
        .max(quanji_pile) as i32;
    limit += flag_val(state, player, "sbjianxiong")
        .max(flag_val(state, player, "sbjianxiong_marks")) as i32;
    // L27 选将美化 Batch B (`xinfu_xionghuo`, `twxiafeng`, `jishe2`)
    limit -= flag_val(state, player, "xinfu_xionghuo_hand_minus") as i32;
    limit += flag_val(state, player, "twxiafeng_x")
        .max(flag_val(state, player, "l27_twxiafeng_spent")) as i32;
    if flag_val(state, player, "l27_jishe_limit_minus") == 0 {
        limit -= flag_val(state, player, "jishe2") as i32;
    }
    // L19 末日乱斗 (`_wujinmoshiMaxHandCard`, `_wujinmoshiMaxHandCardE`)
    if has_ext_skill(state, catalog, player, "_wujinmoshiMaxHandCard")
        || has_ext_skill(state, catalog, player, "_wujinmoshiMaxHandCardE")
    {
        limit += flag_val(state, player, "wujin_hand_bonus")
            .max(flag_val(state, player, "l19_wujin_hand_limit_bonus"))
            .max(1) as i32;
    } else if flag_val(state, player, "wujin_hand_bonus") > 0
        || flag_val(state, player, "l19_wujin_hand_limit_bonus") > 0
    {
        limit += flag_val(state, player, "wujin_hand_bonus")
            .max(flag_val(state, player, "l19_wujin_hand_limit_bonus")) as i32;
    }
    // L20 标记补充 (`_yinyang_mark_add`, `extension.js:1802`: +2 maxHandcard this turn)
    limit += flag_val(state, player, "yinyang_hand_bonus") as i32;
    // L25 祖安武将 Batch A (`xinjuejing` +2 hand limit; `g_nzry_jieying` `mark:ying` +1 hand limit)
    if has_ext_skill(state, catalog, player, "xinjuejing")
        || flag_val(state, player, "xinjuejing_active") > 0
    {
        limit += 2;
    }
    if flag_val(state, player, "mark:ying") > 0 {
        limit += 1;
    }
    // `extension_skill.js:17` (`_txhj_skill_maxCards`: maxHandcardBase += maxHs)
    let max_cards_active = has_ext_skill(state, catalog, player, "_txhj_skill_maxCards")
        || (player == 0
            && state
                .alive_players()
                .any(|h| has_ext_skill(state, catalog, h, "_txhj_skill_maxCards")));
    if max_cards_active {
        limit += flag_val(state, player, "l11_txhj_max_hs") as i32;
    }
    // `extension_skill.js:4373` (`txhj_zhuwei_eff`: maxHandcard +X this turn)
    limit += flag_val(state, player, "skill_turn:l11_zhuwei_eff") as i32;
    d34_l25_mods::apply_hand_limit(state, catalog, player, limit)
}

pub fn apply_draw_count(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    mut cur: i32,
) -> i32 {
    cur += flag_val(state, player, "kuangcai_delta") as i32;
    cur += flag_val(state, player, "sbtongye_ye")
        .max(flag_val(state, player, "sbtongye")) as i32;
    cur += flag_val(state, player, "sbjianxiong")
        .max(flag_val(state, player, "sbjianxiong_marks")) as i32;
    d34_l25_mods::apply_draw_count(state, catalog, player, cur.max(0))
}

pub(super) fn apply_attack_range(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    mut cur: i32,
) -> i32 {
    // `jsrg.js:3836` (`jsrgzhenqiao`: attack range +1)
    if flag_val(state, player, "l11_mod:jsrgzhenqiao_range") > 0
        || has_ext_skill(state, catalog, player, "jsrgzhenqiao")
    {
        cur += 1;
    }
    // `tw/skill.js:27771` (`twqueshi`: Meiying spear +2 range)
    let queshi = flag_val(state, player, "l11_mod:twqueshi_meiying");
    if queshi > 0 {
        cur += queshi as i32;
    } else if has_ext_skill(state, catalog, player, "twqueshi") {
        cur += 2;
    }
    // `tw/skill.js:17764` (`twxiangyu`: attack range +X)
    let xiangyu = flag_val(state, player, "l11_mod:twxiangyu_range");
    if xiangyu > 0 {
        cur += xiangyu as i32;
    } else if has_ext_skill(state, catalog, player, "twxiangyu") {
        cur += 1;
    }
    // L20 标记补充 (`fz_liegong`, `extension.js:5187`): +1 attack range iff `get.zhu(player, 'shouyue')`
    if (has_ext_skill(state, catalog, player, "fz_liegong")
        || flag_val(state, player, "fz_liegong_active") > 0)
        && has_zhu_shouyue(state, catalog, player)
    {
        cur += 1;
    }
    cur
}

fn has_zhu_shouyue(state: &GameStateCore, catalog: &Catalog, player: PlayerId) -> bool {
    let has_shouyue = |p: PlayerId| {
        state.player(p).alive
            && (has_ext_skill(state, catalog, p, "shouyue")
                || flag_val(state, p, "shouyue_active") > 0)
    };
    match &state.mode {
        GameMode::Identity { roles } => roles
            .iter()
            .enumerate()
            .find_map(|(idx, r)| (*r == Role::Lord).then_some(idx as PlayerId))
            .is_some_and(has_shouyue),
        GameMode::Versus { teams, .. } => teams
            .iter()
            .find(|team| team.contains(&player))
            .and_then(|team| team.first().copied())
            .is_some_and(has_shouyue),
        GameMode::Kingdoms => {
            let pf = state.declared_faction(catalog, player);
            pf.is_some()
                && (0..state.players.len() as PlayerId).any(|q| {
                    state.is_revealed_jun(catalog, q)
                        && state.declared_faction(catalog, q) == pf
                        && has_shouyue(q)
                })
        }
        _ => false,
    }
}

pub(super) fn apply_distance_total(
    state: &GameStateCore,
    catalog: &Catalog,
    from: PlayerId,
    to: PlayerId,
    mut dist: i32,
) -> i32 {
    // `sp.js:15556` (`fenxun`) & `sp.js:11358` (`refenxun`): distance to target treated as 1
    // (`tspowei` `inRangeOf` is one-way attack range in `GameStateCore::in_attack_range`, NOT distance=1!)
    if flag_val(state, from, "l11_mod:fenxun_dist1") > 0 {
        return 1;
    }
    // `sp.js:7210` (`olzhuiji`: distance to characters with hp <= owner.hp is 1)
    if (flag_val(state, from, "l11_mod:olzhuiji_dist") > 0
        || has_ext_skill(state, catalog, from, "olzhuiji"))
        && state.player(from).hp >= state.player(to).hp
    {
        return 1;
    }
    // L25 祖安武将 Batch A (`_taohuaskill`, `extension.js:42450`: distance to opposite gender is 1)
    if (has_ext_skill(state, catalog, from, "_taohuaskill")
        || flag_val(state, from, "l25_taohua_active") > 0)
        && state.different_gender_from(catalog, from, to)
    {
        return 1;
    }
    // extension.js:3460 (`txmazhan`: globalFrom - 2, globalTo + 1)
    if flag_val(state, from, "l11_mod:txmazhan") > 0
        || has_ext_skill(state, catalog, from, "txmazhan")
    {
        dist -= 2;
    }
    if flag_val(state, to, "l11_mod:txmazhan") > 0
        || has_ext_skill(state, catalog, to, "txmazhan")
    {
        dist += 1;
    }
    // `sp.js:19457` (`lixia`: others calculate distance to owner -1)
    if flag_val(state, to, "l11_mod:lixia_dist_to_minus1") > 0
        || has_ext_skill(state, catalog, to, "lixia")
    {
        dist -= 1;
    }
    // `refresh.js:17516` (`retuntian`: distance from owner -X `田`)
    dist -= flag_val(state, from, "l11_mod:retuntian_tian") as i32;
    // L27 选将美化 (`qimou` / `reqimou`: distance from owner -X)
    dist -= flag_val(state, from, "qimou_x")
        .max(flag_val(state, from, "reqimou_x")) as i32;
    // extension_card.js:501, 642, 1263 (`txhj_liulongcanjia`, `txhj_juechenjinge`, `txhj_baihu`)
    // `core_ops.rs` already adds +1 to `to` for every physical `CardKind::HorsePlus`.
    let has_liulong_from = has_equipped_named(state, catalog, from, "txhj_liulongcanjia");
    if has_liulong_from
        || has_ext_skill(state, catalog, from, "txhj_liulongcanjia")
        || flag_val(state, from, "l11_mount_active:txhj_liulongcanjia") > 0
    {
        dist -= 1;
    }
    if !has_equipped_named(state, catalog, to, "txhj_liulongcanjia")
        && (has_ext_skill(state, catalog, to, "txhj_liulongcanjia")
            || flag_val(state, to, "l11_mount_active:txhj_liulongcanjia") > 0)
    {
        dist += 1;
    }
    // `txhj_baihu` (`extension_card.js:1263`): globalTo +2 (NOT globalFrom -1)
    if has_equipped_named(state, catalog, to, "txhj_baihu") {
        dist += 1;
    } else if has_ext_skill(state, catalog, to, "txhj_baihu")
        || flag_val(state, to, "l11_mount_active:txhj_baihu") > 0
    {
        dist += 2;
    }
    // `txhj_juechenjinge` (`extension_card.js:642`): self globalTo +2; other alive friends +1
    if has_equipped_named(state, catalog, to, "txhj_juechenjinge") {
        dist += 1;
    } else if has_ext_skill(state, catalog, to, "txhj_juechenjinge")
        || flag_val(state, to, "l11_mount_active:txhj_juechenjinge") > 0
    {
        dist += 2;
    }
    for h in state.alive_players() {
        if h != to
            && state.is_friend_of(catalog, to, h)
            && (has_equipped_named(state, catalog, h, "txhj_juechenjinge")
                || has_ext_skill(state, catalog, h, "txhj_juechenjinge")
                || has_ext_skill(state, catalog, h, "txhj_juechenjinge2")
                || flag_val(state, h, "l11_mount_active:txhj_juechenjinge") > 0)
        {
            dist += 1;
        }
    }
    // `cxyJingQi` (`extension.js:4082`): alive owner gives allies globalFrom -1
    if state.alive_players().any(|h| {
        state.is_friend_of(catalog, from, h)
            && (has_ext_skill(state, catalog, h, "cxyJingQi")
                || flag_val(state, h, "l11_mod:cxyJingQi") > 0)
    }) {
        dist -= 1;
    }
    d34_l25_mods::apply_distance_total(state, catalog, from, to, dist)
}

pub(super) fn apply_sha_use_limit(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    mut cur: i32,
    mut unlimited: bool,
) -> (i32, bool) {
    // extension.js:3441 (`xinmingjian2`), `diy.js:3461` (`kyou_zhidian`), `shenhua.js:2142` (`nzry_chenglve`),
    // `tw/skill.js:1764` (`twwushen`), `sp/skill.js:49243` (`xinfu_limu`)
    if flag_val(state, player, "l11_mod:xinmingjian2") > 0
        || flag_val(state, player, "l11_mod:kyou_zhidian_unlimited") > 0
        || flag_val(state, player, "l11_mod:nzry_chenglve") > 0
        || flag_val(state, player, "l11_mod:twwushen") > 0
        || flag_val(state, player, "l11_mod:xinfu_limu") > 0
        || has_ext_skill(state, catalog, player, "xinmingjian2")
        || has_ext_skill(state, catalog, player, "twwushen")
    {
        unlimited = true;
    }
    // extension.js:6078 (`txhj_oldhuxiao`) & extension_skill.js:1071 (`pyzhuren_diamond`)
    if flag_val(state, player, "l11_mod:txhj_oldhuxiao") > 0 {
        cur += 1;
    }
    if flag_val(state, player, "l11_mod:pyzhuren_diamond") > 0
        || has_ext_skill(state, catalog, player, "pyzhuren_diamond")
    {
        cur += 1;
    }
    // `collab.js:3112` (`dcnutao`), `yijiang.js:8886` (`mingjian`), `sp2/skill.js:9985` (`shunshi`),
    // `sp/skill.js:38664` (`tuifeng`), `sp.js:11432` (`zhuixi`)
    cur += flag_val(state, player, "l11_mod:dcnutao_sha_plus") as i32;
    cur += flag_val(state, player, "l11_mod:mingjian_buff") as i32;
    cur += flag_val(state, player, "l11_mod:shunshi_sha") as i32;
    cur += flag_val(state, player, "l11_mod:tuifeng_sha") as i32;
    if flag_val(state, player, "l11_mod:zhuixi_sha_plus1") > 0
        || has_ext_skill(state, catalog, player, "zhuixi")
    {
        cur += 1;
    }
    // L27 选将美化 (`qimou`, `reqimou`, `kuangcai`, `potzhuangshi`)
    let qimou_bonus = flag_val(state, player, "l27_qimou_range_bonus");
    if qimou_bonus == 0 {
        cur += flag_val(state, player, "qimou_x")
            .max(flag_val(state, player, "reqimou_x")) as i32;
    } else {
        cur += flag_val(state, player, "reqimou_x").saturating_sub(qimou_bonus) as i32;
    }
    if has_ext_skill(state, catalog, player, "kuangcai") {
        unlimited = true;
    }
    let pot_y = flag_val(state, player, "potzhuangshi_damage_y");
    if pot_y > 0 && state.turn.cards_used_play_phase < pot_y {
        unlimited = true;
    }
    let xiafeng_x = flag_val(state, player, "twxiafeng_x")
        .max(flag_val(state, player, "l27_twxiafeng_spent"));
    if xiafeng_x > 0 && state.turn.cards_used_play_phase < xiafeng_x {
        unlimited = true;
    }
    // L19 末日乱斗 (`baozou`, `paoxiao` on `jiangshifz`/`jiangshinj`) & L20 标记补充 (`fz_gzpaoxiao`)
    if has_ext_skill(state, catalog, player, "baozou")
        || has_ext_skill(state, catalog, player, "paoxiao")
        || has_ext_skill(state, catalog, player, "fz_gzpaoxiao")
        || has_ext_skill(state, catalog, player, "wushen")
        || flag_val(state, player, "nzry_longnu_thunder_turn") > 0
        || flag_val(state, player, "baozou_unlimited") > 0
        || flag_val(state, player, "l19_baozou_sha_unlimited") > 0
    {
        unlimited = true;
    }
    if flag_val(state, player, "mark:ying") > 0 {
        cur += 1;
    }
    // `extension_skill.js:4370` (`txhj_zhuwei_eff`: Sha cardUsable +X this turn)
    cur += flag_val(state, player, "skill_turn:l11_zhuwei_eff") as i32;
    d34_l25_mods::apply_sha_use_limit(state, catalog, player, cur, unlimited)
}

pub(super) fn apply_sha_extra_targets(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    kind: CardKind,
    mut extra: i32,
) -> i32 {
    if kind.is_sha_like() {
        // `sp2.js:9417` (`bmcanshi`) & `tw/skill.js:26877` (`twhengjiang`)
        if flag_val(state, player, "l11_mod:bmcanshi") > 0
            || has_ext_skill(state, catalog, player, "bmcanshi")
        {
            extra += 1;
        }
        extra += flag_val(state, player, "l11_mod:guju_extra") as i32;
        extra += flag_val(state, player, "l11_mod:twhengjiang_extra") as i32;
    }
    d34_l25_mods::apply_sha_extra_targets(state, catalog, player, kind, extra)
}

pub(super) fn apply_card_distance_unlimited_to(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    target: Option<PlayerId>,
    card: CardId,
    kind: CardKind,
    mut unlimited: bool,
) -> bool {
    // extension.js:4854 (`txshouyi`) + `diy.js:3461` (`kyou_zhidian`) + `shenhua.js:2142` (`nzry_chenglve`)
    // + `shenhua/skill.js:7595` (`shensu`) + `tw/skill.js:16072` (`twqirang`) + `tw/skill.js:1764` (`twwushen`)
    // + `sp/skill.js:49243` (`xinfu_limu`)
    if flag_val(state, player, "l11_mod:txshouyi") > 0
        || flag_val(state, player, "l11_mod:kyou_zhidian_unlimited") > 0
        || flag_val(state, player, "l11_mod:nzry_chenglve") > 0
        || flag_val(state, player, "l11_mod:shensu_no_dist") > 0
        || flag_val(state, player, "l11_mod:twqirang") > 0
        || flag_val(state, player, "l11_mod:twwushen") > 0
        || flag_val(state, player, "l11_mod:xinfu_limu") > 0
        || has_ext_skill(state, catalog, player, "txshouyi")
        || has_ext_skill(state, catalog, player, "kyou_zhidian")
        || has_ext_skill(state, catalog, player, "twwushen")
    {
        unlimited = true;
    }
    // extension.js:5592 (`txhj_suzhi`: forced/locked, tricks unlimited range during own turn while marks < 3)
    if !kind.is_basic_kind()
        && kind.equip_slot().is_none()
        && state.turn.current == player
        && has_ext_skill(state, catalog, player, "txhj_suzhi")
        && flag_val(state, player, "skill_turn:l11_suzhi_marks") < 3
    {
        unlimited = true;
    }
    // extension_buff.js:376 (`buff_txhj_yuanduandilue`: Bingliang unlimited distance)
    if kind == CardKind::Bing
        && (flag_val(state, player, "l11_yuanduandilue_bing_no_dist") > 0
            || has_ext_skill(state, catalog, player, "buff_txhj_yuanduandilue")
            || has_ext_skill(state, catalog, player, "_buff_txhj_yuanduandilue")
            || has_ext_skill(state, catalog, player, "txhj_yuanduandilue"))
    {
        unlimited = true;
    }
    if has_ext_skill(state, catalog, player, "kuangcai") {
        unlimited = true;
    }
    let pot_x = flag_val(state, player, "potzhuangshi_unblockable_x");
    if pot_x > 0 && state.turn.cards_used_play_phase < pot_x {
        unlimited = true;
    }
    let xiafeng_x = flag_val(state, player, "twxiafeng_x")
        .max(flag_val(state, player, "l27_twxiafeng_spent"));
    if xiafeng_x > 0 && state.turn.cards_used_play_phase < xiafeng_x {
        unlimited = true;
    }
    if kind.is_sha_like()
        && catalog
            .cards
            .get(card as usize)
            .is_some_and(|cd| cd.suit == Suit::Diamond)
        && (has_ext_skill(state, catalog, player, "fz_new_rewusheng")
            || flag_val(state, player, "fz_new_rewusheng_unlimited") > 0)
    {
        unlimited = true;
    }
    if kind.is_sha_like()
        && ((catalog
            .cards
            .get(card as usize)
            .is_some_and(|cd| cd.suit == Suit::Heart)
            && has_ext_skill(state, catalog, player, "wushen"))
            || flag_val(state, player, "nzry_longnu_fire_turn") > 0)
    {
        unlimited = true;
    }
    d34_l25_mods::apply_card_distance_unlimited_to(
        state, catalog, player, target, card, kind, unlimited,
    )
}

pub(super) fn apply_card_target_canceled(
    state: &GameStateCore,
    catalog: &Catalog,
    actor: PlayerId,
    target: PlayerId,
    card: Option<CardId>,
    kind: CardKind,
    mut canceled: bool,
) -> bool {
    let is_trick = !kind.is_basic_kind() && kind.equip_slot().is_none();
    // `extension_card.js:1737` (`txhj_liulongcanjia`): cannot target/equip other mounts while equipped
    let slot = card
        .and_then(|c| catalog.card_slot_of(state, c))
        .or_else(|| kind.equip_slot());
    if matches!(slot, Some(EquipSlot::HorsePlus | EquipSlot::HorseMinus))
        && has_equipped_named(state, catalog, target, "txhj_liulongcanjia")
    {
        canceled = true;
    }
    // extension.js:3593 (`txshixin` legacy flag only; real txshixin runs on CardUsed)
    // extension_card.js:821 (`txhj_guofengyupao`, `guofengyupao`) & extension_buff.js:1212 (`txhj_ruiSkill3`, `txhj_yanSkill1`, `txhj_xuanwuSkill3`)
    if actor != target
        && is_trick
        && (flag_val(state, target, "l11_mod:txshixin") > 0
            || flag_val(state, target, "l11_armor_guard:txhj_guofengyupao") > 0
            || flag_val(state, target, "l11_armor_guard:guofengyupao") > 0
            || flag_val(state, target, "l11_trick_guard:txhj_ruiSkill3") > 0
            || flag_val(state, target, "l11_trick_guard:txhj_yanSkill1") > 0
            || flag_val(state, target, "l11_trick_guard:txhj_xuanwuSkill3") > 0)
    {
        canceled = true;
    }
    if actor != target
        && is_ordinary_trick_kind(kind)
        && (has_equipped_named(state, catalog, target, "txhj_guofengyupao")
            || has_ext_skill(state, catalog, target, "txhj_guofengyupao"))
        && !super::is_target_unequip2(state, target)
        && !super::super::d34_l28_mods::source_ignores_armor(
            state,
            catalog,
            actor,
            target,
            card,
            Some(kind),
            DamageNature::Normal,
        )
    {
        canceled = true;
    }
    // `cxyFengYing` (`extension.js:4173`): ally `target` with strictly unique lowest HP cannot be targeted by an enemy
    if actor != target && !state.is_friend_of(catalog, actor, target) {
        let target_hp = state.player(target).hp;
        let target_unique_min = state.player(target).alive
            && state
                .alive_players()
                .all(|p| p == target || state.player(p).hp > target_hp);
        if target_unique_min
            && state.alive_players().any(|h| {
                state.is_friend_of(catalog, target, h)
                    && (has_ext_skill(state, catalog, h, "cxyFengYing")
                        || flag_val(state, h, "l11_mod:cxyFengYing") > 0)
            })
        {
            canceled = true;
        }
    }
    // `sp2.js:9417` (`bmcanshi`: cancel card from `傀`-marked actor)
    if actor != target
        && flag_val(state, actor, "l11_kui_mark") > 0
        && (flag_val(state, target, "l11_mod:bmcanshi") > 0
            || has_ext_skill(state, catalog, target, "bmcanshi"))
    {
        canceled = true;
    }
    // `mobile.js:11997` (`gaoyuan`) & `tw/skill.js:25864` (`twhuangjin`): Sha deflected/nullified
    if kind.is_sha_like()
        && (flag_val(state, target, "l11_mod:gaoyuan_deflect") > 0
            || flag_val(state, target, "l11_mod:twhuangjin_null") > 0)
    {
        canceled = true;
    }
    // `shenhua/skill.js:1136` (`xinfu_fuyin`: Sha/Duel nullified when target hand <= actor hand)
    if actor != target
        && (kind.is_sha_like() || kind == CardKind::Duel)
        && state.player(target).hand.len() <= state.player(actor).hand.len()
        && (flag_val(state, target, "l11_mod:xinfu_fuyin") > 0
            || has_ext_skill(state, catalog, target, "xinfu_fuyin"))
    {
        canceled = true;
    }
    // `yijiang.js:11491` (`zhichi`: Sha & tricks nullified after out-of-turn damage)
    if (kind.is_sha_like() || is_trick) && flag_val(state, target, "l11_mod:zhichi_ward") > 0 {
        canceled = true;
    }
    // extension.js:658 (`txhj_guimei`: immune to delayed tricks)
    if matches!(
        kind,
        CardKind::Le | CardKind::Bing | CardKind::ShanDian | CardKind::HongShui
    ) && (flag_val(state, target, "l11_mod:txhj_guimei") > 0
        || has_ext_skill(state, catalog, target, "txhj_guimei"))
    {
        canceled = true;
    }
    // extension.js:5303 (`zglkongcheng`: empty hand immune to Sha/Duel)
    if state.player(target).hand.is_empty()
        && (kind.is_sha_like() || kind == CardKind::Duel)
        && (flag_val(state, target, "l11_mod:zglkongcheng") > 0
            || has_ext_skill(state, catalog, target, "zglkongcheng"))
    {
        canceled = true;
    }
    // extension.js:5704 (`txmanzhen`), `extension.js:5388` (`txjuxiang`), & `shiji/skill.js:4520` (`spxiangzhen`): immune to SavageAttack
    if kind == CardKind::SavageAttack
        && (flag_val(state, target, "l11_mod:txmanzhen") > 0
            || has_ext_skill(state, catalog, target, "txmanzhen")
            || flag_val(state, target, "l11_mod:txjuxiang") > 0
            || has_ext_skill(state, catalog, target, "txjuxiang")
            || has_ext_skill(state, catalog, target, "spxiangzhen"))
    {
        canceled = true;
    }
    // extension.js:5660 (`txmanjia` / `txmanjia1`): unarmored Vine Armor immunity to Nanman / Wanjian / normal Sha
    let has_txmanjia_armor = flag_val(state, target, "l11_mod:txmanjia") > 0
        || has_ext_skill(state, catalog, target, "txmanjia")
        || has_ext_skill(state, catalog, target, "txmanjia1");
    if has_txmanjia_armor
        && !has_armor(state, catalog, target)
        && !super::is_target_unequip2(state, target)
        && !super::super::d34_l28_mods::source_ignores_armor(
            state,
            catalog,
            actor,
            target,
            card,
            Some(kind),
            DamageNature::Normal,
        )
        && matches!(
            kind,
            CardKind::SavageAttack | CardKind::ArcheryAttack | CardKind::Sha
        )
    {
        canceled = true;
    }
    if actor != target
        && is_trick
        && flag_val(state, target, "l27_dinghan_negated") > 0
    {
        canceled = true;
    }
    if actor != target
        && kind.is_sha_like()
        && flag_val(state, target, "l27_shouxi_nullified") > 0
    {
        canceled = true;
    }
    if actor != target
        && kind.is_sha_like()
        && flag_val(state, actor, "xinfu_xionghuo_no_sha_turn") > 0
        && (has_ext_skill(state, catalog, target, "xinfu_xionghuo")
            || flag_val(state, target, "xinfu_xionghuo_marks") > 0
            || flag_val(state, target, "xinfu_xionghuo_init") > 0)
    {
        canceled = true;
    }
    if matches!(
        kind,
        CardKind::Le | CardKind::Bing | CardKind::ShanDian | CardKind::HongShui
    ) && flag_val(state, target, "mbwuling_spirit") == 2
    {
        canceled = true;
    }
    // L19 末日乱斗 (`_tamodeskill`: tower `nei` cannot be targeted by normal cards)
    if actor != target
        && (has_ext_skill(state, catalog, target, "_tamodeskill")
            || flag_val(state, target, "_tamodeskill_untouchable") > 0
            || flag_val(state, target, "is_tower_nei") > 0)
    {
        canceled = true;
    }
    // L19 末日乱斗 (`_guanmodeskill`: boss `zhu` immune to delayed tricks)
    if matches!(
        kind,
        CardKind::Le | CardKind::Bing | CardKind::ShanDian | CardKind::HongShui
    ) && (has_ext_skill(state, catalog, target, "_guanmodeskill")
        || flag_val(state, target, "_guanmodeskill_immune") > 0)
    {
        canceled = true;
    }
    // L20 标记补充 (`diaohulishan`, `extension.js:1432`) + U05-7d 对决出局 (`duijue2`): target cannot be targeted by cards
    if catalog.has_undist(state, target) {
        canceled = true;
    }
    // L20 标记补充 (`_aozhan_judge`, `extension.js:2152`: real Tao cannot target ANY player)
    if kind == CardKind::Tao
        && (flag_val(state, actor, "aozhan_active") > 0
            || has_ext_skill(state, catalog, actor, "aozhan"))
    {
        canceled = true;
    }
    // `sp2/skill.js:15853-15862` (`xinfu_tanbei` Mode 0 -> actor cannot target `target` with cards this turn)
    if flag_val(state, actor, &format!("skill_turn:xinfu_tanbei_forbid:{target}")) > 0 {
        canceled = true;
    }
    d34_l25_mods::apply_card_target_canceled(state, catalog, actor, target, kind, canceled)
}

pub(super) fn apply_card_use_canceled(
    state: &GameStateCore,
    catalog: &Catalog,
    actor: PlayerId,
    kind: CardKind,
    in_hand: bool,
    mut canceled: bool,
) -> bool {
    // extension.js:5131 (`mdqianxi72`), `mobile.js:11007` (`jinglve`), `sp/skill.js:39282` (`shefu`)
    if in_hand
        && (flag_val(state, actor, "l11_mod:mdqianxi72") > 0
            || flag_val(state, actor, "l11_jinglve_sishi") > 0
            || flag_val(state, actor, "l11_shefu_ambush") > 0)
    {
        canceled = true;
    }
    // `refresh.js:9638` (`rejinjiu`: others cannot use Jiu during owner's turn)
    let cur_turn = state.turn.current;
    if kind == CardKind::Jiu
        && cur_turn != actor
        && (flag_val(state, cur_turn, "l11_mod:rejinjiu") > 0
            || has_ext_skill(state, catalog, cur_turn, "rejinjiu"))
    {
        canceled = true;
    }
    // L20 标记补充 (`diaohulishan`, `extension.js:1432`) + U05-7d 对决出局 (`duijue2`): cannot use/respond with cards while out of game
    if catalog.has_undist(state, actor) {
        canceled = true;
    }
    // L20 标记补充 (`_aozhan_judge`, `extension.js:2152`: real Tao cannot be used to heal/save)
    if kind == CardKind::Tao
        && (flag_val(state, actor, "aozhan_active") > 0
            || has_ext_skill(state, catalog, actor, "aozhan"))
    {
        canceled = true;
    }
    d34_l25_mods::apply_card_use_canceled(state, catalog, actor, kind, in_hand, canceled)
}

pub(super) fn apply_damage_amount(
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
        || flag_val(state, target, "l11_mod:retianxiang_ward") > 0
        || flag_val(state, target, "l11_mod:wulie_lie") > 0
        || flag_val(state, target, "l11_mod:xinfu_daigong_ward") > 0
        || (flag_val(state, target, "l11_mod:mjmouzhi_suit") > 0 && card.is_some())
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
        let armor_ignored = super::is_target_unequip2(state, target)
            || source.is_some_and(|src| {
                super::super::d34_l28_mods::source_ignores_armor(
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
            || flag_val(state, src, "l11_mod:jieyuan_more") > 0
            || flag_val(state, src, "l11_mod:kyou_zhidian_dmg") > 0
            || flag_val(state, src, "l11_mod:new_yijue_heart_plus1") > 0
            || flag_val(state, src, "l11_mod:reluoyi_dmg_plus1") > 0
            || flag_val(state, src, "l11_mod:repojun_dmg_plus1") > 0
            || flag_val(state, src, "l11_mod:sbliegong_dmg") > 0
            || flag_val(state, src, "l11_mod:xinfu_zengdao_dao") > 0
            || has_ext_skill(state, catalog, src, "txhj_shiyuan")
        {
            cur += 1;
        }
        if flag_val(state, src, "l11_mod:twxuhe_plus2") > 0 {
            cur += 2;
        }
        if is_sha
            && (flag_val(state, src, "l11_mod:txfusha") > 0
                || has_ext_skill(state, catalog, src, "txhj_zuijiu")
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

pub(super) fn tpys_damage_prevented(
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
    if !has_tpys || super::is_target_unequip2(state, target) {
        return false;
    }
    let src_unequip = source.is_some_and(|src| {
        super::super::eval_ignore_armor(state, catalog, src, target, card, nature)
            || (card.is_some_and(|c| catalog.card_kind(c).is_sha_like())
                && (state.has_active_equip_kind(catalog, src, CardKind::Qinggang)
                    || catalog
                        .skill_by_name(crate::vocabulary::names::BOSS_ZHADAO)
                        .is_some_and(|def| state.has_skill(src, def))))
    });
    !src_unequip
}

pub(super) fn apply_ignore_armor(
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
    {
        ignore = true;
    }
    d34_l25_mods::apply_ignore_armor(state, catalog, source, target, card, ignore)
}

pub(super) fn apply_card_unrespondable(
    state: &GameStateCore,
    catalog: &Catalog,
    source: PlayerId,
    kind: CardKind,
    mut unrespondable: bool,
) -> bool {
    if kind.is_sha_like()
        && (flag_val(state, source, "l11_mod:kyou_zhidian_unresp") > 0
            || flag_val(state, source, "l11_mod:sbliegong_unresp") > 0
            || flag_val(state, source, "l11_mod:twwushen") > 0
            || flag_val(state, source, "l11_mod:twxiangyu_unresp") > 0
            || has_ext_skill(state, catalog, source, "twwushen")
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
}
