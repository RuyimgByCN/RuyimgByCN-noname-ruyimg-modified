//! Batch U05-6a (`shahai-hearth` Part 1): `character/hearth/card.js` 44 张衍生卡
//! 与 `hsyaoshui` / `hsnitai_card` 结算实现（从 `u05_hearth_cards.rs` 职责拆分）。

use super::u05_hearth::{
    alive_others, change_max_hp, discard_card_from_zone, equip_card_to_player,
    finish_table_card_hearth, flag, gain_card_to_hand, grant_skill, has_any_skill,
    hearth_card_type_tag, is_hearth_enemy, make_damage, pick_random_hearth_derivable,
    remove_skill, rng_pick, rng_pick_mut, set_flag, spawn_card_by_base, spawn_card_by_kind,
};
use super::u05_hearth_asks::{
    set_active_ask, wait_hs, ASK_HS_BUWENDINGYIBIAN_RECAST, ASK_HS_GUHUO_GIVE,
    ASK_HS_MORIJINGXIANG_GAIN, ASK_HS_NITAI_SKILL, ASK_HS_NVWUDEGANGUO_DISCARD,
};
use super::*;
use crate::catalog::CardKind;
use crate::choice::ChoiceOption;
use crate::event::{DamageNature, EventData};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow, LogEvent};

