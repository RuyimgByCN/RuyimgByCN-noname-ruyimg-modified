use super::*;

pub(super) fn push_intrinsic_park_3(
    ctx: &mut Ctx<'_>,
    def_key: &str,
    behavior: HostBehavior,
    owner: PlayerId,
    target: Option<PlayerId>,
    card: Option<u32>,
    targets: Vec<PlayerId>,
) {
    ctx.res.parks.push(crate::resolution::Park {
        resume_depth: ctx.res.stack.len(),
        slot: crate::resolution::Slot::Intrinsic {
            frame: crate::resolution::NO_FRAME,
        },
        awaiting_answer: false,
        machine: Box::new(HostMachine {
            def_key: def_key.to_string(),
            behavior,
            owner,
            target,
            card,
            cards: Vec::new(),
            targets,
            stage: HostStage::Act,
        }),
    });
}

pub(super) fn eff_card_kind_3(core: &GameStateCore, catalog: &Catalog, card: CardId) -> CardKind {
    core.declared_kinds
        .get(&card)
        .copied()
        .unwrap_or_else(|| catalog.card_kind(card))
}

pub(super) fn is_trick_kind_3(k: CardKind) -> bool {
    !k.is_basic_kind() && !k.is_equip()
}

pub(super) fn is_black_3(
    core: &GameStateCore,
    catalog: &Catalog,
    holder: PlayerId,
    card: CardId,
) -> bool {
    !effective_is_red(core, catalog, holder, card)
}

pub(super) fn turn_stamp_3(core: &GameStateCore) -> u32 {
    core.turn.round.saturating_mul(100) + u32::from(core.turn.current) + 1
}

pub(super) fn play_phase_token(core: &GameStateCore) -> u32 {
    let seq = flag(core, 0, "play_phase_seq").max(1);
    (seq << 12) ^ turn_stamp_3(core)
}

pub(super) fn u01_confirm_spec(prompt: &'static str, allow_cancel: bool) -> ChoiceSpec {
    ChoiceSpec {
        kind: ChoiceKind::Confirm,
        prompt,
        options: if allow_cancel {
            vec![ChoiceOption::Confirm, ChoiceOption::Pass]
        } else {
            vec![ChoiceOption::Confirm]
        },
        min: 1,
        max: 1,
        allow_cancel,
    }
}

pub(super) fn u01_wait_ask(
    machine: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    next_stage: HostStage,
    ask_id: u32,
) -> Flow {
    set_flag(ctx.core, machine.owner, ASK_KEY, ask_id);
    if let Some((observer, spec)) = asks_c::active_ask_u01_fix3_c(
        ask_id,
        next_stage,
        machine.owner,
        machine.target,
        &machine.targets,
        machine.card,
        &machine.cards,
        ctx.core,
        ctx.catalog,
    )
    .or_else(|| {
        asks_d::active_ask_u01_fix3_d(
            ask_id,
            next_stage,
            machine.owner,
            machine.target,
            &machine.targets,
            machine.card,
            &machine.cards,
            ctx.core,
            ctx.catalog,
        )
    }) {
        machine.stage = next_stage;
        Flow::WaitChoice { observer, spec }
    } else {
        u01_clear_ask(ctx, machine.owner);
        Flow::Done
    }
}

pub(super) fn u01_clear_ask(ctx: &mut Ctx<'_>, owner: PlayerId) {
    ctx.core.vm_aux.flags.remove(&(owner, ASK_KEY.to_string()));
}

pub(super) fn is_u01_real_lord(core: &GameStateCore, p: PlayerId) -> bool {
    match &core.mode {
        crate::state::GameMode::Identity { roles } => {
            roles.get(p as usize) == Some(&crate::state::Role::Lord)
        }
        _ => flag(core, p, "is_zhu") == 1,
    }
}

pub(super) fn u01_real_lord_seat(core: &GameStateCore) -> Option<PlayerId> {
    core.alive_players().find(|&p| is_u01_real_lord(core, p))
}

