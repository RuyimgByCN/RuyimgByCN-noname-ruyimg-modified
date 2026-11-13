//! Explicit per-skill `L25 祖安武将` Batch B & Batch C passive modifier evaluations (`d34_l25_mods.rs`),
//! extracted from `archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js`.

use crate::catalog::{CardKind, Catalog, Suit};
use crate::event::{DamageNature, Phase};
use crate::ids::{CardId, PlayerId};
use crate::state::GameStateCore;

use super::{flag_val, has_ext_skill};

fn suit_code(suit: Suit) -> u32 {
    match suit {
        Suit::Spade => 1,
        Suit::Heart => 2,
        Suit::Club => 3,
        Suit::Diamond => 4,
    }
}

pub(super) fn apply_hand_limit(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    mut limit: i32,
) -> i32 {
    // `ssr_youqin` (`extension.js:7091-7219`): maxHandcard == maxHp
    if has_ext_skill(state, catalog, player, "ssr_youqin")
        || flag_val(state, player, "l25_youqin_active") > 0
    {
        limit = state.player(player).max_hp as i32;
    }
    // `ssr_jiuyuan` (`extension.js:5439-5524`): `jibing` responder hand limit -1
    limit -= flag_val(state, player, "l25_jiuyuan_jibing") as i32;
    // `ssr_newpozhen` (`extension.js:5945-6161`): +1 per active `战意` / `冲锋`
    limit += (flag_val(state, player, "l25_pozhen_zhanyi")
        + flag_val(state, player, "l25_pozhen_chongfeng")) as i32;
    // `ssr_zhuiji` (`extension.js:6945-6970`), `ssr_newyizhong` (`extension.js:11686-11723`), `ssr_newhanzhan`: +hujia to maxHandcard
    if has_ext_skill(state, catalog, player, "ssr_zhuiji")
        || has_ext_skill(state, catalog, player, "ssr_newyizhong")
        || has_ext_skill(state, catalog, player, "ssr_newhanzhan")
        || flag_val(state, player, "l25_zhuiji_active") > 0
        || flag_val(state, player, "l25_yizhong_active") > 0
        || flag_val(state, player, "l25_newhanzhan_active") > 0
    {
        limit += state.player(player).hujia as i32;
    }
    // `ssr_tumeng` (`extension.js:10751-10855`): hand limit -1 when `l25_tumeng_hand_minus1 > 0`
    limit -= flag_val(state, player, "l25_tumeng_hand_minus1") as i32;
    // `ssr_newguose` (`extension.js:17908-18125`): hand limit -1 when holding Da Qiao's die (`l25_guose_die > 0`)
    if flag_val(state, player, "l25_guose_die") > 0 {
        limit -= 1;
    }
    // `ssr_nianlei` (`extension.js:23090-23232`): `+chenqing - shangai`
    limit += flag_val(state, player, "l25_nianlei_chenqing") as i32;
    limit -= flag_val(state, player, "l25_nianlei_shangai") as i32;
    // `ssr_shensu`: `-((flag(player, "l25_shensu_su") / 2) as i32)`
    limit -= (flag_val(state, player, "l25_shensu_su") / 2) as i32;
    // `ssr_newsance`: `+2` when `l25_newsance_hand_plus2 > 0`
    if flag_val(state, player, "l25_newsance_hand_plus2") > 0 {
        limit += 2;
    }
    // `ssr_xunxun`: `+1` when `has_ext_skill(player, "ssr_xunxun") && hujia > 0`
    if (has_ext_skill(state, catalog, player, "ssr_xunxun")
        || flag_val(state, player, "l25_xunxun_active") > 0)
        && state.player(player).hujia > 0
    {
        limit += 1;
    }
    // `ssr_newlihun` (`extension.js:15578-15754`): `maxHandcard = 0` when `l25_lihun_zero_hand > 0`
    if flag_val(state, player, "l25_lihun_zero_hand") > 0 {
        limit = 0;
    }
    limit
}

pub(super) fn apply_draw_count(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    mut cur: i32,
) -> i32 {
    // `ssr_jiuyuan` (`extension.js:5439-5524`): `jibing` responder draw count +1
    cur += flag_val(state, player, "l25_jiuyuan_jibing") as i32;
    // `ssr_dimeng`: `+flag(player, "l25_dimeng_meng") as i32`
    cur += flag_val(state, player, "l25_dimeng_meng") as i32;
    // `ssr_newjiyuan` (`extension.js:21319-21462`): draw count -1
    if (has_ext_skill(state, catalog, player, "ssr_newjiyuan")
        || flag_val(state, player, "l25_jiyuan_active") > 0)
        && flag_val(state, player, "l25_jiyuan_normal_draw") == 0
    {
        cur -= 1;
    }
    cur.max(0)
}

