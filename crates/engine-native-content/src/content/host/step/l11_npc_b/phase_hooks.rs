//! Engine lifecycle hooks (`roundStart`, `turnBegin`, `drawBegin`, `cardUseAfter`,
//! `phaseUseEnd`, `phaseEnd`, `turnPhaseAfter`, `playerDied`, `damageEnd`, `hpChanged`,
//! `equipRevoked`, `outerSettled`, `hostEvent`) for `L11` Part B (`l11_npc_b`).

use super::super::l11_common::{
    alive_all, alive_others, discard_player_cards, emit_lose_hp, has_named_skill, move_cards_to_hand,
};
use super::helpers::{
    card_type_group, damage_root_card_use_fid, distinct_alive_factions, flag, grant_npc_b_skill,
    is_locked_unblocked_npc_b, is_unblocked_npc_b, set_flag, spawn_npc_b_park, suit_bit,
    turn_stamp,
};
use crate::catalog::{CardKind, Catalog};
use crate::event::{DamageData, EventData, Phase};
use crate::ids::{CardId, EventId, PlayerId};
use crate::machine::Ctx;
use crate::state::GameStateCore;

pub(super) fn find_field_card_owner(core: &GameStateCore, card: CardId) -> Option<PlayerId> {
    for (pid, p) in core.players.iter().enumerate() {
        if p.equips.contains(&card) || p.judge.contains(&card) {
            return Some(pid as PlayerId);
        }
    }
    None
}

#[inline]
fn catalog_has_l11_npc_b(catalog: &Catalog) -> bool {
    catalog.skill_by_name("twwushen").is_some_and(|sid| {
        catalog.skills[sid as usize].behavior.as_host_behavior()
            == Some(crate::content::host::HostBehavior::L11CrossNpcB)
    })
}

pub(super) fn on_round_start_l11_npc_b(ctx: &mut Ctx<'_>) {
    if !catalog_has_l11_npc_b(ctx.catalog) {
        return;
    }
    let cur_round = ctx.core.turn.round;
    if flag(ctx.core, 0, "l11b_round_handled") == cur_round {
        return;
    }
    set_flag(ctx.core, 0, "l11b_round_handled", cur_round);

    // Rotate per-round damage-source matrix for `twxiongzheng`.
    let seats = ctx.core.players.len() as PlayerId;
    for tgt in 0..seats {
        let cur_k = format!("l11b_xz_cur_dmg_to:{tgt}");
        let prev_k = format!("l11b_xz_prev_dmg_to:{tgt}");
        let mask = flag(ctx.core, 0, &cur_k);
        set_flag(ctx.core, 0, &prev_k, mask);
        set_flag(ctx.core, 0, &cur_k, 0);
    }

    let order = ctx.core.seat_order_from(ctx.core.turn.current);
    for p in order {
        if !ctx.core.player(p).alive || !is_unblocked_npc_b(ctx.core, ctx.catalog, p) {
            continue;
        }
        if has_named_skill(ctx.core, ctx.catalog, p, "twlingfa")
            && flag(ctx.core, p, "l11b_twlingfa_round") != cur_round
        {
            set_flag(ctx.core, p, "l11b_twlingfa_round", cur_round);
            spawn_npc_b_park(
                ctx,
                "twlingfa",
                p,
                None,
                Some(cur_round),
                Vec::new(),
                Vec::new(),
            );
        }
        if has_named_skill(ctx.core, ctx.catalog, p, "twxiongzheng")
            && flag(ctx.core, p, "l11b_twxiongzheng_round") != cur_round
        {
            set_flag(ctx.core, p, "l11b_twxiongzheng_round", cur_round);
            spawn_npc_b_park(ctx, "twxiongzheng", p, None, None, Vec::new(), Vec::new());
        }
        if cur_round >= 2
            && has_named_skill(ctx.core, ctx.catalog, p, "zongkui")
            && flag(ctx.core, p, "l11b_zongkui_round") != cur_round
        {
            let cands: Vec<PlayerId> = alive_others(ctx.core, p)
                .into_iter()
                .filter(|&q| flag(ctx.core, q, "l11_kui_mark") == 0)
                .collect();
            if !cands.is_empty() {
                set_flag(ctx.core, p, "l11b_zongkui_round", cur_round);
                spawn_npc_b_park(ctx, "zongkui", p, None, Some(1), Vec::new(), Vec::new());
            }
        }
    }
}

