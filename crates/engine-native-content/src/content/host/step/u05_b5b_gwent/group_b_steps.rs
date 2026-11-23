//! U05-5b Batch B 技能结算步进（`shuangxi`..`gwfengchi`）。

use super::group_b::collect_fengchi_pool;
use super::group_b_asks::*;
use super::*;
use crate::event::DamageNature;
use crate::state::SkillInstance;

pub(super) fn step_group_b(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.def_key.as_str() {
        "shuangxi" => step_shuangxi(m, ctx),
        "yangfan" => step_yangfan(m, ctx),
        "gwfengshi" => step_gwfengshi(m, ctx),
        "gwchenshui" => step_gwchenshui(m, ctx),
        "gwliedi" => step_gwliedi(m, ctx),
        "julian" => step_julian(m.owner, ctx),
        "gwqinwu" => step_gwqinwu(m, ctx),
        "huanshu" => step_huanshu(m, ctx),
        "huanshu2" | "huanshu3" => step_huanshu2(m, ctx),
        "gwjieyin" => step_gwjieyin(m, ctx),
        "zhengjun" => step_zhengjun(m, ctx),
        "gwxuezhan" => {
            let _ = give_or_draw_card_named(ctx, m.owner, "gw_shizizhaohuan");
            Flow::Done
        }
        "bolang" | "bolang_gain" => step_bolang(m, ctx),
        "gwjushi" => step_gwjushi(m, ctx),
        "gwjushi2" => step_gwjushi2(m, ctx),
        "gwfengchi" => step_gwfengchi(m, ctx),
        _ => Flow::Done,
    }
}

fn step_shuangxi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if let Some(frame) = ctx.res.frame(ctx.current_event_frame_id()) {
        if let EventData::Damage(_) = frame.data {
            set_flag(ctx.core, owner, "gw_shuangxi_turn_damaged", 1);
            return Flow::Done;
        }
    }
    match m.stage {
        HostStage::Act => wait_ask(
            m,
            ctx,
            HostStage::TargetChoice,
            ASK_SHUANGXI_TARGETS,
            build_ask_group_b,
        ),
        HostStage::TargetChoice => {
            clear_ask_id(ctx.core, owner);
            let targets: Vec<PlayerId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Player(p) => Some(p),
                    _ => None,
                })
                .collect();
            if targets.is_empty() {
                return Flow::Done;
            }
            let ready = ctx.core.turn.round + 2;
            set_flag(ctx.core, owner, "gw_shuangxi_ready_round", ready);
            if flag(ctx.core, owner, "gw_shuangxi_turn_damaged") > 0 {
                apply_gw_baishuang_to_targets(ctx, &targets);
            } else {
                let t = targets[0];
                ctx.emit(EventData::CardUsed(crate::event::CardUseData {
                    card: None,
                    player: owner,
                    targets: vec![t],
                    nature: DamageNature::Normal,
                    use_as: Some(CardKind::Ciguhanshuang),
                    cancelled: false,
                }));
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_yangfan(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage == HostStage::DrawSettle {
        if ctx.core.player(owner).alive {
            ctx.start_draw(owner, 1);
        }
        return Flow::Done;
    }
    let Some(uc) = m.card else { return Flow::Done };
    let suit = ctx.catalog.cards[uc as usize].suit;
    let red = is_red_suit(suit);
    let same_suit: Vec<CardId> = ctx
        .core
        .player(owner)
        .hand
        .iter()
        .copied()
        .filter(|&c| {
            c != uc
                && ctx.catalog.cards[c as usize].suit == suit
                && !crate::kernel::is_gw_dudayuanshuai2_locked(ctx.core, ctx.catalog, owner, c)
        })
        .collect();
    let pool = if same_suit.is_empty() {
        ctx.core
            .player(owner)
            .hand
            .iter()
            .copied()
            .filter(|&c| {
                c != uc
                    && is_red_card(ctx.catalog, c) == red
                    && !crate::kernel::is_gw_dudayuanshuai2_locked(
                        ctx.core,
                        ctx.catalog,
                        owner,
                        c,
                    )
            })
            .collect()
    } else {
        same_suit
    };
    if let Some(&picked) =
        (!pool.is_empty()).then(|| &pool[ctx.core.rng.gen_range(pool.len() as u32) as usize])
    {
        ctx.discard_from_hand(owner, picked);
        m.stage = HostStage::DrawSettle;
        return Flow::Yield;
    }
    Flow::Done
}

fn step_gwfengshi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(
            m,
            ctx,
            HostStage::SelectNature,
            ASK_GWFENGSHI_CHOICE,
            build_ask_group_b,
        ),
        HostStage::SelectNature => {
            clear_ask_id(ctx.core, owner);
            let Some(ChoiceOption::Tag(tag)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            let mut enemies = enemies_of(ctx.core, ctx.catalog, owner);
            let mut q = vec![owner];
            if tag == 0 {
                while q.len() < 3 && !enemies.is_empty() {
                    let i = ctx.core.rng.gen_range(enemies.len() as u32) as usize;
                    q.push(enemies.swap_remove(i));
                }
            } else {
                q.push(owner);
                if !enemies.is_empty() {
                    let e = enemies[ctx.core.rng.gen_range(enemies.len() as u32) as usize];
                    q.push(e);
                }
            }
            m.card = Some(u32::from(tag));
            m.targets = q;
            m.stage = HostStage::CompareTargetPick;
            advance_fengshi_queue(m, ctx)
        }
        HostStage::CompareTargetPick => advance_fengshi_queue(m, ctx),
        _ => Flow::Done,
    }
}