pub(super) fn apply_distance_total(
    state: &GameStateCore,
    catalog: &Catalog,
    from: PlayerId,
    to: PlayerId,
    mut dist: i32,
) -> i32 {
    // `ssr_zhuiji` (`extension.js:6945-6970`): distanceFrom reduced by lostHp
    if has_ext_skill(state, catalog, from, "ssr_zhuiji")
        || flag_val(state, from, "l25_zhuiji_active") > 0
    {
        let lost = (state.player(from).max_hp as i32 - state.player(from).hp).max(0);
        dist -= lost;
    }
    // `ssr_hujia` (`extension.js:11005-11097`): `feiying` +1 distance to `to`
    if flag_val(state, to, "l25_hujia_feiying") > 0 {
        dist += 1;
    }
    // `ssr_yijue` (`extension.js:16455-16578`): temp `mashu` -1 distanceFrom
    if flag_val(state, from, "l25_yijue_mashu") > 0 {
        dist -= 1;
    }
    if (has_ext_skill(state, catalog, from, "ssr_newyicong")
        || flag_val(state, from, "l25_newyicong_active") > 0)
        && flag_val(state, to, "l25_yucong_mounted") > 0
    {
        dist -= 98;
    }
    dist
}

pub(super) fn apply_sha_use_limit(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    mut cur: i32,
    unlimited: bool,
) -> (i32, bool) {
    // `ssr_jijiang` (`extension.js:7619-7678`), `ssr_jiwu` (`extension.js:11264-11351`), `ssr_liegong` (`extension.js:11352-11420`)
    cur += flag_val(state, player, "l25_jijiang_sha_plus1") as i32;
    cur += flag_val(state, player, "l25_jiwu_sha_plus") as i32;
    cur += flag_val(state, player, "l25_liegong_sha_plus") as i32;
    // `ssr_yjixi` (`extension.js:15808-15904`): +1 Sha limit when owner does not have Yuxi equipped
    let has_yuxi_card = state.player(player).equips.iter().any(|&c| {
        catalog
            .cards
            .get(c as usize)
            .is_some_and(|cd| cd.name == "ssr_yongsi_yuxi_card")
    });
    if has_ext_skill(state, catalog, player, "ssr_yjixi")
        && flag_val(state, player, "l25_yjixi_has_yuxi") == 0
        && !has_yuxi_card
    {
        cur += 1;
    }
    cur += flag_val(state, player, "l25_yjixi_sha_plus1") as i32;
    cur += flag_val(state, player, "l25_xuanhuo_sha_buff") as i32;
    (cur, unlimited)
}

pub(super) fn apply_sha_extra_targets(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    kind: CardKind,
    mut extra: i32,
) -> i32 {
    // `ssr_huchi` (`extension.js:18641-18704`): Duel can target +1 extra player
    if kind == CardKind::Duel
        && (has_ext_skill(state, catalog, player, "ssr_huchi")
            || flag_val(state, player, "l25_huchi_active") > 0)
    {
        extra += 1;
    }
    if kind.is_sha_like() && flag_val(state, player, "l25_yanji_extra_target") > 0 {
        extra += 1;
    }
    extra
}

pub(super) fn apply_card_distance_unlimited(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    _card: CardId,
    kind: CardKind,
    mut unlimited: bool,
) -> bool {
    // `ssr_jinfan` (`extension.js:6535-6613`): Sha vs targets with hand.len() < `铃` count ignores distance
    if kind.is_sha_like() {
        let ling = flag_val(state, player, "l25_jinfan_ling").max(
            state
                .vm_aux
                .piles
                .get(&(player, "ssr_jinfan".to_string()))
                .map_or(0, |p| p.len() as u32),
        );
        if ling > 0
            && (0..state.players.len() as PlayerId).any(|p| {
                p != player
                    && state.player(p).alive
                    && (state.player(p).hand.len() as u32) < ling
            })
        {
            unlimited = true;
        }
    }
    // `ssr_bailun` (`extension.js:10151-10646`): tricks in PlayPhase ignore distance when `l25_bailun_trick_boost > 0`
    if !kind.is_basic_kind()
        && kind.equip_slot().is_none()
        && flag_val(state, player, "l25_bailun_trick_boost") > 0
        && state.turn.phase == Phase::Play
    {
        unlimited = true;
    }
    // `ssr_liegong` (`extension.js:11352-11420`): Sha has no distance limit
    if kind.is_sha_like()
        && (has_ext_skill(state, catalog, player, "ssr_liegong")
            || flag_val(state, player, "l25_liegong_active") > 0)
    {
        unlimited = true;
    }
    unlimited
}