pub(super) fn on_turn_begin_l11_npc_b(ctx: &mut Ctx<'_>, _player: PlayerId) {
    if !catalog_has_l11_npc_b(ctx.catalog) {
        return;
    }
    if flag(ctx.core, 0, "l11b_round_handled") != ctx.core.turn.round {
        on_round_start_l11_npc_b(ctx);
    }
}

pub(super) fn on_draw_begin_step_l11_npc_b(ctx: &mut Ctx<'_>, player: PlayerId, pending: &mut i32) {
    if !catalog_has_l11_npc_b(ctx.catalog) {
        return;
    }
    if has_named_skill(ctx.core, ctx.catalog, player, "sbyingzi")
        && is_locked_unblocked_npc_b(ctx.core, ctx.catalog, player)
    {
        let p = ctx.core.player(player);
        let num = u32::from(p.hand.len() >= 2)
            + u32::from(p.hp >= 2)
            + u32::from(!p.equips.is_empty());
        if num > 0 {
            super::helpers::log_skill(ctx, player, "sbyingzi");
            *pending += num as i32;
            set_flag(ctx.core, player, "l11b_sbyingzi_turn_stamp", turn_stamp(ctx.core));
            set_flag(ctx.core, player, "l11b_sbyingzi_hand_bonus", num);
        }
    }
    if has_named_skill(ctx.core, ctx.catalog, player, "yongsi")
        && is_locked_unblocked_npc_b(ctx.core, ctx.catalog, player)
    {
        let n = distinct_alive_factions(ctx.core, ctx.catalog) as i32;
        if n > 0 {
            super::helpers::log_skill(ctx, player, "yongsi");
            *pending += n;
        }
    }
    let shunshi = flag(ctx.core, player, "l11b_shunshi_stacks") as i32;
    if shunshi > 0 {
        *pending += shunshi;
    }
}

