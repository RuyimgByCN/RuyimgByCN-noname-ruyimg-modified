use super::asks_bcd::*;
use super::*;

pub(crate) fn step_bcd(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Option<Flow> {
    let flow = match machine.def_key.as_str() {
        "dddzhilian" => step_zhilian(machine, ctx),
        "dddjijian" => step_jijian(machine, ctx),
        "dddjijian_old" => step_jijian_old(machine, ctx),
        "dddfushi" => step_fushi(machine, ctx),
        "dddlangzhi" => step_langzhi(machine, ctx),
        "dddfuzong" => step_fuzong(machine, ctx),
        _ => return None,
    };
    Some(flow)
}

fn zhilian_scoop_turn_shas(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let stamp = l28_stamp(ctx.core);
    let scooped: Vec<CardId> = ctx
        .core
        .discard
        .iter()
        .copied()
        .filter(|&c| {
            ctx.catalog.card_kind(c).is_sha_like()
                && flag(ctx.core, 0, &format!("dddzhilian_sha_d:{c}")) == stamp
        })
        .collect();
    if !scooped.is_empty() && ctx.core.player(owner).alive {
        for &c in &scooped {
            ctx.core.discard.retain(|x| *x != c);
            ctx.core.player_mut(owner).hand.push(c);
        }
        ctx.log(LogEvent::CardsDrawn {
            player: owner,
            cards: scooped,
        });
    }
}

fn step_zhilian(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = machine.owner;
    match machine.stage {
        HostStage::Act => wait_ask_fix(machine, ctx, HostStage::SelectNature, 1051),
        HostStage::SelectNature => {
            let Some(ChoiceOption::Tag(mode)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            let Some(tgts) = dddzhilian_targets_eligible(ctx.core, ctx.catalog, owner) else {
                return Flow::Done;
            };
            if mode == 0 {
                for &t in &tgts {
                    if ctx.core.player(t).alive {
                        u01_draw_now(ctx, t, 1);
                    }
                }
                zhilian_scoop_turn_shas(ctx, owner);
                ctx.log(LogEvent::SkillTriggered {
                    skill: "dddzhilian",
                    owner,
                    presentation: None,
                });
                Flow::Done
            } else {
                machine.targets = tgts;
                zhilian_next_discard(machine, ctx)
            }
        }
        HostStage::SelectCard => {
            if let (Some(t), Some(ChoiceOption::Card(c))) =
                (machine.target, ctx.answer().first().copied())
            {
                if he_cards(ctx.core, t).contains(&c) {
                    u01_discard_one(ctx, t, c);
                }
            }
            zhilian_next_discard(machine, ctx)
        }
        _ => Flow::Done,
    }
}

fn zhilian_next_discard(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    while !machine.targets.is_empty() {
        let t = machine.targets.remove(0);
        if ctx.core.player(t).alive && !he_cards(ctx.core, t).is_empty() {
            machine.target = Some(t);
            return wait_ask_fix(machine, ctx, HostStage::SelectCard, 1052);
        }
    }
    zhilian_scoop_turn_shas(ctx, machine.owner);
    ctx.log(LogEvent::SkillTriggered {
        skill: "dddzhilian",
        owner: machine.owner,
        presentation: None,
    });
    Flow::Done
}

fn step_jijian(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = machine.owner;
    match machine.stage {
        HostStage::Act => wait_ask_fix(machine, ctx, HostStage::SelectCard, 1053),
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(sha)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            if !ctx.core.player(owner).hand.contains(&sha) {
                return Flow::Done;
            }
            machine.cards = vec![sha];
            wait_ask_fix(machine, ctx, HostStage::TargetChoice, 1054)
        }
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            let Some(&sha) = machine.cards.first() else {
                return Flow::Done;
            };
            if !ctx.core.player(t).alive {
                return Flow::Done;
            }
            mark_scope4_used(ctx.core, owner, "dddjijian");
            ctx.log(LogEvent::CardsRevealed {
                cards: vec![sha],
                to: ctx.core.alive_players().collect(),
            });
            u01_give_card_ex(ctx, owner, t, sha, false);
            machine.target = Some(t);
            let order: Vec<PlayerId> = ctx
                .core
                .seat_order_from(ctx.core.turn.current)
                .into_iter()
                .filter(|&p| p == owner || p == t)
                .collect();
            machine.targets = order;
            machine.card = Some(u32::MAX);
            machine.stage = HostStage::DrawSettle;
            Flow::Yield
        }
        HostStage::CardsAsk => {
            let Some(t) = machine.target else {
                return Flow::Done;
            };
            let shown: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Card(c) => Some(*c),
                    _ => None,
                })
                .filter(|c| ctx.core.player(t).hand.contains(c))
                .collect();
            if shown.is_empty() {
                return Flow::Done;
            }
            ctx.log(LogEvent::CardsRevealed {
                cards: shown.clone(),
                to: ctx.core.alive_players().collect(),
            });
            machine.card = Some(shown.len() as u32);
            machine.cards = shown;
            jijian_next_actor(machine, ctx)
        }
        HostStage::SelectNature => {
            let Some(ChoiceOption::Kind(code)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            let Some(actor) = machine.target else {
                return Flow::Done;
            };
            let Some(k) = kind_from_code(code) else {
                return Flow::Done;
            };
            machine.cards.push(u32::from(canonical_jijian_key(k)));
            if virtual_kind_needs_target_prompt(k) {
                set_flag(ctx.core, owner, "u01_active_vkind", u32::from(code));
                wait_ask_fix(machine, ctx, HostStage::JiefenTarget, 1057)
            } else {
                let tgts = virtual_legal_targets(ctx.core, ctx.catalog, actor, k);
                if !tgts.is_empty() {
                    jijian_emit_virtual(ctx, actor, k, tgts);
                }
                machine.stage = HostStage::DrawSettle;
                Flow::Yield
            }
        }
        HostStage::JiefenTarget => {
            let Some(actor) = machine.target else {
                return Flow::Done;
            };
            let tgts: Vec<PlayerId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Player(p) => Some(*p),
                    _ => None,
                })
                .collect();
            let code_val = flag(ctx.core, owner, "u01_active_vkind");
            if let Some(k) = u16::try_from(code_val).ok().and_then(kind_from_code) {
                if let Some(flow) = jijian_handle_target_step(
                    machine, ctx, owner, actor, k, &tgts, 1057,
                ) {
                    return flow;
                }
            }
            set_flag(ctx.core, owner, "u01_active_vkind", 0);
            machine.stage = HostStage::DrawSettle;
            Flow::Yield
        }
        HostStage::DrawSettle => {
            if machine.card == Some(u32::MAX) {
                machine.card = None;
                let Some(t) = machine.target else {
                    return Flow::Done;
                };
                if !ctx.core.player(t).alive {
                    return Flow::Done;
                }
                let valid = dddjijian_color_valid_cards(ctx.core, ctx.catalog, t);
                return if valid.is_empty() {
                    Flow::Done
                } else if ctx.core.player(t).hand.len() <= 1 || valid.len() == 1 {
                    let shown = vec![valid[0]];
                    ctx.log(LogEvent::CardsRevealed {
                        cards: shown.clone(),
                        to: ctx.core.alive_players().collect(),
                    });
                    machine.card = Some(1);
                    machine.cards = shown;
                    jijian_next_actor(machine, ctx)
                } else {
                    wait_ask_fix(machine, ctx, HostStage::CardsAsk, 1055)
                };
            }
            if let Some(actor) = machine.target {
                if let Some(flow) = jijian_step_huogong(machine, ctx, owner, actor) {
                    return flow;
                }
            }
            jijian_next_actor(machine, ctx)
        }
        _ => Flow::Done,
    }
}

