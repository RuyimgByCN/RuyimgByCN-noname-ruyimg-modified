//! U05-3 (`shaai-ow`) 引擎回调与子周期钩子。
//!
//! 覆盖：`gameStart`（机甲/重盾初始化）、`turnBegin`（局回合计数）、`shaBegin`（强固2/侵入/暴风矢/侦查）、
//! `drawBegin`（利用）、`useCardAfter`（暴风矢4/飞刃3）、`playerDied`、`on_skill_removed`。

use super::u05_ow::{
    alive_others, clear_bingqiang_by_owner, flag, has_skill_u05, ow_enabled_in_catalog,
    remove_skill_u05, set_flag, valid_single_sha_target,
};
use super::*;
use crate::catalog::{CardKind, Catalog, Suit};
use crate::event::CardUseData;
use crate::ids::{CardId, EventId, PlayerId};
use crate::machine::{Ctx, LogEvent};
use crate::state::GameStateCore;

pub(crate) fn ensure_ow_player_inited(core: &mut GameStateCore, catalog: &Catalog, p: PlayerId) {
    if !core.player(p).alive {
        return;
    }
    if has_skill_u05(core, catalog, p, "jijia") && flag(core, p, "ow_jijia_inited") == 0 {
        set_flag(core, p, "ow_jijia_inited", 1);
        core.player_mut(p).jijia_hp = 4;
        set_flag(core, p, "ow_jijia_charge", 0);
    }
    if has_skill_u05(core, catalog, p, "zhongdun") && flag(core, p, "ow_zhongdun_inited") == 0 {
        set_flag(core, p, "ow_zhongdun_inited", 1);
        let cur = core.player(p).hujia;
        core.player_mut(p).hujia = cur.saturating_add(8);
    }
}

pub(crate) fn host_on_game_start_ow(core: &mut GameStateCore, catalog: &Catalog) {
    if !ow_enabled_in_catalog(catalog) {
        return;
    }
    let seats = core.players.len() as PlayerId;
    for p in 0..seats {
        ensure_ow_player_inited(core, catalog, p);
    }
}

