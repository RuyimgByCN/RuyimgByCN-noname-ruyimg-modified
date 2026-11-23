//! U05-7b 杀结算与阶段收束宿主钩子：`sadengjinhuan`、`guiyanfadao`、`shaHit`/`shaEnd`/`drawBegin`/`cardUseAfter` 与 `on_outer_settled`。

use super::{asks, respondable_shans};
use super::*;
use crate::catalog::CardKind;
use crate::choice::ChoiceOption;
use crate::event::{DamageNature, EventData};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow, LogEvent};

pub(super) fn step_sadengjinhuan(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let Some(t) = m.target else {
        return Flow::Done;
    };
    match m.stage {
        HostStage::Act => {
            m.stage = HostStage::SelectNature;
            asks::wait_confirm(m.owner, "sadengjinhuan_judge_confirm")
        }
        HostStage::SelectNature => {
            if !matches!(ctx.answer().first(), Some(ChoiceOption::Confirm))
                || !ctx.core.player(t).alive
            {
                return Flow::Done;
            }
            let judged = ctx.draw_judge_card();
            let red = matches!(
                ctx.catalog.cards[judged as usize].suit,
                crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
            );
            ctx.log(LogEvent::Judged {
                player: m.owner,
                card: judged,
            });
            ctx.to_discard(judged);
            if !red {
                return Flow::Done;
            }
            let shans = respondable_shans(ctx.core, ctx.catalog, t);
            if shans.is_empty() {
                emit_sadeng_damage(ctx, m.owner, t, m.card);
                return Flow::Done;
            }
            m.stage = HostStage::CardsAsk;
            asks::wait_select_cards(t, "sadengjinhuan_extra_shan", shans, 0, 1, false)
        }
        HostStage::CardsAsk => {
            if let Some(ChoiceOption::Card(shan)) = ctx.answer().first().copied() {
                if ctx.core.player(t).hand.contains(&shan)
                    || ctx.core.player(t).equips.contains(&shan)
                {
                    engine_native_kernel::kernel::discard_response_card(
                        ctx,
                        t,
                        shan,
                        CardKind::Shan,
                    );
                    ctx.emit(EventData::CardResponded {
                        card: shan,
                        player: t,
                    });
                    return Flow::Done;
                }
            }
            if ctx.core.player(t).alive {
                emit_sadeng_damage(ctx, m.owner, t, m.card);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn step_guiyanfadao(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let Some(t) = m.target else {
        return Flow::Done;
    };
    match m.stage {
        HostStage::Act => {
            if !ctx.core.player(t).alive {
                ctx.core
                    .vm_aux
                    .flags
                    .remove(&(m.owner, format!("u05_guiyanfadao_active_fid:{t}")));
                return Flow::Done;
            }
            m.stage = HostStage::SelectNature;
            asks::wait_confirm(m.owner, "u05_guiyanfadao_confirm")
        }
        HostStage::SelectNature => {
            let fid = ctx
                .core
                .vm_aux
                .flags
                .remove(&(m.owner, format!("u05_guiyanfadao_active_fid:{t}")));
            if matches!(ctx.answer().first(), Some(ChoiceOption::Confirm))
                && ctx.core.player(t).alive
            {
                if let Some(fid) = fid {
                    ctx.core
                        .vm_aux
                        .flags
                        .insert((m.owner, format!("u05_guiyanfadao_unhurt:{fid}:{t}")), 1);
                }
                ctx.emit(EventData::LoseHp {
                    player: t,
                    amount: 1,
                });
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

/// Hooks `shaHit` (`ShaResolveMachine::sha_hit_pre_damage`) before `damageBegin2` (`Hanbing`/`Qilin`) and `DamageBefore`.
pub(crate) fn host_sha_hit_step_u05_7b(
    ctx: &mut Ctx<'_>,
    sha_frame_id: crate::ids::EventId,
    source: PlayerId,
    target: PlayerId,
    card: Option<CardId>,
) -> bool {
    let has_guiyan = ctx
        .catalog
        .skill_by_name("guiyanfadao")
        .is_some_and(|d| ctx.core.has_skill(source, d));
    if !has_guiyan
        || !ctx.core.player(source).alive
        || !ctx.core.player(target).alive
        || crate::kernel::mbcmfangzhu_baiban(ctx.core, source)
    {
        return false;
    }
    let done_key = format!("u05_guiyanfadao_done:{sha_frame_id}:{target}");
    if ctx
        .core
        .vm_aux
        .flags
        .contains_key(&(source, done_key.clone()))
    {
        return false;
    }
    ctx.core.vm_aux.flags.insert((source, done_key), 1);
    ctx.core.vm_aux.flags.insert(
        (source, format!("u05_guiyanfadao_active_fid:{target}")),
        sha_frame_id,
    );
    ctx.res.parks.push(crate::resolution::Park {
        resume_depth: ctx.res.stack.len(),
        slot: crate::resolution::Slot::Trigger {
            frame: sha_frame_id,
        },
        machine: Box::new(HostMachine {
            def_key: "guiyanfadao".to_string(),
            behavior: HostBehavior::U05SwdGuiyanfadao,
            owner: source,
            target: Some(target),
            card,
            cards: Vec::new(),
            targets: Vec::new(),
            stage: HostStage::Act,
        }),
        awaiting_answer: false,
    });
    true
}

fn emit_sadeng_damage(ctx: &mut Ctx<'_>, source: PlayerId, target: PlayerId, card: Option<CardId>) {
    let nature = match card.map(|c| ctx.catalog.card_kind(c)) {
        Some(CardKind::FireSha) => DamageNature::Fire,
        Some(CardKind::LeiSha) => DamageNature::Thunder,
        _ => DamageNature::Normal,
    };
    ctx.emit(EventData::Damage(crate::machine::damage_data_with(
        Some(source),
        target,
        1,
        nature,
        card,
    )));
}

/// 冲虚/赤龙风雪（`shaEnd` 全局触发）：`sha` 结算完毕时令携带者摸 1 并移去 `chilongfengxue`；未命中时清临时 `tianxianjiu`。
pub(crate) fn host_sha_end_step_u05_7b(
    ctx: &mut Ctx<'_>,
    source: PlayerId,
    is_damage: bool,
) -> bool {
    let mut drew = false;
    if let Some(def) = ctx.catalog.skill_by_name("chilongfengxue") {
        let seats = ctx.core.players.len() as PlayerId;
        for p in 0..seats {
            if ctx.core.has_skill(p, def) {
                ctx.core.skills.remove(&(p, def));
                if ctx.core.player(p).alive {
                    ctx.start_draw(p, 1);
                    drew = true;
                }
            }
        }
    }
    if let Some(tdef) = ctx.catalog.skill_by_name("tianxianjiu") {
        ctx.core.skills.remove(&(source, tdef));
    }
    if ctx
        .core
        .vm_aux
        .flags
        .remove(&(source, "u05_tianxianjiu_active_fallback".to_string()))
        .is_some()
        && is_damage
        && ctx.core.player(source).alive
    {
        ctx.start_draw(source, 2);
        drew = true;
    }
    drew
}

/// 摸牌阶段开始钩：`hslingjian_xingtigaizao_equip5` 使摸牌数 +1。
pub(crate) fn host_draw_begin_step_u05_7b(ctx: &mut Ctx<'_>, player: PlayerId, pending: &mut i32) {
    if ctx
        .catalog
        .skill_by_name("hslingjian_xingtigaizao_equip5")
        .is_some_and(|d| ctx.core.has_skill(player, d))
    {
        *pending += 1;
    }
}

/// 用牌后钩：`guiyanfadao2` 清理临时火杀转换状态。
pub(crate) fn host_card_use_after_u05_7b(ctx: &mut Ctx<'_>, u: &crate::event::CardUseData) {
    if ctx
        .catalog
        .skill_by_name("guiyanfadao2")
        .is_some_and(|d| ctx.core.has_skill(u.player, d))
    {
        ctx.core
            .vm_aux
            .flags
            .remove(&(u.player, "u05_zhuque_sha_nature".to_string()));
    }
}

/// 出牌/回合阶段结束清理占位与外层结算清 `swd_qiankundai_pending_lose` 失袋摸牌。
pub(crate) fn on_phase_end_u05_7b(_ctx: &mut Ctx<'_>) {}

pub(crate) fn on_outer_settled_u05_7b(ctx: &mut Ctx<'_>) {
    crate::kernel::flush_qiankundai_pending_lose(ctx);
}
