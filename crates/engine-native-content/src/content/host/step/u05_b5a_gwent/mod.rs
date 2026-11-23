//! U05-5a (`杀海拾遗·gwent`) 完整上游牌面与卡牌伴生技（80 项）。
//!
//! 拆分为职责清晰的子模块：
//! - `gold_spells`: 12 张金法术与 `gw_dieyi` 蝶翼体系
//! - `silver_spells`: 12 张银法术与 `gw_butianshu` / `gw_zirankuizeng`
//! - `adventure_a` / `adventure_b`: `gw_xianzumaijiu` 与 20 张冒险金卡 + 特殊牌
//! - `skills`: `gw_aerdeyin`、`gw_baobaoshu`、`gw_nuhaifengbao`、`gw_youer`、
//!   `gw_dieyi`、`gw_leizhoushu`、`_gainspell`、`gw_xianzumaijiu`、`gwmaoxian_hengsaite_sha`
//! - `asks`: 快照恢复路径 `ChoiceSpec` 重建

use super::super::*;
use crate::catalog::Catalog;
use crate::event::Phase;
use crate::state::GameStateCore;

mod adventure_a;
mod adventure_b;
mod adventure_huoge;
mod asks;
mod gold_spells;
mod silver_spells;
mod skills;

pub(in crate::content::host::step) use adventure_huoge::emit_huoge_use;
pub(in crate::content::host::step) use silver_spells::{
    eligible_bronze_tags, eligible_bronze_targets,
};
pub(in crate::content::host) use silver_spells::apply_gw_baishuang_to_targets;
pub(in crate::content::host) use asks::{ask_u05_gwent_card, ask_u05_gwent_skill};
pub(in crate::content::host::step) use skills::{apply_buff, apply_debuff};
pub(in crate::content::host) use skills::{
    filter_u05_gwent_skill, filter_u05_gwent_skill_with_frame, handle_u05_gwent_skill,
    trigger_meta_u05_gwent_skill, u05_gwent_on_round_start,
};
pub(in crate::content::host::step) fn pull_gwent_or_vanished<F>(
    ctx: &mut Ctx<'_>,
    mut pred: F,
) -> Option<CardId>
where
    F: FnMut(CardId) -> bool,
{
    if let Some(c) = pull_card_matching(ctx, &mut pred) {
        return Some(c);
    }
    if let Some(pile) = ctx
        .core
        .vm_aux
        .piles
        .get_mut(&(0, "u05_vanished".to_string()))
    {
        if let Some(idx) = pile.iter().position(|c| pred(*c)) {
            return Some(pile.remove(idx));
        }
    }
    None
}

pub(in crate::content::host) const GOLD_SPELL_PICK_NAMES: &[&str] = &[
    "gw_hudiewu",
    "gw_yigeniyin",
    "gw_leizhoushu",
    "gw_aerdeyin",
    "gw_ansha",
    "gw_xinsheng",
    "gw_niuquzhijing",
    "gw_zhongmozhizhan",
    "gw_ganhan",
    "gw_tunshi",
    "gw_chongci",
];

pub(in crate::content::host) const SILVER_SPELL_PICK_NAMES: &[&str] = &[
    "gw_youer",
    "gw_tongdi",
    "gw_fuyuan",
    "gw_zhuoshao",
    "gw_butianshu",
    "gw_shizizhaohuan",
    "gw_zuihouyuanwang",
    "gw_zirankuizeng",
    "gw_nuhaifengbao",
    "gw_baishuang",
    "gw_baobaoshu",
    "gw_guaiwuchaoxue",
];

pub(in crate::content::host) const BRONZE_SPELL_NAMES: &[(&str, CardKind)] = &[
    ("gw_qinpendayu", CardKind::Qinpendayu),
    ("gw_birinongwu", CardKind::Birinongwu),
    ("gw_ciguhanshuang", CardKind::Ciguhanshuang),
    ("gw_baoxueyaoshui", CardKind::Baoxueyaoshui),
    ("gw_zhihuanjun", CardKind::Zhihuanjun),
    ("gw_zumoshoukao", CardKind::Zumoshoukao),
    ("gw_aozuzhilei", CardKind::Aozuzhilei),
    ("gw_poxiao", CardKind::Poxiao),
    ("gw_kunenfayin", CardKind::Kunenfayin),
    ("gw_yanziyaoshui", CardKind::Yanziyaoshui),
    ("gw_wenyi", CardKind::Wenyi),
    ("gw_shanbengshu", CardKind::Shanbengshu),
];

pub(in crate::content::host) const WEATHER_SKILL_NAMES: &[&str] = &[
    GW_QINPEN_STATE_KEY,
    "gw_qinpendayu",
    GW_BIRIN_STATE_KEY,
    "gw_birinongwu",
    GW_CIGU_STATE_KEY,
    "gw_ciguhanshuang",
    "gw_baobaoshu",
    "gw_nuhaifengbao",
];

