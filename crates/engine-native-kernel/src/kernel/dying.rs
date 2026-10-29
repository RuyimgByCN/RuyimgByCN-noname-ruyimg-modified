use super::*;

/// 濒死救援：按座位顺序询问持桃者；无人救则进入死亡子事件。
#[derive(Clone)]
pub struct DyingSaveMachine {
    pub(super) dying: PlayerId,
    pub(super) source: Option<PlayerId>,
    pub(super) responders: Vec<PlayerId>,
    pub(super) asked: bool,
    /// 本轮是否已有人用桃（整轮无人用桃 → 濒死结算死亡，不再循环询问）。
    pub(super) round_used: bool,
    /// 归汉（玩点论杀批 17，规则 v280）：limited 确认询问挂起中。
    pub(super) guihan_asked: bool,
}

impl DyingSaveMachine {
    pub(crate) fn new(
        dying: PlayerId,
        source: Option<PlayerId>,
        responders: Vec<PlayerId>,
        asked: bool,
        round_used: bool,
    ) -> Self {
        Self {
            dying,
            source,
            responders,
            asked,
            round_used,
            guihan_asked: false,
        }
    }
}

impl crate::catalog::SkillMachine for DyingSaveMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> crate::snap::MachineSnapshot {
        crate::snap::MachineSnapshot::DyingSave {
            dying: self.dying,
            source: self.source,
            responders: self.responders.clone(),
            asked: self.asked,
            round_used: self.round_used,
            guihan_asked: self.guihan_asked,
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        // 刺猬（U07 玩点论杀韩龙，规则 v111）：濒死即死标记在位——跳
        // 过救援询问直接死亡（上游 ciqiu_dying dyingBegin firstDo die
        // 同口径）
        if ctx.core.turn.ciqiu_dying.contains(&self.dying) {
            ctx.core.turn.ciqiu_dying.retain(|p| *p != self.dying);
            ctx.emit(EventData::Death {
                player: self.dying,
                killer: self.source,
            });
            return Flow::Done;
        }
        // 归汉（玩点论杀批 17，规则 v280）：limited 濒死自救——求救循环
        // 前一次性确认（登记差异：上游 chooseToUse 玩家主动+自选男性
        // 目标，原生锁定确认+座位序首男性其他存活；limited 每局一次同
        // 口径）。确认后自回 1（救回由循环顶 hp>0 判定自然发出）摸 2，
        // 男性目标回 1 摸 2。
        if !self.guihan_asked {
            let gui = ctx
                .catalog
                .skill_by_name(crate::vocabulary::host::WUDIAN_GUIHAN_KEY)
                .map(|d| ctx.core.has_skill(self.dying, d))
                .unwrap_or(false);
            let unused = !ctx.core.player(self.dying).guihan_used;
            if gui && unused {
                self.guihan_asked = true;
                return Flow::WaitChoice {
                    observer: self.dying,
                    spec: crate::choice::ChoiceSpec::confirm("wudian_guihan_ask"),
                };
            }
        } else {
            self.guihan_asked = false;
            if let Some(crate::choice::ChoiceOption::Confirm) = ctx.answer().first().copied() {
                let t = self.dying;
                ctx.core.player_mut(t).guihan_used = true;
                ctx.recover(t, 1, None);
                ctx.draw_cards(t, 2);
                // 座位序首男性其他存活
                let male = (0..ctx.core.players.len() as u8).find(|&i| {
                    i != t
                        && ctx.core.player(i).alive
                        && ctx.core.effective_gender(i) == crate::vocabulary::generals::Gender::Male
                });
                if let Some(m) = male {
                    ctx.recover(m, 1, None);
                    ctx.draw_cards(m, 2);
                }
                // 回血后 hp>0 → 循环顶自然发出 DyingSaved
            }
        }
        loop {
            let p = ctx.core.player(self.dying);
            if p.hp > 0 && !p.dying {
                // 救援成功（dyingAfter 等价，规则 v157）：发射救回事件，
                // 觉醒技（姜城·焚逝）等触发挂在该事件的 AfterTriggers 窗
                // 口——时序在固有救援机之后，与上游 dyingAfter 一致。
                ctx.emit(EventData::DyingSaved {
                    player: self.dying,
                    source: self.source,
                });
                return Flow::Done; // 已被救回
            }
            // 跳过已死亡或无桃的应答者；队列耗尽即一轮问完——
            // 本轮有人用桃且场上仍有桃 → 开新一轮（负体力可由同一
            // 持桃者跨轮连续多张救回）；否则无人愿救，落至死亡。
            loop {
                while let Some(r) = self.responders.first() {
                    let alive = ctx.core.player(*r).alive;
                    let has_tao = alive
                        && ctx
                            .core
                            .player(*r)
                            .hand
                            .iter()
                            .any(|c| is_tao_like(ctx, *r, *c, self.dying));
                    if !has_tao {
                        self.responders.remove(0);
                    } else {
                        break;
                    }
                }
                if !self.responders.is_empty() || !self.round_used {
                    break;
                }
                let any_tao = ctx.core.players.iter().enumerate().any(|(i, p)| {
                    p.alive
                        && p.hand
                            .iter()
                            .any(|c| is_tao_like(ctx, i as PlayerId, *c, self.dying))
                });
                if !any_tao {
                    break;
                }
                self.responders = ctx.core.seat_order_from(self.dying);
                self.round_used = false;
            }
            let Some(responder) = self.responders.first().copied() else {
                ctx.emit(EventData::Death {
                    player: self.dying,
                    killer: self.source,
                });
                return Flow::Done;
            };
            if self.asked {
                self.asked = false;
                self.responders.remove(0);
                let answer = ctx.answer().clone();
                if let Some(ChoiceOption::Card(tao)) = answer.first().copied() {
                    ctx.discard_from_hand(responder, tao);
                    // 救援（孙权主公技，锁定；v202 对齐上游 filter）：主公
                    // 资格 + 用桃者为吴势力 + 非濒死者本人——桃回复 +1
                    let jiuyuan = ctx
                        .catalog
                        .skill_by_name(crate::vocabulary::names::JIU_YUAN)
                        .map(|d| {
                            ctx.core.has_skill(self.dying, d)
                                && responder != self.dying
                                && ctx.core.is_lord(self.dying)
                                && ctx.core.effective_faction(ctx.catalog, responder)
                                    == Some(crate::catalog::Faction::Wu)
                        })
                        .unwrap_or(false);
                    let amount = if jiuyuan { 2 } else { 1 };
                    ctx.recover(self.dying, amount, Some(tao));
                    ctx.emit(EventData::CardResponded {
                        card: tao,
                        player: responder,
                    });
                    ctx.log(LogEvent::DyingSaved {
                        player: self.dying,
                        by_card: tao,
                    });
                    self.round_used = true;
                }
                continue; // 无论用桃或放弃，继续处理队列（轮转见队首过滤）
            }
            self.asked = true;
            let mut options = vec![ChoiceOption::Pass];
            // 司圃（Boss 应昭，规则 v223/v232）：应昭出牌阶段已用牌数≤2
            // 期间其他人不可救（上游 cardSavable 同口径）——非应昭救援者
            // 的桃不入选项（酒同理由 cardRespondable）
            let sipu_ban_save = ctx
                .catalog
                .skill_by_name(crate::vocabulary::names::BOSS_SIPU)
                .map(|def| {
                    ctx.core.has_skill(self.dying, def)
                        && ctx.core.turn.current == self.dying
                        && ctx.core.turn.cards_used_play_phase < 3
                })
                .unwrap_or(false);
            let sipu_holder = if ctx
                .catalog
                .skill_by_name(crate::vocabulary::names::BOSS_SIPU)
                .is_some()
            {
                self.dying
            } else {
                0
            };
            options.extend(
                ctx.core
                    .player(responder)
                    .hand
                    .iter()
                    .filter(|c| is_tao_like(ctx, responder, **c, self.dying))
                    .filter(|c| {
                        !sipu_ban_save
                            || (responder == sipu_holder
                                && is_tao_like(ctx, responder, **c, self.dying))
                    })
                    .copied()
                    .map(ChoiceOption::Card),
            );
            // 冰封·封（swd 批 134）：封锁者不能求桃/自救
            let options = if ctx.catalog.blocks_all_cards(ctx.core, responder) {
                Vec::new()
            } else {
                options
            };
            return Flow::WaitChoice {
                observer: responder,
                spec: ChoiceSpec {
                    kind: ChoiceKind::RespondCard,
                    prompt: "dying_save",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            };
        }
    }
}
