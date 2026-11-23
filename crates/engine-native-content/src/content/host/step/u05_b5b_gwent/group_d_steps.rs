//! U05-5b Batch D 技能结算步进（`nuhou`..`lanquan`）。

use super::group_a::refund_turn_limit;
use super::group_d_asks::*;
use super::*;
use crate::event::DamageNature;
use crate::state::SkillInstance;

pub(super) fn step_group_d(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.def_key.as_str() {
        "nuhou" => step_nuhou(m, ctx),
        "shewu" => step_shewu(m, ctx),
        "gwzhanjiang" => step_gwzhanjiang(m, ctx),
        "gwzhanjiang2" => {
            if let Some(def) = ctx.catalog.skill_by_name("gwzhanjiang2") {
                ctx.remove_skill(m.owner, def);
            }
            Flow::Done
        }
        "gwchuanxin" => step_gwchuanxin(m, ctx),
        "fengjian" => step_fengjian(m, ctx),
        "huandie" => step_huandie(m, ctx),
        "huandie_discard" => step_huandie_discard(m, ctx),
        "xuezhou" | "xuezhou_hp" => step_xuezhou(m, ctx),
        "fayin" => step_fayin(m, ctx),
        "gwbaquan" => step_gwbaquan(m, ctx),
        "huihun" => step_huihun(m, ctx),
        "lanquan" => step_lanquan(m, ctx),
        _ => Flow::Done,
    }
}

