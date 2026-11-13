use super::*;

pub(crate) fn is_black_non_trick(catalog: &Catalog, c: CardId) -> bool {
    let kind = catalog.card_kind(c);
    !is_red(catalog, c) && (kind.is_basic_kind() || kind.is_equip())
}

pub(crate) fn has_bing_in_judge(core: &GameStateCore, catalog: &Catalog, p: PlayerId) -> bool {
    core.player(p).judge.iter().any(|&jc| {
        core.declared_kinds
            .get(&jc)
            .copied()
            .unwrap_or_else(|| catalog.card_kind(jc))
            == CardKind::Bing
    })
}

pub(crate) fn clear_once_used(
    core: &mut GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    skill_name: &str,
) {
    if let Some(def) = catalog.skill_by_name(skill_name) {
        if let Some(inst) = core.skills.get_mut(&(owner, def)) {
            inst.once_used = false;
        }
    }
}

pub(crate) fn compute_shixing_mode(core: &GameStateCore, catalog: &Catalog) -> u32 {
    let has_undamaged_female = core.alive_players().any(|p| {
        let pl = core.player(p);
        pl.hp >= pl.max_hp as i32
            && pl.general_slots.iter().flatten().any(|&g| {
                catalog
                    .general_meta(g)
                    .is_some_and(|(_, _, _, _, gender)| {
                        gender == crate::vocabulary::generals::Gender::Female
                    })
            })
    });
    if has_undamaged_female {
        1
    } else if core.turn.cards_used_turn == 0 {
        2
    } else {
        3
    }
}

fn confirm_spec(prompt: &'static str, allow_cancel: bool) -> ChoiceSpec {
    ChoiceSpec {
        kind: ChoiceKind::Confirm,
        prompt,
        options: vec![ChoiceOption::Confirm, ChoiceOption::Pass],
        min: 1,
        max: 1,
        allow_cancel,
    }
}

