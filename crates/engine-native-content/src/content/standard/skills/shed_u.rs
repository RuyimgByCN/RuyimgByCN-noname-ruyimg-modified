use super::*;

/// 蛇毒标记结算（Boss 相柳毒妒授予，规则 v223；A2 S14）：持有标记>0
/// 的角色**回合开始**（上游 `{player:'phaseBegin'}`＝TurnBegin 窗；原锚
/// 判定阶段开始）——「弃置 X 张牌，或失去 X 点体力」（挂起 SelectCards；
/// 取消/空选=失 X）→ 标记-1；X=当前标记数。
pub(in super::super) struct ShedUBehavior;

impl SkillBehavior for ShedUBehavior {
    fn filter(&self, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::TurnBegin { player }
            if *player == owner
                && ctx.state.player(owner).shedu > 0)
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(ShedUMachine {
            owner,
            asked: false,
        })
    }
}

#[derive(Clone)]
pub(in super::super) struct ShedUMachine {
    pub(in super::super) owner: PlayerId,
    pub(in super::super) asked: bool,
}

impl SkillMachine for ShedUMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::Shedu { owner: self.owner }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        if self.asked {
            // 消费：非空选=弃所选牌（各-1 张？逐张去手）；空选=失 X 体力
            let x = ctx.core.player(self.owner).shedu as usize;
            let chosen: Vec<crate::ids::CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Card(c) => Some(*c),
                    _ => None,
                })
                .collect();
            if chosen.is_empty() {
                ctx.emit(EventData::LoseHp {
                    player: self.owner,
                    amount: x as u32,
                });
            } else {
                for c in chosen {
                    ctx.discard_from_hand(self.owner, c);
                }
            }
            let p = ctx.core.player_mut(self.owner);
            p.shedu = p.shedu.saturating_sub(1);
            return Flow::Done;
        }
        self.asked = true;
        let x = ctx.core.player(self.owner).shedu as usize;
        if x == 0 {
            return Flow::Done;
        }
        let options: Vec<ChoiceOption> = ctx
            .core
            .player(self.owner)
            .hand
            .iter()
            .copied()
            .map(ChoiceOption::Card)
            .collect();
        Flow::WaitChoice {
            observer: self.owner,
            spec: ChoiceSpec {
                kind: ChoiceKind::SelectCards,
                prompt: "shedu_discard_or_lose",
                options,
                min: 0,
                max: x,
                allow_cancel: true, // 空选=失去 X 点体力
            },
        }
    }
}