fn jijian_next_actor(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    while !machine.targets.is_empty() {
        let actor = machine.targets.remove(0);
        if !ctx.core.player(actor).alive {
            continue;
        }
        machine.target = Some(actor);
        let shown_cnt = machine.card.unwrap_or(0) as usize;
        let used = &machine.cards[shown_cnt..];
        let any_usable = machine.cards[..shown_cnt].iter().any(|&c| {
            let k = canonical_virt_kind(ctx.catalog, c);
            !used.contains(&canonical_jijian_key(k))
                && virtual_kind_can_use_play(ctx.core, ctx.catalog, actor, k)
        });
        if !any_usable {
            return Flow::Done;
        }
        return wait_ask_fix(machine, ctx, HostStage::SelectNature, 1056);
    }
    Flow::Done
}

fn jijian_clear_sub_flags(core: &mut GameStateCore, owner: PlayerId, actor: PlayerId) {
    set_flag(core, actor, "u01_jiedao_holder", 0);
    set_flag(core, owner, "u01_hg_stage", 0);
    set_flag(core, owner, "u01_hg_target", 0);
    set_flag(core, owner, "u01_hg_shown", 0);
    set_flag(core, owner, "u01_active_vkind", 0);
}

fn jijian_hg_try_discard(
    machine: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    actor: PlayerId,
    hg_tgt: PlayerId,
    c_show: CardId,
) -> Option<Flow> {
    set_flag(ctx.core, owner, "u01_hg_shown", c_show);
    let suit = ctx.catalog.effective_card_suit(ctx.core, hg_tgt, c_show);
    let has_match = ctx
        .core
        .player(actor)
        .hand
        .iter()
        .any(|&c| ctx.catalog.effective_card_suit(ctx.core, actor, c) == suit);
    if has_match && ctx.core.player(actor).alive {
        set_flag(ctx.core, owner, "u01_hg_stage", 20);
        Some(wait_ask_fix(machine, ctx, HostStage::DrawSettle, 1073))
    } else {
        jijian_clear_sub_flags(ctx.core, owner, actor);
        None
    }
}

