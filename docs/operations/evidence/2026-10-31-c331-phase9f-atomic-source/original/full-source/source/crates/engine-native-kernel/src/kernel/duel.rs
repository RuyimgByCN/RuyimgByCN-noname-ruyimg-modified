use super::*;

/// 决斗：目标先出杀，双方交替出杀；轮到某方其无杀（不询问）或放弃时，
/// 该方受对方造成的 1 点伤害，决斗结束。响应杀不占出牌阶段限一次；
/// 无距离限制——轮转起手/跳过语义/来源归属见覆盖清单行为差异表。
#[derive(Clone)]
pub struct DuelMachine {
    pub(super) card: crate::ids::CardId,
    pub(super) source: PlayerId,
    pub(super) target: PlayerId,
    /// true = 轮到目标出杀；false = 轮到来源出杀。
    pub(super) on_target: bool,
    pub(super) asked: bool,
    /// 无双：对手需再次出杀（本轮已出一张）。
    pub(super) second_pending: bool,
}

impl DuelMachine {
    pub fn new(
        card: crate::ids::CardId,
        source: PlayerId,
        target: PlayerId,
        on_target: bool,
        asked: bool,
        second_pending: bool,
    ) -> Self {
        Self {
            card,
            source,
            target,
            on_target,
            asked,
            second_pending,
        }
    }
}

impl crate::catalog::SkillMachine for DuelMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }

    fn snapshot(&self) -> crate::snap::MachineSnapshot {
        crate::snap::MachineSnapshot::Duel {
            card: self.card,
            source: self.source,
            target: self.target,
            on_target: self.on_target,
            asked: self.asked,
            second_pending: self.second_pending,
        }
    }

    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        loop {
            let responder = if self.on_target {
                self.target
            } else {
                self.source
            };
            let opponent = if self.on_target {
                self.source
            } else {
                self.target
            };
            let shas: Vec<crate::ids::CardId> = ctx
                .core
                .player(responder)
                .hand
                .iter()
                .copied()
                .filter(|c| ctx.catalog.card_kind(*c) == CardKind::Sha)
                .collect();
            if !self.asked {
                if shas.is_empty() || !ctx.core.player(responder).alive {
                    // 轮到方无杀：不询问，直接受伤，决斗结束
                    ctx.emit(EventData::Damage(damage_data(
                        Some(opponent),
                        responder,
                        (1 + luoyi_bonus(ctx, Some(opponent))) as i32,
                    )));
                    return self.finish(ctx);
                }
                self.asked = true;
                let mut options = vec![ChoiceOption::Pass];
                if !ctx.catalog.blocks_all_cards(ctx.core, responder) {
                    options.extend(shas.into_iter().map(ChoiceOption::Card));
                    // 神弓（swd 批 148）：借用他人武器作为杀
                    options.extend((ctx.catalog.hooks.borrow_options)(
                        ctx, responder, false,
                    ));
                }
                return Flow::WaitChoice {
                    observer: responder,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::RespondCard,
                        prompt: "duel_sha",
                        options,
                        min: 1,
                        max: 1,
                        allow_cancel: false,
                    },
                };
            }
            // 应答消费
            self.asked = false;
            let answer = ctx.answer().clone();
            if let Some(ChoiceOption::Card(c)) = answer.first().copied() {
                ctx.discard_from_hand(responder, c);
                ctx.emit(EventData::CardResponded {
                    card: c,
                    player: responder,
                });
                // 无双：与无双持有者决斗的角色每次需连续打出两张杀——
                // 第一张后不轮转，同一响应者再次询问（second_pending）
                let wushuang_holder = if self.on_target {
                    self.source
                } else {
                    self.target
                };
                let opponent_has_wushuang = ctx
                    .catalog
                    .skill_by_name(crate::vocabulary::names::WU_SHUANG)
                    .map(|d| ctx.core.has_skill(wushuang_holder, d))
                    .unwrap_or(false);
                if opponent_has_wushuang && !self.second_pending {
                    self.second_pending = true;
                    continue; // 同一响应者再次询问第二张杀
                }
                self.second_pending = false;
                self.on_target = !self.on_target;
                continue;
            }
            // 放弃 → 受伤，决斗结束（重置无双标记）
            self.second_pending = false;
            ctx.emit(EventData::Damage(damage_data(
                Some(opponent),
                responder,
                (1 + luoyi_bonus(ctx, Some(opponent))) as i32,
            )));
            return self.finish(ctx);
        }
    }
}

impl DuelMachine {
    fn finish(&self, ctx: &mut Ctx<'_>) -> Flow {
        let card = self.card;
        ctx.to_discard(card);
        ctx.core.table.retain(|c| *c != card);
        Flow::Done
    }
}
