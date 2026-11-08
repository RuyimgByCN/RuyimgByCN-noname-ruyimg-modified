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
            let shas: Vec<crate::ids::CardId> =
                response_kind_cards(ctx, responder, CardKind::Sha);
            if !self.asked {
                if shas.is_empty() && ctx.core.player(responder).alive {
                    if let Some((helper, helper_shas)) =
                        jijiang_shu_helper(ctx.core, ctx.catalog, responder)
                    {
                        self.asked = true;
                        let mut options = vec![ChoiceOption::Pass];
                        options.extend(helper_shas.into_iter().map(ChoiceOption::Card));
                        return Flow::WaitChoice {
                            observer: helper,
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
                }
                if shas.is_empty() || !ctx.core.player(responder).alive {
                    // 轮到方无杀：不询问，直接受伤，决斗结束
                    let vm_card_bonus = ctx
                        .core
                        .vm_aux
                        .flags
                        .get(&(self.source, format!("card_damage_bonus:{}", self.card)))
                        .copied()
                        .unwrap_or(0);
                    ctx.emit(EventData::Damage(damage_data(
                        Some(opponent),
                        responder,
                        (1 + luoyi_bonus(ctx, Some(opponent)) + vm_card_bonus) as i32,
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
                let jijiang_helper = if shas.is_empty() {
                    jijiang_shu_helper(ctx.core, ctx.catalog, responder).map(|(h, _)| h)
                } else {
                    None
                };
                ctx.core
                    .vm_aux
                    .flags
                    .remove(&(responder, "duel_jijiang_skip".to_string()));
                let actual_holder = jijiang_helper.unwrap_or_else(|| {
                    if ctx.core.player(responder).hand.contains(&c)
                        || ctx.core.player(responder).equips.contains(&c)
                    {
                        responder
                    } else {
                        (0..ctx.core.players.len() as u8)
                            .find(|&p| {
                                ctx.core.player(p).hand.contains(&c)
                                    || ctx.core.player(p).equips.contains(&c)
                            })
                            .unwrap_or(responder)
                    }
                });
                discard_response_card(ctx, actual_holder, c, CardKind::Sha);
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
            // 激将代打逐人推进：当前蜀势力角色放弃时，继续询问下一位蜀势力角色
            if shas.is_empty() && ctx.core.player(responder).alive {
                if let Some((helper, _)) = jijiang_shu_helper(ctx.core, ctx.catalog, responder) {
                    *ctx.core
                        .vm_aux
                        .flags
                        .entry((responder, "duel_jijiang_skip".to_string()))
                        .or_insert(0) |= 1u32 << helper;
                    continue;
                }
                ctx.core
                    .vm_aux
                    .flags
                    .remove(&(responder, "duel_jijiang_skip".to_string()));
            }
            // 放弃 → 受伤，决斗结束（重置无双标记）
            self.second_pending = false;
            let vm_card_bonus = ctx
                .core
                .vm_aux
                .flags
                .get(&(self.source, format!("card_damage_bonus:{}", self.card)))
                .copied()
                .unwrap_or(0);
            ctx.emit(EventData::Damage(damage_data(
                Some(opponent),
                responder,
                (1 + luoyi_bonus(ctx, Some(opponent)) + vm_card_bonus) as i32,
            )));
            return self.finish(ctx);
        }
    }
}

impl DuelMachine {
    fn finish(&self, ctx: &mut Ctx<'_>) -> Flow {
        let card = self.card;
        ctx.core
            .vm_aux
            .flags
            .remove(&(self.source, format!("card_damage_bonus:{card}")));
        ctx.core
            .vm_aux
            .flags
            .remove(&(self.source, "duel_jijiang_skip".to_string()));
        ctx.core
            .vm_aux
            .flags
            .remove(&(self.target, "duel_jijiang_skip".to_string()));
        if card != 0 {
            ctx.to_discard(card);
            ctx.core.table.retain(|c| *c != card);
        }
        Flow::Done
    }
}

pub(crate) fn jijiang_shu_helper(
    core: &crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
    responder: PlayerId,
) -> Option<(PlayerId, Vec<crate::ids::CardId>)> {
    let has_jijiang = catalog
        .skill_by_name(crate::vocabulary::names::JI_JIANG)
        .map(|d| core.has_skill(responder, d))
        .unwrap_or(false)
        && core.is_lord(responder);
    if !has_jijiang {
        return None;
    }
    let skip = core
        .vm_aux
        .flags
        .get(&(responder, "duel_jijiang_skip".to_string()))
        .copied()
        .unwrap_or(0);
    let n = core.players.len() as u8;
    for offset in 1..n {
        let p = (responder + offset) % n;
        if (skip & (1u32 << p)) != 0
            || !core.player(p).alive
            || core.effective_faction(catalog, p) != Some(crate::catalog::Faction::Shu)
            || catalog.blocks_all_cards(core, p)
        {
            continue;
        }
        let mut shas: Vec<crate::ids::CardId> = core
            .player(p)
            .hand
            .iter()
            .copied()
            .filter(|c| {
                catalog.card_kind(*c).is_sha_like()
                    || catalog
                        .find_player_vm_view_as(core, p, *c, false, Some(CardKind::Sha))
                        .is_some()
            })
            .collect();
        for c in catalog.player_vm_view_as_extra_cards(core, p, false, Some(CardKind::Sha)) {
            if !shas.contains(&c) {
                shas.push(c);
            }
        }
        if !shas.is_empty() {
            return Some((p, shas));
        }
    }
    None
}

