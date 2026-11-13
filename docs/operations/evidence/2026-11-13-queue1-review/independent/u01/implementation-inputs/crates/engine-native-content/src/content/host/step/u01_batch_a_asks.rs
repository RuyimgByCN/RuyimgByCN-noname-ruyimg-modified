use super::*;

pub(crate) const ASK_KEY: &str = "u01_ask_id";

pub(crate) fn flag(core: &GameStateCore, p: PlayerId, k: &str) -> u32 {
    core.vm_aux
        .flags
        .get(&(p, k.to_string()))
        .copied()
        .unwrap_or(0)
}

pub(crate) fn set_flag(core: &mut GameStateCore, p: PlayerId, k: &str, v: u32) {
    core.vm_aux.flags.insert((p, k.to_string()), v);
}

pub(crate) fn turn_stamp(core: &GameStateCore) -> u32 {
    core.turn.round.saturating_mul(100) + u32::from(core.turn.current) + 1
}

pub(crate) fn turn_flag(core: &GameStateCore, p: PlayerId, k: &str, stamp_key: &str) -> u32 {
    let st = flag(core, p, stamp_key);
    if st != 0 && st != turn_stamp(core) {
        0
    } else {
        flag(core, p, k)
    }
}

pub(crate) fn set_turn_flag(
    core: &mut GameStateCore,
    p: PlayerId,
    k: &str,
    stamp_key: &str,
    v: u32,
) {
    let cur = turn_stamp(core);
    if flag(core, p, stamp_key) != cur {
        set_flag(core, p, stamp_key, cur);
    }
    set_flag(core, p, k, v);
}

pub(crate) fn is_red(catalog: &Catalog, card: CardId) -> bool {
    matches!(
        catalog.cards[card as usize].suit,
        Suit::Heart | Suit::Diamond
    )
}

pub(crate) fn suit_bit(catalog: &Catalog, card: CardId) -> u32 {
    match catalog.cards[card as usize].suit {
        Suit::Spade => 1,
        Suit::Heart => 2,
        Suit::Club => 4,
        Suit::Diamond => 8,
    }
}

pub(crate) fn distinct_suits(cards: &[CardId], catalog: &Catalog) -> usize {
    let mut mask = 0u32;
    for &c in cards {
        mask |= suit_bit(catalog, c);
    }
    mask.count_ones() as usize
}

pub(crate) fn he_cards(core: &GameStateCore, p: PlayerId) -> Vec<CardId> {
    let pl = core.player(p);
    let mut out: Vec<CardId> = pl.hand.iter().chain(pl.equips.iter()).copied().collect();
    out.dedup();
    out
}

pub(crate) fn hand_options(core: &GameStateCore, p: PlayerId) -> Vec<ChoiceOption> {
    core.player(p)
        .hand
        .iter()
        .copied()
        .map(ChoiceOption::Card)
        .collect()
}

pub(crate) fn he_options(core: &GameStateCore, p: PlayerId) -> Vec<ChoiceOption> {
    he_cards(core, p)
        .into_iter()
        .map(ChoiceOption::Card)
        .collect()
}

pub(crate) fn slot_occupied(
    core: &GameStateCore,
    catalog: &Catalog,
    p: PlayerId,
    slot: EquipSlot,
) -> bool {
    core.player(p)
        .equips
        .iter()
        .any(|&c| catalog.card_slot_of(core, c) == Some(slot))
}

pub(crate) fn movable_equips(core: &GameStateCore, catalog: &Catalog) -> Vec<ChoiceOption> {
    let mut out = Vec::new();
    for src in core.alive_players() {
        for &c in &core.player(src).equips {
            let Some(slot) = catalog.card_slot_of(core, c) else {
                continue;
            };
            if core
                .alive_players()
                .any(|dst| dst != src && !slot_occupied(core, catalog, dst, slot))
            {
                out.push(ChoiceOption::Card(c));
            }
        }
    }
    out
}

pub(crate) fn zhishu_recipients(
    core: &GameStateCore,
    catalog: &Catalog,
    equip: Option<CardId>,
    src: Option<PlayerId>,
) -> Vec<ChoiceOption> {
    let Some(c) = equip else {
        return Vec::new();
    };
    let Some(slot) = catalog.card_slot_of(core, c) else {
        return Vec::new();
    };
    core.alive_players()
        .filter(|&p| Some(p) != src && !slot_occupied(core, catalog, p, slot))
        .map(ChoiceOption::Player)
        .collect()
}

