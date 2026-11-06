use super::*;

/// 判定阶段机器：对判定区延时锦囊逐张判定。
/// 排序对齐旧线 `pending_delayed_cards`（ADR 0711）：非闪电延时锦囊按
/// 放置序在前、闪电在后，**队首先消费**（先放入先判）。
///
/// 单次判定的流程（N08 首组官方技能，ADR 0712）：
/// 1. 翻判定牌；
/// 2. **改判询问**（张角「鬼道」）：持有者自判定阶段拥有者起按座位顺序
///    （含本人）逐人询问一次，可打出一张**黑色手牌**代替当前判定牌
///    （旧线含装备区牌，原生本批仅手牌，差异登记）；被替换的判定牌弃置；
/// 3. 判定牌生效（乐/兵/闪电）；
/// 4. **天妒询问**（郭嘉「天妒」）：持有者按座位顺序逐人询问，
///    可将最终判定牌收入手牌（一人获得后不再询问）。
///
/// 拥有者判定中途死亡：队列与当前判定牌逐张弃置（防御路径）。
pub struct JudgePhaseMachine {
    pub(super) owner: PlayerId,
    /// 待判锦囊队列（已排序：非闪电放置序在前、闪电在后；队首先消费）。
    pub(super) queue: Vec<crate::ids::CardId>,
    /// 当前判定的进行态（翻牌后到生效/获得前）。
    pub(super) current: Option<JudgementProgress>,
    pub(crate) ordinary: Option<EventId>,
    pub(crate) started: bool,
    pub(crate) payment: Option<crate::snap::JudgeReplacementSnap>,
}

/// 单次判定的进行态。
#[derive(Clone)]
pub struct JudgementProgress {
    pub(super) trick: Option<crate::ids::CardId>,
    pub(super) judge_card: crate::ids::CardId,
    /// 鬼道改判询问队列（座位序，自拥有者起含本人）。
    pub(super) rewriters: Vec<PlayerId>,
    pub(super) rewrite_provider: Option<crate::ids::SkillDefId>,
    pub(super) used_rewrite_providers: Vec<(PlayerId, crate::ids::SkillDefId)>,
    /// 当前改判者是否已被询问（等待应答）。
    pub(super) asked: bool,
    /// 天妒持有人队列（座位序）。
    pub(super) tian_du: Vec<PlayerId>,
    /// 判定牌是否已被天妒获得。
    pub(super) gained: bool,
    /// 是否处于改判询问阶段（false = 天妒阶段）。
    pub(super) in_rewrite: bool,
    /// Finalized 是否已记（恰好一次，效果生效前，N08-d）。
    pub(super) finalized: bool,
    /// 判定牌是否仍由机器持有（未弃置、未进入手牌）。
    pub(super) judge_held: bool,
    /// 昊天塔（swd 卡牌面窗口机批）：插窗持有者（Some = 等待其从两张
    /// 候选强制选一为判定结果；改判/天妒窗随插窗跳过）。
    pub(super) haotian_holder: Option<PlayerId>,
    /// 昊天塔：未被选中的候选牌（选中另一张后弃置）。
    pub(super) haotian_other: Option<crate::ids::CardId>,
    /// 改命（T11 Boss 诸葛亮族批 116）：亮出的牌堆顶 7 张候选（Some =
    /// 等待拥有者选 1 作为判定结果；其余弃置，改判/天妒窗跳过）。
    pub(super) gaiming_cards: Option<Vec<crate::ids::CardId>>,
}

impl Clone for JudgePhaseMachine {
    fn clone(&self) -> Self {
        Self {
            owner: self.owner,
            queue: self.queue.clone(),
            current: self.current.clone(),
            ordinary: self.ordinary,
            started: self.started,
            payment: self.payment.clone(),
        }
    }
}

impl JudgePhaseMachine {