pub(crate) fn host_sha_begin_step_ow(
    ctx: &mut Ctx<'_>,
    sha_frame_id: EventId,
    source: PlayerId,
    target: PlayerId,
    _sha_card: Option<CardId>,
    priority_ge_6: bool,
) -> bool {
    if !ow_enabled_in_catalog(ctx.catalog) {
        return false;
    }
    if priority_ge_6 {
        // 1) qianggu2（useCardToBefore on target）
        let q_done = (source, format!("ow_sha_qianggu2_done:{sha_frame_id}:{target}"));
        if has_skill_u05(ctx.core, ctx.catalog, target, "qianggu2")
            && !ctx.core.vm_aux.flags.contains_key(&q_done)
        {
            ctx.core.vm_aux.flags.insert(q_done, 1);
            ctx.log(LogEvent::SkillTriggered {
                skill: "qianggu2",
                owner: target,
                presentation: None,
            });
            let has_basic = ctx
                .core
                .player(source)
                .hand
                .iter()
                .any(|c| ctx.catalog.card_kind(*c).is_basic_kind());
            if !has_basic {
                ctx.core
                    .vm_aux
                    .flags
                    .insert((target, format!("sha_cancelled_target:{sha_frame_id}")), 1);
                return false;
            }
            ctx.res.parks.push(crate::resolution::Park {
                resume_depth: ctx.res.stack.len(),
                slot: crate::resolution::Slot::Trigger {
                    frame: sha_frame_id,
                },
                machine: Box::new(HostMachine {
                    def_key: "qianggu2".to_string(),
                    behavior: HostBehavior::U05OwBatchA,
                    owner: target,
                    target: Some(target),
                    card: Some(sha_frame_id),
                    cards: Vec::new(),
                    targets: vec![source],
                    stage: HostStage::Act,
                }),
                awaiting_answer: false,
            });
            return true;
        }
        // 2) qinru（useCardToBegin on source）
        let qr_done = (source, format!("ow_sha_qinru_done:{sha_frame_id}:{target}"));
        let cancelled = ctx
            .core
            .vm_aux
            .flags
            .contains_key(&(target, format!("sha_cancelled_target:{sha_frame_id}")));
        if has_skill_u05(ctx.core, ctx.catalog, source, "qinru")
            && !ctx.catalog.is_tieji_sealed(ctx.core, source)
            && target != source
            && !cancelled
            && flag(ctx.core, target, "ow_qinru_fengyin") == 0
            && !ctx.core.vm_aux.flags.contains_key(&qr_done)
        {
            ctx.core.vm_aux.flags.insert(qr_done, 1);
            ctx.res.parks.push(crate::resolution::Park {
                resume_depth: ctx.res.stack.len(),
                slot: crate::resolution::Slot::Trigger {
                    frame: sha_frame_id,
                },
                machine: Box::new(HostMachine {
                    def_key: "qinru".to_string(),
                    behavior: HostBehavior::U05OwBatchA,
                    owner: source,
                    target: Some(target),
                    card: None,
                    cards: Vec::new(),
                    targets: vec![target],
                    stage: HostStage::Act,
                }),
                awaiting_answer: false,
            });
            return true;
        }
        return false;
    }

    // priority < 6（Renwang 之后、Bagua/闪问之前）
    let done_key = (source, format!("sha_ow_begin_done:{sha_frame_id}:{target}"));
    if ctx.core.vm_aux.flags.contains_key(&done_key) {
        return false;
    }
    ctx.core.vm_aux.flags.insert(done_key, 1);

    // 1) bfengshi（shaBegin forced: 20% * cards_used_turn 概率直击）
    if has_skill_u05(ctx.core, ctx.catalog, source, "bfengshi")
        && !has_skill_u05(ctx.core, ctx.catalog, source, "bfengshi4")
    {
        let pct = (ctx.core.turn.cards_used_turn * 20).min(100);
        if pct > 0 && ctx.core.rng.gen_range(100u32) < pct {
            ctx.core.vm_aux.flags.insert(
                (source, format!("sha_ow_bfengshi_direct:{sha_frame_id}:{target}")),
                1,
            );
            ctx.log(LogEvent::SkillTriggered {
                skill: "bfengshi",
                owner: source,
                presentation: None,
            });
        }
    }

    // 2) zhencha（侦查模式：每次用杀指定目标可摸1或回1）
    if has_skill_u05(ctx.core, ctx.catalog, source, "zhencha")
        && !ctx.catalog.is_tieji_sealed(ctx.core, source)
        && ctx.core.player(source).baolei_mode != 1
    {
        ctx.res.parks.push(crate::resolution::Park {
            resume_depth: ctx.res.stack.len(),
            slot: crate::resolution::Slot::Trigger {
                frame: sha_frame_id,
            },
            machine: Box::new(HostMachine {
                def_key: "zhencha".to_string(),
                behavior: HostBehavior::U05OwBatchA,
                owner: source,
                target: Some(target),
                card: Some(1), // 1 = shaBegin mode
                cards: Vec::new(),
                targets: vec![target],
                stage: HostStage::Act,
            }),
            awaiting_answer: false,
        });
        return true;
    }
    false
}

pub(crate) fn host_draw_begin_step_ow(ctx: &mut Ctx<'_>, player: PlayerId, pending: &mut i32) {
    if has_skill_u05(ctx.core, ctx.catalog, player, "owliyong") {
        let extra = flag(ctx.core, player, "ow_pingzhang_draw");
        if extra > 0 {
            *pending += extra as i32;
            ctx.log(LogEvent::SkillTriggered {
                skill: "owliyong",
                owner: player,
                presentation: None,
            });
        }
    }
}

