//! Batch U05-6b (`shahai-hearth` Part 2): 生命周期与内核挂点。

use super::{
    b6b_flag, clear_b6b_flag, has_skill_b6b, hs_b6b_cnt, hs_b6b_inc, inc_b6b_flag,
    pop_stashed_card, push_hearth_machine, set_b6b_flag,
};
use super::super::u05_hearth::{
    alive_others, base_card_name, discard_cards_from_zone, exact_base_name,
    finish_table_card_hearth, gain_card_to_hand, has_any_skill, hearth_formal_enabled,
    remove_skill,
};
use super::super::HostBehavior;
use crate::catalog::{CardKind, Catalog};
use crate::event::{CardUseData, DamageNature, EventData, Phase};
use crate::ids::{CardId, PlayerId, SkillDefId};
use crate::machine::{Ctx, LogEvent};
use crate::state::GameStateCore;

pub(crate) fn b6b_formal_enabled(catalog: &Catalog) -> bool {
    catalog.skill_by_name("xieneng").is_some_and(|d| {
        catalog.skills[d as usize].behavior.as_host_behavior()
            == Some(HostBehavior::U05Hearth2BatchA)
    })
}

pub(crate) fn b6b_on_game_start(core: &mut GameStateCore, catalog: &Catalog) {
    if !hearth_formal_enabled(catalog) && !b6b_formal_enabled(catalog) {
        return;
    }
    let seats = core.players.len() as PlayerId;
    for p in 0..seats {
        if has_skill_b6b(core, catalog, p, "chongsheng") {
            set_b6b_flag(core, p, "hs_chongsheng_init", 1);
            set_b6b_flag(core, p, "hs_chongsheng_left", 2);
        }
    }
}

pub(crate) fn b6b_on_turn_begin(ctx: &mut Ctx<'_>, player: PlayerId) {
    if !b6b_formal_enabled(ctx.catalog) {
        return;
    }
    let seats = ctx.core.players.len() as PlayerId;
    for p in 0..seats {
        clear_b6b_flag(ctx.core, p, "hs_active_extra_turn_source");
        if b6b_flag(ctx.core, p, "hs_bingjia_mianyi") > 0 {
            clear_b6b_flag(ctx.core, p, "hs_bingjia_mianyi");
            remove_skill(ctx, p, "mianyi");
        }
    }
    let extra = ctx
        .core
        .vm_aux
        .flags
        .get(&(0, "extra_turn_in_progress".to_string()))
        .copied()
        .unwrap_or(0)
        > 0;
    if extra {
        let src = b6b_flag(ctx.core, player, "hs_queued_extra_turn_source");
        clear_b6b_flag(ctx.core, player, "hs_queued_extra_turn_source");
        if src > 0 {
            set_b6b_flag(ctx.core, player, "hs_active_extra_turn_source", src);
        }
    }
}

pub(crate) fn b6b_on_draw_begin(ctx: &mut Ctx<'_>, player: PlayerId, pending: &mut i32) {
    if !b6b_formal_enabled(ctx.catalog) || *pending <= 0 {
        return;
    }
    if has_skill_b6b(ctx.core, ctx.catalog, player, "stuxi2")
        && !ctx.catalog.is_tieji_sealed(ctx.core, player)
    {
        *pending -= 1;
        remove_skill(ctx, player, "stuxi2");
        ctx.log(LogEvent::SkillTriggered {
            skill: "stuxi2",
            owner: player,
            presentation: None,
        });
    }
}

