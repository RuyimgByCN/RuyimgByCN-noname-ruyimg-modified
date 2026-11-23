//! U05 仙剑 Batch C（15 技）：`xiaoyue`, `xiaoyue2`, `yanzhan2`, `yanzhan3`,
//! `yanzhan4`, `danqing`, `danqing_count`, `zhangmu_ai`, `leiyu`, `leiyu2`,
//! `leiyu3`, `leiyu4`, `guijin2`, `chengxin2`, `yuexing2`。

use super::xianjian_batch_asks::*;
use super::*;
use crate::catalog::{CardKind, QueryCtx, SkillLimits, TriggerMeta};
use crate::event::{CardUseData, DamageNature, EventData, EventKind, Phase, Timing};
use crate::ids::PlayerId;
use crate::machine::{Ctx, Flow, LogEvent};

pub(super) fn trigger_meta(name: &str) -> Option<TriggerMeta> {
    let meta = |event: EventKind, extra_events: Vec<EventKind>| TriggerMeta {
        event,
        timing: Timing::Normal,
        priority: 0,
        order_selectable: false,
        optional: false,
        extra_events,
    };
    match name {
        "yanzhan2" => Some(meta(EventKind::DamageEnd, Vec::new())),
        "danqing" => Some(meta(EventKind::PhaseEnd, Vec::new())),
        "leiyu" => Some(meta(EventKind::PhaseEnd, Vec::new())),
        "leiyu3" => Some(meta(EventKind::Death, Vec::new())),
        "chengxin2" => Some(meta(EventKind::DamageBefore, Vec::new())),
        _ => None,
    }
}

pub(super) fn limits(_name: &str) -> SkillLimits {
    SkillLimits::default()
}

pub(super) fn filter(skill: &HostSkillBehavior, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
    match skill.name.as_str() {
        "yanzhan2" => matches!(
            event,
            EventData::Damage(d)
            if d.source == Some(owner) && !d.prevented && d.amount > 0 && !d.settlement.notrigger
                && d.card.is_some_and(|c| flag(ctx.state, owner, &format!("xj_yanzhan_src_card:{c}")) > 0)
        ),
        "danqing" => matches!(
            event,
            EventData::Phase { player, phase: Phase::End, begin: false }
            if *player == owner && flag(ctx.state, owner, "xj_danqing_suits").count_ones() == 4
        ),
        "leiyu" => matches!(
            event,
            EventData::Phase { player, phase: Phase::End, begin: false }
            if *player == owner
                && ctx.state.player(owner).hand.iter().any(|&c| is_black_card(ctx.state, ctx.catalog, owner, c))
                && !leiyu_alive_targets(ctx.state, owner).is_empty()
        ),
        "leiyu3" => matches!(
            event,
            EventData::Death { killer: Some(k), .. }
            if *k == owner && flag(ctx.state, owner, "xj_leiyu2_mark") > 0 && ctx.state.player(owner).alive
        ),
        "chengxin2" => matches!(event, EventData::Damage(d) if d.target == owner && !d.prevented),
        _ => false,
    }
}

fn leiyu_alive_targets(core: &crate::state::GameStateCore, owner: PlayerId) -> Vec<PlayerId> {
    core.seat_order_from(owner)
        .into_iter()
        .filter(|&p| p != owner && core.player(p).alive && flag(core, p, "xj_leiyu_marked") == u32::from(owner) + 1)
        .collect()
}

pub(super) fn machine(skill: &HostSkillBehavior, owner: PlayerId, _event: &EventData) -> HostMachine {
    HostMachine {
        def_key: skill.name.clone(),
        behavior: HostBehavior::U05XianjianBatchC,
        owner,
        target: None,
        card: None,
        cards: Vec::new(),
        targets: Vec::new(),
        stage: HostStage::Act,
    }
}

