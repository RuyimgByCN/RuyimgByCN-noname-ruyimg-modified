use super::*;

// 同疾（袁术，P2/N08 标准缺失将收尾批 2026-10-07）：锁定技——其他
// 角色使用【杀】指定唯一目标（你不在目标内、目标对你 inRange）时，
// 目标须选择弃置一张牌或取消：弃置则该杀转移指向你（改写 CardUsed
// 帧载荷，sha 结算机器在 Intrinsic 阶段才读取目标）。
#[derive(Clone)]
pub(in super::super) struct TongJiBehavior;

impl SkillBehavior for TongJiBehavior {
    fn filter(&self, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        let matches_sha = matches!(event, EventData::CardUsed(u)
        if {
            let kind = u.use_as.unwrap_or_else(|| ctx.catalog.card_kind(u.card));
            matches!(kind, crate::catalog::CardKind::Sha | crate::catalog::CardKind::FireSha | crate::catalog::CardKind::LeiSha)
                && u.targets.len() == 1
                && u.player != owner
                && !u.targets.contains(&owner)
        });
        if !matches_sha {
            return false;
        }
        let EventData::CardUsed(u) = event else {
            return false;
        };
        let target = u.targets[0];
        let owner_alive = ctx.state.player(owner).alive;
        let target_ok = ctx.state.player(target).alive
            && (!ctx.state.player(target).hand.is_empty()
                || !ctx.state.player(target).equips.is_empty());
        let in_range = ctx.state.computed_distance(ctx.catalog, target, owner)
            <= ctx.state.attack_range(ctx.catalog, target);
        owner_alive && target_ok && in_range
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(TongJiMachine {
            owner,
            stage: TongJiStage::Ask,
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(in super::super) enum TongJiStage {
    /// 目标弃牌/取消（挂起 = tongji_transfer_discard）。
    Ask,
    /// 恢复读应答：弃置则转移目标。
    Read,
}

#[derive(Clone)]
pub(in super::super) struct TongJiMachine {
    pub(in super::super) owner: PlayerId,
    pub(in super::super) stage: TongJiStage,
}

impl SkillMachine for TongJiMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::TongJi {
            owner: self.owner,
            stage: match self.stage {
                TongJiStage::Ask => TongJiStageSnap::Ask,
                TongJiStage::Read => TongJiStageSnap::Read,
            },
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.stage {
            TongJiStage::Ask => {
                let Ok(u) = ctx.current_card_use() else {
                    return Flow::Done;
                };
                let target = u.targets[0];
                let mut options: Vec<ChoiceOption> = ctx
                    .core
                    .player(target)
                    .hand
                    .iter()
                    .copied()
                    .map(ChoiceOption::Card)
                    .collect();
                options.extend(
                    ctx.core
                        .player(target)
                        .equips
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card),
                );
                if options.is_empty() {
                    return Flow::Done;
                }
                self.stage = TongJiStage::Read;
                Flow::WaitChoice {
                    observer: target,
                    spec: crate::choice::ChoiceSpec {
                        kind: crate::choice::ChoiceKind::SelectCards,
                        prompt: "tongji_transfer_discard",
                        options,
                        min: 0,
                        max: 1,
                        allow_cancel: true,
                    },
                }
            }
            TongJiStage::Read => {
                let Ok(u) = ctx.current_card_use() else {
                    return Flow::Done;
                };
                let target = u.targets[0];
                // 空应答 = 目标取消（不弃牌不转移）。
                if let Some(ChoiceOption::Card(card)) = ctx.answer().first().copied() {
                    let hand = ctx.core.player(target).hand.contains(&card);
                    if hand {
                        ctx.discard_from_hand(target, card);
                    } else if let Some(pos) = ctx
                        .core
                        .player(target)
                        .equips
                        .iter()
                        .position(|c| *c == card)
                    {
                        let equip = ctx.core.player_mut(target).equips.remove(pos);
                        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, target, equip);
                        ctx.to_discard(equip);
                    } else {
                        return Flow::Done;
                    }
                    // 转移：改写 CardUsed 帧载荷目标（sha 结算尚未开始）。
                    let u = ctx.current_card_use_mut();
                    u.targets[0] = self.owner;
                }
                Flow::Done
            }
        }
    }
}
