use super::super::*;
use super::JudgePhaseMachine;

impl JudgePhaseMachine {
    pub(super) fn effective_judge_suit_rank(
        core: &mut crate::state::GameStateCore,
        catalog: &crate::catalog::Catalog,
        owner: PlayerId,
        judge_card: crate::ids::CardId,
    ) -> (crate::catalog::Suit, u8) {
        let suit_from_code = |c: u32| match c {
            1 => crate::catalog::Suit::Spade,
            2 => crate::catalog::Suit::Heart,
            3 => crate::catalog::Suit::Club,
            _ => crate::catalog::Suit::Diamond,
        };
        let def = catalog
            .resolved_card_def(core, judge_card)
            .unwrap_or(&catalog.cards[judge_card as usize]);
        let (mut suit, mut rank) = (def.suit, def.rank);
        let override_suit = core.vm_aux.flags.remove(&(owner, "zhenyi_judge_suit".to_string())).unwrap_or(0);
        let override_num = core.vm_aux.flags.remove(&(owner, "zhenyi_judge_number".to_string())).unwrap_or(0);
        if override_suit > 0 {
            suit = suit_from_code(override_suit);
            rank = if override_num > 0 { override_num as u8 } else { 5 };
        } else if suit == crate::catalog::Suit::Spade
            && !catalog.is_chanyuan_sealed(core, owner)
            && ["xinhongyan", "hongyan", "红颜"]
                .iter()
                .any(|name| catalog.skill_by_name(name).is_some_and(|d| core.has_skill(owner, d)))
        {
            suit = crate::catalog::Suit::Heart;
        }
        let xhy_suit = core.vm_aux.flags.remove(&(owner, "xinhongyan_judge_suit".to_string())).unwrap_or(0);
        if xhy_suit > 0 {
            suit = suit_from_code(xhy_suit);
        }
        crate::rvm_bridge::modifiers::apply_huangtian_suit_rank(core, catalog, owner, suit, rank)
    }

    pub(super) fn notify_judge_settled(
        ctx: &mut Ctx<'_>,
        owner: PlayerId,
        judge_card: crate::ids::CardId,
    ) -> bool {
        if !ctx.catalog.vm_card_history {
            return false;
        }
        let kind_code = ctx
            .core
            .vm_aux
            .flags
            .remove(&(owner, format!("l11_judge_kind:{judge_card}")))
            .unwrap_or(0);
        let suit_code = ctx
            .core
            .vm_aux
            .flags
            .remove(&(owner, format!("l11_judge_suit:{judge_card}")))
            .unwrap_or(0);
        let trick_kind = match kind_code {
            1 => Some(CardKind::Le),
            2 => Some(CardKind::Bing),
            3 => Some(CardKind::ShanDian),
            4 => Some(CardKind::HongShui),
            5 => Some(CardKind::Guiyoujie),
            _ => None,
        };
        let eff_suit = match suit_code {
            1 => crate::catalog::Suit::Spade,
            2 => crate::catalog::Suit::Heart,
            3 => crate::catalog::Suit::Club,
            4 => crate::catalog::Suit::Diamond,
            _ => ctx.catalog.effective_card_suit(ctx.core, owner, judge_card),
        };
        (ctx.catalog.hooks.on_judge_settled)(ctx, owner, trick_kind, eff_suit, judge_card)
    }