pub(super) fn on_card_use_after_l11_npc_b(
    ctx: &mut Ctx<'_>,
    fid: EventId,
    u: &crate::event::CardUseData,
) {
    if !catalog_has_l11_npc_b(ctx.catalog) {
        return;
    }
    let tstamp = turn_stamp(ctx.core);
    let user = u.player;
    set_flag(ctx.core, user, "l11b_turn_used_card", tstamp);
    let kind = u.effective_kind(ctx.catalog);

    // `twhengjiang2`: draw 1 card per other player who responded to this `twhengjiang`-buffed card.
    if ctx
        .core
        .vm_aux
        .flags
        .remove(&(user, format!("l11b_twhengjiang_buffed:{fid}")))
        .is_some()
    {
        let resp_mask = ctx
            .core
            .vm_aux
            .flags
            .remove(&(user, format!("l11b_twhengjiang_resp_mask:{fid}")))
            .unwrap_or(0)
            & !(1u32 << user);
        let n = resp_mask.count_ones();
        if n > 0 && ctx.core.player(user).alive {
            super::helpers::log_skill(ctx, user, "twhengjiang");
            ctx.start_draw(user, n);
        }
    }

    // Clear `sbliegong` block mask tied to this Sha fid and update suit count / clear.
    for pid in 0..(ctx.core.players.len() as PlayerId) {
        ctx.core
            .vm_aux
            .flags
            .remove(&(pid, format!("l11b_sbliegong_block_mask:{fid}")));
        ctx.core
            .vm_aux
            .flags
            .remove(&(pid, format!("l11b_twxiangyu_extra_shan:{fid}")));
    }
    ctx.core
        .vm_aux
        .flags
        .remove(&(user, "l11b_sbliegong_cur_block_mask".to_string()));
    ctx.core
        .vm_aux
        .flags
        .remove(&(user, format!("l11b_sbliegong_dmg:{fid}")));
    ctx.core
        .vm_aux
        .flags
        .remove(&(user, format!("l11b_kannan_sha:{fid}")));
    if kind.is_sha_like() && !u.cancelled {
        set_flag(ctx.core, user, "l11b_kannan_eff", 0);
        set_flag(ctx.core, user, "l11b_sha_direct_hit_cur", 0);
    }

    if flag(ctx.core, user, "l11b_sbliegong_clear_fid") == fid {
        set_flag(ctx.core, user, "l11b_sbliegong_clear_fid", 0);
        set_flag(ctx.core, user, "l11b_sbliegong_mask", 0);
    } else if let Some(c) = u.card {
        if (c as usize) < ctx.catalog.cards.len() {
            let bit = suit_bit(ctx.catalog.effective_card_suit(ctx.core, user, c));
            if has_named_skill(ctx.core, ctx.catalog, user, "sbliegong")
                && is_unblocked_npc_b(ctx.core, ctx.catalog, user)
            {
                let cur = flag(ctx.core, user, "l11b_sbliegong_mask");
                set_flag(ctx.core, user, "l11b_sbliegong_mask", cur | bit);
            }
            for &t in &u.targets {
                if t != user
                    && ctx.core.player(t).alive
                    && has_named_skill(ctx.core, ctx.catalog, t, "sbliegong")
                    && is_unblocked_npc_b(ctx.core, ctx.catalog, t)
                {
                    let cur = flag(ctx.core, t, "l11b_sbliegong_mask");
                    set_flag(ctx.core, t, "l11b_sbliegong_mask", cur | bit);
                }
            }
        }
    }

    // `twxuhe`: clear armed +2 after the next used card finishes.
    if flag(ctx.core, user, "l11b_twxuhe_turn") == tstamp
        && flag(ctx.core, user, "l11b_twxuhe_sha_fid") != fid
    {
        set_flag(ctx.core, user, "l11b_twxuhe_turn", 0);
        set_flag(ctx.core, user, "l11b_twxuhe_target", 0);
        set_flag(ctx.core, user, "l11b_twxuhe_sha_fid", 0);
    }

    // `xinfu_fuyin`: mark first `Sha`/`Duel` targeting each seat this turn.
    if !u.cancelled && (kind.is_sha_like() || kind == CardKind::Duel) {
        for &t in &u.targets {
            if flag(ctx.core, t, "l11b_fuyin_turn") != tstamp {
                set_flag(ctx.core, t, "l11b_fuyin_turn", tstamp);
                set_flag(ctx.core, t, "l11b_fuyin_fid", fid);
            }
        }
    }

    // `twmibei`: track unique card names across basic/trick/equip; complete on 2 each.
    if ctx.core.player(user).alive
        && has_named_skill(ctx.core, ctx.catalog, user, "twmibei")
        && flag(ctx.core, user, "l11b_twmibei_done") == 0
        && is_unblocked_npc_b(ctx.core, ctx.catalog, user)
    {
        let raw_name = u
            .card
            .and_then(|c| ctx.catalog.cards.get(c as usize).map(|cd| cd.name))
            .filter(|_| u.use_as.is_none());
        let seen_key = match raw_name {
            Some(nm) => format!("l11b_mibei_seen_n:{nm}"),
            None => format!("l11b_mibei_seen_k:{}", kind as u32),
        };
        if flag(ctx.core, user, &seen_key) == 0 {
            set_flag(ctx.core, user, &seen_key, 1);
            let grp = card_type_group(kind);
            let cnt_key = format!("l11b_mibei_cnt_{grp}");
            let nc = flag(ctx.core, user, &cnt_key) + 1;
            set_flag(ctx.core, user, &cnt_key, nc);
        }
        if flag(ctx.core, user, "l11b_mibei_cnt_0") >= 2
            && flag(ctx.core, user, "l11b_mibei_cnt_1") >= 2
            && flag(ctx.core, user, "l11b_mibei_cnt_2") >= 2
        {
            set_flag(ctx.core, user, "l11b_twmibei_done", 1);
            super::helpers::log_skill(ctx, user, "twmibei");
            grant_npc_b_skill(ctx, user, "twmouli");
        }
    }

    // `twyingjia`: record trick usage per turn (two distinct/duplicate? Look: `twyingjia` checks 2 tricks of the SAME name in turn!).
    if !kind.is_basic_kind() && kind.equip_slot().is_none() {
        let seen_k = format!("l11b_yj_trick:{}:{}", tstamp, kind as u32);
        if flag(ctx.core, user, &seen_k) > 0 {
            set_flag(ctx.core, user, "l11b_yingjia_ready_turn", tstamp);
        } else {
            set_flag(ctx.core, user, &seen_k, 1);
        }
    }

    // `twzhengrong`: every 2nd card targeting any other in own PlayPhase, or first damage in PlayPhase.
    if ctx.core.turn.current == user
        && ctx.core.turn.phase == Phase::Play
        && ctx.core.player(user).alive
        && u.targets.iter().any(|&t| t != user)
        && has_named_skill(ctx.core, ctx.catalog, user, "twzhengrong")
        && is_unblocked_npc_b(ctx.core, ctx.catalog, user)
        && alive_others(ctx.core, user)
            .into_iter()
            .any(|p| !super::super::l11_common::he_cards(ctx.core, p).is_empty())
    {
        let pstamp = super::helpers::phase_stamp(ctx.core);
        let cnt = if flag(ctx.core, user, "l11b_zr_card_stamp") == pstamp {
            flag(ctx.core, user, "l11b_zr_card_cnt") + 1
        } else {
            1
        };
        set_flag(ctx.core, user, "l11b_zr_card_stamp", pstamp);
        set_flag(ctx.core, user, "l11b_zr_card_cnt", cnt);
        if cnt % 2 == 0 {
            spawn_npc_b_park(ctx, "twzhengrong", user, None, None, Vec::new(), Vec::new());
        }
    }

    // `xinfu_xingluan`: usable 1 per phaseUse, when targeting exact 1 player -> gain random rank-6 card from deck.
    if !u.cancelled
        && ctx.core.turn.current == user
        && ctx.core.turn.phase == Phase::Play
        && u.targets.len() == 1
        && ctx.core.player(user).alive
        && has_named_skill(ctx.core, ctx.catalog, user, "xinfu_xingluan")
        && is_unblocked_npc_b(ctx.core, ctx.catalog, user)
    {
        let pstamp = super::helpers::phase_stamp(ctx.core);
        if flag(ctx.core, user, "l11b_xingluan_stamp") != pstamp {
            let sixes: Vec<CardId> = ctx
                .core
                .deck
                .iter()
                .copied()
                .filter(|&c| ctx.catalog.cards.get(c as usize).is_some_and(|cd| cd.rank == 6))
                .collect();
            if !sixes.is_empty() {
                set_flag(ctx.core, user, "l11b_xingluan_stamp", pstamp);
                super::helpers::log_skill(ctx, user, "xinfu_xingluan");
                let pick = sixes[ctx.core.rng.gen_range(sixes.len() as u32) as usize];
                let _ = super::helpers::give_deck_or_discard_card_to(ctx, user, pick);
            }
        }
    }

    // `weifeng`: first damaging basic/ordinary trick in phaseUse -> mark 1 other target with `l11b_weifeng_kind`.
    let is_hg = u
        .card
        .is_some_and(|c| super::helpers::is_huogong_card(ctx.catalog, c))
        || flag(ctx.core, user, "l11_virtual_huogong") > 0;
    let is_dmg_card = kind.is_sha_like()
        || is_hg
        || matches!(
            kind,
            CardKind::Duel | CardKind::SavageAttack | CardKind::ArcheryAttack
        );
    if !u.cancelled
        && ctx.core.turn.current == user
        && ctx.core.turn.phase == Phase::Play
        && is_dmg_card
        && ctx.core.player(user).alive
        && has_named_skill(ctx.core, ctx.catalog, user, "weifeng")
        && is_unblocked_npc_b(ctx.core, ctx.catalog, user)
    {
        let pstamp = super::helpers::phase_stamp(ctx.core);
        if flag(ctx.core, user, "l11b_weifeng_used_stamp") != pstamp {
            set_flag(ctx.core, user, "l11b_weifeng_used_stamp", pstamp);
            let eligible: Vec<PlayerId> = u
                .targets
                .iter()
                .copied()
                .filter(|&t| {
                    t != user
                        && ctx.core.player(t).alive
                        && flag(ctx.core, t, "l11b_weifeng_kind") == 0
                })
                .collect();
            if !eligible.is_empty() {
                spawn_npc_b_park(
                    ctx,
                    "weifeng",
                    user,
                    None,
                    Some((kind as u32) + 1),
                    Vec::new(),
                    eligible,
                );
            }
        }
    }

    // `spxiangzhen`: after Nanman settles, if any player took damage from THIS Nanman, draw 1 (and user draws 1).
    if kind == CardKind::SavageAttack && flag(ctx.core, 0, &format!("l11b_nanman_dmg:{fid}")) > 0
    {
        set_flag(ctx.core, 0, &format!("l11b_nanman_dmg:{fid}"), 0);
        let order = ctx.core.seat_order_from(ctx.core.turn.current);
        for p in order {
            if ctx.core.player(p).alive
                && has_named_skill(ctx.core, ctx.catalog, p, "spxiangzhen")
                && is_locked_unblocked_npc_b(ctx.core, ctx.catalog, p)
            {
                super::helpers::log_skill(ctx, p, "spxiangzhen");
                ctx.start_draw(p, 1);
                if user != p && ctx.core.player(user).alive {
                    ctx.start_draw(user, 1);
                }
            }
        }
    }

    // `twlingfa`: Round 1 Sha / Round 2 Tao triggers; `twzhian`: delay/equip card useAfter.
    let round = ctx.core.turn.round;
    let order = ctx.core.seat_order_from(ctx.core.turn.current);
    for p in order {
        if !ctx.core.player(p).alive || !is_unblocked_npc_b(ctx.core, ctx.catalog, p) {
            continue;
        }
        if p != user && !u.cancelled {
            if kind.is_sha_like()
                && flag(ctx.core, p, "l11b_twlingfa_sha_round") == round
                && ctx.core.player(user).alive
                && !super::super::l11_common::he_cards(ctx.core, user).is_empty()
            {
                spawn_npc_b_park(
                    ctx,
                    "twlingfa_sha",
                    p,
                    Some(user),
                    None,
                    Vec::new(),
                    Vec::new(),
                );
            } else if kind == CardKind::Tao
                && flag(ctx.core, p, "l11b_twlingfa_tao_round") == round
                && ctx.core.player(user).alive
                && !super::super::l11_common::he_cards(ctx.core, user).is_empty()
            {
                spawn_npc_b_park(
                    ctx,
                    "twlingfa_tao",
                    p,
                    Some(user),
                    None,
                    Vec::new(),
                    Vec::new(),
                );
            }
            if (kind.is_delayed_trick() || kind.equip_slot().is_some())
                && !u.cancelled
                && has_named_skill(ctx.core, ctx.catalog, p, "twzhian")
                && flag(ctx.core, p, "l11b_twzhian_turn") != tstamp
            {
                let on_field = u
                    .card
                    .is_some_and(|cid| find_field_card_owner(ctx.core, cid).is_some());
                if on_field || ctx.core.player(user).alive {
                    spawn_npc_b_park(
                        ctx,
                        "twzhian",
                        p,
                        Some(user),
                        u.card,
                        Vec::new(),
                        Vec::new(),
                    );
                }
            }
        }
    }
}

