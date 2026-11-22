use super::*;

pub(crate) fn trigger_meta_u01_fix3(name: &str) -> Option<TriggerMeta> {
    let meta = |event: EventKind, timing: Timing, priority: i32, extra_events: Vec<EventKind>| {
        TriggerMeta {
            event,
            timing,
            priority,
            order_selectable: false,
            optional: false,
            extra_events,
        }
    };
    match name {
        "dddxiaheng" | "ddddanggu" => Some(meta(EventKind::PhaseBegin, Timing::Normal, 1, Vec::new())),
        "dddfengzheng" | "dddyouxue" | "dddyulv" | "dddyulv_old" => {
            Some(meta(EventKind::TurnBegin, Timing::Normal, 1, Vec::new()))
        }
        "dddbailei" => Some(meta(EventKind::PhaseBegin, Timing::Normal, 1, Vec::new())),
        "dddshichao" => Some(meta(
            EventKind::PhaseBegin,
            Timing::Normal,
            1,
            vec![EventKind::DamageBefore],
        )),
        "dddchengjing" => Some(meta(EventKind::TurnPhaseEnd, Timing::Normal, 1, Vec::new())),
        "dddduanbing" => Some(meta(EventKind::Damage, Timing::Normal, 1, Vec::new())),
        "dddxiaoxing" => Some(meta(EventKind::Dying, Timing::Normal, 1, Vec::new())),
        "dddfuyi" => Some(meta(EventKind::Death, Timing::Normal, 1, Vec::new())),
        "dddzhuanshe" => Some(meta(
            EventKind::PhaseBegin,
            Timing::Normal,
            1,
            vec![EventKind::CardUsed, EventKind::TurnPhaseEnd],
        )),
        "dddweiqiu" => Some(meta(
            EventKind::Recover,
            Timing::First,
            1,
            Vec::new(),
        )),
        "dddlianer" => Some(meta(EventKind::CardUsed, Timing::Normal, 1, Vec::new())),
        "dddshixing" => Some(meta(
            EventKind::PhaseBegin,
            Timing::Normal,
            1,
            vec![
                EventKind::Damage,
                EventKind::Recover,
                EventKind::LoseHp,
                EventKind::CardUsed,
            ],
        )),
        "xiangle" => Some(meta(EventKind::CardUsed, Timing::First, 1, Vec::new())),
        "jiushi" => Some(meta(EventKind::Damage, Timing::Normal, 2, Vec::new())),
        "dddfengzheng_global" | "dddfenye" | "rezhiheng" => {
            Some(meta(EventKind::PhaseBegin, Timing::Normal, 1, Vec::new()))
        }
        "dddlanghuai" => Some(meta(
            EventKind::DrawPhaseBegin1,
            Timing::Normal,
            1,
            Vec::new(),
        )),
        "dddxuanlun" => Some(meta(
            EventKind::Damage,
            Timing::Normal,
            1,
            vec![EventKind::TurnPhaseEnd],
        )),
        "dddjiexing" => Some(meta(
            EventKind::Recover,
            Timing::First,
            1,
            vec![EventKind::Equipped],
        )),
        "dddanzhi" => None,
        _ => None,
    }
}

pub(crate) fn runs_before_phase_begin_3(name: &str) -> bool {
    matches!(
        name,
        "dddzhuanshe"
            | "dddshixing"
            | "dddxiaheng"
            | "ddddanggu"
            | "dddfengzheng"
            | "dddyouxue"
            | "dddyulv"
            | "dddyulv_old"
            | "dddbailei"
            | "dddshichao"
    )
}

pub(crate) fn limits_u01_fix3(name: &str) -> Option<SkillLimits> {
    if is_u01_fix_3_name(name) && name != "dddanzhi" {
        Some(SkillLimits::default())
    } else {
        None
    }
}

