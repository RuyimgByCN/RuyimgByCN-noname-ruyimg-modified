//! Intrinsic resolution for named extension cards (`U05`, `U07`, `L01`, `L06`, `L11`, `L20`,
//! `L25`, `L28`) that share `CardKind::Wuzhong` or `CardKind::Jiguan` under frozen `ABI 759` /
//! `NGSA 107`. Standard `"无中生有"` (`"wuzhong"`) returns `None` so the kernel's
//! `WuzhongMachine` executes unchanged.

use crate::catalog::{CardKind, SkillMachine};
use crate::event::{CardUseData, DamageNature, EventData};
use crate::ids::{CardId, PlayerId};
use crate::machine::Ctx;

fn finish_table_card(ctx: &mut Ctx<'_>, card: CardId) {
    if ctx.core.table.contains(&card) {
        ctx.core.table.retain(|&x| x != card);
        ctx.to_discard(card);
    }
}

fn set_flag(ctx: &mut Ctx<'_>, seat: PlayerId, key: &str, val: u32) {
    ctx.core.vm_aux.flags.insert((seat, key.to_string()), val);
}

fn get_flag(ctx: &Ctx<'_>, seat: PlayerId, key: &str) -> u32 {
    ctx.core
        .vm_aux
        .flags
        .get(&(seat, key.to_string()))
        .copied()
        .unwrap_or(0)
}

fn first_target_or_other(ctx: &Ctx<'_>, u: &CardUseData) -> Option<PlayerId> {
    if let Some(&t) = u.targets.first() {
        if t != u.player && ctx.core.player(t).alive {
            return Some(t);
        }
    }
    ctx.core
        .seat_order_from(u.player)
        .into_iter()
        .find(|&p| p != u.player && ctx.core.player(p).alive)
}

fn recover_if_damaged(ctx: &mut Ctx<'_>, seat: PlayerId, amount: u32) -> bool {
    if ctx.core.player(seat).hp < ctx.core.player(seat).max_hp as i32 {
        ctx.recover(seat, amount, None);
        true
    } else {
        false
    }
}

fn change_hujia(ctx: &mut Ctx<'_>, seat: PlayerId, delta: i32) {
    let cur = i32::from(ctx.core.player(seat).hujia);
    ctx.core.player_mut(seat).hujia = (cur + delta).clamp(0, 99) as u8;
}

fn change_max_hp(ctx: &mut Ctx<'_>, seat: PlayerId, delta: i32) {
    let cur = ctx.core.player(seat).max_hp as i32;
    let next = (cur + delta).max(1) as u32;
    ctx.core.player_mut(seat).max_hp = next;
    if ctx.core.player(seat).hp > next as i32 {
        ctx.core.player_mut(seat).hp = next as i32;
    }
}

fn discard_up_to(ctx: &mut Ctx<'_>, seat: PlayerId, max_n: usize) -> usize {
    let mut moved = 0usize;
    while moved < max_n {
        if let Some(c) = ctx.core.player(seat).hand.first().copied() {
            ctx.take_hand_card(seat, c);
            ctx.to_discard(c);
            moved += 1;
        } else if let Some(c) = ctx.core.player(seat).equips.first().copied() {
            ctx.take_owned_card(seat, c);
            ctx.to_discard(c);
            moved += 1;
        } else {
            break;
        }
    }
    moved
}

fn steal_one_to_hand(ctx: &mut Ctx<'_>, owner: PlayerId, target: PlayerId) -> bool {
    if let Some(c) = ctx.core.player(target).hand.first().copied() {
        ctx.take_hand_card(target, c);
        ctx.core.player_mut(owner).hand.push(c);
        true
    } else if let Some(c) = ctx.core.player(target).equips.first().copied() {
        ctx.take_owned_card(target, c);
        ctx.core.player_mut(owner).hand.push(c);
        true
    } else {
        false
    }
}

fn deal_damage(
    ctx: &mut Ctx<'_>,
    source: PlayerId,
    target: PlayerId,
    amount: i32,
    nature: DamageNature,
) {
    ctx.emit(EventData::Damage(crate::machine::damage_data_with(
        Some(source),
        target,
        amount,
        nature,
        None,
    )));
}

fn base_card_key(raw: &str) -> &str {
    // Strip optional `<prefix>` / `_<index>` suffix added by `cards.json` bridging if present.
    let s = if let Some((stem, tail)) = raw.rsplit_once('_') {
        if !tail.is_empty() && tail.bytes().all(|b| b.is_ascii_digit()) {
            stem
        } else {
            raw
        }
    } else {
        raw
    };
    s
}

