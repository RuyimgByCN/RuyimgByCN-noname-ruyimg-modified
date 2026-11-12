use super::*;

pub(crate) fn is_trick_kind(kind: CardKind) -> bool {
    !kind.is_basic_kind() && !kind.is_equip() && kind.is_basic_trick_or_equip()
}

pub(crate) fn fusi_source(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Option<PlayerId> {
    if owner != core.turn.current && !core.player(owner).hand.is_empty() {
        return Some(owner);
    }
    let xl_def = catalog.skill_by_name("dddfusi");
    if let Some(p) = core.alive_players().find(|&p| {
        !core.player(p).hand.is_empty()
            && (xl_def.is_some_and(|d| core.skills.get(&(p, d)).is_some_and(|i| i.enabled))
                || core
                    .player(p)
                    .general_slots
                    .into_iter()
                    .flatten()
                    .any(|g| {
                        catalog
                            .general_meta(g)
                            .is_some_and(|(name, _, _, _, _)| name == "ddd_xianglang")
                    }))
    }) {
        return Some(p);
    }
    (!core.player(owner).hand.is_empty()).then_some(owner)
}

pub(crate) fn chashi_card_matches(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    u: &CardUseData,
) -> bool {
    let wanted = turn_flag(core, owner, "dddchashi_suit", "dddchashi_stamp");
    if wanted == 0 {
        return false;
    }
    let suit_ok = u.card.is_some_and(|c| (suit_bit(catalog, c) & wanted) != 0);
    let used_kind = u.use_as.or_else(|| u.card.map(|c| catalog.card_kind(c)));
    let trick_ok = turn_flag(core, owner, "dddchashi_trick", "dddchashi_stamp") > 0
        && used_kind.is_some_and(is_trick_kind);
    suit_ok || trick_ok
}

pub(crate) fn is_male_player(core: &GameStateCore, catalog: &Catalog, p: PlayerId) -> bool {
    core.player(p)
        .general_slots
        .into_iter()
        .flatten()
        .any(|g| {
            catalog
                .general_meta(g)
                .is_some_and(|(_, _, _, _, gender)| {
                    gender == crate::vocabulary::generals::Gender::Male
                })
        })
}

pub(crate) fn has_male_other(core: &GameStateCore, catalog: &Catalog, owner: PlayerId) -> bool {
    core.alive_players()
        .any(|p| p != owner && is_male_player(core, catalog, p))
}

pub(crate) fn fenji_tag_options(core: &GameStateCore, owner: PlayerId) -> Vec<ChoiceOption> {
    let mask = flag(core, owner, "dddfenji_used_mask");
    (0u8..3)
        .filter(|&i| (mask & (1 << i)) == 0)
        .map(ChoiceOption::Tag)
        .collect()
}

pub(crate) fn yulv_pile_rank(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Option<u8> {
    core.vm_aux
        .piles
        .get(&(owner, "dddyulv".to_string()))
        .and_then(|p| p.first().copied())
        .map(|c| catalog.cards[c as usize].rank)
}

pub(crate) fn build_ask_c(
    ask_id: u32,
    owner: PlayerId,
    target: Option<PlayerId>,
    card: Option<u32>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<(PlayerId, ChoiceSpec)> {
    match ask_id {
        201 => {
            let n = card.unwrap_or(1) as usize;
            let options = hand_options(core, owner);
            (options.len() >= n && n > 0).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddqiahua_show",
                    options,
                    min: n,
                    max: n,
                    allow_cancel: true,
                },
            ))
        }
        202 => {
            let p = target.unwrap_or(owner);
            let options: Vec<ChoiceOption> = cards.iter().copied().map(ChoiceOption::Card).collect();
            let n = options.len().min(2);
            (n > 0).then_some((
                p,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddxunxun_top",
                    options,
                    min: n,
                    max: n,
                    allow_cancel: true,
                },
            ))
        }
        203 => {
            let src = target.unwrap_or(owner);
            let options = hand_options(core, src);
            (!options.is_empty()).then_some((
                core.turn.current,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddfusi_pick",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        204 => {
            let src = target.unwrap_or(owner);
            Some((src, ChoiceSpec::confirm("dddfusi_agree")))
        }
        205 => Some((owner, ChoiceSpec::confirm("dddtuoji_confirm"))),
        206 => {
            let options = he_options(core, owner);
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddchashi_discard",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        207 => Some((owner, ChoiceSpec::confirm("dddqice_confirm"))),
        208 => Some((
            owner,
            ChoiceSpec {
                kind: ChoiceKind::PickTag,
                prompt: "dddmiaoxing_pile",
                options: vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
                min: 1,
                max: 1,
                allow_cancel: false,
            },
        )),
        209 => Some((
            owner,
            ChoiceSpec {
                kind: ChoiceKind::PickTag,
                prompt: "dddfushi_pile",
                options: vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
                min: 1,
                max: 1,
                allow_cancel: true,
            },
        )),
        210 => {
            let options: Vec<ChoiceOption> = core
                .alive_players()
                .filter(|&p| p != owner && is_male_player(core, catalog, p))
                .map(ChoiceOption::Player)
                .collect();
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddfushi_male_target",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        211 => {
            let options = fenji_tag_options(core, owner);
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::PickTag,
                    prompt: "dddfenji_trick",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        212 => {
            let options: Vec<ChoiceOption> = core
                .alive_players()
                .filter(|&p| p != owner)
                .map(ChoiceOption::Player)
                .collect();
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddfenji_target",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        213 => {
            let mut options: Vec<ChoiceOption> = core
                .alive_players()
                .filter(|&p| !he_cards(core, p).is_empty())
                .map(ChoiceOption::Player)
                .collect();
            if options.is_empty() {
                options = core.alive_players().map(ChoiceOption::Player).collect();
            }
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddxiaheng_discard_target",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        214 => {
            let options: Vec<ChoiceOption> =
                core.alive_players().map(ChoiceOption::Player).collect();
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddxiaheng_damage_target",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        215 => {
            let options = hand_options(core, owner);
            let max = options.len().min(4);
            (max > 0).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddfengzheng_cards",
                    options,
                    min: 1,
                    max,
                    allow_cancel: true,
                },
            ))
        }
        216 => {
            let options: Vec<ChoiceOption> =
                core.alive_players().map(ChoiceOption::Player).collect();
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddfengzheng_give_target",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        217 => {
            let options = he_options(core, owner);
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddyulv_put",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        218 => Some((
            owner,
            ChoiceSpec {
                kind: ChoiceKind::PickTag,
                prompt: "dddyulv_mode",
                options: vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
                min: 1,
                max: 1,
                allow_cancel: true,
            },
        )),
        219 => Some((
            owner,
            ChoiceSpec {
                kind: ChoiceKind::PickTag,
                prompt: "dddyulv_old_declare",
                options: (1u8..=13).map(ChoiceOption::Tag).collect(),
                min: 1,
                max: 1,
                allow_cancel: false,
            },
        )),
        220 => Some((
            owner,
            ChoiceSpec {
                kind: ChoiceKind::PickTag,
                prompt: "dddyulv_old_mode",
                options: vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
                min: 1,
                max: 1,
                allow_cancel: true,
            },
        )),
        221 => {
            let options: Vec<ChoiceOption> = core
                .alive_players()
                .filter(|&p| p != owner && !core.player(p).hand.is_empty())
                .map(ChoiceOption::Player)
                .collect();
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddfenye_target",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        222 => {
            let options = hand_options(core, owner);
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddfenye_owner_card",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        223 => {
            let t = target.unwrap_or(owner);
            let options = hand_options(core, t);
            (!options.is_empty()).then_some((
                t,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddfenye_target_card",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        224 => {
            let options: Vec<ChoiceOption> =
                core.alive_players().map(ChoiceOption::Player).collect();
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddshichao_target",
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

pub(crate) fn wait_ask_c(
    machine: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    stage: HostStage,
    ask_id: u32,
) -> Flow {
    set_flag(ctx.core, machine.owner, ASK_KEY, ask_id);
    if let Some((observer, spec)) = build_ask_c(
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

pub(crate) fn active_ask_u01_batch_c(
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
    let (observer, spec) = build_ask_c(ask_id, owner, target, card, cards, core, catalog)?;
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

pub(crate) fn trigger_meta_u01_batch_c(name: &str) -> Option<TriggerMeta> {
    let meta = |event: EventKind, extra_events: Vec<EventKind>| TriggerMeta {
        event,
        timing: Timing::Normal,
        priority: 1,
        order_selectable: false,
        optional: false,
        extra_events,
    };
    match name {
        "dddqiahua" | "dddxunxun" | "dddfusi" | "dddmiaoxing" | "dddfenji" | "dddxiaheng"
        | "dddfengzheng" | "dddshichao" => Some(meta(EventKind::PhaseBegin, Vec::new())),
        "dddtuoji" => Some(meta(EventKind::CardUsed, Vec::new())),
        "dddchashi" | "dddqice" => Some(meta(EventKind::PhaseBegin, vec![EventKind::CardUsed])),
        "dddfushi" => Some(meta(EventKind::CardUsed, vec![EventKind::CardResponded])),
        "dddyulv" | "dddyulv_old" => {
            Some(meta(EventKind::PhaseBegin, vec![EventKind::CardsDiscarded]))
        }
        _ => None,
    }
}

pub(crate) fn limits_u01_batch_c(name: &str) -> crate::catalog::SkillLimits {
    match name {
        "dddfusi_global" | "dddfengzheng" | "dddfenye" => crate::catalog::SkillLimits {
            once_per_turn: true,
            ..Default::default()
        },
        _ => crate::catalog::SkillLimits::default(),
    }
}

pub(crate) fn filter_u01_batch_c(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    if !ctx.state.player(owner).alive {
        return false;
    }
    match skill.name.as_str() {
        "dddqiahua" => matches!(
            event,
            EventData::Phase { player, phase: Phase::Prepare, begin: true }
            if *player != owner
                && ctx.state.player(*player).alive
                && ctx.state.player(owner).hand.len() >= ctx.state.player(*player).hp.max(1) as usize
        ),
        "dddxunxun" => matches!(
            event,
            EventData::Phase { player, phase: Phase::Draw, begin: true }
            if (*player == owner
                || turn_flag(ctx.state, *player, "dddxunxun_granted", "dddxunxun_stamp") > 0)
                && ctx.state.deck.len() >= 2
        ),
        "dddfusi" => matches!(
            event,
            EventData::Phase { player, phase: Phase::Discard, begin: true }
            if *player == owner
        ),
        "dddtuoji" => matches!(
            event,
            EventData::CardUsed(u)
            if u.player != owner
                && !u.cancelled
                && (turn_flag(ctx.state, owner, "dddfusi_used_card", "dddfusi_stamp") > 0
                    || flag(ctx.state, owner, "ddd_shown_count") > 0)
        ),
        "dddchashi" => match event {
            EventData::Phase {
                player,
                phase: Phase::Play,
                begin: true,
            } => *player != owner && !he_cards(ctx.state, owner).is_empty(),
            EventData::CardUsed(u) => {
                u.player != owner
                    && !u.cancelled
                    && chashi_card_matches(ctx.state, ctx.catalog, owner, u)
            }
            _ => false,
        },
        "dddqice" => match event {
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } => *player == owner && flag(ctx.state, owner, "ddd_qice_disabled") == 0,
            EventData::CardUsed(u) => {
                let kind = u.use_as.or_else(|| u.card.map(|c| ctx.catalog.card_kind(c)));
                u.player == owner && !u.cancelled && kind.is_some_and(is_trick_kind)
            }
            _ => false,
        },
        "dddmiaoxing" => matches!(
            event,
            EventData::Phase { player, phase: Phase::Draw, begin: true }
            if *player == owner
                && (!ctx.state.player(owner).hand.is_empty() || !ctx.state.deck.is_empty())
        ),
        "dddfushi" => {
            if turn_flag(ctx.state, owner, "dddfushi_used_turn", "dddfushi_stamp") > 0 {
                return false;
            }
            match event {
                EventData::CardUsed(u) => {
                    let kind = u.use_as.or_else(|| u.card.map(|c| ctx.catalog.card_kind(c)));
                    u.player == owner
                        && !u.cancelled
                        && kind.is_some_and(|k| k.is_basic_kind())
                }
                EventData::CardResponded { card, player } => {
                    *player == owner && ctx.catalog.card_kind(*card).is_basic_kind()
                }
                _ => false,
            }
        }
        "dddfenji" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::Draw | Phase::Discard,
                begin: true,
            }
            if *player == owner
                && ctx.state.alive_players().any(|p| p != owner)
                && !fenji_tag_options(ctx.state, owner).is_empty()
        ),
        "dddxiaheng" => matches!(
            event,
            EventData::Phase { player, phase: Phase::Play, begin: true }
            if *player == owner
        ),
        "dddfengzheng" => matches!(
            event,
            EventData::Phase { player, phase: Phase::Prepare, begin: true }
            if *player == owner && flag(ctx.state, owner, "dddfengzheng_count") > 0
        ),
        "dddyulv" => match event {
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } => {
                *player == owner
                    && ctx
                        .state
                        .vm_aux
                        .piles
                        .get(&(owner, "dddyulv".to_string()))
                        .is_none_or(Vec::is_empty)
            }
            EventData::CardsDiscarded { cards, .. } => {
                yulv_pile_rank(ctx.state, ctx.catalog, owner)
                    .is_some_and(|r| cards.iter().any(|&c| ctx.catalog.cards[c as usize].rank == r))
            }
            _ => false,
        },
        "dddyulv_old" => match event {
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } => *player == owner && flag(ctx.state, owner, "dddyulv_old_rank") == 0,
            EventData::CardsDiscarded { cards, .. } => {
                let r = flag(ctx.state, owner, "dddyulv_old_rank") as u8;
                r > 0 && cards.iter().any(|&c| ctx.catalog.cards[c as usize].rank == r)
            }
            _ => false,
        },
        "dddshichao" => matches!(
            event,
            EventData::Phase { player, phase: Phase::Prepare, begin: true }
            if *player == owner
        ),
        _ => false,
    }
}

pub(crate) fn activate_u01_batch_c(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<Box<dyn crate::catalog::SkillMachine>> {
    if !ctx.state.player(owner).alive || ctx.state.turn.phase != Phase::Play {
        return None;
    }
    let ok = match skill.name.as_str() {
        "dddfusi_global" => fusi_source(ctx.state, ctx.catalog, owner).is_some(),
        "dddfengzheng" => {
            ctx.state.turn.current == owner && !ctx.state.player(owner).hand.is_empty()
        }
        "dddfenye" => {
            ctx.state.turn.current == owner
                && !ctx.state.player(owner).hand.is_empty()
                && ctx
                    .state
                    .alive_players()
                    .any(|p| p != owner && !ctx.state.player(p).hand.is_empty())
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

pub(crate) fn machine_u01_batch_c(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    let mut target = None;
    let mut card = None;
    match (skill.name.as_str(), event) {
        ("dddqiahua" | "dddxunxun" | "dddchashi", EventData::Phase { player, .. }) => {
            target = Some(*player);
        }
        ("dddchashi", EventData::CardUsed(u)) => {
            target = Some(u.player);
            card = Some(1);
        }
        ("dddqice", EventData::CardUsed(_)) => {
            card = Some(1);
        }
        ("dddfengzheng", EventData::Phase { .. }) => {
            card = Some(1);
        }
        ("dddyulv" | "dddyulv_old", EventData::CardsDiscarded { .. }) => {
            card = Some(1);
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
        cards: Vec::new(),
        stage: HostStage::Act,
    }
}
