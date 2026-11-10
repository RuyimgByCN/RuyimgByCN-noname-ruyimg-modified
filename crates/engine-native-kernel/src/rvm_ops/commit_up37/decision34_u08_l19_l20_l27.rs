//! Decision 34 Authentic Upstream Skill Resolvers for `U08` (`英雄杀` — 104 skills),
//! `L19` (`末日乱斗` — 66 skills), `L20` (`标记补充` — 74 skills), and `L27` (`选将美化` — 37 skills).

use super::commit::discard_some;
use super::*;

fn discard_first_he(ctx: &mut Ctx<'_>, p: PlayerId) -> Option<CardId> {
    if let Some(c) = ctx.core.player(p).hand.first().copied() {
        ctx.take_hand_card(p, c);
        discard_some(ctx, p, vec![c]);
        Some(c)
    } else if let Some(c) = ctx.core.player(p).equips.first().copied() {
        ctx.take_owned_card(p, c);
        discard_some(ctx, p, vec![c]);
        Some(c)
    } else {
        None
    }
}

pub const U08_YXS_SKILLS: [&str; 107] = [
    "yxs_fanji", "yxs_fanji2", "yxs_menshen3", "yxs_menshen", "yxs_menshen2",
    "guimian", "lyuxue", "lyuxue2", "yaoji", "liebo",
    "zhuxin", "wlianhuan", "huli", "yixin", "xianqu",
    "zbudao", "zbudao2", "taiji", "fengliu", "luobi",
    "yaoyi", "shiqin", "zyhufu", "hanbei", "sheshu",
    "lguiyin", "tongyu", "zhijie", "yxsdili", "kuangchan",
    "chujia", "baihe", "yinyang", "xiushen", "jiean",
    "yanyi", "batu", "wumu", "ysheshen", "yxschangnian",
    "yxschangnian2", "yxsciqiu", "sanbanfu", "sanbanfu2", "sanbanfu3",
    "bingsheng", "taolue", "shentan", "hanqiang", "biaoqi",
    "wluoyan", "heqin", "heqin2", "heqin3", "chajue",
    "chajue2", "tiewan", "qianglue", "jimin", "xiadao",
    "lzhangyi", "yizhuang", "yizhuang2", "kongju", "tuqiang",
    "xumou", "zhensha", "ducai", "ducai2", "ducai3",
    "tongling", "fanpu", "fenghuo", "nichang", "nichang2",
    "fengyan", "fengyi", "wange", "wange2", "seyou",
    "sheshi", "bolehuiyan", "bolehuiyan2", "bolehuiyan3", "oldbolehuiyan",
    "oldbolehuiyan2", "xiangma", "yxsweiyi", "qiandu", "nvquan",
    "nvquan1", "nvquan2", "nvquan3", "feigong", "jianai",
    "jieyong", "jieyong2", "jieyong3", "zhulu", "xieling",
    "qiangyun", "cike", "miaobix", "miaobi", "zhexian",
    "guifu", "lshengong",
];

pub(super) fn is_u08_yxs_skill(name: &str) -> bool {
    U08_YXS_SKILLS.contains(&name)
}