pub(super) fn on_phase_skipped_l11_npc_b(ctx: &mut Ctx<'_>, player: PlayerId) {
    if !catalog_has_l11_npc_b(ctx.catalog) {
        return;
    }
    if ctx.core.turn.phase == Phase::Play {
        set_flag(ctx.core, player, "l11b_play_skipped_turn", turn_stamp(ctx.core));
    }
}

pub(super) fn on_phase_use_end_l11_npc_b(ctx: &mut Ctx<'_>, player: PlayerId) {
    if !catalog_has_l11_npc_b(ctx.catalog) {
        return;
    }
    let tstamp = turn_stamp(ctx.core);
    if ctx.core.player(player).alive
        && has_named_skill(ctx.core, ctx.catalog, player, "twmibei")
        && flag(ctx.core, player, "l11b_twmibei_done") == 0
        && flag(ctx.core, player, "l11b_turn_used_card") != tstamp
        && is_unblocked_npc_b(ctx.core, ctx.catalog, player)
    {
        super::helpers::log_skill(ctx, player, "twmibei");
        set_flag(ctx.core, player, "l11b_twmibei_less_stamp", tstamp);
        let keys: Vec<String> = ctx
            .core
            .vm_aux
            .flags
            .keys()
            .filter(|(p, k)| *p == player && k.starts_with("l11b_mibei_"))
            .map(|(_, k)| k.clone())
            .collect();
        for k in keys {
            ctx.core.vm_aux.flags.remove(&(player, k));
        }
    }
}