pub(super) fn dddshichao_nth_group_targets(core: &GameStateCore, mark: usize) -> Vec<PlayerId> {
    let mut counts: Vec<usize> = core
        .alive_players()
        .map(|p| core.player(p).hand.len())
        .collect();
    if counts.is_empty() {
        return Vec::new();
    }
    counts.sort_unstable_by(|a, b| b.cmp(a));
    counts.dedup();
    let target_len = counts
        .get(mark)
        .copied()
        .unwrap_or_else(|| *counts.last().unwrap());
    core.alive_players()
        .filter(|&p| core.player(p).hand.len() == target_len)
        .collect()
}

pub(super) fn global_youxue_holder(core: &GameStateCore) -> Option<PlayerId> {
    let raw = flag(core, 0, "dddyouxue_global_holder");
    if raw > 0 {
        let seat = (raw - 1) as PlayerId;
        if (seat as usize) < core.players.len() && core.player(seat).alive {
            return Some(seat);
        }
    }
    None
}

pub(super) fn set_global_youxue_holder(core: &mut GameStateCore, seat: PlayerId) {
    set_flag(core, 0, "dddyouxue_global_holder", u32::from(seat) + 1);
    let seats = core.players.len() as PlayerId;
    for p in 0..seats {
        if flag(core, p, "dddyouxue_holder") != 0 {
            set_flag(core, p, "dddyouxue_holder", u32::from(seat) + 1);
        }
    }
}

pub(super) fn has_card_used_ancestor(frames: &[crate::event::Frame], frame: &crate::event::Frame) -> bool {
    let mut cur = frame.parent;
    while let Some(pid) = cur {
        let Some(pf) = frames.iter().find(|f| f.id == pid) else {
            break;
        };
        if matches!(pf.data, EventData::CardUsed(_)) {
            return true;
        }
        cur = pf.parent;
    }
    false
}

pub(super) fn dddchengjing_last_qualifying_card(
    core: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
) -> Option<CardId> {
    let stamp = turn_stamp_3(core);
    let len = flag(core, player, &format!("u01_cj_len:{stamp}"));
    for idx in (0..len).rev() {
        let cid = flag(core, player, &format!("u01_cj_card:{stamp}:{idx}"));
        if cid != 0 && core.discard.contains(&cid) {
            let eff = eff_card_kind_3(core, catalog, cid);
            if eff.is_basic_kind() || (is_trick_kind_3(eff) && !is_delayed(eff)) {
                return Some(cid);
            }
        }
    }
    None
}

pub(super) fn dddduanbing_can_use_play(core: &GameStateCore, catalog: &Catalog, owner: PlayerId) -> bool {
    let has_black_he = he_cards(core, owner).into_iter().any(|c| {
        let k = eff_card_kind_3(core, catalog, c);
        is_black_3(core, catalog, owner, c) && (k.is_basic_kind() || k.is_equip())
    });
    let has_bing = core
        .player(owner)
        .judge
        .iter()
        .any(|&jc| eff_card_kind_3(core, catalog, jc) == CardKind::Bing);
    has_black_he && !has_bing
}

pub(super) fn dddshixing_eval_bools(core: &GameStateCore) -> (bool, bool) {
    let bool1 = core.alive_players().any(|p| {
        let pl = core.player(p);
        core.effective_gender(p) == crate::vocabulary::generals::Gender::Female
            && pl.hp >= pl.max_hp as i32
    });
    let bool2 = flag(core, 0, "u01_turn_trick_used_stamp") != turn_stamp_3(core);
    (bool1, bool2)
}

pub(super) fn dddshixing_apply(ctx: &mut Ctx<'_>, owner: PlayerId) {
    if !ctx.core.player(owner).alive {
        return;
    }
    let (bool1, bool2) = dddshixing_eval_bools(ctx.core);
    let want_xiangle = bool1;
    let want_jiushi = bool2;
    let want_rezhiheng = !bool1 && !bool2;
    let old_mask = flag(ctx.core, owner, "dddshixing_granted_mask");
    let mut new_mask = old_mask;
    let specs = [
        (1u32, "xiangle", want_xiangle),
        (2u32, "jiushi", want_jiushi),
        (4u32, "rezhiheng", want_rezhiheng),
    ];
    for (bit, sname, want) in specs {
        let Some(def) = ctx.catalog.skill_by_name(sname) else {
            continue;
        };
        if want {
            if !ctx.core.has_skill(owner, def) {
                ctx.core.skills.insert(
                    (owner, def),
                    crate::state::SkillInstance {
                        owner,
                        enabled: true,
                        once_used: false,
                        expires_end_of_turn: false,
                        rounds: 0,
                    },
                );
                new_mask |= bit;
            }
        } else if (old_mask & bit) != 0 {
            ctx.core.skills.remove(&(owner, def));
            new_mask &= !bit;
        }
    }
    set_flag(ctx.core, owner, "dddshixing_granted_mask", new_mask);
    ctx.log(LogEvent::SkillTriggered {
        skill: "dddshixing",
        owner,
        presentation: None,
    });
}

