//! 栈帧弹栈后清理与用牌后宿主钩子派发（自 `resolution.rs` 按职责拆出）。

use super::*;

/// 记录本次 `Recover` 事件的施救/引发来源（瞬态 `l11_recover_source:{fid}`，仅供 L11 修罗的怜悯消费）。
pub(crate) fn note_recover_source(
    core: &mut GameStateCore,
    res: &Resolution,
    player: PlayerId,
    recover_fid: crate::ids::EventId,
) {
    let src = res.active_actor.or_else(|| {
        res.stack.iter().rev().skip(1).find_map(|f| match &f.data {
            EventData::SkillInvocation(inv) => Some(inv.owner),
            EventData::CardUsed(u) => Some(u.player),
            EventData::Damage(d) => d.source,
            _ => None,
        })
    });
    if let Some(s) = src {
        core.vm_aux.flags.insert(
            (player, format!("l11_recover_source:{recover_fid}")),
            u32::from(s) + 1,
        );
    }
}

pub(super) fn on_frame_popped(
    popped: &Frame,
    core: &mut GameStateCore,
    res: &mut Resolution,
    catalog: &crate::catalog::Catalog,
    out: &mut Vec<LogEvent>,
) {
    if let EventData::CardsDiscarded { player, cards } = &popped.data {
        if catalog.vm_card_history || catalog.skill_by_name("txhj_xuanwuSkill2").is_some() {
            for &c in cards {
                core.vm_aux
                    .flags
                    .remove(&(*player, format!("l11_discard_non_hand:{c}")));
            }
        }
        if catalog.vm_card_history {
            let fid = popped.id;
            for &c in cards {
                core.vm_aux
                    .flags
                    .remove(&(*player, format!("l11_hand_dismantle_card:{c}")));
                core.vm_aux
                    .flags
                    .remove(&(*player, format!("l11_loss_dedup:{fid}:{c}")));
            }
        }
    }
    if let EventData::CardsMoved { from, cards, .. } = &popped.data {
        if catalog.vm_card_history {
            let fid = popped.id;
            for &c in cards {
                core.vm_aux
                    .flags
                    .remove(&(*from, format!("l11_hand_snatch_card:{c}")));
                core.vm_aux
                    .flags
                    .remove(&(*from, format!("l11_loss_dedup:{fid}:{c}")));
            }
        }
    }
    if let EventData::TurnEnded { player } = &popped.data {
        if catalog.vm_card_history {
            core.vm_aux
                .flags
                .remove(&(*player, "l11_jianiang_stacks".to_string()));
        }
        if catalog.skill_by_name("txyinli").is_some() {
            for p in 0..core.players.len() as PlayerId {
                core.vm_aux.flags.remove(&(p, "l11_yinli_ts".to_string()));
                core.vm_aux
                    .flags
                    .retain(|(owner, k), _| !(*owner == p && k.starts_with("l11_yinli_eq:")));
            }
        }
    }
    if let EventData::Recover { player, .. } = &popped.data {
        if catalog.vm_card_history {
            core.vm_aux
                .flags
                .remove(&(*player, format!("l11_recover_source:{}", popped.id)));
        }
    }
    if let EventData::CardResponded { player, .. } = &popped.data {
        core.vm_aux
            .flags
            .remove(&(*player, "dddfushi_resp_eff_kind".to_string()));
    }
    if let EventData::Damage(d) = &popped.data {
        core.vm_aux
            .flags
            .remove(&(d.target, "l28_vdmg_kind".to_string()));
        for p in 0..core.players.len() as PlayerId {
            core.vm_aux
                .flags
                .remove(&(p, format!("mbcmjiushi_dmg:{}", popped.id)));
            core.vm_aux
                .flags
                .remove(&(p, format!("sha_damage_frame:{}", popped.id)));
        }
    }
    if super::is_post_commit_host_event(&popped.data) {
        let mut ctx = Ctx {
            core,
            res,
            catalog,
            out,
        };
        (catalog.hooks.on_host_event)(&mut ctx, &popped.data);
    }
    if let EventData::Equipped { player, card, .. } = &popped.data {
        core.vm_aux
            .flags
            .remove(&(*player, format!("play_equip_use:{}", popped.id)));
        core.vm_aux
            .flags
            .remove(&(*player, format!("dddanzhi_equip_use:{}", popped.id)));
        core.vm_aux
            .flags
            .remove(&(*player, format!("u01_equip_played:{}", popped.id)));
        core.vm_aux
            .flags
            .remove(&(*player, format!("chashi_equip_use:{}", popped.id)));
        core.vm_aux
            .flags
            .remove(&(*player, format!("wuxing_equip_use:{}", popped.id)));
        core.vm_aux
            .flags
            .remove(&(*player, format!("u05_play_equip_frame:{}", popped.id)));
        core.vm_aux
            .flags
            .remove(&(*player, format!("l11_equip_from_hand:{card}")));
        core.vm_aux
            .flags
            .remove(&(*player, format!("l11_move_equip_evt:{card}")));
        if let Some(fusi_owner) = core
            .vm_aux
            .flags
            .remove(&(*player, format!("dddfusi_used_owner_eq:{}", popped.id)))
        {
            core.vm_aux.flags.insert(
                (*player, "last_popped_card_use_fid".to_string()),
                popped.id,
            );
            core.vm_aux.flags.insert(
                (*player, format!("dddfusi_used_owner:{}", popped.id)),
                fusi_owner,
            );
            let synth = crate::event::CardUseData {
                card: Some(*card),
                player: *player,
                targets: vec![*player],
                nature: crate::event::DamageNature::Normal,
                use_as: None,
                cancelled: false,
            };
            let mut ctx = Ctx {
                core,
                res,
                catalog,
                out,
            };
            (catalog.hooks.on_card_use_after)(&mut ctx, popped.id, &synth);
            core.vm_aux
                .flags
                .remove(&(*player, "last_popped_card_use_fid".to_string()));
            core.vm_aux
                .flags
                .remove(&(*player, format!("dddfusi_used_owner:{}", popped.id)));
        }
    }
    if let EventData::OrdinaryJudge { owner, invocation } = &popped.data {
        core.vm_aux
            .flags
            .remove(&(*owner, format!("l11_resolved_ordinary_judge:{invocation}")));
    }
    if let EventData::CardUsed(u) = &popped.data {
        on_card_use_popped(popped.id, u, core, res, catalog, out);
    }
    if matches!(
        popped.kind,
        crate::event::EventKind::CardUsed
            | crate::event::EventKind::CardsDiscarded
            | crate::event::EventKind::CardResponded
            | crate::event::EventKind::OrdinaryJudge
            | crate::event::EventKind::Equipped
    ) && core
        .vm_aux
        .flags
        .get(&(0, "u01_yulv_q_len".to_string()))
        .copied()
        .unwrap_or(0)
        > 0
    {
        let actor = core.turn.current;
        let mut ctx = Ctx {
            core,
            res,
            catalog,
            out,
        };
        (catalog.hooks.on_cards_to_discard)(&mut ctx, actor, &[], false);
    }
}