/// 共享火攻弃置并造成火焰伤害结算（`intrinsic_named_ext` 与虚卡/镜像复刻共用）。
pub(in crate::content::host) fn resolve_huogong_effect(ctx: &mut Ctx<'_>, u: &CardUseData) {
    let owner = u.player;
    if let Some(t) = first_target_or_other(ctx, u) {
        if discard_up_to(ctx, owner, 1) > 0 {
            deal_damage(ctx, owner, t, 1, DamageNature::Fire);
        }
    }
}

/// Shared predicate: custom `CardKind::Jiguan` keys that resolve without `wrap_nullify_window`.
pub(in crate::content::host) fn is_named_jiguan_unwrapped(raw_name: &str) -> bool {
    let key = base_card_key(raw_name);
    let key = key
        .strip_prefix("gj_")
        .or_else(|| key.strip_prefix("swd_"))
        .unwrap_or(key);
    matches!(
        key,
        "yanjiadan_heart"
            | "yanjiadan_diamond"
            | "yanjiadan_club"
            | "yanjiadan_spade"
            | "lingjiandai"
    )
}

/// Shared predicate: custom `CardKind::Wuzhong` keys handled by `try_named_ext_card`
/// (resolved directly without `wrap_nullify_window`).
pub(in crate::content::host) fn is_named_wuzhong_unwrapped(
    catalog: &crate::catalog::Catalog,
    card: CardId,
) -> bool {
    if crate::content::host::is_hearth_named_wuzhong_unwrapped(catalog, card) {
        return true;
    }
    let Some(raw_name) = catalog.cards.get(card as usize).map(|c| c.name) else {
        return false;
    };
    if raw_name == "无中生有" || raw_name == "wuzhong" || raw_name.starts_with("wuzhong_") {
        return false;
    }
    if base_card_key(raw_name) == "yiyi" {
        return catalog
            .skill_by_name(crate::content::host::L01_FIRST_KS_KEY)
            .is_none();
    }
    matches!(
        base_card_key(raw_name),
        "yiyi" | "txhj_yiyi"
            | "diaobingqianjiang"
            | "zhengsu_leijin"
            | "zhengsu_mingzhi"
            | "zhengsu_bianzhen"
            | "th_taipingyaoshu"
            | "ssr_newsance_xicheng"
            | "ssr_newsance_weicheng"
            | "ssr_newsance_shoucheng"
            | "ssr_jinfan"
            | "txhj_zhijizhibi"
            | "txhj_yuanjiao"
            | "txhj_diaohu"
            | "txhj_shengdong"
            | "txzhaoshu_skill"
            | "txzhaoshu_global"
            | "huogong"
            | "mtg_du"
            | "mtg_duzhao"
            | "g_mtg_duzhao"
            | "mtg_cangbaohaiwan"
            | "g_mtg_cangbaohaiwan"
            | "gw_hudiewu"
            | "gw_xinsheng"
            | "gw_huangjiashenpan"
            | "gw_chongci"
            | "gw_butianshu"
            | "gw_shizizhaohuan"
            | "gw_zuihouyuanwang"
            | "gw_zirankuizeng"
            | "gwmaoxian_falanxisika"
            | "gwmaoxian_huoge"
            | "gw_fuyuan"
            | "gwmaoxian_telisi"
            | "gwmaoxian_aisinie"
            | "gw_xianzumaijiu"
            | "gwjinli_jiu"
            | "gw_leizhoushu"
            | "gwmaoxian_yenaifa"
            | "gw_yigeniyin"
            | "gw_zhuoshao"
            | "gw_zhongmozhizhan"
            | "gw_ansha"
            | "gw_ganhan"
            | "gwmaoxian_yioufeisi"
            | "gwmaoxian_luoqi"
            | "gwmaoxian_jieluote"
            | "gwmaoxian_kuite"
            | "gwmaoxian_haluo"
            | "gwmaoxian_airuiting"
            | "gw_wuyao"
            | "gw_aerdeyin"
            | "gwmaoxian_laduoweide"
            | "gw_tunshi"
            | "gw_niuquzhijing"
            | "gw_youer"
            | "gw_tongdi"
            | "gwmaoxian_enxier"
            | "gwmaoxian_fulisi"
            | "gwmaoxian_kaerweite"
            | "gwmaoxian_gaier"
            | "gw_nuhaifengbao"
            | "gw_baishuang"
            | "gw_baobaoshu"
            | "gwmaoxian_dagong"
            | "gw_guaiwuchaoxue"
            | "gwmaoxian_hengsaite"
            | "gwmaoxian_fuertaisite"
            | "gwmaoxian_bulanwang"
            | "gw_lang"
            | "gw_dudayuanshuai1"
            | "gw_dudayuanshuai2"
            | "shandianjian"
            | "ht_shandianjian"
            | "hsqingyu_shandian"
            | "hsqizhou_huo"
            | "hsdusu_huoyanhua"
            | "hsjixie_zhadan"
            | "hsmengjing_feicuiyoulong"
            | "hsshenqi_kongbusangzhong"
            | "hsmengjing_suxing"
            | "hsqizhou_shui"
            | "hsmengjing_huanxiaojiemei"
            | "tuteng3"
            | "hsqizhou_tu"
            | "hsqingyu_hufu"
            | "tuteng2"
            | "tuteng6"
            | "hsqingyu_zhanfang"
            | "hsshenqi_nengliangzhiguang"
            | "hsqingyu_feibiao"
            | "hsdusu_xueji"
            | "hsmengjing_mengjing"
            | "hszuzhou_nvwudeganguo"
            | "hszuzhou_wushushike"
            | "hszuzhou_guhuo"
            | "shenenshu"
            | "ht_shenenshu"
            | "hsfashu_anyingjingxiang"
            | "hsfashu_buwendingyibian"
            | "hsqizhou_feng"
            | "hsdusu_kuyecao"
            | "hsdusu_huangxuecao"
            | "hsdusu_shinancao"
            | "hsbaowu_cangbaotu"
            | "hsbaowu_huangjinyuanhou"
            | "hsshenqi_morijingxiang"
            | "hsmengjing_mengye"
            | "hszuzhou_nvwudepingguo"
            | "hszuzhou_nvwudexuetu"
            | "tuteng1"
            | "tuteng4"
            | "tuteng5"
            | "tuteng7"
            | "tuteng8"
    )
}