pub(crate) fn build_ask_d(
    ask_id: u32,
    owner: PlayerId,
    target: Option<PlayerId>,
    _card: Option<u32>,
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<(PlayerId, ChoiceSpec)> {
    match ask_id {
        301 => {
            let holder_raw = flag(core, owner, "dddyouxue_holder");
            let alive: Vec<PlayerId> = core.alive_players().collect();
            let mut options: Vec<ChoiceOption> = if alive.len() > 1 && holder_raw > 0 {
                alive
                    .iter()
                    .copied()
                    .filter(|&p| u32::from(p) + 1 != holder_raw)
                    .map(ChoiceOption::Player)
                    .collect()
            } else {
                alive.iter().copied().map(ChoiceOption::Player).collect()
            };
            if options.is_empty() {
                options = alive.into_iter().map(ChoiceOption::Player).collect();
            }
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddyouxue_target",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        302 => {
            let options = he_options(core, owner);
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddchengjing_card",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        303 => {
            let options: Vec<ChoiceOption> = core
                .alive_players()
                .filter(|&p| p != owner)
                .map(ChoiceOption::Player)
                .collect();
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddchengjing_target",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        304 => {
            let options: Vec<ChoiceOption> = he_cards(core, owner)
                .into_iter()
                .filter(|&c| is_black_non_trick(catalog, c))
                .map(ChoiceOption::Card)
                .collect();
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddduanbing_card",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        305 => {
            let options: Vec<ChoiceOption> = core
                .alive_players()
                .filter(|&p| p != owner)
                .map(ChoiceOption::Player)
                .collect();
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddduanbing_target",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        306 => Some((owner, confirm_spec("dddduanbing_transfer", true))),
        307 => Some((
            owner,
            ChoiceSpec {
                kind: ChoiceKind::PickTag,
                prompt: "dddlangzhi_mode",
                options: vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
                min: 1,
                max: 1,
                allow_cancel: true,
            },
        )),
        308 => {
            let options = he_options(core, owner);
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddlangzhi_own_card",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        309 => {
            let actor = target.unwrap_or(owner);
            Some((actor, confirm_spec("dddfuyi_confirm", true)))
        }
        310 => {
            let options = hand_options(core, owner);
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddzhuanshe_give",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        311 => Some((owner, confirm_spec("dddzhuanshe_damage", true))),
        312 => Some((owner, confirm_spec("dddlianer_confirm", true))),
        313 => {
            let options: Vec<ChoiceOption> =
                core.alive_players().map(ChoiceOption::Player).collect();
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddanzhi_target",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        314 => {
            let options: Vec<ChoiceOption> = core
                .alive_players()
                .filter(|&p| !core.player(p).linked)
                .map(ChoiceOption::Player)
                .collect();
            let max = options.len();
            (max > 0).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "ddddanggu_targets",
                    options,
                    min: 1,
                    max,
                    allow_cancel: false,
                },
            ))
        }
        315 => {
            let allow_cancel = flag(core, owner, "dddxuanlun_del") == 0;
            Some((owner, confirm_spec("dddlanghuai_confirm", allow_cancel)))
        }
        316 => Some((owner, confirm_spec("dddxuanlun_confirm", true))),
        317 => Some((
            owner,
            ChoiceSpec {
                kind: ChoiceKind::PickTag,
                prompt: "dddxuanlun_end_mode",
                options: vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
                min: 1,
                max: 1,
                allow_cancel: false,
            },
        )),
        318 => {
            let options = he_options(core, owner);
            let max = options.len().min(4);
            (max > 0).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddxuanlun_cards",
                    options,
                    min: 1,
                    max,
                    allow_cancel: false,
                },
            ))
        }
        319 => Some((owner, confirm_spec("dddjiexing_confirm", true))),
        320 => {
            let options: Vec<ChoiceOption> = core
                .alive_players()
                .filter(|&p| p != owner)
                .map(ChoiceOption::Player)
                .collect();
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddbailei_target",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        321 => Some((
            owner,
            ChoiceSpec {
                kind: ChoiceKind::PickTag,
                prompt: "dddbailei_mode",
                options: vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
                min: 1,
                max: 1,
                allow_cancel: false,
            },
        )),
        _ => None,
    }
}