fn step_nuhou(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(
            m,
            ctx,
            HostStage::SelectCard,
            ASK_NUHOU_COST,
            build_ask_group_d,
        ),
        HostStage::SelectCard => {
            clear_ask_id(ctx.core, owner);
            let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            discard_he_card(ctx, owner, c);
            let enemies = enemies_of(ctx.core, ctx.catalog, owner);
            if !enemies.is_empty() {
                let t = enemies[ctx.core.rng.gen_range(enemies.len() as u32) as usize];
                m.target = Some(t);
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
            Flow::Done
        }
        HostStage::DrawSettle => {
            if let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) {
                let he = discardable_hand(ctx.core, ctx.catalog, t);
                if !he.is_empty() {
                    let idx = ctx.core.rng.gen_range(he.len() as u32) as usize;
                    ctx.discard_from_hand(t, he[idx]);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_shewu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(
            m,
            ctx,
            HostStage::SelectCard,
            ASK_SHEWU_COST,
            build_ask_group_d,
        ),
        HostStage::SelectCard => {
            clear_ask_id(ctx.core, owner);
            let chosen: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Card(c) => Some(c),
                    _ => None,
                })
                .take(3)
                .collect();
            if chosen.is_empty() {
                refund_turn_limit(ctx, owner, &m.def_key);
                return Flow::Done;
            }
            m.card = Some(chosen.len() as u32);
            for c in chosen {
                discard_he_card(ctx, owner, c);
            }
            m.stage = HostStage::DrawSettle;
            Flow::Yield
        }
        HostStage::DrawSettle => {
            let count = m.card.unwrap_or(0);
            if count >= 2 && ctx.core.player(owner).alive {
                set_flag(ctx.core, owner, "u05_mod:shewu_dist", 1);
                if let Some(def) = ctx.catalog.skill_by_name("shewu_dist") {
                    ctx.add_skill_until_turn_end(owner, def);
                }
            }
            if count == 3 && ctx.core.player(owner).alive {
                ctx.recover(owner, 1, None);
                m.stage = HostStage::SelectNature;
                return Flow::Yield;
            }
            if ctx.core.player(owner).alive {
                ctx.start_draw(owner, 3);
            }
            Flow::Done
        }
        HostStage::SelectNature => {
            if ctx.core.player(owner).alive {
                ctx.start_draw(owner, 3);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_gwzhanjiang(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(
            m,
            ctx,
            HostStage::SelectCard,
            ASK_GWZHANJIANG_COST,
            build_ask_group_d,
        ),
        HostStage::SelectCard => {
            clear_ask_id(ctx.core, owner);
            let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            discard_he_card(ctx, owner, c);
            if let Some(def) = ctx.catalog.skill_by_name("gwzhanjiang2") {
                ctx.insert_skill(
                    (owner, def),
                    SkillInstance {
                        owner,
                        enabled: true,
                        once_used: false,
                        expires_end_of_turn: false,
                        rounds: 0,
                    },
                );
            }
            let Some(turn_p) = m.target else { return Flow::Done };
            m.targets = ctx
                .core
                .seat_order_from(turn_p)
                .into_iter()
                .filter(|&p| {
                    p != turn_p
                        && ctx.core.player(p).alive
                        && !hand_sha_cards(ctx.core, ctx.catalog, p).is_empty()
                        && !crate::kernel::sha_user_or_target_blocked(
                            ctx.core,
                            ctx.catalog,
                            p,
                            turn_p,
                        )
                })
                .collect();
            m.card = Some(0);
            advance_gwzhanjiang_queue(m, ctx)
        }
        HostStage::CardsAsk => {
            clear_ask_id(ctx.core, owner);
            let shooter = if m.targets.is_empty() {
                return Flow::Done;
            } else {
                m.targets.remove(0)
            };
            let turn_p = m.target.unwrap_or(owner);
            if let Some(ChoiceOption::Card(sha)) = ctx.answer().first().copied() {
                if ctx.core.player(shooter).hand.contains(&sha)
                    && ctx.core.player(turn_p).alive
                    && !crate::kernel::sha_user_or_target_blocked(
                        ctx.core,
                        ctx.catalog,
                        shooter,
                        turn_p,
                    )
                {
                    ctx.take_hand_card(shooter, sha);
                    ctx.note_cards_lost(shooter, &[sha]);
                    ctx.core.table.push(sha);
                    let used = m.card.unwrap_or(0) + 1;
                    m.card = Some(used);
                    m.stage = HostStage::DrawSettle;
                    set_flag(ctx.core, shooter, "vm_va_sha_no_count", 1);
                    let use_as = if ctx.catalog.card_kind(sha) == CardKind::Wuzhong {
                        Some(CardKind::Sha)
                    } else {
                        None
                    };
                    ctx.emit(EventData::CardUsed(crate::machine::card_use_as(
                        sha,
                        shooter,
                        vec![turn_p],
                        use_as,
                    )));
                    ctx.start_draw(shooter, 1);
                    return Flow::Yield;
                }
            }
            advance_gwzhanjiang_queue(m, ctx)
        }
        HostStage::DrawSettle => {
            if m.card.unwrap_or(0) >= 2 {
                return Flow::Done;
            }
            advance_gwzhanjiang_queue(m, ctx)
        }
        _ => Flow::Done,
    }
}

fn advance_gwzhanjiang_queue(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let turn_p = m.target.unwrap_or(owner);
    if !ctx.core.player(turn_p).alive {
        return Flow::Done;
    }
    while let Some(&cand) = m.targets.first() {
        if !ctx.core.player(cand).alive
            || hand_sha_cards(ctx.core, ctx.catalog, cand).is_empty()
            || crate::kernel::sha_user_or_target_blocked(ctx.core, ctx.catalog, cand, turn_p)
        {
            m.targets.remove(0);
            continue;
        }
        return wait_ask(
            m,
            ctx,
            HostStage::CardsAsk,
            ASK_GWZHANJIANG_SHA,
            build_ask_group_d,
        );
    }
    Flow::Done
}

fn step_gwchuanxin(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage == HostStage::Act {
        return wait_ask(
            m,
            ctx,
            HostStage::SelectNature,
            ASK_GWCHUANXIN_CONFIRM,
            build_ask_group_d,
        );
    }
    clear_ask_id(ctx.core, owner);
    let Some(ChoiceOption::Tag(_)) = ctx.answer().first().copied() else {
        return Flow::Done;
    };
    let top = take_top_deck_cards(ctx.core, 1);
    let Some(&cid) = top.first() else {
        return Flow::Done;
    };
    let black = !is_red_card(ctx.catalog, cid);
    ctx.to_discard(cid);
    if black {
        if let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) {
            ctx.log(LogEvent::SkillTriggered {
                skill: "gwchuanxin",
                owner,
                presentation: None,
            });
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
    }
    Flow::Done
}

fn step_fengjian(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage == HostStage::Act {
        return wait_ask(
            m,
            ctx,
            HostStage::TargetChoice,
            ASK_FENGJIAN_TARGET,
            build_ask_group_d,
        );
    }
    clear_ask_id(ctx.core, owner);
    let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
        return Flow::Done;
    };
    if !ctx.core.player(t).alive
        || crate::kernel::sha_user_or_target_blocked(ctx.core, ctx.catalog, owner, t)
    {
        return Flow::Done;
    }
    ctx.core.player_mut(owner).yinshen_active = true;
    set_flag(ctx.core, owner, "vm_va_sha_no_count", 1);
    ctx.emit(EventData::CardUsed(crate::event::CardUseData {
        card: None,
        player: owner,
        targets: vec![t],
        nature: DamageNature::Thunder,
        use_as: Some(CardKind::LeiSha),
        cancelled: false,
    }));
    Flow::Done
}

fn step_huandie(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(
            m,
            ctx,
            HostStage::SelectNature,
            ASK_HUANDIE_CONFIRM,
            build_ask_group_d,
        ),
        HostStage::SelectNature => {
            clear_ask_id(ctx.core, owner);
            if !ctx.answer().contains(&ChoiceOption::Confirm) {
                return Flow::Done;
            }
            if ctx.core.alive_players().any(|p| p != owner) {
                return wait_ask(
                    m,
                    ctx,
                    HostStage::TargetChoice,
                    ASK_HUANDIE_TARGETS,
                    build_ask_group_d,
                );
            }
            if ctx.core.player(owner).alive {
                ctx.start_draw(owner, 1);
            }
            if let Some(def) = ctx.catalog.skill_by_name("huandie_discard") {
                ctx.add_skill_until_turn_end(owner, def);
            }
            Flow::Done
        }
        HostStage::TargetChoice => {
            clear_ask_id(ctx.core, owner);
            let chosen: Vec<PlayerId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Player(p) if p != owner => Some(p),
                    _ => None,
                })
                .collect();
            let ordered: Vec<PlayerId> = ctx
                .core
                .seat_order_from(owner)
                .into_iter()
                .filter(|p| chosen.contains(p))
                .collect();
            for p in ordered.into_iter().rev() {
                if ctx.core.player(p).alive {
                    ctx.start_draw(p, 2);
                }
            }
            if ctx.core.player(owner).alive {
                ctx.start_draw(owner, 1);
            }
            if let Some(def) = ctx.catalog.skill_by_name("huandie_discard") {
                ctx.add_skill_until_turn_end(owner, def);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_huandie_discard(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            m.targets = ctx
                .core
                .seat_order_from(owner)
                .into_iter()
                .filter(|&p| {
                    ctx.core.player(p).alive
                        && ctx.core.player(p).hand.len() > ctx.core.player(p).hp.max(0) as usize
                })
                .collect();
            advance_huandie_discard(m, ctx)
        }
        HostStage::SelectCard => {
            clear_ask_id(ctx.core, owner);
            if !m.targets.is_empty() {
                let victim = m.targets.remove(0);
                let picked: Vec<CardId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match *o {
                        ChoiceOption::Card(c) => Some(c),
                        _ => None,
                    })
                    .take(2)
                    .collect();
                for c in picked {
                    if ctx.core.player(victim).hand.contains(&c)
                        && !crate::kernel::is_gw_dudayuanshuai2_locked(
                            ctx.core,
                            ctx.catalog,
                            victim,
                            c,
                        )
                    {
                        ctx.discard_from_hand(victim, c);
                    }
                }
            }
            advance_huandie_discard(m, ctx)
        }
        _ => Flow::Done,
    }
}

