//! U08 英雄杀 Batch D 内核回调钩子（`u08_batch_d_hooks.rs`）。
//!
//! 从 `u08_batch_d_ext.rs` 拆出，使 `u08_batch_d*.rs` 保持职责边界清晰且有效行数低于上限。

use super::*;
use crate::event::CardUseData;
use crate::ids::{CardId, EventId, PlayerId};
use crate::machine::{Ctx, LogEvent};

/// `shaBegin` 逐目标钩子：
/// - `priority_ge_6 = true`：`bolehuiyan`（`英雄杀/character/skill.js:2407-2476`, `priority 11`），
///   过滤 `elo_is_under_control(target)`，先于仁王盾与八卦判定。
/// - `priority_ge_6 = false`：`cike`（`英雄杀/character/skill.js:3001-3027`, `priority 0`），
///   仅在仁王盾未取消时逐目标询问。
pub(super) fn host_sha_begin_step_impl(
    ctx: &mut Ctx<'_>,
    sha_frame_id: EventId,
    source: PlayerId,
    target: PlayerId,
    sha_card: Option<CardId>,
    priority_ge_6: bool,
) -> bool {
    if !ctx.core.player(source).alive || !ctx.core.player(target).alive {
        return false;
    }
    if priority_ge_6 {
        if crate::content::host::step::l10_scs::sha_begin_l10_scs(
            ctx,
            sha_frame_id,
            source,
            target,
            sha_card,
        ) {
            return true;
        }
        let Some(bhy_def) = ctx.catalog.skill_by_name("bolehuiyan") else {
            return false;
        };
        if crate::kernel::elo_is_under_control(ctx.core, target)
            || ctx.core.player(target).hand.is_empty()
        {
            return false;
        }
        let n = ctx.core.players.len() as PlayerId;
        let cur = ctx.core.turn.current;
        for i in 0..n {
            let owner = (cur + i) % n;
            if !ctx.core.player(owner).alive
                || !has_skill_named(ctx.core, ctx.catalog, owner, "bolehuiyan")
                || flag(ctx.core, owner, "bolehuiyan_used_turn") != 0
                || source == owner
                || target == owner
            {
                continue;
            }
            ctx.out.push(LogEvent::SkillTriggered {
                skill: ctx.catalog.skills[bhy_def as usize].name,
                owner,
                presentation: None,
            });
            ctx.res.parks.push(crate::resolution::Park {
                resume_depth: ctx.res.stack.len(),
                slot: crate::resolution::Slot::Trigger {
                    frame: sha_frame_id,
                },
                machine: Box::new(HostMachine {
                    def_key: "bolehuiyan".to_string(),
                    behavior: HostBehavior::U08YxsBatchD,
                    owner,
                    target: Some(target),
                    card: Some(1),
                    cards: Vec::new(),
                    targets: vec![source],
                    stage: HostStage::Act,
                }),
                awaiting_answer: false,
            });
            return true;
        }
        return false;
    }
    let Some(cike_def) = ctx.catalog.skill_by_name("cike") else {
        return false;
    };
    if !has_skill_named(ctx.core, ctx.catalog, source, "cike")
        || flag(ctx.core, source, &format!("sha_cike_done:{sha_frame_id}")) != 0
    {
        return false;
    }
    set_flag(ctx.core, source, &format!("sha_cike_done:{sha_frame_id}"), 1);
    ctx.out.push(LogEvent::SkillTriggered {
        skill: ctx.catalog.skills[cike_def as usize].name,
        owner: source,
        presentation: None,
    });
    ctx.res.parks.push(crate::resolution::Park {
        resume_depth: ctx.res.stack.len(),
        slot: crate::resolution::Slot::Trigger {
            frame: sha_frame_id,
        },
        machine: Box::new(HostMachine {
            def_key: "cike".to_string(),
            behavior: HostBehavior::U08YxsBatchD,
            owner: source,
            target: Some(target),
            card: None,
            cards: sha_card.into_iter().collect(),
            targets: Vec::new(),
            stage: HostStage::Act,
        }),
        awaiting_answer: false,
    });
    true
}

