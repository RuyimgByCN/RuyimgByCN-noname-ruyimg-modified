use super::super::*;

impl HostMachine {
    pub(super) fn step_swd_card_effects(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            // 神农鼎：SelectCard 弃 2 手牌 → Act 回复 1（受伤时；门控在
            // activate 臂 hp<maxHp+手牌≥2）
            HostBehavior::SwdShennongdingCard => match self.stage {
                HostStage::SelectCard => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
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
                            prompt: "shennongding_cost",
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
                    for c in picked {
                        if ctx.core.player(self.owner).hand.contains(&c) {
                            ctx.discard_from_hand(self.owner, c);
                            ctx.log(LogEvent::CardDiscarded {
                                player: self.owner,
                                cards: vec![c],
                            });
                        }
                    }
                    let p = ctx.core.player(self.owner);
                    if p.alive && p.hp < p.max_hp as i32 {
                        ctx.recover(self.owner, 1, None);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 四非珍面：TargetChoice 选其他有手牌角色 → Act 判定（非红桃
            // 则目标用首张手牌指向座位序首个其他存活者——登记差异上游
            // 随机牌/随机目标）
            HostBehavior::SwdSifeizhenmianCard => match self.stage {
                HostStage::TargetChoice => {
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
                            prompt: "yiluan_target",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::Act => {
                    // 已执行守卫：子帧 CardUsed 结算期间机器可能被再次
                    // 步进（阶段未推进），cards 非空即完成
                    if !self.cards.is_empty() {
                        return Flow::Done;
                    }
                    self.target = ctx.answer().iter().find_map(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    });
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    let judged = ctx.draw_judge_card();
                    if matches!(
                        ctx.catalog.cards[judged as usize].suit,
                        crate::catalog::Suit::Heart
                    ) {
                        return Flow::Done;
                    }
                    let Some(c) = ctx.core.player(t).hand.first().copied() else {
                        return Flow::Done;
                    };
                    self.cards = vec![c];
                    let kind = ctx.catalog.cards[c as usize].kind;
                    let targets: Vec<PlayerId> = (0..ctx.core.players.len())
                        .filter(|&i| {
                            let p = i as PlayerId;
                            p != t && ctx.core.player(p).alive
                        })
                        .map(|i| i as PlayerId)
                        .take(1)
                        .collect();
                    if targets.is_empty() {
                        return Flow::Done;
                    }
                    // 真实牌自动使用（v177 同口径）：机器侧不预取，卡留在
                    // 目标手牌由 CardUsed 提交段自取
                    ctx.emit(EventData::CardUsed(crate::machine::card_use_as(
                        c,
                        t,
                        targets,
                        Some(kind),
                    )));
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 昆仑镜：Act 亮牌堆顶 3 选 1 → SelectCard 手牌替换（弃 1 手
            // 牌，登记差异上游替换牌入特殊区）→ TargetChoice 获得选中牌
            HostBehavior::SwdKunlunjingcCard => match self.stage {
                HostStage::Act => {
                    if ctx.core.deck.len() < 3 {
                        return Flow::Done;
                    }
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .deck
                        .iter()
                        .take(3)
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "kunlunjingc_pick",
                            options,
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
                    self.cards = vec![picked];
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
                            prompt: "kunlunjingc_replace",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(replace) = ctx.answer().iter().find_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    let Some(picked) = self.cards.first().copied() else {
                        return Flow::Done;
                    };
                    if ctx.core.player(self.owner).hand.contains(&replace) {
                        ctx.discard_from_hand(self.owner, replace);
                        ctx.log(LogEvent::CardDiscarded {
                            player: self.owner,
                            cards: vec![replace],
                        });
                    }
                    if let Some(pos) = ctx.core.deck.iter().position(|x| *x == picked) {
                        ctx.core.deck.remove(pos);
                        ctx.core.player_mut(self.owner).hand.push(picked);
                        ctx.log(LogEvent::CardsDrawn {
                            player: self.owner,
                            cards: vec![picked],
                        });
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 东皇钟：SelectCard 弃 1 红牌 → TargetChoice 选目标 → Act 从
            // 牌堆发现延时锦囊实体（Le/Bing/ShanDian 且目标判定区未有）置
            // 入判定区（实体发现替代 createCard 凭空造；登记差异随机→首张）
            HostBehavior::SwdDonghuangzhongCard => match self.stage {
                HostStage::SelectCard => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| {
                            matches!(
                                ctx.catalog.cards[*c as usize].suit,
                                crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                            )
                        })
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
                            prompt: "donghuangzhong_cost",
                            options,
                            min: 1,
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
                        .filter(|&i| i as PlayerId != self.owner)
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
                            prompt: "donghuangzhong_target",
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
                    let delay_kinds = [CardKind::Le, CardKind::Bing, CardKind::ShanDian];
                    let mut placed = None;
                    for i in 0..ctx.core.deck.len() {
                        let c = ctx.core.deck[i];
                        let k = ctx.catalog.cards[c as usize].kind;
                        if !delay_kinds.contains(&k) {
                            continue;
                        }
                        if ctx
                            .core
                            .player(t)
                            .judge
                            .iter()
                            .any(|jc| ctx.catalog.cards[*jc as usize].kind == k)
                        {
                            continue;
                        }
                        placed = Some(c);
                        break;
                    }
                    if let Some(dc) = placed {
                        ctx.core.deck.retain(|x| *x != dc);
                        ctx.core.player_mut(t).judge.push(dc);
                        ctx.log(LogEvent::CardsGiven {
                            from: self.owner,
                            to: t,
                            cards: vec![dc],
                        });
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::SwdLiuxinghuoyuCard => match self.stage {
                HostStage::Act => {
                    // 流星火雨（swd 首批，规则 v86）：目标自选弃两张 he 牌
                    //（可取消）——取消/不足两张则受 1 点火伤（上游 step1
                    // directfalse || !result.bool → damage("fire")）。
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(t).alive {
                        if let Some(c) = self.card {
                            ctx.to_discard(c);
                            ctx.core.table.retain(|x| *x != c);
                        }
                        return Flow::Done;
                    }
                    let mut options: Vec<ChoiceOption> = ctx
                        .core
                        .player(t)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    options.extend(
                        ctx.core
                            .player(t)
                            .equips
                            .iter()
                            .copied()
                            .map(ChoiceOption::Card),
                    );
                    if options.len() < 2 {
                        // he 不足两张：directfalse → 直接火伤
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            Some(self.owner),
                            t,
                            1,
                            crate::event::DamageNature::Fire,
                            self.card,
                        )));
                        if let Some(c) = self.card {
                            ctx.to_discard(c);
                            ctx.core.table.retain(|x| *x != c);
                        }
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: t,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: SWD_LIUXING_DISCARD,
                            options,
                            min: 2,
                            max: 2,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    let picks: Vec<CardId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        })
                        .collect();
                    let valid: Vec<CardId> = picks
                        .into_iter()
                        .filter(|c| {
                            ctx.core.player(t).hand.contains(c)
                                || ctx.core.player(t).equips.contains(c)
                        })
                        .collect();
                    if valid.len() == 2 {
                        let mut discarded = Vec::new();
                        for c in &valid {
                            if ctx.core.player(t).hand.contains(c) {
                                ctx.core.player_mut(t).hand.retain(|x| x != c);
                            } else {
                                ctx.core.player_mut(t).equips.retain(|x| x != c);
                                crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, t, *c);
                            }
                            ctx.to_discard(*c);
                            discarded.push(*c);
                        }
                        ctx.log(LogEvent::CardDiscarded {
                            player: t,
                            cards: discarded,
                        });
                    } else {
                        // 取消/无效 → 1 点火伤
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            Some(self.owner),
                            t,
                            1,
                            crate::event::DamageNature::Fire,
                            self.card,
                        )));
                    }
                    if let Some(c) = self.card {
                        ctx.to_discard(c);
                        ctx.core.table.retain(|x| *x != c);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::SwdYangpijuanCard => match self.stage {
                // 羊皮卷（swd 第十三批，规则 v97）：选类别（0=基本
                // 1=锦囊 2=装备）→ 牌堆随机亮该类别 ≤3 张 → 选一张入手。
                // 上游 createCard 凭空造牌，原生从牌堆发现（登记差异）；
                // 亮出的牌择定前留在牌堆（挂起态区域完整，御夫录同口径）。
                HostStage::Act => {
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::PickTag,
                            prompt: SWD_YANGPIJUAN_CATEGORY,
                            options: (0..3).map(ChoiceOption::Tag).collect(),
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let class = match ctx.answer().first() {
                        Some(ChoiceOption::Tag(t @ 0..=2)) => *t,
                        _ => return Flow::Done,
                    };
                    // 牌堆部分 Fisher-Yates 取前 ≤3 张（RNG 真消除口径）
                    let mut pool: Vec<CardId> = ctx
                        .core
                        .deck
                        .iter()
                        .copied()
                        .filter(|c| yangpijuan_class(ctx.catalog.card_kind(*c)) == Some(class))
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
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: SWD_YANGPIJUAN_PICK,
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
                    // 择定此刻才离牌堆入手（五谷丰登同款日志口径）
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
            HostBehavior::SwdZhiluxiaohuCard => {
                // 指路小虎（U05 swd 机关牌首切片，规则 v189）：随机对一
                // 名其他存活角色使用虚拟杀（无懈窗已在外层包装；虚拟杀
                // 完整管线走闪询问/伤害）。
                let candidates: Vec<PlayerId> = ctx
                    .core
                    .seat_order_from(self.owner)
                    .into_iter()
                    .filter(|p| *p != self.owner && ctx.core.player(*p).alive)
                    .collect();
                if let Some(&pick) =
                    candidates.get(ctx.core.rng.gen_range(candidates.len() as u32) as usize)
                {
                    ctx.use_virtual_card(self.owner, crate::catalog::CardKind::Sha, vec![pick]);
                }
                // 本体弃置（卡牌机器惯例=流星火雨同款）
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            }
            HostBehavior::SwdMujiarenCard => {
                // 木甲人（U05 swd 机关牌第二切片，规则 v190）：弃全部
                // 非基本手牌（静默=上游 lose _triggered=null 同口径）→
                // 随机机关牌 1+N 张入手（实体发现，排除本体）。
                ctx.core.turn.mujiaren_used = true;
                let dropped: Vec<CardId> = ctx
                    .core
                    .player(self.owner)
                    .hand
                    .iter()
                    .copied()
                    .filter(|c| {
                        crate::content::host::yangpijuan_class(ctx.catalog.card_kind(*c)) != Some(0)
                    })
                    .collect();
                for c in &dropped {
                    ctx.take_hand_card(self.owner, *c);
                    ctx.to_discard(*c);
                }
                if !dropped.is_empty() {
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: dropped.clone(),
                    });
                }
                let num = 1 + dropped.len();
                // 实体发现：牌堆+弃堆 Jiguan 池随机（排除本体；不足给余量）
                let mut pool: Vec<CardId> = ctx
                    .core
                    .deck
                    .iter()
                    .chain(ctx.core.discard.iter())
                    .copied()
                    .filter(|x| {
                        Some(*x) != self.card
                            && ctx.catalog.card_kind(*x) == crate::catalog::CardKind::Jiguan
                    })
                    .collect();
                let n = pool.len();
                for i in (1..n).rev() {
                    let j = ctx.core.rng.gen_range(i as u32 + 1) as usize;
                    pool.swap(i, j);
                }
                for x in pool.into_iter().take(num) {
                    if let Some(pos) = ctx.core.deck.iter().position(|y| *y == x) {
                        ctx.core.deck.remove(pos);
                    } else if let Some(pos) = ctx.core.discard.iter().position(|y| *y == x) {
                        ctx.core.discard.remove(pos);
                    }
                    ctx.core.player_mut(self.owner).hand.push(x);
                    ctx.log(LogEvent::CardsDrawn {
                        player: self.owner,
                        cards: vec![x],
                    });
                }
                // 本体弃置（卡牌机器惯例）
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            }
            HostBehavior::SwdGouhunluoCard => {
                // 勾魂锣（U05 swd 机关牌第三切片，规则 v191）：目标挂
                // 3 轮倒计时（storage=3+来源）+授倒计时实例（kind 级
                // def=card_kind_host_skills）。
                let Some(t) = self.target else {
                    return Flow::Done;
                };
                if !ctx.core.player(t).alive {
                    return Flow::Done;
                }
                let pl = ctx.core.player_mut(t);
                pl.gouhunluo_rounds = 3;
                pl.gouhunluo_source = Some(self.owner);
                if let Some(def) = ctx.catalog.skill_by_name(SWD_GOUHUNLUO_SUB_KEY) {
                    ctx.core.skills.insert(
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
                // 本体弃置（卡牌机器惯例）
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            }
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
                            ctx.core.skills.insert(
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
            // 旋风之刃（灵剑，规则 v198）：目标随机弃 1 张 he；本体入弃堆。
            HostBehavior::SwdXuanfengzhirenCard => {
                if let Some(t) = self.target {
                    if ctx.core.player(t).alive {
                        let mut pool: Vec<CardId> = ctx
                            .core
                            .player(t)
                            .hand
                            .iter()
                            .copied()
                            .chain(ctx.core.player(t).equips.iter().copied())
                            .collect();
                        if !pool.is_empty() {
                            let k = ctx.core.rng.gen_range(pool.len() as u32) as usize;
                            let c = pool.swap_remove(k);
                            if ctx.core.player(t).hand.contains(&c) {
                                ctx.discard_from_hand(t, c);
                            } else if let Some(pos) =
                                ctx.core.player(t).equips.iter().position(|x| *x == c)
                            {
                                ctx.core.player_mut(t).equips.remove(pos);
                                crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, t, c);
                                ctx.to_discard(c);
                                // 装备弃置也发弃置事件（上游 loseEnd 含
                                // 装备区牌；机关妖术补偿依赖此事件）
                                ctx.emit(EventData::CardsDiscarded {
                                    player: t,
                                    cards: vec![c],
                                });
                            }
                            ctx.log(LogEvent::CardDiscarded {
                                player: t,
                                cards: vec![c],
                            });
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
            // 中行虎甲（灵剑，规则 v198）：目标非 isMin——从牌堆+弃堆发现防
            // 具 Kind 实体装配（登记差异：上游装备定义池凭空造），再随机弃
            // 1 张手牌；本体入弃堆。
            HostBehavior::SwdZhongxinghujiaCard => {
                if let Some(t) = self.target {
                    if ctx.core.player(t).alive {
                        let mut pool: Vec<CardId> = ctx
                            .core
                            .deck
                            .iter()
                            .chain(ctx.core.discard.iter())
                            .copied()
                            .filter(|x| ctx.catalog.card_kind(*x) == CardKind::Armor)
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
                        let hand = ctx.core.player(t).hand.clone();
                        if !hand.is_empty() {
                            let k = ctx.core.rng.gen_range(hand.len() as u32) as usize;
                            let c = hand[k];
                            ctx.discard_from_hand(t, c);
                            ctx.log(LogEvent::CardDiscarded {
                                player: t,
                                cards: vec![c],
                            });
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
            // 星体改造（灵剑，规则 v198）：使用者（自目标）摸 1 + 手牌上限
            // -1 累加（其下一结束阶段重置，kernel 相位钩）；本体入弃堆。
            HostBehavior::SwdXingtigaizaoCard => {
                let p = ctx.core.player_mut(self.owner);
                p.xingtigaizao_count = p.xingtigaizao_count.saturating_add(1);
                ctx.draw_cards(self.owner, 1);
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                    self.card = None;
                }
                Flow::Done
            }
            // 时间回溯（灵剑，规则 v198）：目标获得其装备区全部牌（装备技
            // 随迁撤销）；本体入弃堆。
            HostBehavior::SwdShijianhuisuCard => {
                if let Some(t) = self.target {
                    if ctx.core.player(t).alive {
                        let es: Vec<CardId> = ctx.core.player(t).equips.iter().copied().collect();
                        for c in es {
                            if let Some(pos) =
                                ctx.core.player(t).equips.iter().position(|x| *x == c)
                            {
                                ctx.core.player_mut(t).equips.remove(pos);
                                crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, t, c);
                                ctx.core.player_mut(t).hand.push(c);
                            }
                        }
                        let hand: Vec<CardId> = ctx.core.player(t).hand.iter().copied().collect();
                        ctx.log(LogEvent::CardsDrawn {
                            player: t,
                            cards: hand,
                        });
                    }
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                    self.card = None;
                }
                Flow::Done
            }
            // 紧急冷冻（灵剑，规则 v198）：目标≠使用者且未翻面——+2 护甲、
            // 翻面；仍翻面则冻结（不能使用牌也不能成为目标，翻回解除，
            // kernel TurnOvered 钩）；本体入弃堆。
            HostBehavior::SwdJinjilengdongCard => {
                if let Some(t) = self.target {
                    if ctx.core.player(t).alive && !ctx.core.player(t).face_down {
                        // 护甲直写字段（灵雪同口径，无日志原语）
                        ctx.core.player_mut(t).hujia += 2;
                        ctx.core.player_mut(t).face_down = true;
                        ctx.emit(EventData::TurnOvered {
                            player: t,
                            face_down: true,
                        });
                        if ctx.core.player(t).face_down {
                            ctx.core.player_mut(t).jileng_active = true;
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
            // 圣休号角（灵剑，规则 v198）：目标获嘲讽（相邻角色只能以其为
            // 杀目标；其下一阶段开始解除，kernel 相位钩）；本体入弃堆。
            HostBehavior::SwdShengxiuhaojiaoCard => {
                if let Some(t) = self.target {
                    if ctx.core.player(t).alive {
                        ctx.core.player_mut(t).chaofeng_active = true;
                    }
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                    self.card = None;
                }
                Flow::Done
            }
            // 隐秘离场（灵剑，规则 v198）：目标≠使用者——潜行（不能成为牌
            // 的目标；当前回合结束解除，kernel 回合收束钩）；本体入弃堆。
            HostBehavior::SwdYinmilichangCard => {
                if let Some(t) = self.target {
                    if ctx.core.player(t).alive {
                        ctx.core.player_mut(t).yinmi_active = true;
                    }
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                    self.card = None;
                }
                Flow::Done
            }
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
            // 天覆阵（zhenfa，规则 v200/T19a）：AOE 逐大国目标强制弃 1 张
            // he（targets 队列；目标自选牌 min 1 不可取消；self.target=当
            // 前询问目标）。全部结算完后本体入弃堆。
            HostBehavior::SwdTianfuzhenCard => match self.stage {
                HostStage::Act => {
                    if self.targets.is_empty() {
                        if let Some(c) = self.card {
                            ctx.to_discard(c);
                            ctx.core.table.retain(|x| *x != c);
                            self.card = None;
                        }
                        return Flow::Done;
                    }
                    let t = self.targets.remove(0);
                    self.target = Some(t);
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(t)
                        .hand
                        .iter()
                        .copied()
                        .chain(ctx.core.player(t).equips.iter().copied())
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        return Flow::Yield;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: t,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "swd_tianfuzhen_discard",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                        let t = self.target.unwrap_or(self.owner);
                        if ctx.core.player(t).hand.contains(&c) {
                            ctx.discard_from_hand(t, c);
                        } else if let Some(pos) =
                            ctx.core.player(t).equips.iter().position(|x| *x == c)
                        {
                            ctx.core.player_mut(t).equips.remove(pos);
                            crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, t, c);
                            ctx.to_discard(c);
                        }
                        ctx.log(LogEvent::CardDiscarded {
                            player: t,
                            cards: vec![c],
                        });
                    }
                    self.stage = HostStage::Act;
                    Flow::Yield
                }
                _ => Flow::Done,
            },
            // 七星阵（zhenfa，规则 v200/T19a）：阶段一 Act——逐围攻者弃 1
            //（SelectCard 询问，self.target=围攻者）；队列空后阶段二——
            // 视为对围攻者用杀（响应闪或 1 伤；登记差异：虚拟杀简化，无
            // 加成/不可无懈额外面），阶段二队列存 self.cards。本体入弃堆。
            HostBehavior::SwdQixingzhenCard => match self.stage {
                HostStage::Act => {
                    if !self.targets.is_empty() {
                        let t = self.targets.remove(0);
                        self.target = Some(t);
                        let options: Vec<ChoiceOption> = ctx
                            .core
                            .player(t)
                            .hand
                            .iter()
                            .copied()
                            .chain(ctx.core.player(t).equips.iter().copied())
                            .map(ChoiceOption::Card)
                            .collect();
                        if options.is_empty() {
                            return Flow::Yield;
                        }
                        self.stage = HostStage::SelectCard;
                        return Flow::WaitChoice {
                            observer: t,
                            spec: ChoiceSpec {
                                kind: ChoiceKind::SelectCards,
                                prompt: "swd_qixingzhen_discard",
                                options,
                                min: 1,
                                max: 1,
                                allow_cancel: false,
                            },
                        };
                    }
                    if self.cards.is_empty() {
                        if let Some(c) = self.card {
                            ctx.to_discard(c);
                            ctx.core.table.retain(|x| *x != c);
                            self.card = None;
                        }
                        return Flow::Done;
                    }
                    let t = self.cards.remove(0) as u8;
                    self.target = Some(t);
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(t)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| ctx.catalog.card_kind(*c) == CardKind::Shan)
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        // 无闪：直接结算 1 点伤害（机关凤同款收口）
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            Some(self.owner),
                            t,
                            1,
                            crate::event::DamageNature::Normal,
                            None,
                        )));
                        self.stage = HostStage::Act;
                        return Flow::Yield;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: t,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "swd_qixingzhen_shan",
                            options,
                            min: 0,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                        let t = self.target.unwrap_or(self.owner);
                        if ctx.core.player(t).hand.contains(&c) {
                            ctx.discard_from_hand(t, c);
                        } else if let Some(pos) =
                            ctx.core.player(t).equips.iter().position(|x| *x == c)
                        {
                            ctx.core.player_mut(t).equips.remove(pos);
                            crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, t, c);
                            ctx.to_discard(c);
                        }
                        ctx.log(LogEvent::CardDiscarded {
                            player: t,
                            cards: vec![c],
                        });
                    }
                    self.stage = HostStage::Act;
                    Flow::Yield
                }
                HostStage::TargetChoice => {
                    let t = self.target.unwrap_or(self.owner);
                    if matches!(ctx.answer().first(), Some(ChoiceOption::Card(_))) {
                        if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                            ctx.discard_from_hand(t, c);
                            ctx.log(LogEvent::CardDiscarded {
                                player: t,
                                cards: vec![c],
                            });
                        }
                    } else {
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            Some(self.owner),
                            t,
                            1,
                            crate::event::DamageNature::Normal,
                            None,
                        )));
                    }
                    self.stage = HostStage::Act;
                    Flow::Yield
                }
                _ => Flow::Done,
            },
            // 鸟翔阵（zhenfa，规则 v200/T19a）：逐合法目标响应闪或受 1 伤
            //（targets 队列；TargetChoice 询问，取消=伤害）。本体入弃堆。
            HostBehavior::SwdNiaoxiangzhenCard => match self.stage {
                HostStage::Act => {
                    if self.targets.is_empty() {
                        if let Some(c) = self.card {
                            ctx.to_discard(c);
                            ctx.core.table.retain(|x| *x != c);
                            self.card = None;
                        }
                        return Flow::Done;
                    }
                    let t = self.targets.remove(0);
                    self.target = Some(t);
                    self.stage = HostStage::TargetChoice;
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(t)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| ctx.catalog.card_kind(*c) == CardKind::Shan)
                        .map(ChoiceOption::Card)
                        .collect();
                    Flow::WaitChoice {
                        observer: t,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "swd_niaoxiangzhen_shan",
                            options,
                            min: 0,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let t = self.target.unwrap_or(self.owner);
                    if matches!(ctx.answer().first(), Some(ChoiceOption::Card(_))) {
                        if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                            ctx.discard_from_hand(t, c);
                            ctx.log(LogEvent::CardDiscarded {
                                player: t,
                                cards: vec![c],
                            });
                        }
                    } else {
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            Some(self.owner),
                            t,
                            1,
                            crate::event::DamageNature::Normal,
                            None,
                        )));
                    }
                    self.stage = HostStage::Act;
                    Flow::Yield
                }
                _ => Flow::Done,
            },
            // 长蛇阵（zhenfa，规则 v201/T19b）：队列中→全队列（含自己）各
            // 摸 1；否则与距离最近的友方换座成队列（swapSeat）；本体入弃堆。
            HostBehavior::SwdChangshezhenCard => {
                let inline_now = ctx.core.inline(ctx.catalog, self.owner);
                let prev_of_owner = ctx.core.alive_neighbors(self.owner).1;
                if inline_now {
                    for q in 0..ctx.core.players.len() as u8 {
                        if ctx.core.inline_with(ctx.catalog, self.owner, q) {
                            ctx.draw_cards(q, 1);
                        }
                    }
                } else if prev_of_owner.is_some() {
                    // 友方按**顺座次序**排序（NC01：顺序语义用 seat_order_from；
                    // 上游按座次轮转取最近）
                    let mut friends: Vec<(u8, u8)> = (0..ctx.core.players.len() as u8)
                        .filter(|q| {
                            *q != self.owner
                                && ctx.core.player(*q).alive
                                && ctx.core.is_friend_of(ctx.catalog, self.owner, *q)
                        })
                        .map(|q| (q, ctx.core.seat_steps_from(self.owner, q)))
                        .collect();
                    friends.sort_by_key(|(_, d)| *d);
                    if let Some((nearest, _)) = friends.first().copied() {
                        let next_of_owner = ctx.core.next_alive(self.owner);
                        ctx.core.swap_seat(nearest, next_of_owner);
                    }
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                    self.card = None;
                }
                Flow::Done
            }
            // 破阵诀（zhenfa，规则 v201/T19b）：n 次随机座位交换（n=存活
            // 数，上游 swapSeat(random, random) ×n 同口径）；本体入弃堆。
            HostBehavior::SwdPozhenjueCard => {
                let n = ctx.core.alive_players().count();
                for _ in 0..n {
                    let seats: Vec<u8> = ctx.core.alive_players().collect();
                    if seats.len() < 2 {
                        break;
                    }
                    let i = ctx.core.rng.gen_range(seats.len() as u32) as usize;
                    let j = ctx.core.rng.gen_range(seats.len() as u32) as usize;
                    ctx.core.swap_seat(seats[i], seats[j]);
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
                                ctx.core.skills.insert(
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