pub(super) fn step(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.def_key.as_str() {
        "yanzhan2" => {
            set_flag(ctx.core, owner, "u05_mod:yanzhan4", 1);
            ensure_temp_skill(ctx, owner, "yanzhan4", true);
            Flow::Done
        }
        "danqing" => match m.stage {
            HostStage::Act => {
                set_flag(ctx.core, owner, "xj_danqing_suits", 0);
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_DANQING_TARGETS)
            }
            HostStage::TargetChoice => {
                let picked: Vec<PlayerId> = answered_players(ctx).into_iter().take(4).collect();
                for t in picked {
                    u05_apply_buff(ctx, t);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "leiyu" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_LEIYU_CARD),
            HostStage::SelectCard => {
                let Some(card) = answered_cards(ctx).into_iter().next() else {
                    return Flow::Done;
                };
                if !ctx.core.player(owner).hand.contains(&card) || !is_black_card(ctx.core, ctx.catalog, owner, card) {
                    return Flow::Done;
                }
                let targets = leiyu_alive_targets(ctx.core, owner);
                if targets.is_empty() {
                    return Flow::Done;
                }
                ctx.discard_from_hand(owner, card);
                ctx.emit(EventData::CardUsed(CardUseData {
                    card: None,
                    player: owner,
                    targets,
                    nature: DamageNature::Thunder,
                    use_as: Some(CardKind::Jingleishan),
                    cancelled: false,
                }));
                Flow::Done
            }
            _ => Flow::Done,
        },
        "leiyu3" => {
            set_flag(ctx.core, owner, "xj_leiyu2_mark", 0);
            if ctx.core.player(owner).alive {
                ctx.recover(owner, 1, None);
            }
            Flow::Done
        }
        "chengxin2" => {
            for frame in ctx.res.stack.iter_mut().rev() {
                if let EventData::Damage(d) = &mut frame.data {
                    if d.target == owner {
                        d.prevented = true;
                        break;
                    }
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(crate) fn host_on_round_start_xianjian(ctx: &mut Ctx<'_>) {
    let Some(def) = ctx.catalog.skill_by_name("xiaoyue") else {
        return;
    };
    let order = ctx.core.seat_order_from(ctx.core.turn.current);
    for owner in order {
        if !ctx.core.player(owner).alive
            || !ctx.core.has_skill(owner, def)
            || ctx.catalog.is_tieji_sealed(ctx.core, owner)
        {
            continue;
        }
        let shas: Vec<_> = ctx
            .core
            .player(owner)
            .hand
            .iter()
            .copied()
            .filter(|&c| ctx.catalog.card_kind(c).is_sha_like())
            .collect();
        let enemies: Vec<PlayerId> = alive_enemies(ctx.core, owner)
            .into_iter()
            .filter(|&p| sha_target_legal(ctx.core, ctx.catalog, owner, p, false))
            .collect();
        if shas.is_empty() || enemies.is_empty() {
            continue;
        }
        let sha = shas[ctx.core.rng.gen_range(shas.len() as u32) as usize];
        let target = enemies[ctx.core.rng.gen_range(enemies.len() as u32) as usize];
        ctx.out.push(LogEvent::SkillTriggered {
            skill: ctx.catalog.skills[def as usize].name,
            owner,
            presentation: None,
        });
        let cur_armor = i32::from(ctx.core.player(owner).hujia);
        ctx.core.player_mut(owner).hujia = (cur_armor + 1).min(99) as u8;
        set_flag(ctx.core, owner, &format!("xj_xiaoyue_card:{sha}"), 1);
        ensure_temp_skill(ctx, target, "xiaoyue2", true);
        ctx.emit(EventData::CardUsed(sha_card_use_data(ctx, owner, sha, vec![target], None)));
    }
}

pub(crate) fn host_play_begin_xianjian(ctx: &mut Ctx<'_>, player: PlayerId) {
    let seats = ctx.core.players.len() as PlayerId;
    for p in 0..seats {
        if flag(ctx.core, p, "xj_leiyu_marked") == u32::from(player) + 1 {
            set_flag(ctx.core, p, "xj_leiyu_marked", 0);
        }
    }
}

pub(crate) fn host_card_use_after_xianjian(ctx: &mut Ctx<'_>, u: &CardUseData) {
    let owner = u.player;
    let eff_kind = u.effective_kind(ctx.catalog);
    if let Some(cid) = u.card {
        set_flag(ctx.core, owner, &format!("xj_tianjian_card:{cid}"), 0);
        set_flag(ctx.core, owner, &format!("xj_qianfang_card:{cid}"), 0);
        set_flag(ctx.core, owner, &format!("xj_yanzhan_src_card:{cid}"), 0);
        set_flag(ctx.core, owner, &format!("xj_yanzhan_card:{cid}"), 0);
        set_flag(ctx.core, owner, &format!("xj_xiaoyue_card:{cid}"), 0);
    }
    if !u.cancelled && ctx.core.player(owner).alive {
        let non_locked_ok = !ctx.catalog.is_tieji_sealed(ctx.core, owner)
            && !ctx.catalog.is_xj_fengyin_sealed(ctx.core, owner);
        if non_locked_ok && has_skill_named(ctx.core, ctx.catalog, owner, "yujia") && eff_kind.is_jiguan_kind() {
            let prev = flag(ctx.core, owner, "xj_yujia_count");
            set_flag(ctx.core, owner, "xj_yujia_count", prev + 1);
        }
        if has_skill_named(ctx.core, ctx.catalog, owner, "yanshi")
            && eff_kind.is_jiguan_kind()
            && flag(ctx.core, owner, "xj_yanshi_done") == 0
            && flag(ctx.core, owner, "xj_yanshi2_turn") == 0
        {
            let prev = flag(ctx.core, owner, "xj_yanshi_count");
            set_flag(ctx.core, owner, "xj_yanshi_count", prev + 1);
            set_flag(ctx.core, owner, "xj_yanshi2_turn", 1);
            ensure_temp_skill(ctx, owner, "yanshi2", true);
        }
        if non_locked_ok {
            if let Some(cid) = u.card {
                let suit_b = suit_bit(ctx.catalog.effective_card_suit(ctx.core, owner, cid));
                if has_skill_named(ctx.core, ctx.catalog, owner, "qixia") {
                    let prev = flag(ctx.core, owner, "xj_qixia_suits");
                    set_flag(ctx.core, owner, "xj_qixia_suits", prev | suit_b);
                }
                if has_skill_named(ctx.core, ctx.catalog, owner, "danqing")
                    || has_skill_named(ctx.core, ctx.catalog, owner, "danqing_count")
                {
                    let prev = flag(ctx.core, owner, "xj_danqing_suits");
                    set_flag(ctx.core, owner, "xj_danqing_suits", prev | suit_b);
                }
            }
        }
    }
    if ctx.core.turn.current == owner
        && (has_skill_named(ctx.core, ctx.catalog, owner, "leiyu") || has_skill_named(ctx.core, ctx.catalog, owner, "leiyu4"))
    {
        for &t in &u.targets {
            if t != owner {
                set_flag(ctx.core, t, "xj_leiyu_marked", u32::from(owner) + 1);
            }
        }
    }
}