/// `shaDamage` / `shaEnd` 出口钩子：
/// - `is_damage = true`：`bolehuiyan3`（`英雄杀/character/skill.js:2495-2512`）仅在所预测杀自身 `shaDamage` 后结算。
/// - `is_damage = false`：`bolehuiyan2`（`英雄杀/character/skill.js:2477-2494`）仅在非取消正常 `shaEnd` 结算。
/// 按上游 `filter(event, player) { return !!player.storage.bolehuiyan; }` 与 CDP `1h` 实测，
/// 嵌套第二张【杀】结算时同样直接消费并结算预测，不再用 `bolehuiyan_card_frame` 门控拦截。
pub(super) fn host_sha_end_step_impl(
    ctx: &mut Ctx<'_>,
    sha_frame_id: EventId,
    source: PlayerId,
    target: PlayerId,
    _sha_card: Option<CardId>,
    is_damage: bool,
) -> bool {
    let sub_name = if is_damage {
        "bolehuiyan3"
    } else {
        "bolehuiyan2"
    };
    let Some(sub_def) = ctx.catalog.skill_by_name(sub_name) else {
        return false;
    };
    let n = ctx.core.players.len() as PlayerId;
    let cur = ctx.core.turn.current;
    for i in 0..n {
        let owner = (cur + i) % n;
        if !ctx.core.player(owner).alive
            || flag(ctx.core, owner, "bolehuiyan_pred") == 0
            || !has_skill_named(ctx.core, ctx.catalog, owner, sub_name)
        {
            continue;
        }
        ctx.out.push(LogEvent::SkillTriggered {
            skill: ctx.catalog.skills[sub_def as usize].name,
            owner,
            presentation: None,
        });
        ctx.res.parks.push(crate::resolution::Park {
            resume_depth: ctx.res.stack.len(),
            slot: crate::resolution::Slot::Trigger {
                frame: sha_frame_id,
            },
            machine: Box::new(HostMachine {
                def_key: sub_name.to_string(),
                behavior: HostBehavior::U08YxsBatchD,
                owner,
                target: Some(target),
                card: Some(if is_damage { 2 } else { 3 }),
                cards: Vec::new(),
                targets: vec![source],
                stage: HostStage::Act,
            }),
            awaiting_answer: false,
        });
        return true;
    }
    false
}

/// `useCardAfter` 弹出钩子：`jieyong`（`英雄杀/character/skill.js:2797-2825`）——
/// 在 `CardUsed` 完整结算完成并弹出后触发，不限卡牌类别，仅检实体牌仍在弃牌堆
/// (`event.cards.filterInD().length > 0`)、未持 `jieyong2`、且手牌/装备区另有黑色牌可弃。
pub(super) fn host_card_use_after_impl(ctx: &mut Ctx<'_>, u: &CardUseData) {
    crate::content::host::step::l27_fixes::host_card_use_after_clandianzhan(ctx, u);
    crate::content::host::step::u01_fixes_1::u01_card_use_after_hook(ctx, u);
    crate::content::host::step::l11_common::on_card_use_after_l11(ctx, u);
    if u.cancelled {
        return;
    }
    let Some(cid) = u.card else {
        return;
    };
    let owner = u.player;
    if !ctx.core.player(owner).alive
        || !has_skill_named(ctx.core, ctx.catalog, owner, "jieyong")
        || ctx.catalog.is_tieji_sealed(ctx.core, owner)
        || flag(ctx.core, owner, "jieyong2_used") != 0
        || has_skill_named(ctx.core, ctx.catalog, owner, "jieyong2")
        || !ctx.core.discard.contains(&cid)
    {
        return;
    }
    let has_black = hand_and_equips(ctx.core, owner)
        .into_iter()
        .any(|c| c != cid && is_black_card(ctx.core, ctx.catalog, owner, c));
    if !has_black {
        return;
    }
    ctx.res.parks.push(crate::resolution::Park {
        resume_depth: ctx.res.stack.len(),
        slot: crate::resolution::Slot::Intrinsic {
            frame: crate::resolution::NO_FRAME,
        },
        awaiting_answer: false,
        machine: Box::new(HostMachine {
            def_key: "jieyong".to_string(),
            behavior: HostBehavior::U08YxsBatchD,
            owner,
            target: None,
            card: Some(1),
            cards: vec![cid],
            targets: Vec::new(),
            stage: HostStage::Act,
        }),
    });
}