fn on_card_use_popped(
    popped_id: EventId,
    u: &crate::event::CardUseData,
    core: &mut GameStateCore,
    res: &mut Resolution,
    catalog: &crate::catalog::Catalog,
    out: &mut Vec<LogEvent>,
) {
    core.vm_aux
        .flags
        .remove(&(u.player, format!("dddanzhi_targets:{popped_id}")));
    core.vm_aux
        .flags
        .remove(&(u.player, format!("dying_save_frame:{popped_id}")));
    let _ = crate::kernel::take_tao_extra_carriers(core, u.player, popped_id);
    core.vm_aux
        .flags
        .remove(&(u.player, format!("ow_card_use_suit_spade:{popped_id}")));
    core.vm_aux
        .flags
        .remove(&(u.player, format!("l11_carduse_lost:{popped_id}")));
    core.vm_aux
        .flags
        .remove(&(u.player, format!("sha_used_jiu:{popped_id}")));
    core.vm_aux
        .flags
        .remove(&(u.player, format!("sha_cike_done:{popped_id}")));
    core.vm_aux
        .flags
        .remove(&(u.player, format!("sha_tianxian_done:{popped_id}")));
    core.vm_aux
        .flags
        .remove(&(u.player, format!("sha_longxiang_done:{popped_id}")));
    core.vm_aux
        .flags
        .remove(&(u.player, format!("sha_yueren_done:{popped_id}")));
    let cid = u.card.unwrap_or(0);
    core.vm_aux
        .flags
        .remove(&(u.player, format!("qilin_fuchen_used:{cid}")));
    let taiji_prefix = format!("txhj_taiji_rem:{popped_id}:");
    core.vm_aux
        .flags
        .retain(|(_, k), _| !k.starts_with(&taiji_prefix));
    core.vm_aux
        .flags
        .remove(&(u.player, format!("sha_end_settled:{popped_id}")));
    core.vm_aux
        .flags
        .remove(&(u.player, format!("sha_pending_end:{popped_id}")));
    core.vm_aux
        .flags
        .remove(&(u.player, format!("sha_jiu_extra:{popped_id}")));
    let eff_kind = u
        .use_as
        .or_else(|| u.card.map(|c| catalog.resolved_card_kind(core, c)));
    if eff_kind == Some(crate::catalog::CardKind::Tao) {
        core.vm_aux
            .flags
            .remove(&(u.player, "vm_va_recover_bonus".to_string()));
    }
    if eff_kind == Some(crate::catalog::CardKind::Youdishenru) {
        core.vm_aux
            .flags
            .remove(&(u.player, "yd_pending_source".to_string()));
        core.vm_aux
            .flags
            .remove(&(u.player, "yd_pending_sha_frame".to_string()));
    }
    if u.card.is_none() && u.use_as == Some(crate::catalog::CardKind::Wuzhong) {
        core.vm_aux
            .flags
            .remove(&(u.player, "virtual_diaobing_use".to_string()));
    }
    for p in 0..core.players.len() as PlayerId {
        core.vm_aux
            .flags
            .remove(&(p, format!("shengke_nw:{popped_id}:{p}")));
        core.vm_aux
            .flags
            .remove(&(p, format!("monkey_asked:{popped_id}")));
        core.vm_aux
            .flags
            .remove(&(p, format!("sha_u01_shan_pending:{popped_id}")));
        core.vm_aux
            .flags
            .remove(&(p, format!("sha_u01_hujia_dodged:{popped_id}")));
    }
    for p in 0..core.players.len() as PlayerId {
        core.vm_aux.flags.remove(&(p, "l28_vdmg_kind".to_string()));
        core.vm_aux
            .flags
            .remove(&(p, "xinfu_lingren_vuln_from".to_string()));
        core.vm_aux
            .flags
            .remove(&(p, "xinfu_lingren_vuln_card".to_string()));
        core.vm_aux
            .flags
            .remove(&(p, "xinfu_lingren_vuln_target".to_string()));
        if let Some(val) = core
            .vm_aux
            .flags
            .remove(&(p, "shouye_pending_gain_card".to_string()))
        {
            if val > 0 {
                let cid = val - 1;
                let in_table = core.table.contains(&cid);
                let in_discard = core.discard.contains(&cid);
                if (in_table || in_discard) && core.player(p).alive {
                    core.table.retain(|&x| x != cid);
                    core.discard.retain(|&x| x != cid);
                    if !core.player(p).hand.contains(&cid) {
                        core.player_mut(p).hand.push(cid);
                    }
                }
            }
        }
        if core
            .vm_aux
            .flags
            .remove(&(p, format!("chixueqingfeng2_frame:{popped_id}")))
            .is_some()
        {
            let prev = core
                .vm_aux
                .flags
                .get(&(p, "chixueqingfeng2_count".to_string()))
                .copied()
                .unwrap_or(0);
            if prev <= 1 {
                core.vm_aux
                    .flags
                    .remove(&(p, "chixueqingfeng2_count".to_string()));
            } else {
                core.vm_aux
                    .flags
                    .insert((p, "chixueqingfeng2_count".to_string()), prev - 1);
            }
        }
    }
    if u.use_as.is_none()
        || u.card
            .is_some_and(|c| u.use_as == catalog.resolved_card_def(core, c).map(|d| d.kind))
    {
        if let Some(c) = u.card.filter(|id| core.discard.contains(id)) {
            let mut ctx = Ctx {
                core,
                res,
                catalog,
                out,
            };
            (catalog.hooks.on_cards_to_discard)(&mut ctx, u.player, &[c], true);
        }
    }
    if matches!(
        u.use_as,
        Some(crate::catalog::CardKind::Shan | crate::catalog::CardKind::Nullify)
    ) {
        if let Some(c) = u.card {
            if core.table.contains(&c) {
                core.table.retain(|&x| x != c);
                if !core.discard.contains(&c) {
                    core.discard.push(c);
                    if catalog.vm_card_history {
                        core.vm_aux
                            .flags
                            .insert((0, format!("turn_discard_card:{c}")), 1);
                    }
                }
            }
        }
    }
    if u.use_as == Some(crate::catalog::CardKind::Shan) {
        for p in 0..core.players.len() as PlayerId {
            core.vm_aux
                .flags
                .remove(&(p, "dddfusi_window_refused".to_string()));
        }
    }
    core.vm_aux.flags.insert(
        (u.player, "last_popped_card_use_fid".to_string()),
        popped_id,
    );
    {
        let mut ctx = Ctx {
            core,
            res,
            catalog,
            out,
        };
        (catalog.hooks.on_card_use_after)(&mut ctx, popped_id, u);
    }
    core.vm_aux
        .flags
        .remove(&(u.player, "last_popped_card_use_fid".to_string()));
    core.vm_aux
        .flags
        .remove(&(u.player, format!("dddxianxi_added:{popped_id}")));
    core.vm_aux
        .flags
        .remove(&(u.player, format!("dddxianxi_dmg:{popped_id}")));
    core.vm_aux
        .flags
        .remove(&(u.player, format!("dddfenji_dmg:{popped_id}")));
    core.vm_aux
        .flags
        .remove(&(u.player, format!("card_use_play_seq:{popped_id}")));
    core.vm_aux
        .flags
        .remove(&(u.player, format!("card_use_play_idx:{popped_id}")));
    core.vm_aux
        .flags
        .remove(&(u.player, format!("card_use_extra_effect:{popped_id}")));
    core.vm_aux
        .flags
        .remove(&(u.player, format!("dddfusi_used_owner:{popped_id}")));
    let dodge_prefix = format!("l11_sha_dodge_ok:{popped_id}:");
    core.vm_aux
        .flags
        .retain(|(_, k), _| !k.starts_with(&dodge_prefix));
}
