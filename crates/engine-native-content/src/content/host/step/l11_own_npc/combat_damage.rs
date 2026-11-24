//! L11-6a 太虚自有 NPC 伤害与防伤/判定类技能：`zuijiu`/`baoji`/`kuangbao`/`shiyuan`/`wangyuan`/`guiyan`/`xiefeng`/`huangjin`。

use super::*;
use crate::catalog::CardKind;
use crate::choice::ChoiceOption;
use crate::event::{DamageNature, EventData};

pub(super) fn handle_combat_damage(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Option<Flow> {
    let owner = m.owner;
    match m.def_key.as_str() {
        "txhj_zuijiu" => {
            with_current_damage_mut(ctx, |dm| dm.amount = dm.amount.saturating_add(1));
            ctx.log(LogEvent::SkillTriggered {
                skill: "醉酒",
                owner,
                presentation: None,
            });
            Some(Flow::Done)
        }
        "txhj_baoji" => {
            if ctx.core.rng.gen_range(100) < 40 {
                with_current_damage_mut(ctx, |dm| dm.amount = dm.amount.saturating_add(1));
                ctx.log(LogEvent::SkillTriggered {
                    skill: "暴击",
                    owner,
                    presentation: None,
                });
            }
            Some(Flow::Done)
        }
        "txhj_kuangbao" => {
            if m.card == Some(MODE_SECONDARY) {
                set_flag(ctx.core, owner, "l11_kuangbao_add", 0);
                set_flag(ctx.core, owner, "l11_kuangbao_debuff", 1);
            } else {
                let bonus = flag(ctx.core, owner, "l11_kuangbao_bonus");
                if bonus > 0 {
                    with_current_damage_mut(ctx, |dm| dm.amount = dm.amount.saturating_add(bonus as i32));
                }
            }
            Some(Flow::Done)
        }
        "txhj_shiyuan" => {
            with_current_damage_mut(ctx, |dm| dm.amount = dm.amount.saturating_add(1));
            Some(Flow::Done)
        }
        "txhj_wangyuan" => {
            let bonus = flag(ctx.core, owner, "l11_wangyuan_marks");
            if bonus > 0 {
                with_current_damage_mut(ctx, |dm| dm.amount = dm.amount.saturating_add(bonus as i32));
            }
            Some(Flow::Done)
        }
        "txhj_guiyan" => Some(handle_guiyan(m, ctx)),
        "txhj_xiefeng" => Some(handle_xiefeng(m, ctx)),
        "txhj_huangjin" => Some(handle_huangjin(m, ctx)),
        _ => None,
    }
}

fn handle_guiyan(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(MODE_SECONDARY) {
        with_current_damage_mut(ctx, |dm| {
            dm.nature = DamageNature::Fire;
            dm.amount = dm.amount.saturating_add(1);
        });
        ctx.log(LogEvent::SkillTriggered {
            skill: "诡炎",
            owner,
            presentation: None,
        });
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_GUIYAN_TARGET),
        HostStage::TargetChoice => {
            m.card = None;
            let own_hp = ctx.core.player(owner).hp;
            let Some(t) = answered_players(ctx)
                .into_iter()
                .next()
                .filter(|&p| p != owner && ctx.core.player(p).alive && ctx.core.player(p).hp >= own_hp)
            else {
                return Flow::Done;
            };
            ctx.log(LogEvent::SkillTriggered {
                skill: "诡炎",
                owner,
                presentation: None,
            });
            set_flag(ctx.core, owner, "l11_guiyan_target", u32::from(t) + 1);
            set_flag(ctx.core, owner, "l11_guiyan_turn", turn_stamp(ctx.core));
            if !has_named_skill(ctx.core, ctx.catalog, t, "ranshang") {
                ensure_perm_skill(ctx, t, "ranshang");
            } else if !crate::rvm_bridge::modifiers::eval_card_target_canceled_opt(
                ctx.core,
                ctx.catalog,
                owner,
                t,
                None,
                CardKind::Wuzhong,
                DamageNature::Fire,
            ) {
                set_flag(ctx.core, owner, VIRTUAL_HUOGONG_FLAG, 1);
                m.stage = HostStage::DrawSettle;
                ctx.emit(EventData::CardUsed(crate::event::CardUseData {
                    card: None,
                    player: owner,
                    targets: vec![t],
                    nature: DamageNature::Fire,
                    use_as: Some(CardKind::Wuzhong),
                    cancelled: false,
                }));
                return Flow::Yield;
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn handle_xiefeng(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(MODE_SHIELD) {
        with_current_damage_mut(ctx, |dm| {
            dm.prevented = true;
            dm.amount = 0;
        });
        return Flow::Done;
    }
    if m.card == Some(MODE_TICK) {
        let stacks = flag(ctx.core, owner, "l11_xiefeng_stacks");
        if stacks > 0 {
            set_flag(ctx.core, owner, "l11_xiefeng_stacks", stacks - 1);
        }
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::SelectCard, ASK_XIEFENG_DISCARD),
        HostStage::SelectCard => {
            m.card = None;
            let picked: Vec<CardId> = answered_cards(ctx)
                .into_iter()
                .filter(|c| ctx.core.player(owner).hand.contains(c) || ctx.core.player(owner).equips.contains(c))
                .take(3)
                .collect();
            if picked.is_empty() {
                return Flow::Done;
            }
            let cnt = picked.len() as u32;
            ctx.log(LogEvent::SkillTriggered {
                skill: "邪风",
                owner,
                presentation: None,
            });
            discard_player_cards(ctx, owner, &picked);
            let cur = flag(ctx.core, owner, "l11_xiefeng_stacks");
            set_flag(ctx.core, owner, "l11_xiefeng_stacks", cur + cnt);
            m.stage = HostStage::DrawSettle;
            ctx.start_draw(owner, 1);
            Flow::Yield
        }
        _ => Flow::Done,
    }
}

fn handle_huangjin(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::CardsAsk, ASK_HUANGJIN_CONFIRM),
        HostStage::CardsAsk => {
            m.card = None;
            if !is_confirmed(ctx) {
                return Flow::Done;
            }
            ctx.log(LogEvent::SkillTriggered {
                skill: "黄巾",
                owner,
                presentation: None,
            });
            m.stage = HostStage::SelectNature;
            ctx.emit(EventData::OrdinaryJudge {
                owner,
                invocation: u32::MAX,
            });
            Flow::Yield
        }
        HostStage::SelectNature => {
            let _ = flag(ctx.core, owner, "last_judge_suit");
            let j_rank = flag(ctx.core, owner, "last_judge_rank");
            set_flag(ctx.core, owner, "last_judge_suit", 0);
            set_flag(ctx.core, owner, "last_judge_rank", 0);
            set_flag(ctx.core, owner, "last_judge_card", 0);
            let sha_rank = m
                .cards
                .first()
                .copied()
                .or_else(|| current_use_card_mut(ctx).and_then(|u| u.card))
                .and_then(|c| ctx.catalog.resolved_card_def(ctx.core, c))
                .map_or(0, |d| d.rank);
            if sha_rank > 0 && j_rank > 0 && (i32::from(sha_rank) - j_rank as i32).abs() <= 1 {
                let mut discarded = None;
                if let Some(u) = current_use_card_mut(ctx) {
                    u.targets.clear();
                    u.cancelled = true;
                    discarded = u.card;
                }
                if let Some(cid) = discarded {
                    finish_table_card(ctx, cid);
                }
            }
            let _ = ChoiceOption::Confirm;
            Flow::Done
        }
        _ => Flow::Done,
    }
}