pub(crate) fn b6b_on_phase_use_end(ctx: &mut Ctx<'_>, player: PlayerId) {
    if !b6b_formal_enabled(ctx.catalog) {
        return;
    }
    let actions = b6b_flag(ctx.core, player, "hs_shixu_actions");
    let explicit_ms = b6b_flag(ctx.core, player, "hs_shixu_elapsed_ms");
    let ms = explicit_ms.max(actions.saturating_mul(1000));
    clear_b6b_flag(ctx.core, player, "hs_shixu_actions");
    clear_b6b_flag(ctx.core, player, "hs_shixu_elapsed_ms");
    let shixu_present = (0..ctx.core.players.len() as PlayerId).any(|p| {
        ctx.core.player(p).alive
            && has_skill_b6b(ctx.core, ctx.catalog, p, "shixu")
            && !ctx.catalog.is_tieji_sealed(ctx.core, p)
    });
    if shixu_present && ms > 3000 {
        set_b6b_flag(ctx.core, player, "hs_shixu_duration", ms);
    } else {
        clear_b6b_flag(ctx.core, player, "hs_shixu_duration");
    }
}

pub(crate) fn b6b_on_phase_end(ctx: &mut Ctx<'_>) {
    if !b6b_formal_enabled(ctx.catalog) {
        return;
    }
    let seats = ctx.core.players.len() as PlayerId;
    for p in 0..seats {
        clear_b6b_flag(ctx.core, p, "hs_duzhang_used_trick");
        clear_b6b_flag(ctx.core, p, "hs_turn_dealt_damage");
        clear_b6b_flag(ctx.core, p, "hs_b6b_turn_card_used");
        clear_b6b_flag(ctx.core, p, "hs_yufa_attacker");
        clear_b6b_flag(ctx.core, p, "hs_shixu_duration");
        clear_b6b_flag(ctx.core, p, "hs_shixu_actions");
        clear_b6b_flag(ctx.core, p, "hs_shixu_elapsed_ms");
        clear_b6b_flag(ctx.core, p, "hs_jingmeng_trig");
        clear_b6b_flag(ctx.core, p, "hs_fbeifa_parked");
        clear_b6b_flag(ctx.core, p, "hs_fengxing_parked");
        clear_b6b_flag(ctx.core, p, "hs_jingxiang_temp_card");
        if has_any_skill(ctx.core, ctx.catalog, p, "shengyan2") {
            remove_skill(ctx, p, "shengyan2");
        }
        if has_any_skill(ctx.core, ctx.catalog, p, "fengxing2") {
            remove_skill(ctx, p, "fengxing2");
        }
        let b6b_cnt_keys: Vec<String> = ctx
            .core
            .vm_aux
            .flags
            .keys()
            .filter_map(|(seat, k)| {
                (*seat == p
                    && (k.starts_with("hs_b6b_cnt:") || k.starts_with("hs_b6b_name_used:")))
                .then(|| k.clone())
            })
            .collect();
        for k in b6b_cnt_keys {
            ctx.core.vm_aux.flags.remove(&(p, k));
        }
        if !ctx.core.player(p).alive && b6b_flag(ctx.core, p, "hs_huanfeng_active") > 0 {
            clear_b6b_flag(ctx.core, p, "hs_huanfeng_active");
            while let Some(c) = pop_stashed_card(ctx.core, p, "hs_huanfeng_hand") {
                ctx.to_discard(c);
            }
            while let Some(c) = pop_stashed_card(ctx.core, p, "hs_huanfeng_eq") {
                ctx.to_discard(c);
            }
        }
    }
}