pub(crate) fn juxian_mode_options(core: &GameStateCore, owner: PlayerId) -> Vec<ChoiceOption> {
    let top_used = turn_flag(core, owner, "dddjuxian_top_used", "dddjuxian_stamp") > 0;
    let hand_used = turn_flag(core, owner, "dddjuxian_hand_used", "dddjuxian_stamp") > 0;
    let has_other_he = core
        .alive_players()
        .any(|p| p != owner && !he_cards(core, p).is_empty());
    let has_other = core.alive_players().any(|p| p != owner);
    let mut out = Vec::new();
    if !top_used && has_other_he {
        out.push(ChoiceOption::Tag(0));
    }
    if !hand_used && has_other {
        out.push(ChoiceOption::Tag(1));
    }
    out
}

pub(crate) fn juxian_can_activate(core: &GameStateCore, owner: PlayerId) -> bool {
    if turn_flag(core, owner, "dddjuxian_ban", "dddjuxian_stamp") > 0 {
        return false;
    }
    if turn_flag(core, owner, "dddjuxian_used", "dddjuxian_stamp") >= 2 {
        return false;
    }
    !he_cards(core, owner).is_empty() && !juxian_mode_options(core, owner).is_empty()
}

pub(crate) fn first_wu_responder(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Option<PlayerId> {
    core.alive_players().find(|&p| {
        p != owner
            && !core.player(p).hand.is_empty()
            && core.player(p).general_slots[0]
                .and_then(|g| catalog.general_meta(g))
                .is_some_and(|(_, fac, _, _, _)| fac == Faction::Wu)
    })
}

pub(crate) fn u01_draw_now(ctx: &mut Ctx<'_>, player: PlayerId, count: usize) -> Vec<CardId> {
    let mut drawn = Vec::new();
    for _ in 0..count {
        if ctx.core.deck.is_empty() && !ctx.core.discard.is_empty() {
            let mut cards = std::mem::take(&mut ctx.core.discard);
            ctx.core.rng.shuffle(&mut cards);
            crate::kernel::elo_on_wash_deck(ctx.core, &cards);
            ctx.core.deck = cards;
            ctx.out.push(LogEvent::DeckReshuffled);
        }
        if let Some(c) = ctx.core.deck.pop() {
            ctx.core.player_mut(player).hand.push(c);
            drawn.push(c);
        }
    }
    if !drawn.is_empty() {
        ctx.log(LogEvent::CardsDrawn {
            player,
            cards: drawn.clone(),
        });
    }
    drawn
}

pub(crate) fn u01_remove_owned(ctx: &mut Ctx<'_>, player: PlayerId, card: CardId) -> bool {
    if ctx.core.player(player).hand.contains(&card) {
        ctx.take_hand_card(player, card);
        ctx.note_cards_lost(player, &[card]);
        ctx.emit(EventData::CardsLost {
            player,
            cards: vec![card],
            origins: vec![crate::event::LostOrigin::Hand],
        });
        return true;
    }
    if ctx.core.player(player).equips.contains(&card) {
        ctx.core.player_mut(player).equips.retain(|&x| x != card);
        ctx.core.declared_slots.remove(&card);
        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, player, card);
        ctx.note_cards_lost(player, &[card]);
        ctx.emit(EventData::CardsLost {
            player,
            cards: vec![card],
            origins: vec![crate::event::LostOrigin::Equip],
        });
        return true;
    }
    if ctx.core.player(player).judge.contains(&card) {
        ctx.core.player_mut(player).judge.retain(|&x| x != card);
        ctx.core.declared_kinds.remove(&card);
        ctx.note_cards_lost(player, &[card]);
        ctx.emit(EventData::CardsLost {
            player,
            cards: vec![card],
            origins: vec![crate::event::LostOrigin::Judge],
        });
        return true;
    }
    false
}

pub(crate) fn u01_discard_one(ctx: &mut Ctx<'_>, player: PlayerId, card: CardId) {
    if u01_remove_owned(ctx, player, card) {
        ctx.to_discard(card);
        ctx.log(LogEvent::CardDiscarded {
            player,
            cards: vec![card],
        });
    }
}

