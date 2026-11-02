use super::*;

/// E03 猴子偷桃：使用前取消窗口（命令层挂起机器，ActivateSkill 槽模式）。
/// 他人出桃且装备猴子时拦截提交：逐持有者确认——确认者弃猴子并把桃收为
/// 手牌（使用不发生，无 CardUsed 帧）；全部放弃则按普通桃结算（回血、
/// 入弃牌堆）。持有者死亡或失猴即跳过。
#[derive(Clone)]
pub struct MonkeyStealMachine {
    pub card: crate::ids::CardId,
    pub user: PlayerId,
    pub holders: Vec<PlayerId>,
    pub asked: bool,
}

impl MonkeyStealMachine {
    pub fn new(card: crate::ids::CardId, user: PlayerId, holders: Vec<PlayerId>) -> Self {
        Self {
            card,
            user,
            holders,
            asked: false,
        }
    }
}

impl crate::catalog::SkillMachine for MonkeyStealMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> crate::snap::MachineSnapshot {
        crate::snap::MachineSnapshot::MonkeySteal {
            card: self.card,
            user: self.user,
            holders: self.holders.clone(),
            asked: self.asked,
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        loop {
            // 跳过死亡或已失去猴子的持有者。
            while let Some(h) = self.holders.first().copied() {
                let def = ctx
                    .catalog
                    .skill_by_name(crate::vocabulary::host::HUANLE_MONKEY_KEY);
                let armed = def.is_some_and(|d| ctx.core.has_skill(h, d));
                if !ctx.core.player(h).alive || !armed {
                    self.holders.remove(0);
                } else {
                    break;
                }
            }
            let Some(holder) = self.holders.first().copied() else {
                // 全部放弃：按普通桃结算（回血、入弃牌堆）。
                ctx.to_discard(self.card);
                ctx.recover(self.user, 1, Some(self.card));
                return Flow::Done;
            };
            if self.asked {
                self.asked = false;
                self.holders.remove(0);
                let confirmed = ctx
                    .answer()
                    .first()
                    .map(|o| *o == ChoiceOption::Confirm)
                    .unwrap_or(false);
                if confirmed {
                    // 取消使用：桃从使用者手牌收为持有者手牌，弃猴子
                    //（离场撤销装备技）。
                    let user_hand = &mut ctx.core.players[self.user as usize].hand;
                    let pos = user_hand
                        .iter()
                        .position(|c| *c == self.card)
                        .expect("被拦截的桃应在使用者手牌");
                    let peach = user_hand.remove(pos);
                    ctx.core.players[holder as usize].hand.push(peach);
                    ctx.log(LogEvent::CardsGivenPublic {
                        from: self.user,
                        to: holder,
                        cards: vec![peach],
                    });
                    if let Some(pos) = ctx.core.players[holder as usize]
                        .equips
                        .iter()
                        .position(|c| ctx.catalog.card_kind(*c) == CardKind::Monkey)
                    {
                        let monkey = ctx.core.players[holder as usize].equips.remove(pos);
                        revoke_equip_skills(&mut ctx.core, ctx.catalog, holder, monkey);
                        ctx.core.discard.push(monkey);
                        ctx.log(LogEvent::CardDiscarded {
                            player: holder,
                            cards: vec![monkey],
                        });
                    }
                    return Flow::Done; // 使用被取消：无 CardUsed 帧。
                }
                continue; // 放弃 → 队列下一持有者。
            }
            self.asked = true;
            return Flow::WaitChoice {
                observer: holder,
                spec: ChoiceSpec {
                    kind: ChoiceKind::RespondCard,
                    prompt: "huanle_monkey_confirm",
                    options: vec![ChoiceOption::Confirm, ChoiceOption::Pass],
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            };
        }
    }
}
