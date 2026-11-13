use super::*;

pub(crate) fn is_male(core: &GameStateCore, catalog: &Catalog, p: PlayerId) -> bool {
    match core
        .player(p)
        .general_slots
        .iter()
        .flatten()
        .next()
        .and_then(|&g| catalog.general_meta(g))
    {
        Some((_, _, _, _, gender)) => gender == crate::vocabulary::generals::Gender::Male,
        None => true,
    }
}

pub(crate) fn is_black_basic(catalog: &Catalog, card: CardId) -> bool {
    !is_red(catalog, card)
        && matches!(
            catalog.card_kind(card),
            CardKind::Sha
                | CardKind::FireSha
                | CardKind::LeiSha
                | CardKind::Shan
                | CardKind::Tao
                | CardKind::Jiu
        )
}

pub(crate) fn has_bingliang(core: &GameStateCore, catalog: &Catalog, p: PlayerId) -> bool {
    core.player(p).judge.iter().any(|&jc| {
        core.declared_kinds
            .get(&jc)
            .copied()
            .unwrap_or_else(|| catalog.card_kind(jc))
            == CardKind::Bing
    })
}

pub(crate) fn dddzhilian_partner(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Option<PlayerId> {
    let mask = turn_flag(core, owner, "dddzhilian_mask", "dddzhilian_stamp");
    if mask.count_ones() == 2 && (mask & (1 << owner)) != 0 {
        for p in core.alive_players() {
            if p != owner && (mask & (1 << p)) != 0 && is_male(core, catalog, p) {
                return Some(p);
            }
        }
    }
    let partner_flag = flag(core, owner, "dddzhilian_partner");
    if partner_flag > 0 {
        let p = (partner_flag - 1) as PlayerId;
        if p != owner
            && (p as usize) < core.players.len()
            && core.player(p).alive
            && is_male(core, catalog, p)
        {
            return Some(p);
        }
    }
    let cur = core.turn.current;
    if flag(core, owner, "dddzhilian_active") > 0
        && cur != owner
        && core.player(cur).alive
        && is_male(core, catalog, cur)
    {
        return Some(cur);
    }
    None
}

pub(crate) fn dddzhengjun_match_cat(
    core: &GameStateCore,
    owner: PlayerId,
    event: &EventData,
) -> Option<u32> {
    if core.turn.current != owner || core.turn.phase != Phase::Play {
        return None;
    }
    let hp_ok = turn_flag(core, owner, "dddzhengjun_hp_used", "dddzhengjun_stamp") == 0
        && core
            .alive_players()
            .any(|p| p != owner && core.player(p).hp == core.player(owner).hp);
    let hs_ok = turn_flag(core, owner, "dddzhengjun_hs_used", "dddzhengjun_stamp") == 0
        && core
            .alive_players()
            .any(|p| p != owner && core.player(p).hand.len() == core.player(owner).hand.len());
    let es_ok = turn_flag(core, owner, "dddzhengjun_es_used", "dddzhengjun_stamp") == 0
        && core
            .alive_players()
            .any(|p| p != owner && core.player(p).equips.len() == core.player(owner).equips.len());
    match event {
        EventData::Damage(_) | EventData::Recover { .. } | EventData::LoseHp { .. } => {
            hp_ok.then_some(1)
        }
        EventData::CardsDrawn { .. } | EventData::CardsMoved { .. } => hs_ok.then_some(2),
        EventData::Equipped { .. } => es_ok.then_some(3),
        EventData::CardsLost { origins, .. } => {
            if origins.contains(&LostOrigin::Equip) {
                if es_ok {
                    Some(3)
                } else if hs_ok {
                    Some(2)
                } else {
                    None
                }
            } else if hs_ok {
                Some(2)
            } else if es_ok {
                Some(3)
            } else {
                None
            }
        }
        _ => None,
    }
}

pub(crate) fn build_ask_b(
    ask_id: u32,
    owner: PlayerId,
    target: Option<PlayerId>,
    _card: Option<u32>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<(PlayerId, ChoiceSpec)> {
    match ask_id {
        101 => {
            let t = target.unwrap_or(owner);
            let mut options = vec![ChoiceOption::Tag(0)];
            if !he_cards(core, owner).is_empty() || !he_cards(core, t).is_empty() {
                options.push(ChoiceOption::Tag(1));
            }
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::PickTag,
                    prompt: "dddzhilian_mode",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        102 => {
            let options: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| catalog.card_kind(c).is_sha_like())
                .map(ChoiceOption::Card)
                .collect();
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddjijian_sha",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        103 => {
            let options: Vec<ChoiceOption> = core
                .alive_players()
                .filter(|&p| p != owner && is_male(core, catalog, p))
                .map(ChoiceOption::Player)
                .collect();
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddjijian_target",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        104 => {
            let t = target.unwrap_or(owner);
            let options = hand_options(core, t);
            let max = options.len().min(2);
            (max > 0).then_some((
                t,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddjijian_show",
                    options,
                    min: 1,
                    max,
                    allow_cancel: false,
                },
            ))
        }
        105 => {
            let options: Vec<ChoiceOption> = core
                .alive_players()
                .filter(|&p| p != owner && is_male(core, catalog, p) && !core.player(p).hand.is_empty())
                .map(ChoiceOption::Player)
                .collect();
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddjijian_old_target",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        106 => {
            let t = target.unwrap_or(owner);
            let options = hand_options(core, t);
            let max = options.len().min(2);
            (max > 0).then_some((
                t,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddjijian_old_show",
                    options,
                    min: 1,
                    max,
                    allow_cancel: false,
                },
            ))
        }
        107 => {
            let options: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| catalog.card_kind(c).is_sha_like())
                .map(ChoiceOption::Card)
                .collect();
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddjijian_old_give_sha",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        108 => Some((
            owner,
            ChoiceSpec {
                kind: ChoiceKind::PickTag,
                prompt: "dddzhengjun_hp_choice",
                options: vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
                min: 1,
                max: 1,
                allow_cancel: true,
            },
        )),
        109 => Some((
            owner,
            ChoiceSpec {
                kind: ChoiceKind::PickTag,
                prompt: "dddzhengjun_hs_choice",
                options: vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
                min: 1,
                max: 1,
                allow_cancel: true,
            },
        )),
        110 => Some((owner, ChoiceSpec::confirm("dddzhengjun_es_confirm"))),
        111 => {
            let excl_mask = flag(core, owner, "dddxianxi_excl_mask");
            let options: Vec<ChoiceOption> = core
                .alive_players()
                .filter(|&p| p != owner && (excl_mask & (1 << p)) == 0)
                .map(ChoiceOption::Player)
                .collect();
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddxianxi_extra_target",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        112 => {
            let options = he_options(core, owner);
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddxianxi_discard",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        113 => Some((owner, ChoiceSpec::confirm("dddlingyong_confirm"))),
        114 => Some((owner, ChoiceSpec::confirm("dddlingyong_repeat"))),
        115 => {
            let source_cards: Vec<CardId> = if !cards.is_empty() {
                cards.to_vec()
            } else {
                core.discard.clone()
            };
            let mut options = Vec::new();
            for c in source_cards {
                if core.discard.contains(&c)
                    && is_black_basic(catalog, c)
                    && !options.contains(&ChoiceOption::Card(c))
                {
                    options.push(ChoiceOption::Card(c));
                }
            }
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddxuxiao_card",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        116 => {
            let options: Vec<ChoiceOption> = core
                .alive_players()
                .filter(|&p| !has_bingliang(core, catalog, p))
                .map(ChoiceOption::Player)
                .collect();
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddxuxiao_target",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        117 => {
            let options = he_options(core, owner);
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddxuyu_discard",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        118 => Some((owner, ChoiceSpec::confirm("dddxuyu_confirm"))),
        119 => {
            let options = he_options(core, owner);
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddshijian_give",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        120 => Some((
            owner,
            ChoiceSpec {
                kind: ChoiceKind::PickTag,
                prompt: "dddshijian_mode",
                options: vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
                min: 1,
                max: 1,
                allow_cancel: false,
            },
        )),
        121 => Some((owner, ChoiceSpec::confirm("dddxuyu_old_confirm"))),
        122 => {
            let options = he_options(core, owner);
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddxuyu_old_discard",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        123 => Some((owner, ChoiceSpec::confirm("dddshijian_old_confirm"))),
        124 => {
            let options: Vec<ChoiceOption> =
                core.alive_players().map(ChoiceOption::Player).collect();
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddtaisi_target",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        125 => {
            let options: Vec<ChoiceOption> =
                core.discard.iter().copied().map(ChoiceOption::Card).collect();
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddtaisi_card",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        126 => {
            let options: Vec<ChoiceOption> = he_cards(core, owner)
                .into_iter()
                .filter(|&c| is_red(catalog, c))
                .map(ChoiceOption::Card)
                .collect();
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddquche_red_card",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        127 => {
            let options: Vec<ChoiceOption> = core
                .alive_players()
                .filter(|&p| p != owner)
                .map(ChoiceOption::Player)
                .collect();
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddquche_give_target",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        128 => {
            let options: Vec<ChoiceOption> = he_cards(core, owner)
                .into_iter()
                .filter(|&c| !is_red(catalog, c))
                .map(ChoiceOption::Card)
                .collect();
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddquche_black_sha",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        129 => {
            let options = hand_options(core, owner);
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "ddddongcha_owner_put",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        130 => {
            let t = target.unwrap_or(owner);
            let options = hand_options(core, t);
            (!options.is_empty()).then_some((
                t,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "ddddongcha_target_put",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        131 => {
            let options: Vec<ChoiceOption> = core
                .vm_aux
                .piles
                .get(&(owner, "ddd_jian".into()))
                .map(|v| v.iter().copied().map(ChoiceOption::Card).collect())
                .unwrap_or_default();
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "ddddongcha_owner_take",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        132 => {
            let options: Vec<ChoiceOption> = core
                .vm_aux
                .piles
                .get(&(owner, "ddd_jian".into()))
                .map(|v| v.iter().copied().map(ChoiceOption::Card).collect())
                .unwrap_or_default();
            (options.len() >= 2).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddzhijie_cards",
                    options,
                    min: 2,
                    max: 2,
                    allow_cancel: true,
                },
            ))
        }
        _ => None,
    }
}

pub(crate) fn wait_ask_b(
    machine: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    stage: HostStage,
    ask_id: u32,
) -> Flow {
    set_flag(ctx.core, machine.owner, ASK_KEY, ask_id);
    if let Some((observer, spec)) = build_ask_b(
        ask_id,
        machine.owner,
        machine.target,
        machine.card,
        &machine.cards,
        ctx.core,
        ctx.catalog,
    ) {
        machine.stage = stage;
        Flow::WaitChoice { observer, spec }
    } else {
        Flow::Done
    }
}

pub(crate) fn active_ask_u01_batch_b(
    _stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    _targets: &[PlayerId],
    card: Option<u32>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    let ask_id = flag(core, owner, ASK_KEY);
    let (observer, spec) = build_ask_b(ask_id, owner, target, card, cards, core, catalog)?;
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

pub(crate) fn trigger_meta_u01_batch_b(name: &str) -> Option<TriggerMeta> {
    let meta = |event: EventKind, extra_events: Vec<EventKind>| TriggerMeta {
        event,
        timing: Timing::Normal,
        priority: 1,
        order_selectable: false,
        optional: false,
        extra_events,
    };
    match name {
        "dddzhilian" => Some(meta(
            EventKind::PhaseBegin,
            vec![EventKind::CardUsed, EventKind::CardResponded],
        )),
        "dddzhengjun" => Some(meta(
            EventKind::Damage,
            vec![
                EventKind::Recover,
                EventKind::LoseHp,
                EventKind::CardsLost,
                EventKind::CardsDrawn,
                EventKind::CardsMoved,
                EventKind::Equipped,
            ],
        )),
        "dddxianxi" | "dddxuyu" | "dddshijian" | "dddxuyu_old" | "dddshijian_old" => {
            Some(meta(EventKind::CardUsed, Vec::new()))
        }
        "dddlingyong" | "ddddongcha" | "dddzhijie" => {
            Some(meta(EventKind::PhaseBegin, Vec::new()))
        }
        "dddxuxiao" => Some(meta(EventKind::CardsLost, vec![EventKind::CardsDiscarded])),
        "dddtaisi" => Some(meta(
            EventKind::PhaseBegin,
            vec![EventKind::Damage, EventKind::Recover, EventKind::LoseHp],
        )),
        "dddqianlong" => Some(meta(EventKind::Death, Vec::new())),
        _ => None,
    }
}

pub(crate) fn limits_u01_batch_b(name: &str) -> crate::catalog::SkillLimits {
    match name {
        "dddjijian" | "dddjijian_old" | "dddzhijie" => crate::catalog::SkillLimits {
            once_per_turn: true,
            ..Default::default()
        },
        _ => crate::catalog::SkillLimits::default(),
    }
}

pub(crate) fn filter_u01_batch_b(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    if !ctx.state.player(owner).alive {
        return false;
    }
    match skill.name.as_str() {
        "dddzhilian" => match event {
            EventData::CardUsed(u) => !u.cancelled,
            EventData::CardResponded { .. } => true,
            EventData::Phase {
                phase: Phase::End,
                begin: true,
                ..
            } => dddzhilian_partner(ctx.state, ctx.catalog, owner).is_some(),
            _ => false,
        },
        "dddzhengjun" => dddzhengjun_match_cat(ctx.state, owner, event).is_some(),
        "dddxianxi" => matches!(
            event,
            EventData::CardUsed(u)
            if u.player == owner
                && !u.cancelled
                && ctx.state.turn.current == owner
                && ctx.state.turn.phase == Phase::Play
                && (u.use_as.is_some_and(|k| k.is_sha_like())
                    || u.card.is_some_and(|c| ctx.catalog.card_kind(c).is_sha_like()))
                && ctx.state.alive_players().any(|p| p != owner && !u.targets.contains(&p))
        ),
        "dddlingyong" => match event {
            EventData::Phase {
                player,
                phase,
                begin: true,
            } => {
                if *player != owner && flag(ctx.state, owner, "dddlingyong_trigger") == 0 {
                    return false;
                }
                if ctx.state.deck.is_empty() && ctx.state.discard.is_empty() {
                    return false;
                }
                match phase {
                    Phase::Prepare => {
                        !ctx.state.player(owner).judge.is_empty()
                            || flag(ctx.state, owner, "dddlingyong_trigger") > 0
                    }
                    Phase::Judge | Phase::Draw | Phase::Play => {
                        ctx.state.turn.skip_draw_phase
                            || ctx.state.turn.skip_play_phase
                            || flag(ctx.state, owner, "dddlingyong_trigger") > 0
                    }
                    _ => flag(ctx.state, owner, "dddlingyong_trigger") > 0,
                }
            }
            _ => false,
        },
        "dddxuxiao" => match event {
            EventData::CardsLost { cards, .. } | EventData::CardsDiscarded { cards, .. } => {
                ctx.state
                    .alive_players()
                    .any(|p| !has_bingliang(ctx.state, ctx.catalog, p))
                    && cards.iter().any(|&c| {
                        ctx.state.discard.contains(&c) && is_black_basic(ctx.catalog, c)
                    })
            }
            _ => false,
        },
        "dddxuyu" => matches!(
            event,
            EventData::CardUsed(u) if u.player == owner && !u.cancelled
        ),
        "dddshijian" | "dddshijian_old" => matches!(
            event,
            EventData::CardUsed(u)
            if u.player != owner
                && !u.cancelled
                && ctx.state.turn.current == u.player
                && ctx.state.turn.phase == Phase::Play
        ),
        "dddxuyu_old" => matches!(
            event,
            EventData::CardUsed(u)
            if u.player == owner
                && !u.cancelled
                && ctx.state.turn.current == owner
                && ctx.state.turn.phase == Phase::Play
        ),
        "dddtaisi" => match event {
            EventData::Damage(d) => d.target == owner && !d.prevented && d.amount > 0,
            EventData::Recover { player, amount, .. } => *player == owner && *amount > 0,
            EventData::LoseHp { player, amount } => *player == owner && *amount > 0,
            EventData::Phase {
                phase: Phase::End,
                begin: true,
                ..
            } => {
                (turn_flag(ctx.state, owner, "dddtaisi_hp_changed", "dddtaisi_stamp") > 0
                    || flag(ctx.state, owner, "dddtaisi_hp_changed") > 0)
                    && !ctx.state.discard.is_empty()
            }
            _ => false,
        },
        "dddqianlong" => matches!(
            event,
            EventData::Death { killer: Some(k), .. } if *k == owner
        ),
        "ddddongcha" => match event {
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } => {
                *player != owner
                    && !ctx.state.player(owner).hand.is_empty()
                    && !ctx.state.player(*player).hand.is_empty()
            }
            EventData::Phase {
                phase: Phase::End,
                begin: true,
                ..
            } => !ctx
                .state
                .vm_aux
                .piles
                .get(&(owner, "ddd_jian".into()))
                .map_or(true, Vec::is_empty),
            _ => false,
        },
        "dddzhijie" => matches!(
            event,
            EventData::Phase {
                player,
                begin: true,
                ..
            } if *player == owner
                && ctx
                    .state
                    .vm_aux
                    .piles
                    .get(&(owner, "ddd_jian".into()))
                    .map_or(0, Vec::len)
                    >= 2
        ),
        _ => false,
    }
}

pub(crate) fn activate_u01_batch_b(
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
        "dddjijian" => {
            ctx.state
                .player(owner)
                .hand
                .iter()
                .any(|&c| ctx.catalog.card_kind(c).is_sha_like())
                && ctx
                    .state
                    .alive_players()
                    .any(|p| p != owner && is_male(ctx.state, ctx.catalog, p))
        }
        "dddjijian_old" => ctx.state.alive_players().any(|p| {
            p != owner
                && is_male(ctx.state, ctx.catalog, p)
                && !ctx.state.player(p).hand.is_empty()
        }),
        "dddquche" => {
            flag(ctx.state, owner, "dddquche_used") == 0
                && he_cards(ctx.state, owner)
                    .iter()
                    .any(|&c| is_red(ctx.catalog, c))
                && ctx.state.alive_players().any(|p| p != owner)
        }
        "dddzhijie" => {
            ctx.state
                .vm_aux
                .piles
                .get(&(owner, "ddd_jian".into()))
                .map_or(0, Vec::len)
                >= 2
        }
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

pub(crate) fn machine_u01_batch_b(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    let mut target = None;
    let mut targets = Vec::new();
    let mut card = None;
    let mut cards = Vec::new();
    match (skill.name.as_str(), event) {
        ("dddzhilian", EventData::CardUsed(u)) => {
            card = Some(1);
            target = Some(u.player);
        }
        ("dddzhilian", EventData::CardResponded { player, .. }) => {
            card = Some(1);
            target = Some(*player);
        }
        ("dddzhengjun", EventData::Damage(_) | EventData::Recover { .. } | EventData::LoseHp { .. }) => {
            card = Some(1);
        }
        ("dddzhengjun", EventData::CardsDrawn { .. } | EventData::CardsMoved { .. }) => {
            card = Some(2);
        }
        ("dddzhengjun", EventData::Equipped { .. }) => {
            card = Some(3);
        }
        ("dddzhengjun", EventData::CardsLost { origins, .. }) => {
            card = Some(if origins.contains(&LostOrigin::Equip) { 3 } else { 2 });
        }
        ("dddxianxi", EventData::CardUsed(u)) => {
            targets = u.targets.clone();
        }
        ("dddxuxiao", EventData::CardsLost { cards: cs, .. } | EventData::CardsDiscarded { cards: cs, .. }) => {
            cards = cs.clone();
        }
        ("dddshijian" | "dddshijian_old", EventData::CardUsed(u)) => {
            target = Some(u.player);
        }
        ("dddtaisi", EventData::Damage(d)) => {
            card = Some(1);
            target = d.source;
        }
        ("dddtaisi", EventData::Recover { .. } | EventData::LoseHp { .. }) => {
            card = Some(1);
        }
        (
            "ddddongcha",
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            },
        ) => {
            target = Some(*player);
            card = Some(0);
        }
        (
            "ddddongcha",
            EventData::Phase {
                player,
                phase: Phase::End,
                begin: true,
            },
        ) => {
            target = Some(*player);
            card = Some(1);
        }
        _ => {}
    }
    HostMachine {
        def_key: skill.name.clone(),
        behavior: skill.behavior,
        owner,
        target,
        targets,
        card,
        cards,
        stage: HostStage::Act,
    }
}
