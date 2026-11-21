use super::*;

pub(crate) fn trigger_meta_u01_fix(name: &str) -> Option<TriggerMeta> {
    let meta = |event: EventKind, extra_events: Vec<EventKind>| TriggerMeta {
        event,
        timing: Timing::Normal,
        priority: 1,
        order_selectable: false,
        optional: false,
        extra_events,
    };
    match name {
        "dddzhishu" | "dddyeshen" | "dddanliu" | "dddguiying" | "dddlangzhi" => {
            Some(meta(EventKind::PhaseBegin, Vec::new()))
        }
        "dddjungui" => Some(meta(EventKind::PhaseBegin, vec![EventKind::CardUsed])),
        "dddqiaoduan" => Some(meta(EventKind::Linked, Vec::new())),
        "dddtongyu" => Some(meta(EventKind::CardUsed, Vec::new())),
        "dddzhilian" => Some(meta(EventKind::TurnPhaseEnd, Vec::new())),
        "dddfushi" => Some(meta(EventKind::CardResponded, Vec::new())),
        "dddfuzong" => Some(meta(EventKind::TurnBegin, vec![EventKind::Death])),
        _ => None,
    }
}

pub(crate) fn limits_u01_fix(name: &str) -> Option<SkillLimits> {
    match name {
        "dddjuxian" | "dddzhishu" | "dddjungui" | "dddshilie" | "dddyeshen" | "dddqiaoduan"
        | "ddddiedang" | "dddanliu" | "dddguiying" | "dddbingjian" | "dddtongyu"
        | "dddzhilian" | "dddjijian" | "dddjijian_old" | "dddfushi" | "dddlangzhi"
        | "dddfuzong" => Some(SkillLimits::default()),
        _ => None,
    }
}