    /// 正常判定初始化（规则 332 起为共享段）：抽判定牌、Judged 日志、
    /// 昊天塔候选窗、鬼道/鬼才/天道/改天道/天伦改判队列与本人天妒资格。
    /// 返回 `Some(Flow)`（昊天塔开窗）时调用方直接返回；`None` 表示已设置
    /// 正常进行态、调用方继续本机循环（与提取前的落空语义逐字相同）。
    /// 改命取消分支在候选归还牌堆后复用本段，不重复询问改命。
    fn begin_judgement(&mut self, ctx: &mut crate::machine::Ctx<'_>, trick: Option<crate::ids::CardId>) -> Option<Flow> {
        let judge_card = ctx.draw_judge_card();
        ctx.log(LogEvent::Judged {
            player: self.owner,
            card: judge_card,
        });
            // 昊天塔（swd 卡牌面窗口机批，equip5）：任意角色判定前，持有
            // 者观看牌堆顶两张、强制选一张直接作为判定结果（锁定技无取
            // 消；上游 trigger.cancel 手动定结果=改判/天妒窗一并绕过，
            // 原生跳过改判/天妒队列登记差异同口径；次候选弃置）。
            let haotian_holder = ctx.core.seat_order_from(self.owner).into_iter().find(|p| {
                ctx.core.player(*p).alive
                    && ctx
                        .core
                        .player(*p)
                        .equips
                        .iter()
                        .any(|c| ctx.catalog.card_kind(*c) == CardKind::Haotianta)
            });
            if let Some(h) = haotian_holder {
                // 次候选=牌堆顶第二张（draw_judge_card 已取顶张；堆空则
                // 洗弃堆，堆弃双空则不开窗）
                if !ctx.core.deck.is_empty() || !ctx.core.discard.is_empty() {
                    if ctx.core.deck.is_empty() {
                        let mut cards = std::mem::take(&mut ctx.core.discard);
                        ctx.core.rng.shuffle(&mut cards);
                        ctx.core.deck = cards;
                        ctx.log(LogEvent::DeckReshuffled);
                    }
                    let other = ctx
                        .core
                        .deck
                        .pop()
                        .expect("judge candidate: deck or discard non-empty");
                    self.current = Some(JudgementProgress {
                        trick,
                        judge_card,
                        rewriters: Vec::new(),
                        rewrite_provider: None,
                        used_rewrite_providers: Vec::new(),
                        asked: false,
                        tian_du: Vec::new(),
                        gained: false,
                        in_rewrite: false,
                        finalized: false,
                        judge_held: true,
                        haotian_holder: Some(h),
                        haotian_other: Some(other),
                        gaiming_cards: None,
                    });
                    return Some(Flow::WaitChoice {
                        observer: h,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "haotianta_pick",
                            options: vec![
                                ChoiceOption::Card(judge_card),
                                ChoiceOption::Card(other),
                            ],
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    });
                }
            }
            let gui_dao = ctx.catalog.skill_by_name(crate::vocabulary::names::GUI_DAO);
            let gui_cai = ctx.catalog.skill_by_name(crate::vocabulary::names::GUI_CAI);
            let tian_du = ctx.catalog.skill_by_name(crate::vocabulary::names::TIAN_DU);
            // 天妒（owner_is_target，对齐旧线内容定义/标准包文本「你的判定牌」）：
            // 仅判定阶段拥有者本人持有时由本人询问。
            let tian_du_self = tian_du
                .map(|d| ctx.core.has_skill(self.owner, d))
                .unwrap_or(false);
            // 改判者：自判定阶段拥有者起按座位序（含本人），逐人一次——
            // 鬼道（黑色牌改判）或鬼才（任意手牌改判）持有者均入队（对齐旧线
            // judgment_rewriters(tx, owner)；同座位双技能只入队一次）。
            // 天道（Boss 张角，规则 v263）：hes 任意牌可改判——改判资格
            // 与鬼道/鬼才并列（座位序同队）
            let tian_dao = ctx
                .catalog
                .skill_by_name(crate::vocabulary::names::BOSS_TIANDAO);
            // 改天道（swd 孤月）：hes 任意牌改判——与天道/鬼道/鬼才并列入队
            let g_tiandao = ctx.catalog.skill_by_name("gtiandao");
            let rewriters: Vec<PlayerId> = ctx
                .core
                .seat_order_from(self.owner)
                .into_iter()
                .filter(|p| {
                    let has = |d: Option<crate::ids::SkillDefId>| {
                        d.map(|d| ctx.core.has_skill(*p, d)).unwrap_or(false)
                    };
                    has(gui_dao)
                        || has(gui_cai)
                        || has(tian_dao)
                        || has(g_tiandao)
                        || has(ctx
                            .catalog
                            .skill_by_name(crate::vocabulary::host::SWD_TIANLUN_KEY))
                })
                .collect();
            self.current = Some(JudgementProgress {
                trick,
                judge_card,
                rewriters,
                rewrite_provider: None,
                used_rewrite_providers: Vec::new(),
                asked: false,
                tian_du: if tian_du_self {
                    vec![self.owner]
                } else {
                    Vec::new()
                },
                gained: false,
                in_rewrite: true,
                finalized: false,
                judge_held: true,
                haotian_holder: None,
                haotian_other: None,
                gaiming_cards: None,
            });
        None
    }
    pub(crate) fn new(
        owner: PlayerId,
        queue: Vec<crate::ids::CardId>,
        current: Option<JudgementProgress>,
    ) -> Self {
        Self {
            owner,
            queue,
            current,
            ordinary: None,
            started: false,
            payment: None,
        }
    }
}

