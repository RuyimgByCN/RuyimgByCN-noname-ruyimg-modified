//! 轩辕剑（swd）宿主技：以牌为代价视为使用杀（花间、剑击、林韵、愚钝）。

use super::super::*;
use super::u08_batch_d::set_flag;

impl HostMachine {
    pub(super) fn step_swd_card_virtual_sha(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            // 花间·剑仙面：SelectCard 弃一张 he → TargetChoice 杀合法目标 →
            // 虚拟杀
            HostBehavior::SwdHuajian => match self.stage {
                HostStage::Act => {
                    let mut pool: Vec<CardId> =
                        ctx.core.player(self.owner).hand.iter().copied().collect();
                    pool.extend(ctx.core.player(self.owner).equips.iter().copied());
                    if pool.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "huajian_discard",
                            options: pool.iter().copied().map(ChoiceOption::Card).collect(),
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(picked) = ctx.answer().iter().find_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    let in_hand = ctx.core.player(self.owner).hand.contains(&picked);
                    if in_hand {
                        ctx.take_hand_card(self.owner, picked);
                    } else {
                        ctx.core
                            .player_mut(self.owner)
                            .equips
                            .retain(|x| *x != picked);
                        if ctx.catalog.card_kind(picked).is_equip() {
                            crate::kernel::revoke_equip_skills(
                                ctx.core,
                                ctx.catalog,
                                self.owner,
                                picked,
                            );
                        }
                    }
                    ctx.to_discard(picked);
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![picked],
                    });
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            *p != self.owner
                                && ctx.core.computed_distance(ctx.catalog, self.owner, *p)
                                    <= ctx.core.attack_range(ctx.catalog, self.owner)
                        })
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
                            prompt: "huajian_target",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(t) = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    if ctx.core.player(t).alive {
                        ctx.use_virtual_card(self.owner, CardKind::Sha, vec![t]);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 剑击·少微面（swd/skill.js:3598）：SelectCard 暂存装备牌 → TargetChoice 选目标后
            // 提交弃牌、置无视防具旗、摸 1 并使用无牌虚拟杀；结算后清旗
            HostBehavior::SwdJianji => match self.stage {
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .chain(ctx.core.player(self.owner).equips.iter().copied())
                        .filter(|c| ctx.catalog.card_kind(*c).is_equip())
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        super::u05_7c_skills::refund_once_used(ctx, self.owner, &self.def_key);
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "jianji_pick",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(c) = ctx.answer().iter().find_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    }) else {
                        super::u05_7c_skills::refund_once_used(ctx, self.owner, &self.def_key);
                        return Flow::Done;
                    };
                    if !ctx.core.player(self.owner).hand.contains(&c)
                        && !ctx.core.player(self.owner).equips.contains(&c)
                    {
                        super::u05_7c_skills::refund_once_used(ctx, self.owner, &self.def_key);
                        return Flow::Done;
                    }
                    self.card = Some(c);
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            *p != self.owner
                                && ctx.core.computed_distance(ctx.catalog, self.owner, *p)
                                    <= ctx.core.attack_range(ctx.catalog, self.owner)
                        })
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() {
                        super::u05_7c_skills::refund_once_used(ctx, self.owner, &self.def_key);
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "jianji_target",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(t) = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    }) else {
                        super::u05_7c_skills::refund_once_used(ctx, self.owner, &self.def_key);
                        return Flow::Done;
                    };
                    let Some(c) = self.card.take() else {
                        super::u05_7c_skills::refund_once_used(ctx, self.owner, &self.def_key);
                        return Flow::Done;
                    };
                    if !ctx.core.player(t).alive {
                        super::u05_7c_skills::refund_once_used(ctx, self.owner, &self.def_key);
                        return Flow::Done;
                    }
                    let Some(orig) = super::u05_7c_viewas::take_he_card_to_table(ctx, self.owner, c)
                    else {
                        super::u05_7c_skills::refund_once_used(ctx, self.owner, &self.def_key);
                        return Flow::Done;
                    };
                    set_flag(ctx.core, self.owner, "u05_swd_jianji_unequip", 1);
                    set_flag(ctx.core, self.owner, "vm_va_sha_no_count", 1);
                    // LIFO：先压杀（以原装备牌 c 作载体）、次压摸 1、最后压 CardsLost（先离区后摸再杀）
                    ctx.emit(EventData::CardUsed(crate::event::CardUseData {
                        card: Some(c),
                        player: self.owner,
                        targets: vec![t],
                        nature: crate::event::DamageNature::Normal,
                        use_as: Some(CardKind::Sha),
                        cancelled: false,
                    }));
                    ctx.start_draw(self.owner, 1);
                    ctx.emit(EventData::CardsLost {
                        player: self.owner,
                        cards: vec![c],
                        origins: vec![orig],
                    });
                    self.stage = HostStage::ShuchuiTargetLoop;
                    Flow::Yield
                }
                HostStage::ShuchuiTargetLoop => {
                    ctx.core
                        .vm_aux
                        .flags
                        .remove(&(self.owner, "u05_swd_jianji_unequip".to_string()));
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 愚钝·福面：SelectCards 两张锦囊手牌（消费段校验）→
            // TargetChoice 攻击距离内其他存活 → 首张作 declared-Sha 载荷
            //（use_as 声明）进入杀结算，次张弃置（丈八蛇矛机同型，
            // 不计出杀次数经 kernel 豁免臂）
            HostBehavior::SwdYudun => match self.stage {
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| {
                            let k = ctx.catalog.card_kind(*c);
                            !k.is_basic_kind()
                                && k.equip_slot().is_none()
                                && k.is_basic_trick_or_equip()
                        })
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
                            prompt: "yudun_pick_two",
                            options,
                            min: 2,
                            max: 2,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let picks: Vec<CardId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        })
                        .collect();
                    if picks.len() != 2 {
                        return Flow::Done;
                    }
                    // 消费段校验：两张均为锦囊且在手
                    let all_tricks = picks.iter().all(|c| {
                        let k = ctx.catalog.card_kind(*c);
                        !k.is_basic_kind()
                            && k.equip_slot().is_none()
                            && k.is_basic_trick_or_equip()
                    });
                    if !all_tricks
                        || picks
                            .iter()
                            .any(|c| !ctx.core.player(self.owner).hand.contains(c))
                    {
                        return Flow::Done;
                    }
                    self.cards = picks;
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            *p != self.owner
                                && ctx.core.computed_distance(ctx.catalog, self.owner, *p)
                                    <= ctx.core.attack_range(ctx.catalog, self.owner)
                        })
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
                            prompt: "yudun_sha_target",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    if let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() {
                        if ctx.core.player(t).alive {
                            let [c1, c2] = self.cards.as_slice() else {
                                return Flow::Done;
                            };
                            let (c1, c2) = (*c1, *c2);
                            // 次张弃为转化素材；首张作 declared-Sha 载荷
                            // 进入杀结算（丈八蛇矛同型）
                            ctx.discard_from_hand(self.owner, c2);
                            if ctx.core.player(self.owner).hand.contains(&c1) {
                                ctx.core.declared_kinds.insert(c1, CardKind::Sha);
                                set_flag(ctx.core, self.owner, &format!("swd_yudun_sha:{c1}"), 1);
                                ctx.emit(EventData::CardUsed(crate::machine::card_use_as(
                                    c1,
                                    self.owner,
                                    vec![t],
                                    Some(CardKind::Sha),
                                )));
                            }
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 林韵：两张牌当杀（手牌或装备；首张为载荷牌，declared Sha）
            HostBehavior::SwdLinyun => match self.stage {
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .chain(ctx.core.player(self.owner).equips.iter())
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.len() < 2 {
                        return Flow::Done;
                    }
                    self.stage = HostStage::CardsAsk;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "linyun_pick_two",
                            options,
                            min: 2,
                            max: 2,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::CardsAsk => {
                    let picked: Vec<CardId> = ctx
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
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "linyun_sha_target",
                            options: candidates,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let (Some(ChoiceOption::Player(target)), [c1, c2]) =
                        (ctx.answer().first(), self.cards.as_slice())
                    else {
                        return Flow::Done;
                    };
                    let target = *target;
                    if !ctx.core.player(target).alive {
                        return Flow::Done;
                    }
                    // 两张牌为转化素材（手牌/装备皆可）：混色（一红一黑）为无色 `"none"`，
                    // 同色保留花色；次张弃置，首张作载荷声明为杀
                    let is_red = |card: CardId| {
                        matches!(
                            ctx.catalog.effective_card_suit(ctx.core, self.owner, card),
                            crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                        )
                    };
                    let mixed_color = is_red(*c1) != is_red(*c2);
                    for c in [*c2, *c1] {
                        if ctx.core.player(self.owner).hand.contains(&c) {
                            if c == *c1 {
                                continue;
                            }
                            ctx.discard_from_hand(self.owner, c);
                        } else if ctx.core.player(self.owner).equips.contains(&c) {
                            if c == *c1 {
                                continue;
                            }
                            ctx.core.player_mut(self.owner).equips.retain(|x| *x != c);
                            crate::kernel::revoke_equip_skills(
                                ctx.core,
                                ctx.catalog,
                                self.owner,
                                c,
                            );
                            ctx.to_discard(c);
                        } else {
                            return Flow::Done;
                        }
                    }
                    // 载荷牌须在手中（emit 后由杀结算弃置）
                    if ctx.core.player(self.owner).equips.contains(c1) {
                        ctx.core.player_mut(self.owner).equips.retain(|x| x != c1);
                        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, self.owner, *c1);
                        ctx.core.player_mut(self.owner).hand.push(*c1);
                    }
                    if !ctx.core.player(self.owner).hand.contains(c1) {
                        return Flow::Done;
                    }
                    ctx.core
                        .declared_kinds
                        .insert(*c1, crate::catalog::CardKind::Sha);
                    set_flag(ctx.core, self.owner, &format!("swd_linyun_sha:{}", *c1), 1);
                    if mixed_color {
                        set_flag(
                            ctx.core,
                            self.owner,
                            &format!("swd_linyun_colorless:{}", *c1),
                            1,
                        );
                    }
                    ctx.emit(crate::event::EventData::CardUsed(
                        crate::machine::card_use_as(
                            *c1,
                            self.owner,
                            vec![target],
                            Some(crate::catalog::CardKind::Sha),
                        ),
                    ));
                    // 置终态：子帧（杀结算）结束后本机再次恢复时直接收束，
                    // 不再读取已消费的应答
                    self.stage = HostStage::ShuchuiTargetLoop;
                    Flow::Yield
                }
                _ => Flow::Done,
            },
            _ => unreachable!("behavior routed to the wrong step module"),
        }
    }
}
