use super::*;

// 丈八蛇矛（配套主动技，批③-b 第七把）：两张手牌当【杀】使用——
// 出牌阶段限一次不限（上游同款）；Stage 1 选两张手牌，Stage 2 选目标
//（射程内其他角色），首张作 declared-Sha 载荷进入杀结算，次张弃置。
#[derive(Clone)]
pub(in super::super) struct ZhangbaBehavior;

impl SkillBehavior for ZhangbaBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, _owner: PlayerId, _event: &EventData) -> bool {
        false
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(ZhangbaMachine {
            owner,
            cards: Vec::new(),
            stage: ZhangbaStage::Cards,
        })
    }

    fn activate(&self, ctx: &QueryCtx<'_>, owner: PlayerId) -> Option<Box<dyn SkillMachine>> {
        // 资格：两张手牌 + 攻击范围内有其他角色。
        if !ctx.state.player(owner).alive || ctx.state.player(owner).hand.len() < 2 {
            return None;
        }
        let in_range = ctx.state.alive_players().any(|p| {
            p != owner
                && ctx.state.computed_distance(ctx.catalog, owner, p)
                    <= ctx.state.attack_range(ctx.catalog, owner)
        });
        if !in_range {
            return None;
        }
        Some(Box::new(ZhangbaMachine {
            owner,
            cards: Vec::new(),
            stage: ZhangbaStage::Cards,
        }))
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(in super::super) enum ZhangbaStage {
    /// 选两张手牌（打开询问）。
    Cards,
    /// 选目标（打开询问）。
    Target,
    /// 读目标应答 → 发射 declared-Sha CardUsed。
    TargetRead,
}

#[derive(Clone)]
pub(in super::super) struct ZhangbaMachine {
    pub(in super::super) owner: PlayerId,
    pub(in super::super) cards: Vec<crate::ids::CardId>,
    pub(in super::super) stage: ZhangbaStage,
}

impl SkillMachine for ZhangbaMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::Zhangba {
            owner: self.owner,
            cards: self.cards.iter().copied().map(|c| c as u32).collect(),
            stage: match self.stage {
                ZhangbaStage::Cards => ZhangbaStageSnap::Cards,
                ZhangbaStage::Target => ZhangbaStageSnap::Target,
                ZhangbaStage::TargetRead => ZhangbaStageSnap::TargetRead,
            },
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.stage {
            ZhangbaStage::Cards => {
                let options: Vec<ChoiceOption> = ctx
                    .core
                    .player(self.owner)
                    .hand
                    .iter()
                    .copied()
                    .map(ChoiceOption::Card)
                    .collect();
                if options.len() < 2 {
                    return Flow::Done;
                }
                self.stage = ZhangbaStage::Target;
                Flow::WaitChoice {
                    observer: self.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "zhangba_pick_two",
                        options,
                        min: 2,
                        max: 2,
                        allow_cancel: false,
                    },
                }
            }
            ZhangbaStage::Target => {
                // 恢复：读两张手牌应答 → 打开目标询问。
                let picked: Vec<crate::ids::CardId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    })
                    .collect();
                if picked.len() != 2 {
                    return Flow::Done;
                }
                self.cards = picked;
                let candidates: Vec<ChoiceOption> = ctx
                    .core
                    .alive_players()
                    .filter(|p| {
                        *p != self.owner
                            && ctx.core.computed_distance(ctx.catalog, self.owner, *p)
                                <= ctx.core.attack_range(ctx.catalog, self.owner)
                    })
                    .map(ChoiceOption::Player)
                    .collect();
                if candidates.is_empty() {
                    return Flow::Done;
                }
                self.stage = ZhangbaStage::TargetRead;
                Flow::WaitChoice {
                    observer: self.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectPlayers,
                        prompt: "zhangba_sha_target",
                        options: candidates,
                        min: 1,
                        max: 1,
                        allow_cancel: false,
                    },
                }
            }
            ZhangbaStage::TargetRead => {
                if let Some(ChoiceOption::Player(target)) = ctx.answer().first().copied() {
                    if ctx.core.player(target).alive {
                        let [c1, c2] = self.cards.as_slice() else {
                            return Flow::Done;
                        };
                        let (c1, c2) = (*c1, *c2);
                        // 次张弃为转化素材；首张作 declared-Sha 载荷进入
                        // 杀结算（出闪/贯石斧/麒麟弓/青龙全流程重走）。
                        ctx.discard_from_hand(self.owner, c2);
                        if ctx.core.player(self.owner).hand.contains(&c1) {
                            ctx.core.declared_kinds.insert(c1, CardKind::Sha);
                            ctx.emit(EventData::CardUsed(crate::machine::card_use_as(
                                c1,
                                self.owner,
                                vec![target],
                                Some(CardKind::Sha),
                            )));
                        }
                    }
                }
                Flow::Done
            }
        }
    }
}
