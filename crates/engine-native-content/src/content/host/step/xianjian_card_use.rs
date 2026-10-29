//! 仙剑（xianjian）宿主技：以牌转化使用或拼点（蝶舞、照妖、天剑、千方、乱剑）。

use super::super::*;

impl HostMachine {
    pub(super) fn step_xianjian_card_use(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            HostBehavior::XianjianDiewu => match self.stage {
                // 蝶舞（U05 xianjian 龙葵，规则 v153）：选 1 张杀+1 目标
                // →直接转移（discard:false give 同口径）→本回合首次摸 1。
                HostStage::SelectCard => {
                    let p = ctx.core.player(self.owner);
                    let options: Vec<ChoiceOption> = p
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| ctx.catalog.card_kind(*c).is_sha_like())
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_XJ_DIEWU_CARDS,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    // 收「牌选择」应答
                    if self.cards.is_empty() {
                        let cards: Vec<CardId> = ctx
                            .answer()
                            .iter()
                            .filter_map(|o| match o {
                                ChoiceOption::Card(c) => Some(*c),
                                _ => None,
                            })
                            .collect();
                        if cards.len() != 1 {
                            return Flow::Done;
                        }
                        self.cards = cards;
                        let options: Vec<ChoiceOption> = ctx
                            .core
                            .alive_players()
                            .filter(|p| *p != self.owner)
                            .map(ChoiceOption::Player)
                            .collect();
                        if options.is_empty() {
                            return Flow::Done;
                        }
                        Flow::WaitChoice {
                            observer: self.owner,
                            spec: ChoiceSpec {
                                kind: ChoiceKind::SelectPlayers,
                                prompt: WUDIAN_XJ_DIEWU_TARGET,
                                options,
                                min: 1,
                                max: 1,
                                allow_cancel: false,
                            },
                        }
                    } else {
                        // 收「目标选择」应答：转移+首次摸 1
                        let Some(t) = ctx.answer().first().and_then(|o| match o {
                            ChoiceOption::Player(p) => Some(*p),
                            _ => None,
                        }) else {
                            return Flow::Done;
                        };
                        let sel = std::mem::take(&mut self.cards);
                        ctx.give_hand_cards(self.owner, t, &sel);
                        if !ctx.core.turn.diewu_draw_used {
                            ctx.core.turn.diewu_draw_used = true;
                            ctx.draw_cards(self.owner, 1);
                        }
                        Flow::Done
                    }
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianZhaoyao => match self.stage {
                // 照妖（U05 xianjian 南宫煌，规则 v172）：确定性首张拼点
                //（舌战群儒同款登记差异）——赢→摸 2+选两张手牌置牌堆顶
                //（先选在上=后选的先压栈）。输/平→无事。首臂比点并发问。
                HostStage::Act => {
                    let own_r = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .first()
                        .map(|c| ctx.catalog.cards[*c as usize].rank);
                    let t_r = self.target.and_then(|t| {
                        ctx.core
                            .player(t)
                            .hand
                            .first()
                            .map(|c| ctx.catalog.cards[*c as usize].rank)
                    });
                    match (own_r, t_r) {
                        (Some(o), Some(t)) if o > t => {
                            ctx.draw_cards(self.owner, 2);
                            let hand: Vec<CardId> = ctx.core.player(self.owner).hand.clone();
                            if hand.len() < 2 {
                                return Flow::Done;
                            }
                            let options: Vec<ChoiceOption> =
                                hand.iter().copied().map(ChoiceOption::Card).collect();
                            self.stage = HostStage::SelectCard;
                            Flow::WaitChoice {
                                observer: self.owner,
                                spec: ChoiceSpec {
                                    kind: ChoiceKind::SelectCards,
                                    prompt: WUDIAN_XJ_ZHAOYAO_CARDS,
                                    options,
                                    min: 2,
                                    max: 2,
                                    allow_cancel: false,
                                },
                            }
                        }
                        _ => Flow::Done,
                    }
                }
                HostStage::SelectCard => {
                    let picked: Vec<CardId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        })
                        .collect();
                    if picked.len() == 2 {
                        for c in &picked {
                            ctx.take_hand_card(self.owner, *c);
                        }
                        // 先选在上：后选的先压栈
                        for c in picked.iter().rev() {
                            ctx.core.deck.push(*c);
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianTianjian => match self.stage {
                // 天剑（U05 xianjian 姜云凡，规则 v178）：弃一张手牌杀
                //（必选）→视为使用万箭齐发（虚拟 AOE 原语：全其他存活）
                // →伤害目标随机弃 he 一张（discard 子技直连=登记差异：
                // 上游为 damageEnd 子技）。
                HostStage::SelectCard => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| ctx.catalog.card_kind(*c) == crate::catalog::CardKind::Sha)
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_XJ_TIANJIAN_CARDS,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(ChoiceOption::Card(c)) = ctx.answer().first() else {
                        return Flow::Done;
                    };
                    let c = *c;
                    ctx.take_hand_card(self.owner, c);
                    ctx.to_discard(c);
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![c],
                    });
                    // 虚拟万箭 AOE
                    let targets: Vec<PlayerId> = ctx
                        .core
                        .seat_order_from(self.owner)
                        .into_iter()
                        .filter(|p| *p != self.owner)
                        .collect();
                    ctx.use_virtual_card(self.owner, CardKind::ArcheryAttack, targets.clone());
                    // 伤害目标随机弃 he 一张（直连）
                    for p in targets {
                        if !ctx.core.player(p).alive {
                            continue;
                        }
                        let mut he: Vec<CardId> = ctx.core.player(p).hand.iter().copied().collect();
                        he.extend(ctx.core.player(p).equips.iter().copied());
                        if he.is_empty() {
                            continue;
                        }
                        let i = ctx.core.rng.gen_range(he.len() as u32) as usize;
                        let dc = he[i];
                        if ctx.core.player(p).hand.contains(&dc) {
                            ctx.discard_from_hand(p, dc);
                        } else {
                            ctx.core.player_mut(p).equips.retain(|x| *x != dc);
                            crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, p, dc);
                            ctx.to_discard(dc);
                            ctx.log(LogEvent::CardDiscarded {
                                player: p,
                                cards: vec![dc],
                            });
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianQianfang => match self.stage {
                // 千方（U05 xianjian 慕容紫英，规则 v182）：回合开始时
                // 玄砚计数换弃牌（num=3-计数 clamp≥0）后清零，视为对全
                // 部其他角色使用万箭齐发（虚拟 AOE 原语）；被伤敌人每名
                // 紫英摸 1（qianfang_draw 直连=登记差异：上游 damageEnd
                // 逐目标）。首臂发问弃牌（可取消=放弃；num=0 时直接
                // AOE）。
                HostStage::Act => {
                    let mark = ctx.core.player(self.owner).xuanning_mark;
                    let num = 3usize.saturating_sub(mark as usize);
                    let targets: Vec<PlayerId> = ctx
                        .core
                        .seat_order_from(self.owner)
                        .into_iter()
                        .filter(|p| *p != self.owner)
                        .collect();
                    if targets.is_empty() {
                        return Flow::Done;
                    }
                    self.targets = targets.clone();
                    self.cards = vec![CardId::from(num as u32)]; // num 暂存
                    if num == 0 {
                        // 纯 AOE（chooseBool 确认=direct 无询问直接发动）
                        ctx.core.player_mut(self.owner).xuanning_mark = 0;
                        ctx.use_virtual_card(self.owner, CardKind::ArcheryAttack, targets);
                        return Flow::Done;
                    }
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .chain(ctx.core.player(self.owner).equips.iter())
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.len() < num {
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_XJ_QIANFANG_CARDS,
                            options,
                            min: num,
                            max: num,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let picked: Vec<CardId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        })
                        .collect();
                    let num =
                        3usize.saturating_sub(ctx.core.player(self.owner).xuanning_mark as usize);
                    if picked.len() != num {
                        return Flow::Done; // 取消=放弃
                    }
                    for c in &picked {
                        if ctx.core.player(self.owner).hand.contains(c) {
                            ctx.discard_from_hand(self.owner, *c);
                        } else {
                            ctx.core.player_mut(self.owner).equips.retain(|x| x != c);
                            crate::kernel::revoke_equip_skills(
                                ctx.core,
                                ctx.catalog,
                                self.owner,
                                *c,
                            );
                            ctx.to_discard(*c);
                        }
                    }
                    ctx.core.player_mut(self.owner).xuanning_mark = 0;
                    let targets = self.targets.clone();
                    ctx.use_virtual_card(self.owner, CardKind::ArcheryAttack, targets.clone());
                    // 被伤敌人每名紫英摸 1（qianfang_draw 直连=登记差异：
                    // 上游 damageEnd 逐目标即时）
                    let hurt: Vec<PlayerId> = targets
                        .into_iter()
                        .filter(|p| ctx.core.player(*p).alive)
                        .collect();
                    let n = hurt.len();
                    ctx.draw_cards(self.owner, n);
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianLuanjian => match self.stage {
                // 乱剑（U05 xianjian 长卿，规则 v187）：两张杀当一张杀
                // 使用（虚拟杀=天剑同款弃素材；机器直发不经 PlayCard
                // 校验=无视距离+任意多目标登记差异）。
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| ctx.catalog.card_kind(*c) == crate::catalog::CardKind::Sha)
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.len() < 2 {
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_XJ_LUANJIAN_CARDS,
                            options,
                            min: 2,
                            max: 2,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let picked: Vec<CardId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        })
                        .collect();
                    if picked.len() != 2 {
                        return Flow::Done; // 取消=放弃
                    }
                    for c in &picked {
                        ctx.discard_from_hand(self.owner, *c);
                    }
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: picked.clone(),
                    });
                    self.cards = picked;
                    // 目标多选（任意名其他存活角色；allow_cancel=false=
                    // 上游 selectTarget [1,∞] 必选，素材已弃不退）
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .seat_order_from(self.owner)
                        .into_iter()
                        .filter(|p| *p != self.owner && ctx.core.player(*p).alive)
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    let max = options.len();
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: WUDIAN_XJ_LUANJIAN_TARGETS,
                            options,
                            min: 1,
                            max,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let picked: Vec<PlayerId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Player(p) => Some(*p),
                            _ => None,
                        })
                        .collect();
                    if picked.is_empty() {
                        return Flow::Done;
                    }
                    // 载体牌锚定（DamageBefore 增伤判定=普通杀/其他虚拟
                    // 技载体不命中）——预读牌堆底=use_virtual_card 载体
                    let carrier = ctx.core.deck.first().copied();
                    ctx.core.turn.luanjian_card = carrier.map(|c| c as u32);
                    ctx.use_virtual_card(self.owner, crate::catalog::CardKind::Sha, picked);
                    Flow::Done
                }
                _ => Flow::Done,
            },
            _ => unreachable!("behavior routed to the wrong step module"),
        }
    }
}