pub(in crate::content::host::step) fn clean_gwent_name(name: &str) -> &str {
    name.rsplit_once('_')
        .and_then(|(head, tail)| {
            (!tail.is_empty() && tail.chars().all(|c| c.is_ascii_digit())).then_some(head)
        })
        .unwrap_or(name)
}

/// Gwent `card/gwent/card.js` & `character/gwent/card.js` 牌名统一分派入口。
pub(in crate::content::host) fn step_u05_gwent_card(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
) -> Flow {
    if m.stage == HostStage::HuanrenPick {
        return finalize_gwent_card_after_yield(m, ctx, true);
    }
    let Some(cname) = m
        .card
        .and_then(|c| ctx.catalog.cards.get(c as usize).map(|d| d.name))
    else {
        return Flow::Done;
    };
    let clean = clean_gwent_name(cname);
    if clean == "gw_dudayuanshuai1" {
        return step_dudayuanshuai1(m, ctx);
    }
    if matches!(
        clean,
        "gw_hudiewu"
            | "gw_yigeniyin"
            | "gw_leizhoushu"
            | "gw_aerdeyin"
            | "gw_ansha"
            | "gw_xinsheng"
            | "gw_niuquzhijing"
            | "gw_zhongmozhizhan"
            | "gw_ganhan"
            | "gw_huangjiashenpan"
            | "gw_tunshi"
            | "gw_chongci"
    ) {
        return gold_spells::step_gold_spell(m, ctx, clean);
    }
    if matches!(
        clean,
        "gw_youer"
            | "gw_tongdi"
            | "gw_zhuoshao"
            | "gw_butianshu"
            | "gw_shizizhaohuan"
            | "gw_zuihouyuanwang"
            | "gw_zirankuizeng"
            | "gw_nuhaifengbao"
            | "gw_baishuang"
            | "gw_baobaoshu"
            | "gw_guaiwuchaoxue"
    ) {
        return silver_spells::step_silver_spell(m, ctx, clean);
    }
    if matches!(
        clean,
        "gw_xianzumaijiu"
            | "gwmaoxian_yioufeisi"
            | "gwmaoxian_luoqi"
            | "gwmaoxian_jieluote"
            | "gwmaoxian_yenaifa"
            | "gwmaoxian_telisi"
            | "gwmaoxian_hengsaite"
            | "gwmaoxian_fuertaisite"
            | "gwmaoxian_laduoweide"
            | "gwmaoxian_enxier"
    ) {
        return adventure_a::step_adventure_a(m, ctx, clean);
    }
    if matches!(
        clean,
        "gwmaoxian_fulisi"
            | "gwmaoxian_kaerweite"
            | "gwmaoxian_bulanwang"
            | "gwmaoxian_kuite"
            | "gwmaoxian_haluo"
            | "gwmaoxian_dagong"
            | "gwmaoxian_gaier"
            | "gwmaoxian_airuiting"
            | "gwmaoxian_aisinie"
            | "gwmaoxian_falanxisika"
            | "gwmaoxian_huoge"
    ) {
        return adventure_b::step_adventure_b(m, ctx, clean);
    }
    // 兜底：清理载体牌
    finish_gwent_card(m, ctx, false)
}

