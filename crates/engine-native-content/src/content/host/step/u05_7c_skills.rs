//! U05 Batch 7c (`swd` 武将技前半) 核心流转机：`kbolan`/`kbolan2`、`touxi`/`touxi2`、`lmazui3`、
//! `tiebi`、`senluo`、`lingxian`、`guaili` 与公共丢弃/返还辅助。

use super::super::*;
use super::u08_batch_d::{flag, has_skill_named, set_flag};

/// 主动技取消提交前退回本次限一次标记，确保取消不白耗每回合次数。
pub(super) fn refund_once_used(ctx: &mut Ctx<'_>, owner: PlayerId, def_key: &str) {
    if let Some(def) = ctx.catalog.skill_by_name(def_key) {
        if let Some(inst) = ctx.core.skills.get_mut(&(owner, def)) {
            inst.once_used = false;
        }
    }
}

/// 弃置目标玩家手牌或装备区的指定实体牌，保持弃置日志、装备技撤销与 `CardsLost` 事件一致。
pub(super) fn discard_he_card(ctx: &mut Ctx<'_>, player: PlayerId, card: CardId) -> bool {
    if ctx.core.player(player).hand.contains(&card) {
        ctx.discard_from_hand(player, card);
        ctx.log(LogEvent::CardDiscarded {
            player,
            cards: vec![card],
        });
        return true;
    }
    if ctx.core.player(player).equips.contains(&card) {
        ctx.core.player_mut(player).equips.retain(|x| *x != card);
        ctx.core.declared_slots.remove(&card);
        ctx.core.player_mut(player).card_mods.remove(&card);
        if ctx.catalog.card_kind(card).is_equip() {
            crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, player, card);
        }
        ctx.to_discard(card);
        ctx.note_cards_lost(player, &[card]);
        ctx.emit(EventData::CardsLost {
            player,
            cards: vec![card],
            origins: vec![crate::event::LostOrigin::Equip],
        });
        ctx.log(LogEvent::CardDiscarded {
            player,
            cards: vec![card],
        });
        return true;
    }
    false
}

/// `Draw` 通用起止钩子：`kbolan`（`drawBegin` +1 并挂 `kbolan2`）与 `kbolan2`（`drawEnd` 选 1 手牌置牌堆顶）。
pub(crate) fn host_draw_event_step(
    ctx: &mut Ctx<'_>,
    player: PlayerId,
    frame_id: crate::ids::EventId,
    is_begin: bool,
    _requested: &mut u32,
) -> bool {
    if !ctx.core.player(player).alive {
        return false;
    }
    if is_begin {
        if let Some(kdef) = ctx.catalog.skill_by_name("kbolan") {
            if has_skill_named(ctx.core, ctx.catalog, player, "kbolan")
                && !ctx.catalog.is_tieji_sealed(ctx.core, player)
                && !ctx.catalog.is_rechanyuan_sealed_def(ctx.core, player, kdef)
                && flag(ctx.core, player, &format!("u05_kbolan_seen:{frame_id}")) == 0
            {
                set_flag(ctx.core, player, &format!("u05_kbolan_seen:{frame_id}"), 1);
                ctx.res.parks.push(crate::resolution::Park {
                    resume_depth: ctx.res.stack.len(),
                    slot: crate::resolution::Slot::Trigger { frame: frame_id },
                    machine: Box::new(HostMachine {
                        def_key: "kbolan".to_string(),
                        behavior: HostBehavior::SwdKbolan,
                        owner: player,
                        target: None,
                        card: None,
                        cards: Vec::new(),
                        targets: Vec::new(),
                        stage: HostStage::Act,
                    }),
                    awaiting_answer: false,
                });
                return true;
            }
        }
        return false;
    }
    let key = format!("u05_kbolan2_frame:{frame_id}");
    if ctx.core.vm_aux.flags.remove(&(player, key)).is_some()
        && has_skill_named(ctx.core, ctx.catalog, player, "kbolan2")
    {
        if let Some(def) = ctx.catalog.skill_by_name("kbolan2") {
            ctx.core.skills.remove(&(player, def));
        }
        if !ctx.core.player(player).hand.is_empty() {
            ctx.res.parks.push(crate::resolution::Park {
                resume_depth: ctx.res.stack.len(),
                slot: crate::resolution::Slot::Trigger { frame: frame_id },
                machine: Box::new(HostMachine {
                    def_key: "kbolan2".to_string(),
                    behavior: HostBehavior::SwdKbolan2,
                    owner: player,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::SelectCard,
                }),
                awaiting_answer: false,
            });
            return true;
        }
    }
    false
}