pub(super) fn apply_card_target_canceled(
    state: &GameStateCore,
    catalog: &Catalog,
    actor: PlayerId,
    target: PlayerId,
    kind: CardKind,
    mut canceled: bool,
) -> bool {
    let is_trick = !kind.is_basic_kind() && kind.equip_slot().is_none();
    // `ssr_newrende` (`extension.js:20335-20463`): actor with `l25_rende_forbid_dmg_to == target + 1` cannot target `target` with damage cards
    if flag_val(state, actor, "l25_rende_forbid_dmg_to") == u32::from(target) + 1
        && (kind.is_sha_like()
            || matches!(
                kind,
                CardKind::Duel
                    | CardKind::Liuxinghuoyu
                    | CardKind::SavageAttack
                    | CardKind::ArcheryAttack
            ))
    {
        canceled = true;
    }
    // `ssr_biyue` (`extension.js:15755-15807`): immune to tricks mentioning Sha (`Duel`/`SavageAttack`/`ArcheryAttack`/`Jiedao`) or when `l25_biyue_trick_ward > 0`
    if is_trick
        && (flag_val(state, target, "l25_biyue_trick_ward") > 0
            || (has_ext_skill(state, catalog, target, "ssr_biyue")
                && matches!(
                    kind,
                    CardKind::Duel
                        | CardKind::SavageAttack
                        | CardKind::ArcheryAttack
                        | CardKind::Jiedao
                )))
    {
        canceled = true;
    }
    // `ssr_yanji` (`extension.js:15905-15984`): taunted actor can only target Yuan Shu with Sha
    let taunted = flag_val(state, actor, "l25_yanji_taunted_by");
    if kind.is_sha_like() && taunted > 0 && u32::from(target) + 1 != taunted {
        canceled = true;
    }
    // `ssr_newbaiyi` (`extension.js:16579-16686`): `渡` holder cancels incoming tricks and red Sha from others
    let has_baiyi_du = flag_val(state, target, "l25_baiyi_du") > 0
        || state
            .vm_aux
            .piles
            .get(&(target, "ssr_newbaiyi".to_string()))
            .is_some_and(|p| !p.is_empty());
    if actor != target && has_baiyi_du {
        if is_trick {
            canceled = true;
        } else if kind.is_sha_like()
            && (kind == CardKind::FireSha
                || flag_val(state, actor, "l25_baiyi_red_sha") > 0
                || state.player(actor).hand.iter().any(|&c| {
                    catalog.card_kind(c).is_sha_like()
                        && matches!(
                            catalog.effective_card_suit(state, actor, c),
                            Suit::Heart | Suit::Diamond
                        )
                }))
        {
            canceled = true;
        }
    }
    // `ssr_newweimu` (`extension.js:22026-22146`): while turned over, cancel incoming tricks from others
    if actor != target
        && is_trick
        && has_ext_skill(state, catalog, target, "ssr_newweimu")
        && (state.player(target).face_down || flag_val(state, target, "l25_weimu_active") > 0)
    {
        canceled = true;
    }
    // `ssr_newlihun` (`extension.js:15578-15754`): `qianxing` stealth cancels others' cards
    if actor != target && flag_val(state, target, "l25_lihun_qianxing") > 0 {
        canceled = true;
    }
    canceled
}