fn advance_huandie_discard(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    while let Some(&v) = m.targets.first() {
        let has_discardable = ctx.core.player(v).hand.iter().any(|&c| {
            !crate::kernel::is_gw_dudayuanshuai2_locked(ctx.core, ctx.catalog, v, c)
        });
        if !ctx.core.player(v).alive || !has_discardable {
            m.targets.remove(0);
            continue;
        }
        return wait_ask(
            m,
            ctx,
            HostStage::SelectCard,
            ASK_HUANDIE_DISCARD_PICK,
            build_ask_group_d,
        );
    }
    Flow::Done
}

fn step_xuezhou(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(1) || m.def_key == "xuezhou_hp" {
        let cur = ctx.core.turn.current;
        if has_named_skill(ctx.core, ctx.catalog, cur, "xuezhou_hp2") {
            return Flow::Done;
        }
        if let Some(def) = ctx.catalog.skill_by_name("xuezhou_hp2") {
            ctx.add_skill_until_turn_end(cur, def);
        }
        let mode = flag(ctx.core, owner, "gw_xuezhou_mode");
        let dmg_target = m.target.unwrap_or(owner);
        let victim = if mode == 1 {
            Some(dmg_target)
        } else {
            m.targets.first().copied()
        };
        if let Some(v) = victim.filter(|&p| p != owner && ctx.core.player(p).alive) {
            ctx.emit(EventData::LoseHp {
                player: v,
                amount: 1,
            });
            if (mode == 1 || owner != dmg_target) && ctx.core.player(owner).alive {
                ctx.recover(owner, 1, None);
            }
        }
        return Flow::Done;
    }
    if m.stage == HostStage::Act {
        return wait_ask(
            m,
            ctx,
            HostStage::SelectNature,
            ASK_XUEZZHOU_MODE,
            build_ask_group_d,
        );
    }
    clear_ask_id(ctx.core, owner);
    match ctx.answer().first().copied() {
        Some(ChoiceOption::Tag(tag @ 0..=1)) => {
            set_flag(ctx.core, 0, "gw_xuezhou_owner", u32::from(owner) + 1);
            set_flag(ctx.core, owner, "gw_xuezhou_mode", u32::from(tag) + 1);
            sync_mark(ctx.core, ctx.catalog, owner, "xuezhou", i32::from(tag) + 1);
        }
        _ => {
            set_flag(ctx.core, 0, "gw_xuezhou_owner", 0);
            set_flag(ctx.core, owner, "gw_xuezhou_mode", 0);
            sync_mark(ctx.core, ctx.catalog, owner, "xuezhou", 0);
        }
    }
    Flow::Done
}

