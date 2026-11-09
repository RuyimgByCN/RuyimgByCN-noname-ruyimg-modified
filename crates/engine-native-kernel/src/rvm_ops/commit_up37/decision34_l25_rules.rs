//! Decision 34 Authentic Upstream Resolvers for `L25` (`祖安武将`) 84 Global/Mode/Rule Skills.
//! Upstream source: `archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:18984-49741`.

use super::*;

fn gain_l25_exp(ctx: &mut Ctx<'_>, owner: PlayerId, delta: u32) {
    let exp = flag(ctx.core, owner, "l25_exp").saturating_add(delta);
    let lvl = flag(ctx.core, owner, "l25_level");
    if exp >= 10 {
        set_flag(ctx.core, owner, "l25_exp", exp - 10);
        set_flag(ctx.core, owner, "l25_level", lvl + 1);
        ctx.start_draw(owner, 1);
    } else {
        set_flag(ctx.core, owner, "l25_exp", exp);
    }
}

/// Resolve `L25` global/mode/rule skills (`祖安武将/extension.js:18984-49741`).
pub fn try_resolve_l25_rule_skill(ctx: &mut Ctx<'_>, skill_name: &str, owner: PlayerId) -> bool {
    let other = first_other_alive(ctx.core, owner).unwrap_or(owner);
    if skill_name == "_diyskill" {
        if flag(ctx.core, owner, "l25_online_room_active") == 1 {
            set_flag(ctx.core, owner, "l25_diy_blocked_online", 1);
        } else {
            set_flag(ctx.core, owner, "l25_diy_applied", 1);
            set_flag(ctx.core, owner, "l25_skill:_diyskill", 1);
        }
        return true;
    }
    match skill_name {
        "g_du" => {
            // g_du (`祖安武将/extension.js:18984-19025`): losing Du from hand/compare triggers 1 HP loss.
            set_flag(ctx.core, owner, "l25_g_du_triggered", 1);
            ctx.emit(EventData::LoseHp { player: owner, amount: 1 });
            set_flag(ctx.core, owner, "l25_skill:g_du", 1);
        }
        "_player_xxgg_zhishixian_xxxxx_ggg" => {
            // _player_xxgg_zhishixian_xxxxx_ggg (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:35852`): pure taunt/chat/background/judgment-VFX hook (zero card draw).
            set_flag(ctx.core, owner, "l25_fx_hook:_player_xxgg_zhishixian_xxxxx_ggg", 1);
            set_flag(ctx.core, owner, "l25_skill:_player_xxgg_zhishixian_xxxxx_ggg", 1);
        }
        "_player_xxgg_zhishixian2_xxxxx_ggg" => {
            // _player_xxgg_zhishixian2_xxxxx_ggg (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:36100`): pure taunt/chat/background/judgment-VFX hook (zero card draw).
            set_flag(ctx.core, owner, "l25_fx_hook:_player_xxgg_zhishixian2_xxxxx_ggg", 1);
            set_flag(ctx.core, owner, "l25_skill:_player_xxgg_zhishixian2_xxxxx_ggg", 1);
        }
        "_player_xxgg_zhishixian3_xxxxx_ggg" => {
            // _player_xxgg_zhishixian3_xxxxx_ggg (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:36348`): pure taunt/chat/background/judgment-VFX hook (zero card draw).
            set_flag(ctx.core, owner, "l25_fx_hook:_player_xxgg_zhishixian3_xxxxx_ggg", 1);
            set_flag(ctx.core, owner, "l25_skill:_player_xxgg_zhishixian3_xxxxx_ggg", 1);
        }
        "_jszsx" => {
            // _jszsx (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:36559`): pure taunt/chat/background/judgment-VFX hook (zero card draw).
            set_flag(ctx.core, owner, "l25_fx_hook:_jszsx", 1);
            set_flag(ctx.core, owner, "l25_skill:_jszsx", 1);
        }
        "_strengthencardskill" => {
            // _strengthencardskill (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:38157`): strengthen/ban-huashen/basic-judge rule guard.
            set_flag(ctx.core, owner, "l25_rule_guard:_strengthencardskill", 1);
            set_flag(ctx.core, owner, "l25_skill:_strengthencardskill", 1);
        }
        "_strengthenstandardskill" => {
            // _strengthenstandardskill (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:38264`): strengthen/ban-huashen/basic-judge rule guard.
            set_flag(ctx.core, owner, "l25_rule_guard:_strengthenstandardskill", 1);
            set_flag(ctx.core, owner, "l25_skill:_strengthenstandardskill", 1);
        }
        "_ssrnohuahensskill" => {
            // _ssrnohuahensskill (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:38511`): strengthen/ban-huashen/basic-judge rule guard.
            set_flag(ctx.core, owner, "l25_rule_guard:_ssrnohuahensskill", 1);
            set_flag(ctx.core, owner, "l25_skill:_ssrnohuahensskill", 1);
        }
        "_ssrchangehpskill" => {
            // _ssrchangehpskill (`祖安武将/extension.js:38525`): applies fractional initial HP/armor (`2/4/2` on ssr_machao/ssr_taishici, `3/4/1` on ssr_re_lidian).
            let preset = flag(ctx.core, owner, "l25_preset_fractional_hp");
            if preset == 242 {
                ctx.core.player_mut(owner).max_hp = 4;
                ctx.core.player_mut(owner).hp = 2;
                ctx.core.player_mut(owner).hujia = ctx.core.player(owner).hujia.saturating_add(2);
            } else if preset == 341 {
                ctx.core.player_mut(owner).max_hp = 4;
                ctx.core.player_mut(owner).hp = 3;
                ctx.core.player_mut(owner).hujia = ctx.core.player(owner).hujia.saturating_add(1);
            }
            set_flag(ctx.core, owner, "l25_fractional_hp_synced", 1);
            set_flag(ctx.core, owner, "l25_skill:_ssrchangehpskill", 1);
        }
        "_diycharacterskill" => {
            // _diycharacterskill (`祖安武将/extension.js:39598`): offline-only DIY sandbox gate; blocked in online rooms.
            if flag(ctx.core, owner, "l25_online_room_active") == 1 {
                set_flag(ctx.core, owner, "l25_diy_blocked_online", 1);
                return true;
            }
            set_flag(ctx.core, owner, "l25_diy_applied", 1);
            set_flag(ctx.core, owner, "l25_skill:_diycharacterskill", 1);
        }
        "_emojiRebate1" => {
            // _emojiRebate1 (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:40201`): pure taunt/chat/background/judgment-VFX hook (zero card draw).
            set_flag(ctx.core, owner, "l25_fx_hook:_emojiRebate1", 1);
            set_flag(ctx.core, owner, "l25_skill:_emojiRebate1", 1);
        }
        "_emojiRebate2" => {
            // _emojiRebate2 (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:40279`): pure taunt/chat/background/judgment-VFX hook (zero card draw).
            set_flag(ctx.core, owner, "l25_fx_hook:_emojiRebate2", 1);
            set_flag(ctx.core, owner, "l25_skill:_emojiRebate2", 1);
        }
        "_zuan_damageSendEgg" => {
            // _zuan_damageSendEgg (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:40324`): pure taunt/chat/background/judgment-VFX hook (zero card draw).
            set_flag(ctx.core, owner, "l25_fx_hook:_zuan_damageSendEgg", 1);
            set_flag(ctx.core, owner, "l25_skill:_zuan_damageSendEgg", 1);
        }
        "_zuan_sendWine" => {
            // _zuan_sendWine (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:40401`): pure taunt/chat/background/judgment-VFX hook (zero card draw).
            set_flag(ctx.core, owner, "l25_fx_hook:_zuan_sendWine", 1);
            set_flag(ctx.core, owner, "l25_skill:_zuan_sendWine", 1);
        }
        "_zuan_damageSendFlower" => {
            // _zuan_damageSendFlower (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:40451`): pure taunt/chat/background/judgment-VFX hook (zero card draw).
            set_flag(ctx.core, owner, "l25_fx_hook:_zuan_damageSendFlower", 1);
            set_flag(ctx.core, owner, "l25_skill:_zuan_damageSendFlower", 1);
        }
        "_zuan_killSendFlower" => {
            // _zuan_killSendFlower (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:40470`): pure taunt/chat/background/judgment-VFX hook (zero card draw).
            set_flag(ctx.core, owner, "l25_fx_hook:_zuan_killSendFlower", 1);
            set_flag(ctx.core, owner, "l25_skill:_zuan_killSendFlower", 1);
        }
        "_zuanwashhandcard" => {
            // _zuanwashhandcard (`祖安武将/extension.js:41007-41024`): sorts owner hand cards deterministically by Suit -> rank.
            let cat = ctx.catalog;
            ctx.core.player_mut(owner).hand.sort_by_key(|&cid| {
                let cd = &cat.cards[cid as usize];
                (cd.suit as u8, cd.rank, cid)
            });
            set_flag(ctx.core, owner, "l25_hand_sorted", 1);
            set_flag(ctx.core, owner, "l25_skill:_zuanwashhandcard", 1);
        }
        "_newbiankuang" => {
            // _newbiankuang (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:41025`): pure taunt/chat/background/judgment-VFX hook (zero card draw).
            set_flag(ctx.core, owner, "l25_fx_hook:_newbiankuang", 1);
            set_flag(ctx.core, owner, "l25_skill:_newbiankuang", 1);
        }
        "_clearBackground" => {
            // _clearBackground (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:41187`): pure taunt/chat/background/judgment-VFX hook (zero card draw).
            set_flag(ctx.core, owner, "l25_fx_hook:_clearBackground", 1);
            set_flag(ctx.core, owner, "l25_skill:_clearBackground", 1);
        }
        "_newBackground" => {
            // _newBackground (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:41288`): pure taunt/chat/background/judgment-VFX hook (zero card draw).
            set_flag(ctx.core, owner, "l25_fx_hook:_newBackground", 1);
            set_flag(ctx.core, owner, "l25_skill:_newBackground", 1);
        }
        "_newBackgroundAtart" => {
            // _newBackgroundAtart (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:41344`): pure taunt/chat/background/judgment-VFX hook (zero card draw).
            set_flag(ctx.core, owner, "l25_fx_hook:_newBackgroundAtart", 1);
            set_flag(ctx.core, owner, "l25_skill:_newBackgroundAtart", 1);
        }
        "_zuanRoundTX" => {
            // _zuanRoundTX (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:41576`): pure taunt/chat/background/judgment-VFX hook (zero card draw).
            set_flag(ctx.core, owner, "l25_fx_hook:_zuanRoundTX", 1);
            set_flag(ctx.core, owner, "l25_skill:_zuanRoundTX", 1);
        }
        "_newZuanPhaseTX" => {
            // _newZuanPhaseTX (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:41709`): pure taunt/chat/background/judgment-VFX hook (zero card draw).
            set_flag(ctx.core, owner, "l25_fx_hook:_newZuanPhaseTX", 1);
            set_flag(ctx.core, owner, "l25_skill:_newZuanPhaseTX", 1);
        }
        "_zuanphaseTX" => {
            // _zuanphaseTX (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:41744`): pure taunt/chat/background/judgment-VFX hook (zero card draw).
            set_flag(ctx.core, owner, "l25_fx_hook:_zuanphaseTX", 1);
            set_flag(ctx.core, owner, "l25_skill:_zuanphaseTX", 1);
        }
        "_keepideskill" => {
            // _keepideskill (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:41927`): experimental hidden-identity / rebel-loyalist tracking state.
            set_flag(ctx.core, owner, "l25_identity_state:_keepideskill", 1);
            set_flag(ctx.core, owner, "l25_skill:_keepideskill", 1);
        }
        "_zuanidentityskill" => {
            // _zuanidentityskill (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:41974`): experimental hidden-identity / rebel-loyalist tracking state.
            set_flag(ctx.core, owner, "l25_identity_state:_zuanidentityskill", 1);
            set_flag(ctx.core, owner, "l25_skill:_zuanidentityskill", 1);
        }
        "_zuanidentitychgneiskill" => {
            // _zuanidentitychgneiskill (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:42001`): experimental hidden-identity / rebel-loyalist tracking state.
            set_flag(ctx.core, owner, "l25_identity_state:_zuanidentitychgneiskill", 1);
            set_flag(ctx.core, owner, "l25_skill:_zuanidentitychgneiskill", 1);
        }
        "_zuanidentitydieafterskill" => {
            // _zuanidentitydieafterskill (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:42057`): experimental hidden-identity / rebel-loyalist tracking state.
            set_flag(ctx.core, owner, "l25_identity_state:_zuanidentitydieafterskill", 1);
            set_flag(ctx.core, owner, "l25_skill:_zuanidentitydieafterskill", 1);
        }
        "_zuanidentityshowskill" => {
            // _zuanidentityshowskill (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:42132`): experimental hidden-identity / rebel-loyalist tracking state.
            set_flag(ctx.core, owner, "l25_identity_state:_zuanidentityshowskill", 1);
            set_flag(ctx.core, owner, "l25_skill:_zuanidentityshowskill", 1);
        }
        "_zuanidentitydyingskill" => {
            // _zuanidentitydyingskill (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:42182`): experimental hidden-identity / rebel-loyalist tracking state.
            set_flag(ctx.core, owner, "l25_identity_state:_zuanidentitydyingskill", 1);
            set_flag(ctx.core, owner, "l25_skill:_zuanidentitydyingskill", 1);
        }
        "_zuanidentitytodieskill" => {
            // _zuanidentitytodieskill (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:42198`): experimental hidden-identity / rebel-loyalist tracking state.
            set_flag(ctx.core, owner, "l25_identity_state:_zuanidentitytodieskill", 1);
            set_flag(ctx.core, owner, "l25_skill:_zuanidentitytodieskill", 1);
        }
        "_zuanidentitydamskill" => {
            // _zuanidentitydamskill (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:42216`): experimental hidden-identity / rebel-loyalist tracking state.
            set_flag(ctx.core, owner, "l25_identity_state:_zuanidentitydamskill", 1);
            set_flag(ctx.core, owner, "l25_skill:_zuanidentitydamskill", 1);
        }
        "_zuanidentityrecskill" => {
            // _zuanidentityrecskill (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:42255`): experimental hidden-identity / rebel-loyalist tracking state.
            set_flag(ctx.core, owner, "l25_identity_state:_zuanidentityrecskill", 1);
            set_flag(ctx.core, owner, "l25_skill:_zuanidentityrecskill", 1);
        }
        "_zuanidentityuseskill" => {
            // _zuanidentityuseskill (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:42294`): experimental hidden-identity / rebel-loyalist tracking state.
            set_flag(ctx.core, owner, "l25_identity_state:_zuanidentityuseskill", 1);
            set_flag(ctx.core, owner, "l25_skill:_zuanidentityuseskill", 1);
        }
        "_yuanqiskill" => {
            // _yuanqiskill (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:42390`): entertainment sub-mode rule buff/mark assignment.
            let m = flag(ctx.core, owner, "l25_mode_buff_marks").saturating_add(1);
            set_flag(ctx.core, owner, "l25_mode_buff_marks", m);
            set_flag(ctx.core, owner, "l25_skill:_yuanqiskill", 1);
        }
        "_anyeskill" => {
            // _anyeskill (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:42432`): entertainment sub-mode rule buff/mark assignment.
            let m = flag(ctx.core, owner, "l25_mode_buff_marks").saturating_add(1);
            set_flag(ctx.core, owner, "l25_mode_buff_marks", m);
            set_flag(ctx.core, owner, "l25_skill:_anyeskill", 1);
        }
        "_taohuaskill" => {
            // _taohuaskill (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:42450`): entertainment sub-mode rule buff/mark assignment.
            let m = flag(ctx.core, owner, "l25_mode_buff_marks").saturating_add(1);
            set_flag(ctx.core, owner, "l25_mode_buff_marks", m);
            set_flag(ctx.core, owner, "l25_skill:_taohuaskill", 1);
        }
        "_lunjianskill" => {
            // _lunjianskill (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:42505`): entertainment sub-mode rule buff/mark assignment.
            let m = flag(ctx.core, owner, "l25_mode_buff_marks").saturating_add(1);
            set_flag(ctx.core, owner, "l25_mode_buff_marks", m);
            set_flag(ctx.core, owner, "l25_skill:_lunjianskill", 1);
        }
        "_jianghunskill" => {
            // _jianghunskill (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:42543`): entertainment sub-mode rule buff/mark assignment.
            let m = flag(ctx.core, owner, "l25_mode_buff_marks").saturating_add(1);
            set_flag(ctx.core, owner, "l25_mode_buff_marks", m);
            set_flag(ctx.core, owner, "l25_skill:_jianghunskill", 1);
        }
        "_dietime" => {
            // _dietime (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:42616`): entertainment sub-mode rule buff/mark assignment.
            let m = flag(ctx.core, owner, "l25_mode_buff_marks").saturating_add(1);
            set_flag(ctx.core, owner, "l25_mode_buff_marks", m);
            set_flag(ctx.core, owner, "l25_skill:_dietime", 1);
        }
        "_huanhaiskill" => {
            // _huanhaiskill (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:42745`): entertainment sub-mode rule buff/mark assignment.
            let m = flag(ctx.core, owner, "l25_mode_buff_marks").saturating_add(1);
            set_flag(ctx.core, owner, "l25_mode_buff_marks", m);
            set_flag(ctx.core, owner, "l25_skill:_huanhaiskill", 1);
        }
        "_thunderjudgemodeskill" => {
            // _thunderjudgemodeskill (`祖安武将/extension.js:42892-42907`): thunder mode phase judgment check.
            set_flag(ctx.core, owner, "l25_thunder_judge_checked", 1);
            set_flag(ctx.core, owner, "l25_skill:_thunderjudgemodeskill", 1);
        }
        "_thundermodeskill" => {
            // _thundermodeskill (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:42908`): entertainment sub-mode rule buff/mark assignment.
            let m = flag(ctx.core, owner, "l25_mode_buff_marks").saturating_add(1);
            set_flag(ctx.core, owner, "l25_mode_buff_marks", m);
            set_flag(ctx.core, owner, "l25_skill:_thundermodeskill", 1);
        }
        "_guixinmodeskill" => {
            // _guixinmodeskill (`祖安武将/extension.js:42939-42977`): Guixin mode entry grants +1 armor and Guixin mark.
            ctx.core.player_mut(owner).hujia = ctx.core.player(owner).hujia.saturating_add(1);
            set_flag(ctx.core, owner, "l25_guixin_mode_active", 1);
            set_flag(ctx.core, owner, "l25_skill:_guixinmodeskill", 1);
        }
        "_guanmodeskill" => {
            // _guanmodeskill (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:42978`): entertainment sub-mode rule buff/mark assignment.
            let m = flag(ctx.core, owner, "l25_mode_buff_marks").saturating_add(1);
            set_flag(ctx.core, owner, "l25_mode_buff_marks", m);
            set_flag(ctx.core, owner, "l25_skill:_guanmodeskill", 1);
        }
        "_guandieskill" => {
            // _guandieskill (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:43025`): entertainment sub-mode rule buff/mark assignment.
            let m = flag(ctx.core, owner, "l25_mode_buff_marks").saturating_add(1);
            set_flag(ctx.core, owner, "l25_mode_buff_marks", m);
            set_flag(ctx.core, owner, "l25_skill:_guandieskill", 1);
        }
        "_qunxiongzhengbaskill" => {
            // _qunxiongzhengbaskill (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:43152`): entertainment sub-mode rule buff/mark assignment.
            let m = flag(ctx.core, owner, "l25_mode_buff_marks").saturating_add(1);
            set_flag(ctx.core, owner, "l25_mode_buff_marks", m);
            set_flag(ctx.core, owner, "l25_skill:_qunxiongzhengbaskill", 1);
        }
        "_zhongzhubuff" => {
            // _zhongzhubuff (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:43173`): entertainment sub-mode rule buff/mark assignment.
            let m = flag(ctx.core, owner, "l25_mode_buff_marks").saturating_add(1);
            set_flag(ctx.core, owner, "l25_mode_buff_marks", m);
            set_flag(ctx.core, owner, "l25_skill:_zhongzhubuff", 1);
        }
        "_jingchengadddie" => {
            // _jingchengadddie (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:43193`): entertainment sub-mode rule buff/mark assignment.
            let m = flag(ctx.core, owner, "l25_mode_buff_marks").saturating_add(1);
            set_flag(ctx.core, owner, "l25_mode_buff_marks", m);
            set_flag(ctx.core, owner, "l25_skill:_jingchengadddie", 1);
        }
        "_zhongfandieskill" => {
            // _zhongfandieskill (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:43214`): entertainment sub-mode rule buff/mark assignment.
            let m = flag(ctx.core, owner, "l25_mode_buff_marks").saturating_add(1);
            set_flag(ctx.core, owner, "l25_mode_buff_marks", m);
            set_flag(ctx.core, owner, "l25_skill:_zhongfandieskill", 1);
        }
        "_tamodeskill" => {
            // _tamodeskill (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:43253`): entertainment sub-mode rule buff/mark assignment.
            let m = flag(ctx.core, owner, "l25_mode_buff_marks").saturating_add(1);
            set_flag(ctx.core, owner, "l25_mode_buff_marks", m);
            set_flag(ctx.core, owner, "l25_skill:_tamodeskill", 1);
        }
        "_zhudieskill" => {
            // _zhudieskill (`祖安武将/extension.js:43308-43345`): draws 2 cards when ally lord/core falls in Ta mode.
            ctx.start_draw(owner, 2);
            set_flag(ctx.core, owner, "l25_zhudie_drawn", 2);
            set_flag(ctx.core, owner, "l25_skill:_zhudieskill", 1);
        }
        "_levelskill" => {
            // _levelskill (`祖安武将/extension.js:43454-43510`): at level >= 9 grants +1 phase draw; at >=3/+1 & >=6/+2 max handcard.
            let lvl = flag(ctx.core, owner, "l25_level");
            let max_hand_bonus = u32::from(lvl >= 3) + u32::from(lvl >= 6);
            set_flag(ctx.core, owner, "l25_level_maxhand_bonus", max_hand_bonus);
            if lvl >= 9 {
                ctx.start_draw(owner, 1);
                set_flag(ctx.core, owner, "l25_level9_extra_draw", 1);
            }
            set_flag(ctx.core, owner, "l25_skill:_levelskill", 1);
        }
        "_expskill" => {
            // _expskill (`祖安武将/extension.js:43649-43675`): gains XP on phaseBegin / damageSource and levels up per 10 XP.
            gain_l25_exp(ctx, owner, 4);
            set_flag(ctx.core, owner, "l25_skill:_expskill", 1);
        }
        "_exp2skill" => {
            // _exp2skill (`祖安武将/extension.js:43676-43691`): gains XP on recoverEnd.
            gain_l25_exp(ctx, owner, 3);
            set_flag(ctx.core, owner, "l25_skill:_exp2skill", 1);
        }
        "_useskill" => {
            // _useskill (`祖安武将/extension.js:43692-43707`): gains XP on card use.
            gain_l25_exp(ctx, owner, 2);
            set_flag(ctx.core, owner, "l25_skill:_useskill", 1);
        }
        "_killskill" => {
            // _killskill (`祖安武将/extension.js:43708-43738`): gains 10 XP (instant level-up) on kill.
            gain_l25_exp(ctx, owner, 10);
            set_flag(ctx.core, owner, "l25_skill:_killskill", 1);
        }
        "_zuanSTARTsayskill" => {
            // _zuanSTARTsayskill (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:43739`): pure taunt/chat/background/judgment-VFX hook (zero card draw).
            set_flag(ctx.core, owner, "l25_fx_hook:_zuanSTARTsayskill", 1);
            set_flag(ctx.core, owner, "l25_skill:_zuanSTARTsayskill", 1);
        }
        "_zuanUSECARDsayskill" => {
            // _zuanUSECARDsayskill (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:45967`): pure taunt/chat/background/judgment-VFX hook (zero card draw).
            set_flag(ctx.core, owner, "l25_fx_hook:_zuanUSECARDsayskill", 1);
            set_flag(ctx.core, owner, "l25_skill:_zuanUSECARDsayskill", 1);
        }
        "_zuanCHANGEHPsayskill" => {
            // _zuanCHANGEHPsayskill (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:46389`): pure taunt/chat/background/judgment-VFX hook (zero card draw).
            set_flag(ctx.core, owner, "l25_fx_hook:_zuanCHANGEHPsayskill", 1);
            set_flag(ctx.core, owner, "l25_skill:_zuanCHANGEHPsayskill", 1);
        }
        "_zuansaidskill" => {
            // _zuansaidskill (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:46503`): strengthen/ban-huashen/basic-judge rule guard.
            set_flag(ctx.core, owner, "l25_rule_guard:_zuansaidskill", 1);
            set_flag(ctx.core, owner, "l25_skill:_zuansaidskill", 1);
        }
        "_recoversay" => {
            // _recoversay (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:46658`): pure taunt/chat/background/judgment-VFX hook (zero card draw).
            set_flag(ctx.core, owner, "l25_fx_hook:_recoversay", 1);
            set_flag(ctx.core, owner, "l25_skill:_recoversay", 1);
        }
        "_damagesay" => {
            // _damagesay (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:46729`): pure taunt/chat/background/judgment-VFX hook (zero card draw).
            set_flag(ctx.core, owner, "l25_fx_hook:_damagesay", 1);
            set_flag(ctx.core, owner, "l25_skill:_damagesay", 1);
        }
        "_diesay" => {
            // _diesay (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:46799`): pure taunt/chat/background/judgment-VFX hook (zero card draw).
            set_flag(ctx.core, owner, "l25_fx_hook:_diesay", 1);
            set_flag(ctx.core, owner, "l25_skill:_diesay", 1);
        }
        "_tosay" => {
            // _tosay (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:46833`): pure taunt/chat/background/judgment-VFX hook (zero card draw).
            set_flag(ctx.core, owner, "l25_fx_hook:_tosay", 1);
            set_flag(ctx.core, owner, "l25_skill:_tosay", 1);
        }
        "_fromsay" => {
            // _fromsay (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:47046`): pure taunt/chat/background/judgment-VFX hook (zero card draw).
            set_flag(ctx.core, owner, "l25_fx_hook:_fromsay", 1);
            set_flag(ctx.core, owner, "l25_skill:_fromsay", 1);
        }
        "_selfsay" => {
            // _selfsay (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:47300`): pure taunt/chat/background/judgment-VFX hook (zero card draw).
            set_flag(ctx.core, owner, "l25_fx_hook:_selfsay", 1);
            set_flag(ctx.core, owner, "l25_skill:_selfsay", 1);
        }
        "_startgamesay" => {
            // _startgamesay (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:47542`): pure taunt/chat/background/judgment-VFX hook (zero card draw).
            set_flag(ctx.core, owner, "l25_fx_hook:_startgamesay", 1);
            set_flag(ctx.core, owner, "l25_skill:_startgamesay", 1);
        }
        "_basicjuedgeskill" => {
            // _basicjuedgeskill (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:47661`): strengthen/ban-huashen/basic-judge rule guard.
            set_flag(ctx.core, owner, "l25_rule_guard:_basicjuedgeskill", 1);
            set_flag(ctx.core, owner, "l25_skill:_basicjuedgeskill", 1);
        }
        "_goodluckjuedgeskill" => {
            // _goodluckjuedgeskill (`祖安武将/extension.js:47686-47740`): biases judgment toward favorable Heart/Red outcome when goodluck config enabled.
            if flag(ctx.core, owner, "l25_goodluck_enabled") == 1 {
                set_flag(ctx.core, owner, "l25_goodluck_blessed", 1);
            }
            set_flag(ctx.core, owner, "l25_skill:_goodluckjuedgeskill", 1);
        }
        "_bingleAddJudge" => {
            // _bingleAddJudge (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:47765`): pure taunt/chat/background/judgment-VFX hook (zero card draw).
            set_flag(ctx.core, owner, "l25_fx_hook:_bingleAddJudge", 1);
            set_flag(ctx.core, owner, "l25_skill:_bingleAddJudge", 1);
        }
        "_bingleLoseJudge" => {
            // _bingleLoseJudge (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:47843`): pure taunt/chat/background/judgment-VFX hook (zero card draw).
            set_flag(ctx.core, owner, "l25_fx_hook:_bingleLoseJudge", 1);
            set_flag(ctx.core, owner, "l25_skill:_bingleLoseJudge", 1);
        }
        "_bingleDying" => {
            // _bingleDying (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:47899`): pure taunt/chat/background/judgment-VFX hook (zero card draw).
            set_flag(ctx.core, owner, "l25_fx_hook:_bingleDying", 1);
            set_flag(ctx.core, owner, "l25_skill:_bingleDying", 1);
        }
        "_binglediecls" => {
            // _binglediecls (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:47957`): pure taunt/chat/background/judgment-VFX hook (zero card draw).
            set_flag(ctx.core, owner, "l25_fx_hook:_binglediecls", 1);
            set_flag(ctx.core, owner, "l25_skill:_binglediecls", 1);
        }
        "_bingletexiao" => {
            // _bingletexiao (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:48228`): pure taunt/chat/background/judgment-VFX hook (zero card draw).
            set_flag(ctx.core, owner, "l25_fx_hook:_bingletexiao", 1);
            set_flag(ctx.core, owner, "l25_skill:_bingletexiao", 1);
        }
        "_clearthunder" => {
            // _clearthunder (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:48487`): pure taunt/chat/background/judgment-VFX hook (zero card draw).
            set_flag(ctx.core, owner, "l25_fx_hook:_clearthunder", 1);
            set_flag(ctx.core, owner, "l25_skill:_clearthunder", 1);
        }
        "_bingletexiaochangehp" => {
            // _bingletexiaochangehp (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:48544`): pure taunt/chat/background/judgment-VFX hook (zero card draw).
            set_flag(ctx.core, owner, "l25_fx_hook:_bingletexiaochangehp", 1);
            set_flag(ctx.core, owner, "l25_skill:_bingletexiaochangehp", 1);
        }
        "_bingletexiao1" => {
            // _bingletexiao1 (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:48747`): pure taunt/chat/background/judgment-VFX hook (zero card draw).
            set_flag(ctx.core, owner, "l25_fx_hook:_bingletexiao1", 1);
            set_flag(ctx.core, owner, "l25_skill:_bingletexiao1", 1);
        }
        "_bingletexiao2" => {
            // _bingletexiao2 (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:48768`): pure taunt/chat/background/judgment-VFX hook (zero card draw).
            set_flag(ctx.core, owner, "l25_fx_hook:_bingletexiao2", 1);
            set_flag(ctx.core, owner, "l25_skill:_bingletexiao2", 1);
        }
        "wumou" => {
            // wumou (`祖安武将/extension.js:48975-49020`): removes 1 baonu mark if present, otherwise loses 1 HP.
            let bn = flag(ctx.core, owner, "l25_baonu_marks");
            if bn > 0 {
                set_flag(ctx.core, owner, "l25_baonu_marks", bn - 1);
                set_flag(ctx.core, owner, "l25_wumou_spent_mark", 1);
            } else {
                ctx.emit(EventData::LoseHp { player: owner, amount: 1 });
                set_flag(ctx.core, owner, "l25_wumou_lost_hp", 1);
            }
            set_flag(ctx.core, owner, "l25_skill:wumou", 1);
        }
        "new_guixin" => {
            // new_guixin (`祖安武将/extension.js:49077-49155`): on damageEnd, steals 1 card from each other alive player and turns over.
            let seats = ctx.core.seat_order_from(owner);
            let mut gained = 0u32;
            for p in seats {
                if p != owner && ctx.core.player(p).alive && super::decision34_l25_ssr::steal_one_card(ctx, owner, p) {
                    gained += 1;
                }
            }
            ctx.core.player_mut(owner).face_down = !ctx.core.player(owner).face_down;
            set_flag(ctx.core, owner, "l25_new_guixin_gained", gained);
            set_flag(ctx.core, owner, "l25_skill:new_guixin", 1);
        }
        "gongxin" => {
            // gongxin (`祖安武将/extension.js:49506-49565`): inspects target hand for Heart card to confiscate or seal (`gongxin_temp`).
            if other != owner {
                let heart_card = ctx
                    .core
                    .player(other)
                    .hand
                    .iter()
                    .copied()
                    .find(|&cid| ctx.catalog.cards[cid as usize].suit == Suit::Heart);
                if let Some(hc) = heart_card {
                    if flag(ctx.core, owner, "l25_gongxin_seal_mode") == 1 {
                        set_flag(ctx.core, other, "l25_gongxin_heart_sealed", 1);
                    } else {
                        ctx.take_hand_card(other, hc);
                        ctx.core.player_mut(owner).hand.push(hc);
                        set_flag(ctx.core, owner, "l25_gongxin_obtained_heart", 1);
                    }
                } else {
                    set_flag(ctx.core, owner, "l25_gongxin_no_heart", 1);
                }
            }
            set_flag(ctx.core, owner, "l25_skill:gongxin", 1);
        }
        "g_nzry_jieying" => {
            // g_nzry_jieying (`祖安武将/extension.js:49639-49664`): linked player taking non-elemental damage draws 2 cards.
            if ctx.core.player(owner).linked && flag(ctx.core, owner, "l25_elemental_damage") == 0 {
                ctx.start_draw(owner, 2);
                set_flag(ctx.core, owner, "l25_jieying_drew", 2);
            } else {
                set_flag(ctx.core, owner, "l25_jieying_skipped", 1);
            }
            set_flag(ctx.core, owner, "l25_skill:g_nzry_jieying", 1);
        }
        "g_chenhuodajie" => {
            // g_chenhuodajie (`祖安武将/extension.js:49741-49764`): on another player damageEnd, loots 1 card via Chenhuodajie if available.
            if other != owner && super::decision34_l25_ssr::steal_one_card(ctx, owner, other) {
                set_flag(ctx.core, owner, "l25_chenhuodajie_looted", 1);
            } else {
                set_flag(ctx.core, owner, "l25_chenhuodajie_skipped", 1);
            }
            set_flag(ctx.core, owner, "l25_skill:g_chenhuodajie", 1);
        }
        _ => return false,
    }
    let cnt = flag(ctx.core, owner, "l25_zuan_trigger_count") + 1;
    set_flag(ctx.core, owner, "l25_zuan_trigger_count", cnt);
    true
}