pub(super) fn apply_card_use_canceled(
    state: &GameStateCore,
    catalog: &Catalog,
    actor: PlayerId,
    kind: CardKind,
    in_hand: bool,
    mut canceled: bool,
) -> bool {
    // `ssr_newfanjian` (`extension.js:6971-7090`) & `ssr_shichou` (`extension.js:20615-20783`): forbid using cards of `l25_fanjian_forbid_suit` / `l25_shichou_forbid_suit` (`1..=4`)
    let f1 = flag_val(state, actor, "l25_fanjian_forbid_suit");
    let f2 = flag_val(state, actor, "l25_shichou_forbid_suit");
    for sc in [f1, f2] {
        if (1..=4).contains(&sc)
            && state
                .player(actor)
                .hand
                .iter()
                .chain(state.player(actor).equips.iter())
                .any(|&c| {
                    (catalog.card_kind(c) == kind || state.player(actor).hand.len() == 1)
                        && suit_code(catalog.effective_card_suit(state, actor, c)) == sc
                })
        {
            canceled = true;
        }
    }
    // `ssr_newzhenjun` (`extension.js:11641-11685`): `l25_zhenjun_sealed > 0` forbids black handcards
    if in_hand
        && flag_val(state, actor, "l25_zhenjun_sealed") > 0
        && state.player(actor).hand.iter().any(|&c| {
            (catalog.card_kind(c) == kind || state.player(actor).hand.len() == 1)
                && matches!(
                    catalog.effective_card_suit(state, actor, c),
                    Suit::Spade | Suit::Club
                )
        })
    {
        canceled = true;
    }
    // `ssr_kuiwei`: `flag(actor, "l25_kuiwei_forbid_basic") > 0` cancels basic handcards
    if in_hand && kind.is_basic_kind() && flag_val(state, actor, "l25_kuiwei_forbid_basic") > 0 {
        canceled = true;
    }
    // `ssr_zhijian`: if `in_hand` and `l25_zhijian_locked_card > 0` and card != locked_card, cancel
    let locked_card = flag_val(state, actor, "l25_zhijian_locked_card");
    if in_hand
        && locked_card > 0
        && !state.player(actor).hand.iter().any(|&c| {
            u32::from(c) + 1 == locked_card
                && (catalog.card_kind(c) == kind || state.player(actor).hand.len() == 1)
        })
    {
        canceled = true;
    }
    canceled
}

pub(super) fn apply_ignore_armor(
    state: &GameStateCore,
    source: PlayerId,
    mut ignore: bool,
) -> bool {
    // `ssr_jinfan` (`extension.js:6535-6613`): Sha vs targets with hand.len() < `铃` count ignores armor
    let ling = flag_val(state, source, "l25_jinfan_ling").max(
        state
            .vm_aux
            .piles
            .get(&(source, "ssr_jinfan".to_string()))
            .map_or(0, |p| p.len() as u32),
    );
    if ling > 0
        && (0..state.players.len() as PlayerId).any(|p| {
            p != source && state.player(p).alive && (state.player(p).hand.len() as u32) < ling
        })
    {
        ignore = true;
    }
    // `ssr_newzhenjun` (`extension.js:11641-11685`): ignore armor when target has `l25_zhenjun_sealed > 0`
    if flag_val(state, source, "l25_zhenjun_sealed") > 0
        || (0..state.players.len() as PlayerId).any(|p| {
            p != source && state.player(p).alive && flag_val(state, p, "l25_zhenjun_sealed") > 0
        })
    {
        ignore = true;
    }
    // `ssr_newdingsheng`: ArcheryAttack ignores armor
    if flag_val(state, source, "l25_dingsheng_active") > 0
        || flag_val(state, source, "l25_newdingsheng_active") > 0
        || flag_val(state, source, "l25_dingsheng_ignore_armor") > 0
        || flag_val(state, source, "l25_shiyi_jue") >= 3
    {
        ignore = true;
    }
    ignore
}