    /// 判定牌生效：按锦囊种类套用效果（`machine_holds` 为真时判定牌仍由
    /// 机器持有——天妒未决——不随效果弃置）。
    pub(super) fn apply_judgement(
        owner: PlayerId,
        ctx: &mut Ctx<'_>,
        trick: crate::ids::CardId,
        judge_card: crate::ids::CardId,
        machine_holds: bool,
    ) -> (CardKind, crate::catalog::Suit) {
        // 国色等 view-as 转化：声明牌种优先于目录牌种（N08 迁移）
        let trick_kind = ctx.catalog.resolved_card_kind(ctx.core, trick);
        // 声明消费后移除（牌已结算，残留条目会在后续洗回场景误判）
        ctx.core.declared_kinds.remove(&trick);
        let (suit, rank) = Self::effective_judge_suit_rank(ctx.core, ctx.catalog, owner, judge_card);
        // 神裔（Boss 青龙共有技，规则 v233）/ 隐月·判（T11 Boss 诸葛亮族批 115，规则 v303）：
        // 判定阶段的延时锦囊判定对持有者恒不生效（牌去向不变：锦囊照常弃置、闪电照常传下家）。
        let shenyi_voids = [
            crate::vocabulary::names::BOSS_SHENYI,
            crate::vocabulary::host::HLSS_YUEYIN2_KEY,
        ]
        .iter()
        .any(|k| ctx.catalog.skill_by_name(k).is_some_and(|def| ctx.core.has_skill(owner, def)));
        let discard_trick = |ctx: &mut Ctx<'_>, owner: PlayerId, card: crate::ids::CardId| {
            ctx.to_discard(card);
            (ctx.catalog.hooks.on_cards_to_discard)(ctx, owner, &[card], false);
        };
        match trick_kind {
            CardKind::Le | CardKind::Bing | CardKind::HongShui => {
                super::judge_u05_b1::resolve_le_bing_hongshui(
                    ctx, owner, trick, trick_kind, judge_card, machine_holds, shenyi_voids, suit, rank,
                );
                return (trick_kind, suit);
            }
            CardKind::Guiyoujie => {
                // 归有界判定（上游 judge 黑色 -3 / effect loseHp+randomDiscard）
                if !shenyi_voids
                    && matches!(
                        suit,
                        crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                    )
                {
                    discard_trick(ctx, owner, trick);
                    ctx.emit(EventData::LoseHp {
                        player: owner,
                        amount: 1,
                    });
                    // 随机弃一张牌（手牌+装备均匀抽取；逐技对照批四口径）
                    let he_len =
                        ctx.core.player(owner).hand.len() + ctx.core.player(owner).equips.len();
                    if he_len > 0 {
                        let pick = ctx.core.rng.gen_range(he_len as u32) as usize;
                        let hand_n = ctx.core.player(owner).hand.len();
                        let c = if pick < hand_n {
                            ctx.core.player(owner).hand[pick]
                        } else {
                            ctx.core.player(owner).equips[pick - hand_n]
                        };
                        let in_hand = ctx.core.player(owner).hand.contains(&c);
                        if in_hand {
                            ctx.discard_from_hand(owner, c);
                        } else {
                            ctx.core.player_mut(owner).equips.retain(|x| *x != c);
                            crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, owner, c);
                            ctx.to_discard(c);
                        }
                        ctx.log(LogEvent::CardDiscarded {
                            player: owner,
                            cards: vec![c],
                        });
                    }
                }
                ctx.to_discard(trick);
                if !machine_holds {
                    ctx.to_discard(judge_card);
                }
            }
            CardKind::ShanDian => {
                if super::judge_hearth::is_fulei_card(ctx.catalog, trick) {
                    super::judge_hearth::resolve_fulei_judge(
                        ctx,
                        owner,
                        trick,
                        judge_card,
                        suit,
                        shenyi_voids,
                        machine_holds,
                    );
                    return (trick_kind, suit);
                }
                let mut hit = suit.is_spade() && (2..=9).contains(&rank);
                if crate::rvm_bridge::modifiers::shandian_reversed(ctx.core, ctx.catalog, owner) {
                    hit = !hit;
                }
                if !shenyi_voids && hit {
                    ctx.to_discard(trick);
                    if !machine_holds {
                        ctx.to_discard(judge_card);
                    }
                    ctx.emit(EventData::Damage(damage_data_with(
                        None,
                        owner,
                        3,
                        crate::event::DamageNature::Thunder,
                        Some(trick),
                    )));
                    return (trick_kind, suit);
                }
                // 未命中：传给下一位可放置闪电的存活玩家；无人可放则直接弃置
                // （对齐旧线 addJudgeNext + pass_lightning + wenhejibei canAddJudge）。
                let pass_to = if ctx.catalog.vm_card_history {
                    let mut cand = ctx.core.next_alive(owner);
                    let mut chosen = None;
                    for _ in 0..ctx.core.players.len() {
                        let has_shandian = ctx.core.player(cand).judge.iter().any(|c| {
                            ctx.catalog.resolved_card_kind(ctx.core, *c) == CardKind::ShanDian
                                && !super::judge_hearth::is_fulei_card(ctx.catalog, *c)
                        });
                        let blocked = crate::rvm_bridge::modifiers::eval_card_target_canceled(
                            ctx.core,
                            ctx.catalog,
                            owner,
                            cand,
                            trick,
                            CardKind::ShanDian,
                            crate::event::DamageNature::Thunder,
                        );
                        let disabled = ctx
                            .core
                            .vm_aux
                            .flags
                            .get(&(cand, "l11_judge_slot_disabled".to_string()))
                            .copied()
                            .unwrap_or(0)
                            > 0;
                        if ctx.core.player(cand).alive && !has_shandian && !blocked && !disabled {
                            chosen = Some(cand);
                            break;
                        }
                        let next_cand = ctx.core.next_alive(cand);
                        if next_cand == cand {
                            break;
                        }
                        cand = next_cand;
                    }
                    chosen
                } else {
                    let next = ctx.core.next_alive(owner);
                    let next_has_shandian = ctx.core.player(next).judge.iter().any(|c| {
                        ctx.catalog.resolved_card_kind(ctx.core, *c) == CardKind::ShanDian
                            && !super::judge_hearth::is_fulei_card(ctx.catalog, *c)
                    });
                    let disabled = ctx
                        .core
                        .vm_aux
                        .flags
                        .get(&(next, "l11_judge_slot_disabled".to_string()))
                        .copied()
                        .unwrap_or(0)
                        > 0;
                    (!next_has_shandian && !disabled).then_some(next)
                };
                if let Some(dest) = pass_to {
                    ctx.core.player_mut(dest).judge.push(trick);
                    if dest != owner {
                        crate::kernel::trigger_qiaoqiao_add_judge(ctx, dest);
                    }
                } else {
                    ctx.to_discard(trick);
                }
                if !machine_holds {
                    ctx.to_discard(judge_card);
                }
            }
            _ => {
                ctx.to_discard(trick);
                if !machine_holds {
                    ctx.to_discard(judge_card);
                }
            }
        }
        (trick_kind, suit)
    }
}