pub(crate) fn filter_u01_fix(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> Option<bool> {
    if !ctx.state.player(owner).alive {
        return is_u01_fix_1_name(&skill.name).then_some(false);
    }
    let res = match skill.name.as_str() {
        "dddjuxian" | "dddshilie" | "ddddiedang" | "dddbingjian" | "dddjijian"
        | "dddjijian_old" => false,
        "dddzhishu" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::Play,
                begin: true,
            } if *player == owner && !asks_a::zhishu_movable_cards(ctx.state, ctx.catalog).is_empty()
        ),
        "dddjungui" => match event {
            EventData::CardUsed(u) => !u.cancelled && u.player == owner,
            EventData::Phase {
                player,
                phase: Phase::End,
                begin: true,
            } => *player == owner,
            _ => false,
        },
        "dddyeshen" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::End,
                begin: true,
            } if ctx.state.player(*player).alive
                && (!ctx.state.deck.is_empty() || !ctx.state.discard.is_empty())
        ),
        "dddqiaoduan" => {
            if turn_flag(ctx.state, owner, "dddqiaoduan_used_turn", "dddqiaoduan_stamp") != 0 {
                false
            } else if let EventData::Linked { player, .. } = event {
                let linked_cnt = ctx
                    .state
                    .alive_players()
                    .filter(|&p| ctx.state.player(p).linked)
                    .count();
                if linked_cnt == 0 {
                    false
                } else if ctx.state.player(*player).linked {
                    ctx.state.player(owner).hand.len() >= linked_cnt
                        && ctx
                            .state
                            .alive_players()
                            .any(|p| ctx.state.player(p).hp < ctx.state.player(p).max_hp as i32)
                } else {
                    true
                }
            } else {
                false
            }
        }
        "dddanliu" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::End,
                begin: true,
            } if *player == owner
                && !ctx.state.player(owner).hand.is_empty()
                && ctx
                    .state
                    .alive_players()
                    .any(|p| p != owner && !ctx.state.player(p).hand.is_empty())
        ),
        "dddguiying" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } if *player == owner
                && is_identity_lord(ctx.state, owner)
                && !asks_a::dddguiying_wu_others(ctx.state, ctx.catalog, owner).is_empty()
        ),
        "dddtongyu" => matches!(
            event,
            EventData::CardUsed(u)
            if !u.cancelled
                && u.player == owner
                && u.use_as == Some(CardKind::Wugu)
                && flag(ctx.state, owner, "dddtongyu_pending_go") == 1
        ),
        "dddzhilian" => matches!(
            event,
            EventData::TurnEndWindow { after: false, .. }
            if asks_bcd::dddzhilian_targets_eligible(ctx.state, ctx.catalog, owner).is_some()
        ),
        "dddfushi" => match event {
            EventData::CardResponded { card, player }
                if *player == owner
                    && {
                        let raw = flag(ctx.state, owner, "dddfushi_resp_eff_kind");
                        let eff = if raw != 0 {
                            crate::archive::kind_from_code((raw.saturating_sub(1)) as u16)
                                .unwrap_or_else(|_| ctx.catalog.card_kind(*card))
                        } else {
                            ctx.catalog.card_kind(*card)
                        };
                        eff.is_basic_kind()
                    }
                    && turn_flag(ctx.state, owner, "dddfushi_used_turn", "dddfushi_stamp") == 0 =>
            {
                let p1 = ctx
                    .state
                    .vm_aux
                    .piles
                    .get(&(owner, "dddmiaoxing_1".to_string()))
                    .is_some_and(|v| !v.is_empty());
                let p2 = ctx
                    .state
                    .vm_aux
                    .piles
                    .get(&(owner, "dddmiaoxing_2".to_string()))
                    .is_some_and(|v| !v.is_empty());
                !ctx.state.player(owner).hand.is_empty() || p1 || p2
            }
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
                && {
                    let atk = ctx.state.attack_range(ctx.catalog, owner);
                    ctx.state.alive_players().any(|p| {
                        p != owner
                            && ctx.state.computed_distance(ctx.catalog, owner, p) <= atk
                            && !he_cards(ctx.state, p).is_empty()
                    })
                }
        ),
        "dddfuzong" => {
            if !is_identity_lord(ctx.state, owner) {
                false
            } else {
                match event {
                    EventData::TurnBegin { .. } => {
                        flag(ctx.state, owner, "dddfuzong_init_done") == 0
                            && ctx.state.turn.round <= 1
                    }
                    EventData::Death { player, .. } => {
                        ctx.state.effective_faction(ctx.catalog, *player)
                            == Some(crate::catalog::Faction::Qun)
                            && !he_cards(ctx.state, owner).is_empty()
                    }
                    _ => false,
                }
            }
        }
        _ => return None,
    };
    Some(res)
}

fn is_u01_fix_1_name(name: &str) -> bool {
    is_u01_fix_1_batch_a(name)
        || is_u01_fix_1_batch_b(name)
        || is_u01_fix_1_batch_c(name)
        || is_u01_fix_1_batch_d(name)
}