/// Intercepts `yanjiadan_*` (`character/gujian/card.js:4-198`) and `lingjiandai`
/// (`card/swd/card.js:1062-1098`) before `CardKind::Jiguan`'s SWD fallback.
pub(super) fn try_named_jiguan_card(
    ctx: &mut Ctx<'_>,
    u: &CardUseData,
    card: CardId,
    raw_name: &str,
) -> Option<Option<Box<dyn SkillMachine>>> {
    let key = base_card_key(raw_name);
    let key = key.strip_prefix("gj_").or_else(|| key.strip_prefix("swd_")).unwrap_or(key);
    let owner = u.player;
    match key {
        // character/gujian/card.js:4-205: yanjiadan_{heart,diamond,club,spade} 3选1衍生机关
        "yanjiadan_heart" | "yanjiadan_diamond" | "yanjiadan_club" | "yanjiadan_spade" => {
            set_flag(ctx, owner, &format!("u05_card_used:{key}"), 1);
            Some(Some(Box::new(crate::content::host::HostMachine {
                def_key: key.to_string(),
                behavior: crate::content::host::HostBehavior::U05GujianCardBatch,
                owner,
                target: u.targets.first().copied(),
                card: Some(card),
                cards: Vec::new(),
                targets: u.targets.clone(),
                stage: crate::content::host::HostStage::Act,
            })))
        }
        // card/swd/card.js:1062-1098: lingjiandai — gain 1 Hslingjian card from deck/discard (or draw 1)
        "lingjiandai" => {
            set_flag(ctx, owner, "u05_card_used:lingjiandai", 1);
            if let Some(pos) = ctx
                .core
                .deck
                .iter()
                .position(|&c| ctx.catalog.card_kind(c) == CardKind::Hslingjian)
            {
                let found = ctx.core.deck.remove(pos);
                ctx.core.player_mut(owner).hand.push(found);
            } else if let Some(pos) = ctx
                .core
                .discard
                .iter()
                .position(|&c| ctx.catalog.card_kind(c) == CardKind::Hslingjian)
            {
                let found = ctx.core.discard.remove(pos);
                ctx.core.player_mut(owner).hand.push(found);
            } else {
                ctx.start_draw(owner, 1);
            }
            finish_table_card(ctx, card);
            Some(None)
        }
        _ => None,
    }
}