impl JudgementProgress {
    pub(crate) fn from_snap(s: &crate::snap::JudgementProgressSnap) -> Self {
        Self {
            trick: s.trick,
            judge_card: s.judge_card,
            rewriters: s.rewriters.clone(),
            rewrite_provider: s.rewrite_provider,
            used_rewrite_providers: s.used_rewrite_providers.clone(),
            asked: s.asked,
            tian_du: s.tian_du.clone(),
            gained: s.gained,
            in_rewrite: s.in_rewrite,
            finalized: s.finalized,
            judge_held: s.judge_held,
            haotian_holder: s.haotian_holder,
            haotian_other: s.haotian_other.map(|c| c as crate::ids::CardId),
            gaiming_cards: s
                .gaiming_cards
                .as_ref()
                .map(|v| v.iter().map(|c| *c as crate::ids::CardId).collect()),
        }
    }
}

impl crate::catalog::SkillMachine for JudgePhaseMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }

    fn snapshot(&self) -> crate::snap::MachineSnapshot {
        let current = self.current.as_ref().map(|c| {
            Box::new(crate::snap::JudgementProgressSnap {
                trick: c.trick,
                judge_card: c.judge_card,
                rewriters: c.rewriters.clone(),
                rewrite_provider: c.rewrite_provider,
                used_rewrite_providers: c.used_rewrite_providers.clone(),
                asked: c.asked,
                tian_du: c.tian_du.clone(),
                gained: c.gained,
                in_rewrite: c.in_rewrite,
                finalized: c.finalized,
                judge_held: c.judge_held,
                haotian_holder: c.haotian_holder,
                haotian_other: c.haotian_other.map(|x| x as u32),
                gaiming_cards: c
                    .gaiming_cards
                    .as_ref()
                    .map(|v| v.iter().map(|x| *x as u32).collect()),
            })
        });
        if self.ordinary.is_some() || self.payment.is_some() || self.started {
            crate::snap::MachineSnapshot::JudgeContinuation {
                owner: self.owner,
                queue: self.queue.clone(),
                current,
                ordinary: self.ordinary,
                started: self.started,
                payment: self.payment.clone(),
            }
        } else {
            crate::snap::MachineSnapshot::JudgePhase {
                owner: self.owner,
                queue: self.queue.clone(),
                current,
            }
        }
    }

    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        loop {
            // 拥有者死亡（防御路径：对齐排序下能致死的闪电在队尾）：
            // 待判队列与进行中的判定牌逐张弃置。
            if !ctx.core.player(self.owner).alive {
                let mut leftover = Vec::new();
                crate::snap::machine_held_cards(&self.snapshot(), &mut leftover);
                self.queue.clear();
                self.current = None;
                self.payment = None;
                if !leftover.is_empty() {
                    ctx.core.discard.extend(leftover.iter().copied());
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: std::mem::take(&mut leftover),
                    });
                }
                return Flow::Done;
            }

            if let Some(payment) = self.payment.take() {
                if payment.discard_old || !ctx.core.player(payment.player).alive {
                    ctx.to_discard(payment.old);
                } else {
                    ctx.core.player_mut(payment.player).hand.push(payment.old);
                    ctx.emit(EventData::CardsMoved {
                        cards: vec![payment.old],
                        from: self.owner,
                        to: payment.player,
                    });
                }
                if let Some(cur) = &self.current {
                    ctx.log(LogEvent::JudgementRewritten {
                        player: payment.player,
                        card: cur.judge_card,
                        replaced: payment.old,
                    });
                }
                return Flow::Yield;
            }
            // 当前判定的进行态
            if let Some(cur) = &mut self.current {
                // 改命应答消费：选中牌成为判定结果、其余弃置并跳过改判/
                // 天妒窗；放弃则 7 张按原序回牌堆顶、照常判定（登记差异：
                // 上游取消后 7 张滞留处理区，原生归还牌堆）。
                if cur.gaiming_cards.is_some() {
                    // 改命应答消费：选中牌成为判定结果、其余弃置并跳过改判/
                    // 天妒窗；放弃则 7 张按原序回牌堆顶、直接复用正常判定
                    // 初始化（规则 332，裁决 24；登记差异：上游取消后 7 张
                    // 滞留处理区，原生归还牌堆）。取消不得保留指向已归还
                    // 候选的机器持有判定牌（旧实现同一实体既在牌堆又被
                    // 结算弃置，物理多重集 +1 且恢复拒绝）。
                    let mut cur = self.current.take().expect("gaiming in current");
                    let cards = cur.gaiming_cards.take().expect("checked");
                    match ctx.answer().first().copied() {
                        Some(ChoiceOption::Card(c)) if cards.contains(&c) => {
                            cur.judge_card = c;
                            for other in cards.iter().copied() {
                                if other != c {
                                    ctx.to_discard(other);
                                }
                            }
                            cur.rewriters.clear();
                            cur.in_rewrite = false;
                            cur.tian_du.clear();
                            ctx.log(LogEvent::Judged {
                                player: self.owner,
                                card: c,
                            });
                            self.current = Some(cur);
                        }
                        _ => {
                            for c in cards.into_iter().rev() {
                                ctx.core.deck.push(c);
                            }
                            if let Some(flow) = self.begin_judgement(ctx, cur.trick) {
                                return flow;
                            }
                        }
                    }
                    continue;
                }
                // 昊天塔（swd 卡牌面窗口机批）：插窗应答消费——强制选一
                // 张候选为判定结果，未选中的弃置。锁定技无取消（上游
                // chooseCardButton(true) 同口径）；登记差异：多持有者仅
                // 座序首位开窗（上游逐人触发 chaos 未定义时序）。
                if cur.haotian_holder.is_some() && cur.haotian_other.is_some() {
                    let holder = cur.haotian_holder.take().expect("checked");
                    let other = cur.haotian_other.take().expect("checked");
                    let picked = match ctx.answer().first().copied() {
                        Some(ChoiceOption::Card(c)) => Some(c),
                        _ => None,
                    };
                    match picked {
                        // 选中另一候选：判定牌换牌，原判定牌弃置
                        Some(c) if c != cur.judge_card => {
                            let old = cur.judge_card;
                            cur.judge_card = c;
                            ctx.to_discard(old);
                            ctx.log(LogEvent::JudgementRewritten {
                                player: holder,
                                card: c,
                                replaced: old,
                            });
                        }
                        // 选中原判定牌：候选弃置
                        Some(_) => ctx.to_discard(other),
                        // 防御（强制选一不应出现）：候选弃置
                        None => ctx.to_discard(other),
                    }
                    continue;
                }
                if cur.in_rewrite {
                    // 应答消费：当前改判者出黑色手牌替换或放弃
                    if cur.asked {
                        cur.asked = false;
                        // 运行时防御：改判队列已空（正常不会出现——asked 与
                        // 非空队列同置）则跳过改判直接进入后续阶段（N04-d）。
                        if cur.rewriters.is_empty() {
                            cur.in_rewrite = false;
                            continue;
                        }
                        let rewriter = cur.rewriters[0];
                        let answer = ctx.answer().clone();
                        if cur.rewrite_provider.is_none() {
                            if let Some(ChoiceOption::Skill(def)) = answer.first().copied() {
                                if judge_rewrite_providers(
                                    ctx.core,
                                    ctx.catalog,
                                    rewriter,
                                    &cur.used_rewrite_providers,
                                )
                                .contains(&def)
                                {
                                    cur.rewrite_provider = Some(def);
                                    continue;
                                }
                            }
                            cur.rewriters.remove(0);
                            continue;
                        }
                        let provider = cur.rewrite_provider.take().expect("selected provider");
                        cur.used_rewrite_providers.push((rewriter, provider));
                        // 天伦（swd 批 138）：从判定区取牌替换判定牌，旧判定
                        // 牌**弃置**（上游 `judging[0].discard()` 同口径——与
                        // 鬼道/鬼才「旧牌归改判者」不同）
                        let tianlun = ctx.catalog.skill_name(provider)
                            == crate::vocabulary::host::SWD_TIANLUN_KEY;
                        if tianlun {
                            if let Some(ChoiceOption::Card(c)) = answer.first().copied() {
                                if c != cur.judge_card {
                                    let mut moved = false;
                                    for p in 0..ctx.core.players.len() as PlayerId {
                                        if ctx.core.player(p).judge.contains(&c) {
                                            ctx.core.player_mut(p).judge.retain(|x| *x != c);
                                            moved = true;
                                            break;
                                        }
                                    }
                                    if !moved && self.queue.contains(&c) {
                                        self.queue.retain(|x| *x != c);
                                        moved = true;
                                    }
                                    if !moved {
                                        // 非判定区牌（异常路径）：忽略
                                        if cur.rewriters.is_empty() {
                                            cur.in_rewrite = false;
                                        }
                                        continue;
                                    }
                                    let old = cur.judge_card;
                                    ctx.to_discard(old);
                                    cur.judge_card = c;
                                    ctx.log(LogEvent::JudgementRewritten {
                                        player: rewriter,
                                        card: c,
                                        replaced: old,
                                    });
                                }
                            }
                            if cur.rewriters.is_empty() {
                                cur.in_rewrite = false;
                            }
                            continue;
                        }
                        if let Some(ChoiceOption::Card(c)) = answer.first().copied() {
                            // 打出的牌离开手牌或装备区、由判定流程持有（成为新判定牌）；
                            // 旧判定牌按已支付的改判能力弃置或获得，跨失牌子树持有。
                            ctx.take_owned_card(rewriter, c);
                            let old = cur.judge_card;
                            cur.judge_card = c;
                            let discard_old = ctx.catalog.skill_name(provider)
                                == crate::vocabulary::names::GUI_CAI;
                            self.payment = Some(crate::snap::JudgeReplacementSnap {
                                player: rewriter,
                                old,
                                discard_old,
                                provider,
                            });
                            if cur.rewriters.is_empty() {
                                cur.in_rewrite = false;
                            }
                            return Flow::Yield;
                        }
                        if cur.rewriters.is_empty() {
                            cur.in_rewrite = false;
                        }
                        continue;
                    }
                    // 询问下一个可用改判者（跳过死亡/无黑色牌者——手牌与
                    // 装备区均可，N08-c：仅装备区黑牌者也保持询问资格）
                    while let Some(r) = cur.rewriters.first().copied() {
                        if !ctx.core.player(r).alive
                            || judge_rewrite_providers(
                                ctx.core,
                                ctx.catalog,
                                r,
                                &cur.used_rewrite_providers,
                            )
                            .is_empty()
                        {
                            cur.rewriters.remove(0);
                            cur.rewrite_provider = None;
                            continue;
                        }
                        break;
                    }
                    if cur.rewriters.is_empty() {
                        cur.in_rewrite = false;
                        continue;
                    }
                    let rewriter = cur.rewriters[0];
                    let providers = judge_rewrite_providers(
                        ctx.core,
                        ctx.catalog,
                        rewriter,
                        &cur.used_rewrite_providers,
                    );
                    if cur.rewrite_provider.is_none() {
                        if providers.len() > 1 {
                            cur.asked = true;
                            return Flow::WaitChoice {
                                observer: rewriter,
                                spec: ChoiceSpec {
                                    kind: ChoiceKind::RespondCard,
                                    prompt: "judge_rewrite_provider",
                                    options: std::iter::once(ChoiceOption::Pass)
                                        .chain(providers.into_iter().map(ChoiceOption::Skill))
                                        .collect(),
                                    min: 1,
                                    max: 1,
                                    allow_cancel: false,
                                },
                            };
                        }
                        cur.rewrite_provider = providers.first().copied();
                    }
                    cur.asked = true;
                    // 天伦（swd 批 138，规则 v316 同批）：候选=本次判定牌 +
                    // 全场判定区牌（上游 tianlun 的 dialog list 同口径）
                    if cur.rewrite_provider.is_some_and(|d| {
                        ctx.catalog.skill_name(d) == crate::vocabulary::host::SWD_TIANLUN_KEY
                    }) {
                        let mut options = vec![ChoiceOption::Pass];
                        if cur.judge_held {
                            options.push(ChoiceOption::Card(cur.judge_card));
                        }
                        for p in 0..ctx.core.players.len() as PlayerId {
                            for c in ctx.core.player(p).judge.iter().copied() {
                                if !options.contains(&ChoiceOption::Card(c)) {
                                    options.push(ChoiceOption::Card(c));
                                }
                            }
                        }
                        // 判定机待结算队列（原生在判定阶段开始即把判定区
                        // 整体取入队列，见 JudgePhaseMachine::new——同口径
                        // 视为「判定区牌」）
                        for c in self.queue.iter().copied() {
                            // 正在结算的延时锦囊已离开判定区（上游
                            // `judging[0]` 之外的判定区牌才可被选）
                            if Some(c) == cur.trick {
                                continue;
                            }
                            if !options.contains(&ChoiceOption::Card(c)) {
                                options.push(ChoiceOption::Card(c));
                            }
                        }
                        if options.len() == 1 {
                            cur.rewriters.remove(0);
                            continue;
                        }
                        return Flow::WaitChoice {
                            observer: rewriter,
                            spec: ChoiceSpec {
                                kind: ChoiceKind::RespondCard,
                                prompt: "tianlun_rewrite",
                                options,
                                min: 1,
                                max: 1,
                                allow_cancel: false,
                            },
                        };
                    }
                    let mut options = vec![ChoiceOption::Pass];
                    options.extend(
                        judge_provider_cards(
                            ctx.core,
                            ctx.catalog,
                            rewriter,
                            cur.rewrite_provider.expect("provider selected"),
                        )
                        .into_iter()
                        .map(ChoiceOption::Card),
                    );
                    return Flow::WaitChoice {
                        observer: rewriter,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::RespondCard,
                            prompt: "gui_dao_rewrite",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    };
                }
                // 判定牌确定（N08-d 顺序：Rewritten×N → Finalized → 效果 →
                // 天妒 Gained）：Finalized 恰好记一次、在效果与天妒询问前。
                if !cur.finalized {
                    cur.finalized = true;
                    let trick = cur.trick;
                    let judge_card = cur.judge_card;
                    // 天妒待询问时判定牌由机器持有，不在此弃置；
                    // 否则随效果弃置（judge_held 置否）。
                    if cur.gained || cur.tian_du.is_empty() {
                        cur.judge_held = false;
                    }
                    let machine_holds = cur.judge_held;
                    ctx.log(LogEvent::JudgementFinalized {
                        player: self.owner,
                        card: judge_card,
                    });
                    if let Some(trick) = trick {
                        Self::apply_judgement(self.owner, ctx, trick, judge_card, machine_holds);
                    } else {
                        if !machine_holds {
                            ctx.to_discard(judge_card);
                        }
                        if let Some(invocation) = self.ordinary {
                            if let Some(frame) = ctx.res.frame_mut(invocation) {
                                if let EventData::SkillInvocation(data) = &mut frame.data {
                                    data.judgement =
                                        Some(ctx.catalog.cards[judge_card as usize].suit);
                                }
                            }
                        }
                    }
                    let resolved = EventData::OrdinaryJudge {
                        owner: self.owner,
                        invocation: judge_card,
                    };
                    let qctx = ctx.query();
                    let has_vm_listener = ctx.core.skills.iter().any(|((owner, sdef), inst)| {
                        let def_entry = &ctx.catalog.skills[*sdef as usize];
                        inst.enabled
                            && ctx.core.player(*owner).alive
                            && def_entry.behavior.is_vm_skill()
                            && def_entry.trigger.as_ref().is_some_and(|m| {
                                m.event == crate::event::EventKind::OrdinaryJudge
                                    || m.extra_events
                                        .contains(&crate::event::EventKind::OrdinaryJudge)
                            })
                            && def_entry.behavior.filter_in_window(
                                &qctx,
                                *owner,
                                &resolved,
                                crate::event::EventKind::OrdinaryJudge,
                            )
                    });
                    if has_vm_listener {
                        ctx.emit(resolved);
                    }
                    return Flow::Yield;
                }
                // 天妒阶段（效果生效后）：跳过死亡持有者；有人获得后不再询问
                while let Some(h) = cur.tian_du.first().copied() {
                    if !ctx.core.player(h).alive || cur.gained {
                        cur.tian_du.remove(0);
                    } else {
                        break;
                    }
                }
                // 天妒应答消费
                if !cur.tian_du.is_empty() && cur.asked && !cur.gained && cur.judge_held {
                    cur.asked = false;
                    cur.judge_held = false;
                    let holder = cur.tian_du.remove(0);
                    if let ChoiceOption::Confirm =
                        ctx.answer().first().copied().unwrap_or(ChoiceOption::Pass)
                    {
                        let card = cur.judge_card;
                        ctx.core.player_mut(holder).hand.push(card);
                        cur.gained = true;
                        ctx.log(LogEvent::JudgementGained {
                            player: holder,
                            card,
                        });
                    } else {
                        // 放弃获得：机器持有的判定牌弃置
                        ctx.to_discard(cur.judge_card);
                    }
                    self.current = None;
                    continue;
                }
                if cur.tian_du.is_empty() || cur.gained || !cur.judge_held {
                    // 完结：机器仍持有的判定牌（天妒者死亡等防御路径）弃置
                    if cur.judge_held && !cur.gained {
                        cur.judge_held = false;
                        ctx.to_discard(cur.judge_card);
                    }
                    self.current = None;
                    continue;
                }
                // 询问天妒持有者
                let holder = cur.tian_du[0];
                cur.asked = true;
                return Flow::WaitChoice {
                    observer: holder,
                    spec: ChoiceSpec::confirm("tian_du_gain"),
                };
            }

            // 取下一锦囊并翻判定牌
            let trick = if self.ordinary.is_some() {
                if self.started {
                    return Flow::Done;
                }
                self.started = true;
                if ctx.core.deck.is_empty() && ctx.core.discard.is_empty() {
                    return Flow::Done;
                }
                None
            } else {
                if self.queue.is_empty() {
                    return Flow::Done;
                }
                Some(self.queue.remove(0))
            };
            // 改命（T11 Boss 诸葛亮族批 116，规则 v303 同批）：判定前——拥有
            // 者持技时亮牌堆顶 7 张，由其选 1 作为判定结果（其余弃置、改判/
            // 天妒窗跳过；上游 judgeBefore chooseCardButton + trigger.cancel
            // + trigger.direct 同口径）。
            let has_gaiming = ctx
                .catalog
                .skill_by_name(crate::vocabulary::host::HLSS_GAIMING_KEY)
                .map(|def| ctx.core.has_skill(self.owner, def))
                .unwrap_or(false);
            if has_gaiming {
                let mut cards: Vec<crate::ids::CardId> = Vec::new();
                for _ in 0..7 {
                    if ctx.core.deck.is_empty() {
                        if ctx.core.discard.is_empty() {
                            break;
                        }
                        let mut d = std::mem::take(&mut ctx.core.discard);
                        ctx.core.rng.shuffle(&mut d);
                        ctx.core.deck = d;
                        ctx.log(LogEvent::DeckReshuffled);
                    }
                    cards.push(ctx.core.deck.pop().expect("deck non-empty"));
                }
                if !cards.is_empty() {
                    self.current = Some(JudgementProgress {
                        trick,
                        judge_card: cards[0],
                        rewriters: Vec::new(),
                        rewrite_provider: None,
                        used_rewrite_providers: Vec::new(),
                        asked: false,
                        tian_du: Vec::new(),
                        gained: false,
                        in_rewrite: false,
                        finalized: false,
                        judge_held: true,
                        haotian_holder: None,
                        haotian_other: None,
                        gaiming_cards: Some(cards.clone()),
                    });
                    return Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "gaiming_pick",
                            options: cards.into_iter().map(ChoiceOption::Card).collect(),
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    };
                }
            }
            if let Some(flow) = self.begin_judgement(ctx, trick) {
                return flow;
            }
            continue;
        }
    }

}

