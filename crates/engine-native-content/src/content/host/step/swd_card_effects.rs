//! 轩辕剑（swd）宝物/锦囊牌效果（神农鼎、四非珍面、昆仑镜、东皇钟、流星火雨、羊皮卷、指路小狐、木甲人、勾魂锣）。

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
            _ => unreachable!("behavior routed to the wrong step module"),
        }
    }
}