pub(super) fn on_phase_end_l11_npc_b(ctx: &mut Ctx<'_>) {
    if !catalog_has_l11_npc_b(ctx.catalog) {
        return;
    }
    let cur = ctx.core.turn.current;
    // Settle `twlinglu_order` quests at target's `Phase::End`.
    if ctx.core.player(cur).alive && flag(ctx.core, cur, "l11b_linglu_active") > 0 {
        let dmg = flag(ctx.core, cur, "l11b_linglu_dmg");
        set_flag(ctx.core, cur, "l11b_linglu_active", 0);
        set_flag(ctx.core, cur, "l11b_linglu_dmg", 0);
        if dmg >= 2 {
            ctx.start_draw(cur, 2);
        } else {
            emit_lose_hp(ctx, cur, 1);
        }
    }
    // Settle `yechou2` on every `Phase::End` (survives even after `yechou` owner's death).
    let order = ctx.core.seat_order_from(cur);
    for p in order {
        if ctx.core.player(p).alive && flag(ctx.core, p, "l11b_yechou2") > 0 {
            emit_lose_hp(ctx, p, 1);
        }
    }
}

pub(super) fn on_turn_phase_after_l11_npc_b(ctx: &mut Ctx<'_>, player: PlayerId) {
    if !catalog_has_l11_npc_b(ctx.catalog) {
        return;
    }
    // Clear `twqirang` phase-temp tag at end of any phase (`twqirang_clear`).
    let qr_keys: Vec<String> = ctx
        .core
        .vm_aux
        .flags
        .keys()
        .filter(|(p, k)| *p == player && k.starts_with("l11b_twqirang_card:"))
        .map(|(_, k)| k.clone())
        .collect();
    for k in qr_keys {
        ctx.core.vm_aux.flags.remove(&(player, k));
    }

    // `Phase::End` means the entire turn is finishing for `player`.
    if ctx.core.turn.phase == Phase::End {
        set_flag(ctx.core, player, "l11b_shunshi_stacks", 0);
        set_flag(ctx.core, player, "l11b_twqianxi3_blocked_color", 0);
        set_flag(ctx.core, player, "l11b_zhichi2", 0);
        set_flag(ctx.core, player, "l11b_wangzun_less", 0);
        for p in 0..(ctx.core.players.len() as PlayerId) {
            set_flag(ctx.core, p, "l11b_zhanyi_basic", 0);
        }
        for &t in &alive_all(ctx.core, player) {
            set_flag(ctx.core, t, "l11b_twqianxi2_stamp", 0);
            set_flag(ctx.core, t, "l11b_twqianxi2_color", 0);
        }
    }
}