pub(crate) fn filter_u01_fix3_with_frames(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    frames: &[crate::event::Frame],
    frame: &crate::event::Frame,
    owner: PlayerId,
    event: &EventData,
) -> Option<bool> {
    if !is_u01_fix_3_name(&skill.name) || skill.name == "dddanzhi" {
        return None;
    }
    if !ctx.state.player(owner).alive {
        return Some(false);
    }
    match (skill.name.as_str(), event) {
        ("dddduanbing", EventData::Damage(d)) => {
            if d.source != Some(owner)
                || d.prevented
                || d.amount <= 0
                || !ctx.state.player(d.target).alive
            {
                return Some(false);
            }
            let is_card_dmg = d.card.is_some() || has_card_used_ancestor(frames, frame);
            let has_bing = ctx
                .state
                .player(owner)
                .judge
                .iter()
                .any(|&jc| eff_card_kind_3(ctx.state, ctx.catalog, jc) == CardKind::Bing);
            let target_has_bing = ctx
                .state
                .player(d.target)
                .judge
                .iter()
                .any(|&jc| eff_card_kind_3(ctx.state, ctx.catalog, jc) == CardKind::Bing);
            Some(is_card_dmg && has_bing && !target_has_bing)
        }
        ("dddweiqiu", EventData::Recover { .. }) => Some(
            matches!(frame.stage, crate::event::Stage::BeforeTriggers)
                && ctx.state.player(owner).hand.is_empty()
                && flag(ctx.state, 0, &format!("u01_recover_cancelled:{}", frame.id)) == 0,
        ),
        ("dddjiexing", EventData::Recover { player, amount, .. }) => {
            if flag(ctx.state, 0, &format!("u01_recover_cancelled:{}", frame.id)) != 0 {
                return Some(false);
            }
            let inst_unused = ctx
                .catalog
                .skill_by_name("dddjiexing")
                .and_then(|d| ctx.state.skills.get(&(owner, d)))
                .is_some_and(|i| !i.once_used)
                && flag(ctx.state, owner, "dddjiexing_used") == 0;
            if matches!(frame.stage, crate::event::Stage::BeforeTriggers) {
                let ow = ctx.state.player(owner);
                Some(
                    inst_unused
                        && *player != owner
                        && *amount > 0
                        && ow.hp < ow.max_hp as i32,
                )
            } else {
                Some(
                    !inst_unused
                        && *player == owner
                        && *amount > 0
                        && flag(ctx.state, owner, &format!("u01_jiexing_self_recover:{}", frame.id))
                            == 0,
                )
            }
        }
        ("dddjiexing", EventData::Equipped { player, .. }) => {
            let played = flag(ctx.state, *player, &format!("u01_equip_played:{}", frame.id)) != 0;
            if !played {
                return Some(false);
            }
            let inst_unused = ctx
                .catalog
                .skill_by_name("dddjiexing")
                .and_then(|d| ctx.state.skills.get(&(owner, d)))
                .is_some_and(|i| !i.once_used)
                && flag(ctx.state, owner, "dddjiexing_used") == 0;
            if matches!(frame.stage, crate::event::Stage::BeforeTriggers) {
                Some(
                    inst_unused
                        && *player != owner
                        && flag(ctx.state, *player, &format!("u01_equip_cancelled:{}", frame.id))
                            == 0,
                )
            } else {
                Some(
                    !inst_unused
                        && *player == owner
                        && flag(ctx.state, owner, &format!("u01_jiexing_self_equip:{}", frame.id))
                            == 0,
                )
            }
        }
        ("dddlanghuai", EventData::DrawPhase { player, outcome, pending, .. }) => Some(
            *player == owner
                && !matches!(outcome, crate::event::DrawOutcome::Cancelled)
                && *pending > 0,
        ),
        (
            "dddlanghuai",
            EventData::DrawWindow {
                root,
                window: crate::event::DrawWindowKind::Begin1,
            },
        ) => Some(frames.iter().any(|f| {
            f.id == *root
                && matches!(
                    &f.data,
                    EventData::DrawPhase { player, outcome, pending, .. }
                    if *player == owner
                        && !matches!(outcome, crate::event::DrawOutcome::Cancelled)
                        && *pending > 0
                )
        })),
        ("dddlanghuai", EventData::DrawWindow { .. }) => Some(false),
        _ => filter_u01_fix3(skill, ctx, owner, event),
    }
}

