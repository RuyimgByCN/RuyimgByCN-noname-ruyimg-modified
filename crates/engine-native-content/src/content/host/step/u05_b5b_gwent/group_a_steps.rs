//! U05-5b Batch A 技能结算步进（`gwjinli*`..`gwyewu`）。

use super::group_a::refund_turn_limit;
use super::group_a_asks::*;
use super::*;
use crate::event::DamageNature;
use crate::state::SkillInstance;

pub(super) fn step_gwjinli(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_GWJINLI_CARD, build_ask_group_a),
        HostStage::SelectCard => {
            clear_ask_id(ctx.core, owner);
            let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                refund_turn_limit(ctx, owner, &m.def_key);
                return Flow::Done;
            };
            m.card = Some(c);
            wait_ask(
                m,
                ctx,
                HostStage::TargetChoice,
                ASK_GWJINLI_TARGET,
                build_ask_group_a,
            )
        }
        HostStage::TargetChoice => {
            clear_ask_id(ctx.core, owner);
            let (Some(c), Some(ChoiceOption::Player(t))) = (m.card, ctx.answer().first().copied())
            else {
                refund_turn_limit(ctx, owner, &m.def_key);
                return Flow::Done;
            };
            if !ctx.core.player(owner).hand.contains(&c) {
                refund_turn_limit(ctx, owner, &m.def_key);
                return Flow::Done;
            }
            ctx.discard_from_hand(owner, c);
            if ctx.core.player(t).alive {
                set_flag(ctx.core, t, "gwjinli_jiu_ticks", 2);
                if let Some(def) = ctx.catalog.skill_by_name("gwjinli_jiu") {
                    ctx.insert_skill(
                        (t, def),
                        SkillInstance {
                            owner: t,
                            enabled: true,
                            once_used: false,
                            expires_end_of_turn: false,
                            rounds: 0,
                        },
                    );
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn step_gwjinli_jiu(owner: PlayerId, ctx: &mut Ctx<'_>) -> Flow {
    if ctx.core.player(owner).alive {
        apply_buff(ctx, owner);
    }
    let left = flag(ctx.core, owner, "gwjinli_jiu_ticks").saturating_sub(1);
    set_flag(ctx.core, owner, "gwjinli_jiu_ticks", left);
    if left == 0 {
        if let Some(def) = ctx.catalog.skill_by_name("gwjinli_jiu") {
            ctx.remove_skill(owner, def);
        }
        let _ = give_or_draw_card_named(ctx, owner, "gw_xianzumaijiu");
    }
    Flow::Done
}

pub(super) fn step_gwliaotian(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            let hs = &ctx.core.player(owner).hand;
            if hs.len() < 2 {
                return Flow::Done;
            }
            let first_red = is_red_card(ctx.catalog, hs[0]);
            if !hs.iter().all(|&c| is_red_card(ctx.catalog, c) == first_red) {
                return Flow::Done;
            }
            let recast: Vec<CardId> = hs
                .iter()
                .copied()
                .filter(|&c| {
                    !crate::kernel::is_gw_dudayuanshuai2_locked(ctx.core, ctx.catalog, owner, c)
                })
                .collect();
            if recast.is_empty() {
                return Flow::Done;
            }
            let used = flag(ctx.core, owner, "gw_use2_cnt:gwliaotian");
            set_flag(ctx.core, owner, "gw_use2_cnt:gwliaotian", used + 1);
            let n = recast.len() as u32;
            for c in recast {
                ctx.discard_from_hand(owner, c);
            }
            m.card = Some(n);
            m.stage = HostStage::SelectCard;
            Flow::Yield
        }
        HostStage::SelectCard => {
            let n = m.card.take().unwrap_or(0);
            if n > 0 && ctx.core.player(owner).alive {
                ctx.start_draw(owner, n);
                m.stage = HostStage::DrawSettle;
                return Flow::Yield;
            }
            Flow::Done
        }
        HostStage::DrawSettle => {
            if !ctx.core.player(owner).alive {
                return Flow::Done;
            }
            let enemies: Vec<PlayerId> = enemies_of(ctx.core, ctx.catalog, owner)
                .into_iter()
                .filter(|&e| {
                    !crate::kernel::sha_user_or_target_blocked(ctx.core, ctx.catalog, owner, e)
                })
                .collect();
            if !enemies.is_empty() {
                let idx = ctx.core.rng.gen_range(enemies.len() as u32) as usize;
                let t = enemies[idx];
                set_flag(ctx.core, owner, "vm_va_sha_no_count", 1);
                ctx.emit(EventData::CardUsed(crate::event::CardUseData {
                    card: None,
                    player: owner,
                    targets: vec![t],
                    nature: DamageNature::Normal,
                    use_as: Some(CardKind::Sha),
                    cancelled: false,
                }));
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn step_gwhuanbi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_GWHUANBI_COST, build_ask_group_a),
        HostStage::SelectCard => {
            clear_ask_id(ctx.core, owner);
            let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                refund_turn_limit(ctx, owner, &m.def_key);
                return Flow::Done;
            };
            discard_he_card(ctx, owner, c);
            let mut pool: Vec<usize> = (0..GWMAOXIAN_NAMES.len()).collect();
            let mut n = 0u32;
            while n < 3 && !pool.is_empty() {
                let idx = ctx.core.rng.gen_range(pool.len() as u32) as usize;
                let tag = pool.swap_remove(idx);
                set_flag(ctx.core, owner, &format!("gw_huanbi_opt:{n}"), tag as u32 + 1);
                n += 1;
            }
            set_flag(ctx.core, owner, "gw_huanbi_n", n);
            wait_ask(
                m,
                ctx,
                HostStage::SelectNature,
                ASK_GWHUANBI_PICK,
                build_ask_group_a,
            )
        }
        HostStage::SelectNature => {
            clear_ask_id(ctx.core, owner);
            let Some(ChoiceOption::Tag(tag)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            let Some(&mname) = GWMAOXIAN_NAMES.get(tag as usize) else {
                return Flow::Done;
            };
            m.card = Some(u32::from(tag));
            let _ = give_or_draw_card_named(ctx, owner, mname);
            let candidates: Vec<PlayerId> = ctx
                .core
                .seat_order_from(owner)
                .into_iter()
                .filter(|&p| p != owner && ctx.core.player(p).alive && !ctx.core.player(p).hand.is_empty())
                .collect();
            if candidates.is_empty() {
                return Flow::Done;
            }
            let t = candidates[ctx.core.rng.gen_range(candidates.len() as u32) as usize];
            m.target = Some(t);
            wait_ask(
                m,
                ctx,
                HostStage::CardsAsk,
                ASK_GWHUANBI_GIVE,
                build_ask_group_a,
            )
        }
        HostStage::CardsAsk => {
            clear_ask_id(ctx.core, owner);
            let (Some(t), Some(ChoiceOption::Card(give_c))) = (m.target, ctx.answer().first().copied())
            else {
                return Flow::Done;
            };
            if ctx.core.player(t).alive && ctx.core.player(t).hand.contains(&give_c) {
                ctx.give_hand_cards(t, owner, &[give_c]);
                if let Some(mname) = m.card.and_then(|idx| GWMAOXIAN_NAMES.get(idx as usize)).copied() {
                    if !give_or_draw_card_named(ctx, t, mname) {
                        let _ = pull_gwent_or_vanished(ctx, |cid| {
                            clean_gwent_name(ctx.catalog.cards[cid as usize].name)
                                .starts_with("gwmaoxian_")
                        })
                        .map(|cid| {
                            ctx.core.player_mut(t).hand.push(cid);
                            ctx.emit(EventData::CardsDrawn {
                                player: t,
                                cards: vec![cid],
                            });
                        });
                    }
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn step_gwminxiang(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(
            m,
            ctx,
            HostStage::SelectCard,
            ASK_GWMINXIANG_CARD,
            build_ask_group_a,
        ),
        HostStage::SelectCard => {
            clear_ask_id(ctx.core, owner);
            let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                refund_turn_limit(ctx, owner, &m.def_key);
                return Flow::Done;
            };
            m.card = Some(c);
            wait_ask(
                m,
                ctx,
                HostStage::TargetChoice,
                ASK_GWMINXIANG_TARGETS,
                build_ask_group_a,
            )
        }
        HostStage::TargetChoice => {
            clear_ask_id(ctx.core, owner);
            let Some(c) = m.card else {
                return Flow::Done;
            };
            let picked: Vec<PlayerId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Player(p) => Some(p),
                    _ => None,
                })
                .take(2)
                .collect();
            if picked.len() < 2 {
                refund_turn_limit(ctx, owner, &m.def_key);
                return Flow::Done;
            }
            m.targets = picked;
            ctx.discard_from_hand(owner, c);
            m.stage = HostStage::SelectNature;
            Flow::Yield
        }
        HostStage::SelectNature => {
            m.stage = HostStage::DrawSettle;
            ctx.start_draw(owner, 1);
            Flow::Yield
        }
        HostStage::DrawSettle => {
            let Some(c) = m.card else { return Flow::Done };
            let kind = ctx.catalog.card_kind(c);
            let nature = match kind {
                CardKind::FireSha => DamageNature::Fire,
                CardKind::LeiSha => DamageNature::Thunder,
                _ => DamageNature::Normal,
            };
            if m.targets.len() == 2
                && ctx.core.player(m.targets[0]).alive
                && ctx.core.player(m.targets[1]).alive
            {
                mark_sha_no_count_if_sha(ctx.core, ctx.catalog, m.targets[0], c);
                m.stage = HostStage::CompareTargetPick;
                ctx.emit(EventData::CardUsed(crate::event::CardUseData {
                    card: None,
                    player: m.targets[0],
                    targets: vec![m.targets[1]],
                    nature,
                    use_as: Some(kind),
                    cancelled: false,
                }));
                return Flow::Yield;
            }
            Flow::Done
        }
        HostStage::CompareTargetPick => {
            let Some(c) = m.card else { return Flow::Done };
            let kind = ctx.catalog.card_kind(c);
            let nature = match kind {
                CardKind::FireSha => DamageNature::Fire,
                CardKind::LeiSha => DamageNature::Thunder,
                _ => DamageNature::Normal,
            };
            if m.targets.len() == 2
                && ctx.core.player(m.targets[0]).alive
                && ctx.core.player(m.targets[1]).alive
            {
                mark_sha_no_count_if_sha(ctx.core, ctx.catalog, m.targets[1], c);
                ctx.emit(EventData::CardUsed(crate::event::CardUseData {
                    card: None,
                    player: m.targets[1],
                    targets: vec![m.targets[0]],
                    nature,
                    use_as: Some(kind),
                    cancelled: false,
                }));
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn step_gwlangshi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(
            m,
            ctx,
            HostStage::TargetChoice,
            ASK_GWLANGSHI_TARGET,
            build_ask_group_a,
        ),
        HostStage::TargetChoice => {
            clear_ask_id(ctx.core, owner);
            if let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() {
                if ctx.core.player(t).alive {
                    set_flag(ctx.core, owner, "gw_langshi_in_effect", 1);
                    m.stage = HostStage::DrawSettle;
                    ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                        Some(owner),
                        t,
                        1,
                        DamageNature::Normal,
                        None,
                    )));
                    return Flow::Yield;
                }
            }
            Flow::Done
        }
        HostStage::DrawSettle => {
            set_flag(ctx.core, owner, "gw_langshi_in_effect", 0);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn step_gwjingtian(owner: PlayerId, ctx: &mut Ctx<'_>) -> Flow {
    let cur = flag(ctx.core, owner, "gwjingtian_marks");
    if cur == 0 || has_named_skill(ctx.core, ctx.catalog, owner, "gwjingtian2") {
        return Flow::Done;
    }
    set_flag(ctx.core, owner, "gwjingtian_marks", cur - 1);
    sync_mark(ctx.core, ctx.catalog, owner, "gwjingtian", (cur - 1) as i32);
    if let Some(def) = ctx.catalog.skill_by_name("gwjingtian2") {
        ctx.add_skill_until_turn_end(owner, def);
    }
    ctx.start_draw(owner, 1);
    Flow::Done
}

pub(super) fn step_gwjingshi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(
            m,
            ctx,
            HostStage::TargetChoice,
            ASK_GWJINGSHI_GUESS,
            build_ask_group_a,
        ),
        HostStage::TargetChoice => {
            clear_ask_id(ctx.core, owner);
            let Some(ChoiceOption::Player(guess)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            let counts: Vec<(PlayerId, usize)> = ctx
                .core
                .alive_players()
                .filter(|&p| !ctx.core.player(p).hand.is_empty())
                .map(|p| {
                    let b = ctx
                        .core
                        .player(p)
                        .hand
                        .iter()
                        .filter(|&&c| !is_red_card(ctx.catalog, c))
                        .count();
                    (p, b)
                })
                .collect();
            let max_black = counts.iter().map(|&(_, b)| b).max().unwrap_or(0);
            let hit = counts.iter().any(|&(p, b)| p == guess && b == max_black);
            if !hit
                || !ctx
                    .core
                    .alive_players()
                    .any(|p| p != owner && !ctx.core.player(p).hand.is_empty())
            {
                return Flow::Done;
            }
            wait_ask(
                m,
                ctx,
                HostStage::SelectCard,
                ASK_GWJINGSHI_STEAL,
                build_ask_group_a,
            )
        }
        HostStage::SelectCard => {
            clear_ask_id(ctx.core, owner);
            let Some(ChoiceOption::Card(stolen)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            let holder = ctx
                .core
                .alive_players()
                .find(|&p| p != owner && ctx.core.player(p).hand.contains(&stolen));
            if let Some(h) = holder {
                ctx.give_hand_cards(h, owner, &[stolen]);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn step_gwgouhun(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(
            m,
            ctx,
            HostStage::SelectCard,
            ASK_GWGOUHUN_CARD,
            build_ask_group_a,
        ),
        HostStage::SelectCard => {
            clear_ask_id(ctx.core, owner);
            let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                refund_turn_limit(ctx, owner, &m.def_key);
                return Flow::Done;
            };
            m.card = Some(u32::from(suit_to_tag(ctx.catalog.cards[c as usize].suit)));
            m.cards = vec![c];
            wait_ask(
                m,
                ctx,
                HostStage::TargetChoice,
                ASK_GWGOUHUN_TARGET,
                build_ask_group_a,
            )
        }
        HostStage::TargetChoice => {
            clear_ask_id(ctx.core, owner);
            let (Some(&c), Some(ChoiceOption::Player(t))) =
                (m.cards.first(), ctx.answer().first().copied())
            else {
                refund_turn_limit(ctx, owner, &m.def_key);
                return Flow::Done;
            };
            m.target = Some(t);
            if ctx.core.player(owner).hand.contains(&c) {
                ctx.give_hand_cards(owner, t, &[c]);
            } else {
                refund_turn_limit(ctx, owner, &m.def_key);
                return Flow::Done;
            }
            if !ctx.core.player(t).alive || ctx.core.player(t).hand.is_empty() {
                set_flag(ctx.core, t, "gw_gouhun_mad", 1);
                ctx.core.player_mut(t).boss_mad = true;
                return Flow::Done;
            }
            wait_ask(
                m,
                ctx,
                HostStage::CardsAsk,
                ASK_GWGOUHUN_REPLY,
                build_ask_group_a,
            )
        }
        HostStage::CardsAsk => {
            clear_ask_id(ctx.core, owner);
            let (Some(t), Some(stag)) = (m.target, m.card.map(|v| v as u8)) else {
                return Flow::Done;
            };
            let Some(ChoiceOption::Tag(picked)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            match picked {
                0 => {
                    let matching: Vec<CardId> = ctx
                        .core
                        .player(t)
                        .hand
                        .iter()
                        .copied()
                        .filter(|&c| suit_to_tag(ctx.catalog.cards[c as usize].suit) == stag)
                        .collect();
                    if !matching.is_empty() {
                        ctx.give_hand_cards(t, owner, &matching);
                    }
                }
                1 => {
                    let non_matching: Vec<CardId> = ctx
                        .core
                        .player(t)
                        .hand
                        .iter()
                        .copied()
                        .filter(|&c| {
                            suit_to_tag(ctx.catalog.cards[c as usize].suit) != stag
                                && !crate::kernel::is_gw_dudayuanshuai2_locked(
                                    ctx.core,
                                    ctx.catalog,
                                    t,
                                    c,
                                )
                        })
                        .collect();
                    for c in non_matching {
                        ctx.discard_from_hand(t, c);
                    }
                }
                _ => {
                    set_flag(ctx.core, t, "gw_gouhun_mad", 1);
                    ctx.core.player_mut(t).boss_mad = true;
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn step_gwfutian(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            let cur_fid = ctx.current_event_frame_id();
            if let Some(frame) = ctx.res.frame_mut(cur_fid) {
                if let EventData::Damage(d) = &mut frame.data {
                    d.prevented = true;
                    return Flow::Done;
                }
            }
            wait_ask(
                m,
                ctx,
                HostStage::TargetChoice,
                ASK_GWFUTIAN_TARGET,
                build_ask_group_a,
            )
        }
        HostStage::TargetChoice => {
            clear_ask_id(ctx.core, owner);
            let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            if !ctx.core.player(t).alive || ctx.core.player(t).hand.is_empty() {
                return Flow::Done;
            }
            m.target = Some(t);
            wait_ask(
                m,
                ctx,
                HostStage::SelectCard,
                ASK_GWFUTIAN_DISCARD,
                build_ask_group_a,
            )
        }
        HostStage::SelectCard => {
            clear_ask_id(ctx.core, owner);
            let (Some(t), Some(ChoiceOption::Card(c))) = (m.target, ctx.answer().first().copied())
            else {
                return Flow::Done;
            };
            if ctx.core.player(t).hand.contains(&c) {
                ctx.discard_from_hand(t, c);
                let rank = u32::from(ctx.catalog.cards[c as usize].rank);
                let sum = flag(ctx.core, owner, "gwfutian_sum") + rank;
                set_flag(ctx.core, owner, "gwfutian_sum", sum);
                sync_mark(ctx.core, ctx.catalog, owner, "gwfutian", sum as i32);
                if sum >= 24 {
                    if let Some(old_def) = ctx.catalog.skill_by_name("gwfutian") {
                        ctx.remove_skill(owner, old_def);
                    }
                    if let Some(new_def) = ctx.catalog.skill_by_name("gwzhongmo") {
                        ctx.insert_skill(
                            (owner, new_def),
                            SkillInstance {
                                owner,
                                enabled: true,
                                once_used: false,
                                expires_end_of_turn: false,
                                rounds: 0,
                            },
                        );
                    }
                    let p = ctx.core.player_mut(owner);
                    p.max_hp += 4;
                    p.hp = p.max_hp as i32;
                    p.dying = false;
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn step_gwyewu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage == HostStage::Act {
        return wait_ask(m, ctx, HostStage::SelectCard, ASK_GWYEWU_COST, build_ask_group_a);
    }
    clear_ask_id(ctx.core, owner);
    let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
        refund_turn_limit(ctx, owner, &m.def_key);
        return Flow::Done;
    };
    let target_red = is_red_card(ctx.catalog, c);
    ctx.discard_from_hand(owner, c);
    let mut nh = 0u32;
    let mut ne = 0u32;
    loop {
        let pool: Vec<PlayerId> = enemies_of(ctx.core, ctx.catalog, owner)
            .into_iter()
            .filter(|&e| !discardable_hand_and_equips(ctx.core, ctx.catalog, e).is_empty())
            .collect();
        if pool.is_empty() {
            break;
        }
        let e = pool[ctx.core.rng.gen_range(pool.len() as u32) as usize];
        let he = discardable_hand_and_equips(ctx.core, ctx.catalog, e);
        let picked = he[ctx.core.rng.gen_range(he.len() as u32) as usize];
        let is_equip = ctx.core.player(e).equips.contains(&picked);
        let same_color = is_red_card(ctx.catalog, picked) == target_red;
        discard_he_card(ctx, e, picked);
        if is_equip {
            ne += 1;
        } else {
            nh += 1;
        }
        if !same_color {
            break;
        }
    }
    for _ in 0..nh {
        let _ = give_or_draw_card_named(ctx, owner, "gw_wuyao");
    }
    for _ in 0..ne {
        let _ = give_or_draw_card_named(ctx, owner, "gw_lang");
    }
    Flow::Done
}

pub(super) fn step_g_gw_yewu(owner: PlayerId, ctx: &mut Ctx<'_>) -> Flow {
    let vanish: Vec<CardId> = ctx
        .core
        .player(owner)
        .hand
        .iter()
        .copied()
        .filter(|&c| matches!(ctx.catalog.cards[c as usize].name, "gw_wuyao" | "gw_lang"))
        .collect();
    for c in vanish {
        vanish_card_from_hand(ctx, owner, c);
    }
    Flow::Done
}