fn step_fayin(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage == HostStage::DrawSettle {
        if let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) {
            ctx.start_draw(t, 1);
        }
        return Flow::Done;
    }
    if m.stage == HostStage::Act {
        return wait_ask(
            m,
            ctx,
            HostStage::SelectCard,
            ASK_FAYIN_COST,
            build_ask_group_d,
        );
    }
    if m.stage == HostStage::SelectCard {
        clear_ask_id(ctx.core, owner);
        let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
            return Flow::Done;
        };
        discard_he_card(ctx, owner, c);
        m.stage = HostStage::SelectNature;
        return Flow::Yield;
    }
    let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) else {
        return Flow::Done;
    };
    let n = if ctx.core.player(t).boss_mad { 4 } else { 5 };
    match ctx.core.rng.gen_range(n) {
        0 => {
            for _ in 0..2 {
                let he = discardable_hand_and_equips(ctx.core, ctx.catalog, t);
                if he.is_empty() {
                    break;
                }
                let idx = ctx.core.rng.gen_range(he.len() as u32) as usize;
                discard_he_card(ctx, t, he[idx]);
            }
        }
        1 => {
            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                Some(owner),
                t,
                1,
                DamageNature::Fire,
                None,
            )));
        }
        2 => {
            ctx.change_hujia(owner, 1);
        }
        3 => {
            ctx.turn_over(t);
            m.stage = HostStage::DrawSettle;
            return Flow::Yield;
        }
        _ => {
            set_flag(ctx.core, t, "gw_fayin_mad", 1);
            ctx.core.player_mut(t).boss_mad = true;
        }
    }
    Flow::Done
}

