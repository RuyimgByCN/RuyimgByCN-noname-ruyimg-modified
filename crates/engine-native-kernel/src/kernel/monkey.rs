use super::*;

/// E03 猴子偷桃：使用前取消窗口（命令层/濒死机挂起 `MonkeyStealMachine`）。
/// 他人出【桃】且场上有存活持猴者时拦截：逐持有者确认——确认者弃猴子并把【桃】
/// 实体载体自处理区收为手牌（使用不发生）；全部放弃则按原路径（出牌发射 `CardUsed`、
/// 濒死直接救回并触发 `CardResponded`）。持有者死亡或失猴即跳过。
#[derive(Clone)]
pub struct MonkeyStealMachine {
    pub card: crate::ids::CardId,
    pub user: PlayerId,
    pub holders: Vec<PlayerId>,
    pub asked: bool,
}

impl MonkeyStealMachine {
    pub fn new(card: crate::ids::CardId, user: PlayerId, holders: Vec<PlayerId>) -> Self {
        Self {
            card,
            user,
            holders,
            asked: false,
        }
    }
}

/// `useCard` 开端（`useCardToBegin` / 猴子拦截之前）记录 `useCard` 历史与出牌统计，
/// 对齐上游 `content.ts:9976-9984`（即使后续被猴子取消仍计已使用）。
pub fn record_tao_use_stats(
    core: &mut crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
    out: &mut Vec<LogEvent>,
    user: PlayerId,
    card: crate::ids::CardId,
    effective_kind: CardKind,
    in_play_cmd: bool,
) {
    core.player_mut(user).total_used += 1;
    if effective_kind.is_basic_kind() {
        core.turn.mufeng_used_basic = true;
    }
    core.turn.cards_used_turn += 1;
    let during_own_play = core.turn.phase == Phase::Play && user == core.turn.current;
    if in_play_cmd || during_own_play {
        core.turn.cards_used_play_phase += 1;
        if effective_kind.is_basic_trick_or_equip() {
            core.turn.bte_used_play_phase += 1;
        }
        if catalog.vm_card_history {
            let bit = if effective_kind.is_basic_kind() {
                1u32
            } else if effective_kind.equip_slot().is_some() {
                4u32
            } else if effective_kind.is_basic_trick_or_equip() {
                2u32
            } else {
                0u32
            };
            if bit != 0 {
                *core
                    .vm_aux
                    .flags
                    .entry((user, "play_cat_mask".to_string()))
                    .or_insert(0) |= bit;
            }
        }
    }
    super::record_dddanzhi_card(core, catalog, Some(card));
    if in_play_cmd {
        out.push(LogEvent::CardPlayed {
            card,
            player: user,
            targets: Vec::new(),
            presentation: None,
        });
        core.vm_aux
            .flags
            .insert((user, "monkey_pre_counted".to_string()), 1);
    }
}

/// 猴子偷桃（欢乐卡牌 `skill.js:5-31`）只读判定：
/// 仅使用【桃】（含实体转化【桃】且排除 `VmViewAsZone::None`、`酒`/`护符`/`天仙酒`/`帝女玄霜`）时，
/// 按当前回合座位序收集存活且在宝物栏装备猴子的其他持猴者。
pub fn monkey_steal_holders(
    core: &crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
    user: PlayerId,
    card: crate::ids::CardId,
    in_play: bool,
) -> Vec<PlayerId> {
    let k = catalog.card_kind(card);
    if in_play {
        let in_hand = core.player(user).hand.contains(&card);
        if k != CardKind::Tao || !in_hand {
            if let Some(spec) =
                catalog.find_player_vm_view_as(core, user, card, true, Some(CardKind::Tao))
            {
                if spec.zone == crate::rvm_bridge::VmViewAsZone::None {
                    return Vec::new();
                }
            }
        }
    } else {
        if matches!(
            k,
            CardKind::Jiu | CardKind::Hufu | CardKind::Tianxianjiu
        ) {
            return Vec::new();
        }
        if let Some(spec) =
            catalog.find_player_vm_view_as(core, user, card, false, Some(CardKind::Tao))
        {
            if spec.zone == crate::rvm_bridge::VmViewAsZone::None {
                return Vec::new();
            }
        } else if k != CardKind::Tao
            && (k == CardKind::Dinvxuanshuang
                || catalog
                    .find_player_vm_view_as(core, user, card, false, Some(CardKind::Jiu))
                    .is_some())
        {
            return Vec::new();
        }
    }
    let mut eligible: Vec<PlayerId> = Vec::new();
    for key in [crate::vocabulary::host::HUANLE_MONKEY_KEY, "monkey"] {
        if let Some(d) = catalog.skill_by_name(key) {
            for ((o, idef), inst) in core.skills.range(..) {
                if *idef == d
                    && inst.enabled
                    && core.player(*o).alive
                    && *o != user
                    && core.player(*o).equips.iter().any(|c| {
                        catalog.card_slot_of(core, *c)
                            == Some(crate::catalog::EquipSlot::Treasure)
                            && catalog.card_kind(*c) == CardKind::Monkey
                    })
                    && !eligible.contains(o)
                {
                    eligible.push(*o);
                }
            }
        }
    }
    core.seat_order_from(core.turn.current)
        .into_iter()
        .filter(|p| eligible.contains(p))
        .collect()
}

