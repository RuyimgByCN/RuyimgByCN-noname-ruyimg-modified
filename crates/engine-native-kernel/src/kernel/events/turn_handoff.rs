//! 回合交接 `turn_handoff` 与 `maybe_abort_ywuhun`（自 `events/turn.rs` 按职责拆分）。

use super::super::*;
use super::turn_reset::reset_new_turn_state;

/// R05（K21／K25）：回合交接——在整回合结束窗 phaseAfter 段的 Commit 中
/// 执行：轮次推进、换人、回合级存量清零、**新回合玩家的回合重置**
/// （`begin_judge` 原 `!turn_opened` 门整体前移至此，使回合开始窗与准备
/// 阶段读到本回合的新值），最后发 TurnBegin（自动接准备阶段→判定）。
pub(crate) fn turn_handoff(ctx: &mut Ctx<'_>, prev: PlayerId, next: PlayerId) {
    // R05-F（§6 兜底位）：任何已死亡角色的 Limit/Combs/Current 语义槽与
    // helper 实例在回合交接时清理（真实死亡经 DeathMachine 前置清理；此
    // 处覆盖无死亡帧的跳过路径——Next 保留）。
    for role in ["limit", "combs", "current"] {
        for def in (ctx.catalog.hooks.r05_role_defs)(ctx.catalog, role) {
            for seat in 0..ctx.core.players.len() as u8 {
                if !ctx.core.player(seat).alive {
                    ctx.core.vm_aux.marks.remove(&(seat, def as u32));
                    ctx.core.skills.remove(&(seat, def));
                }
            }
        }
    }
    // K18：灵武在 phaseAfter 段插入的额外出牌阶段——回合不交接，本人
    // 直接再入出牌阶段（上游 insertPhase；extra_phase_lingwu 防递归，
    // 收束后队列空→正常交接）。
    if !ctx.core.turn.extra_phases.is_empty() && ctx.core.turn.current == prev {
        let code = ctx.core.turn.extra_phases.remove(0);
        let phase = match code {
            0 => Phase::Judge,
            1 => Phase::Draw,
            2 => Phase::Play,
            3 => Phase::Discard,
            _ => Phase::End,
        };
        ctx.core.turn.phase = phase;
        ctx.emit(EventData::Phase {
            player: prev,
            phase,
            begin: true,
        });
        return;
    }
    ctx.expire_temporary_skills(prev);
    if ctx
        .core
        .vm_aux
        .flags
        .remove(&(prev, "dddqiahua_temp_xunxun".to_string()))
        .is_some()
    {
        if let Some(xdef) = ctx.catalog.skill_by_name("dddxunxun") {
            ctx.core.skills.remove(&(prev, xdef));
        }
    }
    ctx.core.vm_aux.flags.retain(|(p, k), _| {
        !k.starts_with("turn_discard_card:")
            && k != "turn_change_hp"
            && k != "turn_dmg_sources_mask"
            && k != "play_phase_seq"
            && k != "play_phase_uses"
            && k != "dddxuyu_pending_fid"
            && k != "dddshijian_extra_target"
            && k != "dddquche_effect_marks"
            && k != "dddfusi_allowed"
            && k != "dddfusi_refused"
            && k != "dddfusi_window_refused"
            && k != "ddddongcha_active_turn"
            && k != "dddchashi_armed"
            && k != "dddchashi_target"
            && k != "dddchashi_suit"
            && k != "dddchashi_kind"
            && !k.starts_with("dddxuyu_old_armed:")
            && !k.starts_with("dddshijian_old_armed:")
            && !matches!(
                k.as_str(),
                "ow_turn_kill_count"
                    | "ow_juji2_target"
                    | "ow_liangou2_from"
                    | "ow_tuijin_target"
                    | "l11_lianpo_kills"
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
            && !(*p == prev
                && matches!(
                    k.as_str(),
                    "ow_shanxian_target" | "ow_shanxian_pending_turnover"
                ))
    });
    // U05-3 ow 回合结束清理：狙击2/狙击3/链钩2/飞跳2/侦查2/暴风矢4/推进2/闪光2 临时技
    for sk in [
        "juji2",
        "juji3",
        "liangou2",
        "feitiao2",
        "zhencha2",
        "bfengshi4",
        "tuijin2",
        "shanguang2",
    ] {
        if let Some(def) = ctx.catalog.skill_by_name(sk) {
            for s in 0..ctx.core.players.len() as PlayerId {
                ctx.core.skills.remove(&(s, def));
            }
        }
    }
    if ctx.catalog.vm_card_history {
        ctx.core
            .vm_aux
            .flags
            .retain(|(_, k), _| k != "sha_played_this_turn" && k != "damaged_this_turn");
    }
    let lianjing_next = {
        let len = ctx
            .core
            .vm_aux
            .flags
            .get(&(0, "gj_lianjing_len".to_string()))
            .copied()
            .unwrap_or(0);
        let mut idx = ctx
            .core
            .vm_aux
            .flags
            .get(&(0, "gj_lianjing_idx".to_string()))
            .copied()
            .unwrap_or(0);
        let mut found = None;
        while idx < len {
            let s = ctx
                .core
                .vm_aux
                .flags
                .get(&(0, format!("gj_lianjing_q:{idx}")))
                .copied()
                .unwrap_or(0) as PlayerId;
            idx += 1;
            if (s as usize) < ctx.core.players.len() && ctx.core.player(s).alive {
                ctx.core
                    .vm_aux
                    .flags
                    .insert((0, "gj_lianjing_idx".to_string()), idx);
                found = Some(s);
                break;
            }
        }
        if found.is_none() && len > 0 {
            ctx.core.vm_aux.flags.remove(&(0, "gj_lianjing_len".to_string()));
            ctx.core.vm_aux.flags.remove(&(0, "gj_lianjing_idx".to_string()));
            for i in 0..len {
                ctx.core
                    .vm_aux
                    .flags
                    .remove(&(0, format!("gj_lianjing_q:{i}")));
            }
            for s in 0..ctx.core.players.len() as PlayerId {
                ctx.core
                    .vm_aux
                    .flags
                    .remove(&(s, "u05_mod:gj_lianjing_out_of_play".to_string()));
            }
        }
        found
    };
    let queued_extra = lianjing_next.or_else(|| {
        (0..ctx.core.players.len() as PlayerId).find(|&s| {
            ctx.core.player(s).alive
                && ctx
                    .core
                    .vm_aux
                    .flags
                    .get(&(s, "extra_turn_insert_queued".to_string()))
                    .copied()
                    .unwrap_or(0)
                    > 0
        })
    });
    let has_queued_extra = queued_extra.is_some();
    let (next, suppress_round_bump, returning_from_shanxian) = if let Some(es) = queued_extra {
        ctx.core
            .vm_aux
            .flags
            .remove(&(es, "extra_turn_insert_queued".to_string()));
        if let Some(st) = ctx
            .core
            .vm_aux
            .flags
            .remove(&(es, "u05_msilian_turn_pending".to_string()))
        {
            ctx.core
                .vm_aux
                .flags
                .insert((es, "u05_msilian_extra_active".to_string()), st);
        }
        if !ctx
            .core
            .vm_aux
            .flags
            .contains_key(&(0, "ow_shanxian_resume_target".to_string()))
        {
            if !ctx
                .core
                .vm_aux
                .flags
                .contains_key(&(0, "extra_turn_resume_next".to_string()))
            {
                ctx.core
                    .vm_aux
                    .flags
                    .insert((0, "extra_turn_resume_next".to_string()), u32::from(next) + 1);
                ctx.core
                    .vm_aux
                    .flags
                    .insert((0, "extra_turn_resume_prev".to_string()), u32::from(prev) + 1);
            }
            ctx.core
                .vm_aux
                .flags
                .insert((0, "extra_turn_in_progress".to_string()), 1);
        }
        (es, true, false)
    } else if ctx
        .core
        .vm_aux
        .flags
        .remove(&(0, "extra_turn_in_progress".to_string()))
        .is_some()
    {
        ctx.core
            .vm_aux
            .flags
            .remove(&(prev, "hs_ylyuchu_extra_draw_minus".to_string()));
        let saved_next = ctx
            .core
            .vm_aux
            .flags
            .remove(&(0, "extra_turn_resume_next".to_string()))
            .unwrap_or(0);
        let saved_prev = ctx
            .core
            .vm_aux
            .flags
            .remove(&(0, "extra_turn_resume_prev".to_string()))
            .unwrap_or(0);
        let resume_p = saved_prev.saturating_sub(1) as PlayerId;
        let candidate_n = saved_next.saturating_sub(1) as PlayerId;
        let actual_n = if (candidate_n as usize) < ctx.core.players.len()
            && ctx.core.player(candidate_n).alive
        {
            candidate_n
        } else {
            ctx.core.next_alive(resume_p)
        };
        if actual_n <= resume_p {
            ctx.core
                .vm_aux
                .flags
                .insert((0, "extra_turn_force_round_bump".to_string()), 1);
        }
        (actual_n, actual_n > resume_p, false)
    } else if let Some(saved_res) = ctx
        .core
        .vm_aux
        .flags
        .remove(&(0, "ow_shanxian_resume_target".to_string()))
    {
        let cand = saved_res.saturating_sub(1) as PlayerId;
        let actual_n = if (cand as usize) < ctx.core.players.len() && ctx.core.player(cand).alive {
            cand
        } else {
            ctx.core.next_alive(cand)
        };
        if actual_n != cand && actual_n <= cand {
            ctx.core
                .vm_aux
                .flags
                .insert((0, "extra_turn_force_round_bump".to_string()), 1);
        }
        (actual_n, actual_n == cand || actual_n > cand, true)
    } else {
        (next, false, false)
    };
    let force_round_bump = !has_queued_extra
        && ctx
            .core
            .vm_aux
            .flags
            .remove(&(0, "extra_turn_force_round_bump".to_string()))
            .is_some();
    if (ctx.catalog.hooks.on_turn_before_skip)(ctx, next) {
        if force_round_bump {
            ctx.core
                .vm_aux
                .flags
                .insert((0, "extra_turn_force_round_bump".to_string()), 1);
        }
        ctx.emit(EventData::TurnEnded { player: next });
        return;
    }
    if ctx.core.player(next).face_down
        && !returning_from_shanxian
        && (ctx.catalog.hooks.before_facedown_turn_skip)(ctx, next, Some(prev))
    {
        if force_round_bump || (!suppress_round_bump && next <= prev) {
            ctx.core
                .vm_aux
                .flags
                .insert((0, "extra_turn_force_round_bump".to_string()), 1);
        }
        return;
    }
    let should_round_start = force_round_bump || (!suppress_round_bump && next <= prev);
    if should_round_start {
        if ctx.catalog.vm_card_history {
            let seats = ctx.core.players.len() as PlayerId;
            for p in 0..seats {
                if !ctx.core.player(p).alive {
                    continue;
                }
                if ctx
                    .catalog
                    .skill_by_name("oljingxian")
                    .is_some_and(|def| ctx.core.has_skill(p, def))
                {
                    let idx = ctx
                        .core
                        .vm_aux
                        .flags
                        .get(&(p, "olfangma_chitu_index".to_string()))
                        .copied()
                        .unwrap_or(0) as usize;
                    if idx >= ctx.core.player(p).hand.len() {
                        ctx.core
                            .vm_aux
                            .flags
                            .insert((p, "oljingxian_at_rightmost".to_string()), 1);
                        ctx.core
                            .vm_aux
                            .flags
                            .insert((p, "oljingxian_extra_turn_granted".to_string()), 1);
                    }
                }
                if ctx
                    .catalog
                    .skill_by_name("olfangma")
                    .is_some_and(|def| ctx.core.has_skill(p, def))
                {
                    ctx.core
                        .vm_aux
                        .flags
                        .remove(&(p, "olfangma_chitu_index".to_string()));
                    ctx.core
                        .vm_aux
                        .flags
                        .remove(&(p, "olfangma_sha_unlimited".to_string()));
                    if !ctx.core.player(p).hand.is_empty() {
                        ctx.core
                            .vm_aux
                            .flags
                            .insert((p, "olfangma_sha_double_shan".to_string()), 1);
                    } else {
                        ctx.core
                            .vm_aux
                            .flags
                            .remove(&(p, "olfangma_sha_double_shan".to_string()));
                    }
                }
            }
        }
        ctx.core.turn.round += 1;
        ctx.core.vm_aux.clear_round();
    }
    // R05（K10）：翻面整回合跳过在回合开始窗之前（上游 phaseBefore 检测）
    // ——翻面者不发 TurnBegin／不进任何阶段（含准备阶段）；`roundStart`
    // 仍于新轮开始且早于 `TurnOverBefore` 结算（LIFO 后压先结）。
    if ctx.core.player(next).face_down {
        ctx.core.turn.current = next;
        ctx.core.player_mut(next).jileng_active = false;
        ctx.core.player_mut(next).boss_mad = false;
        ctx.emit(EventData::TurnEnded { player: next });
        ctx.emit(EventData::TurnOverBefore {
            player: next,
            cancelled: false,
        });
        if should_round_start {
            (ctx.catalog.hooks.on_round_start)(ctx);
        }
        return;
    }
    ctx.core.turn.current = next;
    ctx.core.turn.phase = Phase::Judge;
    ctx.core.turn.sha_used_play_phase = false;
    ctx.core.turn.jiu_used = false;
    ctx.core.turn.jiu_active = false;
    ctx.core.turn.land_used = false;
    ctx.core.turn.land_sha_used = false;
    ctx.core.turn.turn_opened = false;
    // 回合级存量清零（v128 修复 v126 遗留：TurnState 换回合为原地
    // 改字段而非重建——乐动三角/链钩必须显式复位）
    ctx.core.turn.yuedong_num = 0;
    ctx.core.turn.yuedong_eff = 0;
    ctx.core.turn.yuedong_recover = false;
    ctx.core.turn.liangou_hook = None;
    ctx.core.turn.liangou_bonus_used = false;
    ctx.core.turn.feitiao_target = None;
    ctx.core.turn.fanling_used = false;
    ctx.core.turn.xjyufeng_used = 0;
    ctx.core.turn.diewu_draw_used = false;
    ctx.core.turn.lueying_used = false;
    // 冥顽锁定集（Boss 梼杌，规则 v219）：回合更替清零（上游 phaseAfter
    // clear 同窗）
    ctx.core.turn.minwan_targets.clear();
    ctx.core.turn.yanyu_count = 0;
    ctx.core.turn.txianqu_used = false;
    ctx.core.turn.jtjubao_used = false;
    ctx.core.turn.kangyin2 = None;
    ctx.core.turn.ciqiu_dying.clear();
    for seat in [prev, next] {
        ctx.core
            .vm_aux
            .flags
            .remove(&(seat, "q4_play_sha_count".to_string()));
        ctx.core
            .vm_aux
            .flags
            .remove(&(seat, "sha_used_count".to_string()));
        if ctx
            .core
            .vm_aux
            .flags
            .remove(&(seat, "q4_minsi2_active".to_string()))
            .is_some()
        {
            ctx.core
                .vm_aux
                .flags
                .retain(|(p, k), _| !(*p == seat && k.starts_with("q4_minsi2_card:")));
        }
    }
    reset_new_turn_state(ctx, next);
    ctx.emit(EventData::TurnBegin { player: next });
    if should_round_start {
        (ctx.catalog.hooks.on_round_start)(ctx);
    }
}



/// 雾魂（U05 xianjian 龙幽，damageEnd 立即中止回合）：清栈与挂起机，直接压入
/// `Phase::End { begin: false }`，使剩余伤害目标、铁索传导及后置阶段不再继续。
pub(crate) fn maybe_abort_ywuhun(
    core: &mut crate::state::GameStateCore,
    res: &mut crate::resolution::Resolution,
) -> bool {
    if core.outcome.is_some() || res.fault.is_some() {
        return false;
    }
    let cur = core.turn.current;
    if core
        .vm_aux
        .flags
        .remove(&(cur, "xj_ywuhun_abort".to_string()))
        .is_none()
    {
        return false;
    }
    res.parks.clear();
    res.open_choice = None;
    res.answer = None;
    res.stack.clear();
    core.turn.phase = crate::event::Phase::End;
    crate::resolution::push_frame(
        res,
        crate::event::EventData::Phase {
            player: cur,
            phase: crate::event::Phase::End,
            begin: false,
        },
    );
    true
}