/// 破晓（`card/gwent/card.js:1681`）：无天气/判定牌时直走分支 1；
/// 分支 0 令玩家选目标（[1..]）清天气+判定区；分支 1 随机获得 11 种铜法术之一。
pub(in crate::content::host) fn step_poxiao_card(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => {
            let any_eligible = ctx
                .core
                .alive_players()
                .any(|p| has_judge_or_weather(ctx.core, ctx.catalog, p));
            if !any_eligible {
                grant_random_bronze_spell(ctx, m.owner);
                if let Some(c) = m.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                return Flow::Done;
            }
            m.stage = HostStage::SelectCard;
            Flow::WaitChoice {
                observer: m.owner,
                spec: ChoiceSpec {
                    kind: ChoiceKind::PickTag,
                    prompt: GW_POXIAO_BRANCH,
                    options: vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            }
        }
        HostStage::SelectCard => {
            match ctx.answer_first() {
                ChoiceOption::Tag(0) => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .seat_order_from(m.owner)
                        .into_iter()
                        .filter(|&p| {
                            ctx.core.player(p).alive
                                && has_judge_or_weather(ctx.core, ctx.catalog, p)
                        })
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() {
                        if let Some(c) = m.card {
                            ctx.to_discard(c);
                            ctx.core.table.retain(|x| *x != c);
                        }
                        return Flow::Done;
                    }
                    let max = options.len();
                    m.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: m.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: GW_POXIAO_TARGETS,
                            options,
                            min: 1,
                            max,
                            allow_cancel: false,
                        },
                    }
                }
                _ => {
                    grant_random_bronze_spell(ctx, m.owner);
                    if let Some(c) = m.card {
                        ctx.to_discard(c);
                        ctx.core.table.retain(|x| *x != c);
                    }
                    Flow::Done
                }
            }
        }
        HostStage::TargetChoice => {
            let mut chosen: Vec<PlayerId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Player(p) => Some(*p),
                    _ => None,
                })
                .collect();
            let ordered: Vec<PlayerId> = ctx
                .core
                .seat_order_from(m.owner)
                .into_iter()
                .filter(|p| chosen.contains(p))
                .collect();
            chosen.clear();
            let weather_defs: Vec<_> = WEATHER_SKILL_NAMES
                .iter()
                .filter_map(|k| ctx.catalog.skill_by_name(k))
                .collect();
            for p in ordered {
                if !ctx.core.player(p).alive {
                    continue;
                }
                let judge: Vec<CardId> = ctx.core.player(p).judge.clone();
                if !judge.is_empty() {
                    ctx.core.player_mut(p).judge.clear();
                    for &c in &judge {
                        ctx.to_discard(c);
                        ctx.core
                            .vm_aux
                            .flags
                            .remove(&(p, format!("gw_dieyi_card:{c}")));
                    }
                    ctx.note_cards_lost(p, &judge);
                    ctx.emit(EventData::CardsLost {
                        player: p,
                        cards: judge.clone(),
                        origins: vec![crate::event::LostOrigin::Judge; judge.len()],
                    });
                    ctx.log(LogEvent::CardDiscarded {
                        player: p,
                        cards: judge,
                    });
                }
                for def in &weather_defs {
                    ctx.core.skills.remove(&(p, *def));
                    if ctx.catalog.skill_name(*def) == "gw_nuhaifengbao" {
                        ctx.core
                            .vm_aux
                            .flags
                            .remove(&(p, "gw_nuhaifengbao_rounds".to_string()));
                    }
                }
            }
            if let Some(c) = m.card {
                ctx.to_discard(c);
                ctx.core.table.retain(|x| *x != c);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn has_judge_or_weather(
    core: &GameStateCore,
    catalog: &Catalog,
    p: PlayerId,
) -> bool {
    !core.player(p).judge.is_empty()
        || WEATHER_SKILL_NAMES
            .iter()
            .filter_map(|k| catalog.skill_by_name(k))
            .any(|def| core.skills.contains_key(&(p, def)))
}

pub(super) fn grant_random_bronze_spell(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let is_bronze = |cid: CardId| -> bool {
        let name = clean_gwent_name(ctx.catalog.cards[cid as usize].name);
        if name == "gw_poxiao" || ctx.catalog.card_kind(cid) == CardKind::Poxiao {
            return false;
        }
        BRONZE_SPELL_NAMES
            .iter()
            .any(|&(bn, bk)| (bn == name || ctx.catalog.card_kind(cid) == bk) && bn != "gw_poxiao")
    };
    if let Some(c) = pull_card_matching(ctx, is_bronze) {
        ctx.core.player_mut(owner).hand.push(c);
        ctx.emit(EventData::CardsDrawn {
            player: owner,
            cards: vec![c],
        });
    } else {
        ctx.start_draw(owner, 1);
    }
}

pub(in crate::content::host::step) fn pull_card_matching<F>(ctx: &mut Ctx<'_>, mut pred: F) -> Option<CardId>
where
    F: FnMut(CardId) -> bool,
{
    let deck_idx: Vec<usize> = ctx
        .core
        .deck
        .iter()
        .enumerate()
        .filter(|(_, c)| pred(**c))
        .map(|(i, _)| i)
        .collect();
    if !deck_idx.is_empty() {
        let pick = deck_idx[ctx.core.rng.gen_range(deck_idx.len() as u32) as usize];
        return Some(ctx.core.deck.remove(pick));
    }
    let disc_idx: Vec<usize> = ctx
        .core
        .discard
        .iter()
        .enumerate()
        .filter(|(_, c)| pred(**c))
        .map(|(i, _)| i)
        .collect();
    if !disc_idx.is_empty() {
        let pick = disc_idx[ctx.core.rng.gen_range(disc_idx.len() as u32) as usize];
        return Some(ctx.core.discard.remove(pick));
    }
    None
}

/// 杜达元帅·小伙伴 (`character/gwent/card.js:951`): 若触发事件标有 `gw_dudayuanshuai1`
/// 则取消之，并获得该事件对应的牌（从处理区或弃牌堆取入手牌）。
fn step_dudayuanshuai1(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let cur_fid = ctx.current_frame_id();
    if let Some((idx, inc_fid)) = ctx.res.stack.iter().enumerate().rev().find_map(|(i, f)| {
        (f.id != cur_fid && matches!(f.data, EventData::CardUsed(_))).then_some((i, f.id))
    }) {
        let cancel_flag = ctx
            .core
            .vm_aux
            .flags
            .remove(&(m.owner, format!("gw_dudayuanshuai1_evt:{inc_fid}")))
            .is_some_and(|v| v > 0);
        let mut gain_card: Option<CardId> = None;
        if let Some(frame) = ctx.res.stack.get_mut(idx) {
            if let EventData::CardUsed(u) = &mut frame.data {
                if cancel_flag {
                    u.cancelled = true;
                }
                gain_card = u.card;
            }
        }
        if ctx.core.player(m.owner).alive {
            if let Some(c) = gain_card {
                let taken = if let Some(pos) = ctx.core.table.iter().position(|&x| x == c) {
                    ctx.core.table.remove(pos);
                    true
                } else if let Some(pos) = ctx.core.discard.iter().position(|&x| x == c) {
                    ctx.core.discard.remove(pos);
                    true
                } else {
                    false
                };
                if taken {
                    ctx.core.player_mut(m.owner).hand.push(c);
                    ctx.emit(EventData::CardsDrawn {
                        player: m.owner,
                        cards: vec![c],
                    });
                }
            }
        }
    }
    finish_gwent_card(m, ctx, false)
}

pub(super) fn finish_gwent_card(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    end_play_phase: bool,
) -> Flow {
    if end_play_phase {
        m.stage = HostStage::HuanrenPick;
        return Flow::Yield;
    }
    finalize_gwent_card_after_yield(m, ctx, false)
}

fn finalize_gwent_card_after_yield(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    end_play_phase: bool,
) -> Flow {
    if let Some(c) = m.card {
        ctx.core.table.retain(|x| *x != c);
        ctx.to_discard(c);
    }
    let cur_fid = ctx.current_frame_id();
    let in_enclosing_gwent_caster = ctx.res.stack.iter().any(|f| {
        f.id != cur_fid
            && matches!(
                &f.data,
                EventData::CardUsed(u)
                    if u.card.is_some_and(|cid| {
                        matches!(
                            clean_gwent_name(ctx.catalog.cards[cid as usize].name),
                            "gwmaoxian_huoge" | "gwmaoxian_falanxisika"
                        )
                    })
            )
    });
    if end_play_phase
        && !in_enclosing_gwent_caster
        && ctx.core.player(m.owner).alive
        && ctx.core.turn.phase == Phase::Play
        && ctx.core.turn.current == m.owner
    {
        ctx.emit(EventData::Phase {
            player: m.owner,
            phase: Phase::Play,
            begin: false,
        });
    }
    Flow::Done
}

pub(in crate::content::host) fn discard_he_card(ctx: &mut Ctx<'_>, player: PlayerId, c: CardId) {
    ctx.core.declared_slots.remove(&c);
    if ctx.core.player(player).hand.contains(&c) {
        ctx.discard_from_hand(player, c);
        return;
    }
    if ctx.core.player(player).equips.contains(&c) {
        ctx.core.player_mut(player).equips.retain(|x| *x != c);
        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, player, c);
        ctx.to_discard(c);
        ctx.note_cards_lost(player, &[c]);
        ctx.emit(EventData::CardsLost {
            player,
            cards: vec![c],
            origins: vec![crate::event::LostOrigin::Equip],
        });
        ctx.log(LogEvent::CardDiscarded {
            player,
            cards: vec![c],
        });
    }
}

pub(in crate::content::host) fn has_named_skill(
    core: &GameStateCore,
    catalog: &Catalog,
    p: PlayerId,
    name: &str,
) -> bool {
    catalog
        .skill_by_name(name)
        .is_some_and(|def| core.skills.contains_key(&(p, def)))
}

pub(in crate::content::host) fn enemies_of(core: &GameStateCore, catalog: &Catalog, owner: PlayerId) -> Vec<PlayerId> {
    core.seat_order_from(owner)
        .into_iter()
        .filter(|&p| {
            p != owner && core.player(p).alive && !core.is_friend_of(catalog, owner, p)
        })
        .collect()
}

pub(in crate::content::host) fn friends_of(core: &GameStateCore, catalog: &Catalog, owner: PlayerId) -> Vec<PlayerId> {
    core.seat_order_from(owner)
        .into_iter()
        .filter(|&p| p != owner && core.player(p).alive && core.is_friend_of(catalog, owner, p))
        .collect()
}