pub(crate) fn b6b_on_outer_settled(ctx: &mut Ctx<'_>) {
    if !b6b_formal_enabled(ctx.catalog) {
        return;
    }
    let has_dmg = ctx
        .res
        .stack
        .iter()
        .any(|f| matches!(f.data, EventData::Damage(_) | EventData::DamageWindow { .. }));
    if !has_dmg {
        let seats = ctx.core.players.len() as PlayerId;
        for p in 0..seats {
            clear_b6b_flag(ctx.core, p, "hs_hhudun_hujia_full");
        }
    }
    let stale_frames: Vec<(PlayerId, String)> = ctx
        .core
        .vm_aux
        .flags
        .keys()
        .filter(|(_, k)| {
            let fid = k
                .strip_prefix("hs_fbeifa_frame:")
                .or_else(|| k.strip_prefix("hs_oldfbeifa_frame:"))
                .or_else(|| k.strip_prefix("hs_huanjue_frame:"))
                .and_then(|s| s.parse::<u32>().ok());
            fid.is_some_and(|id| !ctx.res.stack.iter().any(|f| f.id == id))
        })
        .cloned()
        .collect();
    for key in stale_frames {
        ctx.core.vm_aux.flags.remove(&key);
    }
    let seats = ctx.core.players.len() as PlayerId;
    for p in 0..seats {
        for sk in ["fbeifa", "oldfbeifa"] {
            let pref = format!("hs_{sk}_frame:");
            let still_live = ctx
                .core
                .vm_aux
                .flags
                .keys()
                .any(|&(seat, ref k)| seat == p && k.starts_with(&pref));
            if !still_live {
                clear_b6b_flag(ctx.core, p, &format!("hs_{sk}_live"));
            }
        }
    }
    if !ctx.res.stack.iter().any(|f| matches!(f.data, EventData::CardUsed(_))) {
        for p in 0..seats {
            clear_b6b_flag(ctx.core, p, "hs_qianghua_firing");
        }
    }
    let pending_fulei: Vec<(CardId, PlayerId)> = ctx
        .core
        .vm_aux
        .flags
        .iter()
        .filter_map(|((seat, k), &val)| {
            if *seat == 0 && val > 0 {
                k.strip_prefix("hs_fulei_pending_move:")
                    .and_then(|s| s.parse::<CardId>().ok())
                    .map(|cid| (cid, (val - 1) as PlayerId))
            } else {
                None
            }
        })
        .collect();
    for (trick, preferred_next) in pending_fulei {
        ctx.core
            .vm_aux
            .flags
            .remove(&(0, format!("hs_fulei_pending_move:{trick}")));
        let expired = ctx
            .core
            .vm_aux
            .flags
            .remove(&(0, format!("hs_fulei_expired:{trick}")))
            .unwrap_or(0)
            > 0;
        if ctx.core.table.contains(&trick) {
            if expired {
                finish_table_card_hearth(ctx, trick, true);
                ctx.core
                    .vm_aux
                    .flags
                    .remove(&(0, format!("hs_fulei_hits:{trick}")));
            } else {
                let target = if (preferred_next as usize) < ctx.core.players.len()
                    && ctx.core.player(preferred_next).alive
                {
                    Some(preferred_next)
                } else {
                    ctx.core.alive_players().next()
                };
                if let Some(nxt) = target {
                    let has_fulei = ctx
                        .core
                        .player(nxt)
                        .judge
                        .iter()
                        .any(|&c| exact_base_name(ctx.catalog.cards[c as usize].name) == "fulei");
                    if has_fulei {
                        finish_table_card_hearth(ctx, trick, true);
                        ctx.core
                            .vm_aux
                            .flags
                            .remove(&(0, format!("hs_fulei_hits:{trick}")));
                    } else {
                        ctx.core.table.retain(|&c| c != trick);
                        ctx.core.player_mut(nxt).judge.push(trick);
                    }
                } else {
                    finish_table_card_hearth(ctx, trick, true);
                    ctx.core
                        .vm_aux
                        .flags
                        .remove(&(0, format!("hs_fulei_hits:{trick}")));
                }
            }
        }
    }
}