fn advance_fengshi_queue(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let is_buff = m.card == Some(1);
    while !m.targets.is_empty() {
        let t = m.targets.remove(0);
        if !ctx.core.player(t).alive {
            continue;
        }
        if is_buff {
            apply_buff(ctx, t);
        } else {
            apply_debuff(ctx, t);
        }
        return Flow::Yield;
    }
    Flow::Done
}

fn step_gwchenshui(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(0) {
        let cur_fid = ctx.current_event_frame_id();
        let mut give_pair = None;
        if let Some(frame) = ctx.res.frame_mut(cur_fid) {
            if let EventData::Damage(d) = &mut frame.data {
                d.prevented = true;
                if let Some(src) = d.source {
                    if src != d.target && ctx.core.player(src).alive && ctx.core.player(d.target).alive
                    {
                        give_pair = Some((d.target, src));
                    }
                }
            }
        }
        let cnt = flag(ctx.core, owner, "gwchenshui_count") + 1;
        set_flag(ctx.core, owner, "gwchenshui_count", cnt);
        sync_mark(ctx.core, ctx.catalog, owner, "gwchenshui", cnt as i32);
        if let Some((from, to)) = give_pair {
            let he = hand_and_equips(ctx.core, from);
            if !he.is_empty() {
                let c = he[ctx.core.rng.gen_range(he.len() as u32) as usize];
                if ctx.core.player(from).hand.contains(&c) {
                    ctx.give_hand_cards(from, to, &[c]);
                } else {
                    ctx.core.player_mut(from).equips.retain(|&x| x != c);
                    ctx.core.declared_slots.remove(&c);
                    crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, from, c);
                    ctx.note_cards_lost(from, &[c]);
                    ctx.core.player_mut(to).hand.push(c);
                    ctx.emit(EventData::CardsMoved {
                        cards: vec![c],
                        from,
                        to,
                    });
                }
            }
        }
        return Flow::Done;
    }
    if m.stage == HostStage::Act {
        if let Some(old_def) = ctx.catalog.skill_by_name("gwchenshui") {
            ctx.remove_skill(owner, old_def);
        }
        set_flag(ctx.core, owner, "gwchenshui_count", 0);
        sync_mark(ctx.core, ctx.catalog, owner, "gwchenshui", 0);
        m.targets = enemies_of(ctx.core, ctx.catalog, owner);
        m.stage = HostStage::SelectNature;
    }
    while !m.targets.is_empty() {
        let e = m.targets.remove(0);
        if ctx.core.player(e).alive {
            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                Some(owner),
                e,
                1,
                DamageNature::Normal,
                None,
            )));
            return Flow::Yield;
        }
    }
    set_flag(ctx.core, owner, "gw_liedi_sleep_state", 0);
    if let Some(new_def) = ctx.catalog.skill_by_name("gwliedi") {
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
    Flow::Done
}