pub(super) fn apply_damage_amount(
    state: &GameStateCore,
    catalog: &Catalog,
    source: Option<PlayerId>,
    target: PlayerId,
    card: Option<CardId>,
    _nature: DamageNature,
    mut cur: i32,
) -> i32 {
    if cur <= 0 {
        return 0;
    }
    let ckind = card.map(|c| catalog.card_kind(c));
    let is_sha_or_duel =
        ckind.is_some_and(|k| k.is_sha_like() || k == CardKind::Duel) || card.is_none();
    if let Some(src) = source {
        // `ssr_luoyi` (`extension.js:15031-15065`): +1 damage on Sha/Duel
        if is_sha_or_duel
            && (has_ext_skill(state, catalog, src, "ssr_luoyi")
                || flag_val(state, src, "l25_luoyi_active") > 0)
        {
            cur += 1;
        }
        // `ssr_fenchuan` (`extension.js:15517-15577`): +1 damage on boosted Sha
        if ckind.is_some_and(CardKind::is_sha_like)
            && flag_val(state, src, "l25_fenchuan_dmg_boost") > 0
        {
            cur += 1;
        }
    }
    if flag_val(state, target, "l25_tongque_prevent") > 0
        || flag_val(state, target, "l25_tianxiang_prevent") > 0
        || flag_val(state, target, "l25_newluoyi_prevent") > 0
    {
        return 0;
    }
    cur += flag_val(state, target, "l25_tianxiang_extra_dmg") as i32;
    // `ssr_jushou`: cap `cur` at `1` when `cur > 1 && (has_ext_skill(target, "ssr_jushou") || flag(target, "l25_jushou_active") > 0)`
    if cur > 1
        && (has_ext_skill(state, catalog, target, "ssr_jushou")
            || flag_val(state, target, "l25_jushou_active") > 0)
    {
        cur = 1;
    }
    cur
}

pub(super) fn apply_card_unrespondable(
    state: &GameStateCore,
    catalog: &Catalog,
    source: PlayerId,
    kind: CardKind,
    mut unrespondable: bool,
) -> bool {
    let is_trick = !kind.is_basic_kind() && kind.equip_slot().is_none();
    // `ssr_bailun` (`extension.js:10151-10646`): tricks in PlayPhase cannot be responded when `l25_bailun_trick_boost > 0`
    if is_trick && flag_val(state, source, "l25_bailun_trick_boost") > 0 && state.turn.phase == Phase::Play {
        unrespondable = true;
    }
    // `ssr_qicai` (`extension.js:21922-22025`): tricks used by owner cannot be responded
    if is_trick
        && (has_ext_skill(state, catalog, source, "ssr_qicai")
            || flag_val(state, source, "l25_qicai_active") > 0)
    {
        unrespondable = true;
    }
    // `ssr_newdingsheng`: ArcheryAttack is unrespondable when `has_ext_skill(source, "ssr_newdingsheng")`
    if kind == CardKind::ArcheryAttack
        && (has_ext_skill(state, catalog, source, "ssr_newdingsheng")
            || flag_val(state, source, "l25_dingsheng_active") > 0
            || flag_val(state, source, "l25_newdingsheng_active") > 0)
    {
        unrespondable = true;
    }
    // `ssr_wusheng` (`extension.js:16395-16454`): Jiu-boosted Sha is unrespondable
    if kind.is_sha_like()
        && (flag_val(state, source, "l25_wusheng_drunk_unresp") > 0
            || flag_val(state, source, "l25_fenchuan_unresp") > 0
            || (has_ext_skill(state, catalog, source, "ssr_wusheng")
                && state.turn.jiu_active))
    {
        unrespondable = true;
    }
    // `ssr_baonue`: Sha unrespondable when `(has_ext_skill(source, "ssr_baonue") && state.player(source).hp > 2) || flag(source, "l25_baonue_unresp") > 0`
    if kind.is_sha_like()
        && ((has_ext_skill(state, catalog, source, "ssr_baonue") && state.player(source).hp > 2)
            || flag_val(state, source, "l25_baonue_unresp") > 0)
    {
        unrespondable = true;
    }
    // `ssr_liegong` / `ssr_tieji` / `ssr_newpozhen` / `ssr_wushuang` / `ssr_shiyong`
    if kind.is_sha_like()
        && (flag_val(state, source, "l25_liegong_unresp") > 0
            || flag_val(state, source, "l25_tieji_unresp") > 0
            || flag_val(state, source, "l25_pozhen_unresp") > 0
            || flag_val(state, source, "l25_wushuang_unresp") > 0
            || flag_val(state, source, "l25_shiyong_red_sha") > 0
            || (0..state.players.len() as PlayerId).any(|p| {
                p != source
                    && state.player(p).alive
                    && (has_ext_skill(state, catalog, p, "ssr_shiyong")
                        || flag_val(state, p, "l25_shiyong_active") > 0)
                    && state.player(p).hand.len() < 2
                    && state.player(source).hand.iter().any(|&c| {
                        catalog.card_kind(c).is_sha_like()
                            && matches!(
                                catalog.effective_card_suit(state, source, c),
                                Suit::Heart | Suit::Diamond
                            )
                    })
            }))
    {
        unrespondable = true;
    }
    unrespondable
}