pub(crate) fn host_card_use_after_ow(ctx: &mut Ctx<'_>, u: &CardUseData) {
    if !ow_enabled_in_catalog(ctx.catalog) {
        return;
    }
    let owner = u.player;
    let is_sha = u.effective_kind(ctx.catalog).is_sha_like();

    if !u.cancelled && is_sha && has_skill_u05(ctx.core, ctx.catalog, owner, "feiren3") {
        if flag(ctx.core, owner, "ow_feiren3_skip_next") > 0 {
            set_flag(ctx.core, owner, "ow_feiren3_skip_next", 0);
            return;
        }
        let spade = u
            .suit(ctx.catalog)
            .is_some_and(|s| s == Suit::Spade);
        if spade
            && ctx.core.player(owner).alive
            && !ctx.catalog.is_tieji_sealed(ctx.core, owner)
            && !u.targets.is_empty()
            && u.targets
                .iter()
                .all(|t| valid_single_sha_target(ctx.core, ctx.catalog, owner, *t))
        {
            let nature_code = match (u.effective_kind(ctx.catalog), u.nature) {
                (CardKind::FireSha, _) | (_, crate::event::DamageNature::Fire) => 1u32,
                (CardKind::LeiSha, _) | (_, crate::event::DamageNature::Thunder) => 2u32,
                _ => 0u32,
            };
            ctx.res.parks.push(crate::resolution::Park {
                resume_depth: ctx.res.stack.len(),
                slot: crate::resolution::Slot::Intrinsic {
                    frame: crate::resolution::NO_FRAME,
                },
                awaiting_answer: false,
                machine: Box::new(HostMachine {
                    def_key: "feiren3".to_string(),
                    behavior: HostBehavior::U05OwBatchC,
                    owner,
                    target: None,
                    card: None,
                    cards: vec![nature_code],
                    targets: u.targets.clone(),
                    stage: HostStage::Act,
                }),
            });
        }
    }
}

pub(crate) fn host_before_facedown_turn_skip_ow(
    ctx: &mut Ctx<'_>,
    victim_b: PlayerId,
    _prev: Option<PlayerId>,
) -> bool {
    if !ow_enabled_in_catalog(ctx.catalog) {
        return false;
    }
    let Some(def) = ctx.catalog.skill_by_name("shanxian") else {
        return false;
    };
    if ctx.catalog.skills[def as usize].behavior.as_host_behavior()
        != Some(HostBehavior::U05OwBatchD)
    {
        return false;
    }
    let order = ctx.core.seat_order_from(victim_b);
    let Some(owner) = order.into_iter().find(|&o| {
        o != victim_b
            && ctx.core.player(o).alive
            && !ctx.core.player(o).face_down
            && ctx.core.skills.get(&(o, def)).is_some_and(|i| i.enabled)
            && !ctx.catalog.is_tieji_sealed(ctx.core, o)
            && flag(ctx.core, o, "ow_shanxian_target") == 0
    }) else {
        return false;
    };
    ctx.res.parks.push(crate::resolution::Park {
        resume_depth: ctx.res.stack.len(),
        slot: crate::resolution::Slot::Intrinsic {
            frame: crate::resolution::NO_FRAME,
        },
        machine: Box::new(HostMachine {
            def_key: "shanxian".to_string(),
            behavior: HostBehavior::U05OwBatchD,
            owner,
            target: Some(victim_b),
            card: None,
            cards: vec![1], // [1] = victim_b 翻面跳过前（phaseBefore）触发模式
            targets: Vec::new(),
            stage: HostStage::Act,
        }),
        awaiting_answer: false,
    });
    true
}

pub(crate) fn host_on_shanxian_turnover_ow(ctx: &mut Ctx<'_>, player: PlayerId) -> bool {
    if !ow_enabled_in_catalog(ctx.catalog)
        || flag(ctx.core, player, "ow_shanxian_pending_turnover") == 0
    {
        return false;
    }
    set_flag(ctx.core, player, "ow_shanxian_pending_turnover", 0);
    if ctx.core.player(player).face_down {
        set_flag(ctx.core, player, "ow_shanxian_target", 0);
        return false;
    }
    ctx.res.parks.push(crate::resolution::Park {
        resume_depth: ctx.res.stack.len(),
        slot: crate::resolution::Slot::Intrinsic {
            frame: crate::resolution::NO_FRAME,
        },
        machine: Box::new(HostMachine {
            def_key: "shanxian".to_string(),
            behavior: HostBehavior::U05OwBatchD,
            owner: player,
            target: Some(player),
            card: None,
            cards: vec![2], // [2] = phaseAfter Commit 翻面后再归还回合交接
            targets: Vec::new(),
            stage: HostStage::Act,
        }),
        awaiting_answer: false,
    });
    ctx.emit(crate::event::EventData::TurnOverBefore {
        player,
        cancelled: false,
    });
    true
}