fn jijian_step_huogong(
    machine: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    actor: PlayerId,
) -> Option<Flow> {
    let hg_stage = flag(ctx.core, owner, "u01_hg_stage");
    if hg_stage == 0 {
        return None;
    }
    let Some(hg_tgt) = flag(ctx.core, owner, "u01_hg_target").checked_sub(1).map(|v| v as PlayerId) else {
        jijian_clear_sub_flags(ctx.core, owner, actor);
        return None;
    };
    match hg_stage {
        1 => {
            if !ctx.core.player(actor).alive
                || !ctx.core.player(hg_tgt).alive
                || ctx.core.player(hg_tgt).hand.is_empty()
            {
                jijian_clear_sub_flags(ctx.core, owner, actor);
                return None;
            }
            if ctx.core.player(hg_tgt).hand.len() == 1 {
                let c_show = ctx.core.player(hg_tgt).hand[0];
                ctx.log(LogEvent::CardsRevealed {
                    cards: vec![c_show],
                    to: ctx.core.alive_players().collect(),
                });
                jijian_hg_try_discard(machine, ctx, owner, actor, hg_tgt, c_show)
            } else {
                set_flag(ctx.core, owner, "u01_hg_stage", 10);
                Some(wait_ask_fix(machine, ctx, HostStage::DrawSettle, 1072))
            }
        }
        10 => {
            let Some(ChoiceOption::Card(c_show)) = ctx.answer().first().copied() else {
                jijian_clear_sub_flags(ctx.core, owner, actor);
                return None;
            };
            if !ctx.core.player(hg_tgt).hand.contains(&c_show) {
                jijian_clear_sub_flags(ctx.core, owner, actor);
                return None;
            }
            ctx.log(LogEvent::CardsRevealed {
                cards: vec![c_show],
                to: ctx.core.alive_players().collect(),
            });
            jijian_hg_try_discard(machine, ctx, owner, actor, hg_tgt, c_show)
        }
        20 => {
            let c_show = flag(ctx.core, owner, "u01_hg_shown");
            let suit = ctx.catalog.effective_card_suit(ctx.core, hg_tgt, c_show);
            if let Some(ChoiceOption::Card(disc)) = ctx.answer().first().copied() {
                if ctx.core.player(actor).hand.contains(&disc)
                    && ctx.catalog.effective_card_suit(ctx.core, actor, disc) == suit
                {
                    u01_discard_batch(ctx, actor, &[disc]);
                    set_flag(ctx.core, owner, "u01_hg_stage", 30);
                    return Some(Flow::Yield);
                }
            }
            jijian_clear_sub_flags(ctx.core, owner, actor);
            None
        }
        30 => {
            jijian_clear_sub_flags(ctx.core, owner, actor);
            if ctx.core.player(hg_tgt).alive {
                ctx.emit(EventData::Damage(damage_data_with(
                    Some(actor),
                    hg_tgt,
                    1,
                    DamageNature::Fire,
                    None,
                )));
                return Some(Flow::Yield);
            }
            None
        }
        _ => {
            jijian_clear_sub_flags(ctx.core, owner, actor);
            None
        }
    }
}