impl JudgePhaseMachine {
    /// 判定牌生效：按锦囊种类套用效果（`machine_holds` 为真时判定牌仍由
    /// 机器持有——天妒未决——不随效果弃置）。
    fn apply_judgement(
        owner: PlayerId,
        ctx: &mut Ctx<'_>,
        trick: crate::ids::CardId,
        judge_card: crate::ids::CardId,
        machine_holds: bool,
    ) {
        // 国色等 view-as 转化：声明牌种优先于目录牌种（N08 迁移）
        let trick_kind = ctx
            .core
            .declared_kinds
            .get(&trick)
            .copied()
            .unwrap_or_else(|| ctx.catalog.card_kind(trick));
        // 声明消费后移除（牌已结算，残留条目会在后续洗回场景误判）
        ctx.core.declared_kinds.remove(&trick);
        let (suit, rank) = {
            let def = &ctx.catalog.cards[judge_card as usize];
            (def.suit, def.rank)
        };
        // 神裔（Boss 青龙共有技，规则 v233）：判定阶段的延时锦囊判定对
        // 持有者恒不生效（上游 mod judge event.type=="phase"：bool
        // true→false、false→null——各臂效果条件一并短路；牌去向不变：
        // 锦囊照常弃置、闪电照常传下家）
        let shenyi_voids = ctx
            .catalog
            .skill_by_name(crate::vocabulary::names::BOSS_SHENYI)
            .map(|def| ctx.core.has_skill(owner, def))
            .unwrap_or(false)
            // 隐月·判（T11 Boss 诸葛亮族批 115，规则 v303）：上游
            // `hlss_yueyin2` mod judge 对阶段判定「true→false、false→null」
            // ——两臂皆不生效（牌去向不变），与神裔同口径。
            || ctx
                .catalog
                .skill_by_name(crate::vocabulary::host::HLSS_YUEYIN2_KEY)
                .map(|def| ctx.core.has_skill(owner, def))
                .unwrap_or(false);
        match trick_kind {
            CardKind::Le => {
                if !shenyi_voids && !suit.is_heart() {
                    ctx.core.turn.skip_play_phase = true;
                }
                ctx.to_discard(trick);
                if !machine_holds {
                    ctx.to_discard(judge_card);
                }
            }
            CardKind::Bing => {
                if !shenyi_voids && !suit.is_club() {
                    ctx.core.turn.skip_draw_phase = true;
                }
                ctx.to_discard(trick);
                if !machine_holds {
                    ctx.to_discard(judge_card);
                }
            }
            CardKind::HongShui => {
                // 洪水：梅花 2–9 → 3 点雷伤（登记差异：上游未标注属性）
                ctx.to_discard(trick);
                if !shenyi_voids && suit.is_club() && (2..=9).contains(&rank) {
                    if !machine_holds {
                        ctx.to_discard(judge_card);
                    }
                    ctx.emit(EventData::Damage(damage_data_with(
                        None,
                        owner,
                        3,
                        crate::event::DamageNature::Thunder,
                        None,
                    )));
                } else if !machine_holds {
                    ctx.to_discard(judge_card);
                }
            }
            CardKind::Guiyoujie => {
                // 归有界判定（上游 judge 黑色 -3 / effect loseHp+randomDiscard）
                if !shenyi_voids
                    && matches!(
                        suit,
                        crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                    )
                {
                    ctx.to_discard(trick);
                    ctx.emit(EventData::LoseHp {
                        player: owner,
                        amount: 1,
                    });
                    // 随机弃一张牌（手牌+装备均匀抽取；逐技对照批四口径）
                    let he_len =
                        ctx.core.player(owner).hand.len() + ctx.core.player(owner).equips.len();
                    if he_len > 0 {
                        let pick = ctx.core.rng.gen_range(he_len as u32) as usize;
                        let hand_n = ctx.core.player(owner).hand.len();
                        let c = if pick < hand_n {
                            ctx.core.player(owner).hand[pick]
                        } else {
                            ctx.core.player(owner).equips[pick - hand_n]
                        };
                        let in_hand = ctx.core.player(owner).hand.contains(&c);
                        if in_hand {
                            ctx.discard_from_hand(owner, c);
                        } else {
                            ctx.core.player_mut(owner).equips.retain(|x| *x != c);
                            crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, owner, c);
                            ctx.to_discard(c);
                        }
                        ctx.log(LogEvent::CardDiscarded {
                            player: owner,
                            cards: vec![c],
                        });
                    }
                }
                ctx.to_discard(trick);
                if !machine_holds {
                    ctx.to_discard(judge_card);
                }
            }
            CardKind::ShanDian => {
                if !shenyi_voids && suit.is_spade() && (2..=9).contains(&rank) {
                    ctx.to_discard(trick);
                    if !machine_holds {
                        ctx.to_discard(judge_card);
                    }
                    ctx.emit(EventData::Damage(damage_data_with(
                        None,
                        owner,
                        3,
                        crate::event::DamageNature::Thunder,
                        None,
                    )));
                    return;
                }
                // 未命中：传给下一位存活玩家；其判定区已有闪电则直接弃置
                // （对齐旧线 pass_lightning）。
                let next = ctx.core.next_alive(owner);
                let next_has_shandian = ctx
                    .core
                    .player(next)
                    .judge
                    .iter()
                    .any(|c| ctx.catalog.card_kind(*c) == CardKind::ShanDian);
                if next_has_shandian {
                    ctx.to_discard(trick);
                } else {
                    ctx.core.player_mut(next).judge.push(trick);
                }
                if !machine_holds {
                    ctx.to_discard(judge_card);
                }
            }
            _ => {
                ctx.to_discard(trick);
                if !machine_holds {
                    ctx.to_discard(judge_card);
                }
            }
        }
    }
}

