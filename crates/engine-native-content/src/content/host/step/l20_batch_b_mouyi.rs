//! L20 标记补充 Batch B 使命/谋弈/残余技能步进实现（`l20_batch_b_mouyi.rs`）：
//! 覆盖 `tspowei`、`qingyu`、`mibei`、`xinfu_lingren`、`sbtieji`、
//! `sbduanliang`、`shouye`、`mbmowang`（`zuoxing` 在上游 `extension.js:6155-6204`
//! 整段注释掉——L20 不覆盖该技能）。

use super::*;
use crate::catalog::CardKind;
use crate::choice::ChoiceOption;
use crate::event::{CardUseData, DamageNature, EventData};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow};

fn check_tspowei_achieve(ctx: &mut Ctx<'_>, owner: PlayerId) -> bool {
    if flag(ctx.core, owner, "tspowei_init") > 0
        && flag(ctx.core, owner, "tspowei_resolved") == 0
        && !alive_others(ctx.core, owner)
            .iter()
            .any(|&p| flag(ctx.core, p, "tspowei_wei") > 0)
    {
        set_flag(ctx.core, owner, "tspowei_resolved", 1);
        set_flag(ctx.core, owner, "tspowei_succeeded", 1);
        set_flag(ctx.core, owner, "tspowei_achieved", 1);
        grant_skill_named(ctx, owner, "shenzhu");
        set_flag(ctx.core, owner, "shenzhu_granted", 1);
        return true;
    }
    false
}

