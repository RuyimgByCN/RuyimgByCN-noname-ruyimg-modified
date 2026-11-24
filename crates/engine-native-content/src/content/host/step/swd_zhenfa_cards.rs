//! 轩辕剑（swd）阵法牌效果（天覆阵、七星阵、鸟翔阵、长蛇阵、破阵诀）。

use super::super::*;
use crate::snap::{ask, ExpectedAsk};

pub(in crate::content::host) fn ask_swd_zhenfa_card(
    behavior: HostBehavior,
    stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    core: &crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
) -> Option<ExpectedAsk> {
    let t = target?;
    match behavior {
        HostBehavior::SwdTianfuzhenCard if stage == HostStage::SelectCard => {
            let options: Vec<ChoiceOption> = core
                .player(t)
                .hand
                .iter()
                .copied()
                .chain(core.player(t).equips.iter().copied())
                .map(ChoiceOption::Card)
                .collect();
            (!options.is_empty()).then(|| {
                ask(t, "swd_tianfuzhen_discard", ChoiceKind::SelectCards, options, 1, 1, false)
            })
        }
        HostBehavior::SwdQixingzhenCard if stage == HostStage::SelectCard => {
            let options: Vec<ChoiceOption> = core
                .player(t)
                .hand
                .iter()
                .chain(core.player(t).equips.iter())
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            (!options.is_empty()).then(|| {
                ask(owner, "swd_qixingzhen_discard", ChoiceKind::SelectCards, options, 1, 1, false)
            })
        }
        HostBehavior::SwdNiaoxiangzhenCard if stage == HostStage::TargetChoice => {
            let options: Vec<ChoiceOption> = core
                .player(t)
                .hand
                .iter()
                .copied()
                .filter(|c| catalog.card_kind(*c) == CardKind::Shan)
                .map(ChoiceOption::Card)
                .collect();
            Some(ask(
                t,
                "swd_niaoxiangzhen_shan",
                ChoiceKind::SelectCards,
                options,
                0,
                1,
                true,
            ))
        }
        _ => None,
    }
}