pub(super) fn on_skill_removed_l11_npc_b(
    core: &mut GameStateCore,
    catalog: &Catalog,
    _owner: PlayerId,
    skill_name: &str,
) {
    if catalog_has_l11_npc_b(catalog) && skill_name == "weifeng" {
        for p in 0..(core.players.len() as PlayerId) {
            set_flag(core, p, "l11b_weifeng_kind", 0);
        }
    }
}

pub(super) fn on_player_died_l11_npc_b(
    ctx: &mut Ctx<'_>,
    dead: PlayerId,
    _killer: Option<PlayerId>,
) {
    if !catalog_has_l11_npc_b(ctx.catalog) {
        return;
    }
    // Clean up expansions/marks on `dead` cleanly.
    for pile in ["l11b_shefu", "l11b_tuifeng", "l11b_zhaoxin", "l11b_yinbing", "l11b_quanji", "l11b_twzhengrong", "l11b_zengdao"] {
        if let Some(cards) = ctx.core.vm_aux.piles.remove(&(dead, pile.to_string())) {
            for c in cards {
                ctx.to_discard(c);
            }
        }
    }
    set_flag(ctx.core, dead, "l11b_yechou2", 0);
    set_flag(ctx.core, dead, "l11b_weifeng_kind", 0);
    if has_named_skill(ctx.core, ctx.catalog, dead, "weifeng") {
        for p in 0..(ctx.core.players.len() as PlayerId) {
            set_flag(ctx.core, p, "l11b_weifeng_kind", 0);
        }
    }
    set_flag(ctx.core, dead, "l11b_twjuntun_init", 0);
}