fn jijian_handle_target_step(
    machine: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    actor: PlayerId,
    k: CardKind,
    tgts: &[PlayerId],
    ask_id: u32,
) -> Option<Flow> {
    if tgts.is_empty() {
        return None;
    }
    if k == CardKind::Jiedao {
        let jh = flag(ctx.core, actor, "u01_jiedao_holder");
        if jh == 0 {
            let holder = tgts[0];
            if holder != actor
                && ctx.core.player(holder).alive
                && ctx.core.player(holder).equips.iter().any(|&c| {
                    ctx.catalog.card_slot_of(ctx.core, c) == Some(EquipSlot::Weapon)
                })
                && !jiedao_victims_for(ctx.core, ctx.catalog, holder).is_empty()
            {
                set_flag(ctx.core, actor, "u01_jiedao_holder", u32::from(holder) + 1);
                return Some(wait_ask_fix(machine, ctx, HostStage::JiefenTarget, ask_id));
            }
            return None;
        }
        let holder = (jh - 1) as PlayerId;
        set_flag(ctx.core, actor, "u01_jiedao_holder", 0);
        let victim = tgts[0];
        if ctx.core.player(holder).alive && ctx.core.player(victim).alive {
            jijian_emit_virtual(ctx, actor, CardKind::Jiedao, vec![holder, victim]);
        }
        return None;
    }
    if k == CardKind::Linghunzhihuo {
        let hg_tgt = tgts[0];
        if ctx.core.player(hg_tgt).alive && !ctx.core.player(hg_tgt).hand.is_empty() {
            set_flag(ctx.core, owner, "u01_hg_target", u32::from(hg_tgt) + 1);
            set_flag(ctx.core, owner, "u01_hg_stage", 1);
            jijian_emit_virtual(ctx, actor, CardKind::Linghunzhihuo, vec![hg_tgt]);
        }
        return None;
    }
    jijian_emit_virtual(ctx, actor, k, tgts.to_vec());
    None
}

fn jijian_emit_virtual(ctx: &mut Ctx<'_>, actor: PlayerId, k: CardKind, targets: Vec<PlayerId>) {
    let nature = match k {
        CardKind::FireSha | CardKind::Linghunzhihuo => DamageNature::Fire,
        CardKind::LeiSha => DamageNature::Thunder,
        _ => DamageNature::Normal,
    };
    if k.is_sha_like() {
        set_flag(ctx.core, actor, "vm_va_sha_no_count", 1);
    }
    if matches!(k, CardKind::Jiu | CardKind::Tianxianjiu) {
        set_flag(ctx.core, actor, "vm_va_jiu_no_count", 1);
    }
    ctx.emit(EventData::CardUsed(CardUseData {
        card: None,
        player: actor,
        targets,
        nature,
        use_as: Some(k),
        cancelled: false,
    }));
}