pub(super) fn tiebi_black_hand_options(
    core: &crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
    owner: PlayerId,
) -> Vec<ChoiceOption> {
    core.player(owner)
        .hand
        .iter()
        .copied()
        .filter(|c| {
            matches!(
                catalog.effective_card_suit(core, owner, *c),
                crate::catalog::Suit::Spade | crate::catalog::Suit::Club
            )
        })
        .map(ChoiceOption::Card)
        .collect()
}

pub(super) fn he_card_options(
    core: &crate::state::GameStateCore,
    player: PlayerId,
) -> Vec<ChoiceOption> {
    core.player(player)
        .hand
        .iter()
        .chain(core.player(player).equips.iter())
        .copied()
        .map(ChoiceOption::Card)
        .collect()
}

impl HostMachine {
    fn tiebi_next_or_done(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        while !self.targets.is_empty() {
            let t = self.targets.remove(0);
            if !ctx.core.player(t).alive
                || ctx.core.player(t).hujia != 0
                || ctx.core.computed_distance(ctx.catalog, self.owner, t) > 1
            {
                continue;
            }
            let options = tiebi_black_hand_options(ctx.core, ctx.catalog, self.owner);
            if options.is_empty() {
                return Flow::Done;
            }
            self.target = Some(t);
            self.stage = HostStage::SelectCard;
            return Flow::WaitChoice {
                observer: self.owner,
                spec: ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "tiebi_discard",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            };
        }
        Flow::Done
    }