pub(crate) fn build_ask(
    ask_id: u32,
    owner: PlayerId,
    target: Option<PlayerId>,
    card: Option<u32>,
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<(PlayerId, ChoiceSpec)> {
    match ask_id {
        1 => {
            let t = target.unwrap_or(owner);
            let options = he_options(core, t);
            (!options.is_empty()).then_some((
                t,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddyuanzhen_discard",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        2 => {
            let options = movable_equips(core, catalog);
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddzhishu_card",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        3 => {
            let options = zhishu_recipients(core, catalog, card, target);
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddzhishu_target",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        4 => {
            let options = juxian_mode_options(core, owner);
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::PickTag,
                    prompt: "dddjuxian_mode",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        5 => {
            let options = he_options(core, owner);
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddjuxian_card",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        6 => {
            let need_he = card == Some(0);
            let options: Vec<ChoiceOption> = core
                .alive_players()
                .filter(|&p| p != owner && (!need_he || !he_cards(core, p).is_empty()))
                .map(ChoiceOption::Player)
                .collect();
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddjuxian_target",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        7 => {
            let options: Vec<ChoiceOption> =
                core.alive_players().map(ChoiceOption::Player).collect();
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddjungui_target",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        8 => {
            let max_tag = card.unwrap_or(1) as u8;
            let options: Vec<ChoiceOption> = (0..=max_tag).map(ChoiceOption::Tag).collect();
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::PickTag,
                    prompt: "dddjungui_reduce",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        9 => {
            let t = target.unwrap_or(owner);
            let rem = card.unwrap_or(1) as usize;
            let options = hand_options(core, t);
            (options.len() >= rem && rem > 0).then_some((
                t,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddjungui_discard",
                    options,
                    min: rem,
                    max: rem,
                    allow_cancel: false,
                },
            ))
        }
        10 => {
            let options = hand_options(core, owner);
            let max = options.len();
            (max > 0).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddshilie_cards",
                    options,
                    min: 1,
                    max,
                    allow_cancel: true,
                },
            ))
        }
        11 => {
            let options: Vec<ChoiceOption> = core
                .alive_players()
                .filter(|&p| p != owner)
                .map(ChoiceOption::Player)
                .collect();
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddshilie_target",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        12 => Some((owner, ChoiceSpec::confirm("dddyeshen_confirm"))),
        13 => {
            let t = target.unwrap_or(owner);
            let options: Vec<ChoiceOption> =
                core.alive_players().map(ChoiceOption::Player).collect();
            let max = options.len().min(2);
            (max > 0).then_some((
                t,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddyeshen_target",
                    options,
                    min: 1,
                    max,
                    allow_cancel: true,
                },
            ))
        }
        14 => {
            let num = card.unwrap_or(1) as usize;
            let options = he_options(core, owner);
            (options.len() >= num && num > 0).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddqiaoduan_cards",
                    options,
                    min: num,
                    max: num,
                    allow_cancel: true,
                },
            ))
        }
        15 => {
            let options: Vec<ChoiceOption> = core
                .alive_players()
                .filter(|&p| core.player(p).hp < core.player(p).max_hp as i32)
                .map(ChoiceOption::Player)
                .collect();
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddqiaoduan_recover_target",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        16 => {
            let num = card.unwrap_or(1) as usize;
            let options: Vec<ChoiceOption> =
                core.alive_players().map(ChoiceOption::Player).collect();
            let max = options.len().min(num);
            (max > 0).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddqiaoduan_draw_targets",
                    options,
                    min: 1,
                    max,
                    allow_cancel: true,
                },
            ))
        }
        17 => {
            let options = he_options(core, owner);
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "ddddiedang_discard",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        18 => {
            let options = he_options(core, owner);
            (options.len() >= 3).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "ddddiedang_discard",
                    options,
                    min: 3,
                    max: 3,
                    allow_cancel: true,
                },
            ))
        }
        19 => {
            let options: Vec<ChoiceOption> = core
                .alive_players()
                .filter(|&p| p != owner && !core.player(p).hand.is_empty())
                .map(ChoiceOption::Player)
                .collect();
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddanliu_target",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        20 => {
            let t = target.unwrap_or(owner);
            let options = hand_options(core, t);
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddanliu_pick_target",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        21 => {
            let t = target.unwrap_or(owner);
            let options = hand_options(core, owner);
            (!options.is_empty()).then_some((
                t,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddanliu_pick_owner",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        22 => Some((owner, ChoiceSpec::confirm("dddanliu_swap_confirm"))),
        23 => {
            let t = target.unwrap_or(owner);
            Some((t, ChoiceSpec::confirm("dddguiying_confirm")))
        }
        24 => {
            let diff = card.unwrap_or(1) as usize;
            let options = hand_options(core, owner);
            (options.len() >= diff && diff > 0).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddbingjian_discard",
                    options,
                    min: diff,
                    max: diff,
                    allow_cancel: true,
                },
            ))
        }
        25 => {
            let options: Vec<ChoiceOption> = core
                .alive_players()
                .filter(|&p| p != owner)
                .map(ChoiceOption::Player)
                .collect();
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddbingjian_partner",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        26 => {
            let options = hand_options(core, owner);
            let max = options.len().min(4);
            (max > 0).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddtongyu_cards",
                    options,
                    min: 1,
                    max,
                    allow_cancel: true,
                },
            ))
        }
        27 => {
            let t = target.unwrap_or(owner);
            Some((
                t,
                ChoiceSpec {
                    kind: ChoiceKind::PickTag,
                    prompt: "dddtongyu_target_choice",
                    options: vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        28 => {
            let player = core.player(target?);
            let options =
                crate::event::qinglong_options(owner, target?, &player.hand, &player.equips);
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddjuxian_gain",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        _ => None,
    }
}

pub(crate) fn wait_ask(
    machine: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    stage: HostStage,
    ask_id: u32,
) -> Flow {
    set_flag(ctx.core, machine.owner, ASK_KEY, ask_id);
    if let Some((observer, spec)) = build_ask(
        ask_id,
        machine.owner,
        machine.target,
        machine.card,
        ctx.core,
        ctx.catalog,
    ) {
        machine.stage = stage;
        Flow::WaitChoice { observer, spec }
    } else {
        Flow::Done
    }
}

pub(crate) fn active_ask_u01_batch_a(
    _stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    _targets: &[PlayerId],
    card: Option<u32>,
    _cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    let ask_id = flag(core, owner, ASK_KEY);
    let (observer, spec) = build_ask(ask_id, owner, target, card, core, catalog)?;
    Some(crate::snap::ask(
        observer,
        spec.prompt,
        spec.kind,
        spec.options,
        spec.min,
        spec.max,
        spec.allow_cancel,
    ))
}

pub(crate) fn trigger_meta_u01_batch_a(name: &str) -> Option<TriggerMeta> {
    let meta = |event: EventKind, extra_events: Vec<EventKind>| TriggerMeta {
        event,
        timing: Timing::Normal,
        priority: 1,
        order_selectable: false,
        optional: false,
        extra_events,
    };
    match name {
        "dddyuanzhen" => Some(meta(EventKind::CardUsed, Vec::new())),
        "dddzhishu" | "dddyeshen" | "dddanliu" | "dddguiying" => {
            Some(meta(EventKind::PhaseBegin, Vec::new()))
        }
        "dddjungui" => Some(meta(EventKind::PhaseBegin, vec![EventKind::CardUsed])),
        "dddqiaoduan" => Some(meta(EventKind::Linked, Vec::new())),
        _ => None,
    }
}

pub(crate) fn limits_u01_batch_a(name: &str) -> crate::catalog::SkillLimits {
    match name {
        "dddshilie" | "ddddiedang" | "dddtongyu" => crate::catalog::SkillLimits {
            once_per_turn: true,
            ..Default::default()
        },
        _ => crate::catalog::SkillLimits::default(),
    }
}

pub(crate) fn filter_u01_batch_a(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    if !ctx.state.player(owner).alive {
        return false;
    }
    match skill.name.as_str() {
        "dddyuanzhen" => matches!(
            event,
            EventData::CardUsed(u)
            if u.player == owner
                && !u.cancelled
                && u.targets.len() == 1
                && u.targets[0] != owner
                && ctx.state.player(u.targets[0]).alive
                && ctx.state.computed_distance(ctx.catalog, owner, u.targets[0]) != 1
        ),
        "dddzhishu" => matches!(
            event,
            EventData::Phase { player, phase: Phase::Play, begin: true }
            if *player == owner && !movable_equips(ctx.state, ctx.catalog).is_empty()
        ),
        "dddjungui" => match event {
            EventData::CardUsed(u) => u.player == owner && !u.cancelled && u.card.is_some(),
            EventData::Phase {
                player,
                phase: Phase::End,
                begin: true,
            } => *player == owner,
            _ => false,
        },
        "dddyeshen" => matches!(
            event,
            EventData::Phase { phase: Phase::End, begin: true, .. }
            if !ctx.state.deck.is_empty() || !ctx.state.discard.is_empty()
        ),
        "dddqiaoduan" => match event {
            EventData::Linked { linked, .. } => {
                if turn_flag(
                    ctx.state,
                    owner,
                    "dddqiaoduan_used_turn",
                    "dddqiaoduan_stamp",
                ) > 0
                {
                    return false;
                }
                let num = ctx
                    .state
                    .alive_players()
                    .filter(|&p| ctx.state.player(p).linked)
                    .count();
                if num == 0 {
                    return false;
                }
                if *linked {
                    he_cards(ctx.state, owner).len() >= num
                        && ctx
                            .state
                            .alive_players()
                            .any(|p| ctx.state.player(p).hp < ctx.state.player(p).max_hp as i32)
                } else {
                    true
                }
            }
            _ => false,
        },
        "dddanliu" => matches!(
            event,
            EventData::Phase { player, phase: Phase::End, begin: true }
            if *player == owner
                && !ctx.state.player(owner).hand.is_empty()
                && ctx.state.alive_players().any(|p| p != owner && !ctx.state.player(p).hand.is_empty())
        ),
        "dddguiying" => matches!(
            event,
            EventData::Phase { player, phase: Phase::Prepare, begin: true }
            if *player == owner && first_wu_responder(ctx.state, ctx.catalog, owner).is_some()
        ),
        _ => false,
    }
}

pub(crate) fn activate_u01_batch_a(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<Box<dyn crate::catalog::SkillMachine>> {
    if !ctx.state.player(owner).alive
        || ctx.state.turn.phase != Phase::Play
        || ctx.state.turn.current != owner
    {
        return None;
    }
    let ok = match skill.name.as_str() {
        "dddjuxian" => juxian_can_activate(ctx.state, owner),
        "dddshilie" => {
            let hand = &ctx.state.player(owner).hand;
            let sum: u32 = hand
                .iter()
                .map(|&c| u32::from(ctx.catalog.cards[c as usize].rank))
                .sum();
            let need = (ctx.state.player(owner).hp + ctx.state.player(ctx.state.turn.current).hp)
                .max(1) as u32;
            !hand.is_empty() && sum >= need && ctx.state.alive_players().any(|p| p != owner)
        }
        "ddddiedang" => {
            let st = flag(ctx.state, owner, "ddddiedang_state");
            st == 0 || he_cards(ctx.state, owner).len() >= 3
        }
        "dddbingjian" => {
            turn_flag(
                ctx.state,
                owner,
                "dddbingjian_used_turn",
                "dddbingjian_stamp",
            ) < 2
                && ctx.state.player(owner).hand.len() != 2
        }
        "dddtongyu" => !ctx.state.player(owner).hand.is_empty(),
        _ => false,
    };
    ok.then(|| {
        Box::new(HostMachine {
            def_key: skill.name.clone(),
            behavior: skill.behavior,
            owner,
            target: None,
            targets: Vec::new(),
            card: None,
            cards: Vec::new(),
            stage: HostStage::Act,
        }) as Box<dyn crate::catalog::SkillMachine>
    })
}

pub(crate) fn machine_u01_batch_a(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    let mut target = None;
    let mut card = None;
    let mut cards = Vec::new();
    match (skill.name.as_str(), event) {
        ("dddyuanzhen", EventData::CardUsed(u)) => {
            target = u.targets.first().copied();
        }
        ("dddjungui", EventData::CardUsed(u)) => {
            card = Some(1);
            if let Some(c) = u.card {
                cards.push(c);
            }
        }
        ("dddyeshen", EventData::Phase { player, .. }) => {
            target = Some(*player);
        }
        ("dddqiaoduan", EventData::Linked { linked, .. }) => {
            card = Some(u32::from(*linked));
        }
        _ => {}
    }
    HostMachine {
        def_key: skill.name.clone(),
        behavior: skill.behavior,
        owner,
        target,
        targets: Vec::new(),
        card,
        cards,
        stage: HostStage::Act,
    }
}