fn step_jijian_old(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = machine.owner;
    match machine.stage {
        HostStage::Act => wait_ask_fix(machine, ctx, HostStage::TargetChoice, 1058),
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            if !ctx.core.player(t).alive || ctx.core.player(t).hand.is_empty() {
                return Flow::Done;
            }
            mark_scope4_used(ctx.core, owner, "dddjijian_old");
            machine.target = Some(t);
            wait_ask_fix(machine, ctx, HostStage::CardsAsk, 1059)
        }
        HostStage::CardsAsk => {
            let Some(t) = machine.target else {
                return Flow::Done;
            };
            let shown: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Card(c) => Some(*c),
                    _ => None,
                })
                .filter(|c| ctx.core.player(t).hand.contains(c))
                .collect();
            if shown.is_empty() {
                return Flow::Done;
            }
            ctx.log(LogEvent::CardsRevealed {
                cards: shown.clone(),
                to: ctx.core.alive_players().collect(),
            });
            machine.cards = vec![shown.len() as u32];
            machine.cards.extend(shown);
            if ctx
                .core
                .player(owner)
                .hand
                .iter()
                .any(|&c| ctx.catalog.card_kind(c).is_sha_like())
            {
                wait_ask_fix(machine, ctx, HostStage::SelectCard, 1060)
            } else {
                Flow::Done
            }
        }
        HostStage::SelectCard => {
            let (Some(t), Some(ChoiceOption::Card(sha))) =
                (machine.target, ctx.answer().first().copied())
            else {
                return Flow::Done;
            };
            if !ctx.core.player(owner).hand.contains(&sha) {
                return Flow::Done;
            }
            let sha_red = effective_is_red(ctx.core, ctx.catalog, owner, sha);
            u01_give_card_ex(ctx, owner, t, sha, false);
            let shown_cnt = machine.cards.first().copied().unwrap_or(0) as usize;
            let same_color = machine.cards[1..1 + shown_cnt]
                .iter()
                .filter(|&&c| effective_is_red(ctx.core, ctx.catalog, t, c) == sha_red)
                .count();
            if same_color == 0 {
                return Flow::Done;
            }
            machine.targets = vec![same_color as u8];
            machine.card = Some(u32::MAX);
            machine.stage = HostStage::DrawSettle;
            Flow::Yield
        }
        HostStage::SelectNature => {
            let Some(ChoiceOption::Kind(code)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            let Some(t) = machine.target else {
                return Flow::Done;
            };
            let Some(k) = kind_from_code(code) else {
                return Flow::Done;
            };
            let shown_cnt = machine.cards.first().copied().unwrap_or(0) as usize;
            if let Some(pos) = machine.cards[1 + shown_cnt..]
                .iter()
                .position(|&c| c == u32::from(code))
            {
                machine.cards.remove(1 + shown_cnt + pos);
            }
            let rem = machine.targets.first().copied().unwrap_or(0).saturating_sub(1);
            machine.targets = vec![rem];
            machine.card = Some(u32::from(code));
            if virtual_kind_needs_target_prompt(k) {
                wait_ask_fix(machine, ctx, HostStage::JiefenTarget, 1062)
            } else {
                let tgts = virtual_legal_targets(ctx.core, ctx.catalog, t, k);
                if !tgts.is_empty() {
                    jijian_emit_virtual(ctx, t, k, tgts);
                }
                machine.stage = HostStage::DrawSettle;
                Flow::Yield
            }
        }
        HostStage::JiefenTarget => {
            let Some(t) = machine.target else {
                return Flow::Done;
            };
            let tgts: Vec<PlayerId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Player(p) => Some(*p),
                    _ => None,
                })
                .collect();
            if let Some(k) = machine
                .card
                .and_then(|c| u16::try_from(c).ok())
                .and_then(kind_from_code)
            {
                if let Some(flow) = jijian_handle_target_step(machine, ctx, owner, t, k, &tgts, 1062)
                {
                    return flow;
                }
            }
            machine.stage = HostStage::DrawSettle;
            Flow::Yield
        }
        HostStage::DrawSettle => {
            if machine.card == Some(u32::MAX) {
                let Some(t) = machine.target else {
                    return Flow::Done;
                };
                if !ctx.core.player(t).alive {
                    return Flow::Done;
                }
                let shown_cnt = machine.cards.first().copied().unwrap_or(0) as usize;
                let shown = machine.cards[1..1 + shown_cnt].to_vec();
                let rem_names = dddjijian_old_snapshot_names(ctx.core, ctx.catalog, t, &shown);
                machine.cards.truncate(1 + shown_cnt);
                machine.cards.extend(rem_names);
                machine.card = Some(u32::from(machine.targets.first().copied().unwrap_or(0)));
                return jijian_old_next_use(machine, ctx);
            }
            if let Some(t) = machine.target {
                if let Some(flow) = jijian_step_huogong(machine, ctx, owner, t) {
                    return flow;
                }
            }
            let rem = u32::from(machine.targets.first().copied().unwrap_or(0));
            machine.card = Some(rem);
            jijian_old_next_use(machine, ctx)
        }
        _ => Flow::Done,
    }
}

