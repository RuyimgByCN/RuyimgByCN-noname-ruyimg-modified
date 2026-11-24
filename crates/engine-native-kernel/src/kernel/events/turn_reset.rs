//! 新回合玩家的回合状态重置（自 `events/turn.rs` 按职责拆分）。

use super::*;

/// 新回合玩家的回合重置（K25：原 `begin_judge` 的 `!turn_opened` 门整体
/// 前移；`turn_opened` 置真保留字段语义）。含红莲回合开始效果（原判定
/// 开始执行——时点前移属 A2 审计 K25 项，行为面等价：仍先于回合开始窗
/// 与准备阶段）。
pub(crate) fn reset_new_turn_state(ctx: &mut Ctx<'_>, player: PlayerId) {
    if ctx.core.turn.turn_opened {
        return;
    }
    ctx.core.turn.turn_opened = true;
    ctx.core.vm_aux.flags.retain(|(_, k), _| {
        !k.starts_with("dddzhilian_sha_d:")
            && k != "dddtongyu_hand_locked_stamp"
            && k != "turn_use_resp_mask"
            && k != "turn_use_resp_stamp"
            && k != "jiu_active_offturn"
            && k != "jiu_extra_stacks"
            && k != "dddjijian_jiu_used"
            && k != "dddfushi_resp_eff_kind"
            && k != "dddlianer_rank_cap"
            && k != "dddyulv_turn_cnt"
            && k != "dddxuanlun_choose"
            && !k.starts_with("u01_zhuanshe_used:")
            && !k.starts_with("u01_cj_len:")
            && !k.starts_with("u01_cj_card:")
            && !k.starts_with("dddzhuanshe_given:")
            && k != "u01_turn_trick_used_stamp"
            && !matches!(
                k.as_str(),
                "l11_lianpo_kills"
                    | "l11_shenxian_used"
                    | "l11_turn_dmg_dealt"
                    | "l11_cur_card_turn"
                    | "l11_cur_card_play"
                    | "l11_qiangwu_active"
                    | "l11_qiangwu_rank"
                    | "l11_xiongluan1_active"
                    | "l11_xiongluan2_hand_ban"
                    | "l11_fulin_phase_discard"
                    | "nzry_longnu_fire_turn"
                    | "nzry_longnu_thunder_turn"
            )
            && !k.starts_with("l11_langmie_play_cat:")
            && !k.starts_with("l11_xiongluan2_from:")
            && !k.starts_with("l11_fulin_tag:")
            && !k.starts_with("l11_gz_")
    });
    ctx.core.vm_aux.once.retain(|(_, _, scope)| *scope != 4);
    ctx.reset_turn_limits(player);
    // 本回合已用牌数（swd 灵武批 120，规则 v307）：回合开始清零
    ctx.core.turn.cards_used_turn = 0;
    // 回合伤害统计清零（U05 xianjian 明绣，规则 v160）
    ctx.core.player_mut(player).turn_damage = 0;
    // 忘忧戳清零（U05 xianjian 紫萱，规则 v162；登记差异：保留期延长）
    ctx.core.player_mut(player).wangyou3_round = 0;
    // 摄灵回合次数清零（U05 xianjian 南宫煌，规则 v168）
    ctx.core.turn.sheling_used = 0;
    // 射影回合清零（U05 xianjian 苏媚，规则 v175）
    ctx.core.turn.sheying_used = false;
    // 张目回合清零（U05 xianjian 南宫煌，规则 v181）
    ctx.core.turn.zhangmu_used = false;
    // 古剑奇谭 黑龙鳞片（card/gujian/skill.js:456-467）：持续至使用者下回合开始前清除
    ctx.core
        .vm_aux
        .flags
        .remove(&(player, "u05_mod:gj_heilonglinpian".to_string()));
    ctx.core
        .vm_aux
        .flags
        .remove(&(player, "u05_mod:heilonglinpian".to_string()));
    // 红莲（Boss 毕方，规则 v228）：回合开始——顺序获牌堆前缀红牌至多
    // 3 张，并对座位序前至多 3 名其他角色各造成 1 点火伤
    if let Some(hdef) = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_HONGLIANX)
    {
        if ctx.core.has_skill(player, hdef) {
            let mut gained = 0usize;
            while gained < 3 && !ctx.core.deck.is_empty() {
                let c = ctx.core.deck[ctx.core.deck.len() - 1];
                if matches!(
                    ctx.catalog.cards[c as usize].suit,
                    crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                ) {
                    ctx.core.deck.pop();
                    ctx.core.player_mut(player).hand.push(c);
                    gained += 1;
                } else {
                    break;
                }
            }
            if gained > 0 {
                ctx.log(LogEvent::CardsDrawn {
                    player,
                    cards: ctx
                        .core
                        .player(player)
                        .hand
                        .iter()
                        .rev()
                        .take(gained)
                        .copied()
                        .collect::<Vec<_>>(),
                });
            }
            let targets: Vec<PlayerId> = ctx
                .core
                .seat_order_from(player)
                .into_iter()
                .filter(|t| *t != player && ctx.core.player(*t).alive)
                .take(3)
                .collect();
            for t in targets {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(player),
                    t,
                    1,
                    crate::event::DamageNature::Fire,
                    None,
                )));
            }
        }
    }
    // K02/K03/K04：三个「至 phaseBegin」到期解除（先于回合开始窗）。
    if ctx.core.player(player).qianggu_active {
        ctx.core.player_mut(player).qianggu_active = false;
    }
    if ctx.core.player(player).yinshen_active {
        ctx.core.player_mut(player).yinshen_active = false;
    }
    if ctx.core.player(player).chaofeng_active {
        ctx.core.player_mut(player).chaofeng_active = false;
    }
    ctx.core
        .vm_aux
        .flags
        .remove(&(player, "u05_tianxianjiu_used_turn".to_string()));
    let had_begin = ctx
        .core
        .vm_aux
        .flags
        .remove(&(player, "u05_yinmi_until_phase_begin".to_string()))
        .is_some();
    if had_begin
        && ctx
            .core
            .vm_aux
            .flags
            .get(&(player, "u05_yinmi_until_phase_after".to_string()))
            .copied()
            .unwrap_or(0)
            == 0
    {
        ctx.core.player_mut(player).yinmi_active = false;
    }
    // U05-5b `fayin` (`character/gwent/skill.js:3024`: goMad until target's phaseBegin):
    if ctx
        .core
        .vm_aux
        .flags
        .remove(&(player, "gw_fayin_mad".to_string()))
        .is_some()
    {
        ctx.core.player_mut(player).boss_mad = false;
    }
    // U05-5b `shuangxi` / `shewu_dist` / `gw_use2_cnt:*` 回合开始清零
    ctx.core
        .vm_aux
        .flags
        .remove(&(player, "gw_shuangxi_turn_damaged".to_string()));
    ctx.core
        .vm_aux
        .flags
        .remove(&(player, "u05_mod:shewu_dist".to_string()));
    ctx.core
        .vm_aux
        .flags
        .remove(&(player, "gw_use2_cnt:gwliaotian".to_string()));
    ctx.core
        .vm_aux
        .flags
        .remove(&(player, "gw_use2_cnt:gwxiaoshou".to_string()));
    // K09：屏障标志转计数（上游 pingzhang_count phaseBegin silent）。
    {
        let used = u8::from(ctx.core.player(player).pingzhang_self_used)
            + u8::from(ctx.core.player(player).pingzhang_other_used);
        let p = ctx.core.player_mut(player);
        p.pingzhang_count = used;
        p.pingzhang_self_used = false;
        p.pingzhang_other_used = false;
    }
    // K30：形态轮换（Boss 年兽系，规则 v249）——上游 phaseBefore 换将，
    // 须先于 TurnBegin 使新形态的回合开始技生效。
    if let Some(nbdef) = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_NBIANSHEN)
    {
        if ctx.core.has_skill(player, nbdef) {
            let armed = ctx.core.player(player).nbianshen_toggle;
            if armed {
                let base = (ctx.catalog.general_count()
                    - crate::vocabulary::generals::BOSS_GENERALS.len())
                    as u16;
                let cur = ctx.core.player(player).general_slots[0];
                let other = (base..ctx.catalog.general_count() as u16).find(|&d| {
                    Some(d) != cur
                        && ctx
                            .catalog
                            .general_meta(d)
                            .map(|m| m.2.contains(&crate::vocabulary::names::BOSS_NBIANSHEN))
                            .unwrap_or(false)
                });
                if let (Some(other), Some(_cur)) = (other, cur) {
                    let removed: Vec<crate::ids::SkillDefId> = ctx
                        .core
                        .skills
                        .keys()
                        .filter(|(o, _)| *o == player)
                        .map(|(_, d)| *d)
                        .collect();
                    for d in removed {
                        ctx.core.skills.remove(&(player, d));
                    }
                    ctx.core.player_mut(player).general_slots[0] = Some(other);
                    crate::state::apply_general_assembly(ctx.core, ctx.catalog, player, 0);
                    ctx.log(LogEvent::SkillToggled {
                        player,
                        skill: "形态轮换",
                        enabled: true,
                    });
                }
            }
            ctx.core.player_mut(player).nbianshen_toggle = true;
        }
    }
}
