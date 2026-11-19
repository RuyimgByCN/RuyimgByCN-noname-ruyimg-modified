use super::super::*;
use crate::catalog::{Catalog, EquipSlot};
use crate::snap::{ask, ExpectedAsk};
use crate::state::GameStateCore;

impl HostMachine {
    /// 欢乐卡牌（U06）：猴子偷桃（useCardToBegin 使用前取消）/ 漩涡面具（翻面取消）。
    pub(super) fn step_huanle(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            HostBehavior::HuanleMonkeyCard => match self.stage {
                HostStage::Act => {
                    let Some((frame_id, user, card, cancelled, is_tao)) =
                        ctx.res.stack.last().and_then(|frame| match &frame.data {
                            EventData::CardUsed(u) => Some((
                                frame.id,
                                u.player,
                                u.card,
                                u.cancelled,
                                u.use_as.unwrap_or_else(|| u.effective_kind(ctx.catalog))
                                    == CardKind::Tao,
                            )),
                            _ => None,
                        })
                    else {
                        return Flow::Done;
                    };
                    if cancelled
                        || !is_tao
                        || user == self.owner
                        || !ctx.core.player(self.owner).alive
                    {
                        return Flow::Done;
                    }
                    let has_monkey = ctx.core.player(self.owner).equips.iter().any(|c| {
                        ctx.catalog.card_slot_of(ctx.core, *c) == Some(EquipSlot::Treasure)
                            && ctx.catalog.card_kind(*c) == CardKind::Monkey
                    });
                    if !has_monkey {
                        return Flow::Done;
                    }
                    let has_table_carrier = card
                        .into_iter()
                        .chain(crate::kernel::peek_tao_extra_carriers(
                            ctx.core, user, frame_id,
                        ))
                        .any(|c| ctx.core.table.contains(&c));
                    if !has_table_carrier {
                        return Flow::Done;
                    }
                    ctx.core
                        .vm_aux
                        .flags
                        .insert((self.owner, format!("monkey_asked:{frame_id}")), 1);
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::RespondCard,
                            prompt: "huanle_monkey_confirm",
                            options: vec![ChoiceOption::Confirm, ChoiceOption::Pass],
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    if ctx.answer().first().copied() != Some(ChoiceOption::Confirm) {
                        return Flow::Done;
                    }
                    let Some((frame_id, user, card, cancelled)) =
                        ctx.res.stack.last().and_then(|frame| match &frame.data {
                            EventData::CardUsed(u) => {
                                Some((frame.id, u.player, u.card, u.cancelled))
                            }
                            _ => None,
                        })
                    else {
                        return Flow::Done;
                    };
                    if cancelled {
                        return Flow::Done;
                    }
                    let mut in_d_cards = Vec::new();
                    for c in card
                        .into_iter()
                        .chain(crate::kernel::peek_tao_extra_carriers(
                            ctx.core, user, frame_id,
                        ))
                    {
                        if ctx.core.table.contains(&c) && !in_d_cards.contains(&c) {
                            in_d_cards.push(c);
                        }
                    }
                    if in_d_cards.is_empty() {
                        return Flow::Done;
                    }
                    let Some(pos) = ctx.core.players[self.owner as usize]
                        .equips
                        .iter()
                        .position(|c| {
                            ctx.catalog.card_slot_of(ctx.core, *c) == Some(EquipSlot::Treasure)
                                && ctx.catalog.card_kind(*c) == CardKind::Monkey
                        })
                    else {
                        return Flow::Done;
                    };
                    let _ = crate::kernel::take_tao_extra_carriers(ctx.core, user, frame_id);
                    if let Some(def) = ctx.catalog.skill_by_name(&self.def_key) {
                        let sname = ctx.catalog.skills[def as usize].name;
                        ctx.log_skill_triggered(sname, self.owner, def);
                    }
                    ctx.core.table.retain(|c| !in_d_cards.contains(c));
                    ctx.core.players[self.owner as usize]
                        .hand
                        .extend_from_slice(&in_d_cards);
                    ctx.log(LogEvent::CardsGivenPublic {
                        from: user,
                        to: self.owner,
                        cards: in_d_cards,
                    });
                    let monkey = ctx.core.players[self.owner as usize].equips.remove(pos);
                    ctx.core.declared_slots.remove(&monkey);
                    crate::kernel::revoke_equip_skills(
                        &mut ctx.core,
                        ctx.catalog,
                        self.owner,
                        monkey,
                    );
                    ctx.core.discard.push(monkey);
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![monkey],
                    });
                    ctx.core
                        .vm_aux
                        .flags
                        .remove(&(user, "vm_va_recover_bonus".to_string()));
                    ctx.current_card_use_mut().cancelled = true;
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::HuanleMianjuCard => {
                // 强制锁定：写回翻面取消位，提交阶段跳过翻转。
                let (_p, c) = ctx.current_turn_over_before_mut();
                *c = true;
                Flow::Done
            }
            _ => unreachable!("behavior routed to the wrong step module"),
        }
    }
}

/// 欢乐卡牌（U06）宿主询问派生（猴子偷桃确认 / 写轮眼目标与技能选择）。
pub(in crate::content::host) fn huanle_active_ask(
    behavior: HostBehavior,
    stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    match behavior {
        HostBehavior::HuanleMonkeyCard => {
            if stage != HostStage::SelectCard {
                return None;
            }
            Some(ask(
                owner,
                "huanle_monkey_confirm",
                ChoiceKind::RespondCard,
                vec![ChoiceOption::Confirm, ChoiceOption::Pass],
                1,
                1,
                false,
            ))
        }
        HostBehavior::HuanleXuelunyangCard => match stage {
            HostStage::CompareTargetPick => {
                let candidates: Vec<ChoiceOption> =
                    super::skills_targets::xuelunyang_targets(core, catalog, owner)
                        .into_iter()
                        .map(ChoiceOption::Player)
                        .collect();
                if candidates.is_empty() {
                    return None;
                }
                Some(ask(
                    owner,
                    "huanle_xuelunyang_target",
                    ChoiceKind::SelectPlayers,
                    candidates,
                    1,
                    1,
                    true,
                ))
            }
            HostStage::ComparePlayerPick => {
                let t = target?;
                let count =
                    super::skills_targets::xuelunyang_stealable_skills(core, catalog, owner, t)
                        .len();
                if count == 0 {
                    return None;
                }
                let tags: Vec<ChoiceOption> = (0..count).map(|i| ChoiceOption::Tag(i as u8)).collect();
                Some(ask(
                    owner,
                    "huanle_xuelunyang_skill",
                    ChoiceKind::PickTag,
                    tags,
                    1,
                    1,
                    true,
                ))
            }
            _ => None,
        },
        _ => None,
    }
}