pub(super) fn handle_l20_duty_three(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.def_key.as_str() {
        "tspowei" => match m.stage {
            HostStage::Act => match m.card {
                Some(0) | Some(2) => {
                    let is_turn_begin = m.card == Some(0);
                    let turn_p = m.target.unwrap_or(owner);
                    if is_turn_begin {
                        set_flag(ctx.core, owner, "tspowei_in_range", 0);
                        set_flag(ctx.core, owner, "tspowei_turn_prompted", 0);
                    }
                    if flag(ctx.core, owner, "tspowei_init") == 0 {
                        set_flag(ctx.core, owner, "tspowei_init", 1);
                        for p in alive_others(ctx.core, owner) {
                            set_flag(ctx.core, p, "tspowei_wei", 1);
                            set_flag(ctx.core, p, "dulie", 1);
                        }
                        if turn_p != owner && flag(ctx.core, turn_p, "tspowei_wei") > 0 {
                            let can_act = !ctx.core.player(owner).hand.is_empty()
                                || (ctx.core.player(owner).hp >= ctx.core.player(turn_p).hp
                                    && !ctx.core.player(turn_p).hand.is_empty());
                            if can_act {
                                set_flag(
                                    ctx.core,
                                    owner,
                                    "tspowei_turn_prompted",
                                    u32::from(turn_p) + 1,
                                );
                                return wait_ask(m, ctx, ASK_TSPOWEI_CHOOSE_ACTION);
                            }
                        }
                        return Flow::Done;
                    }
                    if turn_p == owner {
                        if !is_turn_begin && check_tspowei_achieve(ctx, owner) {
                            return Flow::Done;
                        }
                        if is_turn_begin {
                            let others = alive_others(ctx.core, owner);
                            let marked: Vec<PlayerId> = others
                                .iter()
                                .copied()
                                .filter(|&p| flag(ctx.core, p, "tspowei_wei") > 0)
                                .collect();
                            for &p in &marked {
                                set_flag(ctx.core, p, "tspowei_wei", 0);
                                set_flag(ctx.core, p, "dulie", 0);
                            }
                            if !others.is_empty() {
                                for p in marked {
                                    let pos = others.iter().position(|&x| x == p).unwrap_or(0);
                                    let next = others[(pos + 1) % others.len()];
                                    set_flag(ctx.core, next, "tspowei_wei", 1);
                                    set_flag(ctx.core, next, "dulie", 1);
                                }
                            }
                        }
                        return Flow::Done;
                    }
                    if flag(ctx.core, turn_p, "tspowei_wei") > 0 {
                        set_flag(
                            ctx.core,
                            owner,
                            "tspowei_turn_prompted",
                            u32::from(turn_p) + 1,
                        );
                        return wait_ask(m, ctx, ASK_TSPOWEI_CHOOSE_ACTION);
                    }
                    Flow::Done
                }
                Some(1) => {
                    let t = m.target.unwrap_or(owner);
                    set_flag(ctx.core, t, "tspowei_wei", 0);
                    set_flag(ctx.core, t, "dulie", 0);
                    Flow::Done
                }
                Some(4) => {
                    set_flag(ctx.core, owner, "tspowei_in_range", 0);
                    Flow::Done
                }
                Some(3) => {
                    set_flag(ctx.core, owner, "tspowei_resolved", 1);
                    set_flag(ctx.core, owner, "tspowei_failed", 1);
                    for p in 0..ctx.core.players.len() as PlayerId {
                        set_flag(ctx.core, p, "tspowei_wei", 0);
                        set_flag(ctx.core, p, "dulie", 0);
                    }
                    if ctx.core.player(owner).hp < 1 {
                        let heal = (1 - ctx.core.player(owner).hp) as u32;
                        ctx.recover(owner, heal, None);
                    }
                    let eqs = ctx.core.player(owner).equips.clone();
                    if !eqs.is_empty() {
                        discard_cards(ctx, owner, &eqs);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostStage::SelectCard => {
                let ask_code = m.card.unwrap_or(0);
                let t = m.target.unwrap_or(owner);
                if ask_code == ASK_TSPOWEI_CHOOSE_ACTION {
                    let chosen = ctx.answer().iter().find_map(|o| match o {
                        ChoiceOption::Tag(tag) => Some(*tag),
                        _ => None,
                    });
                    let Some(act) = chosen else {
                        return Flow::Done;
                    };
                    set_flag(ctx.core, owner, "tspowei_in_range", u32::from(t) + 1);
                    if act == 0 {
                        if ctx.core.player(owner).hand.is_empty() {
                            return Flow::Done;
                        }
                        return wait_ask(m, ctx, ASK_TSPOWEI_CHOOSE_CARD);
                    }
                    if ctx.core.player(t).hand.len() > 1 {
                        return wait_ask(m, ctx, ASK_TSPOWEI_GAIN_CARD);
                    }
                    if let Some(&c) = ctx.core.player(t).hand.first() {
                        move_cards_to_hand(ctx, t, owner, &[c]);
                    }
                    Flow::Done
                } else if ask_code == ASK_TSPOWEI_CHOOSE_CARD {
                    if let Some(cid) = ctx.answer().iter().find_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    }) {
                        discard_cards(ctx, owner, &[cid]);
                        emit_damage(ctx, Some(owner), t, 1, DamageNature::Normal);
                    }
                    Flow::Done
                } else if ask_code == ASK_TSPOWEI_GAIN_CARD {
                    if let Some(cid) = ctx.answer().iter().find_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    }) {
                        move_cards_to_hand(ctx, t, owner, &[cid]);
                    }
                    Flow::Done
                } else {
                    Flow::Done
                }
            }
            _ => Flow::Done,
        },
        "qingyu" => match m.stage {
            HostStage::Act => match m.card {
                Some(1) => {
                    set_flag(ctx.core, owner, "qingyu_resolved", 1);
                    set_flag(ctx.core, owner, "qingyu_succeeded", 1);
                    grant_skill_named(ctx, owner, "xuancun");
                    set_flag(ctx.core, owner, "xuancun_granted", 1);
                    Flow::Done
                }
                Some(2) => {
                    set_flag(ctx.core, owner, "qingyu_resolved", 2);
                    set_flag(ctx.core, owner, "qingyu_failed", 1);
                    ctx.lose_max_hp(owner, 1);
                    Flow::Done
                }
                _ => wait_ask(m, ctx, ASK_QINGYU_DISCARD_TWO),
            },
            HostStage::SelectCard => {
                let chosen: Vec<_> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    })
                    .collect();
                if chosen.len() >= 2 {
                    ctx.current_damage_mut().prevented = true;
                    set_flag(ctx.core, owner, "qingyu_prevented", 1);
                    discard_cards(ctx, owner, &chosen[..2]);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "mibei" => {
            match m.card {
                Some(0) => {
                    set_flag(ctx.core, owner, "mibei_used_this_play", 0);
                    if flag(ctx.core, owner, "mibei_basic_cnt") == 0
                        && flag(ctx.core, owner, "mibei_trick_cnt") == 0
                        && flag(ctx.core, owner, "mibei_equip_cnt") == 0
                        && flag(ctx.core, owner, "xingqi_count") == 0
                    {
                        set_flag(ctx.core, owner, "mibei_mark", 1);
                    }
                }
                Some(1) => {
                    let used = flag(ctx.core, owner, "mibei_used_this_play") + 1;
                    set_flag(ctx.core, owner, "mibei_used_this_play", used);
                    let xq = flag(ctx.core, owner, "xingqi_count") + 1;
                    set_flag(ctx.core, owner, "xingqi_count", xq);
                    if let Some(&cid) = m.cards.first() {
                        if let Some(cd) = ctx.catalog.cards.get(cid as usize) {
                            set_flag(ctx.core, owner, &format!("xingqi_card:{}", cd.name), 1);
                        }
                    }
                    let k = m
                        .cards
                        .first()
                        .map(|&c| ctx.catalog.card_kind(c))
                        .unwrap_or(CardKind::Sha);
                    let key = if k.is_basic_kind() {
                        "mibei_basic_cnt"
                    } else if k.equip_slot().is_some() {
                        "mibei_equip_cnt"
                    } else {
                        "mibei_trick_cnt"
                    };
                    let cur = flag(ctx.core, owner, key) + 1;
                    set_flag(ctx.core, owner, key, cur);
                    if flag(ctx.core, owner, "mibei_basic_cnt") >= 2
                        && flag(ctx.core, owner, "mibei_trick_cnt") >= 2
                        && flag(ctx.core, owner, "mibei_equip_cnt") >= 2
                    {
                        set_flag(ctx.core, owner, "mibei_resolved", 1);
                        set_flag(ctx.core, owner, "mibei_succeeded", 1);
                        let mut drawn = Vec::new();
                        for want_type in [0u8, 1, 2] {
                            let matches_type = |c: CardId, cat: &crate::catalog::Catalog| {
                                let ck = cat.card_kind(c);
                                match want_type {
                                    0 => ck.is_basic_kind(),
                                    1 => ck.equip_slot().is_some(),
                                    _ => !ck.is_basic_kind() && ck.equip_slot().is_none(),
                                }
                            };
                            if let Some(pos) = ctx
                                .core
                                .deck
                                .iter()
                                .rposition(|&c| matches_type(c, ctx.catalog))
                            {
                                let c = ctx.core.deck.remove(pos);
                                ctx.core.player_mut(owner).hand.push(c);
                                drawn.push(c);
                            }
                        }
                        if !drawn.is_empty() {
                            ctx.emit(EventData::CardsDrawn {
                                player: owner,
                                cards: drawn,
                            });
                        }
                        grant_skill_named(ctx, owner, "xinmouli");
                        set_flag(ctx.core, owner, "xinmouli_granted", 1);
                    }
                }
                Some(4)
                    if flag(ctx.core, owner, "mibei_mark") > 0
                        && flag(ctx.core, owner, "mibei_basic_cnt") == 0
                        && flag(ctx.core, owner, "mibei_trick_cnt") == 0
                        && flag(ctx.core, owner, "mibei_equip_cnt") == 0
                        && flag(ctx.core, owner, "xingqi_count") == 0
                        && flag(ctx.core, owner, "mibei_resolved") == 0 =>
                {
                    set_flag(ctx.core, owner, "mibei_resolved", 2);
                    set_flag(ctx.core, owner, "mibei_failed", 1);
                    set_flag(ctx.core, owner, "mibei_mark", 0);
                    ctx.lose_max_hp(owner, 1);
                }
                _ => {}
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

const LINGREN_TEMP_SKILLS: [(&str, &str); 2] = [
    ("jianxiong", "xinfu_lingren_jianxiong"),
    ("xingshang", "xinfu_lingren_xingshang"),
];

/// 目录技能名可能是上游英文键或标准包中文名（`奸雄`/`行殇`）。
fn lingren_skill_def(catalog: &crate::catalog::Catalog, en: &str) -> Option<u16> {
    catalog.skill_by_name(en).or_else(|| match en {
        "jianxiong" => catalog.skill_by_name(crate::vocabulary::names::JIAN_XIONG),
        "xingshang" => catalog.skill_by_name("行殇"),
        _ => None,
    })
}

pub(super) fn handle_xinfu_lingren(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if m.card == Some(2) {
                set_flag(ctx.core, owner, "xinfu_lingren_dmg_bonus", 0);
                for p in 0..ctx.core.players.len() as PlayerId {
                    if flag(ctx.core, p, "xinfu_lingren_vuln_from") == u32::from(owner) + 1 {
                        set_flag(ctx.core, p, "xinfu_lingren_vuln_from", 0);
                        set_flag(ctx.core, p, "xinfu_lingren_vuln_card", 0);
                    }
                }
                refund_once_used(ctx, owner, "xinfu_lingren");
                return Flow::Done;
            }
            if m.card == Some(1) {
                set_flag(ctx.core, owner, "xinfu_lingren_used_turn", 0);
                set_flag(ctx.core, owner, "xinfu_lingren_dmg_bonus", 0);
                for (en, key) in LINGREN_TEMP_SKILLS {
                    if flag(ctx.core, owner, key) > 0 {
                        set_flag(ctx.core, owner, key, 0);
                        if let Some(def) = lingren_skill_def(ctx.catalog, en) {
                            ctx.core.skills.remove(&(owner, def));
                        }
                    }
                }
                for p in 0..ctx.core.players.len() as PlayerId {
                    if flag(ctx.core, p, "xinfu_lingren_vuln_from") == u32::from(owner) + 1 {
                        set_flag(ctx.core, p, "xinfu_lingren_vuln_from", 0);
                        set_flag(ctx.core, p, "xinfu_lingren_vuln_card", 0);
                    }
                }
                refund_once_used(ctx, owner, "xinfu_lingren");
                return Flow::Done;
            }
            // 新的一次用牌：先清除上一张牌残留的 +1 伤害标记。
            set_flag(ctx.core, owner, "xinfu_lingren_dmg_bonus", 0);
            for p in 0..ctx.core.players.len() as PlayerId {
                if flag(ctx.core, p, "xinfu_lingren_vuln_from") == u32::from(owner) + 1 {
                    set_flag(ctx.core, p, "xinfu_lingren_vuln_from", 0);
                    set_flag(ctx.core, p, "xinfu_lingren_vuln_card", 0);
                }
            }
            let fid = ctx.current_event_frame_id();
            let use_data = ctx.res.frame_mut(fid).and_then(|f| f.card_use().cloned());
            let eligible = use_data.is_some_and(|u| {
                flag(ctx.core, owner, "xinfu_lingren_used_turn") == 0
                    && !u.targets.is_empty()
                    && super::l20_batch_b_ext::is_lingren_damage_card(ctx.catalog, &u)
            });
            if !eligible {
                refund_once_used(ctx, owner, "xinfu_lingren");
                return Flow::Done;
            }
            let mut t_mask = 0u32;
            for &p in &m.targets {
                if p < 32 && ctx.core.player(p).alive {
                    t_mask |= 1u32 << p;
                }
            }
            set_flag(ctx.core, owner, "xinfu_lingren_targets_mask", t_mask);
            set_flag(
                ctx.core,
                owner,
                "xinfu_lingren_cur_card",
                m.cards.first().map(|&c| c + 1).unwrap_or(0),
            );
            wait_ask(m, ctx, ASK_XINFU_LINGREN_TARGET)
        }
        HostStage::SelectCard => {
            let ask_code = m.card.unwrap_or(0);
            let ans = ctx.answer();
            match ask_code {
                ASK_XINFU_LINGREN_TARGET => {
                    let chosen = ans.iter().find_map(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    });
                    let Some(t) = chosen else {
                        refund_once_used(ctx, owner, "xinfu_lingren");
                        return Flow::Done;
                    };
                    m.target = Some(t);
                    wait_ask(m, ctx, ASK_XINFU_LINGREN_GUESS_MASK)
                }
                ASK_XINFU_LINGREN_GUESS_MASK => {
                    let chosen = ans.iter().find_map(|o| match o {
                        ChoiceOption::Tag(tag) => Some(*tag),
                        _ => None,
                    });
                    let Some(mask) = chosen else {
                        refund_once_used(ctx, owner, "xinfu_lingren");
                        return Flow::Done;
                    };
                    set_flag(ctx.core, owner, "xinfu_lingren_used_turn", 1);
                    let t = m.target.unwrap_or(owner);
                    let mut has_mask = 0u8;
                    for &cid in &ctx.core.player(t).hand {
                        let k = ctx.catalog.card_kind(cid);
                        if k.is_basic_kind() {
                            has_mask |= 1;
                        } else if k.equip_slot().is_some() {
                            has_mask |= 4;
                        } else {
                            has_mask |= 2;
                        }
                    }
                    let reality_mask = (!has_mask) & 0b111;
                    let hits = (0..3)
                        .filter(|b| ((mask >> b) & 1) == ((reality_mask >> b) & 1))
                        .count() as u32;
                    set_flag(ctx.core, owner, "xinfu_lingren_hits", hits);
                    if hits >= 1 {
                        let cur_card = flag(ctx.core, owner, "xinfu_lingren_cur_card");
                        set_flag(ctx.core, owner, "xinfu_lingren_dmg_bonus", 1);
                        set_flag(ctx.core, t, "xinfu_lingren_vuln_from", u32::from(owner) + 1);
                        set_flag(ctx.core, t, "xinfu_lingren_vuln_card", cur_card);
                    }
                    if hits >= 2 {
                        draw_direct(ctx, owner, 2);
                    }
                    if hits >= 3 {
                        // 上游 `addTempSkill(['jianxiong','xingshang'], {player:'phaseBegin'})`：
                        // 仅记录本技能新授予的技能，下回合开始时只撤销这些。
                        for (en, key) in LINGREN_TEMP_SKILLS {
                            if let Some(def) = lingren_skill_def(ctx.catalog, en) {
                                if !ctx.core.has_skill(owner, def) {
                                    set_flag(ctx.core, owner, key, 1);
                                    grant_skill_named(ctx, owner, ctx.catalog.skills[def as usize].name);
                                }
                            }
                        }
                    }
                    refund_once_used(ctx, owner, "xinfu_lingren");
                    Flow::Done
                }
                _ => Flow::Done,
            }
        }
        _ => Flow::Done,
    }
}

fn advance_sbtieji_target(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if !m.targets.is_empty() {
        m.targets.remove(0);
    }
    if let Some(&next_t) = m.targets.first() {
        m.target = Some(next_t);
        return wait_ask(m, ctx, ASK_SBTIEJI_CONFIRM);
    }
    Flow::Done
}

pub(super) fn handle_sbtieji(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if m.card == Some(1) {
                for p in 0..ctx.core.players.len() as PlayerId {
                    set_flag(ctx.core, p, "sbtieji_target_sealed", 0);
                    set_flag(ctx.core, p, "sbtieji_sealed", 0);
                    set_flag(ctx.core, p, "sbtieji_direct_hit", 0);
                }
                return Flow::Done;
            }
            if m.targets.is_empty() {
                if let Some(t) = m.target {
                    m.targets.push(t);
                }
            }
            m.target = m.targets.first().copied();
            wait_ask(m, ctx, ASK_SBTIEJI_CONFIRM)
        }
        HostStage::SelectCard => {
            let ask_code = m.card.unwrap_or(0);
            let ans = ctx.answer();
            match ask_code {
                ASK_SBTIEJI_CONFIRM => {
                    if !ans.contains(&ChoiceOption::Confirm) {
                        return advance_sbtieji_target(m, ctx);
                    }
                    let t = m.target.unwrap_or(owner);
                    set_flag(ctx.core, owner, "sbtieji_sealed", 1);
                    set_flag(ctx.core, t, "sbtieji_sealed", 1);
                    set_flag(ctx.core, t, "sbtieji_target_sealed", 1);
                    set_flag(ctx.core, owner, "sbtieji_direct_hit", 1);
                    set_flag(ctx.core, t, "sbtieji_direct_hit", 1);
                    mark_sha_unrespondable(ctx, owner, t, m.cards.first().copied());
                    wait_ask(m, ctx, ASK_SBTIEJI_MOUYI_OWNER)
                }
                ASK_SBTIEJI_MOUYI_OWNER => {
                    let chosen = ans.iter().find_map(|o| match o {
                        ChoiceOption::Tag(tag) => Some(*tag),
                        _ => None,
                    });
                    let Some(tag) = chosen else {
                        return advance_sbtieji_target(m, ctx);
                    };
                    set_flag(ctx.core, owner, "sbtieji_owner_choice", u32::from(tag));
                    wait_ask(m, ctx, ASK_SBTIEJI_MOUYI_TARGET)
                }
                ASK_SBTIEJI_MOUYI_TARGET => {
                    let chosen = ans.iter().find_map(|o| match o {
                        ChoiceOption::Tag(tag) => Some(*tag),
                        _ => None,
                    });
                    let Some(target_tag) = chosen else {
                        return advance_sbtieji_target(m, ctx);
                    };
                    let owner_tag = flag(ctx.core, owner, "sbtieji_owner_choice") as u8;
                    let t = m.target.unwrap_or(owner);
                    if owner_tag == target_tag {
                        set_flag(ctx.core, owner, "sbtieji_won", 1);
                        if owner_tag == 0 {
                            if !hand_and_equips(ctx.core, t).is_empty() {
                                return wait_ask(m, ctx, ASK_SBTIEJI_TARGET_GIVE);
                            }
                        } else {
                            draw_direct(ctx, owner, 2);
                        }
                    } else {
                        set_flag(ctx.core, owner, "sbtieji_won", 0);
                    }
                    advance_sbtieji_target(m, ctx)
                }
                ASK_SBTIEJI_TARGET_GIVE => {
                    let t = m.target.unwrap_or(owner);
                    let chosen = ans.iter().find_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    });
                    if let Some(cid) = chosen {
                        move_cards_to_hand(ctx, t, owner, &[cid]);
                        set_flag(ctx.core, owner, "sbtieji_gained_card", 1);
                    }
                    advance_sbtieji_target(m, ctx)
                }
                _ => Flow::Done,
            }
        }
        _ => Flow::Done,
    }
}