impl crate::machine::Ctx<'_> {
    /// 内联 `draw_judge_card` 判定牌的有效花色与点数（L11 皇天族改判）。
    pub fn inline_judge_suit_rank(
        &self,
        owner: PlayerId,
        card: crate::ids::CardId,
    ) -> (crate::catalog::Suit, u8) {
        let raw_def = &self.catalog.cards[card as usize];
        if self.catalog.vm_card_history {
            crate::rvm_bridge::modifiers::apply_huangtian_suit_rank(
                self.core,
                self.catalog,
                owner,
                raw_def.suit,
                raw_def.rank,
            )
        } else {
            (raw_def.suit, raw_def.rank)
        }
    }

    /// 判定牌已进入弃牌堆时，若场上存在 `txhj_zhuwei` 等 `L11GeneralBossC`
    /// `OrdinaryJudge` 监听技，则补发已结算 `OrdinaryJudge { owner, invocation: jc }`
    /// 并标记 `l11_resolved_ordinary_judge:{jc}` 以免 `intrinsic` 误作前判定启动。
    pub fn emit_zhuwei_ordinary_judge_if_needed(
        &mut self,
        owner: PlayerId,
        judge_card: crate::ids::CardId,
    ) -> bool {
        if !self.catalog.vm_card_history || !self.core.discard.contains(&judge_card) {
            return false;
        }
        let resolved = EventData::OrdinaryJudge {
            owner,
            invocation: judge_card,
        };
        let qctx = self.query();
        let has_zhuwei = self.core.skills.iter().any(|((s_owner, sdef), inst)| {
            let def_entry = &self.catalog.skills[*sdef as usize];
            inst.enabled
                && self.core.player(*s_owner).alive
                && def_entry.behavior.as_host_behavior()
                    == Some(crate::vocabulary::host::HostBehavior::L11GeneralBossC)
                && def_entry.trigger.as_ref().is_some_and(|m| {
                    m.event == crate::event::EventKind::OrdinaryJudge
                        || m.extra_events
                            .contains(&crate::event::EventKind::OrdinaryJudge)
                })
                && def_entry.behavior.filter_in_window(
                    &qctx,
                    *s_owner,
                    &resolved,
                    crate::event::EventKind::OrdinaryJudge,
                )
        });
        if has_zhuwei {
            self.core.vm_aux.flags.insert(
                (owner, format!("l11_resolved_ordinary_judge:{judge_card}")),
                1,
            );
            self.emit(resolved);
            return true;
        }
        false
    }

    /// 内联 `draw_judge_card` + `to_discard` 结算后的通用收尾：
    /// 触发 `on_judge_settled` 钩子并在判定牌入弃堆时派发 `txhj_zhuwei` 等后置窗口。
    pub fn settle_inline_judge_card(
        &mut self,
        owner: PlayerId,
        suit: crate::catalog::Suit,
        judge_card: crate::ids::CardId,
    ) -> bool {
        if !self.core.discard.contains(&judge_card) {
            self.to_discard(judge_card);
        }
        if !self.catalog.vm_card_history {
            return false;
        }
        let mut spawned =
            (self.catalog.hooks.on_judge_settled)(self, owner, None, suit, judge_card);
        spawned |= self.emit_zhuwei_ordinary_judge_if_needed(owner, judge_card);
        spawned
    }
}