pub(super) fn dddzhuanshe_card_key(kind: CardKind) -> u32 {
    if kind.is_sha_like() {
        CardKind::Sha as u32
    } else {
        kind as u32
    }
}

pub(super) fn is_strict_max_equip(core: &GameStateCore, p: PlayerId) -> bool {
    let eq = core.player(p).equips.len();
    core.alive_players()
        .all(|other| other == p || core.player(other).equips.len() < eq)
}

pub(super) fn dddbailei_eligible_targets(core: &GameStateCore) -> Vec<PlayerId> {
    core.alive_players()
        .filter(|&p| {
            let pl = core.player(p);
            (is_strict_max_equip(core, p) && (!pl.hand.is_empty() || !pl.equips.is_empty()))
                || pl.hp == 1
        })
        .collect()
}

pub(super) fn u01_remove_owned_card_quiet(ctx: &mut Ctx<'_>, from: PlayerId, card: CardId) {
    if ctx.core.player(from).hand.contains(&card) {
        ctx.core.player_mut(from).hand.retain(|&c| c != card);
    } else if ctx.core.player(from).equips.contains(&card) {
        ctx.core.player_mut(from).equips.retain(|&c| c != card);
        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, from, card);
    }
}

pub(super) fn flush_yulv_discard_queue(ctx: &mut Ctx<'_>) {
    let q_len = ctx
        .core
        .vm_aux
        .flags
        .remove(&(0, "u01_yulv_q_len".to_string()))
        .unwrap_or(0);
    if q_len == 0 {
        return;
    }
    let mut batch = Vec::with_capacity(q_len as usize);
    for idx in 0..q_len {
        if let Some(cid) = ctx
            .core
            .vm_aux
            .flags
            .remove(&(0, format!("u01_yulv_q:{idx}")))
        {
            if cid != 0 && ctx.core.discard.contains(&cid) {
                batch.push(cid);
            }
        }
    }
    if batch.is_empty() || !ctx.core.turn.turn_opened {
        return;
    }
    let cur = ctx.core.turn.current;
    if !ctx.core.player(cur).alive {
        return;
    }
    let n = ctx.core.players.len() as PlayerId;
    for off in (0..n).rev() {
        let owner = (cur + off) % n;
        if !ctx.core.player(owner).alive || ctx.catalog.is_tieji_sealed(ctx.core, owner) {
            continue;
        }
        if ctx
            .catalog
            .skill_by_name("dddyulv")
            .is_some_and(|d| ctx.core.has_skill(owner, d))
        {
            if let Some(&pile_top) = ctx
                .core
                .vm_aux
                .piles
                .get(&(owner, "dddyulv".to_string()))
                .and_then(|v| v.first())
            {
                let target_rank = ctx.catalog.cards[pile_top as usize].rank;
                if batch
                    .iter()
                    .any(|&c| ctx.catalog.cards[c as usize].rank == target_rank)
                {
                    push_intrinsic_park_3(
                        ctx,
                        "dddyulv",
                        HostBehavior::U01DddBatchC,
                        owner,
                        Some(cur),
                        Some(1),
                        Vec::new(),
                    );
                }
            }
        }
        if ctx
            .catalog
            .skill_by_name("dddyulv_old")
            .is_some_and(|d| ctx.core.has_skill(owner, d))
        {
            let target_rank = flag(ctx.core, owner, "dddyulv_old_rank") as u8;
            if target_rank > 0
                && batch
                    .iter()
                    .any(|&c| ctx.catalog.cards[c as usize].rank == target_rank)
            {
                push_intrinsic_park_3(
                    ctx,
                    "dddyulv_old",
                    HostBehavior::U01DddBatchC,
                    owner,
                    Some(cur),
                    Some(1),
                    Vec::new(),
                );
            }
        }
    }
}