fn step_gwbaquan(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(
            m,
            ctx,
            HostStage::TargetChoice,
            ASK_GWBAQUAN_TARGET,
            build_ask_group_d,
        ),
        HostStage::TargetChoice => {
            clear_ask_id(ctx.core, owner);
            let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                refund_turn_limit(ctx, owner, &m.def_key);
                return Flow::Done;
            };
            let hs = ctx.core.player(t).hand.clone();
            if hs.is_empty() {
                return Flow::Done;
            }
            m.target = Some(t);
            m.cards = hs.clone();
            ctx.give_hand_cards(t, owner, &hs);
            m.stage = HostStage::ComparePlayerPick;
            Flow::Yield
        }
        HostStage::ComparePlayerPick => {
            if !ctx.core.player(owner).alive || ctx.core.player(owner).hand.is_empty() {
                return Flow::Done;
            }
            wait_ask(
                m,
                ctx,
                HostStage::SelectCard,
                ASK_GWBAQUAN_RETURN,
                build_ask_group_d,
            )
        }
        HostStage::SelectCard => {
            clear_ask_id(ctx.core, owner);
            let Some(t) = m.target else { return Flow::Done };
            let ret: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Card(c) => Some(c),
                    _ => None,
                })
                .filter(|c| ctx.core.player(owner).hand.contains(c))
                .collect();
            let all_orig = ret.iter().all(|c| m.cards.contains(c));
            m.card = Some(u32::from(all_orig));
            if !ret.is_empty() && ctx.core.player(t).alive {
                ctx.give_hand_cards(owner, t, &ret);
            }
            m.stage = HostStage::CompareTargetPick;
            Flow::Yield
        }
        HostStage::CompareTargetPick => {
            let Some(t) = m.target else { return Flow::Done };
            if m.card == Some(1)
                && ctx.core.player(owner).alive
                && ctx.core.player(t).alive
                && ctx.core.player(owner).hp <= ctx.core.player(t).hp
            {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    t,
                    1,
                    DamageNature::Normal,
                    None,
                )));
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_huihun(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::ComparePlayerPick => {
            if let Some(c) = m.card.filter(|&c| is_red_card(ctx.catalog, c)) {
                let n = flag(ctx.core, owner, "gw_huihun_n");
                let enc = u32::from(c) + 1;
                let dup = (0..n).any(|i| flag(ctx.core, owner, &format!("gw_huihun_c:{i}")) == enc);
                if !dup {
                    set_flag(ctx.core, owner, &format!("gw_huihun_c:{n}"), enc);
                    set_flag(ctx.core, owner, "gw_huihun_n", n + 1);
                }
            }
            Flow::Done
        }
        HostStage::DrawSettle => {
            let n = flag(ctx.core, owner, "gw_huihun_n");
            let mut list = Vec::new();
            for i in 0..n {
                let v = flag(ctx.core, owner, &format!("gw_huihun_c:{i}"));
                if v > 0 {
                    let c = (v - 1) as CardId;
                    if ctx.core.discard.contains(&c) {
                        list.push(c);
                        if list.len() >= 2 {
                            break;
                        }
                    }
                }
            }
            let mut gained = Vec::new();
            for c in list {
                if let Some(pos) = ctx.core.discard.iter().position(|&x| x == c) {
                    ctx.core.discard.remove(pos);
                    crate::kernel::clear_u05_discard_vanishtags(ctx.core, &[c]);
                    ctx.core.player_mut(owner).hand.push(c);
                    gained.push(c);
                }
            }
            if !gained.is_empty() {
                ctx.emit(EventData::CardsDrawn {
                    player: owner,
                    cards: gained,
                });
            }
            Flow::Done
        }
        HostStage::HuanrenPick => {
            let n = flag(ctx.core, owner, "gw_huihun_n");
            for i in 0..n {
                set_flag(ctx.core, owner, &format!("gw_huihun_c:{i}"), 0);
            }
            set_flag(ctx.core, owner, "gw_huihun_n", 0);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_lanquan(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if ctx.core.deck.len() < 6 && !ctx.core.discard.is_empty() {
                let mut washed = std::mem::take(&mut ctx.core.discard);
                ctx.core.rng.shuffle(&mut washed);
                crate::kernel::elo_on_wash_deck(ctx.core, &washed);
                washed.extend(std::mem::take(&mut ctx.core.deck));
                ctx.core.deck = washed;
                ctx.log(LogEvent::DeckReshuffled);
            }
            if lanquan_playable_options(ctx.core, ctx.catalog, owner).is_empty() {
                refund_turn_limit(ctx, owner, &m.def_key);
                return Flow::Done;
            }
            wait_ask(
                m,
                ctx,
                HostStage::SelectCard,
                ASK_LANQUAN_PICK,
                build_ask_group_d,
            )
        }
        HostStage::SelectCard => {
            clear_ask_id(ctx.core, owner);
            let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                refund_turn_limit(ctx, owner, &m.def_key);
                return Flow::Done;
            };
            if !b5b_card_can_use(ctx.core, ctx.catalog, owner, c, true) {
                refund_turn_limit(ctx, owner, &m.def_key);
                return Flow::Done;
            }
            m.card = Some(c);
            m.target = None;
            if b5b_card_targets(ctx.core, ctx.catalog, owner, c).is_some() {
                return wait_ask(
                    m,
                    ctx,
                    HostStage::TargetChoice,
                    ASK_LANQUAN_TARGET,
                    build_ask_group_d,
                );
            }
            if let Some(pos) = ctx.core.deck.iter().position(|&x| x == c) {
                ctx.core.deck.remove(pos);
                emit_huoge_use(ctx, owner, c, Vec::new());
            }
            Flow::Done
        }
        HostStage::TargetChoice => {
            clear_ask_id(ctx.core, owner);
            let Some(c) = m.card.take() else {
                return Flow::Done;
            };
            let targets: Vec<PlayerId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Player(p) => Some(p),
                    _ => None,
                })
                .collect();
            if targets.is_empty() {
                refund_turn_limit(ctx, owner, &m.def_key);
                return Flow::Done;
            }
            if ctx.catalog.card_kind(c) == CardKind::Jiedao {
                let holder = targets[0];
                if b5b_jiedao_victim_options(ctx.core, ctx.catalog, owner, holder).is_empty() {
                    refund_turn_limit(ctx, owner, &m.def_key);
                    return Flow::Done;
                }
                m.card = Some(c);
                m.target = Some(holder);
                return wait_ask(
                    m,
                    ctx,
                    HostStage::CardsAsk,
                    ASK_LANQUAN_TARGET,
                    build_ask_group_d,
                );
            }
            if let Some(pos) = ctx.core.deck.iter().position(|&x| x == c) {
                ctx.core.deck.remove(pos);
                emit_huoge_use(ctx, owner, c, targets);
            }
            Flow::Done
        }
        HostStage::CardsAsk => {
            clear_ask_id(ctx.core, owner);
            let (Some(c), Some(holder)) = (m.card.take(), m.target.take()) else {
                return Flow::Done;
            };
            let Some(ChoiceOption::Player(victim)) = ctx.answer().first().copied() else {
                refund_turn_limit(ctx, owner, &m.def_key);
                return Flow::Done;
            };
            if let Some(pos) = ctx.core.deck.iter().position(|&x| x == c) {
                ctx.core.deck.remove(pos);
                emit_huoge_use(ctx, owner, c, vec![holder, victim]);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}