pub(crate) fn b6b_on_card_use_after(ctx: &mut Ctx<'_>, u: &CardUseData) {
    if !b6b_formal_enabled(ctx.catalog) || u.cancelled {
        return;
    }
    let user = u.player;
    inc_b6b_flag(ctx.core, user, "hs_b6b_turn_card_used");
    let eff = u.effective_kind(ctx.catalog);
    if !eff.is_basic_kind() && eff.equip_slot().is_none() {
        set_b6b_flag(ctx.core, user, "hs_duzhang_used_trick", 1);
    }
    let last_vsrc = ctx
        .core
        .vm_aux
        .flags
        .remove(&(user, "hs_last_virtual_src".to_string()))
        .unwrap_or(0);
    let name_key = super::b6b_card_name_key(ctx.core, ctx.catalog, u, last_vsrc);
    inc_b6b_flag(ctx.core, user, &format!("hs_b6b_name_used:{name_key}"));
    if ctx.core.turn.phase == Phase::Play && ctx.core.turn.current == user {
        let shixu_present = (0..ctx.core.players.len() as PlayerId).any(|p| {
            ctx.core.player(p).alive
                && has_skill_b6b(ctx.core, ctx.catalog, p, "shixu")
                && !ctx.catalog.is_tieji_sealed(ctx.core, p)
        });
        if shixu_present {
            inc_b6b_flag(ctx.core, user, "hs_shixu_actions");
        }
    }

    // `qianghua` (`character/hearth/skill.js:4931-4995`): useCardAfter in own Play phase
    let is_qianghua_parent = b6b_flag(ctx.core, user, "hs_qianghua_firing") > 0;
    if is_qianghua_parent {
        clear_b6b_flag(ctx.core, user, "hs_qianghua_firing");
    }
    let src_named_cid = super::b6b_resolve_src_card(ctx.core, ctx.catalog, u, last_vsrc);
    let is_shihuawuqi = src_named_cid
        .is_some_and(|c| base_card_name(ctx.catalog.cards[c as usize].name) == "shihuawuqi");
    let tao_ok = eff != CardKind::Tao
        || is_shihuawuqi
        || ctx.core.player(user).hp < ctx.core.player(user).max_hp as i32;
    let needs_explicit_target = eff.is_sha_like()
        || matches!(
            eff,
            CardKind::Duel
                | CardKind::Dismantle
                | CardKind::Snatch
                | CardKind::Chain
                | CardKind::Linghunzhihuo
                | CardKind::Zhaomingdan
                | CardKind::Shijieshu
                | CardKind::Liuxinghuoyu
                | CardKind::Zhiliaobo
        )
        || src_named_cid.is_some_and(|c| {
            let b = base_card_name(ctx.catalog.cards[c as usize].name);
            b == "shandianjian" || b == "huogong"
        });
    let targets_ok = if u.targets.is_empty() {
        !needs_explicit_target
    } else {
        u.targets.iter().all(|&t| {
            super::is_b6b_card_target_legal(
                ctx.core,
                ctx.catalog,
                user,
                t,
                src_named_cid,
                src_named_cid.or(u.card),
                eff,
                u.nature,
            )
        })
    };
    let ok_type = eff.is_basic_kind()
        || (!eff.is_basic_kind()
            && eff.equip_slot().is_none()
            && !super::super::u05_hearth::is_delayed_trick_hearth(eff));
    if !is_qianghua_parent
        && ctx.core.player(user).alive
        && ctx.core.turn.current == user
        && ctx.core.turn.phase == Phase::Play
        && has_skill_b6b(ctx.core, ctx.catalog, user, "qianghua")
        && !ctx.catalog.is_tieji_sealed(ctx.core, user)
        && hs_b6b_cnt(ctx.core, user, "qianghua") == 0
        && ok_type
        && eff != CardKind::Jiedao
        && tao_ok
        && targets_ok
    {
        set_b6b_flag(
            ctx.core,
            user,
            "hs_qianghua_eff_kind",
            u32::from(crate::archive::kind_code(eff)),
        );
        let nat_code = match u.nature {
            DamageNature::Normal => 0,
            DamageNature::Fire => 1,
            DamageNature::Thunder => 2,
        };
        set_b6b_flag(ctx.core, user, "hs_qianghua_nature", nat_code);
        push_hearth_machine(
            ctx,
            HostBehavior::U05Hearth2BatchB,
            "qianghua",
            user,
            u.targets.first().copied(),
            src_named_cid,
            u.targets.clone(),
            Vec::new(),
        );
    }
}