fn step_gwliedi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.card {
        Some(0) => {
            let cur_fid = ctx.current_event_frame_id();
            let dist = ctx.res.frame(cur_fid).and_then(|f| {
                if let EventData::Damage(d) = &f.data {
                    Some(ctx.core.computed_distance(ctx.catalog, owner, d.target))
                } else {
                    None
                }
            });
            if let Some(dist) = dist.filter(|&d| d >= 2) {
                if let Some(frame) = ctx.res.frame_mut(cur_fid) {
                    if let EventData::Damage(d) = &mut frame.data {
                        d.amount += (dist / 2) as i32;
                    }
                }
            }
        }
        Some(1) => {
            set_flag(ctx.core, owner, "gw_liedi_sleep_state", 2);
        }
        _ => {
            let cur = flag(ctx.core, owner, "gw_liedi_sleep_state");
            match cur {
                2 => set_flag(ctx.core, owner, "gw_liedi_sleep_state", 0),
                0 => set_flag(ctx.core, owner, "gw_liedi_sleep_state", 1),
                _ => {
                    set_flag(ctx.core, owner, "gw_liedi_sleep_state", 0);
                    if let Some(old_def) = ctx.catalog.skill_by_name("gwliedi") {
                        ctx.remove_skill(owner, old_def);
                    }
                    if let Some(new_def) = ctx.catalog.skill_by_name("gwchenshui") {
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
                }
            }
        }
    }
    Flow::Done
}

fn step_julian(owner: PlayerId, ctx: &mut Ctx<'_>) -> Flow {
    let my_h = ctx.core.player(owner).hand.len();
    let mx = ctx
        .core
        .alive_players()
        .filter(|&p| p != owner)
        .map(|p| ctx.core.player(p).hand.len())
        .max()
        .unwrap_or(my_h);
    if mx > my_h {
        ctx.start_draw(owner, (mx - my_h) as u32);
    }
    Flow::Done
}

fn step_gwqinwu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage == HostStage::Act {
        return wait_ask(
            m,
            ctx,
            HostStage::TargetChoice,
            ASK_GWQINWU_TARGET,
            build_ask_group_b,
        );
    }
    clear_ask_id(ctx.core, owner);
    let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
        return Flow::Done;
    };
    if !ctx.core.player(t).alive {
        return Flow::Done;
    }
    let stamp = ctx.core.turn.round.saturating_mul(16) + u32::from(ctx.core.turn.current) + 1;
    set_flag(ctx.core, owner, "gw_qinwu_used_turn", stamp);
    ctx.start_draw(t, 1);
    if !has_named_skill(ctx.core, ctx.catalog, t, "gwqinwu") {
        for sk in ["gwqinwu", "gwqinwu2"] {
            if let Some(def) = ctx.catalog.skill_by_name(sk) {
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
        set_flag(ctx.core, t, "gw_qinwu_temp", 1);
    }
    Flow::Done
}

fn step_huanshu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage == HostStage::Act {
        return wait_ask(
            m,
            ctx,
            HostStage::SelectCard,
            ASK_HUANSHU_CARD,
            build_ask_group_b,
        );
    }
    clear_ask_id(ctx.core, owner);
    let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
        return Flow::Done;
    };
    if !ctx.core.player(owner).hand.contains(&c) {
        return Flow::Done;
    }
    ctx.take_hand_card(owner, c);
    ctx.note_cards_lost(owner, &[c]);
    off_court_put(ctx.core, owner, c);
    set_flag(ctx.core, owner, "gw_huanshu2_card", u32::from(c) + 1);
    ctx.emit(EventData::CardsLost {
        player: owner,
        cards: vec![c],
        origins: vec![crate::event::LostOrigin::Hand],
    });
    if let Some(def) = ctx.catalog.skill_by_name("huanshu2") {
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
    Flow::Done
}