pub(crate) fn wait_ask_d(
    machine: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    stage: HostStage,
    ask_id: u32,
) -> Flow {
    set_flag(ctx.core, machine.owner, ASK_KEY, ask_id);
    if let Some((observer, spec)) = build_ask_d(
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

pub(crate) fn active_ask_u01_batch_d(
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
    let (observer, spec) = build_ask_d(ask_id, owner, target, card, core, catalog)?;
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

pub(crate) fn trigger_meta_u01_batch_d(name: &str) -> Option<TriggerMeta> {
    let meta = |event: EventKind, extra_events: Vec<EventKind>| TriggerMeta {
        event,
        timing: Timing::Normal,
        priority: 1,
        order_selectable: false,
        optional: false,
        extra_events,
    };
    match name {
        "dddyouxue" | "dddchengjing" | "dddlangzhi" | "dddzhuanshe" | "ddddanggu"
        | "dddlanghuai" | "dddbailei" => Some(meta(EventKind::PhaseBegin, Vec::new())),
        "dddduanbing" => Some(meta(EventKind::Damage, Vec::new())),
        "dddxiaoxing" => Some(meta(EventKind::PhaseBegin, vec![EventKind::Dying])),
        "dddfuyi" => Some(meta(EventKind::Death, Vec::new())),
        "dddweiqiu" => Some(meta(EventKind::Recover, Vec::new())),
        "dddlianer" => Some(meta(EventKind::CardUsed, Vec::new())),
        "dddanzhi" => Some(meta(EventKind::PhaseBegin, vec![EventKind::CardUsed])),
        "dddshixing" => Some(meta(
            EventKind::PhaseBegin,
            vec![EventKind::Damage, EventKind::Recover, EventKind::CardUsed],
        )),
        "dddfuzong" => Some(meta(EventKind::PhaseBegin, vec![EventKind::Death])),
        "dddxuanlun" => Some(meta(EventKind::Damage, vec![EventKind::PhaseBegin])),
        "dddjiexing" => Some(meta(EventKind::Recover, vec![EventKind::Equipped])),
        _ => None,
    }
}

pub(crate) fn limits_u01_batch_d(name: &str) -> crate::catalog::SkillLimits {
    match name {
        "dddchengjing" | "dddduanbing" => crate::catalog::SkillLimits {
            once_per_turn: true,
            ..Default::default()
        },
        _ => crate::catalog::SkillLimits::default(),
    }
}

pub(crate) fn filter_u01_batch_d(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    if !ctx.state.player(owner).alive {
        return false;
    }
    match skill.name.as_str() {
        "dddyouxue" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } if *player == owner
        ),
        "dddchengjing" => matches!(
            event,
            EventData::Phase {
                phase: Phase::End,
                begin: true,
                ..
            } if !ctx.state.discard.is_empty()
        ),
        "dddduanbing" => matches!(
            event,
            EventData::Damage(d)
            if d.source == Some(owner)
                && d.target != owner
                && ctx.state.player(d.target).alive
                && !ctx.state.player(owner).judge.is_empty()
        ),
        "dddxiaoxing" => match event {
            EventData::Dying { player, .. } => {
                *player == owner && flag(ctx.state, owner, "dddxiaoxing_lost") == 0
            }
            EventData::Phase {
                player,
                phase: Phase::Draw,
                begin: true,
            } => *player == owner && flag(ctx.state, owner, "dddxiaoxing_lost") == 0,
            _ => false,
        },
        "dddlangzhi" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::End,
                begin: true,
            } if *player == owner
                && flag(ctx.state, owner, "dddlangzhi_lost") == 0
                && ctx
                    .state
                    .alive_players()
                    .any(|p| p != owner && !he_cards(ctx.state, p).is_empty())
        ),
        "dddfuyi" => matches!(
            event,
            EventData::Death { .. }
            if flag(ctx.state, owner, "dddxiaoxing_lost") > 0
                || flag(ctx.state, owner, "dddlangzhi_lost") > 0
        ),
        "dddzhuanshe" => match event {
            EventData::Phase {
                player,
                phase: Phase::Play,
                begin: true,
            } => *player != owner && !ctx.state.player(owner).hand.is_empty(),
            EventData::Phase {
                player,
                phase: Phase::End,
                begin: true,
            } => {
                *player != owner
                    && turn_flag(ctx.state, owner, "dddzhuanshe_given", "dddzhuanshe_stamp") > 0
            }
            _ => false,
        },
        "dddweiqiu" => matches!(
            event,
            EventData::Recover { .. } if ctx.state.player(owner).hand.is_empty()
        ),
        "dddlianer" => matches!(
            event,
            EventData::CardUsed(u)
            if u.player == owner && !u.cancelled && u.card.is_some_and(|c| is_red(ctx.catalog, c))
        ),
        "dddanzhi" => match event {
            EventData::CardUsed(u) => !u.cancelled,
            EventData::Phase {
                phase: Phase::Prepare,
                begin: true,
                ..
            } => flag(ctx.state, owner, "dddanzhi_last_black") > 0,
            _ => false,
        },
        "dddshixing" => match event {
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } => *player == owner,
            EventData::Damage(_) | EventData::Recover { .. } | EventData::CardUsed(_) => true,
            _ => false,
        },
        "ddddanggu" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::End,
                begin: true,
            } if *player == owner && ctx.state.alive_players().any(|p| !ctx.state.player(p).linked)
        ),
        "dddfuzong" => match event {
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } => *player == owner && flag(ctx.state, owner, "dddfuzong_init") == 0,
            EventData::Death { .. } => !he_cards(ctx.state, owner).is_empty(),
            _ => false,
        },
        "dddlanghuai" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::Draw,
                begin: true,
            } if *player == owner
        ),
        "dddxuanlun" => match event {
            EventData::Damage(d) => d.target == owner,
            EventData::Phase {
                phase: Phase::End,
                begin: true,
                ..
            } => turn_flag(ctx.state, owner, "dddxuanlun_triggered", "dddxuanlun_stamp") > 0,
            _ => false,
        },
        "dddjiexing" => match event {
            EventData::Recover { player, .. } => {
                if *player == owner {
                    flag(ctx.state, owner, "dddjiexing_used") > 0
                } else {
                    flag(ctx.state, owner, "dddjiexing_used") == 0
                        && ctx.state.player(owner).hp < ctx.state.player(owner).max_hp as i32
                }
            }
            EventData::Equipped { player, card, .. } => {
                if *player == owner {
                    flag(ctx.state, owner, "dddjiexing_used") > 0
                } else {
                    flag(ctx.state, owner, "dddjiexing_used") == 0
                        && ctx.state.player(*player).equips.contains(card)
                }
            }
            _ => false,
        },
        "dddbailei" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } if *player == owner
                && flag(ctx.state, owner, "dddbailei_lost") == 0
                && ctx.state.alive_players().any(|p| p != owner)
        ),
        _ => false,
    }
}

