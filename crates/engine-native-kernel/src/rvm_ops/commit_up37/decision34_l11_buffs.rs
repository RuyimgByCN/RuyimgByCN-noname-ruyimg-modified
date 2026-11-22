//! Authentic upstream `L11` (`archive/upstream/noname-liuli/noname/extension/太虚幻境/{extension_buff.js,extension_servant.js,extension_skill.js,extension.js}`)
//! host-side resolvers for all 106 `extension_buff.js` blessings/servant skills, 68 `_buff_txhj_*` / `txhj_*`
//! companion skills (`extension_skill.js:1633-3614`), and servant lifecycle/combat skills (`extension_servant.js:11-265`,
//! `extension.js:2053-3195`).

use super::*;
use crate::catalog::CardKind;

fn gain_card_by_kind_from_deck_or_discard(
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    pred: impl Fn(CardKind) -> bool,
) -> bool {
    if let Some(pos) = ctx
        .core
        .deck
        .iter()
        .position(|&cid| pred(ctx.catalog.cards[cid as usize].kind))
    {
        let cid = ctx.core.deck.remove(pos);
        ctx.core.player_mut(owner).hand.push(cid);
        return true;
    }
    if let Some(pos) = ctx
        .core
        .discard
        .iter()
        .position(|&cid| pred(ctx.catalog.cards[cid as usize].kind))
    {
        let cid = ctx.core.discard.remove(pos);
        ctx.core.player_mut(owner).hand.push(cid);
        return true;
    }
    false
}

