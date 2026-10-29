//! 宿主技能：声明或转化后使用牌（起风、集控、蛊惑、极略、此桥邪）。

use super::super::*;

impl HostMachine {
    pub(super) fn step_skills_virtual_use(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            // 起风：选一张未用过的可用锦囊牌名并使用（上游 chooseButton
            // + chooseUseTarget；原生以目录牌种码承载牌名选择）
            HostBehavior::HlssFengqi => match self.stage {
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = hlss_fengqi_candidates(ctx, self.owner)
                        .into_iter()
                        .map(|k| ChoiceOption::Kind(crate::archive::kind_code(k)))
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "fengqi_card",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(ChoiceOption::Kind(code)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    let Ok(kind) = crate::archive::kind_from_code(code) else {
                        return Flow::Done;
                    };
                    let (min, max) = hlss_fengqi_targets(kind);
                    if max == 0 {
                        // 无目标锦囊：直接视为使用（无中/五谷/桃园/南蛮/万箭）
                        let targets = hlss_fengqi_all_targets(ctx, self.owner, kind);
                        ctx.use_virtual_card(self.owner, kind, targets);
                        ctx.core.player_mut(self.owner).hlss_fengqi_used.push(code);
                        return Flow::Done;
                    }
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| hlss_fengqi_target_ok(ctx, self.owner, *p, kind))
                        .map(ChoiceOption::Player)
                        .collect();
                    if (options.len() as u8) < min {
                        return Flow::Done;
                    }
                    self.cards.push(u32::from(code));
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "fengqi_targets",
                            max: options.len().min(max as usize),
                            options,
                            min: min as usize,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(code) = self.cards.first().copied() else {
                        return Flow::Done;
                    };
                    let Ok(kind) = crate::archive::kind_from_code(code as u16) else {
                        return Flow::Done;
                    };
                    let targets: Vec<PlayerId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Player(p) => Some(*p),
                            _ => None,
                        })
                        .collect();
                    if targets.is_empty() {
                        return Flow::Done;
                    }
                    ctx.use_virtual_card(self.owner, kind, targets);
                    ctx.core
                        .player_mut(self.owner)
                        .hlss_fengqi_used
                        .push(code as u16);
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 集控：选攻击距离内一名其他角色 → 虚拟雷杀（取消=不消耗次数）
            // 每步：弃一张牌 → 令目标本回合止息（+ 条件性距离视为 1）
            HostBehavior::NewMeibu => match self.stage {
                HostStage::Act => {
                    let Some(target) = self.target else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(target).alive {
                        return Flow::Done;
                    }
                    let mut options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .chain(ctx.core.player(self.owner).equips.iter())
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    options.truncate(64);
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "new_meibu_discard",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(target) = self.target else {
                        return Flow::Done;
                    };
                    let Some(ChoiceOption::Card(card)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(target).alive {
                        return Flow::Done;
                    }
                    let is_sha = ctx.catalog.card_kind(card).is_sha_like();
                    let is_trick = {
                        let k = ctx.catalog.card_kind(card);
                        !k.is_basic_kind() && k.equip_slot().is_none()
                    };
                    let is_black = matches!(
                        ctx.catalog.cards[card as usize].suit,
                        crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                    );
                    if ctx.core.player(self.owner).hand.contains(&card) {
                        ctx.discard_from_hand(self.owner, card);
                    } else if ctx.core.player(self.owner).equips.contains(&card) {
                        ctx.core
                            .player_mut(self.owner)
                            .equips
                            .retain(|c| *c != card);
                        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, self.owner, card);
                        ctx.to_discard(card);
                    }
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "每步",
                        owner: self.owner,
                    });
                    if let Some(def) = ctx.catalog.skill_by_name(L27_NEW_ZHIXI_KEY) {
                        ctx.core.skills.insert(
                            (target, def),
                            crate::state::SkillInstance {
                                owner: target,
                                enabled: true,
                                once_used: false,
                                expires_end_of_turn: true,
                                rounds: 0,
                            },
                        );
                        let hp = ctx.core.player(target).hp.max(1);
                        ctx.core.vm_aux.marks.insert((target, u32::from(def)), hp);
                    }
                    if !is_sha && !is_trick && !is_black {
                        if let Some(def) = ctx.catalog.skill_by_name(L27_NEW_MEIBU_RANGE_KEY) {
                            ctx.core.skills.insert(
                                (target, def),
                                crate::state::SkillInstance {
                                    owner: target,
                                    enabled: true,
                                    once_used: false,
                                    expires_end_of_turn: true,
                                    rounds: 0,
                                },
                            );
                            ctx.core
                                .vm_aux
                                .marks
                                .insert((target, u32::from(def)), i32::from(self.owner));
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 蛊惑：弃一张手牌 → 声明基本/锦囊牌名 → 选目标 → 转化使用
            HostBehavior::Reguhuo => match self.stage {
                HostStage::Act => {
                    // 蛊惑：每回合一次（第 942 轮）——进入执行即记账，turn 结束清
                    // 蛊惑：每回合一次（第 942 轮）——进入执行即记账，turn 结束清
                    ctx.core
                        .vm_aux
                        .once
                        .insert((self.owner, self.def_key.clone(), 1));
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "reguhuo_card",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(ChoiceOption::Card(card)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(self.owner).hand.contains(&card) {
                        return Flow::Done;
                    }
                    self.card = Some(card);
                    self.stage = HostStage::SelectNature;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::PickTag,
                            prompt: "reguhuo_name",
                            options: reguhuo_kind_options(),
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectNature => {
                    let Some(ChoiceOption::Tag(idx)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    let Some(card) = self.card else {
                        return Flow::Done;
                    };
                    let Some(kind) = reguhuo_kind_at(idx) else {
                        return Flow::Done;
                    };
                    let (min, max, targets) = ctx.core.legal_targets(ctx.catalog, kind, self.owner);
                    self.card = Some(card);
                    // 存**下标**（非 CardKind 判别值）——TargetChoice 按同表映射
                    self.cards = vec![u32::from(idx)];
                    if min == 0 && max == 0 {
                        ctx.log(LogEvent::SkillTriggered {
                            skill: "蛊惑",
                            owner: self.owner,
                        });
                        ctx.emit(EventData::CardUsed(crate::machine::card_use_as(
                            card,
                            self.owner,
                            Vec::new(),
                            Some(kind),
                        )));
                        self.stage = HostStage::ShuchuiTargetLoop;
                        return Flow::Yield;
                    }
                    if targets.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "reguhuo_targets",
                            options: targets.into_iter().map(ChoiceOption::Player).collect(),
                            min: min as usize,
                            max: max as usize,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(card) = self.card else {
                        return Flow::Done;
                    };
                    let Some(kind) = self.cards.first().and_then(|k| reguhuo_kind_at(*k as u8))
                    else {
                        return Flow::Done;
                    };
                    let targets: Vec<PlayerId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Player(p) => Some(*p),
                            _ => None,
                        })
                        .collect();
                    if targets.is_empty() {
                        return Flow::Done;
                    }
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "蛊惑",
                        owner: self.owner,
                    });
                    ctx.emit(EventData::CardUsed(crate::machine::card_use_as(
                        card,
                        self.owner,
                        targets,
                        Some(kind),
                    )));
                    self.stage = HostStage::ShuchuiTargetLoop;
                    Flow::Yield
                }
                _ => Flow::Done,
            },
            // 极略：看牌堆顶 3 张 → 选一张基本/锦囊牌 → 选目标 → 使用
            HostBehavior::Jilve => match self.stage {
                HostStage::Act => {
                    let top: Vec<CardId> = ctx.core.deck.iter().rev().take(3).copied().collect();
                    let options: Vec<ChoiceOption> = top
                        .into_iter()
                        .filter(|c| {
                            let kind = ctx.catalog.card_kind(*c);
                            if kind.is_basic_kind() || kind.equip_slot().is_none() {
                                // 基本牌或锦囊（非装备）且目标规则可满足
                                let (min, _, targets) =
                                    ctx.core.legal_targets(ctx.catalog, kind, self.owner);
                                min == 0 || !targets.is_empty()
                            } else {
                                false
                            }
                        })
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "jilve_card",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(ChoiceOption::Card(card)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    if !ctx.core.deck.contains(&card) {
                        return Flow::Done;
                    }
                    let kind = ctx.catalog.card_kind(card);
                    let (min, max, targets) = ctx.core.legal_targets(ctx.catalog, kind, self.owner);
                    self.card = Some(card);
                    if min == 0 && max == 0 {
                        // 无目标牌：直接使用（牌堆 → 手牌 → CardUsed）
                        ctx.core.deck.retain(|c| *c != card);
                        ctx.core.player_mut(self.owner).hand.push(card);
                        ctx.core.player_mut(self.owner).jilve_used += 1;
                        ctx.emit(EventData::CardUsed(crate::machine::card_use(
                            card,
                            self.owner,
                            Vec::new(),
                        )));
                        self.stage = HostStage::ShuchuiTargetLoop;
                        return Flow::Yield;
                    }
                    if targets.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "jilve_targets",
                            options: targets.into_iter().map(ChoiceOption::Player).collect(),
                            min: min as usize,
                            max: max as usize,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(card) = self.card else {
                        return Flow::Done;
                    };
                    let targets: Vec<PlayerId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Player(p) => Some(*p),
                            _ => None,
                        })
                        .collect();
                    if targets.is_empty() || !ctx.core.deck.contains(&card) {
                        return Flow::Done;
                    }
                    ctx.core.deck.retain(|c| *c != card);
                    ctx.core.player_mut(self.owner).hand.push(card);
                    ctx.core.player_mut(self.owner).jilve_used += 1;
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "极略",
                        owner: self.owner,
                    });
                    ctx.emit(EventData::CardUsed(crate::machine::card_use(
                        card, self.owner, targets,
                    )));
                    self.stage = HostStage::ShuchuiTargetLoop;
                    Flow::Yield
                }
                HostStage::ShuchuiTargetLoop => Flow::Done,
                _ => Flow::Done,
            },
            // 此桥邪：失去装备后逐次使用机关牌（自目标/单目标/AOE）
            HostBehavior::Cyqiaoxie => match self.stage {
                HostStage::Act => {
                    let lost_equips = match ctx.res.frame(ctx.current_frame_id()).map(|f| &f.data) {
                        Some(EventData::CardsLost { origins, .. }) => origins
                            .iter()
                            .filter(|o| **o == crate::event::LostOrigin::Equip)
                            .count(),
                        _ => 0,
                    };
                    if lost_equips == 0 {
                        return Flow::Done;
                    }
                    self.cards = (0..lost_equips).map(|_| 0u32).collect();
                    self.stage = HostStage::ShuchuiTargetLoop;
                    Flow::Yield
                }
                HostStage::ShuchuiTargetLoop => {
                    if self.cards.is_empty() {
                        return Flow::Done;
                    }
                    let options = cyqiaoxie_candidates(ctx, self.owner);
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.cards.pop();
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "cyqiaoxie_card",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(ChoiceOption::Card(card)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    self.card = Some(card);
                    let name = ctx.catalog.cards[card as usize].name;
                    match cyqiaoxie_rule(name) {
                        Some(CyqiaoxieRule::AllOthers) => {
                            let targets: Vec<PlayerId> = ctx
                                .core
                                .seat_order_from(self.owner)
                                .into_iter()
                                .filter(|p| *p != self.owner && ctx.core.player(*p).alive)
                                .collect();
                            cyqiaoxie_use(ctx, self.owner, card, targets);
                            self.stage = HostStage::ShuchuiTargetLoop;
                            Flow::Yield
                        }
                        Some(CyqiaoxieRule::OneOther) => {
                            let options: Vec<ChoiceOption> = ctx
                                .core
                                .alive_players()
                                .filter(|p| *p != self.owner && cyqiaoxie_target_ok(ctx, *p, name))
                                .map(ChoiceOption::Player)
                                .collect();
                            if options.is_empty() {
                                return Flow::Done;
                            }
                            self.stage = HostStage::TargetChoice;
                            Flow::WaitChoice {
                                observer: self.owner,
                                spec: ChoiceSpec {
                                    kind: ChoiceKind::SelectPlayers,
                                    prompt: "cyqiaoxie_target",
                                    options,
                                    min: 1,
                                    max: 1,
                                    allow_cancel: false,
                                },
                            }
                        }
                        Some(CyqiaoxieRule::SelfTarget) => {
                            cyqiaoxie_use(ctx, self.owner, card, vec![self.owner]);
                            self.stage = HostStage::ShuchuiTargetLoop;
                            Flow::Yield
                        }
                        None => Flow::Done,
                    }
                }
                HostStage::TargetChoice => {
                    let Some(ChoiceOption::Player(target)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    let Some(card) = self.card else {
                        return Flow::Done;
                    };
                    cyqiaoxie_use(ctx, self.owner, card, vec![target]);
                    self.stage = HostStage::ShuchuiTargetLoop;
                    Flow::Yield
                }
                _ => Flow::Done,
            },
            _ => unreachable!("behavior routed to the wrong step module"),
        }
    }
}