pub(crate) fn activate_u01_fix(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<Option<Box<dyn crate::catalog::SkillMachine>>> {
    if !is_u01_fix_1_name(&skill.name) {
        return None;
    }
    if !ctx.state.player(owner).alive
        || ctx.state.turn.phase != Phase::Play
        || ctx.state.turn.current != owner
    {
        return Some(None);
    }
    let ok = match skill.name.as_str() {
        "dddjuxian" => {
            turn_flag(ctx.state, owner, "dddjuxian_ban", "dddjuxian_stamp") == 0
                && !he_cards(ctx.state, owner).is_empty()
                && (!is_scope4_used(ctx.state, owner, "dddjuxian_top")
                    || (!is_scope4_used(ctx.state, owner, "dddjuxian_hand")
                        && ctx.state.alive_players().any(|p| p != owner)))
        }
        "dddshilie" => {
            turn_flag(ctx.state, owner, "dddshilie_used_turn", "dddshilie_stamp") == 0
                && can_use_sha_play_phase(ctx.state, ctx.catalog, owner)
                && !asks_a::dddshilie_unshown_hand(ctx.state, owner).is_empty()
                && asks_a::dddshilie_max_unshown_sum(ctx.state, ctx.catalog, owner)
                    >= asks_a::dddshilie_threshold(ctx.state, owner)
                && !virtual_legal_targets(ctx.state, ctx.catalog, owner, CardKind::Sha).is_empty()
        }
        "ddddiedang" => {
            !is_scope4_used(ctx.state, owner, "ddddiedang") && {
                let st = flag(ctx.state, owner, "ddddiedang_state");
                st == 0 || he_cards(ctx.state, owner).len() >= 3
            }
        }
        "dddbingjian" => {
            turn_flag(ctx.state, owner, "dddbingjian_used_count", "dddbingjian_stamp") < 2
                && can_use_sha_play_phase(ctx.state, ctx.catalog, owner)
                && ctx.state.player(owner).hand.len() != 2
                && !virtual_legal_targets(ctx.state, ctx.catalog, owner, CardKind::Sha).is_empty()
        }
        "dddtongyu" => {
            !is_scope4_used(ctx.state, owner, "dddtongyu")
                && !ctx.state.player(owner).hand.is_empty()
        }
        "dddjijian" => {
            !is_scope4_used(ctx.state, owner, "dddjijian")
                && ctx
                    .state
                    .player(owner)
                    .hand
                    .iter()
                    .any(|&c| ctx.catalog.card_kind(c).is_sha_like())
                && ctx
                    .state
                    .alive_players()
                    .any(|p| p != owner && is_male(ctx.state, ctx.catalog, p))
        }
        "dddjijian_old" => {
            !is_scope4_used(ctx.state, owner, "dddjijian_old")
                && ctx.state.alive_players().any(|p| {
                    p != owner
                        && is_male(ctx.state, ctx.catalog, p)
                        && !ctx.state.player(p).hand.is_empty()
                })
        }
        _ => false,
    };
    Some(ok.then(|| {
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
    }))
}

pub(crate) fn machine_u01_fix(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> Option<HostMachine> {
    if !is_u01_fix_1_name(&skill.name) {
        return None;
    }
    let mut target = None;
    let mut card = None;
    let mut cards = Vec::new();
    let mut targets = Vec::new();
    match (skill.name.as_str(), event) {
        ("dddjungui", EventData::CardUsed(u)) => {
            targets.push(0xFF);
            if let Some(c) = u.card {
                cards.push(c);
            }
        }
        ("dddyeshen", EventData::Phase { player, .. }) => {
            target = Some(*player);
        }
        ("dddqiaoduan", EventData::Linked { player, .. }) => {
            target = Some(*player);
        }
        ("dddtongyu", EventData::CardUsed(_)) => {
            card = Some(1);
        }
        ("dddfuzong", EventData::TurnBegin { .. }) => {
            card = Some(0);
        }
        ("dddfuzong", EventData::Death { player, .. }) => {
            card = Some(1);
            target = Some(*player);
        }
        _ => {}
    }
    Some(HostMachine {
        def_key: skill.name.clone(),
        behavior: skill.behavior,
        owner,
        target,
        targets,
        card,
        cards,
        stage: HostStage::Act,
    })
}

pub(crate) fn step_u01_fix(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Option<Flow> {
    if !is_u01_fix_1_name(&machine.def_key) {
        return None;
    }
    // `dddjungui` turn suit tracking on CardUsed (including virtual suit cards!)
    if machine.def_key == "dddjungui" && machine.targets.first().copied() == Some(0xFF) {
        let mut mask = turn_flag(
            ctx.core,
            machine.owner,
            "dddjungui_suits_mask",
            "dddjungui_suits_stamp",
        );
        if let Some(&cid) = machine.cards.first() {
            mask |= effective_suit_bit(ctx.core, ctx.catalog, machine.owner, cid);
            set_turn_flag(
                ctx.core,
                machine.owner,
                "dddjungui_suits_mask",
                "dddjungui_suits_stamp",
                mask,
            );
        }
        return Some(Flow::Done);
    }
    step_a1(machine, ctx)
        .or_else(|| step_a2(machine, ctx))
        .or_else(|| step_bcd(machine, ctx))
}
