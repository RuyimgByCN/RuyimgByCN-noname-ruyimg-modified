//! 轩辕剑（swd）机关牌效果（机关铜、机关凤、机关怨、机关术、机关妖术）。

use super::super::*;

impl HostMachine {
    pub(super) fn step_swd_jiguan_cards(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            HostBehavior::SwdJiguantongCard => match self.stage {
                // 机关桶（U05 swd 机关牌第四切片，规则 v192；v193 补齐
                // contentAfter 与逆序）：AOE 逐目标「弃一张手牌或受 1 火
                // 伤」（上游 ♦4/♣7，reverseOrder 逆序由 kernel 工厂臂在
                // 目标表反转）。无手牌目标直接火伤。
                HostStage::Act => {
                    // 本体已弃置 = AOE 已完成；targets 空 = 结算完 →
                    // contentAfter（无人受伤 → 使用者摸 1）→ 弃本体
                    if self.card.is_none() {
                        return Flow::Done;
                    }
                    let Some(t) = self.targets.first().copied() else {
                        let Some(body) = self.card else {
                            return Flow::Done;
                        };
                        // 上游 contentAfter !preResult：本次结算中无以本
                        // 牌归因的实际伤害（无懈抵消/取消弃牌不算；弃牌
                        // 免伤不算）→ 使用者摸 1（按日志实伤判定）
                        let damaged = ctx.out.iter().any(|l| {
                            matches!(l, LogEvent::DamageDealt { card: Some(c), .. } if *c == body)
                        });
                        if !damaged {
                            ctx.draw_cards(self.owner, 1);
                        }
                        // 本体弃置（卡牌机器惯例）
                        ctx.to_discard(body);
                        ctx.core.table.retain(|x| *x != body);
                        self.card = None;
                        return Flow::Done;
                    };
                    self.targets.remove(0);
                    if !ctx.core.player(t).alive {
                        return Flow::Yield;
                    }
                    if ctx.core.player(t).hand.is_empty() {
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            Some(self.owner),
                            t,
                            1,
                            crate::event::DamageNature::Fire,
                            self.card,
                        )));
                        return Flow::Yield;
                    }
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(t)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    self.targets.insert(0, t); // 未决目标回队首（SelectCard 段消费）
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: t,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: SWD_JIGUANTONG_DISCARD,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                        // 取消=受 1 火伤
                        let Some(t) = self.targets.first().copied() else {
                            return Flow::Done;
                        };
                        self.targets.remove(0);
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            Some(self.owner),
                            t,
                            1,
                            crate::event::DamageNature::Fire,
                            self.card,
                        )));
                        self.stage = HostStage::Act;
                        return Flow::Yield;
                    };
                    let Some(t) = self.targets.first().copied() else {
                        return Flow::Done;
                    };
                    self.targets.remove(0);
                    ctx.discard_from_hand(t, c);
                    ctx.log(LogEvent::CardDiscarded {
                        player: t,
                        cards: vec![c],
                    });
                    self.stage = HostStage::Act;
                    Flow::Yield
                }
                _ => Flow::Done,
            },
            // 机关凤（U05 swd 第二十批，规则 v195）：单体——目标选择打出
            // 一张闪（无闪/取消=未响应）；未响应→目标获得"封印"（全部技
            // 能实例禁用，其下一阶段开始解除[SwdFengyinState]，登记差异：
            // 原生无锁定技区分）+受 1 点伤害；响应则无事。本体入弃堆。
            HostBehavior::SwdJiguanfengCard => match self.stage {
                HostStage::Act => {
                    let Some(t) = self.target else {
                        if let Some(c) = self.card {
                            ctx.to_discard(c);
                            ctx.core.table.retain(|x| *x != c);
                            self.card = None;
                        }
                        return Flow::Done;
                    };
                    if !ctx.core.player(t).alive {
                        if let Some(c) = self.card {
                            ctx.to_discard(c);
                            ctx.core.table.retain(|x| *x != c);
                            self.card = None;
                        }
                        return Flow::Done;
                    }
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(t)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| ctx.catalog.card_kind(*c) == crate::catalog::CardKind::Shan)
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        // 无闪：上游 chooseToRespond 自动失败 → 未响应路径
                        self.seal_and_damage(ctx);
                        if let Some(c) = self.card {
                            ctx.to_discard(c);
                            ctx.core.table.retain(|x| *x != c);
                            self.card = None;
                        }
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: t,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: SWD_JIGUANFENG_SHAN,
                            options,
                            min: 0,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let responded = matches!(ctx.answer().first(), Some(ChoiceOption::Card(_)));
                    if responded {
                        // 打出闪响应：闪入弃牌堆（上游 chooseToRespond 同）
                        if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                            if let Some(t) = self.target {
                                ctx.discard_from_hand(t, c);
                                ctx.log(LogEvent::CardDiscarded {
                                    player: t,
                                    cards: vec![c],
                                });
                            }
                        }
                    } else {
                        // 未响应：封印 + 1 点伤害（上游 addTempSkill fengyin
                        // + damage 同序）
                        self.seal_and_damage(ctx);
                    }
                    if let Some(c) = self.card {
                        ctx.to_discard(c);
                        ctx.core.table.retain(|x| *x != c);
                        self.card = None;
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 机关怨（U05 swd 第二十一批，规则 v196）：单体——使用者选一
            // 张手牌，与本体一起置于目标的武将牌上（离场区），其下一结束
            // 阶段强制获得（SwdJiguanyuanGain）；使用者摸 1。本体不进弃
            // 堆（随置牌走）。
            HostBehavior::SwdJiguanyuanCard => match self.stage {
                HostStage::Act => {
                    let Some(t) = self.target else {
                        if let Some(c) = self.card {
                            ctx.to_discard(c);
                            ctx.core.table.retain(|x| *x != c);
                            self.card = None;
                        }
                        return Flow::Done;
                    };
                    if !ctx.core.player(t).alive || !ctx.core.player(self.owner).alive {
                        if let Some(c) = self.card {
                            ctx.to_discard(c);
                            ctx.core.table.retain(|x| *x != c);
                            self.card = None;
                        }
                        return Flow::Done;
                    }
                    if ctx.core.player(self.owner).hand.is_empty() {
                        // 上游 step0 else：无手牌 event.finish（本体入弃堆）
                        if let Some(c) = self.card {
                            ctx.to_discard(c);
                            ctx.core.table.retain(|x| *x != c);
                            self.card = None;
                        }
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
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: SWD_JIGUANYUAN_CHOOSE,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                        // 使用者手牌 + 本体 → 目标离场区；记录待获得实体
                        ctx.take_hand_card(self.owner, c);
                        ctx.core.player_mut(t).off_court.push(c);
                        if let Some(body) = self.card {
                            ctx.core.table.retain(|x| *x != body);
                            ctx.core.player_mut(t).off_court.push(body);
                            self.card = None;
                        }
                        let moved: Vec<crate::ids::CardId> =
                            ctx.core.player(t).off_court.iter().copied().collect();
                        ctx.core.player_mut(t).jiguanyuan_cards = moved;
                        if let Some(gdef) = ctx.catalog.skill_by_name(SWD_JIGUANYUAN_GAIN_KEY) {
                            ctx.insert_skill(
                                (t, gdef),
                                crate::state::SkillInstance {
                                    owner: t,
                                    enabled: true,
                                    once_used: false,
                                    expires_end_of_turn: false,
                                    rounds: 0,
                                },
                            );
                        }
                        ctx.log(LogEvent::CardsGiven {
                            from: self.owner,
                            to: t,
                            cards: ctx.core.player(t).jiguanyuan_cards.clone(),
                        });
                        ctx.draw_cards(self.owner, 1);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 机关术（U05 swd 机关术批，规则 v199/ADR 0780）：自目标——
            // 每张有效装备受控合成：随机附着一张灵剑实体（PlayerState.
            // card_mods 表）。登记差异：①名称合成/数值升级仅入改造表（目
            // 录不可变）；②jiqi/yuchan 池暂缓，全部附着灵剑；③演示呈现暂
            // 缓。本体入弃堆。
            HostBehavior::SwdJiguanshuCard => {
                let valid: Vec<CardId> =
                    ctx.core.player(self.owner).equips.iter().copied().collect();
                let lingjian: Vec<CardId> = ctx
                    .catalog
                    .cards
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| c.kind == CardKind::Hslingjian)
                    .map(|(i, _)| i as CardId)
                    .collect();
                for equip in valid {
                    if lingjian.is_empty()
                        || ctx.core.player(self.owner).card_mods.contains_key(&equip)
                    {
                        continue;
                    }
                    let k = ctx.core.rng.gen_range(lingjian.len() as u32) as usize;
                    let attached = lingjian[k];
                    ctx.core
                        .player_mut(self.owner)
                        .card_mods
                        .insert(equip, attached);
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                    self.card = None;
                }
                Flow::Done
            }
            // 机关妖术（U05 swd 第二十二批，规则 v197）：单体（range.global
            // 1）——从牌堆+弃堆装备实体中发现一张「目标空装备栏」子类型的
            // 装备，置入其手牌后 emit Equipped 走完整装配（commit_equip 同
            // 栏替换/装备技授予），并授予防重标记（登记差异：①发现池=牌
            // 堆+弃堆可见实体，上游=全部已定义装备牌凭空造牌，无候选则不
            // 装备；②上游 jiguanyaoshu_skill 的补偿触发面——失去装备区牌
            // 时补获 hslingjian 灵剑——依赖灵剑类型池，暂缓登记至机关术
            // 批，本切片仅保留防重面）。本体入弃堆。
            HostBehavior::SwdJiguanyaoshuCard => {
                if let Some(t) = self.target {
                    if ctx.core.player(t).alive {
                        let occupied: Vec<Option<crate::catalog::EquipSlot>> = ctx
                            .core
                            .player(t)
                            .equips
                            .iter()
                            .map(|c| ctx.catalog.card_slot_of(ctx.core, *c))
                            .collect();
                        let mut pool: Vec<CardId> = ctx
                            .core
                            .deck
                            .iter()
                            .chain(ctx.core.discard.iter())
                            .copied()
                            .filter(|x| {
                                let slot = ctx.catalog.card_slot_of(ctx.core, *x);
                                matches!(slot, Some(_)) && !occupied.contains(&slot)
                            })
                            .collect();
                        let n = pool.len();
                        for i in (1..n).rev() {
                            let j = ctx.core.rng.gen_range(i as u32 + 1) as usize;
                            pool.swap(i, j);
                        }
                        if let Some(x) = pool.first().copied() {
                            if let Some(pos) = ctx.core.deck.iter().position(|y| *y == x) {
                                ctx.core.deck.remove(pos);
                            } else if let Some(pos) = ctx.core.discard.iter().position(|y| *y == x)
                            {
                                ctx.core.discard.remove(pos);
                            }
                            ctx.core.player_mut(t).hand.push(x);
                            ctx.emit(EventData::Equipped {
                                player: t,
                                card: x,
                                replaced: None,
                            });
                        }
                        for gkey in [SWD_JIGUANYAOSHU_STATE_KEY, SWD_JIGUANYAO_GAIN_KEY] {
                            if let Some(sdef) = ctx.catalog.skill_by_name(gkey) {
                                ctx.insert_skill(
                                    (t, sdef),
                                    crate::state::SkillInstance {
                                        owner: t,
                                        enabled: true,
                                        once_used: false,
                                        expires_end_of_turn: false,
                                        rounds: 0,
                                    },
                                );
                            }
                        }
                    }
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