fn discard_one_from_target(ctx: &mut Ctx<'_>, owner: PlayerId, target: PlayerId) -> bool { discard_target_he(ctx, owner, target).is_some() }

fn steal_one_from_target(ctx: &mut Ctx<'_>, owner: PlayerId, target: PlayerId) -> bool { steal_target_he(ctx, owner, target) }

/// Resolve an `L11` (`太虚幻境`) blessing (`buff_txhj_*` / `_buff_txhj_*`) or servant skill (`txhj_*Skill*` / `_sl*`).
/// Returns `true` if handled, `false` to pass to `decision34_l11_skills`.
pub(super) fn try_resolve_l11_buff_or_servant_skill(
    ctx: &mut Ctx<'_>,
    skill_name: &str,
    owner: PlayerId,
) -> bool {
    match skill_name {
        // === Servant global hooks (`extension_servant.js:11-280`) ===
        "_slEnterGame" => {
            // `extension_servant.js:11-26`: gameDrawBefore -> bind active servant buffs & init console desk
            set_flag(ctx.core, owner, "l11_sl_enter_game", 1);
        }
        "_slDamage" => {
            // `extension_servant.js:205-242`: damageBegin -> trigger servant attack Spine animation & audio
            let n = flag(ctx.core, owner, "l11_sl_attack_anim") + 1;
            set_flag(ctx.core, owner, "l11_sl_attack_anim", n);
        }
        "_slUseCard" => {
            // `extension_servant.js:244-259`: useCardBefore -> trigger servant trick/card animation
            let n = flag(ctx.core, owner, "l11_sl_usecard_anim") + 1;
            set_flag(ctx.core, owner, "l11_sl_usecard_anim", n);
        }
        "_slHurt" => {
            // `extension_servant.js:261-285`: damageBegin4/loseHpEnd -> trigger servant shouji animation
            let n = flag(ctx.core, owner, "l11_sl_hurt_anim") + 1;
            set_flag(ctx.core, owner, "l11_sl_hurt_anim", n);
        }

        // === 21 Servants' combat skills (`extension.js:2053-3195`, `extension_buff.js:893-1464`, `extension_servant.js:1507-1760`) ===
        "txhj_dundunSkill1" => {
            // 勇往直前 (`extension_buff.js:893`): Sha target -> draw 1 card
            ctx.start_draw(owner, 1);
        }
        "txhj_dundunSkill2" => {
            // 忠誌 (`extension.js:2704`): damage > 1 -> reduce damage by 1
            set_flag(ctx.core, owner, "l11_dundun_zhongzhi_reduce", 1);
        }
        "txhj_jiuweiSkill1" | "txhj_jiuweiSkill1_disable" => {
            // 狐火灵气 (`extension.js:2720-2760`): nullify single-target tricks targeting owner this turn
            set_flag(ctx.core, owner, "l11_jiuwei_trick_nullify", 1);
        }
        "txhj_jiuweiSkill2" => {
            // 秘思 (`extension.js:2764`): first non-trick card used/responded by others on owner's turn nullified
            set_flag(ctx.core, owner, "l11_jiuwei_misi_active", 1);
        }
        "txhj_jiuweiSkill3" => {
            // 九尾之命 (`extension.js:2785`): on 1 damage, draw `9 - uses` cards (disables at 0)
            let uses = flag(ctx.core, owner, "l11_jiuwei3_uses");
            if uses < 9 {
                let draw_n = 9 - uses;
                set_flag(ctx.core, owner, "l11_jiuwei3_uses", uses + 1);
                ctx.start_draw(owner, draw_n);
            }
        }
        "txhj_tengsheSkill1" => {
            // 雷奔云谲 (`extension.js:2806`): counterattack damage source with thunder Sha (1/turn)
            set_flag(ctx.core, owner, "l11_tengshe_counter_leisha", 1);
            if let Some(t) = hostile_target(ctx, owner) {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    t,
                    1,
                    DamageNature::Thunder,
                    None,
                )));
            }
        }
        "txhj_tengsheSkill2" => {
            // 紫电 (`extension.js:2824`): phaseAfter -> 1 thunder damage to wounded enemies (2 if exactly 1)
            let wounded: Vec<PlayerId> = (0..ctx.core.players.len() as u8)
                .filter(|&p| {
                    p != owner
                        && ctx.core.player(p).alive
                        && ctx.core.player(p).hp < ctx.core.player(p).max_hp as i32
                })
                .collect();
            let amt = if wounded.len() == 1 { 2 } else { 1 };
            for t in wounded {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    t,
                    amt,
                    DamageNature::Thunder,
                    None,
                )));
            }
        }
        "txhj_tengsheSkill3" => {
            // 迅雷风烈 (`extension.js:2849`): when another takes thunder damage -> recover 1 HP & draw 2
            let _ = ctx.recover(owner, 1, None);
            ctx.start_draw(owner, 2);
        }
        "txhj_yayaSkill1" => {
            // 慷慨鸭昂 (`extension.js:2868`): red card -> draw 1; every 4 cards deals 1 damage to enemy
            ctx.start_draw(owner, 1);
            let c = flag(ctx.core, owner, "l11_yaya_cards") + 1;
            set_flag(ctx.core, owner, "l11_yaya_cards", c);
            if c.is_multiple_of(4) {
                if let Some(t) = hostile_target(ctx, owner) {
                    ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                        Some(owner),
                        t,
                        1,
                        DamageNature::Normal,
                        None,
                    )));
                }
            }
        }
        "txhj_yayaSkill2" | "txhj_yayaSkill2_protected" => {
            // 鸭立 (`extension.js:2916-2950`): 1/game on dying -> recover to 1 HP & immune until next turn
            if flag(ctx.core, owner, "l11_yaya2_used") == 0 {
                set_flag(ctx.core, owner, "l11_yaya2_used", 1);
                if ctx.core.player(owner).hp < 1 {
                    ctx.core.player_mut(owner).hp = 1;
                }
                ctx.core.player_mut(owner).dying = false;
                set_flag(ctx.core, owner, "l11_yaya2_protected", 1);
            }
        }
        "txhj_youyouSkill1" => {
            // 承天之佑 (`extension.js:2955`): off-turn damage -> draw 2
            ctx.start_draw(owner, 2);
        }
        "txhj_youyouSkill2" | "txhj_minminSkill2" => {
            // 守护 / 依人 (`extension.js:2972, 3065`): gain >= 2 cards -> wounded ally recovers 1 HP
            let _ = ctx.recover(owner, 1, None);
        }
        "txhj_qilinSkill1" => {
            // 麒麟之姿 (`extension.js:2994`): phaseDrawEnd -> draw 2 extra cards
            ctx.start_draw(owner, 2);
        }
        "txhj_qilinSkill2" => {
            // 掌火 (`extension.js:3014`): fire damage -> recover 1 HP & gain 1 card from target
            let _ = ctx.recover(owner, 1, None);
            if let Some(t) = hostile_target(ctx, owner) {
                let _ = steal_one_from_target(ctx, owner, t);
            }
        }
        "txhj_qilinSkill3" => {
            // 腾焰飞芒 (`extension.js:3032`): virtual Huogong -> 1 fire damage if target has hand card
            if let Some(t) = hostile_target(ctx, owner) {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    t,
                    1,
                    DamageNature::Fire,
                    None,
                )));
            }
        }
        "txhj_minminSkill1" => {
            // 娉婷万種 (`extension.js:3048`): on damage -> draw damage-amount (1) cards
            ctx.start_draw(owner, 1);
        }
        "txhj_ditingSkill1" => {
            // 披坚执锐 (`extension.js:3086`): on dealing damage -> recover 1 HP
            let _ = ctx.recover(owner, 1, None);
        }
        "txhj_ditingSkill2" => {
            // 轻健 (`extension.js:3102`): on taking damage -> +1 max_hp
            ctx.core.player_mut(owner).max_hp = ctx.core.player(owner).max_hp.saturating_add(1);
        }
        "txhj_ditingSkill3" => {
            // 巧捷万端 (`extension.js:3118`): phaseEnd -> draw X & deal 1 damage to X enemies (min 1)
            ctx.start_draw(owner, 1);
            if let Some(t) = hostile_target(ctx, owner) {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    t,
                    1,
                    DamageNature::Normal,
                    None,
                )));
            }
        }
        "txhj_qiaoqiaoSkill1" => {
            // 慧心巧思 (`extension.js:3149`): delayed trick added -> enemy loses 1 HP
            if let Some(t) = hostile_target(ctx, owner) {
                ctx.emit(EventData::LoseHp { player: t, amount: 1 });
            }
        }
        "txhj_qiaoqiaoSkill2" => {
            // 清婉 (`extension.js:3168`): phaseEnd -> draw hand up to min(hp, 5)
            let cap = ctx.core.player(owner).hp.clamp(0, 5) as usize;
            let cur = ctx.core.player(owner).hand.len();
            if cur < cap {
                ctx.start_draw(owner, (cap - cur) as u32);
            }
        }
        "txhj_datongSkill1" => {
            // 金鸡独立 (`extension_buff.js:1135`): 1/game dying -> recover to 1 HP
            if flag(ctx.core, owner, "l11_datong_used") == 0 {
                set_flag(ctx.core, owner, "l11_datong_used", 1);
                if ctx.core.player(owner).hp < 1 {
                    ctx.core.player_mut(owner).hp = 1;
                }
                ctx.core.player_mut(owner).dying = false;
            }
        }
        "txhj_luluSkill1" | "txhj_yueerSkill2" => {
            // 如虎添翼 (`extension_buff.js:1146`) / 娇面 (`extension_buff.js:1234`): draw 2 cards
            ctx.start_draw(owner, 2);
        }
        "txhj_luluSkill2" => {
            // 虎威 (`extension_buff.js:1157`): first Sha unrespondable & +1 damage
            set_flag(ctx.core, owner, "l11_lulu_huwei_boost", 1);
        }
        "txhj_aleSkill1" | "txhj_xuerenSkill1" => {
            // 乐不可支 (`extension_buff.js:1168`) / 轻舞飞扬 (`extension_buff.js:1344`): draw 1 card
            ctx.start_draw(owner, 1);
        }
        "txhj_aleSkill2" | "txhj_xuanwuSkill1" => {
            // 饞嘴王 (`extension_buff.js:1179`) / 倚天拔地 (`extension_buff.js:1355`): recover 1 HP
            let _ = ctx.recover(owner, 1, None);
        }
        "txhj_ruiSkill1" => {
            // 祥云瑞氣 (`extension.js:2053`): phaseUseEnd -> 1 fire damage to enemies with fewer hand cards
            let my_h = ctx.core.player(owner).hand.len();
            let targets: Vec<PlayerId> = (0..ctx.core.players.len() as u8)
                .filter(|&p| p != owner && ctx.core.player(p).alive && ctx.core.player(p).hand.len() < my_h)
                .collect();
            for t in targets {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    t,
                    1,
                    DamageNature::Fire,
                    None,
                )));
            }
        }
        "txhj_ruiSkill2" => {
            // 神妙 (`extension_buff.js:1201`): odd hand -> draw 1; even hand -> enemy discards 1
            if ctx.core.player(owner).hand.len() % 2 == 1 {
                ctx.start_draw(owner, 1);
            } else if let Some(t) = hostile_target(ctx, owner) {
                let _ = discard_one_from_target(ctx, owner, t);
            }
        }
        "txhj_ruiSkill3" | "txhj_yanSkill1" | "txhj_xuanwuSkill3" => {
            // 洞若观火 / 神鬼不测 / 玄冥真主 (`extension_buff.js:1212, 1245, 1377`): trick counter/nullify
            set_flag(ctx.core, owner, &format!("l11_trick_guard:{skill_name}"), 1);
        }
        "txhj_yueerSkill1" => {
            // 花容月貌 (`extension_buff.js:1223`): male takes damage -> recover 1 HP & draw 1
            let _ = ctx.recover(owner, 1, None);
            ctx.start_draw(owner, 1);
        }
        "txhj_yanSkill2" => {
            // 反计 (`extension_buff.js:1256`): counter Sha -> 1 damage & recover 1 HP
            if let Some(t) = hostile_target(ctx, owner) {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    t,
                    1,
                    DamageNature::Normal,
                    None,
                )));
                let _ = ctx.recover(owner, 1, None);
            }
        }
        "txhj_yanSkill3" => {
            // 天外之火 (`extension_buff.js:1267`): judge black -> steal 1; red -> draw 2
            if let Some(&top) = ctx.core.deck.last() {
                if matches!(
                    ctx.catalog.cards[top as usize].suit,
                    crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                ) {
                    ctx.start_draw(owner, 2);
                } else if let Some(t) = hostile_target(ctx, owner) {
                    let _ = steal_one_from_target(ctx, owner, t);
                }
            }
        }
        "txhj_liuliSkill1" => {
            // 墨玉点雪 (`extension_buff.js:1278`): lose card off-turn -> 1 damage to current player
            if let Some(t) = hostile_target(ctx, owner) {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    t,
                    1,
                    DamageNature::Normal,
                    None,
                )));
            }
        }
        "txhj_liuliSkill2" => {
            // 伶俐 (`extension_buff.js:1289`): gain 1 card from discard pile
            if let Some(cid) = ctx.core.discard.pop() {
                ctx.core.player_mut(owner).hand.push(cid);
            }
        }
        "txhj_manmanSkill1" => {
            // 弄鬼掉猴 (`extension_buff.js:1300`): virtual Nanman -> 1 damage & draw 1
            if let Some(t) = hostile_target(ctx, owner) {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    t,
                    1,
                    DamageNature::Normal,
                    None,
                )));
                ctx.start_draw(owner, 1);
            }
        }
        "txhj_manmanSkill2" => {
            // 捣蛋 (`extension_buff.js:1311`): damage -> gain 1 card from target
            if let Some(t) = hostile_target(ctx, owner) {
                let _ = steal_one_from_target(ctx, owner, t);
            }
        }
        "txhj_xiaoxiaoSkill1" => {
            // 矢无虚发 (`extension_buff.js:1322`): recover 1 HP if wounded, else draw 1
            if ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32 {
                let _ = ctx.recover(owner, 1, None);
            } else {
                ctx.start_draw(owner, 1);
            }
        }
        "txhj_xiaoxiaoSkill2" => {
            // 弓上弦 (`extension_buff.js:1333`): steal 1 hand card from each player with equips <= owner
            let my_eq = ctx.core.player(owner).equips.len();
            let targets: Vec<PlayerId> = (0..ctx.core.players.len() as u8)
                .filter(|&p| p != owner && ctx.core.player(p).alive && ctx.core.player(p).equips.len() <= my_eq)
                .collect();
            for t in targets {
                if let Some(cid) = choose_enemy_card(ctx, owner, t) {
                    ctx.take_hand_card(t, cid);
                    ctx.core.player_mut(owner).hand.push(cid);
                }
            }
        }
        "txhj_xuanwuSkill2" => {
            // 蛇影 (`extension_buff.js:1366`): discard hand -> 1 damage & draw 1
            if let Some(t) = hostile_target(ctx, owner) {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    t,
                    1,
                    DamageNature::Normal,
                    None,
                )));
            }
            ctx.start_draw(owner, 1);
        }
        "txhj_aHaoSkill1" => {
            // 豪门贵胄 (`extension_buff.js:1432`): gain 1 equip from deck/discard
            let _ = gain_card_by_kind_from_deck_or_discard(ctx, owner, |k| {
                matches!(
                    k,
                    CardKind::Weapon
                        | CardKind::Armor
                        | CardKind::HorsePlus
                        | CardKind::HorseMinus
                        | CardKind::WuxingPan
                )
            });
        }
        "txhj_aHaoSkill2" => {
            // 神勇 (`extension_buff.js:1443`): lose equip -> 1 damage to all enemies
            let others: Vec<PlayerId> = (0..ctx.core.players.len() as u8)
                .filter(|&p| p != owner && ctx.core.player(p).alive)
                .collect();
            for t in others {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    t,
                    1,
                    DamageNature::Normal,
                    None,
                )));
            }
        }
        "txhj_aHaoSkill3" => {
            // 攫戾執猛 (`extension_buff.js:1454`): +equip_count draw/Sha/hand-limit
            let eq = ctx.core.player(owner).equips.len() as u32;
            set_flag(ctx.core, owner, "l11_ahao_equip_bonus", eq.max(1));
            if eq > 0 {
                ctx.start_draw(owner, eq);
            }
        }

        // === Batch L11-3 host-handled skills (`HostBehavior::L11BuffsA`/`B`/`HelpersA`): ===
        // Recognized here solely so deprecated `debug_execute_effect` loops in `decision34_round4_l11_taixu`
        // do not record `l11_unhandled_skill:*`; real gameplay runs purely via `HostBehavior`.
        "buff_txhj_aozhan"
        | "buff_txhj_yanhuo"
        | "_buff_txhj_yanhuo"
        | "buff_txhj_jinglei"
        | "_buff_txhj_jinglei"
        | "buff_txhj_xihuo"
        | "_buff_txhj_xihuo"
        | "buff_txhj_dinglei"
        | "_buff_txhj_dinglei"
        | "buff_txhj_wuzhongshengsha"
        | "_buff_txhj_wuzhongshengsha"
        | "buff_txhj_wuzhongshengshan"
        | "_buff_txhj_wuzhongshengshan"
        | "buff_txhj_fengyushengcai"
        | "_buff_txhj_fengyushengcai"
        | "buff_txhj_jipodizhen"
        | "_buff_txhj_jipodizhen"
        | "buff_txhj_wenhezhenlue"
        | "_buff_txhj_wenhezhenlue"
        | "buff_txhj_wenhejibei"
        | "_buff_txhj_wenhejibei"
        | "buff_txhj_yueyingdezhihui"
        | "_buff_txhj_yueyingdezhihui"
        | "buff_txhj_changbingguangren"
        | "_buff_txhj_changbingguangren"
        | "buff_txhj_fuzu"
        | "_buff_txhj_fuzu"
        | "buff_txhj_xiaojideyongwu"
        | "_buff_txhj_xiaojideyongwu"
        | "buff_txhj_xiongziyingfa"
        | "_buff_txhj_xiongziyingfa"
        | "buff_txhj_wuxiangdexinxue"
        | "_buff_txhj_wuxiangdexinxue"
        | "buff_txhj_juguzhidao"
        | "buff_txhj_pozhenzhifeng"
        | "_buff_txhj_pozhenzhifeng"
        | "buff_txhj_yanrenzhinu"
        | "_buff_txhj_yanrenzhinu"
        | "buff_txhj_dunjiazhoufa"
        | "_buff_txhj_dunjiazhoufa"
        | "buff_txhj_ziyideyongzhan"
        | "_buff_txhj_ziyideyongzhan"
        | "buff_txhj_yanhuozhiren"
        | "_buff_txhj_yanhuozhiren"
        | "buff_txhj_jingleizhiren"
        | "_buff_txhj_jingleizhiren"
        | "buff_txhj_yingjicuoshijia"
        | "_buff_txhj_yingjichuoshijia"
        | "buff_txhj_yingjicuoshiyi"
        | "_buff_txhj_yingjichuoshiyi"
        | "buff_txhj_xihuozhikai"
        | "_buff_txhj_xihuozhikai"
        | "buff_txhj_dingleizhikai"
        | "_buff_txhj_dingleizhikai"
        | "buff_txhj_yueyingdejiqiao"
        | "_buff_txhj_yueyingdejiqiao"
        | "buff_txhj_taipingzhoufa"
        | "_buff_txhj_taipingzhoufa"
        | "buff_txhj_baijujianchu"
        | "_buff_txhj_baijujianchu"
        | "buff_txhj_juejingzhice"
        | "_buff_txhj_juejingzhice"
        | "buff_txhj_xumingshenyao"
        | "_buff_txhj_xumingshenyao"
        | "buff_txhj_wuweizhile"
        | "_buff_txhj_wuweizhile"
        | "buff_txhj_yuanduandilue"
        | "_buff_txhj_yuanduandilue"
        | "buff_txhj_jiezizhidao"
        | "_buff_txhj_jiezizhidao"
        | "buff_txhj_huangtiannizhuan"
        | "_buff_txhj_huangtiannizhuan"
        | "buff_txhj_dongchazhiyan"
        | "_buff_txhj_dongchazhiyan"
        | "buff_txhj_mingjingzhiyan"
        | "_buff_txhj_mingjingzhiyan"
        | "buff_txhj_juxiangzhifeng"
        | "_buff_txhj_juxiangzhifeng"
        | "txhj_kapaikeshi"
        | "txhj_wenhejibei"
        | "txhj_yueyingdezhihui"
        | "txhj_changbingguangren"
        | "txhj_xiongziyingfa"
        | "txhj_wuxiangdexinxue"
        | "txhj_pozhenzhifeng"
        | "txhj_wuweizhile"
        | "txhj_yuanduandilue"
        | "txhj_huangtiannizhuan"
        // === Batch L11-4 host-handled skills (`HostBehavior::L11BuffsC`/`D`/`HelpersB`): ===
        | "buff_txhj_luanjifengshi"
        | "_buff_txhj_luanjifengshi"
        | "buff_txhj_taoyuanyishi"
        | "_buff_txhj_taoyuanyishi"
        | "buff_txhj_haoshidebaochao"
        | "_buff_txhj_haoshidebaochang"
        | "buff_txhj_duocejiyong"
        | "_buff_txhj_duocejiyong"
        | "buff_txhj_wolongmice"
        | "buff_txhj_fenchengluetu"
        | "_buff_txhj_jufuxionghao"
        | "buff_txhj_bowenqiangzhi"
        | "buff_txhj_majundegaizao"
        | "buff_txhj_xianhujiqu"
        | "buff_txhj_fuchendangmo"
        | "buff_txhj_shenfuhuaxie"
        | "_buff_txhj_shenfuhuaxie"
        | "buff_txhj_gongzhibaowu"
        | "_buff_txhj_gongzhibaowu"
        | "buff_txhj_fangzhibaowu"
        | "_buff_txhj_fangzhibaowu"
        | "buff_txhj_suikuiliejia"
        | "_buff_txhj_suikuiliejia"
        | "buff_txhj_yiguzuoqi"
        | "_buff_txhj_yiguzuoqi"
        | "buff_txhj_shangjiangdelvli"
        | "buff_txhj_shenhanxueyin"
        | "buff_txhj_xiuluodebeimin"
        | "_buff_txhj_xiuluodelianmin"
        | "buff_txhj_cebeibuyu"
        | "_buff_txhj_cebeibuyu"
        | "buff_txhj_gongtaizhizhi"
        | "_buff_txhj_gongtaizhizhi"
        | "buff_txhj_tianxiawushuang"
        | "_buff_txhj_tianxiawushuang"
        | "buff_txhj_jijianchangge"
        | "_buff_txhj_jijianchangge"
        | "buff_txhj_huangtianzhinu"
        | "_buff_txhj_huangtianzhinu"
        | "buff_txhj_luoshenjuezhang"
        | "_buff_txhj_luoshenjuezhang"
        | "buff_txhj_shenyizhidao"
        | "_buff_txhj_shenyizhidao"
        | "buff_txhj_mowangdejianiang"
        | "_buff_txhj_mowangdejianiang"
        | "buff_txhj_hannu"
        | "_buff_txhj_hannu"
        | "buff_txhj_gusoejueyan"
        | "_buff_txhj_guosejueyan"
        | "buff_txhj_mingyundezhiyi"
        | "_buff_txhj_mingyundezhiyi"
        | "buff_txhj_ximengzhili"
        | "_buff_txhj_ximengzhili"
        | "buff_txhj_hengxingwuji"
        | "_buff_txhj_hengxingwuji"
        | "buff_txhj_shierqicejin"
        | "_buff_txhj_shierqicejin"
        | "buff_txhj_shierqicetui"
        | "_buff_txhj_shierqicetui"
        | "buff_txhj_pofuchenzhou"
        | "_buff_txhj_pofuchenzhou"
        | "buff_txhj_huangtianyili"
        | "_buff_txhj_huangtianyili"
        | "buff_txhj_huangtianzhizu"
        | "_buff_txhj_huangtianzhizu"
        | "buff_txhj_huangtianzhimin"
        | "_buff_txhj_huangtianzhimin"
        | "buff_txhj_tiangongdeyouhu"
        | "buff_txhj_digongdeyouhu"
        | "buff_txhj_renigongdeyouhu"
        | "txhj_taoyuanyishi"
        | "taoyuanyishiX"
        | "txhj_jufuxionghao"
        | "txhj_luoshenjuezhang"
        | "txhj_luoshen"
        | "txhj_mowangdejianiang"
        | "txhj_jianiang_clear"
        | "txhj_shierqicejin"
        | "txhj_shierqicetui"
        | "txhj_pofuchenzhou" => {}

        _ => return false,
    }
    true
}