/// 猴子偷桃挂起前把全部【桃】实体载体移入处理区（`core.table` = `filterInD`）。
pub fn stage_monkey_steal_carriers(
    core: &mut crate::state::GameStateCore,
    res: &mut crate::resolution::Resolution,
    catalog: &crate::catalog::Catalog,
    user: PlayerId,
    card: crate::ids::CardId,
    in_play: bool,
) {
    let in_hand = core.player(user).hand.contains(&card);
    let extra = if catalog.card_kind(card) != CardKind::Tao || !in_hand {
        if let Some(spec) = catalog
            .find_player_vm_view_as(core, user, card, in_play, Some(CardKind::Tao))
            .cloned()
        {
            if spec.cost == Some(crate::rvm_bridge::VmViewAsCost::TurnOver) {
                let _ = crate::resolution::push_frame(
                    res,
                    EventData::TurnOverBefore {
                        player: user,
                        cancelled: false,
                    },
                );
            }
            spec.consume_and_record(core, catalog, user, card)
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };
    if core.player(user).hand.contains(&card) {
        core.player_mut(user).hand.retain(|&c| c != card);
    } else if core.player(user).equips.contains(&card) {
        core.player_mut(user).equips.retain(|&c| c != card);
        core.declared_slots.remove(&card);
        revoke_equip_skills(core, catalog, user, card);
    } else {
        for ((owner, _), pile) in core.vm_aux.piles.iter_mut() {
            if *owner == user {
                pile.retain(|&c| c != card);
            }
        }
    }
    if !core.table.contains(&card) {
        core.table.push(card);
    }
    if !extra.is_empty() {
        core.vm_aux
            .flags
            .insert((user, "monkey_extra_len".to_string()), extra.len() as u32);
        for (i, c2) in extra.into_iter().enumerate() {
            if !core.table.contains(&c2) {
                core.table.push(c2);
            }
            core.vm_aux
                .flags
                .insert((user, format!("monkey_extra:{i}")), c2);
        }
    }
}

impl crate::catalog::SkillMachine for MonkeyStealMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> crate::snap::MachineSnapshot {
        crate::snap::MachineSnapshot::MonkeySteal {
            card: self.card,
            user: self.user,
            holders: self.holders.clone(),
            asked: self.asked,
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        let carriers = monkey_carriers(ctx.core, self.user, self.card);
        loop {
            let in_d_cards: Vec<crate::ids::CardId> = carriers
                .iter()
                .copied()
                .filter(|c| ctx.core.table.contains(c))
                .collect();
            if in_d_cards.is_empty() {
                let dying_target = ctx
                    .core
                    .vm_aux
                    .flags
                    .remove(&(self.user, "monkey_dying_target".to_string()))
                    .and_then(|v| v.checked_sub(1))
                    .map(|v| v as PlayerId);
                clear_monkey_extra_flags(ctx.core, self.user);
                if let Some(dying) = dying_target {
                    crate::kernel::dying::DyingSaveMachine::new(
                        dying,
                        None,
                        Vec::new(),
                        false,
                        false,
                    )
                    .apply_tao_save(ctx, self.user, self.card);
                } else {
                    let use_as = (ctx.catalog.card_kind(self.card) != CardKind::Tao)
                        .then_some(CardKind::Tao);
                    ctx.emit(EventData::CardUsed(crate::machine::card_use_as(
                        self.card,
                        self.user,
                        Vec::new(),
                        use_as,
                    )));
                }
                return Flow::Done;
            }
            // 跳过死亡或已失去猴子的持有者。
            while let Some(h) = self.holders.first().copied() {
                let has_monkey_card = ctx.core.player(h).equips.iter().any(|c| {
                    ctx.catalog.card_slot_of(ctx.core, *c)
                        == Some(crate::catalog::EquipSlot::Treasure)
                        && ctx.catalog.card_kind(*c) == CardKind::Monkey
                });
                let armed = has_monkey_card
                    && [crate::vocabulary::host::HUANLE_MONKEY_KEY, "monkey"]
                        .into_iter()
                        .any(|k| {
                            ctx.catalog
                                .skill_by_name(k)
                                .is_some_and(|d| ctx.core.has_skill(h, d))
                        });
                if !ctx.core.player(h).alive || !armed {
                    self.holders.remove(0);
                } else {
                    break;
                }
            }
            let Some(holder) = self.holders.first().copied() else {
                // 全部放弃：出牌发射 `CardUsed`（触发 `useCardToBegin` 等监听与弃置/回血）；
                // 濒死则直接弃置载体并按普通救回结算。
                let dying_target = ctx
                    .core
                    .vm_aux
                    .flags
                    .remove(&(self.user, "monkey_dying_target".to_string()))
                    .and_then(|v| v.checked_sub(1))
                    .map(|v| v as PlayerId);
                clear_monkey_extra_flags(ctx.core, self.user);
                if let Some(dying) = dying_target {
                    for c in &in_d_cards {
                        ctx.core.table.retain(|x| x != c);
                        ctx.to_discard(*c);
                    }
                    crate::kernel::dying::DyingSaveMachine::new(
                        dying,
                        None,
                        Vec::new(),
                        false,
                        false,
                    )
                    .apply_tao_save(ctx, self.user, self.card);
                } else {
                    for c in &in_d_cards {
                        if *c != self.card {
                            ctx.core.table.retain(|x| x != c);
                            ctx.to_discard(*c);
                        }
                    }
                    let use_as = (ctx.catalog.card_kind(self.card) != CardKind::Tao)
                        .then_some(CardKind::Tao);
                    ctx.emit(EventData::CardUsed(crate::machine::card_use_as(
                        self.card,
                        self.user,
                        Vec::new(),
                        use_as,
                    )));
                }
                return Flow::Done;
            };
            if self.asked {
                self.asked = false;
                self.holders.remove(0);
                let confirmed = ctx
                    .answer()
                    .first()
                    .map(|o| *o == ChoiceOption::Confirm)
                    .unwrap_or(false);
                if confirmed {
                    // 取消使用：从处理区（filterInD）收为持有者手牌，弃猴子（离场撤销装备技）。
                    ctx.core.table.retain(|c| !in_d_cards.contains(c));
                    ctx.core.players[holder as usize]
                        .hand
                        .extend_from_slice(&in_d_cards);
                    ctx.log(LogEvent::CardsGivenPublic {
                        from: self.user,
                        to: holder,
                        cards: in_d_cards,
                    });
                    if let Some(pos) = ctx.core.players[holder as usize]
                        .equips
                        .iter()
                        .position(|c| {
                            ctx.catalog.card_slot_of(ctx.core, *c)
                                == Some(crate::catalog::EquipSlot::Treasure)
                                && ctx.catalog.card_kind(*c) == CardKind::Monkey
                        })
                    {
                        let monkey = ctx.core.players[holder as usize].equips.remove(pos);
                        revoke_equip_skills(&mut ctx.core, ctx.catalog, holder, monkey);
                        ctx.core.discard.push(monkey);
                        crate::rvm_bridge::modifiers::d34_l28_mods::note_pro_qinyin_discard(
                            ctx.core,
                            ctx.catalog,
                            monkey,
                        );
                        ctx.log(LogEvent::CardDiscarded {
                            player: holder,
                            cards: vec![monkey],
                        });
                    }
                    clear_monkey_extra_flags(ctx.core, self.user);
                    ctx.core
                        .vm_aux
                        .flags
                        .remove(&(self.user, "monkey_dying_target".to_string()));
                    ctx.core
                        .vm_aux
                        .flags
                        .remove(&(self.user, "monkey_pre_counted".to_string()));
                    ctx.core
                        .vm_aux
                        .flags
                        .remove(&(self.user, "vm_va_recover_bonus".to_string()));
                    return Flow::Done; // 使用被取消：无 CardUsed 帧。
                }
                continue; // 放弃 → 队列下一持有者。
            }
            self.asked = true;
            return Flow::WaitChoice {
                observer: holder,
                spec: ChoiceSpec {
                    kind: ChoiceKind::RespondCard,
                    prompt: "huanle_monkey_confirm",
                    options: vec![ChoiceOption::Confirm, ChoiceOption::Pass],
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            };
        }
    }
}

fn monkey_carriers(
    core: &crate::state::GameStateCore,
    user: PlayerId,
    primary: crate::ids::CardId,
) -> Vec<crate::ids::CardId> {
    let mut out = vec![primary];
    let extra_len = core
        .vm_aux
        .flags
        .get(&(user, "monkey_extra_len".to_string()))
        .copied()
        .unwrap_or(0);
    for i in 0..extra_len {
        if let Some(&c) = core.vm_aux.flags.get(&(user, format!("monkey_extra:{i}"))) {
            if !out.contains(&c) {
                out.push(c);
            }
        }
    }
    out
}

fn clear_monkey_extra_flags(core: &mut crate::state::GameStateCore, user: PlayerId) {
    let extra_len = core
        .vm_aux
        .flags
        .remove(&(user, "monkey_extra_len".to_string()))
        .unwrap_or(0);
    for i in 0..extra_len {
        core.vm_aux
            .flags
            .remove(&(user, format!("monkey_extra:{i}")));
    }
}