fn step_huanshu2(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let stored = flag(ctx.core, owner, "gw_huanshu2_card");
    if stored == 0 {
        return Flow::Done;
    }
    let sc = stored - 1;
    set_flag(ctx.core, owner, "gw_huanshu2_card", 0);
    if off_court_take(ctx.core, owner, sc) {
        ctx.to_discard(sc);
    }
    if let Some(def) = ctx.catalog.skill_by_name("huanshu2") {
        ctx.core.skills.remove(&(owner, def));
    }
    if m.card == Some(1) {
        let cur_fid = ctx.current_event_frame_id();
        if let Some(frame) = ctx.res.frame_mut(cur_fid) {
            if let EventData::CardUsed(u) = &mut frame.data {
                u.cancelled = true;
            }
        }
        if ctx.core.player(owner).alive {
            ctx.start_draw(owner, 2);
        }
    }
    Flow::Done
}

fn step_gwjieyin(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(99) {
        let prev_round = flag(ctx.core, owner, "gw_jieyin_round");
        let cur = ctx.core.turn.round;
        if prev_round == 0 {
            set_flag(ctx.core, owner, "gw_jieyin_round", cur.max(1));
        } else if cur > prev_round && (cur - prev_round) % 2 == 0 {
            set_flag(ctx.core, owner, "gw_jieyin_mask", 0);
            set_flag(ctx.core, owner, "gw_jieyin_round", cur);
        }
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => wait_ask(
            m,
            ctx,
            HostStage::SelectNature,
            ASK_GWJIEYIN_PICK,
            build_ask_group_b,
        ),
        HostStage::SelectNature => {
            clear_ask_id(ctx.core, owner);
            let Some(ChoiceOption::Tag(tag)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            if tag > 2 {
                return Flow::Done;
            }
            m.card = Some(u32::from(tag));
            if tag == 0 {
                let mask = flag(ctx.core, owner, "gw_jieyin_mask") | 1;
                set_flag(ctx.core, owner, "gw_jieyin_mask", mask);
                ctx.emit(EventData::CardUsed(crate::event::CardUseData {
                    card: None,
                    player: owner,
                    targets: Vec::new(),
                    nature: DamageNature::Normal,
                    use_as: Some(CardKind::Wenyi),
                    cancelled: false,
                }));
                return Flow::Done;
            }
            wait_ask(
                m,
                ctx,
                HostStage::TargetChoice,
                ASK_GWJIEYIN_TARGET,
                build_ask_group_b,
            )
        }
        HostStage::TargetChoice => {
            clear_ask_id(ctx.core, owner);
            let (Some(tag), Some(ChoiceOption::Player(t))) =
                (m.card, ctx.answer().first().copied())
            else {
                return Flow::Done;
            };
            let mask = flag(ctx.core, owner, "gw_jieyin_mask") | (1u32 << tag);
            set_flag(ctx.core, owner, "gw_jieyin_mask", mask);
            let kind = if tag == 1 {
                CardKind::Yanziyaoshui
            } else {
                CardKind::Kunenfayin
            };
            ctx.emit(EventData::CardUsed(crate::event::CardUseData {
                card: None,
                player: owner,
                targets: vec![t],
                nature: DamageNature::Normal,
                use_as: Some(kind),
                cancelled: false,
            }));
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_zhengjun(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage == HostStage::ComparePlayerPick {
        if let Some(c) = m.card {
            let cname = clean_gwent_name(ctx.catalog.cards[c as usize].name).to_string();
            let k1 = format!("gw_zj_1:{cname}");
            let k2 = format!("gw_zj_2:{cname}");
            if flag(ctx.core, owner, &k1) > 0 {
                if flag(ctx.core, owner, &k2) == 0 {
                    set_flag(ctx.core, owner, &k2, 1);
                    let cnt = flag(ctx.core, owner, "gw_zj_cnt") + 1;
                    set_flag(ctx.core, owner, "gw_zj_cnt", cnt);
                    sync_mark(ctx.core, ctx.catalog, owner, "zhengjun", cnt as i32);
                    ctx.core.player_mut(owner).max_hp += 1;
                    ctx.recover(owner, 1, None);
                }
            } else {
                set_flag(ctx.core, owner, &k1, 1);
            }
        }
        return Flow::Done;
    }
    if m.stage == HostStage::DrawSettle {
        let cnt = flag(ctx.core, owner, "gw_zj_cnt") as usize;
        if cnt == 1 {
            ctx.start_draw(owner, 1);
            return Flow::Done;
        }
        if cnt >= 2 {
            let peek = take_top_deck_cards(ctx.core, cnt);
            if peek.is_empty() {
                return Flow::Done;
            }
            for &c in &peek {
                off_court_put(ctx.core, owner, c);
            }
            m.cards = peek;
            return wait_ask(
                m,
                ctx,
                HostStage::SelectCard,
                ASK_ZHENGJUN_PICK,
                build_ask_group_b,
            );
        }
        return Flow::Done;
    }
    if m.stage == HostStage::SelectCard {
        clear_ask_id(ctx.core, owner);
        if let Some(ChoiceOption::Card(picked)) = ctx.answer().first().copied() {
            if off_court_take(ctx.core, owner, picked) {
                ctx.core.player_mut(owner).hand.push(picked);
                ctx.emit(EventData::CardsDrawn {
                    player: owner,
                    cards: vec![picked],
                });
            }
            m.cards.retain(|&c| c != picked);
        }
        return_off_court_to_deck_top(ctx.core, owner, &std::mem::take(&mut m.cards));
    }
    Flow::Done
}

fn step_bolang(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(1) || m.def_key == "bolang_gain" {
        if m.stage == HostStage::Act {
            return wait_ask(
                m,
                ctx,
                HostStage::CardsAsk,
                ASK_BOLANG_GAIN,
                build_ask_group_b,
            );
        }
        clear_ask_id(ctx.core, owner);
        if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
            if let Some(pos) = ctx.core.discard.iter().position(|&x| x == c) {
                ctx.core.discard.remove(pos);
                crate::kernel::clear_u05_discard_vanishtags(ctx.core, &[c]);
                let stamp =
                    ctx.core.turn.round.saturating_mul(16) + u32::from(ctx.core.turn.current) + 1;
                set_flag(ctx.core, owner, "gw_bolang_gain_turn", stamp);
                ctx.core.player_mut(owner).hand.push(c);
                ctx.emit(EventData::CardsDrawn {
                    player: owner,
                    cards: vec![c],
                });
            }
        }
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => {
            let peek = take_top_deck_cards(ctx.core, 6);
            if peek.is_empty() {
                return Flow::Done;
            }
            for &c in &peek {
                off_court_put(ctx.core, owner, c);
            }
            m.cards = peek;
            wait_ask(
                m,
                ctx,
                HostStage::SelectCard,
                ASK_BOLANG_PICK,
                build_ask_group_b,
            )
        }
        HostStage::SelectCard => {
            clear_ask_id(ctx.core, owner);
            let picked: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Card(c) => Some(c),
                    _ => None,
                })
                .filter(|c| m.cards.contains(c))
                .take(3)
                .collect();
            for &c in &picked {
                if off_court_take(ctx.core, owner, c) {
                    ctx.to_discard(c);
                    set_flag(ctx.core, owner, &format!("gw_bolang_tag:{c}"), 1);
                }
            }
            m.cards.retain(|c| !picked.contains(c));
            return_off_court_to_deck_top(ctx.core, owner, &std::mem::take(&mut m.cards));
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_gwjushi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage == HostStage::Act {
        return wait_ask(
            m,
            ctx,
            HostStage::TargetChoice,
            ASK_GWJUSHI_TARGET,
            build_ask_group_b,
        );
    }
    clear_ask_id(ctx.core, owner);
    let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
        super::group_a::refund_turn_limit(ctx, owner, &m.def_key);
        return Flow::Done;
    };
    let he = hand_and_equips(ctx.core, t);
    if he.is_empty() {
        return Flow::Done;
    }
    let c = he[ctx.core.rng.gen_range(he.len() as u32) as usize];
    let was_equip = ctx.core.player(t).equips.contains(&c);
    if was_equip {
        ctx.core.player_mut(t).equips.retain(|&x| x != c);
        ctx.core.declared_slots.remove(&c);
        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, t, c);
    } else {
        ctx.take_hand_card(t, c);
    }
    ctx.note_cards_lost(t, &[c]);
    off_court_put(ctx.core, owner, c);
    ctx.emit(EventData::CardsLost {
        player: t,
        cards: vec![c],
        origins: vec![if was_equip {
            crate::event::LostOrigin::Equip
        } else {
            crate::event::LostOrigin::Hand
        }],
    });
    set_flag(ctx.core, owner, "gw_jushi2_card", u32::from(c) + 1);
    set_flag(ctx.core, owner, "gw_jushi2_from", u32::from(t) + 1);
    set_flag(
        ctx.core,
        owner,
        "gw_jushi2_equip",
        if was_equip { 1 } else { 0 },
    );
    if let Some(def) = ctx.catalog.skill_by_name("gwjushi2") {
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
    Flow::Done
}

fn step_gwjushi2(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let stored = flag(ctx.core, owner, "gw_jushi2_card");
    let from = flag(ctx.core, owner, "gw_jushi2_from").checked_sub(1).map(|x| x as PlayerId);
    let was_equip = flag(ctx.core, owner, "gw_jushi2_equip") > 0;
    set_flag(ctx.core, owner, "gw_jushi2_card", 0);
    set_flag(ctx.core, owner, "gw_jushi2_from", 0);
    set_flag(ctx.core, owner, "gw_jushi2_equip", 0);
    if let Some(def) = ctx.catalog.skill_by_name("gwjushi2") {
        ctx.core.skills.remove(&(owner, def));
    }
    let Some(c) = stored.checked_sub(1) else {
        return Flow::Done;
    };
    if !off_court_take(ctx.core, owner, c) {
        return Flow::Done;
    }
    if m.card == Some(1) {
        if let Some(t) = from.filter(|&p| ctx.core.player(p).alive) {
            if was_equip && ctx.catalog.card_kind(c).is_equip() {
                if let Some(slot) = ctx.catalog.card_slot_of(ctx.core, c) {
                    let old = ctx
                        .core
                        .player(t)
                        .equips
                        .iter()
                        .copied()
                        .find(|&eq| ctx.catalog.card_slot_of(ctx.core, eq) == Some(slot));
                    if let Some(old_eq) = old {
                        ctx.core.player_mut(t).equips.retain(|&x| x != old_eq);
                        ctx.core.declared_slots.remove(&old_eq);
                        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, t, old_eq);
                        ctx.note_cards_lost(t, &[old_eq]);
                        ctx.to_discard(old_eq);
                        ctx.emit(EventData::CardsLost {
                            player: t,
                            cards: vec![old_eq],
                            origins: vec![crate::event::LostOrigin::Equip],
                        });
                    }
                }
                ctx.core.player_mut(t).equips.push(c);
                crate::kernel::grant_equip_skills(ctx, t, c);
            } else {
                ctx.core.player_mut(t).hand.push(c);
                ctx.emit(EventData::CardsDrawn {
                    player: t,
                    cards: vec![c],
                });
            }
        } else {
            ctx.to_discard(c);
        }
    } else if ctx.core.player(owner).alive {
        ctx.core.player_mut(owner).hand.push(c);
        ctx.emit(EventData::CardsDrawn {
            player: owner,
            cards: vec![c],
        });
    } else {
        ctx.to_discard(c);
    }
    Flow::Done
}

fn step_gwfengchi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage == HostStage::Act {
        let mut pool = collect_fengchi_pool(ctx.catalog, ctx.core, owner);
        if pool.is_empty() {
            return Flow::Done;
        }
        let mut n = 0u32;
        while n < 3 && !pool.is_empty() {
            let idx = ctx.core.rng.gen_range(pool.len() as u32) as usize;
            let def = pool.swap_remove(idx);
            set_flag(
                ctx.core,
                owner,
                &format!("gw_fengchi_opt:{n}"),
                u32::from(def) + 1,
            );
            n += 1;
        }
        set_flag(ctx.core, owner, "gw_fengchi_n", n);
        return wait_ask(
            m,
            ctx,
            HostStage::SelectCard,
            ASK_GWFENGCHI_PICK,
            build_ask_group_b,
        );
    }
    clear_ask_id(ctx.core, owner);
    if let Some(ChoiceOption::Skill(def)) = ctx.answer().first().copied() {
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
        let cnt = flag(ctx.core, owner, "l27_temp_phase_use_count");
        set_flag(
            ctx.core,
            owner,
            &format!("l27_temp_phase_use_def:{cnt}"),
            u32::from(def) + 1,
        );
        set_flag(ctx.core, owner, "l27_temp_phase_use_count", cnt + 1);
    }
    Flow::Done
}