    fn senluo_next_or_done(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        while !self.targets.is_empty() {
            let t = self.targets.remove(0);
            if !ctx.core.player(t).alive || ctx.core.player(t).hand.is_empty() {
                continue;
            }
            let hand: Vec<CardId> = ctx.core.player(t).hand.iter().copied().collect();
            if hand.len() > 2 {
                self.target = Some(t);
                self.stage = HostStage::SelectCard;
                return Flow::WaitChoice {
                    observer: t,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "senluo_discard",
                        options: hand.into_iter().map(ChoiceOption::Card).collect(),
                        min: 2,
                        max: 2,
                        allow_cancel: false,
                    },
                };
            }
            for c in &hand {
                ctx.discard_from_hand(t, *c);
            }
            if ctx.core.player(t).alive {
                ctx.start_draw(t, 1);
            }
            self.stage = HostStage::DrawSettle;
            return Flow::Yield;
        }
        Flow::Done
    }

    pub(super) fn step_u05_7c_skills(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            // 拷贝栏（swd/skill.js:672）：frequent:true 可选发动——确认后摸牌数 +1 并挂 kbolan2，拒绝则不加摸且无 kbolan2
            HostBehavior::SwdKbolan => match self.stage {
                HostStage::Act => {
                    self.stage = HostStage::CompareTargetPick;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec::confirm("kbolan_confirm"),
                    }
                }
                HostStage::CompareTargetPick => {
                    if !matches!(ctx.answer().first(), Some(ChoiceOption::Confirm)) {
                        return Flow::Done;
                    }
                    let frame_id = ctx.res.stack.iter().rev().find_map(|f| match &f.data {
                        EventData::Draw {
                            player,
                            held_cards,
                            terminal: crate::event::DrawTerminal::Running,
                            ..
                        } if *player == self.owner && held_cards.is_empty() => Some(f.id),
                        _ => None,
                    });
                    let Some(fid) = frame_id else {
                        return Flow::Done;
                    };
                    if let Some(frame) = ctx.res.frame_mut(fid) {
                        if let EventData::Draw { requested, .. } = &mut frame.data {
                            *requested = requested.saturating_add(1);
                        }
                    }
                    set_flag(ctx.core, self.owner, &format!("u05_kbolan2_frame:{fid}"), 1);
                    if let Some(def) = ctx.catalog.skill_by_name("kbolan2") {
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
                    if let Some(kdef) = ctx.catalog.skill_by_name("kbolan") {
                        ctx.log(LogEvent::SkillTriggered {
                            skill: ctx.catalog.skills[kdef as usize].name,
                            owner: self.owner,
                            presentation: None,
                        });
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 拷贝栏·置顶：将 1 张手牌放回牌堆顶（deck.push）并记录失牌历史与 CardsLost(Hand)
            HostBehavior::SwdKbolan2 => match self.stage {
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
                    self.stage = HostStage::Act;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "kbolan2_top",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::Act => {
                    if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                        if ctx.core.player(self.owner).hand.contains(&c) {
                            ctx.take_hand_card(self.owner, c);
                            ctx.core.deck.push(c);
                            ctx.note_cards_lost(self.owner, &[c]);
                            ctx.emit(EventData::CardsLost {
                                player: self.owner,
                                cards: vec![c],
                                origins: vec![crate::event::LostOrigin::Hand],
                            });
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 偷袭（swd/skill.js:3924）：他角色回合结束经 OrdinaryJudge 判定（鬼才/鬼道/改天命/天妒接入）：黑→1 雷伤+挂 touxi2；红→其可弃自己一张 he
            HostBehavior::SwdTouxi => match self.stage {
                HostStage::Act => {
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(self.owner).alive || !ctx.core.player(t).alive {
                        return Flow::Done;
                    }
                    self.stage = HostStage::DrawSettle;
                    ctx.emit(EventData::OrdinaryJudge {
                        owner: self.owner,
                        invocation: u32::MAX,
                    });
                    Flow::Yield
                }
                HostStage::DrawSettle => {
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    let code = ctx
                        .core
                        .vm_aux
                        .flags
                        .remove(&(self.owner, "last_judge_suit".to_string()))
                        .unwrap_or(0);
                    let _ = ctx
                        .core
                        .vm_aux
                        .flags
                        .remove(&(self.owner, "last_judge_rank".to_string()));
                    let judged = ctx
                        .core
                        .vm_aux
                        .flags
                        .remove(&(self.owner, "last_judge_card".to_string()));
                    let is_black = match code {
                        1 | 3 => true,
                        2 | 4 => false,
                        _ => judged.is_some_and(|c| {
                            matches!(
                                ctx.catalog.effective_card_suit(ctx.core, self.owner, c),
                                crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                            )
                        }),
                    };
                    if is_black {
                        if ctx.core.player(t).alive {
                            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                                Some(self.owner),
                                t,
                                1,
                                crate::event::DamageNature::Thunder,
                                None,
                            )));
                        }
                        if let Some(def) = ctx.catalog.skill_by_name("touxi2") {
                            ctx.insert_skill(
                                (self.owner, def),
                                crate::state::SkillInstance {
                                    owner: self.owner,
                                    enabled: true,
                                    once_used: false,
                                    expires_end_of_turn: false,
                                    rounds: 0,
                                },
                            );
                        }
                        return Flow::Done;
                    }
                    if !ctx.core.player(t).alive {
                        return Flow::Done;
                    }
                    let options = he_card_options(ctx.core, self.owner);
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: t,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "touxi_discard",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                        let _ = discard_he_card(ctx, self.owner, c);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 偷袭·冷却：自己回合开始移除自身
            HostBehavior::SwdTouxi2 => {
                if let Some(def) = ctx.catalog.skill_by_name("touxi2") {
                    ctx.core.skills.remove(&(self.owner, def));
                }
                Flow::Done
            }
            // 麻醉·回收：伤害结算后或归零时取回存储黑牌并移除自身
            HostBehavior::SwdLmazui3 => {
                let stored = ctx
                    .core
                    .vm_aux
                    .flags
                    .remove(&(self.owner, "swd_lmazui2_card".to_string()))
                    .and_then(|v| v.checked_sub(1))
                    .map(|id| id as CardId);
                if let Some(c) = stored {
                    if ctx.core.player(self.owner).off_court.contains(&c) {
                        ctx.core.player_mut(self.owner).off_court.retain(|x| *x != c);
                        if ctx.core.player(self.owner).alive {
                            ctx.core.player_mut(self.owner).hand.push(c);
                            ctx.log(LogEvent::CardsTaken {
                                from: self.owner,
                                to: self.owner,
                                cards: vec![c],
                                public: true,
                            });
                        } else {
                            ctx.to_discard(c);
                        }
                    }
                }
                if let Some(def) = ctx.catalog.skill_by_name("lmazui3") {
                    ctx.core.skills.remove(&(self.owner, def));
                }
                Flow::Done
            }
            // 铁壁：逐符合距离 ≤1 且无护甲之杀目标询问自选弃一张黑牌手牌 → +1 护甲
            HostBehavior::SwdTiebi => match self.stage {
                HostStage::Act => {
                    let targets = match ctx.res.frame(ctx.current_event_frame_id()).map(|f| &f.data) {
                        Some(EventData::CardUsed(u)) => u
                            .targets
                            .iter()
                            .copied()
                            .filter(|t| {
                                ctx.core.player(*t).alive
                                    && ctx.core.player(*t).hujia == 0
                                    && ctx.core.computed_distance(ctx.catalog, self.owner, *t) <= 1
                            })
                            .collect(),
                        _ => Vec::new(),
                    };
                    self.targets = targets;
                    self.tiebi_next_or_done(ctx)
                }
                HostStage::SelectCard => {
                    if let (Some(t), Some(ChoiceOption::Card(c))) =
                        (self.target, ctx.answer().first().copied())
                    {
                        if ctx.core.player(self.owner).hand.contains(&c) {
                            ctx.discard_from_hand(self.owner, c);
                            ctx.log(LogEvent::CardDiscarded {
                                player: self.owner,
                                cards: vec![c],
                            });
                            if ctx.core.player(t).alive {
                                ctx.change_hujia(t, 1);
                            }
                        }
                    }
                    self.tiebi_next_or_done(ctx)
                }
                _ => Flow::Done,
            },
            // 森罗（swd/skill.js:947）：其他有手牌者按座位序逐人弃后即摸 1（弃1→摸1→下家）
            HostBehavior::SwdSenluo => match self.stage {
                HostStage::Act => {
                    self.targets = ctx
                        .core
                        .seat_order_from(self.owner)
                        .into_iter()
                        .filter(|p| {
                            *p != self.owner
                                && ctx.core.player(*p).alive
                                && !ctx.core.player(*p).hand.is_empty()
                        })
                        .collect();
                    self.senluo_next_or_done(ctx)
                }
                HostStage::SelectCard => {
                    if let Some(t) = self.target {
                        let picked: Vec<CardId> = ctx
                            .answer()
                            .iter()
                            .filter_map(|o| match o {
                                ChoiceOption::Card(c) if ctx.core.player(t).hand.contains(c) => {
                                    Some(*c)
                                }
                                _ => None,
                            })
                            .collect();
                        for c in &picked {
                            ctx.discard_from_hand(t, *c);
                        }
                        if ctx.core.player(t).alive {
                            ctx.start_draw(t, 1);
                        }
                        self.stage = HostStage::DrawSettle;
                        return Flow::Yield;
                    }
                    self.senluo_next_or_done(ctx)
                }
                HostStage::DrawSettle => self.senluo_next_or_done(ctx),
                _ => Flow::Done,
            },
            // 灵弦（swd/skill.js:2671）：自选一名攻击距离 >1 的其他存活角色，先自己后目标各摸 1（LIFO 先压 target 后压 owner）
            HostBehavior::SwdLingxian => match self.stage {
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            *p != self.owner
                                && ctx.core.computed_distance(ctx.catalog, self.owner, *p) > 1
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
                            prompt: "lingxian_target",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    if let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() {
                        if ctx.core.player(t).alive && t != self.owner {
                            ctx.start_draw(t, 1);
                            ctx.start_draw(self.owner, 1);
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 乖戾（swd/skill.js:2600）：Before 面 +1 并标 armed；After 面自选弃 2 张手牌
            HostBehavior::SwdGuaili => match self.stage {
                HostStage::Act => {
                    let root_id = ctx.current_event_frame_id();
                    let frame = ctx.res.frame(root_id).map(|f| &f.data);
                    let Some(EventData::Damage(d)) = frame else {
                        return Flow::Done;
                    };
                    if d.prevented || d.source != Some(self.owner) {
                        return Flow::Done;
                    }
                    let after = !ctx
                        .current_damage()
                        .expect("damage root")
                        .settlement
                        .progress
                        .is_pre_hp();
                    let arm_key = format!("u05_guaili_armed:{root_id}");
                    if !after {
                        let frame = ctx.res.frame_mut(root_id).expect("frame");
                        if let EventData::Damage(d) = &mut frame.data {
                            if let Some(amount) = d.amount.checked_add(1) {
                                d.amount = amount;
                            } else {
                                ctx.fail_damage_amount();
                                return Flow::Done;
                            }
                        }
                        set_flag(ctx.core, self.owner, &arm_key, 1);
                        return Flow::Done;
                    }
                    if ctx.core.vm_aux.flags.remove(&(self.owner, arm_key)).is_none() {
                        return Flow::Done;
                    }
                    let hand: Vec<CardId> = ctx.core.player(self.owner).hand.iter().copied().collect();
                    if hand.is_empty() {
                        return Flow::Done;
                    }
                    if hand.len() <= 2 {
                        for c in &hand {
                            ctx.discard_from_hand(self.owner, *c);
                        }
                        ctx.log(LogEvent::CardDiscarded {
                            player: self.owner,
                            cards: hand,
                        });
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "guaili2_discard",
                            options: hand.into_iter().map(ChoiceOption::Card).collect(),
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
                            ChoiceOption::Card(c) if ctx.core.player(self.owner).hand.contains(c) => Some(*c),
                            _ => None,
                        })
                        .collect();
                    for c in &picked {
                        ctx.discard_from_hand(self.owner, *c);
                    }
                    if !picked.is_empty() {
                        ctx.log(LogEvent::CardDiscarded {
                            player: self.owner,
                            cards: picked,
                        });
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            _ => unreachable!("behavior routed to the wrong step module"),
        }
    }
}
