//! 杀结算的挂起应答恢复（复仇、霰弹、掠影与各武器/防具询问）：有待恢复的询问时返回本步结果。

use super::*;

impl ShaResolveMachine {
    /// 杀结算的挂起应答恢复（复仇、霰弹、掠影与各武器/防具询问）：有待恢复的询问时返回本步结果。
    pub(super) fn resume_pending_answer(&mut self, ctx: &mut Ctx<'_>) -> Option<Flow> {
        // 复仇（玩点论杀批 14，规则 v279）：成为杀目标时弃 1 张手牌取消
        // 该杀（上游 target shaBefore direct chooseCard 同面；所选牌归来
        // 源+标记来源供 fuchou2 回合末结算；杀回来源手=上游 cancel 回手
        // 同口径）。取消（空应答）则照常响应流程。
        if !self.asking_fuchou && self.fuchou_card.is_none() {
            let t = self.target;
            let has = ctx
                .catalog
                .skill_by_name(crate::vocabulary::host::WUDIAN_FUCHOU_KEY)
                .map(|d| ctx.core.has_skill(t, d))
                .unwrap_or(false);
            if has && !ctx.core.player(t).hand.is_empty() {
                self.asking_fuchou = true;
                let options: Vec<ChoiceOption> = ctx
                    .core
                    .player(t)
                    .hand
                    .iter()
                    .copied()
                    .map(ChoiceOption::Card)
                    .collect();
                return Some(Flow::WaitChoice {
                    observer: t,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "wudian_fuchou_cancel",
                        options,
                        min: 1,
                        max: 1,
                        allow_cancel: true,
                    },
                });
            }
        } else if self.asking_fuchou {
            self.asking_fuchou = false;
            let t = self.target;
            let picked = ctx.answer().iter().find_map(|o| match o {
                ChoiceOption::Card(c) => Some(*c),
                _ => None,
            });
            if let Some(c) = picked {
                self.fuchou_card = Some(c);
                ctx.core.table.retain(|x| *x != self.card);
                ctx.core.player_mut(self.source).hand.push(self.card);
                ctx.core.player_mut(t).hand.retain(|x| *x != c);
                ctx.core.player_mut(self.source).hand.push(c);
                ctx.core.player_mut(t).fuchou_marked.push(self.source);
                ctx.log(LogEvent::CardsGiven {
                    from: t,
                    to: self.source,
                    cards: vec![c],
                });
                return Some(Flow::Done); // 杀整体取消
            }
        }
        // 寒冰剑（逐技对照批一，规则 v75）：目标自选弃牌恢复——上游
        // discardPlayerCard("he", forced) 逐张强制弃，目标从手牌+装备
        // 自选；次数用尽或目标无牌可弃后跳过伤害。
        // 霰弹（U05 ow 死神，规则 v140）：弃牌应答——红=直击（哨卫同
        // 款直中路径）；黑=本杀伤害 +1；取消=照常进入防具/闪询问。
        if self.asking_xiandan {
            self.asking_xiandan = false;
            if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                let red = matches!(
                    ctx.catalog.cards[c as usize].suit,
                    crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                );
                let in_hand = ctx.core.player(self.source).hand.contains(&c);
                if in_hand {
                    ctx.discard_from_hand(self.source, c);
                } else if ctx.core.player(self.source).equips.contains(&c) {
                    ctx.core.player_mut(self.source).equips.retain(|x| *x != c);
                    ctx.to_discard(c);
                }
                ctx.log(LogEvent::CardDiscarded {
                    player: self.source,
                    cards: vec![c],
                });
                if red {
                    if ctx.core.player(self.target).alive {
                        return Some(self.emit_sha_damage(ctx));
                    }
                    return Some(self.advance_target(ctx));
                }
                self.xiandan_bonus = true;
            }
            // 黑弃/取消：复位 asked 落回 !asked 分支重开出闪询问（青龙
            // 再杀 reboot 同款——否则霰弹应答会被门控应答区误读）。
            self.asked = false;
        }
        // 掠影（规则 v154；v194 两相协议）：相以应答形状区分——确认相
        // 只会收到 Confirm（确认）或空（放弃），选人相只会收到 Player。
        // Confirm=消耗 usable、获得目标 he 随机一张，仍有其他有牌者则
        // 转入目标选人相；空=放弃（不消耗 usable，重开闪询问）；Player=
        // 受害者选定，来源随机弃其 he 一张后复位 asked。
        if self.asking_lueying {
            match ctx.answer().first().copied() {
                Some(ChoiceOption::Player(v)) => {
                    self.asking_lueying = false;
                    self.lueying_victim = Some(v);
                    let pool: Vec<crate::ids::CardId> = {
                        let p = ctx.core.player(v);
                        p.hand
                            .iter()
                            .copied()
                            .chain(p.equips.iter().copied())
                            .collect()
                    };
                    if !pool.is_empty() {
                        let k = ctx.core.rng.gen_range(pool.len() as u32) as usize;
                        let dc = pool[k];
                        ctx.take_owned_card(v, dc);
                        ctx.to_discard(dc);
                        ctx.log(LogEvent::CardDiscarded {
                            player: v,
                            cards: vec![dc],
                        });
                    }
                    self.asked = false;
                }
                Some(ChoiceOption::Confirm) => {
                    ctx.core.turn.lueying_used = true;
                    // 获得目标 he 随机一张
                    let pool: Vec<crate::ids::CardId> = {
                        let p = ctx.core.player(self.target);
                        p.hand
                            .iter()
                            .copied()
                            .chain(p.equips.iter().copied())
                            .collect()
                    };
                    if !pool.is_empty() {
                        let k = ctx.core.rng.gen_range(pool.len() as u32) as usize;
                        let card = pool[k];
                        ctx.take_owned_card(self.target, card);
                        ctx.core.player_mut(self.source).hand.push(card);
                        ctx.log(LogEvent::CardsGiven {
                            from: self.target,
                            to: self.source,
                            cards: vec![card],
                        });
                    }
                    // 仍有其他有牌者→目标选人相；否则本技结束
                    let others: Vec<PlayerId> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            *p != self.source
                                && *p != self.target
                                && (!ctx.core.player(*p).hand.is_empty()
                                    || !ctx.core.player(*p).equips.is_empty())
                        })
                        .collect();
                    if others.is_empty() {
                        self.asking_lueying = false;
                        self.asked = false;
                    } else {
                        let options: Vec<ChoiceOption> =
                            others.into_iter().map(ChoiceOption::Player).collect();
                        return Some(Flow::WaitChoice {
                            observer: self.target,
                            spec: ChoiceSpec {
                                kind: ChoiceKind::SelectPlayers,
                                prompt: "lueying_choose_victim",
                                options,
                                min: 1,
                                max: 1,
                                allow_cancel: false,
                            },
                        });
                    }
                }
                _ => {
                    // 放弃：不消耗 usable（同上游 usable 口径），标记本杀
                    // 已问过（阻止 step 重入再问），重开闪询问
                    self.asking_lueying = false;
                    self.lueying_declined = true;
                    self.asked = false;
                }
            }
        }
        if self.hanbing_pending > 0 {
            let pick = ctx.answer().first().and_then(|o| match o {
                ChoiceOption::Card(c) => Some(*c),
                _ => None,
            });
            self.hanbing_pending -= 1;
            if let Some(c) = pick {
                if ctx.core.player(self.target).hand.contains(&c) {
                    ctx.discard_from_hand(self.target, c);
                } else if ctx.core.player(self.target).equips.contains(&c) {
                    ctx.core.player_mut(self.target).equips.retain(|x| *x != c);
                    crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, self.target, c);
                    ctx.to_discard(c);
                }
            }
            if self.hanbing_pending > 0 && self.target_has_he(ctx) {
                return Some(self.open_hanbing_target_ask(ctx));
            }
            self.hanbing_pending = 0;
            return Some(self.advance_target(ctx));
        }
        // 寒冰剑（批③-b 第三把）：来源防止伤害确认恢复——确认后转入
        // 目标自选弃牌（两次；逐技对照批一消除「自动依次」差异）。
        // 风雪刃恢复：确认→目标翻面+摸 1（无伤害）；放弃→照常伤害
        if self.asking_fengxueren {
            self.asking_fengxueren = false;
            let confirmed = matches!(ctx.answer().first(), Some(ChoiceOption::Confirm));
            if confirmed {
                let flipped = {
                    let t = ctx.core.player_mut(self.target);
                    t.face_down = !t.face_down;
                    t.face_down
                };
                ctx.emit(EventData::TurnOvered {
                    player: self.target,
                    face_down: flipped,
                });
                ctx.draw_cards(self.target, 1);
                return Some(self.advance_target(ctx));
            }
            return Some(self.emit_sha_damage(ctx));
        }
        if self.asking_hanbing {
            self.asking_hanbing = false;
            let confirmed = matches!(ctx.answer().first(), Some(ChoiceOption::Confirm));
            if confirmed {
                self.hanbing_pending = 2;
                if self.target_has_he(ctx) {
                    return Some(self.open_hanbing_target_ask(ctx));
                }
                self.hanbing_pending = 0;
                return Some(self.advance_target(ctx));
            }
            // 放弃：照常进入麒麟弓/伤害结算（单武器互斥，实际直接伤害）。
            return Some(self.emit_sha_damage(ctx));
        }
        // 贯石斧（批③-b 第五把）：弃二询问恢复——按应答弃牌，强制伤害。
        if self.asking_guanshi {
            self.asking_guanshi = false;
            let picks: Vec<crate::ids::CardId> = ctx
                .answer()
                .iter()
                .filter_map(|c| match c {
                    ChoiceOption::Card(card) => Some(*card),
                    _ => None,
                })
                .collect();
            let fired = picks.len() == 2;
            for card in picks {
                let in_hand = ctx.core.player(self.source).hand.contains(&card);
                if in_hand {
                    ctx.discard_from_hand(self.source, card);
                } else if ctx.core.player(self.source).equips.contains(&card) {
                    ctx.core
                        .player_mut(self.source)
                        .equips
                        .retain(|x| *x != card);
                    ctx.to_discard(card);
                }
            }
            self.guanshi_cards.clear();
            // 恰弃两张 → 强制伤害（被闪仍命中）；放弃 → 无伤。
            if fired && ctx.core.player(self.target).alive {
                ctx.emit(EventData::Damage(damage_data_with(
                    Some(self.source),
                    self.target,
                    1,
                    self.nature,
                    Some(self.card),
                )));
            }
            return Some(self.advance_target(ctx));
        }
        // 青龙偃月刀（批③-b 第六把）：再杀询问恢复——弃置选中的杀并
        // 以其重启对同一目标的杀结算（防/闪/贯石斧/麒麟弓流程全部重走）。
        if self.asking_qinglong {
            self.asking_qinglong = false;
            if let Some(ChoiceOption::Card(new_sha)) = ctx.answer().first().copied() {
                if ctx.core.player(self.source).hand.contains(&new_sha) {
                    ctx.discard_from_hand(self.source, new_sha);
                    ctx.emit(EventData::CardResponded {
                        card: new_sha,
                        player: self.source,
                    });
                    self.card = new_sha;
                    self.asked = false;
                    self.armor_judged = false;
                    self.dodges_used = 0;
                    self.tieji_red = None;
                    self.liuli_done = false;
                    self.liuli_stage = 0;
                    self.liuli_card = None;
                    return Some(Flow::Yield);
                }
            }
            // 放弃：当前目标按被闪结算（无伤）→ 推进队列/结束。
            return Some(self.advance_target(ctx));
        }
        // 萨登荆环（swd 卡牌面窗口机批）：被闪后判定确认恢复——确定性判
        // 定（内联口径同铁骑，登记差异：无鬼道改判窗）；红色→目标额外
        // 出闪询问，未打出则本杀改为命中（上游 shaMiss→judge→shaHit 改
        // 写同口径）；确认放弃或判定黑→被闪结算维持。
        if self.asking_sadeng {
            self.asking_sadeng = false;
            let confirmed = matches!(ctx.answer().first(), Some(ChoiceOption::Confirm));
            if confirmed && ctx.core.player(self.target).alive {
                let judged = ctx.draw_judge_card();
                let red = matches!(
                    ctx.catalog.cards[judged as usize].suit,
                    crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                );
                ctx.to_discard(judged);
                if red {
                    let longdan = ctx
                        .catalog
                        .skill_by_name(crate::vocabulary::names::LONG_DAN)
                        .map(|d| ctx.core.has_skill(self.target, d))
                        .unwrap_or(false);
                    let qingguo = ctx
                        .catalog
                        .skill_by_name(crate::vocabulary::names::QING_GUO)
                        .map(|d| ctx.core.has_skill(self.target, d))
                        .unwrap_or(false);
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.target)
                        .hand
                        .iter()
                        .filter(|c| {
                            let kind = ctx.catalog.card_kind(**c);
                            let black = matches!(
                                ctx.catalog.cards[**c as usize].suit,
                                crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                            );
                            kind == CardKind::Shan
                                || (longdan && kind.is_sha_like())
                                || (qingguo && black)
                        })
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    if !options.is_empty() {
                        self.asking_sadeng_shan = true;
                        return Some(Flow::WaitChoice {
                            observer: self.target,
                            spec: ChoiceSpec {
                                kind: ChoiceKind::SelectCards,
                                prompt: "sadengjinhuan_extra_shan",
                                options,
                                min: 0,
                                max: 1,
                                allow_cancel: false,
                            },
                        });
                    }
                    // 无闪可打：改为命中（照常走寒冰/麒麟窗）
                    return Some(self.sadeng_continue_to_damage(ctx));
                }
            }
            // 未确认/判定黑/目标已死：被闪结算维持 → 推进
            return Some(self.advance_target(ctx));
        }
        // 萨登荆环：额外出闪恢复——打出则本杀仍被闪避；未打出则改为
        // 命中（照常走寒冰/麒麟窗）。
        if self.asking_sadeng_shan {
            self.asking_sadeng_shan = false;
            if let Some(ChoiceOption::Card(shan)) = ctx.answer().first().copied() {
                if ctx.core.player(self.target).hand.contains(&shan) {
                    ctx.discard_from_hand(self.target, shan);
                    ctx.emit(EventData::CardResponded {
                        card: shan,
                        player: self.target,
                    });
                    return Some(self.advance_target(ctx));
                }
            }
            if ctx.core.player(self.target).alive {
                return Some(self.sadeng_continue_to_damage(ctx));
            }
            return Some(self.advance_target(ctx));
        }
        // 雌雄双股剑（批③-b 第八把；逐技对照批一 v75 消除首张差异）：
        // 弃牌/令摸恢复——上游 chooseToDiscard 一张手牌、可取消（取消=
        // 令来源摸 1），随后照常进入防具/出闪流程。
        if self.asking_cixiong {
            self.asking_cixiong = false;
            self.asked = false;
            let pick = ctx.answer().first().and_then(|o| match o {
                ChoiceOption::Card(c) => Some(*c),
                _ => None,
            });
            if let Some(c) = pick.filter(|c| ctx.core.player(self.target).hand.contains(c)) {
                ctx.discard_from_hand(self.target, c);
            } else if ctx.core.player(self.source).alive {
                ctx.draw_cards(self.source, 1);
            }
            // 落入下方 !asked 分支重开出闪询问（防具判定已由
            // armor_judged 标记幂等）。
        }
        // 麒麟弓（批③-b 第二把）：弃马询问恢复——按应答弃马，随后结算伤害。
        if self.asking_qilin {
            self.asking_qilin = false;
            // 空应答 = 来源放弃弃马。
            if let Some(ChoiceOption::Card(horse)) = ctx.answer().first().copied() {
                if ctx.core.player(self.target).equips.contains(&horse) {
                    ctx.core
                        .player_mut(self.target)
                        .equips
                        .retain(|x| *x != horse);
                    ctx.to_discard(horse);
                }
            }
            self.qilin_horse = None;
            self.emit_sha_damage(ctx);
            return Some(Flow::Done);
        }
        None
    }
}