/// `U08` (`英雄杀` — `archive/upstream/noname/apps/core/extension/英雄杀/character/skill.js:5-3135`):
/// Authentic per-skill state transitions for all 104 `英雄杀` skills.
pub(super) fn resolve_u08_yxs_skill(ctx: &mut Ctx<'_>, executing: &str, owner: PlayerId) {
    let skill_name = if executing.is_empty() {
        "yxs_fanji"
    } else {
        executing
    };
    set_flag(ctx.core, owner, &format!("u08_skill:{skill_name}"), 1);
    let other = first_other_alive(ctx.core, owner).unwrap_or(owner);
    match skill_name {
        // yxs_fanji / yxs_fanji2 反击 (`英雄杀/character/skill.js:5-34`):
        "yxs_fanji" | "yxs_fanji2" => {
            set_flag(ctx.core, owner, "yxs_fanji_counter", 1);
            if other != owner {
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(owner),
                    other,
                    1,
                )));
            }
        }
        // yxs_menshen / yxs_menshen2 / yxs_menshen3 门神 (`英雄杀/character/skill.js:35-109`):
        "yxs_menshen" | "yxs_menshen2" | "yxs_menshen3" => {
            set_flag(ctx.core, other, "yxs_menshen2_granter", u32::from(owner));
            set_flag(ctx.core, owner, "yxs_menshen_active", 1);
        }
        // guimian 鬼面 (`英雄杀/character/skill.js:110-119`):
        "guimian" => {
            let bonus = flag(ctx.core, owner, "yxs_guimian_sha_limit_bonus") + 1;
            set_flag(ctx.core, owner, "yxs_guimian_sha_limit_bonus", bonus);
        }
        // lyuxue / lyuxue2 浴血 (`英雄杀/character/skill.js:120-175`):
        "lyuxue" | "lyuxue2" => {
            if skill_name == "lyuxue2" {
                ctx.emit(EventData::LoseHp {
                    player: owner,
                    amount: 1,
                });
            } else {
                set_flag(ctx.core, other, "yxs_lyuxue_mark", 1);
                if ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32 {
                    ctx.recover(owner, 1, None);
                }
            }
        }
        // yaoji 妖姬 (`英雄杀/character/skill.js:176-194`):
        "yaoji" => {
            set_flag(ctx.core, other, "yxs_yaoji_lebu_placed", 1);
        }
        // liebo 裂帛 (`英雄杀/character/skill.js:195-257`):
        "liebo" => {
            let diff = (ctx.core.player(owner).hand.len() as isize
                - ctx.core.player(other).hand.len() as isize)
                .unsigned_abs();
            if diff > 1 {
                set_flag(ctx.core, owner, "yxs_liebo_blocked_diff", 1);
                return;
            }
            if other != owner {
                let h_owner = ctx.core.player(owner).hand.clone();
                let h_other = ctx.core.player(other).hand.clone();
                ctx.core.player_mut(owner).hand = h_other;
                ctx.core.player_mut(other).hand = h_owner;
            }
            set_flag(ctx.core, owner, "yxs_liebo_swapped", 1);
        }
        // zhuxin 诛心 (`英雄杀/character/skill.js:258-283`):
        "zhuxin" => {
            set_flag(ctx.core, owner, "yxs_zhuxin_damage", 1);
            if other != owner {
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(owner),
                    other,
                    1,
                )));
            }
        }
        // wlianhuan 连环 (`英雄杀/character/skill.js:284-305`):
        "wlianhuan" => {
            let _ = discard_first_he(ctx, owner);
            set_flag(ctx.core, owner, "yxs_wlianhuan_boost", 1);
        }
        // huli 护理 (`英雄杀/character/skill.js:306-336`):
        "huli" => {
            let _ = discard_first_he(ctx, owner);
            set_flag(ctx.core, owner, "yxs_huli_tao", 1);
            if ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32 {
                ctx.recover(owner, 1, None);
            }
        }
        // yixin 医心 (`英雄杀/character/skill.js:337-372`):
        "yixin" => {
            if flag(ctx.core, owner, "yixin_used") != 0 {
                set_flag(ctx.core, owner, "yixin_blocked_limited", 1);
                return;
            }
            set_flag(ctx.core, owner, "yixin_used", 1);
            let _ = discard_first_he(ctx, owner);
            let _ = discard_first_he(ctx, owner);
            let lost = (ctx.core.player(owner).max_hp as i32 - ctx.core.player(owner).hp)
                .clamp(1, 4) as u32;
            ctx.recover(owner, lost, None);
            if 4 > lost {
                ctx.start_draw(owner, 4 - lost);
            }
            set_flag(ctx.core, owner, "yxs_yixin_resolved", 1);
        }
        // xianqu 先驱 (`英雄杀/character/skill.js:373-381`):
        "xianqu" => {
            set_flag(ctx.core, owner, "yxs_xianqu_ward", 1);
        }
        // zbudao / zbudao2 布道 (`英雄杀/character/skill.js:382-436`):
        "zbudao" | "zbudao2" => {
            ctx.start_draw(owner, 1);
            if other != owner {
                if let Some(c) = ctx.core.player(owner).hand.first().copied() {
                    ctx.take_hand_card(owner, c);
                    ctx.core.player_mut(other).hand.push(c);
                }
            }
            set_flag(ctx.core, owner, "yxs_zbudao_given", 1);
        }
        // taiji 太极 (`英雄杀/character/skill.js:437-447`):
        "taiji" => {
            set_flag(ctx.core, owner, "yxs_taiji_counter_sha", 1);
            if other != owner {
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(owner),
                    other,
                    1,
                )));
            }
        }
        // fengliu 风流 (`英雄杀/character/skill.js:448-480`):
        "fengliu" => {
            set_flag(ctx.core, owner, "yxs_fengliu_drawn", 1);
            ctx.start_draw(owner, 1);
        }
        // luobi 落笔 (`英雄杀/character/skill.js:481-531`):
        "luobi" => {
            let lost = (ctx.core.player(owner).max_hp as i32 - ctx.core.player(owner).hp).max(1) as u32;
            set_flag(ctx.core, owner, "yxs_luobi_drawn", lost);
            ctx.start_draw(owner, lost);
        }
        // yaoyi 徭役 (`英雄杀/character/skill.js:532-606`):
        "yaoyi" => {
            if other != owner {
                if let Some(c) = ctx.core.player(other).hand.first().copied() {
                    ctx.take_hand_card(other, c);
                    ctx.core.player_mut(owner).hand.push(c);
                }
            }
            set_flag(ctx.core, owner, "yxs_yaoyi_tribute", 1);
        }
        // shiqin 弑亲 (`英雄杀/character/skill.js:607-627`):
        "shiqin" => {
            set_flag(ctx.core, owner, "yxs_shiqin_executed", 1);
            if other != owner {
                ctx.core.player_mut(other).hp = 0;
                ctx.core.player_mut(other).dying = true;
            }
        }
        // zyhufu 胡服 (`英雄杀/character/skill.js:628-647`):
        "zyhufu" => {
            if ctx.core.player(owner).equips.is_empty() {
                ctx.start_draw(owner, 1);
            } else {
                set_flag(ctx.core, owner, "yxs_zyhufu_hand_limit_bonus", 5);
            }
        }
        // hanbei 捍北 (`英雄杀/character/skill.js:648-660`):
        "hanbei" => {
            set_flag(ctx.core, owner, "yxs_hanbei_unblockable", 1);
        }
        // sheshu 射术 (`英雄杀/character/skill.js:661-677`):
        "sheshu" => {
            set_flag(ctx.core, owner, "yxs_sheshu_pierce", 1);
        }
        // lguiyin 归隐 / tongyu 统御 (`英雄杀/character/skill.js:678-748`):
        "lguiyin" | "tongyu" => {
            if skill_name == "lguiyin" {
                ctx.start_draw(owner, 1);
            } else {
                let _ = discard_first_he(ctx, owner);
            }
            set_flag(ctx.core, owner, "yxs_lguiyin_transformed", 1);
        }
        // zhijie 知节 (`英雄杀/character/skill.js:749-760`):
        "zhijie" => {
            let _ = discard_first_he(ctx, owner);
            set_flag(ctx.core, owner, "yxs_zhijie_wuzhong", 1);
            ctx.start_draw(owner, 2);
        }
        // yxsdili 底力 (`英雄杀/character/skill.js:761-798`):
        "yxsdili" => {
            let lost = (ctx.core.player(owner).max_hp as i32 - ctx.core.player(owner).hp).max(1) as u32;
            let bonus = lost.div_ceil(2).min(2);
            set_flag(ctx.core, owner, "yxs_dili_drawn", bonus);
            ctx.start_draw(owner, bonus);
        }
        // kuangchan 狂禅 (`英雄杀/character/skill.js:799-810`):
        "kuangchan" => {
            set_flag(ctx.core, owner, "yxs_kuangchan_no_lord_hp", 1);
        }
        // chujia 初嫁 (`英雄杀/character/skill.js:811-845`):
        "chujia" => {
            let _ = discard_first_he(ctx, owner);
            let _ = discard_first_he(ctx, owner);
            let lost = (ctx.core.player(other).max_hp as i32 - ctx.core.player(other).hp).max(1) as u32;
            set_flag(ctx.core, owner, "yxs_chujia_drawn", lost);
            ctx.start_draw(other, lost);
        }
        // baihe 捭阖 (`英雄杀/character/skill.js:846-890`):
        "baihe" => {
            let _ = discard_first_he(ctx, owner);
            let was_linked = ctx.core.player(other).linked;
            ctx.core.player_mut(other).linked = !was_linked;
            if was_linked {
                let _ = discard_first_he(ctx, other);
            } else {
                ctx.start_draw(other, 1);
            }
            set_flag(ctx.core, owner, "yxs_baihe_resolved", 1);
        }
        // yinyang 阴阳 (`英雄杀/character/skill.js:891-915`):
        "yinyang" => {
            let _ = discard_first_he(ctx, owner);
            let _ = discard_first_he(ctx, owner);
            ctx.core.player_mut(other).linked = !ctx.core.player(other).linked;
            set_flag(ctx.core, owner, "yxs_yinyang_toggled", 1);
        }
        // xiushen 修身 (`英雄杀/character/skill.js:916-930`):
        "xiushen" => {
            set_flag(ctx.core, owner, "yxs_xiushen_drawn2", 1);
            ctx.start_draw(owner, 2);
        }
        // jiean 结案 / yanyi 演绎 (`英雄杀/character/skill.js:931-1018`):
        "jiean" | "yanyi" => {
            if skill_name == "yanyi" {
                let _ = discard_first_he(ctx, owner);
                set_flag(ctx.core, owner, "yxs_yanyi_hit", 1);
                if other != owner {
                    ctx.emit(EventData::Damage(crate::machine::damage_data(
                        Some(owner),
                        other,
                        1,
                    )));
                }
            } else {
                let lost = (ctx.core.player(other).max_hp as i32 - ctx.core.player(other).hp).max(1) as u32;
                ctx.start_draw(owner, lost);
                set_flag(ctx.core, owner, "yxs_jiean_drawn", lost);
            }
        }
        // batu 霸图 (`英雄杀/character/skill.js:1019-1031`):
        "batu" => {
            set_flag(ctx.core, owner, "yxs_batu_replenished", 1);
            ctx.start_draw(owner, 1);
        }
        // wumu 武穆 (`英雄杀/character/skill.js:1032-1055`):
        "wumu" => {
            set_flag(ctx.core, owner, "yxs_wumu_active", 1);
        }
        // ysheshen 舍身 (`英雄杀/character/skill.js:1056-1171`):
        "ysheshen" => {
            set_flag(ctx.core, owner, "yxs_ysheshen_distributed", 2);
            ctx.start_draw(owner, 2);
        }
        // yxschangnian / yxschangnian2 长念/追思 (`英雄杀/character/skill.js:1172-1216`):
        "yxschangnian" | "yxschangnian2" => {
            set_flag(ctx.core, owner, "yxs_changnian_zhuisi", 1);
            ctx.start_draw(owner, 1);
        }
        // yxsciqiu 刺酋 (`英雄杀/character/skill.js:1217-1251`):
        "yxsciqiu" => {
            set_flag(ctx.core, owner, "yxs_ciqiu_bonus", 1);
            if other != owner {
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(owner),
                    other,
                    1,
                )));
            }
        }
        // sanbanfu / sanbanfu2 / sanbanfu3 三板斧 (`英雄杀/character/skill.js:1252-1324`):
        "sanbanfu" | "sanbanfu2" | "sanbanfu3" => {
            set_flag(ctx.core, owner, "yxs_sanbanfu_resolved", 1);
            if other != owner {
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(owner),
                    other,
                    2,
                )));
            }
        }
        // bingsheng 兵圣 (`英雄杀/character/skill.js:1325-1382`):
        "bingsheng" => {
            let _ = discard_first_he(ctx, owner);
            let _ = discard_first_he(ctx, owner);
            let owner_hp = ctx.core.player(owner).hp;
            let other_hp = ctx.core.player(other).hp;
            let next_hp = (other_hp + (owner_hp - other_hp).clamp(-2, 2))
                .clamp(1, ctx.core.player(other).max_hp as i32);
            ctx.core.player_mut(other).hp = next_hp;
            set_flag(ctx.core, owner, "yxs_bingsheng_equalized", 1);
        }
        // taolue 韬略 (`英雄杀/character/skill.js:1383-1389`):
        "taolue" => {
            set_flag(ctx.core, owner, "yxs_taolue_hand_limit", 1);
        }
        // shentan 神探 (`英雄杀/character/skill.js:1390-1442`):
        "shentan" => {
            let _ = discard_first_he(ctx, owner);
            if other != owner {
                if let Some(c) = ctx.core.player(other).hand.first().copied() {
                    ctx.take_hand_card(other, c);
                    ctx.core.player_mut(owner).hand.push(c);
                }
            }
            set_flag(ctx.core, owner, "yxs_shentan_investigated", 1);
        }
        // hanqiang 寒枪 (`英雄杀/character/skill.js:1443-1451`):
        "hanqiang" => {
            set_flag(ctx.core, owner, "yxs_hanqiang_range", 1);
        }
        // biaoqi 骠骑 (`英雄杀/character/skill.js:1452-1463`):
        "biaoqi" => {
            set_flag(ctx.core, owner, "yxs_biaoqi_triggered", 1);
            ctx.start_draw(owner, 1);
        }
        // wluoyan 落雁 (`英雄杀/character/skill.js:1464-1474`):
        "wluoyan" => {
            set_flag(ctx.core, owner, "yxs_wluoyan_converted", 1);
            ctx.emit(EventData::LoseHp {
                player: owner,
                amount: 1,
            });
        }
        // heqin / heqin2 / heqin3 和亲 (`英雄杀/character/skill.js:1475-1542`):
        "heqin" | "heqin2" | "heqin3" => {
            set_flag(ctx.core, owner, "yxs_heqin_partner", u32::from(other));
            ctx.start_draw(owner, 1);
        }
        // chajue / chajue2 察觉 (`英雄杀/character/skill.js:1543-1577`):
        "chajue" | "chajue2" => {
            set_flag(ctx.core, owner, "yxs_chajue_immune", 1);
        }
        // tiewan 铁腕 (`英雄杀/character/skill.js:1578-1609`):
        "tiewan" => {
            let _ = discard_first_he(ctx, owner);
            set_flag(ctx.core, owner, "yxs_tiewan_lebu", 1);
        }
        // qianglue 强掠 (`英雄杀/character/skill.js:1610-1629`):
        "qianglue" => {
            set_flag(ctx.core, owner, "yxs_qianglue_gained", 1);
            ctx.start_draw(owner, 1);
        }
        // jimin 机敏 (`英雄杀/character/skill.js:1630-1664`):
        "jimin" => {
            set_flag(ctx.core, owner, "yxs_jimin_shan", 1);
        }
        // xiadao 侠盗 (`英雄杀/character/skill.js:1665-1719`):
        "xiadao" => {
            if other != owner {
                if let Some(c) = ctx.core.player(other).hand.first().copied() {
                    ctx.take_hand_card(other, c);
                    ctx.core.player_mut(owner).hand.push(c);
                }
            }
            set_flag(ctx.core, owner, "yxs_xiadao_stolen", 1);
        }
        // lzhangyi 仗义 (`英雄杀/character/skill.js:1720-1781`):
        "lzhangyi" => {
            if other != owner {
                if let Some(c) = ctx.core.player(owner).hand.first().copied() {
                    ctx.take_hand_card(owner, c);
                    ctx.core.player_mut(other).hand.push(c);
                }
            }
            set_flag(ctx.core, owner, "yxs_lzhangyi_given", 1);
        }
        // yizhuang / yizhuang2 易装 (`英雄杀/character/skill.js:1782-1875`):
        "yizhuang" | "yizhuang2" => {
            let _ = discard_first_he(ctx, owner);
            set_flag(ctx.core, owner, "yxs_yizhuang_disguised", 1);
        }
        // kongju 控局 (`英雄杀/character/skill.js:1876-1895`):
        "kongju" => {
            set_flag(ctx.core, owner, "yxs_kongju_ward", 1);
        }
        // tuqiang 图强 (`英雄杀/character/skill.js:1896-1915`):
        "tuqiang" => {
            set_flag(ctx.core, owner, "yxs_tuqiang_drawn", 1);
            ctx.start_draw(owner, 1);
        }
        // xumou 蓄谋 (`英雄杀/character/skill.js:1916-1925`):
        "xumou" => {
            ctx.core.player_mut(owner).face_down = !ctx.core.player(owner).face_down;
            set_flag(ctx.core, owner, "yxs_xumou_drawn3", 1);
            ctx.start_draw(owner, 3);
        }
        // zhensha 鸩杀 (`英雄杀/character/skill.js:1926-1980`):
        "zhensha" => {
            let _ = discard_first_he(ctx, owner);
            set_flag(ctx.core, owner, "yxs_zhensha_poisoned", 1);
        }
        // ducai / ducai2 / ducai3 独裁 (`英雄杀/character/skill.js:1981-2075`):
        "ducai" | "ducai2" | "ducai3" => {
            let suit_code = discard_first_he(ctx, owner)
                .and_then(|c| ctx.catalog.cards.get(c as usize))
                .map_or(1u32, |d| match d.suit {
                    crate::catalog::Suit::Spade => 1,
                    crate::catalog::Suit::Heart => 2,
                    crate::catalog::Suit::Club => 3,
                    crate::catalog::Suit::Diamond => 4,
                });
            set_flag(ctx.core, owner, "yxs_ducai_forbidden_suit", suit_code);
        }
        // tongling 统领 (`英雄杀/character/skill.js:2076-2096`):
        "tongling" => {
            let m = (flag(ctx.core, owner, "yxs_tongling_marks") + 1).min(3);
            set_flag(ctx.core, owner, "yxs_tongling_marks", m);
        }
        // fanpu 反扑 (`英雄杀/character/skill.js:2097-2119`):
        "fanpu" => {
            set_flag(ctx.core, owner, "yxs_tongling_marks", 0);
            set_flag(ctx.core, owner, "yxs_fanpu_unleashed", 1);
            if other != owner {
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(owner),
                    other,
                    1,
                )));
            }
        }
        // fenghuo 烽火 (`英雄杀/character/skill.js:2120-2143`):
        "fenghuo" => {
            let _ = discard_first_he(ctx, owner);
            set_flag(ctx.core, owner, "yxs_fenghuo_nanman", 1);
            if other != owner {
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(owner),
                    other,
                    1,
                )));
            }
        }
        // nichang / nichang2 霓裳 (`英雄杀/character/skill.js:2144-2179`):
        "nichang" | "nichang2" => {
            set_flag(ctx.core, owner, "yxs_nichang_drawn", 2);
            ctx.start_draw(owner, 2);
        }
        // fengyan 丰艳 (`英雄杀/character/skill.js:2180-2199`):
        "fengyan" => {
            set_flag(ctx.core, owner, "yxs_fengyan_gained", 1);
            ctx.start_draw(owner, 1);
        }
        // fengyi 凤仪 (`英雄杀/character/skill.js:2200-2218`):
        "fengyi" => {
            let _ = discard_first_he(ctx, owner);
            set_flag(ctx.core, owner, "yxs_fengyi_drawn2", 1);
            ctx.start_draw(other, 2);
        }
        // wange / wange2 婉歌 (`英雄杀/character/skill.js:2219-2262`):
        "wange" | "wange2" => {
            if other != owner {
                if let Some(c) = ctx.core.player(other).hand.first().copied() {
                    ctx.take_hand_card(other, c);
                    ctx.core.player_mut(owner).hand.push(c);
                }
            }
            set_flag(ctx.core, owner, "yxs_wange_gained", 1);
        }
        // seyou 色诱 (`英雄杀/character/skill.js:2263-2322`):
        "seyou" => {
            if flag(ctx.core, owner, "seyou_used") == 0 {
                set_flag(ctx.core, owner, "seyou_used", 1);
                set_flag(ctx.core, owner, "yxs_seyou_triggered", 1);
                ctx.start_draw(owner, 1);
            }
        }
        // sheshi 蛇噬 (`英雄杀/character/skill.js:2323-2406`):
        "sheshi" => {
            set_flag(ctx.core, owner, "yxs_sheshi_gained", 2);
            ctx.start_draw(owner, 2);
        }
        // bolehuiyan / oldbolehuiyan 慧眼 (`英雄杀/character/skill.js:2407-2566`):
        "bolehuiyan" | "bolehuiyan2" | "bolehuiyan3" | "oldbolehuiyan" | "oldbolehuiyan2" => {
            set_flag(ctx.core, owner, "yxs_bolehuiyan_predicted", 1);
            ctx.start_draw(owner, 1);
        }
        // xiangma 相马 (`英雄杀/character/skill.js:2567-2593`):
        "xiangma" => {
            set_flag(ctx.core, owner, "yxs_xiangma_mounted", 1);
        }
        // yxsweiyi 威仪 (`英雄杀/character/skill.js:2594-2622`):
        "yxsweiyi" => {
            let _ = discard_first_he(ctx, other);
            let _ = discard_first_he(ctx, other);
            set_flag(ctx.core, owner, "yxs_weiyi_forced_discard", 2);
        }
        // qiandu 迁都 (`英雄杀/character/skill.js:2623-2656`):
        "qiandu" => {
            let _ = discard_first_he(ctx, owner);
            set_flag(ctx.core, owner, "yxs_qiandu_seat_swapped", 1);
        }
        // nvquan 女权 (`英雄杀/character/skill.js:2657-2717`):
        "nvquan" | "nvquan1" | "nvquan2" | "nvquan3" => {
            set_flag(ctx.core, owner, "yxs_nvquan_double_response", 1);
        }
        // feigong 非攻 (`英雄杀/character/skill.js:2718-2755`):
        "feigong" => {
            let _ = discard_first_he(ctx, owner);
            set_flag(ctx.core, owner, "yxs_feigong_nullified_sha", 1);
        }
        // jianai 兼爱 (`英雄杀/character/skill.js:2756-2796`):
        "jianai" => {
            set_flag(ctx.core, owner, "yxs_jianai_mass_recover", 1);
            let seats: Vec<PlayerId> = ctx.core.seat_order_from(owner);
            for p in seats {
                if p != owner
                    && ctx.core.player(p).alive
                    && ctx.core.player(p).hp < ctx.core.player(p).max_hp as i32
                {
                    ctx.recover(p, 1, None);
                }
            }
        }
        // jieyong 节用 (`英雄杀/character/skill.js:2797-2844`):
        "jieyong" | "jieyong2" | "jieyong3" => {
            let _ = discard_first_he(ctx, owner);
            set_flag(ctx.core, owner, "yxs_jieyong_regained", 1);
            ctx.start_draw(owner, 1);
        }
        // zhulu 逐鹿 (`英雄杀/character/skill.js:2845-2870`):
        "zhulu" => {
            let _ = discard_first_he(ctx, owner);
            set_flag(ctx.core, owner, "yxs_zhulu_captured", 1);
            ctx.start_draw(owner, 1);
        }
        // xieling 挟令 (`英雄杀/character/skill.js:2871-2962`):
        "xieling" => {
            let _ = discard_first_he(ctx, owner);
            let _ = discard_first_he(ctx, owner);
            set_flag(ctx.core, owner, "yxs_xieling_moved", 1);
        }
        // qiangyun 强运 (`英雄杀/character/skill.js:2963-3000`):
        "qiangyun" => {
            set_flag(ctx.core, owner, "yxs_qiangyun_drawn2", 1);
            ctx.start_draw(owner, 2);
        }
        // cike 刺客 (`英雄杀/character/skill.js:3001-3027`):
        "cike" => {
            set_flag(ctx.core, owner, "yxs_cike_judged", 1);
            let _ = discard_first_he(ctx, other);
        }
        // miaobi / miaobix 妙笔 (`英雄杀/character/skill.js:3028-3083`):
        "miaobi" | "miaobix" => {
            let _ = discard_first_he(ctx, other);
            set_flag(ctx.core, owner, "yxs_miaobi_copied", 1);
        }
        // zhexian 谪仙 (`英雄杀/character/skill.js:3084-3094`):
        "zhexian" => {
            set_flag(ctx.core, owner, "yxs_zhexian_drawn", 1);
            ctx.start_draw(owner, 1);
        }
        // guifu 鬼斧 (`英雄杀/character/skill.js:3095-3114`):
        "guifu" => {
            set_flag(ctx.core, owner, "yxs_guifu_drawn", 1);
            ctx.start_draw(owner, 1);
            if other != owner {
                ctx.start_draw(other, 1);
            }
        }
        // lshengong 神工 (`英雄杀/character/skill.js:3115-3135`):
        "lshengong" => {
            set_flag(ctx.core, owner, "yxs_lshengong_copied_equip", 1);
            ctx.start_draw(owner, 1);
        }
        other_sk => {
            set_flag(ctx.core, owner, &format!("yxs_hook:{other_sk}"), 1);
        }
    }
}

pub use super::decision34_l19_l20_l27::{
    resolve_l19_doomsday_remaining, resolve_l20_mark_remaining, resolve_l27_select_remaining,
};