pub(super) fn step_hearth_derived_card(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let key = m.def_key.as_str();
    let owner = m.owner;
    match key {
        "hs_card:hsfashu_buwendingyibian" => step_buwendingyibian(m, ctx),
        "hs_card:hsjixie_zhadan" => {
            let enemies: Vec<PlayerId> = ctx
                .core
                .alive_players()
                .filter(|&p| is_hearth_enemy(ctx.core, ctx.catalog, owner, p))
                .collect();
            if let Some(t) = rng_pick(ctx, &enemies) {
                ctx.emit(EventData::Damage(make_damage(
                    Some(owner),
                    t,
                    1,
                    DamageNature::Fire,
                    m.card,
                )));
            }
            if let Some(c) = m.card {
                finish_table_card_hearth(ctx, c, true);
            }
            Flow::Done
        }
        "hs_card:hsqizhou_feng" => step_qizhou_feng(m, ctx),
        "hs_card:hsqizhou_shui" => {
            let t = m.target.unwrap_or(owner);
            if ctx.core.player(t).alive {
                ctx.recover(t, 2, m.card);
            }
            if let Some(c) = m.card {
                finish_table_card_hearth(ctx, c, true);
            }
            Flow::Done
        }
        "hs_card:hsqizhou_huo" => {
            let t = m.target.unwrap_or(owner);
            if ctx.core.player(t).alive {
                ctx.emit(EventData::Damage(make_damage(
                    Some(owner),
                    t,
                    1,
                    DamageNature::Fire,
                    m.card,
                )));
            }
            if let Some(c) = m.card {
                finish_table_card_hearth(ctx, c, true);
            }
            Flow::Done
        }
        "hs_card:hsqizhou_tu" => {
            for t in m.targets.clone() {
                if ctx.core.player(t).alive {
                    ctx.change_hujia(t, 1);
                }
            }
            if let Some(c) = m.card {
                finish_table_card_hearth(ctx, c, true);
            }
            Flow::Done
        }
        "hs_card:hsqingyu_feibiao" => {
            if let Some(t) = m.target {
                if ctx.core.player(t).alive {
                    let mut dropped = Vec::new();
                    if let Some(hc) = rng_pick(ctx, &ctx.core.player(t).hand.clone()) {
                        discard_card_from_zone(ctx, t, hc);
                        dropped.push(hc);
                    }
                    if let Some(ec) = rng_pick(ctx, &ctx.core.player(t).equips.clone()) {
                        discard_card_from_zone(ctx, t, ec);
                        dropped.push(ec);
                    }
                    if !dropped.is_empty() {
                        ctx.log(LogEvent::CardDiscarded {
                            player: t,
                            cards: dropped,
                        });
                    }
                }
            }
            if let Some(c) = m.card {
                finish_table_card_hearth(ctx, c, true);
            }
            Flow::Done
        }
        "hs_card:hsqingyu_zhanfang" => {
            let t = m.target.unwrap_or(owner);
            if ctx.core.player(t).alive {
                change_max_hp(ctx, t, 1);
                ctx.start_draw(t, 1);
            }
            if let Some(c) = m.card {
                finish_table_card_hearth(ctx, c, true);
            }
            Flow::Done
        }
        "hs_card:hsqingyu_hufu" => {
            let t = m.target.unwrap_or(owner);
            if ctx.core.player(t).alive {
                ctx.change_hujia(t, 1);
            }
            if let Some(c) = m.card {
                finish_table_card_hearth(ctx, c, true);
            }
            Flow::Done
        }
        "hs_card:hsqingyu_shandian" => {
            let t = m.target.unwrap_or(owner);
            if ctx.core.player(t).alive {
                ctx.emit(EventData::Damage(make_damage(
                    Some(owner),
                    t,
                    1,
                    DamageNature::Thunder,
                    m.card,
                )));
            }
            if let Some(c) = m.card {
                finish_table_card_hearth(ctx, c, true);
            }
            Flow::Done
        }
        "hs_card:hsdusu_xueji" => {
            if let Some(t) = m.target {
                let mut eqs = ctx.core.player(t).equips.clone();
                let mut dropped = Vec::new();
                for _ in 0..2 {
                    if let Some(ec) = rng_pick_mut(ctx, &mut eqs) {
                        discard_card_from_zone(ctx, t, ec);
                        dropped.push(ec);
                    }
                }
                if !dropped.is_empty() {
                    ctx.log(LogEvent::CardDiscarded {
                        player: t,
                        cards: dropped,
                    });
                }
            }
            if let Some(c) = m.card {
                finish_table_card_hearth(ctx, c, true);
            }
            Flow::Done
        }
        "hs_card:hsdusu_kuyecao" => {
            let t = m.target.unwrap_or(owner);
            if ctx.core.player(t).alive {
                ctx.core.player_mut(t).yinshen_active = true;
            }
            if let Some(c) = m.card {
                finish_table_card_hearth(ctx, c, true);
            }
            Flow::Done
        }
        "hs_card:hsdusu_huangxuecao" => {
            let t = m.target.unwrap_or(owner);
            if ctx.core.player(t).alive {
                ctx.start_draw(t, 2);
            }
            if let Some(c) = m.card {
                finish_table_card_hearth(ctx, c, true);
            }
            Flow::Done
        }
        "hs_card:hsdusu_huoyanhua" => {
            let t = m.target.unwrap_or(owner);
            if ctx.core.player(t).alive {
                ctx.emit(EventData::Damage(make_damage(
                    Some(owner),
                    t,
                    1,
                    DamageNature::Fire,
                    m.card,
                )));
            }
            if let Some(c) = m.card {
                finish_table_card_hearth(ctx, c, true);
            }
            Flow::Done
        }
        "hs_card:hsdusu_shinancao" => {
            let t = m.target.unwrap_or(owner);
            if ctx.core.player(t).alive {
                grant_skill(ctx, t, "hsdusu_shinancao", false);
            }
            if let Some(c) = m.card {
                finish_table_card_hearth(ctx, c, true);
            }
            Flow::Done
        }
        "hs_card:hsbaowu_cangbaotu" => {
            let t = m.target.unwrap_or(owner);
            grant_skill(ctx, t, "hsbaowu_cangbaotu", false);
            ctx.start_draw(t, 1);
            if let Some(c) = m.card {
                finish_table_card_hearth(ctx, c, true);
            }
            Flow::Done
        }
        "hs_card:hsbaowu_huangjinyuanhou" => {
            let t = m.target.unwrap_or(owner);
            let hand = ctx.core.player(t).hand.clone();
            let lost = hand.len();
            for c in hand {
                ctx.core.player_mut(t).hand.retain(|&x| x != c);
                finish_table_card_hearth(ctx, c, true);
            }
            let need = 1 + lost;
            let mut gained = Vec::new();
            for _ in 0..need {
                if let Some(dc) = pick_random_hearth_derivable(ctx, 1).pop() {
                    ctx.core.detached_cards.retain(|&(x, _)| x != dc);
                    ctx.core.player_mut(t).hand.push(dc);
                    gained.push(dc);
                }
            }
            if !gained.is_empty() {
                ctx.log(LogEvent::CardsDrawn {
                    player: t,
                    cards: gained,
                });
            }
            ctx.core.player_mut(t).yinshen_active = true;
            grant_skill(ctx, t, "hsbaowu_huangjinyuanhou", false);
            grant_skill(ctx, t, "hsbaowu_huangjinyuanhou2", false);
            if let Some(c) = m.card {
                finish_table_card_hearth(ctx, c, true);
            }
            Flow::Done
        }
        "hs_card:hsshenqi_nengliangzhiguang" => {
            let t = m.target.unwrap_or(owner);
            change_max_hp(ctx, t, 1);
            ctx.recover(t, 1, None);
            ctx.start_draw(t, 4);
            if !ctx.core.player(owner).face_down {
                ctx.turn_over(owner);
            }
            if let Some(c) = m.card {
                finish_table_card_hearth(ctx, c, true);
            }
            Flow::Done
        }
        "hs_card:hsshenqi_kongbusangzhong" => {
            for t in alive_others(ctx.core, owner) {
                ctx.emit(EventData::Damage(make_damage(
                    Some(owner),
                    t,
                    1,
                    DamageNature::Normal,
                    m.card,
                )));
            }
            if !ctx.core.player(owner).face_down {
                ctx.turn_over(owner);
            }
            if let Some(c) = m.card {
                finish_table_card_hearth(ctx, c, true);
            }
            Flow::Done
        }
        "hs_card:hsshenqi_morijingxiang" => step_morijingxiang(m, ctx),
        "hs_card:hsmengjing_feicuiyoulong" => {
            let t = m.target.unwrap_or(owner);
            ctx.emit(EventData::Damage(make_damage(
                Some(owner),
                t,
                1,
                DamageNature::Normal,
                m.card,
            )));
            if let Some(c) = m.card {
                finish_table_card_hearth(ctx, c, true);
            }
            Flow::Done
        }
        "hs_card:hsmengjing_suxing" => {
            for t in alive_others(ctx.core, owner) {
                ctx.emit(EventData::LoseHp {
                    player: t,
                    amount: 1,
                });
                let mut he: Vec<CardId> = ctx
                    .core
                    .player(t)
                    .hand
                    .iter()
                    .chain(ctx.core.player(t).equips.iter())
                    .copied()
                    .collect();
                let mut dropped = Vec::new();
                for _ in 0..2 {
                    if let Some(dc) = rng_pick_mut(ctx, &mut he) {
                        discard_card_from_zone(ctx, t, dc);
                        dropped.push(dc);
                    }
                }
                if !dropped.is_empty() {
                    ctx.log(LogEvent::CardDiscarded {
                        player: t,
                        cards: dropped,
                    });
                }
            }
            if let Some(c) = m.card {
                finish_table_card_hearth(ctx, c, true);
            }
            Flow::Done
        }
        "hs_card:hsmengjing_mengye" => {
            let t = m.target.unwrap_or(owner);
            ctx.start_draw(t, 1);
            grant_skill(ctx, t, "hsmengjing_mengye", false);
            if let Some(c) = m.card {
                finish_table_card_hearth(ctx, c, true);
            }
            Flow::Done
        }
        "hs_card:hsmengjing_mengjing" => {
            let t = m.target.unwrap_or(owner);
            let eq = ctx.core.player(t).equips.clone();
            if !eq.is_empty() {
                for &ec in &eq {
                    ctx.core.player_mut(t).equips.retain(|&x| x != ec);
                    crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, t, ec);
                    ctx.core.player_mut(t).hand.push(ec);
                }
                ctx.log(LogEvent::CardsDrawn {
                    player: t,
                    cards: eq,
                });
            }
            let has_le = ctx
                .core
                .player(t)
                .judge
                .iter()
                .any(|&jc| ctx.catalog.card_kind(jc) == CardKind::Le);
            if !has_le {
                if let Some(le) = spawn_card_by_kind(ctx, CardKind::Le) {
                    ctx.core.player_mut(t).judge.push(le);
                }
            }
            if let Some(c) = m.card {
                finish_table_card_hearth(ctx, c, true);
            }
            Flow::Done
        }
        "hs_card:hsmengjing_huanxiaojiemei" => {
            let t = m.target.unwrap_or(owner);
            ctx.recover(t, 1, m.card);
            if let Some(c) = m.card {
                finish_table_card_hearth(ctx, c, true);
            }
            Flow::Done
        }
        "hs_card:hszuzhou_nvwudeganguo" => step_nvwudeganguo(m, ctx),
        "hs_card:hszuzhou_nvwudepingguo" => {
            let t = m.target.unwrap_or(owner);
            for _ in 0..2 {
                if let Some(sc) = spawn_card_by_kind(ctx, CardKind::Sha) {
                    gain_card_to_hand(ctx, t, sc);
                }
            }
            if let Some(c) = m.card {
                finish_table_card_hearth(ctx, c, true);
            }
            Flow::Done
        }
        "hs_card:hszuzhou_nvwudexuetu" => {
            let t = m.target.unwrap_or(owner);
            if !has_any_skill(ctx.core, ctx.catalog, t, "zhoujiang") {
                set_flag(ctx.core, t, "hs_fengyin_xuetu", 1);
                grant_skill(ctx, t, "zhoujiang", false);
                set_flag(ctx.core, t, "hs_xuetu_clear_phase_after", 1);
            }
            if let Some(c) = m.card {
                finish_table_card_hearth(ctx, c, true);
            }
            Flow::Done
        }
        "hs_card:hszuzhou_wushushike" => {
            for t in ctx.core.seat_order_from(owner) {
                let shans: Vec<CardId> = ctx
                    .core
                    .player(t)
                    .hand
                    .iter()
                    .copied()
                    .filter(|&cid| ctx.catalog.card_kind(cid) == CardKind::Shan)
                    .collect();
                let n = shans.len();
                for cid in shans {
                    ctx.core.player_mut(t).hand.retain(|&x| x != cid);
                    finish_table_card_hearth(ctx, cid, false);
                }
                for _ in 0..n {
                    if let Some(sha_c) = spawn_card_by_kind(ctx, CardKind::Sha) {
                        gain_card_to_hand(ctx, t, sha_c);
                    }
                }
            }
            if let Some(c) = m.card {
                finish_table_card_hearth(ctx, c, true);
            }
            Flow::Done
        }
        "hs_card:hszuzhou_guhuo" => step_guhuo(m, ctx),
        "hs_card:hsyaoshui" => step_hsyaoshui(m, ctx),
        "hs_card:hsnitai_card" => step_hsnitai_card(m, ctx),
        _ => Flow::Done,
    }
}