fn jijian_old_next_use(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let rem = machine.targets.first().copied().unwrap_or(0);
    let Some(t) = machine.target else {
        return Flow::Done;
    };
    if rem == 0 || !ctx.core.player(t).alive {
        return Flow::Done;
    }
    let shown_cnt = machine.cards.first().copied().unwrap_or(0) as usize;
    let rem_names = &machine.cards[1 + shown_cnt..];
    if dddjijian_old_usable_codes(ctx.core, ctx.catalog, t, rem_names).is_empty() {
        return Flow::Done;
    }
    wait_ask_fix(machine, ctx, HostStage::SelectNature, 1061)
}

fn fushi_swap_pile(ctx: &mut Ctx<'_>, owner: PlayerId, target: PlayerId, idx: u8) {
    let pile_key = (
        owner,
        if idx == 0 {
            "dddmiaoxing_1".to_string()
        } else {
            "dddmiaoxing_2".to_string()
        },
    );
    let from_pile = ctx.core.vm_aux.piles.remove(&pile_key).unwrap_or_default();
    let old_hand: Vec<CardId> = ctx.core.player(target).hand.clone();
    for &c in &old_hand {
        ctx.take_hand_card(target, c);
    }
    if !old_hand.is_empty() {
        let lost_n = old_hand.len();
        ctx.core.vm_aux.piles.insert(pile_key, old_hand.clone());
        ctx.emit(EventData::CardsLost {
            player: target,
            cards: old_hand,
            origins: vec![LostOrigin::Hand; lost_n],
        });
    }
    if !from_pile.is_empty() {
        ctx.core.player_mut(target).hand.extend(from_pile.iter().copied());
        ctx.log(LogEvent::CardsDrawn {
            player: target,
            cards: from_pile,
        });
    }
}