pub(crate) fn b6b_on_hand_card_removed(ctx: &mut Ctx<'_>, player: PlayerId, card: CardId) {
    if !b6b_formal_enabled(ctx.catalog) || !ctx.core.player(player).alive {
        return;
    }
    if b6b_flag(ctx.core, player, "hs_jingxiang_temp_card") == card + 1 {
        clear_b6b_flag(ctx.core, player, "hs_jingxiang_temp_card");
        return;
    }
    let sealed = ctx.catalog.is_tieji_sealed(ctx.core, player);
    if sealed {
        return;
    }
    if ctx.core.player(player).hand.is_empty() {
        for sk in ["fbeifa", "oldfbeifa"] {
            if has_skill_b6b(ctx.core, ctx.catalog, player, sk)
                && hs_b6b_cnt(ctx.core, player, sk) < 3
                && b6b_flag(ctx.core, player, "hs_fbeifa_parked") == 0
                && !super::sha_target_options(ctx.core, ctx.catalog, player).is_empty()
            {
                set_b6b_flag(ctx.core, player, "hs_fbeifa_parked", 1);
                let beh = HostBehavior::U05Hearth2BatchA;
                push_hearth_machine(ctx, beh, sk, player, None, None, Vec::new(), Vec::new());
            }
        }
    }
    if ctx.core.turn.current != player
        && has_skill_b6b(ctx.core, ctx.catalog, player, "fengxing")
        && !has_any_skill(ctx.core, ctx.catalog, player, "fengxing2")
        && b6b_flag(ctx.core, player, "hs_fengxing_parked") == 0
        && (!ctx.core.player(player).hand.is_empty() || !ctx.core.player(player).equips.is_empty())
    {
        set_b6b_flag(ctx.core, player, "hs_fengxing_parked", 1);
        push_hearth_machine(
            ctx,
            HostBehavior::U05Hearth2BatchD,
            "fengxing",
            player,
            None,
            None,
            Vec::new(),
            Vec::new(),
        );
    }
}

pub(crate) fn b6b_on_skill_removed(
    core: &mut GameStateCore,
    _catalog: &Catalog,
    owner: PlayerId,
    skill_name: &str,
) -> Vec<CardId> {
    let mut ret = Vec::new();
    match skill_name {
        "bingjia2" => {
            if let Some(c) = pop_stashed_card(core, owner, "hs_bingjia") {
                ret.push(c);
            }
        }
        "mdzhoufu2" => {
            if let Some(c) = pop_stashed_card(core, owner, "hs_mdzhoufu2") {
                ret.push(c);
            }
            clear_b6b_flag(core, owner, "hs_mdzhoufu3");
        }
        "mengun2" => {
            clear_b6b_flag(core, owner, "hs_mengun2_card");
        }
        "bingshi" | "bingshi2" => {
            if !core.player(owner).alive {
                set_b6b_flag(core, owner, "hs_bingshi_died", 1);
            }
        }
        "huanfeng" => {
            clear_b6b_flag(core, owner, "hs_huanfeng_pending_extra");
            if b6b_flag(core, owner, "hs_huanfeng_active") > 0 {
                clear_b6b_flag(core, owner, "hs_huanfeng_active");
                while let Some(c) = pop_stashed_card(core, owner, "hs_huanfeng_hand") {
                    ret.push(c);
                }
                while let Some(c) = pop_stashed_card(core, owner, "hs_huanfeng_eq") {
                    ret.push(c);
                }
            }
        }
        _ => {}
    }
    ret
}