fn step_buwendingyibian(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let t = m.target.unwrap_or(m.owner);
    match m.stage {
        HostStage::Act => {
            let used = flag(ctx.core, m.owner, "hs_buwendingyibian_used_turn") + 1;
            set_flag(ctx.core, m.owner, "hs_buwendingyibian_used_turn", used);
            if ctx.core.player(t).hand.is_empty() {
                if let Some(c) = m.card {
                    finish_table_card_hearth(ctx, c, true);
                }
                return Flow::Done;
            }
            wait_hs(m, ctx, HostStage::SelectCard, ASK_HS_BUWENDINGYIBIAN_RECAST)
        }
        HostStage::SelectCard => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::SelectCard, 0);
            if let Some(ChoiceOption::Card(picked)) = ctx.answer().first().copied() {
                let wanted_type = hearth_card_type_tag(ctx.catalog, picked);
                let old_name = ctx.catalog.cards[picked as usize].name;
                ctx.take_hand_card(t, picked);
                finish_table_card_hearth(ctx, picked, false);
                let mut found = ctx
                    .core
                    .deck
                    .iter()
                    .rposition(|&c| {
                        hearth_card_type_tag(ctx.catalog, c) == wanted_type
                            && ctx.catalog.cards[c as usize].name != old_name
                    })
                    .or_else(|| {
                        ctx.core
                            .deck
                            .iter()
                            .rposition(|&c| hearth_card_type_tag(ctx.catalog, c) == wanted_type)
                    })
                    .map(|idx| ctx.core.deck.remove(idx));
                if found.is_none() {
                    found = ctx.core.deck.pop();
                }
                if let Some(g) = found {
                    gain_card_to_hand(ctx, t, g);
                }
                if let Some(clone) = spawn_card_by_base(ctx, "hsfashu_buwendingyibian") {
                    set_flag(
                        ctx.core,
                        m.owner,
                        &format!("hs_buwendingyibian_destroy:{clone}"),
                        1,
                    );
                    grant_skill(ctx, m.owner, "buwendingyibian_lose", false);
                    gain_card_to_hand(ctx, m.owner, clone);
                }
            }
            if let Some(c) = m.card {
                finish_table_card_hearth(ctx, c, true);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_qizhou_feng(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let enemies: Vec<PlayerId> = ctx
        .core
        .seat_order_from(m.owner)
        .into_iter()
        .filter(|&p| is_hearth_enemy(ctx.core, ctx.catalog, m.owner, p))
        .collect();
    if let Some(c) = m.card {
        finish_table_card_hearth(ctx, c, true);
    }
    if !enemies.is_empty() {
        super::u05_hearth::emit_hearth_virtual(
            ctx,
            m.owner,
            CardKind::ArcheryAttack,
            enemies,
        );
    }
    Flow::Done
}

fn step_morijingxiang(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    loop {
        match m.stage {
            HostStage::Act => {
                if m.cards.is_empty() && flag(ctx.core, m.owner, "hs_mori_init") == 0 {
                    set_flag(ctx.core, m.owner, "hs_mori_init", 1);
                    m.targets = alive_others(ctx.core, m.owner)
                        .into_iter()
                        .filter(|&p| {
                            let pl = ctx.core.player(p);
                            !pl.hand.is_empty() || !pl.equips.is_empty() || !pl.judge.is_empty()
                        })
                        .collect();
                }
                let Some(t) = m.targets.first().copied() else {
                    set_flag(ctx.core, m.owner, "hs_mori_init", 0);
                    if !ctx.core.player(m.owner).face_down {
                        ctx.turn_over(m.owner);
                    }
                    if let Some(c) = m.card {
                        finish_table_card_hearth(ctx, c, true);
                    }
                    return Flow::Done;
                };
                m.targets.remove(0);
                let pl = ctx.core.player(t);
                if !pl.alive || (pl.hand.is_empty() && pl.equips.is_empty() && pl.judge.is_empty())
                {
                    continue;
                }
                m.target = Some(t);
                return wait_hs(m, ctx, HostStage::SelectCard, ASK_HS_MORIJINGXIANG_GAIN);
            }
            HostStage::SelectCard => {
                set_active_ask(ctx.core, m.behavior, m.owner, HostStage::SelectCard, 0);
                if let (Some(t), Some(ChoiceOption::Card(picked))) =
                    (m.target, ctx.answer().first().copied())
                {
                    if ctx.core.player(t).hand.contains(&picked) {
                        ctx.take_hand_card(t, picked);
                    } else if ctx.core.player(t).equips.contains(&picked) {
                        ctx.core.player_mut(t).equips.retain(|&x| x != picked);
                        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, t, picked);
                    } else {
                        ctx.core.player_mut(t).judge.retain(|&x| x != picked);
                    }
                    gain_card_to_hand(ctx, m.owner, picked);
                }
                m.stage = HostStage::Act;
            }
            _ => return Flow::Done,
        }
    }
}

fn step_nvwudeganguo(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let Some(t) = m.target else {
        if let Some(c) = m.card {
            finish_table_card_hearth(ctx, c, true);
        }
        return Flow::Done;
    };
    match m.stage {
        HostStage::Act => {
            let pl = ctx.core.player(t);
            if pl.hand.is_empty() && pl.equips.is_empty() {
                if let Some(c) = m.card {
                    finish_table_card_hearth(ctx, c, true);
                }
                return Flow::Done;
            }
            wait_hs(m, ctx, HostStage::SelectCard, ASK_HS_NVWUDEGANGUO_DISCARD)
        }
        HostStage::SelectCard => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::SelectCard, 0);
            if let Some(ChoiceOption::Card(picked)) = ctx.answer().first().copied() {
                discard_card_from_zone(ctx, t, picked);
                ctx.log(LogEvent::CardDiscarded {
                    player: t,
                    cards: vec![picked],
                });
            }
            if let Some(g) = pick_random_hearth_derivable(ctx, 1).pop() {
                ctx.core.detached_cards.retain(|&(x, _)| x != g);
                gain_card_to_hand(ctx, t, g);
            } else {
                ctx.start_draw(t, 1);
            }
            if let Some(c) = m.card {
                finish_table_card_hearth(ctx, c, true);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_guhuo(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let Some(t) = m.target else {
        if let Some(c) = m.card {
            finish_table_card_hearth(ctx, c, true);
        }
        return Flow::Done;
    };
    match m.stage {
        HostStage::Act => {
            let pl = ctx.core.player(t);
            if pl.hand.is_empty() && pl.equips.is_empty() {
                if let Some(c) = m.card {
                    finish_table_card_hearth(ctx, c, true);
                }
                return Flow::Done;
            }
            wait_hs(m, ctx, HostStage::SelectCard, ASK_HS_GUHUO_GIVE)
        }
        HostStage::SelectCard => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::SelectCard, 0);
            if let Some(ChoiceOption::Card(picked)) = ctx.answer().first().copied() {
                if ctx.core.player(t).hand.contains(&picked) {
                    ctx.take_hand_card(t, picked);
                } else if ctx.core.player(t).equips.contains(&picked) {
                    ctx.core.player_mut(t).equips.retain(|&x| x != picked);
                    crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, t, picked);
                }
                gain_card_to_hand(ctx, m.owner, picked);
            }
            if let Some(c) = m.card {
                finish_table_card_hearth(ctx, c, true);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_hsyaoshui(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let Some(cid) = m.card else {
        return Flow::Done;
    };
    let effs: Vec<u32> = (0..3)
        .filter_map(|i| {
            let v = flag(ctx.core, 0, &format!("hs_yaoshui_eff_{i}:{cid}"));
            (v > 0).then_some(v)
        })
        .collect();
    let targets = m.targets.clone();
    for code in effs {
        if let Ok(kind) = crate::archive::kind_from_code(code as u16) {
            if kind.is_equip() {
                if let (Some(&t), Some(ec)) = (targets.first(), spawn_card_by_kind(ctx, kind)) {
                    equip_card_to_player(ctx, t, ec);
                }
            } else {
                let use_targets = if matches!(
                    kind,
                    CardKind::SavageAttack | CardKind::ArcheryAttack | CardKind::Taoyuan | CardKind::Wugu
                ) {
                    alive_others(ctx.core, m.owner)
                } else {
                    targets.clone()
                };
                ctx.use_virtual_card(m.owner, kind, use_targets);
            }
        }
    }
    finish_table_card_hearth(ctx, cid, true);
    Flow::Done
}

fn step_hsnitai_card(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let Some(cid) = m.card else {
        return Flow::Done;
    };
    let t = m.target.unwrap_or(m.owner);
    match m.stage {
        HostStage::Act => {
            let cnt = flag(ctx.core, 0, &format!("hs_nitai_cnt:{cid}"));
            if cnt == 0 {
                finish_table_card_hearth(ctx, cid, true);
                return Flow::Done;
            }
            set_flag(ctx.core, m.owner, "hs_choice_count", cnt);
            if cnt == 1 {
                let sk_def = flag(ctx.core, 0, &format!("hs_nitai_sk_0:{cid}"));
                apply_nitai_skill(ctx, t, sk_def);
                finish_table_card_hearth(ctx, cid, true);
                return Flow::Done;
            }
            wait_hs(m, ctx, HostStage::SelectCard, ASK_HS_NITAI_SKILL)
        }
        HostStage::SelectCard => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::SelectCard, 0);
            let idx = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(i)) => i,
                _ => 0,
            };
            let sk_def = flag(ctx.core, 0, &format!("hs_nitai_sk_{idx}:{cid}"));
            apply_nitai_skill(ctx, t, sk_def);
            finish_table_card_hearth(ctx, cid, true);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn apply_nitai_skill(ctx: &mut Ctx<'_>, t: PlayerId, sk_def: u32) {
    if sk_def == 0 {
        return;
    }
    let old = flag(ctx.core, t, "hs_nitai_active_def");
    if old > 0 {
        let old_name = ctx.catalog.skills[(old - 1) as usize].name.to_string();
        remove_skill(ctx, t, &old_name);
    }
    if let Some(def) = ctx.catalog.skills.get((sk_def - 1) as usize) {
        let name = def.name.to_string();
        grant_skill(ctx, t, &name, false);
        set_flag(ctx.core, t, "hs_nitai_active_def", sk_def);
        set_flag(ctx.core, t, "hs_nitai_turns_left", 2);
    }
}
