use super::super::*;

impl HostMachine {
    pub(super) fn step_gwent(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            HostBehavior::GwentYanzhiCard => {
                // 目标为手牌数最少者（之一）→ 摸 2；否则摸 1
                let Some(t) = self.target else {
                    return Flow::Done;
                };
                if ctx.core.player(t).alive {
                    let min_hand = ctx
                        .core
                        .alive_players()
                        .all(|p| ctx.core.player(p).hand.len() >= ctx.core.player(t).hand.len());
                    let n = if min_hand { 2 } else { 1 };
                    ctx.draw_cards(t, n);
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            }
            HostBehavior::GwentAozuCard => {
                // 1 点雷电伤害，若存活摸 1
                let Some(t) = self.target else {
                    return Flow::Done;
                };
                if ctx.core.player(t).alive {
                    ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                        Some(self.owner),
                        t,
                        1,
                        crate::event::DamageNature::Thunder,
                        self.card,
                    )));
                    if ctx.core.player(t).alive {
                        ctx.draw_cards(t, 1);
                    }
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            }
            HostBehavior::GwentWenyiCard => {
                // 有手牌 → 弃首张（登记差异：随机改首张）；无手牌 →
                // 失去 1 点体力（规则 v77 原语）
                let Some(t) = self.target else {
                    return Flow::Done;
                };
                if ctx.core.player(t).alive {
                    // 上游 target.randomDiscard("h")：随机一张手牌
                    //（逐技对照批四 v78 种子 RNG 均匀抽取）
                    let hand_len = ctx.core.player(t).hand.len();
                    if hand_len > 0 {
                        let i = ctx.core.rng.gen_range(hand_len as u32) as usize;
                        let c = ctx.core.player(t).hand[i];
                        ctx.discard_from_hand(t, c);
                        ctx.log(LogEvent::CardDiscarded {
                            player: t,
                            cards: vec![c],
                        });
                    } else {
                        // 失去 1 点体力（逐技对照批三 v77 消除登记差异）
                        ctx.emit(EventData::LoseHp {
                            player: t,
                            amount: 1,
                        });
                    }
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            }
            HostBehavior::GwentZhihuanCard => {
                // 反复失去 1 点体力上限，直至未受伤或手牌数 ≥ 体力上限
                let Some(t) = self.target else {
                    return Flow::Done;
                };
                if ctx.core.player(t).alive {
                    loop {
                        let p = ctx.core.player(t);
                        let damaged = p.hp < p.max_hp as i32;
                        if !(damaged && (p.hand.len() as u32) < p.max_hp) {
                            break;
                        }
                        let p = ctx.core.player_mut(t);
                        p.max_hp -= 1;
                        if p.hp > p.max_hp as i32 {
                            p.hp = p.max_hp as i32;
                        }
                    }
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            }
            HostBehavior::GwentBaoxueCard => match self.stage {
                HostStage::SelectCard => {
                    // 上游 chooseToDiscard("h",2,true)：目标自选两张手牌
                    //（不足则全选，逐技对照批五 v79 消除「前两张」登记差异）
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    let n = ctx.core.player(t).hand.len().min(2);
                    if n == 0 || !ctx.core.player(t).alive {
                        // 无手牌：跳过弃牌；上游 step1 draw 恒执行
                        if ctx.core.player(t).alive {
                            ctx.draw_cards(t, 1);
                        }
                        if let Some(c) = self.card {
                            ctx.to_discard(c);
                            ctx.core.table.retain(|x| *x != c);
                        }
                        return Flow::Done;
                    }
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(t)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    self.stage = HostStage::Act;
                    Flow::WaitChoice {
                        observer: t,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: GW_BAOXUE_DISCARD,
                            options,
                            min: n,
                            max: n,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::Act => {
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
                    let mut discarded = Vec::new();
                    for c in picks {
                        if ctx.core.player(t).hand.contains(&c) {
                            ctx.core.player_mut(t).hand.retain(|x| *x != c);
                            discarded.push(c);
                        }
                    }
                    if !discarded.is_empty() {
                        ctx.log(LogEvent::CardDiscarded {
                            player: t,
                            cards: discarded,
                        });
                    }
                    ctx.draw_cards(t, 1);
                    if let Some(c) = self.card {
                        ctx.to_discard(c);
                        ctx.core.table.retain(|x| *x != c);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::GwentQinpenState | HostBehavior::GwentBirinState => {
                // 天气到期移除（效果为被动钩子，无步骤）
                food_remove(ctx, self.owner, &self.def_key);
                Flow::Done
            }
            HostBehavior::GwentCiguState => {
        // 上游 `gw_ciguhanshuang` content: `trigger.num--; player.removeSkill(...)`
        // ⇒ 摸牌数 -1 由 `kernel/phase/draw.rs` 计算，这里**只移除状态**
        //（第 938 轮修正：原实现是「弃首张手牌」，效果不对）
        food_remove(ctx, self.owner, &self.def_key);
        Flow::Done
    }
    HostBehavior::GwentWeatherGrant => {
                // 对 targets（目标 + 距离 1 邻座）逐个安装对应天气状态
                //（已持有者跳过，对齐上游 filterTarget）
                let kind = self.card.map(|c| ctx.catalog.card_kind(c));
                let listener = match kind {
                    Some(CardKind::Qinpendayu) => Some(GW_QINPEN_STATE_KEY),
                    Some(CardKind::Birinongwu) => Some(GW_BIRIN_STATE_KEY),
                    Some(CardKind::Ciguhanshuang) => Some(GW_CIGU_STATE_KEY),
                    _ => None,
                };
                let Some(listener) = listener else {
                    return Flow::Done;
                };
                let Some(def) = ctx.catalog.skill_by_name(listener) else {
                    return Flow::Done;
                };
                let targets = self.targets.clone();
                for t in targets {
                    if !ctx.core.player(t).alive || ctx.core.skills.contains_key(&(t, def)) {
                        continue;
                    }
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
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            }
            HostBehavior::GwentKunenState => {
                // 防止此次普通伤害并按次数扣减计数，归零移除状态
                ctx.current_damage_mut().prevented = true;
                food_tick(ctx, self.owner, &self.def_key);
                Flow::Done
            }
            HostBehavior::GwentKunenCard => {
                // 授予昆恩状态：rounds = min(5, 存活数)
                let Some(t) = self.target else {
                    return Flow::Done;
                };
                let Some(def) = ctx.catalog.skill_by_name(GW_KUNEN_STATE_KEY) else {
                    return Flow::Done;
                };
                let rounds = (ctx.core.alive_count() as u8).min(5);
                ctx.insert_skill(
                    (t, def),
                    crate::state::SkillInstance {
                        owner: t,
                        enabled: true,
                        once_used: false,
                        expires_end_of_turn: false,
                        rounds,
                    },
                );
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            }
            HostBehavior::GwentShanbengCard => {
                // 所有其他存活角色装备区合集均匀随机取两张，恰两张才弃置
                //（上游 equips.randomGets(2) + length===2 门控；逐技对照
                // 批四 v78 种子 RNG，无序对均匀：i + j 越过法）
                let pool: Vec<(crate::ids::PlayerId, CardId)> = ctx
                    .core
                    .alive_players()
                    .filter(|v| *v != self.owner)
                    .flat_map(|v| {
                        ctx.core
                            .player(v)
                            .equips
                            .iter()
                            .copied()
                            .map(move |c| (v, c))
                            .collect::<Vec<_>>()
                    })
                    .collect();
                if pool.len() >= 2 {
                    let n = pool.len() as u32;
                    let i = ctx.core.rng.gen_range(n) as usize;
                    let mut j = ctx.core.rng.gen_range(n - 1) as usize;
                    if j >= i {
                        j += 1;
                    }
                    for (v, c) in [pool[i], pool[j]] {
                        ctx.core.player_mut(v).equips.retain(|x| *x != c);
                        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, v, c);
                        ctx.to_discard(c);
                        ctx.log(LogEvent::CardDiscarded {
                            player: v,
                            cards: vec![c],
                        });
                    }
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            }
            HostBehavior::GwentPoxiaoCard => match self.stage {
                HostStage::Act => {
                    self.stage = HostStage::SelectCard;
                    let options = vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)];
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::PickTag,
                            prompt: GW_POXIAO_BRANCH,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    // 分支 0：清除所有天气状态并弃置全部判定区牌
                    // 分支 1：获得牌堆中首张天气/暴雪法术（登记差异：
                    // 随机改牌堆序首张；上游分支一为自选角色）
                    match ctx.answer_first() {
                        ChoiceOption::Tag(0) => {
                            let weather_defs: Vec<_> =
                                [GW_QINPEN_STATE_KEY, GW_BIRIN_STATE_KEY, GW_CIGU_STATE_KEY]
                                    .iter()
                                    .filter_map(|k| ctx.catalog.skill_by_name(k))
                                    .collect();
                            let mut to_remove = Vec::new();
                            for ((o, def), _) in ctx.core.skills.iter() {
                                if weather_defs.contains(def) {
                                    to_remove.push((*o, *def));
                                }
                            }
                            for key in to_remove {
                                ctx.core.skills.remove(&key);
                            }
                            for p in ctx.core.alive_players().collect::<Vec<_>>() {
                                let judge: Vec<CardId> = ctx.core.player(p).judge.clone();
                                for c in judge {
                                    ctx.core.player_mut(p).judge.retain(|x| *x != c);
                                    ctx.to_discard(c);
                                }
                            }
                        }
                        ChoiceOption::Tag(1) => {
                            // 随机获得一张铜卡法术（上游随机；逐技对照批四
                            // v78 种子 RNG 均匀抽取）
                            let bronze: Vec<CardKind> = vec![
                                CardKind::Qinpendayu,
                                CardKind::Birinongwu,
                                CardKind::Ciguhanshuang,
                                CardKind::Baoxueyaoshui,
                            ];
                            let positions: Vec<usize> = ctx
                                .core
                                .deck
                                .iter()
                                .enumerate()
                                .filter(|(_, c)| bronze.contains(&ctx.catalog.card_kind(**c)))
                                .map(|(i, _)| i)
                                .collect();
                            if positions.is_empty() {
                                ctx.draw_cards(self.owner, 1);
                            } else {
                                let i = positions
                                    [ctx.core.rng.gen_range(positions.len() as u32) as usize];
                                let c = ctx.core.deck.remove(i);
                                ctx.core.player_mut(self.owner).hand.push(c);
                            }
                        }
                        _ => {}
                    }
                    if let Some(c) = self.card {
                        ctx.to_discard(c);
                        ctx.core.table.retain(|x| *x != c);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::GwentFengyinState => {
                // 拥有者任意阶段结束：解除封印——全部技能实例重新
                // 启用，并移除封印实例自身
                for ((p, _), inst) in ctx.core.skills.iter_mut() {
                    if *p == self.owner {
                        inst.enabled = true;
                    }
                }
                if let Some(def) = ctx.catalog.skill_by_name(GW_FENGYIN_STATE_KEY) {
                    ctx.core.skills.remove(&(self.owner, def));
                }
                Flow::Done
            }
            HostBehavior::GwentZumoCard => {
                // 目标技能全部失效（授予封印实例 + 禁用其余实例；
                // 登记差异：原生无锁定技区分）
                let Some(t) = self.target else {
                    return Flow::Done;
                };
                let Some(fdef) = ctx.catalog.skill_by_name(GW_FENGYIN_STATE_KEY) else {
                    return Flow::Done;
                };
                let keys: Vec<(PlayerId, crate::ids::SkillDefId)> = ctx
                    .core
                    .skills
                    .iter()
                    .map(|((p, d), _)| (*p, *d))
                    .filter(|(p, d)| *p == t && *d != fdef)
                    .collect();
                for k in keys {
                    if let Some(inst) = ctx.core.skills.get_mut(&k) {
                        inst.enabled = false;
                    }
                }
                ctx.insert_skill(
                    (t, fdef),
                    crate::state::SkillInstance {
                        owner: t,
                        enabled: true,
                        once_used: false,
                        expires_end_of_turn: false,
                        rounds: 0,
                    },
                );
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