pub(crate) fn filter_u01_fix3(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> Option<bool> {
    if !is_u01_fix_3_name(&skill.name) || skill.name == "dddanzhi" {
        return None;
    }
    if !ctx.state.player(owner).alive {
        return Some(false);
    }
    let res = match skill.name.as_str() {
        "dddxiaheng" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::End,
                begin: true,
            } if *player == owner
        ),
        "dddfengzheng" => match event {
            EventData::TurnBegin { .. } => {
                let r = ctx.state.turn.round.max(1);
                r > 1
                    && flag(ctx.state, owner, "dddfengzheng_last_round") != r
                    && {
                        let prev_r = r - 1;
                        let mask = flag(ctx.state, 0, &format!("dddfengzheng_round:{prev_r}_mask"));
                        ctx.state.alive_players().any(|p| (mask & (1u32 << p)) != 0)
                    }
            }
            _ => false,
        },
        "dddfengzheng_global" | "dddfenye" | "rezhiheng" => false,
        "dddyulv" => matches!(
            event,
            EventData::TurnBegin { .. }
            if ctx.state.turn.phase_number == 0
                && flag(ctx.state, owner, "dddyulv_init") == 0
                && ctx
                    .state
                    .vm_aux
                    .piles
                    .get(&(owner, "dddyulv".to_string()))
                    .is_none_or(Vec::is_empty)
        ),
        "dddyulv_old" => matches!(
            event,
            EventData::TurnBegin { .. }
            if ctx.state.turn.phase_number == 0 && flag(ctx.state, owner, "dddyulv_old_rank") == 0
        ),
        "dddshichao" => match event {
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } => {
                let mark = flag(ctx.state, owner, "dddshichao_mark") as usize;
                *player == owner
                    && u01_real_lord_seat(ctx.state).is_some()
                    && mark + 1 <= ctx.state.alive_count()
            }
            EventData::Damage(d) => {
                !d.prevented
                    && d.amount > 0
                    && d.target == owner
                    && d.source.is_some_and(|src| {
                        ctx.state.player(src).alive
                            && flag(ctx.state, owner, &format!("dddshichao_guard:{src}")) != 0
                    })
            }
            _ => false,
        },
        "dddyouxue" => match event {
            EventData::TurnBegin { .. } => {
                let r = ctx.state.turn.round.max(1);
                flag(ctx.state, owner, "dddyouxue_last_round") != r
                    && {
                        let cur_holder = global_youxue_holder(ctx.state).unwrap_or(owner);
                        ctx.state.alive_players().any(|p| p != cur_holder)
                    }
            }
            _ => false,
        },
        "dddchengjing" => matches!(
            event,
            EventData::TurnEndWindow { player, after: false }
            if global_youxue_holder(ctx.state) == Some(*player)
                && dddchengjing_last_qualifying_card(ctx.state, ctx.catalog, *player).is_some()
        ),
        "dddxiaoxing" => matches!(event, EventData::Dying { player, .. } if *player == owner),
        "dddfuyi" => false,
        "dddzhuanshe" => match event {
            EventData::Phase {
                player,
                phase: Phase::Play,
                begin: true,
            } => *player != owner && !ctx.state.player(owner).hand.is_empty(),
            EventData::CardUsed(u) => {
                if u.cancelled || !ctx.state.turn.turn_opened || u.player != ctx.state.turn.current {
                    return Some(false);
                }
                let stamp = turn_stamp_3(ctx.state);
                let eff = u.effective_kind(ctx.catalog);
                let ok_kind =
                    eff.is_basic_kind() || (is_trick_kind_3(eff) && !is_delayed(eff));
                let mult = matches!(
                    eff,
                    CardKind::Taoyuan
                        | CardKind::Wugu
                        | CardKind::SavageAttack
                        | CardKind::ArcheryAttack
                );
                let key_kind = dddzhuanshe_card_key(eff);
                let given_same = flag(
                    ctx.state,
                    owner,
                    &format!("dddzhuanshe_given:{stamp}:{key_kind}"),
                ) != 0;
                ok_kind
                    && !mult
                    && given_same
                    && !virtual_legal_targets(ctx.state, ctx.catalog, u.player, eff)
                        .into_iter()
                        .all(|t| u.targets.contains(&t))
            }
            EventData::TurnEndWindow { player, after: false } => {
                let stamp = turn_stamp_3(ctx.state);
                ctx.state.player(*player).alive
                    && *player != owner
                    && ctx.state.vm_aux.flags.iter().any(|((o, k), &v)| {
                        *o == owner
                            && v != 0
                            && k.starts_with(&format!("dddzhuanshe_given:{stamp}:"))
                            && {
                                let k_id = k.rsplit(':').next().unwrap_or("0");
                                flag(
                                    ctx.state,
                                    *player,
                                    &format!("u01_zhuanshe_used:{stamp}:{k_id}"),
                                ) == 0
                            }
                    })
            }
            _ => false,
        },
        "dddlianer" => false,
        "dddshixing" => match event {
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } => *player == owner,
            EventData::Damage(d) => !d.prevented && d.amount > 0,
            EventData::Recover { amount, .. } | EventData::LoseHp { amount, .. } => *amount > 0,
            EventData::CardUsed(u) => {
                !u.cancelled && is_trick_kind_3(u.effective_kind(ctx.catalog))
            }
            _ => false,
        },
        "xiangle" => matches!(
            event,
            EventData::CardUsed(u)
            if !u.cancelled
                && u.effective_kind(ctx.catalog).is_sha_like()
                && u.targets.contains(&owner)
        ),
        "jiushi" => matches!(
            event,
            EventData::Damage(d)
            if !d.prevented && d.amount > 0 && d.target == owner && ctx.state.player(owner).face_down
        ),
        "ddddanggu" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::End,
                begin: true,
            } if *player == owner && {
                let qun_cnt = ctx
                    .state
                    .alive_players()
                    .filter(|&p| ctx.state.effective_faction(ctx.catalog, p) == Some(crate::catalog::Faction::Qun))
                    .count();
                let linked_cnt = ctx.state.alive_players().filter(|&p| ctx.state.player(p).linked).count();
                let unlinked_cnt = ctx.state.alive_players().filter(|&p| !ctx.state.player(p).linked).count();
                linked_cnt < qun_cnt && unlinked_cnt >= (qun_cnt - linked_cnt)
            }
        ),
        "dddxuanlun" => match event {
            EventData::Damage(d) => !d.prevented && d.amount > 0 && d.target == owner,
            EventData::TurnEndWindow { after: false, .. } => {
                if flag(ctx.state, owner, "dddxuanlun_choose") != 1 {
                    false
                } else {
                    let del = flag(ctx.state, owner, "dddxuanlun_del") != 0;
                    !(del && ctx.state.player(owner).hand.is_empty())
                        && (he_cards(ctx.state, owner).len() >= 4 || del)
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
            } if *player == owner && !dddbailei_eligible_targets(ctx.state).is_empty()
        ),
        _ => false,
    };
    Some(res)
}