pub(crate) fn activate_u01_batch_d(
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
        "dddchengjing" => {
            turn_flag(
                ctx.state,
                owner,
                "dddchengjing_used_turn",
                "dddchengjing_stamp",
            ) == 0
                && !he_cards(ctx.state, owner).is_empty()
                && ctx.state.alive_players().any(|p| p != owner)
        }
        "dddduanbing" => {
            turn_flag(
                ctx.state,
                owner,
                "dddduanbing_used_turn",
                "dddduanbing_stamp",
            ) == 0
                && !has_bing_in_judge(ctx.state, ctx.catalog, owner)
                && he_cards(ctx.state, owner)
                    .into_iter()
                    .any(|c| is_black_non_trick(ctx.catalog, c))
                && ctx.state.alive_players().any(|p| p != owner)
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

pub(crate) fn machine_u01_batch_d(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    let mut target = None;
    let mut card = None;
    let mut cards = Vec::new();
    match (skill.name.as_str(), event) {
        (
            "dddchengjing",
            EventData::Phase {
                phase: Phase::End,
                begin: true,
                ..
            },
        ) => {
            card = Some(1);
        }
        ("dddduanbing", EventData::Damage(d)) => {
            target = Some(d.target);
            card = Some(1);
        }
        ("dddxiaoxing", EventData::Dying { .. }) => {
            card = Some(1);
        }
        ("dddfuyi", EventData::Death { killer, .. }) => {
            target = Some(killer.unwrap_or(owner));
        }
        ("dddzhuanshe", EventData::Phase { player, phase, .. }) => {
            target = Some(*player);
            card = Some(u32::from(*phase == Phase::End));
        }
        ("dddlianer", EventData::CardUsed(u)) => {
            card = u.card;
        }
        ("dddanzhi", EventData::CardUsed(u)) => {
            card = Some(1);
            if let Some(c) = u.card {
                cards.push(c);
            }
        }
        ("dddfuzong", EventData::Death { .. }) => {
            card = Some(1);
        }
        (
            "dddxuanlun",
            EventData::Phase {
                phase: Phase::End,
                begin: true,
                ..
            },
        ) => {
            card = Some(1);
        }
        ("dddjiexing", EventData::Recover { player, .. }) => {
            target = Some(*player);
        }
        ("dddjiexing", EventData::Equipped { player, card: c, .. }) => {
            target = Some(*player);
            card = Some(*c);
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