pub(crate) fn b6b_on_player_died(ctx: &mut Ctx<'_>, dead: PlayerId) {
    if !b6b_formal_enabled(ctx.catalog) {
        return;
    }
    clear_b6b_flag(ctx.core, dead, "hs_huanfeng_pending_extra");
    if b6b_flag(ctx.core, dead, "hs_huanfeng_active") > 0 {
        clear_b6b_flag(ctx.core, dead, "hs_huanfeng_active");
        while let Some(c) = pop_stashed_card(ctx.core, dead, "hs_huanfeng_hand") {
            ctx.to_discard(c);
        }
        while let Some(c) = pop_stashed_card(ctx.core, dead, "hs_huanfeng_eq") {
            ctx.to_discard(c);
        }
    }
    let died_bingshi = b6b_flag(ctx.core, dead, "hs_bingshi_died") > 0
        || has_skill_b6b(ctx.core, ctx.catalog, dead, "bingshi");
    clear_b6b_flag(ctx.core, dead, "hs_bingshi_died");
    if died_bingshi {
        let targets: Vec<PlayerId> = ctx
            .core
            .seat_order_from(dead)
            .into_iter()
            .filter(|&p| p != dead && ctx.core.player(p).alive)
            .collect();
        for t in targets.into_iter().rev() {
            ctx.emit(EventData::Damage(super::super::u05_hearth::make_damage(
                None,
                t,
                1,
                crate::event::DamageNature::Thunder,
                None,
            )));
        }
    }
}

pub(crate) fn b6b_dying_save_skills(
    core: &GameStateCore,
    catalog: &Catalog,
    responder: PlayerId,
    dying: PlayerId,
) -> Vec<SkillDefId> {
    let mut out = Vec::new();
    if !b6b_formal_enabled(catalog)
        || responder != dying
        || !core.player(responder).alive
        || catalog.is_tieji_sealed(core, responder)
    {
        return out;
    }
    if has_skill_b6b(core, catalog, responder, "chongsheng") {
        let left = if b6b_flag(core, responder, "hs_chongsheng_init") == 0 {
            2
        } else {
            b6b_flag(core, responder, "hs_chongsheng_left")
        };
        if left > 0 {
            if let Some(def) = catalog.skill_by_name("chongsheng") {
                out.push(def);
            }
        }
    }
    out
}

pub(crate) fn b6b_resolve_dying_save(
    ctx: &mut Ctx<'_>,
    responder: PlayerId,
    dying: PlayerId,
    def: SkillDefId,
) -> bool {
    if ctx.catalog.skill_name(def) != "chongsheng" || responder != dying {
        return false;
    }
    let left = if b6b_flag(ctx.core, dying, "hs_chongsheng_init") == 0 {
        set_b6b_flag(ctx.core, dying, "hs_chongsheng_init", 1);
        set_b6b_flag(ctx.core, dying, "hs_chongsheng_left", 2);
        2
    } else {
        b6b_flag(ctx.core, dying, "hs_chongsheng_left")
    };
    if left == 0 {
        return false;
    }
    let mhp = ctx.core.player(dying).max_hp;
    ctx.core.player_mut(dying).hp = (left.min(mhp)) as i32;
    ctx.core.player_mut(dying).dying = false;
    set_b6b_flag(ctx.core, dying, "hs_chongsheng_left", left - 1);
    ctx.log(LogEvent::SkillTriggered {
        skill: "chongsheng",
        owner: dying,
        presentation: None,
    });
    let all_hej: Vec<CardId> = ctx
        .core
        .player(dying)
        .hand
        .iter()
        .chain(ctx.core.player(dying).equips.iter())
        .chain(ctx.core.player(dying).judge.iter())
        .copied()
        .collect();
    if !all_hej.is_empty() {
        discard_cards_from_zone(ctx, dying, &all_hej);
        ctx.log(LogEvent::CardDiscarded {
            player: dying,
            cards: all_hej,
        });
    }
    if ctx.core.player(dying).linked {
        ctx.core.player_mut(dying).linked = false;
        ctx.log(LogEvent::Linked {
            player: dying,
            linked: false,
        });
    }
    if ctx.core.player(dying).face_down {
        ctx.emit(EventData::TurnOverBefore {
            player: dying,
            cancelled: false,
        });
    }
    ctx.start_draw(dying, left);
    false
}

