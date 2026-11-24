//! L11-6a 太虚自有 NPC 死亡/复活/中断类技能：`beiming`/`wangzu`/`juhun`/`zhuhun`/`shiyuan_fuhuo`/`wangyuan`。

use super::*;
use crate::event::{EventData, Phase};

pub(super) fn handle_death_revive(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Option<Flow> {
    let owner = m.owner;
    match m.def_key.as_str() {
        "txhj_beiming" => {
            if let Some(k) = m.target.filter(|&k| k != owner && ctx.core.player(k).alive) {
                let hand = ctx.core.player(k).hand.clone();
                if !hand.is_empty() {
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "悲鸣",
                        owner,
                        presentation: None,
                    });
                    discard_player_cards(ctx, k, &hand);
                }
            }
            Some(Flow::Done)
        }
        "txhj_wangzu" => {
            let cur = ctx.core.turn.current;
            if cur != owner && ctx.core.turn.phase == Phase::Play && ctx.core.player(cur).alive {
                ctx.log(LogEvent::SkillTriggered {
                    skill: "亡阻",
                    owner,
                    presentation: None,
                });
                ctx.emit(EventData::Phase {
                    player: cur,
                    phase: Phase::Play,
                    begin: false,
                });
            }
            Some(Flow::Done)
        }
        "txhj_juhun" => {
            if m.card == Some(MODE_SECONDARY) {
                set_flag(ctx.core, owner, "l11_juhun_armed", 1);
                return Some(Flow::Done);
            }
            let mask = flag(ctx.core, owner, "l11_juhun_dead_mask");
            set_flag(ctx.core, owner, "l11_juhun_dead_mask", 0);
            let dead: Vec<PlayerId> = (0..ctx.core.players.len() as PlayerId)
                .filter(|&p| !ctx.core.player(p).alive && (mask & (1u32 << p)) != 0)
                .collect();
            if let Some(&pick) = (!dead.is_empty()).then(|| {
                let idx = ctx.core.rng.gen_range(dead.len() as u32) as usize;
                &dead[idx]
            }) {
                ctx.log(LogEvent::SkillTriggered {
                    skill: "拘魂",
                    owner,
                    presentation: None,
                });
                crate::kernel::revive_with_hp_draw(ctx, pick, 3, 3);
            }
            Some(Flow::Done)
        }
        "txhj_zhuhun" => Some(handle_zhuhun(m, ctx)),
        _ => None,
    }
}

fn handle_zhuhun(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_ZHUHUN_TARGET),
        HostStage::TargetChoice => {
            m.card = None;
            let Some(t) = answered_players(ctx).into_iter().next().filter(|&p| {
                p != owner && (p as usize) < ctx.core.players.len() && !ctx.core.player(p).alive
            }) else {
                return Flow::Done;
            };
            let mask = flag(ctx.core, owner, "l11_zhuhun_dead_mask");
            let bit = 1u32 << t;
            if (mask & bit) == 0 {
                return Flow::Done;
            }
            ctx.core.vm_aux.inc_once(owner, 1, "txhj_zhuhun");
            set_flag(ctx.core, owner, "l11_zhuhun_dead_mask", mask & !bit);
            ctx.log(LogEvent::SkillTriggered {
                skill: "注魂",
                owner,
                presentation: None,
            });
            crate::kernel::revive_as_txhj_changyuanshibing(ctx, owner, t);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(crate) fn on_player_died_l11_own_npc(
    ctx: &mut Ctx<'_>,
    dead: PlayerId,
    killer: Option<PlayerId>,
) {
    let seats = ctx.core.players.len() as PlayerId;
    for owner in 0..seats {
        if owner == dead || !ctx.core.player(owner).alive {
            continue;
        }
        if has_named_skill(ctx.core, ctx.catalog, owner, "txhj_juhun")
            && is_locked_unblocked_l11_owner(ctx.core, ctx.catalog, owner)
            && flag(ctx.core, owner, "l11_juhun_armed") > 0
            && ctx.core.is_friend_of(ctx.catalog, owner, dead)
        {
            let m = flag(ctx.core, owner, "l11_juhun_dead_mask");
            set_flag(ctx.core, owner, "l11_juhun_dead_mask", m | (1u32 << dead));
        }
        if has_named_skill(ctx.core, ctx.catalog, owner, "txhj_zhuhun")
            && is_unblocked_l11_owner(ctx.core, ctx.catalog, owner)
            && ctx.core.is_friend_of(ctx.catalog, owner, dead)
        {
            let m = flag(ctx.core, owner, "l11_zhuhun_dead_mask");
            set_flag(ctx.core, owner, "l11_zhuhun_dead_mask", m | (1u32 << dead));
        }
        if has_named_skill(ctx.core, ctx.catalog, owner, "txhj_wangyuan")
            && is_locked_unblocked_l11_owner(ctx.core, ctx.catalog, owner)
        {
            ctx.log(LogEvent::SkillTriggered {
                skill: "亡怨",
                owner,
                presentation: None,
            });
            ctx.core.player_mut(owner).max_hp = ctx.core.player(owner).max_hp.saturating_add(1);
            ctx.recover(owner, 1, None);
            let cur = flag(ctx.core, owner, "l11_wangyuan_marks");
            set_flag(ctx.core, owner, "l11_wangyuan_marks", cur + 1);
        }
    }
    if let Some(actor) = killer {
        let is_lord = |seat: PlayerId| {
            matches!(&ctx.core.mode, crate::state::GameMode::Identity { roles }
                if roles.get(seat as usize) == Some(&crate::state::Role::Lord))
        };
        if actor != dead
            && ctx.core.player(actor).alive
            && has_named_skill(ctx.core, ctx.catalog, actor, "txhj_shiyuan")
            && is_locked_unblocked_l11_owner(ctx.core, ctx.catalog, actor)
            && !is_lord(actor)
            && !is_lord(dead)
        {
            ctx.log(LogEvent::SkillTriggered {
                skill: "尸怨",
                owner: actor,
                presentation: None,
            });
            crate::kernel::revive_as_txhj_changyuanshibing(ctx, actor, dead);
        }
    }
}