/// Source GuiCai is hs outside guozhan, hes inside; GuiDao is black he, Tiandao is any he.
pub fn judge_rewrite_cards(
    core: &crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
    player: PlayerId,
) -> Vec<crate::ids::CardId> {
    let mut cards = Vec::new();
    for def in judge_rewrite_providers(core, catalog, player, &[]) {
        for card in judge_provider_cards(core, catalog, player, def) {
            if !cards.contains(&card) {
                cards.push(card);
            }
        }
    }
    cards
}
pub fn judge_rewrite_providers(
    core: &crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
    player: PlayerId,
    used: &[(PlayerId, crate::ids::SkillDefId)],
) -> Vec<crate::ids::SkillDefId> {
    [
        crate::vocabulary::names::GUI_CAI,
        crate::vocabulary::names::GUI_DAO,
        crate::vocabulary::names::BOSS_TIANDAO,
        "gtiandao",
        crate::vocabulary::host::SWD_TIANLUN_KEY,
    ]
    .into_iter()
    .filter_map(|name| catalog.skill_by_name(name))
    .filter(|d| {
        core.has_skill(player, *d)
            && !used.contains(&(player, *d))
            && (catalog.skill_name(*d) == crate::vocabulary::host::SWD_TIANLUN_KEY
                || !judge_provider_cards(core, catalog, player, *d).is_empty())
    })
    .collect()
}
pub fn judge_provider_cards(
    core: &crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
    player: PlayerId,
    provider: crate::ids::SkillDefId,
) -> Vec<crate::ids::CardId> {
    let name = catalog.skill_name(provider);
    let cai = name == crate::vocabulary::names::GUI_CAI;
    let dao = name == crate::vocabulary::names::GUI_DAO;
    let any = name == crate::vocabulary::names::BOSS_TIANDAO || name == "gtiandao";
    let black = |c: &crate::ids::CardId| {
        matches!(
            catalog.cards[*c as usize].suit,
            crate::catalog::Suit::Spade | crate::catalog::Suit::Club
        )
    };
    let p = core.player(player);
    p.hand
        .iter()
        .filter(|c| any || cai || dao && black(c))
        .chain(p.equips.iter().filter(|c| {
            any || cai && matches!(core.mode, crate::state::GameMode::Kingdoms) || dao && black(c)
        }))
        .copied()
        .collect()
}