pub(crate) fn host_on_player_died_ow(ctx: &mut Ctx<'_>, dead: PlayerId) {
    if !ow_enabled_in_catalog(ctx.catalog) {
        return;
    }
    set_flag(ctx.core, dead, "ow_shanxian_target", 0);
    set_flag(ctx.core, dead, "ow_shanxian_pending_turnover", 0);
    // 1) 回收 dead 身上陷阱牌（ow_shoujia / ow_dulei）
    for pile_key in ["ow_shoujia", "ow_dulei"] {
        if let Some(cards) = ctx.core.vm_aux.piles.remove(&(dead, pile_key.to_string())) {
            for c in cards {
                ctx.to_discard(c);
            }
        }
    }
    let seats = ctx.core.players.len() as PlayerId;
    // 2) 若 dead 曾使用强固/侵入/冰墙/谐/乱/涡流，按对应 subskill 清理场上衍生标记
    for p in 0..seats {
        if flag(ctx.core, p, &format!("ow_qinru_turn:{dead}")) > 0 {
            set_flag(ctx.core, p, &format!("ow_qinru_turn:{dead}"), 0);
        }
        if flag(ctx.core, p, "ow_qianggu2_caster") == u32::from(dead) + 1 {
            set_flag(ctx.core, p, "ow_qianggu2_caster", 0);
            remove_skill_u05(ctx, p, "qianggu2");
        }
        if flag(ctx.core, p, "ow_xie_owner") == u32::from(dead) + 1 {
            remove_skill_u05(ctx, p, "xie2");
            set_flag(ctx.core, p, "ow_xie_owner", 0);
            set_flag(ctx.core, p, "ow_xie_phase", 0);
        }
        if flag(ctx.core, p, "ow_luan_owner") == u32::from(dead) + 1 {
            remove_skill_u05(ctx, p, "luan2");
            set_flag(ctx.core, p, "ow_luan_owner", 0);
        }
        if flag(ctx.core, dead, "ow_woliu_used") > 0 {
            remove_skill_u05(ctx, p, "woliu2");
        }
    }
    clear_bingqiang_by_owner(ctx, dead);
    let _ = alive_others;
}

pub(crate) fn on_skill_removed_ow(
    core: &mut GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    skill_name: &str,
) -> Vec<CardId> {
    let mut out = Vec::new();
    if !ow_enabled_in_catalog(catalog) {
        return out;
    }
    match skill_name {
        "shoujia2" => {
            set_flag(core, owner, "ow_shoujia2_caster", 0);
            if let Some(c) = core.vm_aux.piles.remove(&(owner, "ow_shoujia".to_string())) {
                out.extend(c);
            }
        }
        "dulei2" => {
            if let Some(c) = core.vm_aux.piles.remove(&(owner, "ow_dulei".to_string())) {
                out.extend(c);
            }
        }
        "qianggu2" => set_flag(core, owner, "ow_qianggu2_caster", 0),
        "tuijin2" => set_flag(core, owner, "ow_tuijin_target", 0),
        "juji2" => set_flag(core, owner, "ow_juji2_target", 0),
        "liangou2" => {
            set_flag(core, owner, "ow_liangou2_from", 0);
            set_flag(core, owner, "ow_liangou2_dmg", 0);
        }
        "xie2" => {
            set_flag(core, owner, "ow_xie_owner", 0);
            set_flag(core, owner, "ow_xie_phase", 0);
        }
        "luan2" => set_flag(core, owner, "ow_luan_owner", 0),
        "guangshu_diamond" | "guangshu_heart" | "guangshu_club" | "guangshu_spade" => {}
        _ => {}
    }
    let _ = CardKind::Sha;
    out
}