pub(super) fn handle_sbduanliang(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, ASK_SBDUANLIANG_TARGET),
        HostStage::SelectCard => {
            let ask_code = m.card.unwrap_or(0);
            let ans = ctx.answer();
            match ask_code {
                ASK_SBDUANLIANG_TARGET => {
                    let chosen = ans.iter().find_map(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    });
                    let Some(t) = chosen else {
                        refund_once_used(ctx, owner, "sbduanliang");
                        return Flow::Done;
                    };
                    m.target = Some(t);
                    wait_ask(m, ctx, ASK_SBDUANLIANG_MOUYI_OWNER)
                }
                ASK_SBDUANLIANG_MOUYI_OWNER => {
                    let chosen = ans.iter().find_map(|o| match o {
                        ChoiceOption::Tag(tag) => Some(*tag),
                        _ => None,
                    });
                    let Some(tag) = chosen else {
                        return Flow::Done;
                    };
                    set_flag(ctx.core, owner, "sbduanliang_owner_choice", u32::from(tag));
                    wait_ask(m, ctx, ASK_SBDUANLIANG_MOUYI_TARGET)
                }
                ASK_SBDUANLIANG_MOUYI_TARGET => {
                    let chosen = ans.iter().find_map(|o| match o {
                        ChoiceOption::Tag(tag) => Some(*tag),
                        _ => None,
                    });
                    let Some(target_tag) = chosen else {
                        return Flow::Done;
                    };
                    let owner_tag = flag(ctx.core, owner, "sbduanliang_owner_choice") as u8;
                    let t = m.target.unwrap_or(owner);
                    if owner_tag == target_tag {
                        set_flag(ctx.core, owner, "sbduanliang_won", 1);
                        if owner_tag == 0 {
                            let has_bing = ctx.core.player(t).judge.iter().any(|&c| {
                                ctx.core
                                    .declared_kinds
                                    .get(&c)
                                    .copied()
                                    .unwrap_or_else(|| ctx.catalog.card_kind(c))
                                    == CardKind::Bing
                            });
                            if has_bing {
                                if !hand_and_equips(ctx.core, t).is_empty() {
                                    return wait_ask(m, ctx, ASK_SBDUANLIANG_GAIN_CARD);
                                }
                            } else if let Some(cid) = ctx.core.deck.pop() {
                                ctx.core.declared_kinds.insert(cid, CardKind::Bing);
                                ctx.core.player_mut(t).judge.push(cid);
                                set_flag(ctx.core, owner, "sbduanliang_bing_placed", 1);
                                set_flag(ctx.core, t, "sbduanliang_bing_placed", 1);
                                ctx.emit(EventData::CardUsed(CardUseData {
                                    card: None,
                                    player: owner,
                                    targets: vec![t],
                                    nature: DamageNature::Normal,
                                    use_as: Some(CardKind::Bing),
                                    cancelled: false,
                                }));
                            }
                        } else {
                            set_flag(ctx.core, owner, "sbduanliang_duel", 1);
                            set_flag(ctx.core, t, "sbduanliang_duel", 1);
                            ctx.emit(EventData::CardUsed(CardUseData {
                                card: None,
                                player: owner,
                                targets: vec![t],
                                nature: DamageNature::Normal,
                                use_as: Some(CardKind::Duel),
                                cancelled: false,
                            }));
                        }
                    } else {
                        set_flag(ctx.core, owner, "sbduanliang_won", 0);
                    }
                    Flow::Done
                }
                ASK_SBDUANLIANG_GAIN_CARD => {
                    let t = m.target.unwrap_or(owner);
                    let chosen = ans.iter().find_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    });
                    if let Some(cid) = chosen {
                        move_cards_to_hand(ctx, t, owner, &[cid]);
                        set_flag(ctx.core, owner, "sbduanliang_gained_card", 1);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            }
        }
        _ => Flow::Done,
    }
}