fn step_fushi(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = machine.owner;
    match machine.stage {
        HostStage::Act => wait_ask_fix(machine, ctx, HostStage::SelectNature, 1063),
        HostStage::SelectNature => {
            let Some(ChoiceOption::Tag(idx)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            set_turn_flag(ctx.core, owner, "dddfushi_used_turn", "dddfushi_stamp", 1);
            fushi_swap_pile(ctx, owner, owner, idx);
            machine.stage = HostStage::DrawSettle;
            Flow::Yield
        }
        HostStage::DrawSettle => wait_ask_fix(machine, ctx, HostStage::TargetChoice, 1064),
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            if !ctx.core.player(t).alive {
                return Flow::Done;
            }
            machine.target = Some(t);
            wait_ask_fix(machine, ctx, HostStage::JiefenTarget, 1065)
        }
        HostStage::JiefenTarget => {
            if let (Some(t), Some(ChoiceOption::Tag(idx))) =
                (machine.target, ctx.answer().first().copied())
            {
                if ctx.core.player(t).alive {
                    fushi_swap_pile(ctx, owner, t, idx);
                }
            }
            ctx.log(LogEvent::SkillTriggered {
                skill: "dddfushi",
                owner,
                presentation: None,
            });
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn langzhi_cancel_and_gain_all(machine: &mut HostMachine, ctx: &mut Ctx<'_>, owner: PlayerId) {
    for c in std::mem::take(&mut machine.cards) {
        let holder = ctx
            .core
            .alive_players()
            .find(|&p| p != owner && he_cards(ctx.core, p).contains(&c));
        if let Some(holder) = holder {
            u01_give_card_ex(ctx, holder, owner, c, true);
        }
    }
    if let Some(def) = ctx.catalog.skill_by_name("dddlangzhi") {
        ctx.core.skills.remove(&(owner, def));
        ctx.log(LogEvent::SkillToggled {
            player: owner,
            skill: "dddlangzhi",
            enabled: false,
        });
    }
    set_flag(ctx.core, owner, "dddlangzhi_lost", 1);
}

fn langzhi_in_range_targets(core: &GameStateCore, catalog: &Catalog, owner: PlayerId) -> Vec<PlayerId> {
    let atk = core.attack_range(catalog, owner);
    core.seat_order_from(owner)
        .into_iter()
        .filter(|&p| {
            p != owner
                && core.player(p).alive
                && core.computed_distance(catalog, owner, p) <= atk
                && !he_cards(core, p).is_empty()
        })
        .collect()
}

fn step_langzhi(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = machine.owner;
    match machine.stage {
        HostStage::Act => wait_ask_fix(machine, ctx, HostStage::JiefenTarget, 1066),
        HostStage::JiefenTarget => {
            if !matches!(ctx.answer().first().copied(), Some(ChoiceOption::Confirm)) {
                return Flow::Done;
            }
            machine.targets = langzhi_in_range_targets(ctx.core, ctx.catalog, owner);
            machine.cards.clear();
            langzhi_next_reveal(machine, ctx)
        }
        HostStage::SelectCard => {
            if let Some(t) = machine.target {
                let keep_len = flag(ctx.core, owner, "dddlangzhi_rev_len") as usize;
                let picked = match ctx.answer().first().copied() {
                    Some(ChoiceOption::HiddenCard(slot)) => machine
                        .cards
                        .get(keep_len + slot as usize)
                        .copied()
                        .filter(|c| ctx.core.player(t).hand.contains(c)),
                    Some(ChoiceOption::Card(c)) if ctx.core.player(t).equips.contains(&c) => {
                        Some(c)
                    }
                    _ => None,
                };
                machine.cards.truncate(keep_len);
                if let Some(c) = picked {
                    ctx.log(LogEvent::CardsRevealed {
                        cards: vec![c],
                        to: ctx.core.alive_players().collect(),
                    });
                    machine.cards.push(c);
                    set_flag(ctx.core, owner, "dddlangzhi_rev_len", machine.cards.len() as u32);
                }
            }
            langzhi_next_reveal(machine, ctx)
        }
        HostStage::CardsAsk => {
            let picks: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Card(c) => Some(*c),
                    _ => None,
                })
                .filter(|c| he_cards(ctx.core, owner).contains(c))
                .collect();
            if picks.is_empty() {
                langzhi_cancel_and_gain_all(machine, ctx, owner);
                return Flow::Done;
            }
            let k = picks.len();
            machine.card = Some(k as u32);
            machine.cards.extend(picks);
            wait_ask_fix(machine, ctx, HostStage::SelectNature, 1069)
        }
        HostStage::SelectNature => {
            let k = machine.card.unwrap_or(0) as usize;
            let pool_end = machine.cards.len().saturating_sub(k);
            let own_picks: Vec<CardId> = machine.cards[pool_end..].to_vec();
            let chosen_rev: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Card(c) => Some(*c),
                    _ => None,
                })
                .take(k)
                .collect();
            for (c_own, c_rev) in own_picks.into_iter().zip(chosen_rev.into_iter()) {
                let holder = ctx
                    .core
                    .alive_players()
                    .find(|&p| p != owner && he_cards(ctx.core, p).contains(&c_rev));
                if let Some(holder) = holder {
                    if he_cards(ctx.core, owner).contains(&c_own) {
                        u01_swap_handcards_simultaneous(ctx, owner, &[c_own], holder, &[c_rev]);
                    }
                }
            }
            ctx.log(LogEvent::SkillTriggered {
                skill: "dddlangzhi",
                owner,
                presentation: None,
            });
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn langzhi_next_reveal(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    while !machine.targets.is_empty() {
        let t = machine.targets.remove(0);
        if ctx.core.player(t).alive && !he_cards(ctx.core, t).is_empty() {
            machine.target = Some(t);
            set_flag(ctx.core, machine.owner, "dddlangzhi_rev_len", machine.cards.len() as u32);
            let mut shuffled = ctx.core.player(t).hand.clone();
            if shuffled.len() > 1 {
                ctx.core.rng.shuffle(&mut shuffled);
            }
            machine.cards.extend(shuffled);
            return wait_ask_fix(machine, ctx, HostStage::SelectCard, 1067);
        }
    }
    if machine.cards.is_empty() {
        return Flow::Done;
    }
    if he_cards(ctx.core, machine.owner).is_empty() {
        langzhi_cancel_and_gain_all(machine, ctx, machine.owner);
        return Flow::Done;
    }
    wait_ask_fix(machine, ctx, HostStage::CardsAsk, 1068)
}

fn step_fuzong(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = machine.owner;
    match machine.stage {
        HostStage::Act => {
            if machine.card == Some(0) {
                set_flag(ctx.core, owner, "dddfuzong_init_done", 1);
                let qun_count = ctx
                    .core
                    .alive_players()
                    .filter(|&p| {
                        ctx.core.effective_faction(ctx.catalog, p) == Some(crate::catalog::Faction::Qun)
                    })
                    .count();
                if qun_count > 0 {
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "dddfuzong",
                        owner,
                        presentation: None,
                    });
                    ctx.start_draw(owner, qun_count as u32);
                }
                Flow::Done
            } else {
                let Some(t) = machine.target else {
                    return Flow::Done;
                };
                if he_cards(ctx.core, owner).is_empty() {
                    return Flow::Done;
                }
                let mut shuffled = ctx.core.player(owner).hand.clone();
                if shuffled.len() > 1 {
                    ctx.core.rng.shuffle(&mut shuffled);
                }
                machine.cards = shuffled;
                let _ = t;
                wait_ask_fix(machine, ctx, HostStage::SelectCard, 1070)
            }
        }
        HostStage::SelectCard => {
            let picked = match ctx.answer().first().copied() {
                Some(ChoiceOption::HiddenCard(slot)) => machine
                    .cards
                    .get(slot as usize)
                    .copied()
                    .filter(|c| ctx.core.player(owner).hand.contains(c)),
                Some(ChoiceOption::Card(c)) if ctx.core.player(owner).equips.contains(&c) => Some(c),
                _ => None,
            };
            if let Some(c) = picked {
                u01_discard_one(ctx, owner, c);
            }
            ctx.log(LogEvent::SkillTriggered {
                skill: "dddfuzong",
                owner,
                presentation: None,
            });
            Flow::Done
        }
        _ => Flow::Done,
    }
}
