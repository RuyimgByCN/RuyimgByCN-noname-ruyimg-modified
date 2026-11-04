//! 轩辕剑（swd）装备与宝物的持有效果（疾影、沐风、望尘、炼妖壶、空洞印、逆攻、四象之宝、神草结、女娲石）。

use super::super::*;

impl HostMachine {
    pub(super) fn step_swd_equipment(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            // 疾影：无效果步（sha_range_free 链查询）
            HostBehavior::SwdJiyingHeld => Flow::Done,
            // 沐风：确认门后**发现**一张牌（上游 `player.discoverCard()`：亮出 3 张选 1，
            // 余入弃堆）。第 973 轮由「摸 1」改为真正的 discover。
            HostBehavior::SwdMufengHeld => match self.stage {
                HostStage::Act => {
                    // 亮出牌堆顶最多 3 张，暂存到离场区，等待选择
                    let mut revealed = Vec::new();
                    for _ in 0..3 {
                        let Some(c) = ctx.core.deck.pop() else { break };
                        revealed.push(c);
                    }
                    if revealed.is_empty() {
                        return Flow::Done;
                    }
                    ctx.core
                        .player_mut(self.owner)
                        .off_court
                        .extend(revealed.iter().copied());
                    self.cards = revealed;
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "mufeng_discover",
                            options: self.cards.iter().copied().map(ChoiceOption::Card).collect(),
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let picked = match ctx.answer().first().copied() {
                        Some(ChoiceOption::Card(c)) => c,
                        _ => {
                            // 异常：把亮出的牌退回牌堆，结束
                            for c in std::mem::take(&mut self.cards) {
                                ctx.core.deck.push(c);
                            }
                            return Flow::Done;
                        }
                    };
                    let rest: Vec<_> = self
                        .cards
                        .iter()
                        .copied()
                        .filter(|&c| c != picked)
                        .collect();
                    self.cards.clear();
                    // 未选的入弃堆
                    for c in rest {
                        if let Some(pos) = ctx
                            .core
                            .player(self.owner)
                            .off_court
                            .iter()
                            .position(|x| *x == c)
                        {
                            ctx.core.player_mut(self.owner).off_court.remove(pos);
                        }
                        ctx.to_discard(c);
                    }
                    if let Some(pos) = ctx
                        .core
                        .player(self.owner)
                        .off_court
                        .iter()
                        .position(|x| *x == picked)
                    {
                        ctx.core.player_mut(self.owner).off_court.remove(pos);
                    }
                    ctx.core.player_mut(self.owner).hand.push(picked);
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 望尘：确认门后 TargetChoice 选任意角色翻面
            HostBehavior::SwdWangchenHeld => match self.stage {
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = (0..ctx.core.players.len())
                        .filter(|&i| ctx.core.player(i as PlayerId).alive)
                        .map(|i| ChoiceOption::Player(i as u8))
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "wangchen_target",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    self.target = ctx.answer().iter().find_map(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    });
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    if ctx.core.player(t).alive {
                        let flipped = {
                            let pl = ctx.core.player_mut(t);
                            pl.face_down = !pl.face_down;
                            pl.face_down
                        };
                        ctx.emit(EventData::TurnOvered {
                            player: t,
                            face_down: flipped,
                        });
                    }
                    ctx.core.turn.wangchen_discard_basic = false;
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 收纳：SelectCard 弃 1 手牌 → TargetChoice 其他有手牌角色 →
            // Act 目标首张手牌移入壶仓（登记差异上游随机取）
            HostBehavior::SwdLianyaohuShouna => match self.stage {
                HostStage::SelectCard => {
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
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "shouna_cost",
                            options,
                            min: 0,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(cost) = ctx.answer().iter().find_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    self.cards = vec![cost];
                    let options: Vec<ChoiceOption> = (0..ctx.core.players.len())
                        .filter(|&i| {
                            i as PlayerId != self.owner
                                && ctx.core.player(i as PlayerId).alive
                                && !ctx.core.player(i as PlayerId).hand.is_empty()
                        })
                        .map(|i| ChoiceOption::Player(i as u8))
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::Act;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "shouna_target",
                            options,
                            min: 0,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::Act => {
                    self.target = ctx.answer().iter().find_map(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    });
                    let (Some(cost), Some(t)) = (self.cards.first().copied(), self.target) else {
                        return Flow::Done;
                    };
                    if ctx.core.player(self.owner).hand.contains(&cost) {
                        ctx.discard_from_hand(self.owner, cost);
                        ctx.log(LogEvent::CardDiscarded {
                            player: self.owner,
                            cards: vec![cost],
                        });
                    }
                    if ctx.core.player(t).alive {
                        if let Some(c) = ctx.core.player(t).hand.first().copied() {
                            ctx.take_hand_card(t, c);
                            ctx.core.player_mut(self.owner).lianyaohu_store.push(c);
                            ctx.log(LogEvent::CardsGiven {
                                from: t,
                                to: self.owner,
                                cards: vec![c],
                            });
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 炼化：SelectCard 壶仓选 2 → Act 弃 2+牌堆顶首张类别均不同入手
            HostBehavior::SwdLianyaohuLianhua => match self.stage {
                HostStage::SelectCard => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .lianyaohu_store
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.len() < 2 {
                        return Flow::Done;
                    }
                    self.stage = HostStage::Act;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "lianhua_pick",
                            options,
                            min: 2,
                            max: 2,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::Act => {
                    let picked: Vec<CardId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        })
                        .collect();
                    if picked.len() < 2 {
                        return Flow::Done;
                    }
                    let mut types: Vec<u8> = Vec::new();
                    for c in &picked {
                        if let Some(pos) = ctx
                            .core
                            .player(self.owner)
                            .lianyaohu_store
                            .iter()
                            .position(|x| x == c)
                        {
                            ctx.core.player_mut(self.owner).lianyaohu_store.remove(pos);
                        }
                        types.push(ctx.catalog.cards[*c as usize].kind as u8);
                        ctx.to_discard(*c);
                        ctx.log(LogEvent::CardDiscarded {
                            player: self.owner,
                            cards: vec![*c],
                        });
                    }
                    // 牌堆序扫描首张类别均不同（上游 ui.cardPile 顺序同口径）
                    for i in 0..ctx.core.deck.len() {
                        let c = ctx.core.deck[i];
                        let k = ctx.catalog.cards[c as usize].kind as u8;
                        if !types.contains(&k) {
                            ctx.core.deck.remove(i);
                            ctx.core.player_mut(self.owner).hand.push(c);
                            ctx.log(LogEvent::CardsDrawn {
                                player: self.owner,
                                cards: vec![c],
                            });
                            break;
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 壶境：弃牌堆回收炼妖壶入装备区（宝物栏）
            HostBehavior::SwdHujing => {
                if let Some(pos) = ctx
                    .core
                    .discard
                    .iter()
                    .position(|c| ctx.catalog.card_kind(*c) == CardKind::Lianyaohu)
                {
                    let hu = ctx.core.discard.remove(pos);
                    // 同栏已有宝物则先卸下（commit_equip 同栏替换口径）
                    if let Some(epos) = ctx.core.player(self.owner).equips.iter().position(|c| {
                        ctx.catalog.card_slot_of(ctx.core, *c)
                            == Some(crate::catalog::EquipSlot::Treasure)
                    }) {
                        let old = ctx.core.player_mut(self.owner).equips.remove(epos);
                        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, self.owner, old);
                        ctx.to_discard(old);
                    }
                    ctx.core.player_mut(self.owner).equips.push(hu);
                    crate::kernel::grant_equip_skills(ctx, self.owner, hu);
                    ctx.log(LogEvent::CardsGiven {
                        from: self.owner,
                        to: self.owner,
                        cards: vec![hu],
                    });
                }
                Flow::Done
            }
            // 空洞印：死亡取消——hp=1、摸 1、弃全部宝物（death_cancelled
            // 基建消费，DeathMachine 开头拦截跳过死亡执行）
            HostBehavior::SwdKongdongyinGuard => match self.stage {
                HostStage::Act => {
                    ctx.core.player_mut(self.owner).death_cancelled = true;
                    ctx.core.player_mut(self.owner).hp = 1;
                    // R05-C2：摸 1 走 R 帧；宝物入弃堆在续行段（同步序＝
                    // 摸先于宝物入弃堆，洗牌边界下序敏感）。
                    ctx.start_draw(self.owner, 1);
                    self.stage = HostStage::DrawSettle;
                    Flow::Yield
                }
                // R05-C2 续行位：空洞印摸 1 交付后弃全部宝物。
                HostStage::DrawSettle => {
                    let baowu: Vec<CardId> = ctx
                        .core
                        .player(self.owner)
                        .equips
                        .iter()
                        .filter(|c| {
                            ctx.catalog.card_slot_of(ctx.core, **c)
                                == Some(crate::catalog::EquipSlot::Treasure)
                        })
                        .copied()
                        .collect();
                    for c in baowu {
                        if let Some(pos) = ctx
                            .core
                            .player(self.owner)
                            .equips
                            .iter()
                            .position(|x| *x == c)
                        {
                            let old = ctx.core.player_mut(self.owner).equips.remove(pos);
                            // 弃宝物同步撤销装备技实例（上游 onLose 同口径——
                            // 空洞印自身守卫技亦随之移除=一次性）
                            crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, self.owner, old);
                            ctx.to_discard(old);
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 逆攻积攒：标记 +amount（上限 4）
            HostBehavior::SwdNigongAccumulate => {
                if let Some(EventData::Damage(d)) =
                    ctx.res.frame(ctx.current_event_frame_id()).map(|f| &f.data)
                {
                    let lv = ctx.core.player(self.owner).nigong_lv;
                    ctx.core.player_mut(self.owner).nigong_lv = (lv + d.amount as u8).min(4);
                }
                Flow::Done
            }
            // 青龙之圭：摸 2 → 弃 1 张 he（SelectCard min1）
            HostBehavior::SwdQinglongzhiguiHeld => match self.stage {
                HostStage::Act => {
                    // R05-C2：摸 2 走 R 帧；弃牌选项（he）须含刚摸的牌
                    // → 续行位发询问。
                    ctx.start_draw(self.owner, 2);
                    self.stage = HostStage::DrawSettle;
                    Flow::Yield
                }
                // R05-C2 续行位：青龙之圭摸 2 交付后建弃牌询问。
                HostStage::DrawSettle => {
                    let options: Vec<ChoiceOption> = ctx
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
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "qinglongzhigui_discard",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    if let Some(c) = ctx.answer().iter().find_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    }) {
                        if ctx.core.player(self.owner).hand.contains(&c) {
                            ctx.discard_from_hand(self.owner, c);
                        } else if let Some(pos) = ctx
                            .core
                            .player(self.owner)
                            .equips
                            .iter()
                            .position(|x| *x == c)
                        {
                            let old = ctx.core.player_mut(self.owner).equips.remove(pos);
                            crate::kernel::revoke_equip_skills(
                                ctx.core,
                                ctx.catalog,
                                self.owner,
                                old,
                            );
                            ctx.to_discard(old);
                        }
                        ctx.log(LogEvent::CardDiscarded {
                            player: self.owner,
                            cards: vec![c],
                        });
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 朱雀之璋：来源火 1 伤（上游 fire；native 火属性同壳）
            HostBehavior::SwdZhuquezhizhangHeld => {
                if let Some(EventData::Damage(d)) =
                    ctx.res.frame(ctx.current_event_frame_id()).map(|f| &f.data)
                {
                    if let Some(src) = d.source {
                        if ctx.core.player(src).alive {
                            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                                Some(self.owner),
                                src,
                                1,
                                crate::event::DamageNature::Fire,
                                None,
                            )));
                        }
                    }
                }
                Flow::Done
            }
            // 白兽之琥：TargetChoice 选其他有牌角色 → Act 其首张 he 弃置
            //（确定性首张，登记差异上游随机）
            HostBehavior::SwdBaishouzhihuHeld => match self.stage {
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = (0..ctx.core.players.len())
                        .filter(|&i| {
                            let p = i as PlayerId;
                            p != self.owner
                                && ctx.core.player(p).alive
                                && (!ctx.core.player(p).hand.is_empty()
                                    || !ctx.core.player(p).equips.is_empty())
                        })
                        .map(|i| ChoiceOption::Player(i as u8))
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "baishouzhihu_target",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    self.target = ctx.answer().iter().find_map(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    });
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    // 首张 he 弃置（确定性首张，登记差异上游随机）
                    if let Some(c) = ctx
                        .core
                        .player(t)
                        .hand
                        .first()
                        .copied()
                        .or_else(|| ctx.core.player(t).equips.first().copied())
                    {
                        if ctx.core.player(t).hand.contains(&c) {
                            ctx.discard_from_hand(t, c);
                        } else if let Some(pos) =
                            ctx.core.player(t).equips.iter().position(|x| *x == c)
                        {
                            let old = ctx.core.player_mut(t).equips.remove(pos);
                            crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, t, old);
                            ctx.to_discard(old);
                        }
                        ctx.log(LogEvent::CardDiscarded {
                            player: t,
                            cards: vec![c],
                        });
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 玄武之璜：回复等量体力（伤害结算帧读 amount，hp<max 门在 filter）
            HostBehavior::SwdXuanwuzhihuangHeld => {
                if let Some(EventData::Damage(d)) =
                    ctx.res.frame(ctx.current_event_frame_id()).map(|f| &f.data)
                {
                    let p = ctx.core.player(self.owner);
                    if p.alive && p.hp < p.max_hp as i32 {
                        ctx.recover(self.owner, d.amount.max(0) as u32, None);
                    }
                }
                Flow::Done
            }
            // 黄麟之琮：+1 护甲（hujia==0 门在 filter）
            HostBehavior::SwdHuanglinzhicongHeld => {
                ctx.change_hujia(self.owner, 1);
                Flow::Done
            }
            // 神草结：确认门后弃一张神草结——受伤者是自己则伤-1、是自己
            // 造成则伤+1
            HostBehavior::SwdShencaojieGuard => {
                if let Some(EventData::Damage(dm)) =
                    ctx.res.frame(ctx.current_event_frame_id()).map(|f| &f.data)
                {
                    let toward_self = dm.target == self.owner;
                    if let Some(pos) = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .position(|c| ctx.catalog.card_kind(*c) == CardKind::Shencaojie)
                    {
                        let c = ctx.core.player_mut(self.owner).hand.remove(pos);
                        ctx.to_discard(c);
                        ctx.log(LogEvent::CardDiscarded {
                            player: self.owner,
                            cards: vec![c],
                        });
                        let frame = ctx
                            .res
                            .frame_mut(ctx.current_event_frame_id())
                            .expect("frame");
                        if let EventData::Damage(d2) = &mut frame.data {
                            if toward_self {
                                d2.amount = d2.amount.saturating_sub(1);
                            } else {
                                d2.amount += 1;
                            }
                        }
                    }
                }
                Flow::Done
            }
            // 逆攻发动：TargetChoice 选目标 → Act 弃全部标记——奇数摸 1+
            // floor(标记/2) 伤；标记=1 时仅摸 1（activate 分流两 activate 面）
            HostBehavior::SwdNigongStrike => match self.stage {
                HostStage::TargetChoice => {
                    let options: Vec<ChoiceOption> = (0..ctx.core.players.len())
                        .filter(|&i| {
                            i as PlayerId != self.owner && ctx.core.player(i as PlayerId).alive
                        })
                        .map(|i| ChoiceOption::Player(i as u8))
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::Act;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "nigong_target",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::Act => {
                    self.target = ctx.answer().iter().find_map(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    });
                    let lv = ctx.core.player(self.owner).nigong_lv;
                    ctx.core.player_mut(self.owner).nigong_lv = 0;
                    if lv % 2 == 1 {
                        // R05-C2：奇数层摸 1 走 R 帧；直伤帧在续行段发射
                        // （同步序＝摸牌交付先于伤害帧解算）。层值经 cards 槽
                        // 带入续行段（快照承载）。
                        self.cards = vec![u32::from(lv)];
                        ctx.start_draw(self.owner, 1);
                        self.stage = HostStage::DrawSettle;
                        return Flow::Yield;
                    }
                    nigong_strike(ctx, self.owner, self.target, lv);
                    Flow::Done
                }
                // R05-C2 续行位：逆攻奇数层摸 1 交付后直伤。
                HostStage::DrawSettle => {
                    let lv = self.cards.first().copied().unwrap_or(1) as u8;
                    nigong_strike(ctx, self.owner, self.target, lv);
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 女娲石：确认门后濒死者回复 1、自己失去 1（hp>1 门在 filter）
            HostBehavior::SwdNvwashiGuard => {
                let Some(t) = self.target else {
                    return Flow::Done;
                };
                if ctx.core.player(t).alive || ctx.core.player(t).hp <= 0 {
                    ctx.recover(t, 1, None);
                }
                let p = ctx.core.player(self.owner);
                if p.alive && p.hp > 1 {
                    ctx.emit(EventData::LoseHp {
                        player: self.owner,
                        amount: 1,
                    });
                }
                Flow::Done
            }
            _ => unreachable!("behavior routed to the wrong step module"),
        }
    }
}

/// 逆攻直伤（R05-C2 抽出）：层值/2 点伤害指向所选目标（无目标或目标
/// 阵亡则无事——「无目标仅摸 1 清零」与偶数层即时结算共用）。
fn nigong_strike(ctx: &mut Ctx<'_>, owner: PlayerId, target: Option<PlayerId>, lv: u8) {
    let Some(t) = target else {
        return;
    };
    let dmg = lv / 2;
    if dmg > 0 && ctx.core.player(t).alive {
        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
            Some(owner),
            t,
            i32::from(dmg),
            crate::event::DamageNature::Normal,
            None,
        )));
    }
}