pub(super) fn on_damage_end_l11_npc_b(ctx: &mut Ctx<'_>, d: &DamageData) {
    if !catalog_has_l11_npc_b(ctx.catalog) || d.prevented || d.amount <= 0 {
        return;
    }
    // `spxiangzhen_draw`: remember Nanman dealt damage.
    if let Some(root_fid) = damage_root_card_use_fid(ctx) {
        if let Some(fr) = ctx.res.frame(root_fid) {
            if let EventData::CardUsed(u) = &fr.data {
                if u.effective_kind(ctx.catalog) == CardKind::SavageAttack {
                    set_flag(ctx.core, 0, &format!("l11b_nanman_dmg:{root_fid}"), 1);
                }
            }
        }
        // `xinkuangfu`: track if xinkuangfu's Sha dealt damage.
        if let Some(src) = d.source {
            if flag(ctx.core, src, "l11b_xinkuangfu_sha_fid") == root_fid {
                set_flag(ctx.core, src, "l11b_xinkuangfu_dealt_dmg", 1);
            }
        }
    }

    // `twxiongzheng`: track per-round damage source mask onto `d.target`.
    if let Some(src) = d.source {
        let tstamp = turn_stamp(ctx.core);
        set_flag(ctx.core, src, "l11b_dmg_dealt_turn", tstamp);
        if let Some(kind) = super::helpers::damage_effective_kind(ctx, d) {
            if kind.is_sha_like() {
                set_flag(
                    ctx.core,
                    src,
                    &format!("l11b_twqianxi_sha_dmg_to:{}", d.target),
                    tstamp,
                );
            }
        }
        let cur_k = format!("l11b_xz_cur_dmg_to:{}", d.target);
        let cur_m = flag(ctx.core, 0, &cur_k);
        set_flag(ctx.core, 0, &cur_k, cur_m | (1u32 << src));

        // `twlinglu_order`: track source damage count on `src`.
        if ctx.core.player(src).alive && flag(ctx.core, src, "l11b_linglu_active") > 0 {
            let cur = flag(ctx.core, src, "l11b_linglu_dmg");
            set_flag(ctx.core, src, "l11b_linglu_dmg", cur + d.amount as u32);
        }

        // `twzhengrong`: first sourceDamage in own phaseUse.
        if ctx.core.turn.current == src
            && ctx.core.turn.phase == Phase::Play
            && ctx.core.player(src).alive
            && has_named_skill(ctx.core, ctx.catalog, src, "twzhengrong")
            && is_unblocked_npc_b(ctx.core, ctx.catalog, src)
        {
            let pstamp = super::helpers::phase_stamp(ctx.core);
            if flag(ctx.core, src, "l11b_zr_dmg_stamp") != pstamp
                && alive_others(ctx.core, src)
                    .into_iter()
                    .any(|p| !super::super::l11_common::he_cards(ctx.core, p).is_empty())
            {
                set_flag(ctx.core, src, "l11b_zr_dmg_stamp", pstamp);
                spawn_npc_b_park(ctx, "twzhengrong", src, None, None, Vec::new(), Vec::new());
            }
        }
    }

    // `xianfu2`: when linked target takes damage, mirror damage loss (nosource).
    let amt = d.amount.max(0) as u32;
    let order = ctx.core.seat_order_from(ctx.core.turn.current);
    for p in order {
        if !ctx.core.player(p).alive
            || flag(ctx.core, p, "l11b_xianfu_target") != u32::from(d.target) + 1
            || !ctx.core.player(d.target).alive
            || !has_named_skill(ctx.core, ctx.catalog, p, "xianfu")
            || !is_unblocked_npc_b(ctx.core, ctx.catalog, p)
        {
            continue;
        }
        super::helpers::log_skill(ctx, p, "xianfu");
        super::super::l11_common::emit_damage(
            ctx,
            None,
            p,
            amt,
            crate::event::DamageNature::Normal,
            None,
        );
    }
}

