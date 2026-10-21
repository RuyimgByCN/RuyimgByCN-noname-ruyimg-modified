use super::super::*;

impl HostMachine {
    pub(super) fn step_swd_equipment(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            // 疾影：无效果步（sha_range_free 链查询）
            HostBehavior::SwdJiyingHeld => Flow::Done,
            // 沐风：确认门后摸 1（discoverCard 确定性=牌堆末摸 1）
            HostBehavior::SwdMufengHeld => {
                ctx.draw_cards(self.owner, 1);
                Flow::Done
            }
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
            HostBehavior::SwdKongdongyinGuard => {
                ctx.core.player_mut(self.owner).death_cancelled = true;
                ctx.core.player_mut(self.owner).hp = 1;
                ctx.draw_cards(self.owner, 1);
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
            // 逆攻积攒：标记 +amount（上限 4）
            HostBehavior::SwdNigongAccumulate => {
                if let Some(EventData::Damage(d)) =
                    ctx.res.frame(ctx.current_frame_id()).map(|f| &f.data)
                {
                    let lv = ctx.core.player(self.owner).nigong_lv;
                    ctx.core.player_mut(self.owner).nigong_lv = (lv + d.amount as u8).min(4);
                }
                Flow::Done
            }
            // 青龙之圭：摸 2 → 弃 1 张 he（SelectCard min1）
            HostBehavior::SwdQinglongzhiguiHeld => match self.stage {
                HostStage::Act => {
                    ctx.draw_cards(self.owner, 2);
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
                    ctx.res.frame(ctx.current_frame_id()).map(|f| &f.data)
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
                    ctx.res.frame(ctx.current_frame_id()).map(|f| &f.data)
                {
                    let p = ctx.core.player(self.owner);
                    if p.alive && p.hp < p.max_hp as i32 {
                        ctx.recover(self.owner, d.amount, None);
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
                    ctx.res.frame(ctx.current_frame_id()).map(|f| &f.data)
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
                        let frame = ctx.res.frame_mut(ctx.current_frame_id()).expect("frame");
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
            // 神秘果窗口（卡牌面窗口机批）：确认后——发射载体神秘果用牌
            //（提交段自取+Shenmiguo 臂弃置=上游 useCard 同簿记），随后
            // 原体自弃堆上桌面重发射（登记差异：上游凭空复制同花色点数
            // 新实体，原生原体复用——弃堆在再结算窗口内少一张观测差）。
            // 确认即置位每阶段一次旗（上游 shenmiguo2 临时技：再结算牌
            // 的用牌后窗口被同一旗挡住=循环守卫）。
            HostBehavior::SwdShenmiguoWindow => match self.stage {
                HostStage::Act => {
                    let (x_card, x_targets) = {
                        let u = ctx.current_card_use().expect("card use frame");
                        (u.card, u.targets.clone())
                    };
                    let Some(fruit) = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .find(|c| ctx.catalog.card_kind(*c) == CardKind::Shenmiguo)
                    else {
                        return Flow::Done;
                    };
                    self.card = Some(fruit);
                    self.cards = vec![x_card];
                    self.targets = x_targets;
                    ctx.core.turn.shenmiguo_used = true;
                    ctx.emit(EventData::CardUsed(crate::event::CardUseData {
                        card: fruit,
                        player: self.owner,
                        targets: Vec::new(),
                        nature: crate::event::DamageNature::Normal,
                        use_as: None,
                        cancelled: false,
                    }));
                    self.stage = HostStage::ShenmiguoRestage;
                    Flow::Yield
                }
                HostStage::ShenmiguoRestage => {
                    let Some(x_card) = self.cards.first().copied() else {
                        return Flow::Done;
                    };
                    // 原体自弃堆上桌面（处理区）；不在弃堆（特殊流转）则放弃
                    if ctx.core.discard.contains(&x_card) {
                        ctx.core.discard.retain(|c| *c != x_card);
                        ctx.core.table.push(x_card);
                    } else if !ctx.core.table.contains(&x_card) {
                        return Flow::Done;
                    }
                    let kind = ctx.catalog.card_kind(x_card);
                    let nature = match kind {
                        CardKind::FireSha => crate::event::DamageNature::Fire,
                        CardKind::LeiSha => crate::event::DamageNature::Thunder,
                        _ => crate::event::DamageNature::Normal,
                    };
                    ctx.emit(EventData::CardUsed(crate::event::CardUseData {
                        card: x_card,
                        player: self.owner,
                        targets: self.targets.clone(),
                        nature,
                        use_as: None,
                        cancelled: false,
                    }));
                    self.cards.clear();
                    self.targets.clear();
                    self.card = None;
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 鼠槌（equip5 窗口机批）：凤略 Yield 循环同型——选攻击范围
            // 内目标（sha_range_free 链同 engine 出杀口径）→ 至多 3 张
            // 逐张自动用出（每次重扫手中首张杀=上游 getCards("h")[0]
            // 确定性口径）；终局若链上任一杀造成伤害（commit_damage 钩
            // 置位 shuchui_hit）则摸 1。cards 复用为剩余次数计数。
            HostBehavior::SwdShuchuiUse => match self.stage {
                // 首步：目标询问（挂起前 stage 前移到消费阶段——active_ask
                // 铁律：快照 stage=挂起期间等待的下一阶段）
                HostStage::Act => {
                    let sha_range_free = ctx
                        .catalog
                        .skill_by_name("jiying")
                        .map(|d| ctx.core.has_skill(self.owner, d))
                        .unwrap_or(false);
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            *p != self.owner
                                && (sha_range_free
                                    || ctx.core.computed_distance(ctx.catalog, self.owner, *p)
                                        <= ctx.core.attack_range(ctx.catalog, self.owner))
                        })
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::ShuchuiTarget;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "shuchui_target",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::ShuchuiTarget => {
                    if let Some(ChoiceOption::Player(t)) = ctx.answer().first().cloned() {
                        if t != self.owner && ctx.core.player(t).alive {
                            self.target = Some(t);
                        }
                    }
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    // 链头武装：伤害钩置位窗开（commit_damage 消费）
                    ctx.core.turn.shuchui_armed = true;
                    ctx.core.turn.shuchui_hit = false;
                    self.cards = vec![0, 0, 0];
                    self.stage = HostStage::ShuchuiTargetLoop;
                    self.shuchui_loop(ctx, t)
                }
                HostStage::ShuchuiTargetLoop => {
                    self.shuchui_loop(ctx, self.target.expect("loop stage implies target"))
                }
                _ => Flow::Done,
            },
            // 玉蝉换形: 手牌中每张玉蝉实体轮换为八卦序下一 kind 的实体
            //（牌堆/弃堆确定性发现首张；登记差异上游随机+init 原地变形）
            HostBehavior::SwdYuchanSwap => {
                const ORDER: [CardKind; 8] = [
                    CardKind::Yuchanqian,
                    CardKind::Yuchankun,
                    CardKind::Yuchanzhen,
                    CardKind::Yuchanxun,
                    CardKind::Yuchankan,
                    CardKind::Yuchanli,
                    CardKind::Yuchangen,
                    CardKind::Yuchandui,
                ];
                // 收集「换出牌 → 换入牌」对（按牌 id；不记索引——`to_discard`
                // 现在会按牌区唯一性移动牌，索引在循环中会失效）
                let mut swaps: Vec<(CardId, CardId)> = Vec::new();
                for c in ctx.core.player(self.owner).hand.iter().rev() {
                    let Some(pos) = ctx
                        .catalog
                        .card_kind(*c)
                        .yuchan_auto_view_as()
                        .and_then(|_| ORDER.iter().position(|k| k == &ctx.catalog.card_kind(*c)))
                    else {
                        continue;
                    };
                    let want = ORDER[(pos + 1) % 8];
                    if let Some(newc) = ctx
                        .core
                        .deck
                        .iter()
                        .chain(ctx.core.discard.iter())
                        .copied()
                        .find(|x| ctx.catalog.card_kind(*x) == want)
                    {
                        swaps.push((*c, newc));
                    }
                }
                for (oldc, newc) in swaps {
                    // 旧实体从手牌移出入弃堆（守恒），新实体从来源区移除后入手
                    ctx.take_hand_card(self.owner, oldc);
                    ctx.to_discard(oldc);
                    if let Some(dpos) = ctx.core.deck.iter().position(|x| *x == newc) {
                        ctx.core.deck.remove(dpos);
                    } else if let Some(dpos) = ctx.core.discard.iter().position(|x| *x == newc) {
                        ctx.core.discard.remove(dpos);
                    }
                    let hand = &mut ctx.core.player_mut(self.owner).hand;
                    if let Some(pos) = hand.iter().position(|c| *c == oldc) {
                        hand[pos] = newc;
                    } else {
                        // 旧牌已由 to_discard 移出：新牌直接入手（换形语义不变）
                        hand.push(newc);
                    }
                    ctx.log(LogEvent::CardsGiven {
                        from: self.owner,
                        to: self.owner,
                        cards: vec![newc],
                    });
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
                        ctx.draw_cards(self.owner, 1);
                    }
                    if self.target.is_none() {
                        // lv==1 面：无目标仅摸 1 清零
                        return Flow::Done;
                    }
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    let dmg = lv / 2;
                    if dmg > 0 && ctx.core.player(t).alive {
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            Some(self.owner),
                            t,
                            dmg as u32,
                            crate::event::DamageNature::Normal,
                            None,
                        )));
                    }
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
            HostBehavior::SwdShentouEquipSkill => match self.stage {
                HostStage::Act => {
                    // 潜偷（swd 第三批，规则 v92）：出牌阶段限一次，弃一张
                    // 手牌为代价 → 判定梅花则失败 → 成功则获得手牌数比你多
                    // 的目标一张随机手牌（上游 shentou chooseCard cost +
                    // judge 梅花失败 + randomGet 同口径）
                    let nh = ctx.core.player(self.owner).hand.len();
                    let richer = ctx
                        .core
                        .alive_players()
                        .any(|p| p != self.owner && ctx.core.player(p).hand.len() > nh);
                    if nh == 0 || !richer {
                        // 无代价可付或无人手牌更多：技能空发（上游 filter）
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
                            prompt: SWD_SHENTOU_COST,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    // 消费代价牌 → 判定（梅花失败）→ 成功偷牌
                    let cost = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    });
                    if let Some(c) = cost.filter(|c| ctx.core.player(self.owner).hand.contains(c)) {
                        ctx.discard_from_hand(self.owner, c);
                        ctx.log(LogEvent::CardDiscarded {
                            player: self.owner,
                            cards: vec![c],
                        });
                    }
                    let judged = ctx.draw_judge_card();
                    let club = matches!(
                        ctx.catalog.cards[judged as usize].suit,
                        crate::catalog::Suit::Club
                    );
                    ctx.log(LogEvent::Judged {
                        player: self.owner,
                        card: judged,
                    });
                    ctx.to_discard(judged);
                    if club {
                        return Flow::Done;
                    }
                    // 成功：随机一名手牌更多的其他角色 → 随机一张手牌
                    let cands: Vec<crate::ids::PlayerId> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            *p != self.owner
                                && ctx.core.player(*p).hand.len()
                                    > ctx.core.player(self.owner).hand.len()
                        })
                        .collect();
                    if !cands.is_empty() {
                        let v = cands[ctx.core.rng.gen_range(cands.len() as u32) as usize];
                        let n = ctx.core.player(v).hand.len();
                        let i = ctx.core.rng.gen_range(n as u32) as usize;
                        let c = ctx.core.player_mut(v).hand.remove(i);
                        ctx.core.player_mut(self.owner).hand.push(c);
                        ctx.log(LogEvent::CardsGiven {
                            from: v,
                            to: self.owner,
                            cards: vec![c],
                        });
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::SwdYufuluEquipSkill => match self.stage {
                // 御夫录（swd 第十二批，规则 v96）：三段——Act 问武器
                //（手牌/装备区 equip1）→ SelectCard 消费并问目标 →
                // TargetChoice 目标受 1 点伤害并获得武器牌（上游
                // damage + gain 同序；dieThen 仍获得登记差异）。
                HostStage::Act => {
                    let is_weapon = |c: CardId| {
                        ctx.catalog.card_slot_of(ctx.core, c)
                            == Some(crate::catalog::EquipSlot::Weapon)
                    };
                    let has_weapon = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .chain(ctx.core.player(self.owner).equips.iter())
                        .copied()
                        .any(is_weapon);
                    let others = ctx.core.alive_players().any(|p| p != self.owner);
                    if !has_weapon || !others {
                        return Flow::Done;
                    }
                    let mut options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| is_weapon(*c))
                        .map(ChoiceOption::Card)
                        .collect();
                    options.extend(
                        ctx.core
                            .player(self.owner)
                            .equips
                            .iter()
                            .copied()
                            .filter(|c| is_weapon(*c))
                            .map(ChoiceOption::Card),
                    );
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: SWD_YUFULU_WEAPON,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    // 记录所选武器（保留在原区域，上游 prepare:"give" 同
                    // 口径——卡牌在 content 执行时才移动）→ 问目标
                    let w = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    });
                    let w = w.filter(|c| {
                        ctx.core.player(self.owner).hand.contains(c)
                            || ctx.core.player(self.owner).equips.contains(c)
                    });
                    let Some(weapon) = w else {
                        return Flow::Done;
                    };
                    self.card = Some(weapon);
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| *p != self.owner)
                        .map(ChoiceOption::Player)
                        .collect();
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: SWD_YUFULU_TARGET,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(weapon) = self.card.take() else {
                        return Flow::Done;
                    };
                    let target = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    });
                    let Some(t) = target.filter(|p| ctx.core.player(*p).alive) else {
                        return Flow::Done;
                    };
                    // 此刻才移动武器（手牌→直接取出；装备区→取出并撤销
                    // 装备技），保持存档核算各询问挂起态区域完整
                    if ctx.core.player(self.owner).hand.contains(&weapon) {
                        ctx.core
                            .player_mut(self.owner)
                            .hand
                            .retain(|x| *x != weapon);
                    } else if ctx.core.player(self.owner).equips.contains(&weapon) {
                        ctx.core
                            .player_mut(self.owner)
                            .equips
                            .retain(|x| *x != weapon);
                        crate::kernel::revoke_equip_skills(
                            ctx.core,
                            ctx.catalog,
                            self.owner,
                            weapon,
                        );
                    } else {
                        return Flow::Done;
                    }
                    // 上游 content：damage 后 gain
                    ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                        Some(self.owner),
                        t,
                        1,
                        crate::event::DamageNature::Normal,
                        None,
                    )));
                    ctx.core.player_mut(t).hand.push(weapon);
                    ctx.log(LogEvent::CardsGiven {
                        from: self.owner,
                        to: t,
                        cards: vec![weapon],
                    });
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::SwdLangeguaiyiEquipSkill => match self.stage {
                // 蓝格怪衣「龙帆」（swd 第十五批，规则 v99）：判定按花色
                // 分支——红桃回 1 体力（ctx.recover 满血饱和为既有路径，
                // 上游 recover 同钳制）、方片摸 1、梅花随机敌人随机弃
                // 一张、黑桃无事。上游 judge 的 suit 提示函数为 AI 排序
                // （不迁移）；判定牌入弃堆（Shentou 同款）。
                HostStage::Act => {
                    let judged = ctx.draw_judge_card();
                    let suit = ctx.catalog.cards[judged as usize].suit;
                    ctx.log(LogEvent::Judged {
                        player: self.owner,
                        card: judged,
                    });
                    ctx.to_discard(judged);
                    use crate::catalog::Suit;
                    match suit {
                        Suit::Heart => {
                            ctx.recover(self.owner, 1, None);
                        }
                        Suit::Diamond => {
                            ctx.draw_cards(self.owner, 1);
                        }
                        Suit::Club => {
                            // 候选=其他存活且（手牌∪装备区）非空（上游
                            // he filter 同口径）；随机取一。上游
                            // randomDiscard() 默认区不可从归档核验（现有
                            // 调用点均显式 "h"）——登记差异：手牌优先、
                            // 无手牌退随机装备（离装撤销装备技）。
                            let mut cands: Vec<crate::ids::PlayerId> = ctx
                                .core
                                .alive_players()
                                .filter(|p| {
                                    *p != self.owner
                                        && (!ctx.core.player(*p).hand.is_empty()
                                            || !ctx.core.player(*p).equips.is_empty())
                                })
                                .collect();
                            if !cands.is_empty() {
                                let i = ctx.core.rng.gen_range(cands.len() as u32) as usize;
                                let t = cands.swap_remove(i);
                                if !ctx.core.player(t).hand.is_empty() {
                                    let n = ctx.core.player(t).hand.len();
                                    let j = ctx.core.rng.gen_range(n as u32) as usize;
                                    let c = ctx.core.player_mut(t).hand.remove(j);
                                    ctx.to_discard(c);
                                    ctx.log(LogEvent::CardDiscarded {
                                        player: t,
                                        cards: vec![c],
                                    });
                                } else {
                                    let n = ctx.core.player(t).equips.len();
                                    let j = ctx.core.rng.gen_range(n as u32) as usize;
                                    let c = ctx.core.player_mut(t).equips.remove(j);
                                    crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, t, c);
                                    ctx.to_discard(c);
                                    ctx.log(LogEvent::CardDiscarded {
                                        player: t,
                                        cards: vec![c],
                                    });
                                }
                            }
                        }
                        Suit::Spade => {}
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            _ => unreachable!("behavior routed to the wrong step module"),
        }
    }
}