/// Intercepts custom named extension cards mapped to `CardKind::Wuzhong`.
pub(super) fn try_named_ext_card(
    ctx: &mut Ctx<'_>,
    u: &CardUseData,
    card: CardId,
    effective_kind: CardKind,
) -> Option<Option<Box<dyn SkillMachine>>> {
    if effective_kind == CardKind::Youdishenru {
        let opt_card = (card != u32::MAX).then_some(card);
        let defender = u.player;
        let attacker = ctx
            .core
            .vm_aux
            .flags
            .remove(&(defender, "yd_pending_source".to_string()))
            .and_then(|v| v.checked_sub(1))
            .map(|v| v as PlayerId);
        let Some(attacker) = attacker else {
            if let Some(cid) = opt_card {
                finish_table_card(ctx, cid);
            }
            return Some(None);
        };
        let wuxie_cid = opt_card.unwrap_or_else(|| {
            ctx.catalog
                .cards
                .iter()
                .position(|c| c.kind == CardKind::Youdishenru)
                .unwrap_or(0) as CardId
        });
        let effect = Box::new(crate::content::host::HostMachine {
            def_key: "youdishenru".to_string(),
            behavior: crate::content::host::HostBehavior::WudianYouzhan,
            owner: defender,
            target: Some(attacker),
            card: opt_card,
            cards: Vec::new(),
            targets: vec![defender],
            stage: crate::content::host::HostStage::HuanrenPick,
        });
        return Some(Some(Box::new(crate::kernel::NullifyWindowMachine::new(
            wuxie_cid,
            defender,
            false,
            ctx.core.seat_order_from(defender),
            false,
            true,
            opt_card.is_some(),
            Some(effect),
        ))));
    }
    if effective_kind != CardKind::Wuzhong {
        return None;
    }
    let key = if card == u32::MAX {
        "diaobingqianjiang"
    } else {
        if u.use_as.is_some() {
            return None;
        }
        let raw_name = ctx.catalog.cards.get(card as usize)?.name;
        if raw_name == "无中生有" || raw_name == "wuzhong" || raw_name.starts_with("wuzhong_") {
            return None;
        }
        let k = base_card_key(raw_name);
        k.strip_prefix("wd_").unwrap_or(k)
    };
    let owner = u.player;

    match key {
        // L01 AI优化/extension.js:9205-9262: 以逸待劳（guozhan/versus 全体友方，其余 1..=3 目标，各摸 2 再交互弃 2 he）；
        // 当仅加载旧 decision34 共存包未加载 L01 ai-tuning 时保持旧版兼容路径
        "yiyi" if ctx.catalog.skill_by_name(crate::content::host::L01_FIRST_KS_KEY).is_some() => {
            let (auto_allies, candidates) =
                ctx.core.yiyi_target_candidates(ctx.catalog, owner);
            let mut targets: Vec<PlayerId> = if auto_allies || u.targets.is_empty() {
                if auto_allies {
                    candidates
                } else {
                    vec![owner]
                }
            } else {
                let mut ordered = Vec::new();
                for p in candidates {
                    if u.targets.contains(&p) && !ordered.contains(&p) {
                        ordered.push(p);
                    }
                }
                if ordered.is_empty() {
                    vec![owner]
                } else {
                    ordered
                }
            };
            if targets.is_empty() {
                targets.push(owner);
            }
            let opt_card = (card != u32::MAX).then_some(card);
            return Some(Some(Box::new(crate::content::host::HostMachine {
                def_key: "yiyi".to_string(),
                behavior: crate::vocabulary::host::HostBehavior::L01YiyiCard,
                owner,
                target: None,
                card: opt_card,
                cards: Vec::new(),
                targets,
                stage: crate::content::host::HostStage::Act,
            })));
        }
        // L01 兼容兜底 & L11 太虚幻境/extension_card.js:334: 以逸待劳 (draw 2, discard 2)
        "yiyi" | "txhj_yiyi" => {
            set_flag(ctx, owner, "d34_card_used:yiyi", 1);
            ctx.start_draw(owner, 2);
            let _ = discard_up_to(ctx, owner, 2);
        }
        // U07 玩点论杀/card/card.js:4-142: 调兵遣将 — 使用者摸 1，亮出牌堆顶 ceil(alive/2) 张，按座次构建完整合法目标队列，各目标可选换牌
        "diaobingqianjiang" => {
            let opt_card = (card != u32::MAX).then_some(card);
            let targets: Vec<PlayerId> = if u.targets.is_empty() {
                ctx.core
                    .seat_order_from(owner)
                    .into_iter()
                    .filter(|&p| {
                        ctx.core.player(p).alive
                            && (p == owner || !ctx.core.player(p).hand.is_empty())
                    })
                    .collect()
            } else {
                ctx.core
                    .seat_order_from(owner)
                    .into_iter()
                    .filter(|p| u.targets.contains(p) && ctx.core.player(*p).alive)
                    .collect()
            };
            return Some(Some(Box::new(crate::content::host::HostMachine {
                def_key: crate::vocabulary::host::WUDIAN_DIAOBING_KEY.to_string(),
                behavior: crate::vocabulary::host::HostBehavior::WudianDiaobingCard,
                owner,
                target: None,
                card: opt_card,
                cards: Vec::new(),
                targets,
                stage: crate::content::host::HostStage::Act,
            })));
        }
        // L20 标记补充/extension.js:6319-6321: 整肃奖励牌 (recover 1 if damaged, else draw 2)
        "zhengsu_leijin" | "zhengsu_mingzhi" | "zhengsu_bianzhen" => {
            set_flag(ctx, owner, "l20_zhengsu_active", 1);
            set_flag(ctx, owner, &format!("l20_card:{key}"), 1);
            if !recover_if_damaged(ctx, owner, 1) {
                ctx.start_draw(owner, 2);
            }
        }
        // L28 雷霆万钧/extension.js:8968: 太平要术 (+2 hand limit & elemental immunity + draw 2)
        "th_taipingyaoshu" => {
            set_flag(ctx, owner, "l28_taipingyaoshu_equipped", 1);
            ctx.start_draw(owner, 2);
        }
        // L25 祖安武将/extension.js:28179-28410: 三策 & 锦帆
        "ssr_newsance_xicheng" | "ssr_newsance_weicheng" | "ssr_newsance_shoucheng" => {
            if key == "ssr_newsance_weicheng" {
                set_flag(ctx, owner, "l25_sance_weicheng", 1);
            }
            return Some(Some(Box::new(crate::content::host::HostMachine {
                def_key: key.to_string(),
                behavior: crate::vocabulary::host::HostBehavior::L25ZuanBatchA,
                owner,
                target: first_target_or_other(ctx, u),
                card: Some(card),
                cards: Vec::new(),
                targets: u.targets.clone(),
                stage: crate::content::host::HostStage::Act,
            })));
        }
        "ssr_jinfan" => {
            if let Some(t) = first_target_or_other(ctx, u) {
                if !steal_one_to_hand(ctx, owner, t) {
                    ctx.start_draw(owner, 1);
                }
            }
        }
        // L11 太虚幻境/extension_card.js:233-2041
        "txhj_zhijizhibi" => {
            set_flag(ctx, owner, "l11_zhijizhibi_peeked", 1);
            ctx.start_draw(owner, 1);
        }
        "txhj_yuanjiao" => {
            if let Some(t) = first_target_or_other(ctx, u) {
                ctx.start_draw(t, 1);
            }
            ctx.start_draw(owner, 3);
        }
        "txhj_diaohu" => {
            if let Some(t) = first_target_or_other(ctx, u) {
                ctx.turn_over(t);
            }
            ctx.start_draw(owner, 1);
        }
        "txhj_shengdong" => {
            if let Some(t) = first_target_or_other(ctx, u) {
                let _ = steal_one_to_hand(ctx, owner, t);
            }
            ctx.start_draw(owner, 1);
        }
        "txzhaoshu_skill" | "txzhaoshu_global" => {
            let n = get_flag(ctx, owner, "l11_zhaoshu_suits") + 1;
            set_flag(ctx, owner, "l11_zhaoshu_suits", n);
            if n >= 2 {
                if let Some(t) = first_target_or_other(ctx, u) {
                    deal_damage(ctx, owner, t, 2, DamageNature::Thunder);
                }
                set_flag(ctx, owner, "l11_zhaoshu_suits", 0);
            } else {
                ctx.start_draw(owner, 1);
            }
        }
        "huogong" => resolve_huogong_effect(ctx, u),
        // U05 MTG land & token cards (`card/mtg/card.js:99, 391`, `card/yongjian.js:10`)
        "mtg_du" => {
            finish_table_card(ctx, card);
            return Some(None);
        }
        "mtg_duzhao" | "g_mtg_duzhao" | "mtg_cangbaohaiwan" | "g_mtg_cangbaohaiwan" => {
            return Some(Some(Box::new(crate::content::host::HostMachine {
                def_key: key.to_string(),
                behavior: crate::content::host::HostBehavior::U05MtgLandExtraBatch,
                owner,
                target: None,
                card: Some(card),
                cards: Vec::new(),
                targets: Vec::new(),
                stage: crate::content::host::HostStage::Act,
            })));
        }
        // U05 Zhenfa cards (`card/zhenfa/card.js:123-319`, nullify window wrapped)
        "fengyangzhen" | "zf_fengyangzhen" => {
            let targets: Vec<PlayerId> = ctx
                .core
                .alive_players()
                .filter(|&p| ctx.core.sieged(ctx.catalog, p))
                .collect();
            let effect = Box::new(crate::content::host::HostMachine {
                def_key: key.to_string(),
                behavior: crate::content::host::HostBehavior::U05MainStoneBatch,
                owner,
                target: None,
                card: Some(card),
                cards: Vec::new(),
                targets,
                stage: crate::content::host::HostStage::Act,
            });
            return Some(Some(Box::new(crate::kernel::wrap_nullify_window(
                ctx, card, owner, effect,
            ))));
        }
        "yunchuizhen" | "zf_yunchuizhen" => {
            let targets: Vec<PlayerId> = ctx
                .core
                .alive_players()
                .filter(|&p| {
                    ctx.core
                        .alive_players()
                        .any(|v| ctx.core.sieges(ctx.catalog, p, v))
                })
                .collect();
            let effect = Box::new(crate::content::host::HostMachine {
                def_key: key.to_string(),
                behavior: crate::content::host::HostBehavior::U05MainStoneBatch,
                owner,
                target: None,
                card: Some(card),
                cards: Vec::new(),
                targets,
                stage: crate::content::host::HostStage::Act,
            });
            return Some(Some(Box::new(crate::kernel::wrap_nullify_window(
                ctx, card, owner, effect,
            ))));
        }
        "shepanzhen" | "zf_shepanzhen" => {
            let effect = Box::new(crate::content::host::HostMachine {
                def_key: key.to_string(),
                behavior: crate::content::host::HostBehavior::U05MainStoneBatch,
                owner,
                target: None,
                card: Some(card),
                cards: Vec::new(),
                targets: Vec::new(),
                stage: crate::content::host::HostStage::Act,
            });
            return Some(Some(Box::new(crate::kernel::wrap_nullify_window(
                ctx, card, owner, effect,
            ))));
        }
        "longfeizhen" | "zf_longfeizhen" => {
            let targets: Vec<PlayerId> = if !u.targets.is_empty() {
                ctx.core
                    .seat_order_from(owner)
                    .into_iter()
                    .filter(|p| u.targets.contains(p) && ctx.core.player(*p).alive)
                    .collect()
            } else {
                let (next, prev) = ctx.core.alive_neighbors(owner);
                let mut t = Vec::new();
                if let Some(n) = next {
                    if n != owner {
                        t.push(n);
                    }
                }
                if let Some(p) = prev {
                    if p != owner && !t.contains(&p) {
                        t.push(p);
                    }
                }
                t
            };
            let effect = Box::new(crate::content::host::HostMachine {
                def_key: key.to_string(),
                behavior: crate::content::host::HostBehavior::U05MainStoneBatch,
                owner,
                target: None,
                card: Some(card),
                cards: Vec::new(),
                targets,
                stage: crate::content::host::HostStage::Act,
            });
            return Some(Some(Box::new(crate::kernel::wrap_nullify_window(
                ctx, card, owner, effect,
            ))));
        }
        "huyizhen" | "zf_huyizhen" => {
            let mut pairs: Vec<PlayerId> = Vec::new();
            let mut first_victim = None;
            for &victim in &u.targets {
                if !ctx.core.player(victim).alive {
                    continue;
                }
                if first_victim.is_none() {
                    first_victim = Some(victim);
                }
                for p in ctx.core.seat_order_from(owner) {
                    if ctx.core.player(p).alive && ctx.core.sieges(ctx.catalog, p, victim) {
                        pairs.push(victim);
                        pairs.push(p);
                    }
                }
            }
            let Some(primary_victim) = first_victim else {
                finish_table_card(ctx, card);
                return Some(None);
            };
            let effect = Box::new(crate::content::host::HostMachine {
                def_key: key.to_string(),
                behavior: crate::content::host::HostBehavior::U05MainStoneBatch,
                owner,
                target: Some(primary_victim),
                card: Some(card),
                cards: Vec::new(),
                targets: pairs,
                stage: crate::content::host::HostStage::Act,
            });
            return Some(Some(Box::new(crate::kernel::wrap_nullify_window(
                ctx, card, owner, effect,
            ))));
        }
        // U05 Gwent gold/silver/adventure cards (`card/gwent/card.js` & `character/gwent/card.js`)
        "gw_hudiewu" | "gw_xinsheng" | "gw_huangjiashenpan" | "gw_chongci"
        | "gw_butianshu" | "gw_shizizhaohuan" | "gw_zuihouyuanwang" | "gw_zirankuizeng"
        | "gwmaoxian_falanxisika" | "gwmaoxian_huoge" => {
            set_flag(ctx, owner, &format!("u05_card_used:{key}"), 1);
            ctx.start_draw(owner, 2);
        }
        "gw_fuyuan" | "gwmaoxian_telisi" | "gwmaoxian_aisinie" | "gw_xianzumaijiu" | "gwjinli_jiu" => {
            set_flag(ctx, owner, &format!("u05_card_used:{key}"), 1);
            let _ = recover_if_damaged(ctx, owner, 1);
            ctx.start_draw(owner, 1);
        }
        "gw_leizhoushu" | "gwmaoxian_yenaifa" => {
            set_flag(ctx, owner, &format!("u05_card_used:{key}"), 1);
            if let Some(t) = first_target_or_other(ctx, u) {
                deal_damage(ctx, owner, t, 1, DamageNature::Thunder);
            }
        }
        "gw_yigeniyin" | "gw_zhuoshao" | "gw_zhongmozhizhan" => {
            set_flag(ctx, owner, &format!("u05_card_used:{key}"), 1);
            if let Some(t) = first_target_or_other(ctx, u) {
                deal_damage(ctx, owner, t, 1, DamageNature::Fire);
            }
        }
        "gw_ansha" | "gw_ganhan" | "gwmaoxian_yioufeisi" | "gwmaoxian_luoqi"
        | "gwmaoxian_jieluote" | "gwmaoxian_kuite" | "gwmaoxian_haluo" | "gwmaoxian_airuiting"
        | "gw_wuyao" => {
            set_flag(ctx, owner, &format!("u05_card_used:{key}"), 1);
            if let Some(t) = first_target_or_other(ctx, u) {
                deal_damage(ctx, owner, t, 1, DamageNature::Normal);
            }
        }
        "gw_aerdeyin" | "gwmaoxian_laduoweide" => {
            set_flag(ctx, owner, &format!("u05_card_used:{key}"), 1);
            if let Some(t) = first_target_or_other(ctx, u) {
                ctx.turn_over(t);
            }
        }
        "gw_tunshi" => {
            set_flag(ctx, owner, "u05_card_used:gw_tunshi", 1);
            if ctx.core.player(owner).max_hp > 1 {
                change_max_hp(ctx, owner, -1);
                change_hujia(ctx, owner, 2);
                ctx.start_draw(owner, 1);
            }
        }
        "gw_niuquzhijing" | "gw_youer" | "gw_tongdi" | "gwmaoxian_enxier"
        | "gwmaoxian_fulisi" | "gwmaoxian_kaerweite" | "gwmaoxian_gaier" => {
            set_flag(ctx, owner, &format!("u05_card_used:{key}"), 1);
            if let Some(t) = first_target_or_other(ctx, u) {
                let _ = steal_one_to_hand(ctx, owner, t);
            }
            ctx.start_draw(owner, 1);
        }
        "gw_nuhaifengbao" | "gw_baishuang" | "gw_baobaoshu" | "gwmaoxian_dagong" => {
            set_flag(ctx, owner, &format!("u05_card_used:{key}"), 1);
            if let Some(t) = first_target_or_other(ctx, u) {
                let _ = discard_up_to(ctx, t, 1);
            }
        }
        "gw_guaiwuchaoxue" | "gwmaoxian_hengsaite" | "gwmaoxian_fuertaisite"
        | "gwmaoxian_bulanwang" | "gw_lang" | "gw_dudayuanshuai1" | "gw_dudayuanshuai2" => {
            set_flag(ctx, owner, &format!("u05_card_used:{key}"), 1);
            change_hujia(ctx, owner, 1);
            ctx.start_draw(owner, 1);
        }
        // U05 Hearthstone spells / artifacts / dreams / curses / totems (`character/hearth/card.js`)
        "shandianjian" | "ht_shandianjian" | "hsqingyu_shandian" => {
            set_flag(ctx, owner, &format!("u05_card_used:{key}"), 1);
            if let Some(t) = first_target_or_other(ctx, u) {
                deal_damage(ctx, owner, t, 1, DamageNature::Thunder);
            }
        }
        "hsqizhou_huo" | "hsdusu_huoyanhua" | "hsjixie_zhadan" => {
            set_flag(ctx, owner, &format!("u05_card_used:{key}"), 1);
            if let Some(t) = first_target_or_other(ctx, u) {
                deal_damage(ctx, owner, t, 1, DamageNature::Fire);
            }
        }
        "hsmengjing_feicuiyoulong" | "hsshenqi_kongbusangzhong" => {
            set_flag(ctx, owner, &format!("u05_card_used:{key}"), 1);
            if let Some(t) = first_target_or_other(ctx, u) {
                deal_damage(ctx, owner, t, 1, DamageNature::Normal);
            }
        }
        "hsmengjing_suxing" => {
            set_flag(ctx, owner, "u05_card_used:hsmengjing_suxing", 1);
            let others: Vec<PlayerId> = (0..ctx.core.players.len() as PlayerId)
                .filter(|&p| p != owner && ctx.core.player(p).alive)
                .collect();
            for t in others {
                ctx.emit(EventData::LoseHp {
                    player: t,
                    amount: 1,
                });
            }
        }
        "hsqizhou_shui" | "hsmengjing_huanxiaojiemei" | "tuteng3" => {
            set_flag(ctx, owner, &format!("u05_card_used:{key}"), 1);
            if !recover_if_damaged(ctx, owner, 1) {
                ctx.start_draw(owner, 1);
            }
        }
        "hsqizhou_tu" | "hsqingyu_hufu" | "tuteng2" | "tuteng6" => {
            set_flag(ctx, owner, &format!("u05_card_used:{key}"), 1);
            change_hujia(ctx, owner, 1);
        }
        "hsqingyu_zhanfang" | "hsshenqi_nengliangzhiguang" => {
            set_flag(ctx, owner, &format!("u05_card_used:{key}"), 1);
            change_max_hp(ctx, owner, 1);
            ctx.start_draw(owner, 1);
        }
        "hsqingyu_feibiao" | "hsdusu_xueji" | "hsmengjing_mengjing"
        | "hszuzhou_nvwudeganguo" | "hszuzhou_wushushike" => {
            set_flag(ctx, owner, &format!("u05_card_used:{key}"), 1);
            if let Some(t) = first_target_or_other(ctx, u) {
                let _ = discard_up_to(ctx, t, 1);
            }
        }
        "hszuzhou_guhuo" => {
            set_flag(ctx, owner, "u05_card_used:hszuzhou_guhuo", 1);
            if let Some(t) = first_target_or_other(ctx, u) {
                let _ = steal_one_to_hand(ctx, owner, t);
            }
        }
        "shenenshu" | "ht_shenenshu" | "hsfashu_anyingjingxiang" | "hsfashu_buwendingyibian"
        | "hsqizhou_feng" | "hsdusu_kuyecao" | "hsdusu_huangxuecao" | "hsdusu_shinancao"
        | "hsbaowu_cangbaotu" | "hsbaowu_huangjinyuanhou" | "hsshenqi_morijingxiang"
        | "hsmengjing_mengye" | "hszuzhou_nvwudepingguo" | "hszuzhou_nvwudexuetu"
        | "tuteng1" | "tuteng4" | "tuteng5" | "tuteng7" => {
            set_flag(ctx, owner, &format!("u05_card_used:{key}"), 1);
            ctx.start_draw(owner, 2);
        }
        "tuteng8" => {
            set_flag(ctx, owner, "u05_mod:tuteng8", 1);
            ctx.start_draw(owner, 1);
        }
        _ => return None,
    }

    finish_table_card(ctx, card);
    Some(None)
}
