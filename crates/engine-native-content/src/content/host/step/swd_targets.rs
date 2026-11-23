use super::super::*;

impl HostMachine {
    pub(super) fn step_swd_targets(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            // 浴尘：Act 选目标（he 非空其他角色）→ 目标自选弃其一张 he 牌
            HostBehavior::SwdYuchen => match self.stage {
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            *p != self.owner && {
                                let pl = ctx.core.player(*p);
                                !pl.hand.is_empty() || !pl.equips.is_empty()
                            }
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
                            prompt: "yuchen_target",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(t) = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(t).alive {
                        return Flow::Done;
                    }
                    let options = super::u05_7c_skills::he_card_options(ctx.core, t);
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.target = Some(t);
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "yuchen_discard",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    if let (Some(t), Some(ChoiceOption::Card(c))) =
                        (self.target, ctx.answer().first().copied())
                    {
                        let _ = super::u05_7c_skills::discard_he_card(ctx, t, c);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 挥琪：TargetChoice 选一名其他角色 → 摸 X（X=自己已损失体力）
            HostBehavior::SwdHuiqi => match self.stage {
                HostStage::Act => {
                    let x =
                        ctx.core.player(self.owner).max_hp as i32 - ctx.core.player(self.owner).hp;
                    if x <= 0 {
                        return Flow::Done;
                    }
                    self.card = Some(x as u32);
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
                            prompt: "huiqi_target",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(t) = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    if ctx.core.player(t).alive {
                        let x = self.card.take().unwrap_or(0);
                        if x > 0 {
                            ctx.start_draw(t, x);
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 断行：TargetChoice 选杀合法目标（距离内）→ 虚拟杀
            HostBehavior::SwdDuanxing => match self.stage {
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            *p != self.owner
                                && ctx.core.computed_distance(ctx.catalog, self.owner, *p)
                                    <= ctx.core.attack_range(ctx.catalog, self.owner)
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
                            prompt: "duanxing_target",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(t) = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    if ctx.core.player(t).alive {
                        ctx.use_virtual_card(self.owner, CardKind::Sha, vec![t]);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 敷衍·护甲面：TargetChoice 选无护甲角色 → +1 护甲
            HostBehavior::SwdFuyan => match self.stage {
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| ctx.core.player(*p).hujia == 0)
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
                            prompt: "fuyan_target",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(t) = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    if ctx.core.player(t).alive {
                        ctx.change_hujia(t, 1);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 聒噪·疾鹏面：TargetChoice 距离 2 内有手牌角色（含自己）→
            // 其手牌全部弃置，牌堆顶三张入手（上游 lose._triggered=null +
            // gain "draw" 同口径；登记差异：上游观看面为二段交互，原生
            // 单窗收敛）
            HostBehavior::SwdGuozao => match self.stage {
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            !ctx.core.player(*p).hand.is_empty()
                                && ctx.core.computed_distance(ctx.catalog, self.owner, *p) <= 2
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
                            prompt: "guozao_target",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(t) = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(t).alive {
                        return Flow::Done;
                    }
                    // 牌堆顶三张（不足先洗弃堆；仍不足则按实有张数）
                    let mut top: Vec<CardId> = Vec::new();
                    for _ in 0..3 {
                        if ctx.core.deck.is_empty() {
                            if ctx.core.discard.is_empty() {
                                break;
                            }
                            let mut cards = std::mem::take(&mut ctx.core.discard);
                            ctx.core.rng.shuffle(&mut cards);
                            ctx.core.deck = cards;
                            ctx.log(LogEvent::DeckReshuffled);
                        }
                        top.push(ctx.core.deck.pop().expect("guozao top"));
                    }
                    if top.is_empty() {
                        return Flow::Done;
                    }
                    // 目标手牌全部弃置（上游 lose(...)._triggered=null 同
                    // 口径不发射弃置事件），三张入手（gain "draw"）
                    let taken: Vec<CardId> = ctx.core.player(t).hand.iter().copied().collect();
                    for c in &taken {
                        ctx.take_hand_card(t, *c);
                        ctx.to_discard(*c);
                    }
                    for c in &top {
                        ctx.core.player_mut(t).hand.push(*c);
                    }
                    ctx.log(LogEvent::CardsDrawn {
                        player: t,
                        cards: top.clone(),
                    });
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 真威：确认 → TargetChoice（非响应者存活角色）→ 弃堆牌入手
            HostBehavior::SwdFzhenwei => match self.stage {
                HostStage::Act => {
                    let Some(crate::event::EventData::CardResponded { card, player }) =
                        ctx.res.frame(ctx.current_event_frame_id()).map(|f| &f.data)
                    else {
                        return Flow::Done;
                    };
                    let c = *card;
                    let responder = *player;
                    self.card = Some(c);
                    self.target = Some(responder);
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| *p != responder)
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
                            prompt: "fzhenwei_target",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(t) = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    let Some(c) = self.card else {
                        return Flow::Done;
                    };
                    if ctx.core.discard.contains(&c) && ctx.core.player(t).alive {
                        ctx.core.discard.retain(|x| *x != c);
                        ctx.core.player_mut(t).hand.push(c);
                        ctx.log(LogEvent::CardsGiven {
                            from: self.target.unwrap_or(self.owner),
                            to: t,
                            cards: vec![c],
                        });
                    }
                    self.card = None;
                    self.target = None;
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 还灵：Act 问选其他角色 → TargetChoice 自己与目标同时翻面
            //（取消=不发动；上游 turnOver×2）
            HostBehavior::SwdHuanling => match self.stage {
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = (0..ctx.core.players.len())
                        .filter(|&i| {
                            i as PlayerId != self.owner && ctx.core.player(i as PlayerId).alive
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
                            prompt: "huanling_target",
                            options,
                            min: 0,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let picked: Option<PlayerId> = ctx.answer().iter().find_map(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    });
                    let Some(t) = picked else {
                        // 取消：双方都不翻
                        return Flow::Done;
                    };
                    for p in [t, self.owner] {
                        if ctx.core.player(p).alive {
                            ctx.emit(EventData::TurnOverBefore {
                                player: p,
                                cancelled: false,
                            });
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 破舟·封印（swd/skill.js:3415）：多选 1..pozhou_count 名未封印其他角色，按人数扣计数并仅禁用非锁定技
            HostBehavior::SwdPozhou2 => match self.stage {
                HostStage::Act => {
                    let cap = ctx.core.player(self.owner).pozhou_count as usize;
                    if cap == 0 {
                        return Flow::Done;
                    }
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            if *p == self.owner {
                                return false;
                            }
                            let sealed = ctx
                                .catalog
                                .skill_by_name(SWD_POZHOU_SEALED_KEY)
                                .map(|d| ctx.core.has_skill(*p, d))
                                .unwrap_or(false);
                            !sealed
                        })
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    let max = cap.min(options.len());
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "pozhou_target",
                            options,
                            min: 1,
                            max,
                            allow_cancel: true,
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
                    let cap = ctx.core.player(self.owner).pozhou_count as usize;
                    let take = targets.len().min(cap);
                    ctx.core.player_mut(self.owner).pozhou_count =
                        (cap - take) as u8;
                    if let Some(sdef) = ctx.catalog.skill_by_name(SWD_POZHOU_SEALED_KEY) {
                        for t in targets.into_iter().take(take) {
                            if !ctx.core.player(t).alive || ctx.core.has_skill(t, sdef) {
                                continue;
                            }
                            let keys: Vec<(PlayerId, crate::ids::SkillDefId)> = ctx
                                .core
                                .skills
                                .iter()
                                .map(|((p, d), _)| (*p, *d))
                                .filter(|(p, d)| {
                                    *p == t
                                        && *d != sdef
                                        && !ctx.catalog.is_charlotte_skill(*d)
                                        && !crate::catalog::Catalog::is_locked_skill_name(
                                            ctx.catalog.skills[*d as usize].name,
                                            ctx.catalog.skills[*d as usize].trigger.as_ref(),
                                        )
                                })
                                .collect();
                            for k in keys {
                                if let Some(inst) = ctx.core.skills.get_mut(&k) {
                                    inst.enabled = false;
                                }
                            }
                            ctx.insert_skill(
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
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 孤行：将全部手牌当杀使用（1–3 目标），结算后摸伤害数
            HostBehavior::SwdGuxing => match self.stage {
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            *p != self.owner
                                && ctx.core.computed_distance(ctx.catalog, self.owner, *p)
                                    <= ctx.core.attack_range(ctx.catalog, self.owner)
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
                            prompt: "guxing_targets",
                            options,
                            min: 1,
                            max: 3,
                            allow_cancel: true,
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
                    let hand: Vec<crate::ids::CardId> = ctx.core.player(self.owner).hand.clone();
                    for c in hand {
                        ctx.discard_from_hand(self.owner, c);
                    }
                    ctx.core.turn.guxing_armed = true;
                    ctx.core.turn.guxing_count = 0;
                    ctx.use_virtual_card(self.owner, CardKind::Sha, targets);
                    self.stage = HostStage::SelectCard;
                    Flow::Yield
                }
                HostStage::SelectCard => {
                    let n = ctx.core.turn.guxing_count;
                    if n > 0 {
                        ctx.start_draw(self.owner, n as u32);
                    }
                    ctx.core.turn.guxing_count = 0;
                    ctx.core.turn.guxing_armed = false;
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 锦鳞：选任意名角色各 +3 护甲并授予「锦鳞·退」（计数 3）
            HostBehavior::SwdJinlin => match self.stage {
                HostStage::Act => {
                    let options: Vec<ChoiceOption> =
                        ctx.core.alive_players().map(ChoiceOption::Player).collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    let max = options.len();
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "jinlin_targets",
                            options,
                            min: 1,
                            max,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let picked: Vec<PlayerId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Player(p) => Some(*p),
                            _ => None,
                        })
                        .collect();
                    if picked.is_empty() {
                        return Flow::Done;
                    }
                    let Some(def) = ctx.catalog.skill_by_name(SWD_JINLIN2_KEY) else {
                        return Flow::Done;
                    };
                    for t in &picked {
                        if !ctx.core.player(*t).alive {
                            continue;
                        }
                        ctx.change_hujia(*t, 3);
                        ctx.insert_skill(
                            (*t, def),
                            crate::state::SkillInstance {
                                owner: *t,
                                enabled: true,
                                once_used: false,
                                expires_end_of_turn: false,
                                rounds: 3,
                            },
                        );
                    }
                    ctx.core.player_mut(self.owner).jinlin_used = true;
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::SwdJikong => match self.stage {
                HostStage::Act => {
                    let range = ctx.core.attack_range(ctx.catalog, self.owner);
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            *p != self.owner
                                && ctx.core.computed_distance(ctx.catalog, self.owner, *p) <= range
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
                            prompt: "jikong_target",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(ChoiceOption::Player(target)) = ctx.answer().first().copied() else {
                        // 取消：不消耗每回合一次（上游 counttrigger-- 同口径
                        // ——原生可选门在取消时本就不消费）
                        return Flow::Done;
                    };
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "集控",
                        owner: self.owner, presentation: None });
                    if let Some(def) = ctx.catalog.skill_by_name(SWD_JIKONG_KEY) {
                        if let Some(inst) = ctx.core.skills.get_mut(&(self.owner, def)) {
                            inst.once_used = true;
                        }
                    }
                    ctx.use_virtual_card(
                        self.owner,
                        crate::catalog::CardKind::LeiSha,
                        vec![target],
                    );
                    Flow::Done
                }
                _ => Flow::Done,
            },
            _ => unreachable!("behavior routed to the wrong step module"),
        }
    }
}