pub(super) fn on_hp_changed_l11_npc_b(ctx: &mut Ctx<'_>, player: PlayerId, delta: i32) {
    if !catalog_has_l11_npc_b(ctx.catalog) {
        return;
    }
    if delta < 0 {
        // `xiehui2`: losing HP clears all `xiehui` card locks on `player`.
        let xk: Vec<String> = ctx
            .core
            .vm_aux
            .flags
            .keys()
            .filter(|(p, k)| *p == player && k.starts_with("l11b_xiehui2_card:"))
            .map(|(_, k)| k.clone())
            .collect();
        for k in xk {
            ctx.core.vm_aux.flags.remove(&(player, k));
        }
    } else if delta > 0 {
        // `xianfu2`: when linked target recovers HP, mirror recovery if damaged.
        let order = ctx.core.seat_order_from(ctx.core.turn.current);
        for p in order {
            if ctx.core.player(p).alive
                && flag(ctx.core, p, "l11b_xianfu_target") == u32::from(player) + 1
                && ctx.core.player(p).hp < ctx.core.player(p).max_hp as i32
                && has_named_skill(ctx.core, ctx.catalog, p, "xianfu")
                && is_unblocked_npc_b(ctx.core, ctx.catalog, p)
            {
                super::helpers::log_skill(ctx, p, "xianfu");
                ctx.recover(p, delta as u32, None);
            }
        }
    }
}

pub(super) fn on_equip_revoked_l11_npc_b(
    core: &mut GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    card: CardId,
) {
    if !catalog_has_l11_npc_b(catalog) {
        return;
    }
    if (card as usize) < catalog.cards.len() && catalog.cards[card as usize].name == "taipingyaoshu"
    {
        if !crate::rvm_bridge::modifiers::is_target_unequip2(core, player) {
            let cur = flag(core, player, "l11b_tpys_lost_pending");
            set_flag(core, player, "l11b_tpys_lost_pending", cur + 1);
        }
    }
}

pub(super) fn on_outer_settled_l11_npc_b(ctx: &mut Ctx<'_>) {
    if !catalog_has_l11_npc_b(ctx.catalog) {
        return;
    }
    let order = ctx.core.seat_order_from(ctx.core.turn.current);
    for p in order {
        let pending = flag(ctx.core, p, "l11b_tpys_lost_pending");
        if pending > 0 && ctx.core.player(p).alive {
            set_flag(ctx.core, p, "l11b_tpys_lost_pending", 0);
            for _ in 0..pending {
                if !ctx.core.player(p).alive {
                    break;
                }
                super::helpers::log_skill(ctx, p, "taipingyaoshu");
                if ctx.core.player(p).hp > 1 {
                    emit_lose_hp(ctx, p, 1);
                }
                ctx.start_draw(p, 2);
            }
        }
    }
}

pub(super) fn on_host_event_l11_npc_b(ctx: &mut Ctx<'_>, event: &EventData) {
    if !catalog_has_l11_npc_b(ctx.catalog) {
        return;
    }
    if let EventData::Phase {
        player,
        phase: Phase::Prepare,
        begin: true,
    } = event
    {
        // `yechou2` expires when the affected player starts their turn (`phaseZhunbeiBegin`).
        set_flag(ctx.core, *player, "l11b_yechou2", 0);
        // `wangzun2` owner unmark (handled on owner prepare).
        set_flag(ctx.core, *player, "l11b_wangzun_active", 0);
        // `weifeng3`: clear all `l11b_weifeng_kind` marks at owner's Prepare begin.
        if has_named_skill(ctx.core, ctx.catalog, *player, "weifeng") {
            for p in 0..(ctx.core.players.len() as PlayerId) {
                set_flag(ctx.core, p, "l11b_weifeng_kind", 0);
            }
        }
    } else if let EventData::TurnEnded { .. } = event {
        for p in 0..(ctx.core.players.len() as PlayerId) {
            set_flag(ctx.core, p, "l11b_zhanyi_basic", 0);
        }
    }
}

pub(super) fn steal_field_card(ctx: &mut Ctx<'_>, owner: PlayerId, cid: CardId) -> bool {
    if let Some(holder) = find_field_card_owner(ctx.core, cid) {
        move_cards_to_hand(ctx, holder, owner, &[cid]);
        return true;
    }
    false
}

pub(super) fn discard_field_card(ctx: &mut Ctx<'_>, cid: CardId) -> bool {
    if let Some(holder) = find_field_card_owner(ctx.core, cid) {
        discard_player_cards(ctx, holder, &[cid]);
        return true;
    }
    false
}