impl HostMachine {
    pub(super) fn step_swd_zhenfa_cards(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            // 天覆阵（zhenfa，规则 v200/T19a）：AOE 逐大国目标强制弃 1 张
            // he（targets 队列；目标自选牌 min 1 不可取消；self.target=当
            // 前询问目标）。全部结算完后本体入弃堆。
            HostBehavior::SwdTianfuzhenCard => match self.stage {
                HostStage::Act => {
                    if self.targets.is_empty() {
                        if let Some(c) = self.card {
                            ctx.to_discard(c);
                            ctx.core.table.retain(|x| *x != c);
                            self.card = None;
                        }
                        return Flow::Done;
                    }
                    let t = self.targets.remove(0);
                    self.target = Some(t);
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(t)
                        .hand
                        .iter()
                        .copied()
                        .chain(ctx.core.player(t).equips.iter().copied())
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        return Flow::Yield;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: t,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "swd_tianfuzhen_discard",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                        let t = self.target.unwrap_or(self.owner);
                        if ctx.core.player(t).hand.contains(&c) {
                            ctx.discard_from_hand(t, c);
                        } else if let Some(pos) =
                            ctx.core.player(t).equips.iter().position(|x| *x == c)
                        {
                            ctx.core.player_mut(t).equips.remove(pos);
                            crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, t, c);
                            ctx.to_discard(c);
                        }
                        ctx.log(LogEvent::CardDiscarded {
                            player: t,
                            cards: vec![c],
                        });
                    }
                    self.stage = HostStage::Act;
                    Flow::Yield
                }
                _ => Flow::Done,
            },
            // 七星阵（zhenfa，规则 v200/T19a）：阶段一 Act——逐围攻者弃 1
            //（SelectCard 询问，self.target=围攻者）；队列空后阶段二——
            // 视为对围攻者用杀（响应闪或 1 伤；登记差异：虚拟杀简化，无
            // 加成/不可无懈额外面），阶段二队列存 self.cards。本体入弃堆。
            HostBehavior::SwdQixingzhenCard => match self.stage {
                HostStage::Act => {
                    if !self.targets.is_empty() {
                        let t = self.targets.remove(0);
                        self.target = Some(t);
                        let options: Vec<ChoiceOption> = ctx
                            .core
                            .player(t)
                            .hand
                            .iter()
                            .chain(ctx.core.player(t).equips.iter())
                            .copied()
                            .map(ChoiceOption::Card)
                            .collect();
                        if options.is_empty() {
                            return Flow::Yield;
                        }
                        self.stage = HostStage::SelectCard;
                        return Flow::WaitChoice {
                            observer: self.owner,
                            spec: ChoiceSpec {
                                kind: ChoiceKind::SelectCards,
                                prompt: "swd_qixingzhen_discard",
                                options,
                                min: 1,
                                max: 1,
                                allow_cancel: false,
                            },
                        };
                    }
                    if self.cards.is_empty() {
                        if let Some(c) = self.card.take() {
                            ctx.to_discard(c);
                            ctx.core.table.retain(|x| *x != c);
                        }
                        return Flow::Done;
                    }
                    if let Some(c) = self.card.take() {
                        ctx.to_discard(c);
                        ctx.core.table.retain(|x| *x != c);
                    }
                    // 七星阵阶段二（card/zhenfa/card.js:187-194）：弃牌后视为对所有
                    // 围攻者使用一张无实体虚拟【杀】，且 `false` 不计入出牌阶段出杀次数。
                    let besiegers = std::mem::take(&mut self.cards);
                    let sha_targets: Vec<PlayerId> = ctx
                        .core
                        .seat_order_from(self.owner)
                        .into_iter()
                        .filter(|&p| {
                            p != self.owner
                                && besiegers.contains(&u32::from(p))
                                && ctx.core.player(p).alive
                                && ctx.core.in_attack_range(ctx.catalog, self.owner, p)
                                && !super::u01_fixes_1::is_target_kongcheng(
                                    ctx.core,
                                    ctx.catalog,
                                    p,
                                )
                                && !crate::rvm_bridge::modifiers::eval_card_target_canceled_opt(
                                    ctx.core,
                                    ctx.catalog,
                                    self.owner,
                                    p,
                                    None,
                                    CardKind::Sha,
                                    crate::event::DamageNature::Normal,
                                )
                        })
                        .collect();
                    if !sha_targets.is_empty() {
                        ctx.core
                            .vm_aux
                            .flags
                            .insert((self.owner, "vm_va_sha_no_count".to_string()), 1);
                        ctx.emit(EventData::CardUsed(crate::event::CardUseData {
                            card: None,
                            player: self.owner,
                            targets: sha_targets,
                            nature: crate::event::DamageNature::Normal,
                            use_as: Some(CardKind::Sha),
                            cancelled: false,
                        }));
                    }
                    Flow::Done
                }
                HostStage::SelectCard => {
                    if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                        let t = self.target.unwrap_or(self.owner);
                        if ctx.core.player(t).hand.contains(&c) {
                            ctx.discard_from_hand(t, c);
                        } else if let Some(pos) =
                            ctx.core.player(t).equips.iter().position(|x| *x == c)
                        {
                            ctx.core.player_mut(t).equips.remove(pos);
                            crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, t, c);
                            ctx.to_discard(c);
                            ctx.log(LogEvent::CardDiscarded {
                                player: t,
                                cards: vec![c],
                            });
                        }
                    }
                    self.stage = HostStage::Act;
                    Flow::Yield
                }
                _ => Flow::Done,
            },
            // 鸟翔阵（zhenfa，规则 v200/T19a）：逐合法目标响应闪或受 1 伤
            //（targets 队列；TargetChoice 询问，取消=伤害）。本体入弃堆。
            HostBehavior::SwdNiaoxiangzhenCard => match self.stage {
                HostStage::Act => {
                    if self.targets.is_empty() {
                        if let Some(c) = self.card {
                            ctx.to_discard(c);
                            ctx.core.table.retain(|x| *x != c);
                            self.card = None;
                        }
                        return Flow::Done;
                    }
                    let t = self.targets.remove(0);
                    self.target = Some(t);
                    self.stage = HostStage::TargetChoice;
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(t)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| ctx.catalog.card_kind(*c) == CardKind::Shan)
                        .map(ChoiceOption::Card)
                        .collect();
                    Flow::WaitChoice {
                        observer: t,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "swd_niaoxiangzhen_shan",
                            options,
                            min: 0,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let t = self.target.unwrap_or(self.owner);
                    if matches!(ctx.answer().first(), Some(ChoiceOption::Card(_))) {
                        if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                            ctx.discard_from_hand(t, c);
                            ctx.log(LogEvent::CardDiscarded {
                                player: t,
                                cards: vec![c],
                            });
                        }
                    } else {
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            Some(self.owner),
                            t,
                            1,
                            crate::event::DamageNature::Normal,
                            None,
                        )));
                    }
                    self.stage = HostStage::Act;
                    Flow::Yield
                }
                _ => Flow::Done,
            },
            // 长蛇阵（zhenfa，规则 v201/T19b）：队列中→全队列（含自己）各
            // 摸 1；否则与距离最近的友方换座成队列（swapSeat）；本体入弃堆。
            HostBehavior::SwdChangshezhenCard => {
                let inline_now = ctx.core.inline(ctx.catalog, self.owner);
                let prev_of_owner = ctx.core.alive_neighbors(self.owner).1;
                if inline_now {
                    for q in 0..ctx.core.players.len() as u8 {
                        if ctx.core.inline_with(ctx.catalog, self.owner, q) {
                            ctx.start_draw(q, 1);
                        }
                    }
                } else if prev_of_owner.is_some() {
                    // 友方按**顺座次序**排序（NC01：顺序语义用 seat_order_from；
                    // 上游按座次轮转取最近）
                    let mut friends: Vec<(u8, u8)> = (0..ctx.core.players.len() as u8)
                        .filter(|q| {
                            *q != self.owner
                                && ctx.core.player(*q).alive
                                && ctx.core.is_friend_of(ctx.catalog, self.owner, *q)
                        })
                        .map(|q| (q, ctx.core.seat_steps_from(self.owner, q)))
                        .collect();
                    friends.sort_by_key(|(_, d)| *d);
                    if let Some((nearest, _)) = friends.first().copied() {
                        let next_of_owner = ctx.core.next_alive(self.owner);
                        ctx.core.swap_seat(nearest, next_of_owner);
                    }
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                    self.card = None;
                }
                Flow::Done
            }
            // 破阵诀（zhenfa，规则 v201/T19b）：n 次随机座位交换（n=存活
            // 数，上游 swapSeat(random, random) ×n 同口径）；本体入弃堆。
            HostBehavior::SwdPozhenjueCard => {
                let n = ctx.core.alive_players().count();
                for _ in 0..n {
                    let seats: Vec<u8> = ctx.core.alive_players().collect();
                    if seats.len() < 2 {
                        break;
                    }
                    let i = ctx.core.rng.gen_range(seats.len() as u32) as usize;
                    let j = ctx.core.rng.gen_range(seats.len() as u32) as usize;
                    ctx.core.swap_seat(seats[i], seats[j]);
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                    self.card = None;
                }
                Flow::Done
            }
            _ => unreachable!("behavior routed to the wrong step module"),
        }
    }
}