pub(super) fn handle_shouye(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if m.card == Some(1) {
                set_flag(ctx.core, owner, "shouye_used_turn", 0);
                refund_once_used(ctx, owner, "shouye");
                return Flow::Done;
            }
            wait_ask(m, ctx, ASK_SHOUYE_TARGETS)
        }
        HostStage::SelectCard => {
            let ask_code = m.card.unwrap_or(0);
            let ans = ctx.answer();
            match ask_code {
                ASK_SHOUYE_TARGETS => {
                    if !ans.contains(&ChoiceOption::Confirm) {
                        refund_once_used(ctx, owner, "shouye");
                        return Flow::Done;
                    }
                    set_flag(ctx.core, owner, "shouye_used_turn", 1);
                    wait_ask(m, ctx, ASK_SHOUYE_MOUYI_OWNER)
                }
                ASK_SHOUYE_MOUYI_OWNER => {
                    let chosen = ans.iter().find_map(|o| match o {
                        ChoiceOption::Tag(tag) => Some(*tag),
                        _ => None,
                    });
                    let Some(tag) = chosen else {
                        refund_once_used(ctx, owner, "shouye");
                        return Flow::Done;
                    };
                    set_flag(ctx.core, owner, "shouye_owner_choice", u32::from(tag));
                    wait_ask(m, ctx, ASK_SHOUYE_MOUYI_TARGET)
                }
                ASK_SHOUYE_MOUYI_TARGET => {
                    let chosen = ans.iter().find_map(|o| match o {
                        ChoiceOption::Tag(tag) => Some(*tag),
                        _ => None,
                    });
                    let Some(target_tag) = chosen else {
                        refund_once_used(ctx, owner, "shouye");
                        return Flow::Done;
                    };
                    let owner_tag = flag(ctx.core, owner, "shouye_owner_choice") as u8;
                    if owner_tag == target_tag {
                        set_flag(ctx.core, owner, "shouye_won", 1);
                        set_flag(ctx.core, owner, "shouye_nullified", 1);
                        let fid = ctx.current_event_frame_id();
                        if let Some(f) = ctx.res.frame_mut(fid) {
                            if let EventData::CardUsed(u) = &mut f.data {
                                u.targets.retain(|&p| p != owner);
                                if u.targets.is_empty() {
                                    u.cancelled = true;
                                }
                            }
                        }
                        if let Some(&cid) = m.cards.first() {
                            set_flag(ctx.core, owner, "shouye_pending_gain_card", cid + 1);
                        }
                    } else {
                        set_flag(ctx.core, owner, "shouye_won", 0);
                    }
                    refund_once_used(ctx, owner, "shouye");
                    Flow::Done
                }
                _ => Flow::Done,
            }
        }
        _ => Flow::Done,
    }
}

