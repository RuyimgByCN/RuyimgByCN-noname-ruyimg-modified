use super::super::*;

impl HostMachine {
    pub(super) fn step_mtg(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            HostBehavior::MtgLongshanfengLand => match self.stage {
                // 龙颅山峰（mtg 首切片，规则 v100）：Act 换地（覆盖
                // current_land；land_used 本回合门在 PlayCard 门控）→
                // 问获得杀/闪 → SelectCard 从牌堆随机取一张该类牌入手
                //（上游 createCard 凭空造牌→牌堆取，羊皮卷同款登记差异；
                // 牌堆无该类牌则放弃该臂）。
                HostStage::Act => {
                    sync_land_skill(ctx, self.owner, CardKind::Longlushanfeng);
                    ctx.core.current_land = Some(crate::catalog::CardKind::Longlushanfeng);
                    ctx.core.turn.land_used = true;
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::PickTag,
                            prompt: MTG_LONGSHANFENG_GAIN,
                            options: vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let want_sha = match ctx.answer().first() {
                        Some(ChoiceOption::Tag(0)) => true,
                        Some(ChoiceOption::Tag(1)) => false,
                        _ => return Flow::Done,
                    };
                    let want = if want_sha {
                        crate::catalog::CardKind::Sha
                    } else {
                        crate::catalog::CardKind::Shan
                    };
                    let Some(pos) = ctx
                        .core
                        .deck
                        .iter()
                        .position(|c| ctx.catalog.card_kind(*c) == want)
                    else {
                        return Flow::Done;
                    };
                    // 上游 get.cardPile 取牌堆顶侧；原生取牌堆内首张该类
                    //（确定性；登记差异与羊皮卷 RNG 口径并注）
                    let c = ctx.core.deck.remove(pos);
                    ctx.core.player_mut(self.owner).hand.push(c);
                    ctx.log(LogEvent::CardsDrawn {
                        player: self.owner,
                        cards: vec![c],
                    });
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::MtgHaidaoLand => match self.stage {
                // 海岛（mtg 第二批，规则 v101）：Act 换地 → 满血直接 +1
                // 护甲（不受伤，转换不触发）；受伤问弃一张（he，可取消）
                // → SelectCard 弃则回 1 / 取消得 1 护甲（上游 _temp 标记
                // 口径：此路线绕过转换原语，路径级抑制）。
                HostStage::Act => {
                    sync_land_skill(ctx, self.owner, CardKind::Haidao);
                    ctx.core.current_land = Some(crate::catalog::CardKind::Haidao);
                    ctx.core.turn.land_used = true;
                    let p = ctx.core.player(self.owner);
                    if p.hp >= p.max_hp as i32 {
                        ctx.change_hujia(self.owner, 1);
                        return Flow::Done;
                    }
                    let mut options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    options.extend(
                        ctx.core
                            .player(self.owner)
                            .equips
                            .iter()
                            .copied()
                            .map(ChoiceOption::Card),
                    );
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: MTG_HAIDAO_DISCARD,
                            options,
                            min: 0,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    // 取消=空选择（allow_cancel 既有口径）→ +1 护甲
                    let picked = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    });
                    match picked {
                        Some(c) => {
                            // 弃一张（手牌直弃；装备离装撤销装备技）
                            if ctx.core.player(self.owner).hand.contains(&c) {
                                ctx.discard_from_hand(self.owner, c);
                                ctx.log(LogEvent::CardDiscarded {
                                    player: self.owner,
                                    cards: vec![c],
                                });
                            } else if ctx.core.player(self.owner).equips.contains(&c) {
                                ctx.core.player_mut(self.owner).equips.retain(|x| *x != c);
                                crate::kernel::revoke_equip_skills(
                                    ctx.core,
                                    ctx.catalog,
                                    self.owner,
                                    c,
                                );
                                ctx.to_discard(c);
                                ctx.log(LogEvent::CardDiscarded {
                                    player: self.owner,
                                    cards: vec![c],
                                });
                            }
                            ctx.recover(self.owner, 1, None);
                        }
                        None => {
                            // 路径级抑制（上游 _temp_mtg_haidao 同口径）：
                            // 直写护甲字段不经转换原语
                            let cur = ctx.core.player(self.owner).hujia;
                            ctx.core.player_mut(self.owner).hujia = cur + 1;
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::MtgShuimomuxueLand => match self.stage {
                // 水没墓穴（mtg 第三批，规则 v102）：Act 换地 → he 有牌
                // 则问弃一张（可取消）→ SelectCard 弃则摸二/取消无事
                //（上游 chooseToDiscard result.bool 同口径）。
                HostStage::Act => {
                    sync_land_skill(ctx, self.owner, CardKind::Shuimomuxue);
                    ctx.core.current_land = Some(crate::catalog::CardKind::Shuimomuxue);
                    ctx.core.turn.land_used = true;
                    let mut options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    options.extend(
                        ctx.core
                            .player(self.owner)
                            .equips
                            .iter()
                            .copied()
                            .map(ChoiceOption::Card),
                    );
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: MTG_SHUIMOMUXUE_DISCARD,
                            options,
                            min: 0,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let picked = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    });
                    if let Some(c) = picked {
                        if ctx.core.player(self.owner).hand.contains(&c) {
                            ctx.discard_from_hand(self.owner, c);
                            ctx.log(LogEvent::CardDiscarded {
                                player: self.owner,
                                cards: vec![c],
                            });
                        } else if ctx.core.player(self.owner).equips.contains(&c) {
                            ctx.core.player_mut(self.owner).equips.retain(|x| *x != c);
                            crate::kernel::revoke_equip_skills(
                                ctx.core,
                                ctx.catalog,
                                self.owner,
                                c,
                            );
                            ctx.to_discard(c);
                            ctx.log(LogEvent::CardDiscarded {
                                player: self.owner,
                                cards: vec![c],
                            });
                        }
                        // R05-C2：弃后摸 2 走 R 帧（步末无后续读）。
                        ctx.start_draw(self.owner, 2);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::MtgFeixuLand => match self.stage {
                // 废墟（mtg 第四批，规则 v103）：Act 换地 → 弃牌堆随机
                // 亮 ≤3 张（留弃堆，羊皮卷同口径）→ 问三选一 → SelectCard
                // 选定从弃堆入手；弃堆空则空发。
                HostStage::Act => {
                    sync_land_skill(ctx, self.owner, CardKind::Feixu);
                    ctx.core.current_land = Some(crate::catalog::CardKind::Feixu);
                    ctx.core.turn.land_used = true;
                    let mut pool: Vec<CardId> = ctx.core.discard.iter().copied().collect();
                    if pool.is_empty() {
                        return Flow::Done;
                    }
                    let n = pool.len().min(3);
                    for i in 0..n {
                        let j = i + ctx.core.rng.gen_range((pool.len() - i) as u32) as usize;
                        pool.swap(i, j);
                    }
                    self.cards = pool[..n].to_vec();
                    let options: Vec<ChoiceOption> =
                        self.cards.iter().copied().map(ChoiceOption::Card).collect();
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: MTG_FEIXU_DISCOVER,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(picked) = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    // 择定此刻才离弃堆（亮出态仍在弃堆，区域完整）
                    let Some(pos) = ctx.core.discard.iter().position(|c| *c == picked) else {
                        return Flow::Done;
                    };
                    ctx.core.discard.remove(pos);
                    ctx.core.player_mut(self.owner).hand.push(picked);
                    ctx.log(LogEvent::CardsDrawn {
                        player: self.owner,
                        cards: vec![picked],
                    });
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::MtgYixialanLand => match self.stage {
                // 依夏兰（mtg 第五批，规则 v104）：Act 换地 + 同步地附
                // 技实例 → 问类型（0=基本 1=锦囊）→ SelectCard 牌堆随机
                // 取该类一张入手（上游 createCard(inpile.randomGet())→
                // 牌堆实体取，羊皮卷登记差异口径；无该类牌则空发）。
                HostStage::Act => {
                    sync_land_skill(ctx, self.owner, CardKind::Yixialan);
                    ctx.core.current_land = Some(CardKind::Yixialan);
                    ctx.core.turn.land_used = true;
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::PickTag,
                            prompt: MTG_YIXIALAN_GAIN,
                            options: vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let class = match ctx.answer().first() {
                        Some(ChoiceOption::Tag(t @ 0..=1)) => *t,
                        _ => return Flow::Done,
                    };
                    let pool: Vec<CardId> = ctx
                        .core
                        .deck
                        .iter()
                        .copied()
                        .filter(|c| yangpijuan_class(ctx.catalog.card_kind(*c)) == Some(class))
                        .collect();
                    if pool.is_empty() {
                        return Flow::Done;
                    }
                    let i = ctx.core.rng.gen_range(pool.len() as u32) as usize;
                    let c = pool[i];
                    let pos = ctx
                        .core
                        .deck
                        .iter()
                        .position(|x| *x == c)
                        .expect("deck contains");
                    ctx.core.deck.remove(pos);
                    ctx.core.player_mut(self.owner).hand.push(c);
                    ctx.log(LogEvent::CardsDrawn {
                        player: self.owner,
                        cards: vec![c],
                    });
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::MtgYixialanRecast => match self.stage {
                // 重铸（mtg 第五批，规则 v104）：Act 问手牌基本牌 →
                // SelectCard 放回牌堆 → 获得牌堆一张锦囊（无锦囊摸一，
                // 上游 recast 回调补偿同口径）。
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| yangpijuan_class(ctx.catalog.card_kind(*c)) == Some(0))
                        .map(ChoiceOption::Card)
                        .collect();
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: MTG_YIXIALAN_RECAST_CARD,
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
                    // 放回牌堆（顶侧惯例，上游 recast 洗回同义）
                    ctx.core.player_mut(self.owner).hand.retain(|x| *x != basic);
                    ctx.core.deck.push(basic);
                    // 牌堆找一张锦囊获得；无则摸一补偿
                    let trick = ctx
                        .core
                        .deck
                        .iter()
                        .copied()
                        .find(|c| yangpijuan_class(ctx.catalog.card_kind(*c)) == Some(1));
                    match trick {
                        Some(t) => {
                            let pos = ctx
                                .core
                                .deck
                                .iter()
                                .position(|x| *x == t)
                                .expect("deck contains");
                            ctx.core.deck.remove(pos);
                            ctx.core.player_mut(self.owner).hand.push(t);
                            ctx.log(LogEvent::CardsDrawn {
                                player: self.owner,
                                cards: vec![t],
                            });
                        }
                        None => {
                            // R05-C2：无锦囊补偿摸 1 走 R 帧（步末无后续读）。
                            ctx.start_draw(self.owner, 1);
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::MtgShamolvzhouLand => match self.stage {
                // 沙漠绿洲（mtg 第六批，规则 v105）：Act 换地 → 牌堆基本
                // 牌随机亮 ≤3 张（留牌堆）→ 问三选一 → SelectCard 选定
                // 从牌堆入手；牌堆无基本牌空发。
                HostStage::Act => {
                    sync_land_skill(ctx, self.owner, CardKind::Shamolvzhou);
                    ctx.core.current_land = Some(CardKind::Shamolvzhou);
                    ctx.core.turn.land_used = true;
                    let mut pool: Vec<CardId> = ctx
                        .core
                        .deck
                        .iter()
                        .copied()
                        .filter(|c| yangpijuan_class(ctx.catalog.card_kind(*c)) == Some(0))
                        .collect();
                    if pool.is_empty() {
                        return Flow::Done;
                    }
                    let n = pool.len().min(3);
                    for i in 0..n {
                        let j = i + ctx.core.rng.gen_range((pool.len() - i) as u32) as usize;
                        pool.swap(i, j);
                    }
                    self.cards = pool[..n].to_vec();
                    let options: Vec<ChoiceOption> =
                        self.cards.iter().copied().map(ChoiceOption::Card).collect();
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: MTG_SHAMOLVZHOU_DISCOVER,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(picked) = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    let Some(pos) = ctx.core.deck.iter().position(|c| *c == picked) else {
                        return Flow::Done;
                    };
                    ctx.core.deck.remove(pos);
                    ctx.core.player_mut(self.owner).hand.push(picked);
                    ctx.log(LogEvent::CardsDrawn {
                        player: self.owner,
                        cards: vec![picked],
                    });
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::MtgLinzhongjiantaLand => match self.stage {
                // 林中尖塔（mtg 第七批，规则 v106）：Act 换地 → 牌堆武器
                // 亮 ≤3 三选一 → SelectCard 选定 emit Equipped（走完整装
                // 配：替换/装备技授予）；无武器空发。
                HostStage::Act => {
                    sync_land_skill(ctx, self.owner, CardKind::Linzhongjianta);
                    ctx.core.current_land = Some(CardKind::Linzhongjianta);
                    ctx.core.turn.land_used = true;
                    let mut pool: Vec<CardId> = ctx
                        .core
                        .deck
                        .iter()
                        .copied()
                        .filter(|c| {
                            ctx.catalog.card_slot_of(ctx.core, *c)
                                == Some(crate::catalog::EquipSlot::Weapon)
                        })
                        .collect();
                    if pool.is_empty() {
                        return Flow::Done;
                    }
                    let n = pool.len().min(3);
                    for i in 0..n {
                        let j = i + ctx.core.rng.gen_range((pool.len() - i) as u32) as usize;
                        pool.swap(i, j);
                    }
                    self.cards = pool[..n].to_vec();
                    let options: Vec<ChoiceOption> =
                        self.cards.iter().copied().map(ChoiceOption::Card).collect();
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: MTG_LINZHONGJIANTA_DISCOVER,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(picked) = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    let Some(pos) = ctx.core.deck.iter().position(|c| *c == picked) else {
                        return Flow::Done;
                    };
                    ctx.core.deck.remove(pos);
                    // 实体牌入手后 emit Equipped（commit_equip 从手牌取
                    // 牌的适配；走完整装配：同栏替换/装备技授予）
                    ctx.core.player_mut(self.owner).hand.push(picked);
                    ctx.emit(EventData::Equipped {
                        player: self.owner,
                        card: picked,
                        replaced: None,
                    });
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::MtgLinzhongjiantaStrike => match self.stage {
                // 塔击（mtg 第七批，规则 v106）：Act 问手牌基本牌 →
                // SelectCard 问目标 → declared-Sha 进入杀结算（丈八蛇矛
                // TargetRead 同构；登记差异：上游响应/出牌双入口，原生
                // 仅出牌入口）。
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| yangpijuan_class(ctx.catalog.card_kind(*c)) == Some(0))
                        .map(ChoiceOption::Card)
                        .collect();
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: MTG_LINZHONGJIANTA_CARD,
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
                            prompt: MTG_LINZHONGJIANTA_TARGET,
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
                    // 素材牌留手牌由杀结算消费（丈八 TargetRead 同构；
                    // commit 从手牌取牌）
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
            HostBehavior::MtgBingheyaosaiLand => match self.stage {
                // 冰河要塞（mtg 第八批，规则 v107）：Act 换地 + 摸 2 →
                // 问弃两张（he=手牌∪装备，强制 min2 max2；摸 2 保证手牌
                // 恒 ≥2）→ SelectCard 弃所选项。
                HostStage::Act => {
                    sync_land_skill(ctx, self.owner, CardKind::Bingheyaosai);
                    ctx.core.current_land = Some(CardKind::Bingheyaosai);
                    ctx.core.turn.land_used = true;
                    // R05-C2：摸 2 走 R 帧；弃两选（he min2max2）须含刚摸
                    // 的牌（「摸 2 保证手牌恒 ≥2」）→ 续行位发询问。
                    ctx.start_draw(self.owner, 2);
                    self.stage = HostStage::DrawSettle;
                    Flow::Yield
                }
                // R05-C2 续行位：冰河要塞摸 2 交付后建弃两选。
                HostStage::DrawSettle => {
                    let mut options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    options.extend(
                        ctx.core
                            .player(self.owner)
                            .equips
                            .iter()
                            .copied()
                            .map(ChoiceOption::Card),
                    );
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: MTG_BINGHEYAOSAI_DISCARD,
                            options,
                            min: 2,
                            max: 2,
                            allow_cancel: false,
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
                    for c in picked {
                        if ctx.core.player(self.owner).hand.contains(&c) {
                            ctx.discard_from_hand(self.owner, c);
                            ctx.log(LogEvent::CardDiscarded {
                                player: self.owner,
                                cards: vec![c],
                            });
                        } else if ctx.core.player(self.owner).equips.contains(&c) {
                            ctx.core.player_mut(self.owner).equips.retain(|x| *x != c);
                            crate::kernel::revoke_equip_skills(
                                ctx.core,
                                ctx.catalog,
                                self.owner,
                                c,
                            );
                            ctx.to_discard(c);
                            ctx.log(LogEvent::CardDiscarded {
                                player: self.owner,
                                cards: vec![c],
                            });
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::MtgYoulinLand => match self.stage {
                // 幽林（mtg 第九批，规则 v108）：Act 换地 → 池=牌堆中的
                // 决斗/无中生有（登记差异：上游全普通锦囊池+正收益 AI
                // 判定；原生收窄为内核结算已确证者，无懈窗口照常）→ 随
                // 机一张随机其他存活目标（无中无目标）→ emit 结算；池
                // 空空发。
                HostStage::Act => {
                    sync_land_skill(ctx, self.owner, CardKind::Youlin);
                    ctx.core.current_land = Some(CardKind::Youlin);
                    ctx.core.turn.land_used = true;
                    let pool: Vec<(CardId, bool)> = ctx
                        .core
                        .deck
                        .iter()
                        .copied()
                        .filter_map(|c| match ctx.catalog.card_kind(c) {
                            CardKind::Duel => Some((c, true)),
                            CardKind::Wuzhong => Some((c, false)),
                            _ => None,
                        })
                        .collect();
                    if pool.is_empty() {
                        return Flow::Done;
                    }
                    let i = ctx.core.rng.gen_range(pool.len() as u32) as usize;
                    let (card, has_target) = pool[i];
                    let pos = ctx
                        .core
                        .deck
                        .iter()
                        .position(|x| *x == card)
                        .expect("deck contains");
                    ctx.core.deck.remove(pos);
                    // 实体牌入手后 emit CardUsed（commit_card_used 从手
                    // 牌取牌的适配；内核各锦囊结算/无懈窗口自动复用）
                    ctx.core.player_mut(self.owner).hand.push(card);
                    let targets: Vec<PlayerId> = if has_target {
                        let cands: Vec<PlayerId> = ctx
                            .core
                            .alive_players()
                            .filter(|p| *p != self.owner)
                            .collect();
                        if cands.is_empty() {
                            ctx.core.player_mut(self.owner).hand.retain(|x| *x != card);
                            ctx.to_discard(card);
                            return Flow::Done;
                        }
                        let j = ctx.core.rng.gen_range(cands.len() as u32) as usize;
                        vec![cands[j]]
                    } else {
                        Vec::new()
                    };
                    ctx.emit(EventData::CardUsed(crate::machine::card_use_as(
                        card, self.owner, targets, None,
                    )));
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::MtgYoulinDiscard => match self.stage {
                // 幽林地附技（mtg 第九批，规则 v108）：Act 问手牌锦囊 →
                // SelectCard 弃 → 牌堆随机亮 ≤3（全类别，留牌堆）→ 问三
                // 选一 → TargetChoice 选定入手。
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| yangpijuan_class(ctx.catalog.card_kind(*c)) == Some(1))
                        .map(ChoiceOption::Card)
                        .collect();
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: MTG_YOULIN_DISCARD,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(trick) = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(self.owner).hand.contains(&trick) {
                        return Flow::Done;
                    }
                    ctx.discard_from_hand(self.owner, trick);
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![trick],
                    });
                    let mut pool: Vec<CardId> = ctx.core.deck.iter().copied().collect();
                    if pool.is_empty() {
                        return Flow::Done;
                    }
                    let n = pool.len().min(3);
                    for i in 0..n {
                        let j = i + ctx.core.rng.gen_range((pool.len() - i) as u32) as usize;
                        pool.swap(i, j);
                    }
                    self.cards = pool[..n].to_vec();
                    let options: Vec<ChoiceOption> =
                        self.cards.iter().copied().map(ChoiceOption::Card).collect();
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: MTG_YOULIN_PICK,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(picked) = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    let Some(pos) = ctx.core.deck.iter().position(|c| *c == picked) else {
                        return Flow::Done;
                    };
                    ctx.core.deck.remove(pos);
                    ctx.core.player_mut(self.owner).hand.push(picked);
                    ctx.log(LogEvent::CardsDrawn {
                        player: self.owner,
                        cards: vec![picked],
                    });
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::MtgLindixiliuLand => match self.stage {
                // 邻地溪流（mtg 第十批，规则 v109）：Act 换地 → 牌堆♥
                // 首张入手（get.cardPile2 顺序同义）→ 手牌∪装备♥非空则
                // 问弃任意张（可取消）→ SelectCard 逐张弃置，每张随机
                // 敌方判定区置入其未持有的延时锦囊（牌堆 Le→Bing→
                // ShanDian 顺序，上游 inpile("delay") 同序；判定区直接
                // push，JudgePhase 阶段取队；专用日志暂缓登记）。
                HostStage::Act => {
                    sync_land_skill(ctx, self.owner, CardKind::Lindixiliu);
                    ctx.core.current_land = Some(CardKind::Lindixiliu);
                    ctx.core.turn.land_used = true;
                    // 牌堆♥首张入手
                    if let Some(pos) = ctx.core.deck.iter().position(|c| {
                        ctx.catalog.cards[*c as usize].suit == crate::catalog::Suit::Heart
                    }) {
                        let c = ctx.core.deck.remove(pos);
                        ctx.core.player_mut(self.owner).hand.push(c);
                        ctx.log(LogEvent::CardsDrawn {
                            player: self.owner,
                            cards: vec![c],
                        });
                    }
                    let mut options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| {
                            ctx.catalog.cards[*c as usize].suit == crate::catalog::Suit::Heart
                        })
                        .map(ChoiceOption::Card)
                        .collect();
                    // 装备区♥并入（chooseToDiscard "he"）
                    let equip_hearts: Vec<CardId> = ctx
                        .core
                        .player(self.owner)
                        .equips
                        .iter()
                        .copied()
                        .filter(|c| {
                            ctx.catalog.cards[*c as usize].suit == crate::catalog::Suit::Heart
                        })
                        .collect();
                    options.extend(equip_hearts.iter().copied().map(ChoiceOption::Card));
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.cards = equip_hearts;
                    self.stage = HostStage::SelectCard;
                    let max_n = options.len();
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: MTG_LINDIXILIU_DISCARD,
                            options,
                            min: 0,
                            max: max_n,
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
                    for c in picked {
                        // 弃置（手牌直弃/装备离装撤销技）
                        if ctx.core.player(self.owner).hand.contains(&c) {
                            ctx.discard_from_hand(self.owner, c);
                            ctx.log(LogEvent::CardDiscarded {
                                player: self.owner,
                                cards: vec![c],
                            });
                        } else if ctx.core.player(self.owner).equips.contains(&c) {
                            ctx.core.player_mut(self.owner).equips.retain(|x| *x != c);
                            crate::kernel::revoke_equip_skills(
                                ctx.core,
                                ctx.catalog,
                                self.owner,
                                c,
                            );
                            ctx.to_discard(c);
                            ctx.log(LogEvent::CardDiscarded {
                                player: self.owner,
                                cards: vec![c],
                            });
                        }
                        // 随机敌方 → 置入其未持有的延时锦囊（Le→Bing→
                        // ShanDian 顺序，牌堆实体）
                        let cands: Vec<crate::ids::PlayerId> = ctx
                            .core
                            .alive_players()
                            .filter(|p| *p != self.owner)
                            .collect();
                        if cands.is_empty() {
                            continue;
                        }
                        let j = ctx.core.rng.gen_range(cands.len() as u32) as usize;
                        let t = cands[j];
                        let want = [CardKind::Le, CardKind::Bing, CardKind::ShanDian]
                            .into_iter()
                            .find(|k| {
                                !ctx.core
                                    .player(t)
                                    .judge
                                    .iter()
                                    .any(|jc| ctx.catalog.card_kind(*jc) == *k)
                            });
                        let Some(kind) = want else { continue };
                        let Some(pos) = ctx
                            .core
                            .deck
                            .iter()
                            .position(|dc| ctx.catalog.card_kind(*dc) == kind)
                        else {
                            continue;
                        };
                        let dc = ctx.core.deck.remove(pos);
                        if ctx.core.player(t).alive {
                            ctx.core.player_mut(t).judge.push(dc);
                        } else {
                            ctx.to_discard(dc);
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            _ => unreachable!("behavior routed to the wrong step module"),
        }
    }
}
