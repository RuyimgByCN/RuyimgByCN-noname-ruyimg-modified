use super::super::*;

impl HostMachine {
    pub(super) fn step_skills_damage(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            // 反击：授予临时红杀直伤 → 选一张【杀】对来源使用
            HostBehavior::YxsFanji => match self.stage {
                HostStage::Act => {
                    let Some(EventData::Damage(d)) = ctx
                        .res
                        .frame(ctx.current_event_frame_id())
                        .map(|f| f.data.clone())
                    else {
                        return Flow::Done;
                    };
                    let Some(source) = d.source else {
                        return Flow::Done;
                    };
                    self.target = Some(source);
                    // 授予临时「反击·红杀直伤」
                    if let Some(def) = ctx.catalog.skill_by_name(U08_YXS_FANJI2_KEY) {
                        ctx.insert_skill(
                            (self.owner, def),
                            crate::state::SkillInstance {
                                owner: self.owner,
                                enabled: true,
                                once_used: false,
                                expires_end_of_turn: true,
                                rounds: 0,
                            },
                        );
                    }
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| ctx.catalog.card_kind(*c).is_sha_like())
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
                            prompt: "yxs_fanji_sha",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(target) = self.target else {
                        return Flow::Done;
                    };
                    let Some(ChoiceOption::Card(card)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(self.owner).hand.contains(&card)
                        || !ctx.core.player(target).alive
                    {
                        return Flow::Done;
                    }
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "反击",
                        owner: self.owner, presentation: None });
                    ctx.emit(EventData::CardUsed(crate::machine::card_use(
                        card,
                        self.owner,
                        vec![target],
                    )));
                    self.stage = HostStage::ShuchuiTargetLoop;
                    Flow::Yield
                }
                _ => Flow::Done,
            },
            // 禳星：累计计数；达 3 判定免除/失星，随后分配 3 点雷伤
            HostBehavior::HlssXiangxing => {
                let damage_num = ctx
                    .current_damage()
                    .map(|d| d.amount.max(0) as u32)
                    .or_else(|_| ctx.current_lose_hp())
                    .unwrap_or(0);
                let p = ctx.core.player_mut(self.owner);
                p.hlss_count = p.hlss_count.saturating_add(damage_num.min(255) as u8);
                if ctx.core.player(self.owner).hlss_count < 3 {
                    return Flow::Done;
                }
                // 隐月免除：当前星数对应的伤害类型已在记录位图中
                let stars = ctx.core.player(self.owner).hlss_stars;
                let recorded = ctx.core.player(self.owner).hlss_damage;
                let need = match stars {
                    3 => HLSS_DAMAGE_THUNDER,
                    2 => HLSS_DAMAGE_FIRE,
                    1 => HLSS_DAMAGE_TRICK,
                    _ => 0,
                };
                let has_yueyin = ctx
                    .catalog
                    .skill_by_name(HLSS_YUEYIN_KEY)
                    .map(|d| ctx.core.has_skill(self.owner, d))
                    .unwrap_or(false);
                if has_yueyin && need != 0 && recorded & need != 0 {
                    // 免除：不失星、不放雷；仅当星已归零时回满体力
                    if stars == 0 {
                        let max = ctx.core.player(self.owner).max_hp as i32;
                        ctx.core.player_mut(self.owner).hp = max;
                    }
                    return Flow::Done;
                }
                // 失星：星 -1、计数清零、记录清空
                {
                    let p = ctx.core.player_mut(self.owner);
                    p.hlss_stars = p.hlss_stars.saturating_sub(1);
                    p.hlss_count = 0;
                    p.hlss_damage = 0;
                }
                // 雷伤分配（上游：全场随机 1 名 +1；随机敌人两名各 +1，
                // 无敌人时退化为全场）
                let others: Vec<PlayerId> = ctx
                    .core
                    .alive_players()
                    .filter(|p| *p != self.owner)
                    .collect();
                if others.is_empty() {
                    return Flow::Done;
                }
                let mut damage_map: std::collections::BTreeMap<PlayerId, u32> =
                    std::collections::BTreeMap::new();
                let pick = others[ctx.core.rng.gen_range(others.len() as u32) as usize];
                *damage_map.entry(pick).or_insert(0) += 1;
                let enemies: Vec<PlayerId> = others
                    .iter()
                    .copied()
                    .filter(|p| ctx.core.is_boss_side(*p) != ctx.core.is_boss_side(self.owner))
                    .collect();
                let enemy_pool = if enemies.is_empty() {
                    others.clone()
                } else {
                    enemies
                };
                for _ in 0..2 {
                    let e = enemy_pool[ctx.core.rng.gen_range(enemy_pool.len() as u32) as usize];
                    *damage_map.entry(e).or_insert(0) += 1;
                }
                for (target, num) in damage_map {
                    if !ctx.core.player(target).alive {
                        continue;
                    }
                    ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                        Some(self.owner),
                        target,
                        num as i32,
                        crate::event::DamageNature::Thunder,
                        None,
                    )));
                }
                // 星归零：回满体力并觉醒（不再回满）
                if ctx.core.player(self.owner).hlss_stars == 0 {
                    let max = ctx.core.player(self.owner).max_hp as i32;
                    ctx.core.player_mut(self.owner).hp = max;
                    if let Some(def) = ctx.catalog.skill_by_name(HLSS_XIANGXING_KEY) {
                        if let Some(inst) = ctx.core.skills.get_mut(&(self.owner, def)) {
                            inst.enabled = false;
                        }
                    }
                    ctx.log(LogEvent::SkillToggled {
                        player: self.owner,
                        skill: ctx.catalog.skill_name(
                            ctx.catalog
                                .skill_by_name(HLSS_XIANGXING_KEY)
                                .expect("hlss 禳星在目录"),
                        ),
                        enabled: false,
                    });
                }
                Flow::Done
            }
            // 魔焰：选 N 张红牌 → 选恰 N 名其他角色 → 逐个 1 点火焰伤害
            HostBehavior::Moyan => match self.stage {
                HostStage::Act => {
                    let reds: Vec<ChoiceOption> = ctx
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
                    if reds.is_empty() {
                        return Flow::Done;
                    }
                    let others = ctx
                        .core
                        .alive_players()
                        .filter(|p| *p != self.owner)
                        .count();
                    let hp_gap = (ctx.core.player(self.owner).max_hp as i32
                        - ctx.core.player(self.owner).hp)
                        .max(0) as usize;
                    let max = reds.len().min(others).min(hp_gap);
                    if max == 0 {
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "moyan_cards",
                            options: reds,
                            min: 1,
                            max,
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
                    if picked.is_empty() {
                        return Flow::Done;
                    }
                    // 弃置红色手牌（phaseUse 用牌消耗）
                    for c in picked.iter().copied() {
                        if ctx.core.player(self.owner).hand.contains(&c) {
                            ctx.discard_from_hand(self.owner, c);
                        }
                    }
                    let need = picked.len();
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| *p != self.owner)
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.len() < need {
                        return Flow::Done;
                    }
                    self.cards = picked.iter().map(|c| u32::from(*c)).collect();
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "moyan_targets",
                            options,
                            min: need,
                            max: need,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let targets: Vec<PlayerId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Player(p) => Some(*p),
                            _ => None,
                        })
                        .collect();
                    if targets.is_empty() {
                        return Flow::Done;
                    }
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "魔焰",
                        owner: self.owner, presentation: None });
                    self.targets = targets;
                    self.stage = HostStage::ShuchuiTargetLoop;
                    Flow::Yield
                }
                HostStage::ShuchuiTargetLoop => {
                    // 逐个目标造成 1 点火焰伤害（保持目标序：每次发一帧再让出）
                    if self.targets.is_empty() {
                        return Flow::Done;
                    }
                    let target = self.targets.remove(0);
                    if ctx.core.player(target).alive {
                        ctx.emit(EventData::Damage(crate::event::DamageData {
                            settlement: Default::default(),
                            source: Some(self.owner),
                            target,
                            amount: 1,
                            nature: crate::event::DamageNature::Fire,
                            prevented: false,
                            redirect_chain: Vec::new(),
                            card: None,
                            from_chain: false,
                        }));
                    }
                    if self.targets.is_empty() {
                        Flow::Done
                    } else {
                        Flow::Yield
                    }
                }
                _ => Flow::Done,
            },
            // 轩辕剑·伤增：帧内 +1 并转雷属性、置标记
            HostBehavior::XuanyuanjianDamage => {
                let frame = ctx.current_event_frame_id();
                if let Some(f) = ctx.res.frame_mut(frame) {
                    if let EventData::Damage(d) = &mut f.data {
                        if let Some(amount) = d.amount.checked_add(1) {
                            d.amount = amount;
                        } else {
                            ctx.fail_damage_amount();
                            return Flow::Done;
                        }
                        if d.nature == crate::event::DamageNature::Normal {
                            d.nature = crate::event::DamageNature::Thunder;
                        }
                    }
                }
                ctx.core.turn.xuanyuanjian_marked = true;
                ctx.log(LogEvent::SkillTriggered {
                    skill: "轩辕剑",
                    owner: self.owner, presentation: None });
                Flow::Done
            }
            // 天香：选红桃手牌 → 选目标 → 弃牌 + 伤害转移 + 授予天香·受
            HostBehavior::Tianxiang => match self.stage {
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
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
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "tianxiang_heart",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(ChoiceOption::Card(card)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(self.owner).hand.contains(&card) {
                        return Flow::Done;
                    }
                    self.card = Some(card);
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| *p != self.owner)
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
                            prompt: "tianxiang_target",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(ChoiceOption::Player(target)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    let Some(card) = self.card else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(target).alive
                        || !ctx.core.player(self.owner).hand.contains(&card)
                    {
                        return Flow::Done;
                    }
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "天香",
                        owner: self.owner, presentation: None });
                    // 弃红桃牌
                    ctx.discard_from_hand(self.owner, card);
                    // 伤害转移（帧 target 改写 + 转移链记录）
                    {
                        let frame = ctx
                            .res
                            .frame_mut(ctx.current_event_frame_id())
                            .expect("frame");
                        if let EventData::Damage(d) = &mut frame.data {
                            d.redirect_chain.push(d.target);
                            d.target = target;
                        }
                    }
                    // 授予「天香·受」（运行时实例；承受后按其失去体力摸牌）
                    if let Some(def) = ctx.catalog.skill_by_name(SWD_TIANXIANG2_KEY) {
                        if !ctx.core.skills.contains_key(&(target, def)) {
                            ctx.insert_skill(
                                (target, def),
                                crate::state::SkillInstance {
                                    owner: target,
                                    enabled: true,
                                    once_used: false,
                                    expires_end_of_turn: false,
                                    rounds: 0,
                                },
                            );
                            ctx.log(LogEvent::SkillToggled {
                                player: target,
                                skill: ctx.catalog.skill_name(def),
                                enabled: true,
                            });
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 清正：选花色 → 选目标 → 选目标花色 → 双向弃牌 → 少弃者受伤
            HostBehavior::MbcmQingzheng => match self.stage {
                HostStage::Act => {
                    let suits = mbcm_qingzheng_suits(ctx, self.owner);
                    if suits.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectNature;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::PickTag,
                            prompt: "mbcmqingzheng_suit",
                            options: suits.into_iter().map(ChoiceOption::Tag).collect(),
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectNature => {
                    let Some(ChoiceOption::Tag(suit)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    let mut chosen: Vec<CardId> = Vec::new();
                    for c in ctx.core.player(self.owner).hand.iter().copied() {
                        if mbcm_suit_of(ctx, c) == Some(suit) {
                            chosen.push(c);
                        }
                    }
                    if chosen.is_empty() {
                        return Flow::Done;
                    }
                    self.cards = chosen.iter().map(|c| u32::from(*c)).collect();
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| *p != self.owner && !ctx.core.player(*p).hand.is_empty())
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
                            prompt: "mbcmqingzheng_target",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(ChoiceOption::Player(target)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    self.target = Some(target);
                    let suits = mbcm_qingzheng_suits(ctx, target);
                    if suits.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::CompareTargetPick;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::PickTag,
                            prompt: "mbcmqingzheng_target_suit",
                            options: suits.into_iter().map(ChoiceOption::Tag).collect(),
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::CompareTargetPick => {
                    let Some(ChoiceOption::Tag(suit)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    let Some(target) = self.target else {
                        return Flow::Done;
                    };
                    let mine: Vec<CardId> = self.cards.iter().map(|c| *c as CardId).collect();
                    let theirs: Vec<CardId> = ctx
                        .core
                        .player(target)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| mbcm_suit_of(ctx, *c) == Some(suit))
                        .collect();
                    let mine_n = mine.len();
                    let theirs_n = theirs.len();
                    for c in mine {
                        ctx.discard_from_hand(self.owner, c);
                    }
                    for c in theirs {
                        ctx.discard_from_hand(target, c);
                    }
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "清正",
                        owner: self.owner, presentation: None });
                    if theirs_n < mine_n {
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            None,
                            target,
                            1,
                            crate::event::DamageNature::Normal,
                            None,
                        )));
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::TargetDiscardEquipOrDamage => match self.stage {
                HostStage::TargetChoice => {
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    let spec = Self::target_choice_spec(ctx, t);
                    // card.js:41-44：目标 he 区无装备牌时，直接受到使用者造成的 1 点普通伤害（不弹窗口）
                    if spec.options.len() <= 1 {
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            Some(self.owner),
                            t,
                            1,
                            crate::event::DamageNature::Normal,
                            self.card,
                        )));
                        if let Some(c) = self.card {
                            ctx.to_discard(c);
                            ctx.core.table.retain(|x| *x != c);
                        }
                        return Flow::Done;
                    }
                    self.stage = HostStage::Act;
                    Flow::WaitChoice { observer: t, spec }
                }
                HostStage::Act => {
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    match ctx.answer().first().copied() {
                        Some(ChoiceOption::Card(equip)) => {
                            if ctx.core.player(t).hand.contains(&equip)
                                && ctx.catalog.card_slot_of(ctx.core, equip).is_some()
                            {
                                ctx.discard_from_hand(t, equip);
                                ctx.log(LogEvent::CardDiscarded {
                                    player: t,
                                    cards: vec![equip],
                                });
                            } else if ctx.core.player(t).equips.contains(&equip) {
                                ctx.core.player_mut(t).equips.retain(|x| *x != equip);
                                crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, t, equip);
                                ctx.to_discard(equip);
                                ctx.log(LogEvent::CardDiscarded {
                                    player: t,
                                    cards: vec![equip],
                                });
                            }
                        }
                        _ => {
                            // 弃权/取消：受 1 点伤害（来源 = 使用者，牌归因 = 手里剑）
                            let source = Some(self.owner);
                            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                                source,
                                t,
                                1,
                                crate::event::DamageNature::Normal,
                                self.card,
                            )));
                        }
                    }
                    if let Some(c) = self.card {
                        ctx.to_discard(c);
                        ctx.core.table.retain(|x| *x != c);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::MaxRangeWeaponDamage => match self.stage {
                HostStage::TargetChoice => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| *p != self.owner)
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::Act;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: D3_JINGGOU_TARGET,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::Act => {
                    if let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() {
                        if ctx.core.player(t).alive {
                            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                                Some(self.owner),
                                t,
                                1,
                                crate::event::DamageNature::Normal,
                                None,
                            )));
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::SanjianWeaponDamage => match self.stage {
                HostStage::SelectCard => {
                    let Some(damaged) = self.target else {
                        return Flow::Done;
                    };
                    // 合法目标：与受损者距离 ≤1 的其他存活角色
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            *p != self.owner
                                && *p != damaged
                                && ctx.core.computed_distance(ctx.catalog, damaged, *p) <= 1
                        })
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() || ctx.core.player(self.owner).hand.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectNature;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WD_SANJIAN_DISCARD,
                            options: ctx
                                .core
                                .player(self.owner)
                                .hand
                                .iter()
                                .copied()
                                .map(ChoiceOption::Card)
                                .collect(),
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectNature => {
                    // 记住所弃手牌（复用 cards[0]）
                    if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                        self.cards = vec![c];
                    }
                    self.stage = HostStage::Act;
                    let Some(damaged) = self.target else {
                        return Flow::Done;
                    };
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            *p != self.owner
                                && *p != damaged
                                && ctx.core.computed_distance(ctx.catalog, damaged, *p) <= 1
                        })
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: WD_SANJIAN_TARGET,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::Act => {
                    if let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() {
                        if let Some(&c) = self.cards.first() {
                            if ctx.core.player(self.owner).hand.contains(&c) {
                                ctx.discard_from_hand(self.owner, c);
                                ctx.log(LogEvent::CardDiscarded {
                                    player: self.owner,
                                    cards: vec![c],
                                });
                            }
                            if ctx.core.player(t).alive {
                                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                                    Some(self.owner),
                                    t,
                                    1,
                                    crate::event::DamageNature::Normal,
                                    None,
                                )));
                            }
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::LinghunzhihuoCard => {
                // 目标受 1 点火焰伤害，使用者弃一张手牌（确定性首张）
                let Some(t) = self.target else {
                    return Flow::Done;
                };
                if ctx.core.player(t).alive {
                    ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                        Some(self.owner),
                        t,
                        1,
                        crate::event::DamageNature::Fire,
                        self.card,
                    )));
                }
                if let Some(c) = ctx.core.player(self.owner).hand.first().copied() {
                    ctx.discard_from_hand(self.owner, c);
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![c],
                    });
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            }
            HostBehavior::CaochuanCard => match self.stage {
                // 分支选择（PickTag：0=交杀+反噬 1=展示+被弃一张）
                HostStage::SelectNature => {
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    let has_sha = ctx
                        .core
                        .player(t)
                        .hand
                        .iter()
                        .any(|c| ctx.catalog.card_kind(*c) == CardKind::Sha);
                    if !has_sha {
                        // 无杀：直接走展示+被弃分支
                        self.stage = HostStage::Act;
                        self.cards = vec![1];
                        let options = vec![ChoiceOption::Tag(1)];
                        return Flow::WaitChoice {
                            observer: self.owner,
                            spec: ChoiceSpec {
                                kind: ChoiceKind::PickTag,
                                prompt: WD_CAOCHUAN_BRANCH,
                                options,
                                min: 1,
                                max: 1,
                                allow_cancel: false,
                            },
                        };
                    }
                    self.stage = HostStage::Act;
                    let options = vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)];
                    Flow::WaitChoice {
                        observer: t,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::PickTag,
                            prompt: WD_CAOCHUAN_BRANCH,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::Act => {
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    match ctx.answer().first() {
                        Some(ChoiceOption::Tag(0)) => {
                            // 交出所有手牌杀给使用者；使用者受 1 点反噬杀
                            //（登记差异：上游走完整杀结算）
                            let shas: Vec<CardId> = ctx
                                .core
                                .player(t)
                                .hand
                                .iter()
                                .copied()
                                .filter(|c| ctx.catalog.card_kind(*c) == CardKind::Sha)
                                .collect();
                            for c in &shas {
                                ctx.core.player_mut(t).hand.retain(|x| x != c);
                                ctx.core.player_mut(self.owner).hand.push(*c);
                            }
                            if !shas.is_empty() {
                                ctx.log(LogEvent::CardsGiven {
                                    from: t,
                                    to: self.owner,
                                    cards: shas,
                                });
                            }
                            if ctx.core.player(self.owner).alive {
                                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                                    Some(t),
                                    self.owner,
                                    1,
                                    crate::event::DamageNature::Normal,
                                    None,
                                )));
                            }
                        }
                        _ => {
                            // 展示手牌 + 被弃一张（确定性首张）
                            if let Some(c) = ctx.core.player(t).hand.first().copied() {
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
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::D3Moyan => {
                let frame_id = ctx.current_event_frame_id();
                let is_damage = ctx
                    .res
                    .frame(frame_id)
                    .is_some_and(|frame| matches!(frame.data, EventData::Damage(_)));
                if is_damage {
                    if !ctx.modify_damage_amount(1) {
                        return Flow::Done;
                    }
                    Flow::Done
                } else {
                    moyan_round_step(self, ctx)
                }
            }
            _ => unreachable!("behavior routed to the wrong step module"),
        }
    }
}