pub(super) fn handle_mbmowang(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.card {
        Some(3) => {
            if flag(ctx.core, owner, "mbdanggu_init") == 0
                && flag(ctx.core, owner, "mbdanggu_empty") == 0
            {
                set_flag(ctx.core, owner, "mbdanggu_init", 1);
                if flag(ctx.core, owner, "mbdanggu_count") == 0 {
                    set_flag(ctx.core, owner, "mbdanggu_count", 2);
                }
            }
            Flow::Done
        }
        Some(0) => {
            if flag(ctx.core, owner, "mbdanggu_init") == 0
                && flag(ctx.core, owner, "mbdanggu_empty") == 0
            {
                set_flag(ctx.core, owner, "mbdanggu_init", 1);
                if flag(ctx.core, owner, "mbdanggu_count") == 0 {
                    set_flag(ctx.core, owner, "mbdanggu_count", 2);
                }
            }
            if flag(ctx.core, owner, "mbmowang_used") > 0
                || flag(ctx.core, owner, "mbdanggu_empty") > 0
                || flag(ctx.core, owner, "mbdanggu_count") == 0
            {
                return Flow::Done;
            }
            set_flag(ctx.core, owner, "mbmowang_used", 1);
            set_flag(ctx.core, owner, "mbmowang_triggered", 1);
            // 先弃置区域内所有牌（休整标记会令该角色视为离场，弃牌原语随之短路）。
            let he: Vec<CardId> = hand_and_equips(ctx.core, owner);
            discard_cards(ctx, owner, &he);
            let judge = std::mem::take(&mut ctx.core.player_mut(owner).judge);
            for c in judge {
                ctx.core.declared_kinds.remove(&c);
                ctx.to_discard(c);
            }
            set_flag(ctx.core, owner, "mbmowang_resting", 1);
            let p = ctx.core.player_mut(owner);
            p.linked = false;
            p.face_down = false;
            p.dying = false;
            p.hp = 0;
            if ctx
                .res
                .stack
                .iter()
                .any(|f| matches!(f.data, EventData::Death { player, .. } if player == owner))
            {
                ctx.core.player_mut(owner).death_cancelled = true;
            }
            Flow::Done
        }
        Some(1) => {
            set_flag(ctx.core, owner, "mbmowang_resting", 0);
            set_flag(ctx.core, owner, "mbmowang_doomed", 1);
            let max_hp = ctx.core.player(owner).max_hp as i32;
            let cur_hp = ctx.core.player(owner).hp;
            if max_hp > cur_hp {
                ctx.recover(owner, (max_hp - cur_hp) as u32, None);
            }
            ctx.core.player_mut(owner).hp = max_hp;
            Flow::Done
        }
        Some(2) => {
            set_flag(ctx.core, owner, "mbmowang_doomed", 0);
            set_flag(ctx.core, owner, "mbmowang_died", 1);
            ctx.core.player_mut(owner).hp = 0;
            ctx.emit(EventData::Death {
                player: owner,
                killer: None,
            });
            Flow::Done
        }
        _ => Flow::Done,
    }
}
