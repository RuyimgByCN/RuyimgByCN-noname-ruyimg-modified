//! 武典（wudian）守望先锋（ow）子包宿主技：目镜、光书、炮台、扩音、呼唤、链钩、音波、强固、跳跃、屏障。

use super::super::*;

impl HostMachine {
    pub(super) fn step_wudian_ow_cards(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            HostBehavior::WudianMujing => match self.stage {
                // 目镜（U07 ow，规则 v118）：Act 问黑色牌 → SelectCard 问
                // 目标 → declared-Sha 结算（塔击同构）。
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| {
                            matches!(
                                ctx.catalog.cards[*c as usize].suit,
                                crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                            )
                        })
                        .map(ChoiceOption::Card)
                        .collect();
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_MUJING_CARD,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(basic) = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(self.owner).hand.contains(&basic) {
                        return Flow::Done;
                    }
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| *p != self.owner)
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.cards = vec![basic];
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: WUDIAN_MUJING_TARGET,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(basic) = self.cards.first().copied() else {
                        return Flow::Done;
                    };
                    let Some(t) = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(t).alive
                        || !ctx.core.player(self.owner).hand.contains(&basic)
                    {
                        return Flow::Done;
                    }
                    // 素材牌留手牌由杀结算消费（丈八 TargetRead 同构）
                    ctx.core.declared_kinds.insert(basic, CardKind::Sha);
                    ctx.emit(EventData::CardUsed(crate::machine::card_use_as(
                        basic,
                        self.owner,
                        vec![t],
                        Some(CardKind::Sha),
                    )));
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianGuangshu => match self.stage {
                // 光书（U05 ow 批二 v119）：出牌阶段弃 he 一张并指定一名
                // 角色，按弃牌花色授对应子技（♥受伤回1/♦伤害后摸2/
                // ♣至下阶段结束不能用杀/♠下阶段结束受1无源雷伤；各一次
                // 性移除；已有任意子技者不可指定）。
                HostStage::Act => {
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
                            prompt: WUDIAN_GUANGSHU_CARD,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    self.cards = vec![c];
                    // 目标：四子技同查排除（已有任意子技者不可指定）
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .into_iter()
                        .filter(|t| {
                            *t != self.owner
                                && ![
                                    WUDIAN_GUANGSHU_HEART_KEY,
                                    WUDIAN_GUANGSHU_DIAMOND_KEY,
                                    WUDIAN_GUANGSHU_CLUB_KEY,
                                    WUDIAN_GUANGSHU_SPADE_KEY,
                                ]
                                .iter()
                                .any(|k| {
                                    ctx.catalog
                                        .skill_by_name(k)
                                        .map(|d| ctx.core.skills.contains_key(&(*t, d)))
                                        .unwrap_or(false)
                                })
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
                            prompt: WUDIAN_GUANGSHU_TARGET,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    let Some(c) = self.cards.first().copied() else {
                        return Flow::Done;
                    };
                    // 按弃牌花色授对应子技实例
                    let suit = ctx.catalog.cards[c as usize].suit;
                    let key = match suit {
                        crate::catalog::Suit::Heart => WUDIAN_GUANGSHU_HEART_KEY,
                        crate::catalog::Suit::Diamond => WUDIAN_GUANGSHU_DIAMOND_KEY,
                        crate::catalog::Suit::Club => WUDIAN_GUANGSHU_CLUB_KEY,
                        crate::catalog::Suit::Spade => WUDIAN_GUANGSHU_SPADE_KEY,
                    };
                    if let Some(def) = ctx.catalog.skill_by_name(key) {
                        ctx.insert_skill(
                            (t, def),
                            crate::state::SkillInstance {
                                owner: t,
                                enabled: true,
                                once_used: false,
                                expires_end_of_turn: false,
                                rounds: 0,
                            },
                        );
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianPaotai => match self.stage {
                // 炮台（U05 ow，规则 v123）：Act 弃一张杀 → 布置/升级
                //（≤3；层数跨回合持久入 PlayerState）。
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| ctx.catalog.card_kind(*c).is_sha_like())
                        .map(ChoiceOption::Card)
                        .collect();
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_PAOTAI_CARD,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(c) = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(self.owner).hand.contains(&c)
                        || !ctx.catalog.card_kind(c).is_sha_like()
                    {
                        return Flow::Done;
                    }
                    ctx.discard_from_hand(self.owner, c);
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![c],
                    });
                    if ctx.core.player(self.owner).paotai_lv < 3 {
                        ctx.core.player_mut(self.owner).paotai_lv += 1;
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianKuoyin => match self.stage {
                // 扩音（U05 ow 卢西奥，规则 v126）：弃 1 张→乐动目标数
                // +2；弃 2 张→乐动摸/回复量 +1。选择张数由存量决定：
                // 量已 +1→恰 1 张；数已 +2→恰 2 张；否则 1–2 张（上游
                // selectCard 同口径；he 区域=手牌+装备）。
                HostStage::SelectCard => {
                    let p = ctx.core.player(self.owner);
                    let mut options: Vec<ChoiceOption> =
                        p.hand.iter().copied().map(ChoiceOption::Card).collect();
                    options.extend(p.equips.iter().copied().map(ChoiceOption::Card));
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    let (min, max) = if ctx.core.turn.yuedong_eff > 0 {
                        (1, 1)
                    } else if ctx.core.turn.yuedong_num > 0 {
                        (2, 2)
                    } else {
                        (1, 2)
                    };
                    self.stage = HostStage::Act;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_KUOYIN_CARDS,
                            options,
                            min,
                            max,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::Act => {
                    let sel: Vec<CardId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        })
                        .collect();
                    if sel.is_empty() {
                        return Flow::Done; // 弃权
                    }
                    for c in &sel {
                        ctx.take_owned_card(self.owner, *c);
                        ctx.to_discard(*c);
                    }
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: sel.clone(),
                    });
                    if sel.len() == 1 {
                        ctx.core.turn.yuedong_num = ctx.core.turn.yuedong_num.saturating_add(2);
                    } else {
                        ctx.core.turn.yuedong_eff = ctx.core.turn.yuedong_eff.saturating_add(1);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianHuhuan => match self.stage {
                // 呼唤（U05 ow 卢西奥，规则 v126）：弃 2 张牌，本回合乐动
                // 改为回复模式（限一次——recover 已置位时 activate 拒绝）。
                HostStage::SelectCard => {
                    let p = ctx.core.player(self.owner);
                    let mut options: Vec<ChoiceOption> =
                        p.hand.iter().copied().map(ChoiceOption::Card).collect();
                    options.extend(p.equips.iter().copied().map(ChoiceOption::Card));
                    if options.len() < 2 {
                        return Flow::Done;
                    }
                    self.stage = HostStage::Act;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_HUHUAN_CARDS,
                            options,
                            min: 2,
                            max: 2,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::Act => {
                    let sel: Vec<CardId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        })
                        .collect();
                    if sel.len() < 2 {
                        return Flow::Done; // 弃权
                    }
                    for c in &sel {
                        ctx.take_owned_card(self.owner, *c);
                        ctx.to_discard(*c);
                    }
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: sel.clone(),
                    });
                    ctx.core.turn.yuedong_recover = true;
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianLiangou => match self.stage {
                // 链钩（U05 ow 路霸，规则 v128）：弃 1 张 he（必选）→
                // 选目标（必选）→ 弃置+判定（非红桃成功）→ 挂钩
                // TurnState.liangou_hook（本回合内：距离恒 1+其首次受
                // 伤 +1；换回合随 TurnState 清零解除）。
                HostStage::SelectCard => {
                    let p = ctx.core.player(self.owner);
                    let mut options: Vec<ChoiceOption> =
                        p.hand.iter().copied().map(ChoiceOption::Card).collect();
                    options.extend(p.equips.iter().copied().map(ChoiceOption::Card));
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_LIANGOU_CARDS,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    // 收「牌选择」应答：存入 self.cards，前移 Act 问目标
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
                    self.stage = HostStage::Act;
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
                            prompt: WUDIAN_LIANGOU_TARGET,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::Act => {
                    // 收「目标选择」应答：弃牌+判定+挂钩
                    let Some(t) = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    let sel = std::mem::take(&mut self.cards);
                    for c in &sel {
                        ctx.take_owned_card(self.owner, *c);
                        ctx.to_discard(*c);
                    }
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: sel,
                    });
                    // 判定：非红桃成功（上游 judge suit!=="heart"）
                    let judged = ctx.draw_judge_card();
                    let suit = ctx.catalog.cards[judged as usize].suit;
                    ctx.log(LogEvent::Judged {
                        player: self.owner,
                        card: judged,
                    });
                    ctx.to_discard(judged);
                    if suit != crate::catalog::Suit::Heart {
                        ctx.core.turn.liangou_hook = Some(t);
                        ctx.core.turn.liangou_bonus_used = false;
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianYinbo => match self.stage {
                // 音波（U05 ow 半藏，规则 v135）：弃 1 张黑桃→随机 ≤3
                // 名有牌的其他角色各随机弃 1 张（上游 randomGets(3)+
                // sort seat+randomGet 同口径；登记差异：原型无阵营——
                // enemies=其他存活玩家）。
                HostStage::SelectCard => {
                    let p = ctx.core.player(self.owner);
                    let options: Vec<ChoiceOption> = p
                        .hand
                        .iter()
                        .copied()
                        .chain(p.equips.iter().copied())
                        .filter(|c| {
                            matches!(
                                ctx.catalog.cards[*c as usize].suit,
                                crate::catalog::Suit::Spade
                            )
                        })
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::Act;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_YINBO_CARDS,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::Act => {
                    let Some(c) = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    ctx.take_owned_card(self.owner, c);
                    ctx.to_discard(c);
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![c],
                    });
                    // 候选=其他存活且 he 非空；随机取 ≤3（龙帆 swap_remove
                    // 抽取式）
                    let mut cands: Vec<PlayerId> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            *p != self.owner
                                && (!ctx.core.player(*p).hand.is_empty()
                                    || !ctx.core.player(*p).equips.is_empty())
                        })
                        .collect();
                    let take = cands.len().min(3);
                    for _ in 0..take {
                        let j = ctx.core.rng.gen_range(cands.len() as u32) as usize;
                        let t = cands.swap_remove(j);
                        // he 池随机弃 1（暴雪同口径：手牌+装备）
                        let pool: Vec<CardId> = {
                            let p = ctx.core.player(t);
                            p.hand
                                .iter()
                                .copied()
                                .chain(p.equips.iter().copied())
                                .collect()
                        };
                        let k = ctx.core.rng.gen_range(pool.len() as u32) as usize;
                        let dc = pool[k];
                        ctx.take_owned_card(t, dc);
                        ctx.to_discard(dc);
                        ctx.log(LogEvent::CardDiscarded {
                            player: t,
                            cards: vec![dc],
                        });
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianQianggu => match self.stage {
                // 强固（U05 ow 奥丽莎，规则 v136）：弃 2 张 he（必选）→
                // +2 护甲+强固置位（自己下回合判定阶段开始解除）。
                HostStage::SelectCard => {
                    let p = ctx.core.player(self.owner);
                    let mut options: Vec<ChoiceOption> =
                        p.hand.iter().copied().map(ChoiceOption::Card).collect();
                    options.extend(p.equips.iter().copied().map(ChoiceOption::Card));
                    if options.len() < 2 {
                        return Flow::Done;
                    }
                    self.stage = HostStage::Act;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_QIANGGU_CARDS,
                            options,
                            min: 2,
                            max: 2,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::Act => {
                    let sel: Vec<CardId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        })
                        .collect();
                    if sel.len() < 2 {
                        return Flow::Done;
                    }
                    for c in &sel {
                        ctx.take_owned_card(self.owner, *c);
                        ctx.to_discard(*c);
                    }
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: sel,
                    });
                    ctx.change_hujia(self.owner, 2);
                    ctx.core.player_mut(self.owner).qianggu_active = true;
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianQiangguGuard => match self.stage {
                // 强固守卫（U05 ow 奥丽莎，规则 v136）：Act 问使用者——
                // 选一张基本牌=弃置放行；Pass/无基本牌=杀无效（帧
                // cancelled，Intrinsic 按 cancelled 跳过结算——筑城守卫
                // 同款）。
                HostStage::Act => {
                    let Some(attacker) = self.target else {
                        return Flow::Done;
                    };
                    let mut options = vec![ChoiceOption::Pass];
                    options.extend(
                        ctx.core
                            .player(attacker)
                            .hand
                            .iter()
                            .copied()
                            .filter(|c| {
                                ctx.catalog.card_kind(*c).aoe_need().is_none()
                                    && matches!(
                                        ctx.catalog.card_kind(*c),
                                        CardKind::Sha
                                            | CardKind::Shan
                                            | CardKind::Tao
                                            | CardKind::Jiu
                                    )
                            })
                            .map(ChoiceOption::Card),
                    );
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: attacker,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_QIANGGU_GUARD_ASK,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    match ctx.answer().first().copied() {
                        Some(ChoiceOption::Card(c)) => {
                            ctx.discard_from_hand(self.target.expect("attacker"), c);
                        }
                        _ => {
                            // 未弃基本牌：杀对持有者无效
                            ctx.current_card_use_mut().cancelled = true;
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianFeitiao => match self.stage {
                // 跳跃（U05 ow 温斯顿，规则 v137）：弃 1 张 he（必选）→
                // 选一名其他角色（必选）→ 弃置+其随机弃 1+置跳跃目标
                //（本回合自己算到其距离恒 1；换回合解除）。
                HostStage::SelectCard => {
                    let p = ctx.core.player(self.owner);
                    let options: Vec<ChoiceOption> = p
                        .hand
                        .iter()
                        .copied()
                        .chain(p.equips.iter().copied())
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
                            prompt: WUDIAN_FEITIAO_CARDS,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    // 收「牌选择」应答：存 cards 后前移 Act 问目标
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
                    self.stage = HostStage::Act;
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
                            prompt: WUDIAN_FEITIAO_TARGET,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::Act => {
                    // 收「目标选择」应答：结算
                    let Some(t) = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    let sel = std::mem::take(&mut self.cards);
                    for c in &sel {
                        ctx.take_owned_card(self.owner, *c);
                        ctx.to_discard(*c);
                    }
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: sel,
                    });
                    // 目标随机弃 1（he 池）
                    let pool: Vec<CardId> = {
                        let p = ctx.core.player(t);
                        p.hand
                            .iter()
                            .copied()
                            .chain(p.equips.iter().copied())
                            .collect()
                    };
                    if !pool.is_empty() {
                        let k = ctx.core.rng.gen_range(pool.len() as u32) as usize;
                        let dc = pool[k];
                        ctx.take_owned_card(t, dc);
                        ctx.to_discard(dc);
                        ctx.log(LogEvent::CardDiscarded {
                            player: t,
                            cards: vec![dc],
                        });
                    }
                    ctx.core.turn.feitiao_target = Some(t);
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianPingzhang => match self.stage {
                // 屏障（U05 ow 查莉娅，规则 v138）：Act 建对应花色弃牌选
                // 项（自己受伤=红桃/他人受伤=黑桃；可取消=不发动）→
                // SelectCard 弃置+伤害 -1+置对应标志（各每回合一次）。
                HostStage::Act => {
                    let hurt = self.target.unwrap_or(self.owner);
                    let self_case = hurt == self.owner;
                    let used = if self_case {
                        ctx.core.player(self.owner).pingzhang_self_used
                    } else {
                        ctx.core.player(self.owner).pingzhang_other_used
                    };
                    if used || !ctx.core.player(hurt).alive {
                        return Flow::Done;
                    }
                    let want = if self_case {
                        crate::catalog::Suit::Heart
                    } else {
                        crate::catalog::Suit::Spade
                    };
                    let options: Vec<ChoiceOption> = {
                        let p = ctx.core.player(self.owner);
                        p.hand
                            .iter()
                            .copied()
                            .chain(p.equips.iter().copied())
                            .filter(|c| ctx.catalog.cards[*c as usize].suit == want)
                            .map(ChoiceOption::Card)
                            .collect()
                    };
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.card = Some(crate::ids::CardId::from(self_case as u8));
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_PINGZHANG_CARDS,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(c) = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    }) else {
                        return Flow::Done; // 放弃：不发动
                    };
                    ctx.take_owned_card(self.owner, c);
                    ctx.to_discard(c);
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![c],
                    });
                    // 伤害 -1（前置窗口直写帧量；上游 trigger.num--）
                    let d = ctx.current_damage_mut();
                    d.amount = d.amount.saturating_sub(1);
                    let self_case = self.target == Some(self.owner);
                    if self_case {
                        ctx.core.player_mut(self.owner).pingzhang_self_used = true;
                    } else {
                        ctx.core.player_mut(self.owner).pingzhang_other_used = true;
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            _ => unreachable!("behavior routed to the wrong step module"),
        }
    }
}