pub(crate) fn machine_u01_fix3(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> Option<HostMachine> {
    if !is_u01_fix_3_name(&skill.name) || skill.name == "dddanzhi" {
        return None;
    }
    let mut card = None;
    let mut target = None;
    match (skill.name.as_str(), event) {
        ("dddshichao", EventData::Phase { player, .. }) => {
            card = Some(0);
            target = Some(*player);
        }
        ("dddshichao", EventData::Damage(d)) => {
            card = Some(1);
            target = d.source;
        }
        ("dddzhuanshe", EventData::Phase { player, .. }) => {
            card = Some(0);
            target = Some(*player);
        }
        ("dddzhuanshe", EventData::CardUsed(u)) => {
            card = Some(1);
            target = Some(u.player);
        }
        ("dddzhuanshe", EventData::TurnEndWindow { player, .. }) => {
            card = Some(2);
            target = Some(*player);
        }
        ("dddduanbing", EventData::Damage(d)) => {
            card = Some(1);
            target = Some(d.target);
        }
        ("xiangle", EventData::CardUsed(u)) => {
            card = Some(0);
            target = Some(u.player);
        }
        ("dddxuanlun", EventData::Damage(_)) => {
            card = Some(0);
        }
        ("dddxuanlun", EventData::TurnEndWindow { .. }) => {
            card = Some(1);
        }
        ("dddjiexing", EventData::Recover { .. }) => {
            card = Some(1);
        }
        ("dddjiexing", EventData::Equipped { .. }) => {
            card = Some(2);
        }
        _ => {}
    }
    Some(HostMachine {
        def_key: skill.name.clone(),
        behavior: skill.behavior,
        owner,
        target,
        card,
        cards: Vec::new(),
        targets: Vec::new(),
        stage: HostStage::Act,
    })
}

pub(crate) fn activate_u01_fix3(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<Option<Box<dyn crate::catalog::SkillMachine>>> {
    if !is_u01_fix_3_name(&skill.name) {
        return None;
    }
    if !ctx.state.player(owner).alive
        || ctx.state.turn.phase != Phase::Play
        || ctx.state.turn.current != owner
    {
        return Some(None);
    }
    let tok = play_phase_token(ctx.state);
    let ok = match skill.name.as_str() {
        "dddfengzheng_global" => {
            flag(ctx.state, owner, "dddfengzheng_global_used_tok") != tok
                && !ctx.state.player(owner).hand.is_empty()
                && ctx
                    .catalog
                    .skill_by_name("dddfengzheng")
                    .is_some_and(|d| ctx.state.alive_players().any(|p| ctx.state.has_skill(p, d)))
        }
        "dddfenye" => {
            flag(ctx.state, owner, "dddfenye_used_tok") != tok
                && !ctx.state.player(owner).hand.is_empty()
                && ctx
                    .state
                    .alive_players()
                    .any(|p| p != owner && !ctx.state.player(p).hand.is_empty())
        }
        "dddchengjing" => {
            flag(ctx.state, owner, "dddchengjing_used_tok") != tok
                && !he_cards(ctx.state, owner).is_empty()
                && ctx
                    .state
                    .vm_aux
                    .piles
                    .get(&(owner, "dddchengjing".to_string()))
                    .is_some_and(|v| !v.is_empty())
        }
        "dddduanbing" => dddduanbing_can_use_play(ctx.state, ctx.catalog, owner),
        "jiushi" => !ctx.state.player(owner).face_down,
        "rezhiheng" => {
            flag(ctx.state, owner, "rezhiheng_used_tok") != tok
                && !he_cards(ctx.state, owner).is_empty()
        }
        _ => false,
    };
    Some(ok.then(|| {
        Box::new(HostMachine {
            def_key: skill.name.clone(),
            behavior: skill.behavior,
            owner,
            target: None,
            card: Some(0),
            cards: Vec::new(),
            targets: Vec::new(),
            stage: HostStage::Act,
        }) as Box<dyn crate::catalog::SkillMachine>
    }))
}