/// `xianzhi` (`character/hearth/skill.js:6592-6636`): `global: "judgeBegin"`.
pub(crate) fn host_on_judge_pre_hearth(ctx: &mut Ctx<'_>, judge_owner: PlayerId) -> bool {
    if !b6b_formal_enabled(ctx.catalog) || b6b_flag(ctx.core, judge_owner, "hs_xianzhi_asked") > 0 {
        return false;
    }
    if ctx.core.deck.len() <= 1 {
        return false;
    }
    let holder = ctx.core.seat_order_from(judge_owner).into_iter().find(|&p| {
        ctx.core.player(p).alive
            && has_skill_b6b(ctx.core, ctx.catalog, p, "xianzhi")
            && !ctx.catalog.is_tieji_sealed(ctx.core, p)
    });
    let Some(h) = holder else {
        return false;
    };
    set_b6b_flag(ctx.core, judge_owner, "hs_xianzhi_asked", 1);
    push_hearth_machine(
        ctx,
        HostBehavior::U05Hearth2BatchE,
        "xianzhi",
        h,
        Some(judge_owner),
        None,
        Vec::new(),
        Vec::new(),
    );
    true
}

/// `mdzhoufu2` (`character/hearth/skill.js:6559-6591`): `trigger: { player: "judge" }`.
pub(crate) fn host_on_judge_rewrite_pre_hearth(
    ctx: &mut Ctx<'_>,
    judge_owner: PlayerId,
    judge_card: &mut CardId,
) -> bool {
    if !b6b_formal_enabled(ctx.catalog)
        || !has_skill_b6b(ctx.core, ctx.catalog, judge_owner, "mdzhoufu2")
    {
        return false;
    }
    let Some(new_card) = pop_stashed_card(ctx.core, judge_owner, "hs_mdzhoufu2") else {
        return false;
    };
    let old = *judge_card;
    *judge_card = new_card;
    let giver_plus = b6b_flag(ctx.core, judge_owner, "hs_mdzhoufu3");
    clear_b6b_flag(ctx.core, judge_owner, "hs_mdzhoufu3");
    remove_skill(ctx, judge_owner, "mdzhoufu2");
    if giver_plus > 0 {
        let giver = (giver_plus - 1) as PlayerId;
        if ctx.core.player(giver).alive {
            gain_card_to_hand(ctx, giver, old);
            ctx.emit(EventData::CardsMoved {
                cards: vec![old],
                from: judge_owner,
                to: giver,
            });
        } else {
            finish_table_card_hearth(ctx, old, true);
        }
    } else {
        finish_table_card_hearth(ctx, old, true);
    }
    ctx.log(LogEvent::JudgementRewritten {
        player: judge_owner,
        card: new_card,
        replaced: old,
    });
    true
}

pub(crate) fn hearth_response_skill_jingxiang(
    core: &GameStateCore,
    catalog: &Catalog,
    responder: PlayerId,
    resp_ctx: u8,
) -> Option<SkillDefId> {
    if !b6b_formal_enabled(catalog)
        || !core.player(responder).alive
        || catalog.blocks_all_cards(core, responder)
        || !has_skill_b6b(core, catalog, responder, "jingxiang")
        || catalog.is_tieji_sealed(core, responder)
        || hs_b6b_cnt(core, responder, "jingxiang") > 0
    {
        return None;
    }
    let any_hand = alive_others(core, responder)
        .into_iter()
        .any(|p| !core.player(p).hand.is_empty());
    if !any_hand {
        return None;
    }
    let _ = resp_ctx;
    catalog.skill_by_name("jingxiang")
}

pub(crate) fn jingxiang_spawn_response(
    ctx: &mut Ctx<'_>,
    responder: PlayerId,
    resp_ctx: u8,
    dying: PlayerId,
) {
    hs_b6b_inc(ctx.core, responder, "jingxiang");
    push_hearth_machine(
        ctx,
        HostBehavior::U05Hearth2BatchE,
        "jingxiang",
        responder,
        (resp_ctx == 4 || resp_ctx == 5).then_some(dying),
        None,
        Vec::new(),
        vec![u32::from(resp_ctx)],
    );
}
