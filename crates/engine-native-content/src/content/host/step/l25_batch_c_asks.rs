//! L25 `祖安武将` Batch C —— `ExpectedAsk` 重建与前半技能（郭嘉/曹操/吕布/黄忠）步进实现。

use super::l25_batch_c_ext::handle_l25_batch_c_ext;
use super::*;
use crate::catalog::{Catalog, Suit};
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::event::{DamageNature, EventData};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow, LogEvent};
use crate::snap::ExpectedAsk;
use crate::state::GameStateCore;

fn spec_ask(
    chooser: PlayerId,
    kind: ChoiceKind,
    prompt: &'static str,
    options: Vec<ChoiceOption>,
    min: usize,
    max: usize,
    allow_cancel: bool,
) -> Option<(PlayerId, ChoiceSpec)> {
    (!options.is_empty() && options.len() >= min).then_some((
        chooser,
        ChoiceSpec {
            kind,
            prompt,
            options,
            min,
            max,
            allow_cancel,
        },
    ))
}

fn confirm_ask(chooser: PlayerId, prompt: &'static str) -> Option<(PlayerId, ChoiceSpec)> {
    Some((
        chooser,
        ChoiceSpec {
            kind: ChoiceKind::Confirm,
            prompt,
            options: vec![
                ChoiceOption::Confirm,
                ChoiceOption::Pass,
                ChoiceOption::Tag(1),
                ChoiceOption::Tag(0),
            ],
            min: 1,
            max: 1,
            allow_cancel: true,
        },
    ))
}

fn tag_ask(
    chooser: PlayerId,
    prompt: &'static str,
    tags: impl IntoIterator<Item = u8>,
    allow_cancel: bool,
) -> Option<(PlayerId, ChoiceSpec)> {
    spec_ask(
        chooser,
        ChoiceKind::PickTag,
        prompt,
        tags.into_iter().map(ChoiceOption::Tag).collect(),
        1,
        1,
        allow_cancel,
    )
}

fn player_ask(
    chooser: PlayerId,
    prompt: &'static str,
    players: Vec<PlayerId>,
    min: usize,
    max: usize,
    allow_cancel: bool,
) -> Option<(PlayerId, ChoiceSpec)> {
    spec_ask(
        chooser,
        ChoiceKind::SelectPlayers,
        prompt,
        players.into_iter().map(ChoiceOption::Player).collect(),
        min,
        max,
        allow_cancel,
    )
}

fn card_ask(
    chooser: PlayerId,
    prompt: &'static str,
    cards: Vec<CardId>,
    min: usize,
    max: usize,
    allow_cancel: bool,
) -> Option<(PlayerId, ChoiceSpec)> {
    spec_ask(
        chooser,
        ChoiceKind::SelectCards,
        prompt,
        cards.into_iter().map(ChoiceOption::Card).collect(),
        min,
        max,
        allow_cancel,
    )
}

pub(super) fn is_confirmed(ctx: &Ctx<'_>) -> bool {
    matches!(
        ctx.answer().first().copied(),
        Some(ChoiceOption::Confirm) | Some(ChoiceOption::Tag(1))
    )
}

pub(super) fn build_ask(
    ask_id: u32,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    _card: Option<CardId>,
    _cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<(PlayerId, ChoiceSpec)> {
    let tgt = target.unwrap_or(owner);
    match ask_id {
        ASK_NEWYICE_CARD => card_ask(
            owner,
            "ssr_newyice:choose_card",
            core.player(owner).hand.clone(),
            1,
            1,
            true,
        ),
        ASK_NEWYICE_TARGET => player_ask(
            owner,
            "ssr_newyice:choose_target",
            alive_others(core, owner),
            1,
            1,
            false,
        ),
        ASK_NEWYICE_MODE => tag_ask(owner, "ssr_newyice:choose_mode", 0..=3, false),
        ASK_TUMENG_TARGET => {
            let mut cands = alive_others(core, owner);
            if cands.is_empty() {
                cands = alive_all(core, owner);
            }
            player_ask(owner, "ssr_tumeng:choose_target", cands, 1, 1, true)
        }
        ASK_ZHUJIU_DISCARD => card_ask(
            owner,
            "ssr_zhujiu:discard_card",
            black_he_cards(core, catalog, owner),
            1,
            1,
            true,
        ),
        ASK_ZHUJIU_TARGET => player_ask(
            owner,
            "ssr_zhujiu:choose_target",
            alive_all(core, owner),
            1,
            1,
            false,
        ),
        ASK_HUJIA_CONFIRM => confirm_ask(owner, "ssr_hujia:confirm"),
        ASK_HUJIA_GIVE => card_ask(
            owner,
            "ssr_hujia:give_card",
            he_cards(core, owner),
            1,
            1,
            false,
        ),
        ASK_LIEGONG_DISCARD => card_ask(
            owner,
            "ssr_liegong:discard_card",
            he_cards(core, owner),
            1,
            1,
            true,
        ),
        ASK_LIEGONG_EQUIP => {
            let eq_tgt = targets.first().copied().or(target).unwrap_or(owner);
            card_ask(
                owner,
                "ssr_liegong:choose_card",
                core.player(eq_tgt).equips.clone(),
                1,
                1,
                true,
            )
        }
        ASK_NEWGANGLIE_CONFIRM => confirm_ask(owner, "ssr_newganglie:confirm"),
        ASK_NEWGANGLIE_SHA => confirm_ask(owner, "ssr_newganglie:target_choice"),
        ASK_ZUNI_DISCARD => card_ask(
            owner,
            "ssr_zuni:discard_card",
            core.player(owner).hand.clone(),
            1,
            1,
            true,
        ),
        ASK_CHULI_TARGET => {
            let cands: Vec<PlayerId> = alive_all(core, owner)
                .into_iter()
                .filter(|&p| !core.player(p).hand.is_empty())
                .collect();
            player_ask(owner, "ssr_chuli:choose_target", cands, 1, 1, true)
        }
        ASK_CHULI_CARD => card_ask(
            owner,
            "ssr_chuli:choose_card",
            core.player(tgt).hand.clone(),
            1,
            1,
            false,
        ),
        ASK_XIANFANG_CONFIRM => confirm_ask(owner, "ssr_xianfang:confirm"),
        ASK_NEWZHENJUN_TARGET => player_ask(
            owner,
            "ssr_newzhenjun:choose_target",
            alive_others(core, owner),
            1,
            1,
            true,
        ),
        ASK_SHENDAO_DISCARD => card_ask(
            owner,
            "ssr_shendao:discard_cards",
            core.player(owner).hand.clone(),
            2,
            2,
            true,
        ),
        ASK_SHENDAO_TARGET => player_ask(
            owner,
            "ssr_shendao:choose_target",
            alive_all(core, owner),
            1,
            1,
            false,
        ),
        ASK_SHENDAO_OPTION => tag_ask(owner, "ssr_shendao:choose_option", 0..=2, false),
        ASK_NEWJIAHE_CONFIRM => confirm_ask(owner, "ssr_newjiahe:confirm"),
        ASK_MIEDI_TARGET => player_ask(
            owner,
            "ssr_miedi:choose_target",
            alive_others(core, owner),
            1,
            1,
            true,
        ),
        ASK_MIEDI_CHOICE => tag_ask(tgt, "ssr_miedi:target_choice", 0..=1, false),
        ASK_MIEDI_GIVE => {
            let cands = he_cards(core, tgt);
            let n = cands.len().min(2).max(1);
            card_ask(tgt, "ssr_miedi:target_give", cands, n, n, false)
        }
        ASK_NEWKUROU_TARGET => {
            let cands: Vec<PlayerId> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| !core.player(p).hand.is_empty())
                .collect();
            player_ask(owner, "ssr_newkurou:choose_target", cands, 1, 1, true)
        }
        ASK_NEWKUROU_CARD => card_ask(
            owner,
            "ssr_newkurou:choose_card",
            core.player(tgt).hand.clone(),
            1,
            1,
            false,
        ),
        _ => None,
    }
}

pub(super) fn wait_ask(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    stage: HostStage,
    ask_id: u32,
) -> Flow {
    m.card = Some(ask_id);
    set_flag(ctx.core, m.owner, "l25_c_ask_id", ask_id);
    if let Some((observer, spec)) = build_ask(
        ask_id,
        m.owner,
        m.target,
        &m.targets,
        m.card,
        &m.cards,
        ctx.core,
        ctx.catalog,
    ) {
        m.stage = stage;
        Flow::WaitChoice { observer, spec }
    } else {
        set_flag(ctx.core, m.owner, "l25_c_ask_id", 0);
        Flow::Done
    }
}

pub fn ask_l25_batch_c(
    stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    card: Option<CardId>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    if stage == HostStage::Act || stage == HostStage::DrawSettle || stage == HostStage::CardsAsk {
        return None;
    }
    let ask_id = card
        .filter(|&id| id > 0 && id < 40)
        .unwrap_or_else(|| flag(core, owner, "l25_c_ask_id"));
    let (observer, spec) =
        build_ask(ask_id, owner, target, targets, card, cards, core, catalog)?;
    Some(crate::snap::ask(
        observer,
        spec.prompt,
        spec.kind,
        spec.options,
        spec.min,
        spec.max,
        spec.allow_cancel,
    ))
}

fn refund_once_used(ctx: &mut Ctx<'_>, owner: PlayerId, name: &str) {
    if let Some(def) = ctx.catalog.skill_by_name(name) {
        if let Some(inst) = ctx.core.skills.get_mut(&(owner, def)) {
            inst.once_used = false;
        }
    }
}

pub(super) fn discard_single_he(ctx: &mut Ctx<'_>, p: PlayerId, cid: CardId) {
    if ctx.core.player(p).hand.contains(&cid) {
        ctx.discard_from_hand(p, cid);
    } else if ctx.core.player(p).equips.contains(&cid) {
        ctx.take_owned_card(p, cid);
        ctx.to_discard(cid);
        ctx.out.push(LogEvent::CardDiscarded {
            player: p,
            cards: vec![cid],
        });
    }
}

pub(super) fn give_single_he(ctx: &mut Ctx<'_>, from: PlayerId, to: PlayerId, cid: CardId) {
    if ctx.core.player(from).hand.contains(&cid) {
        ctx.give_hand_card(from, to, cid);
    } else if ctx.core.player(from).equips.contains(&cid) {
        ctx.take_owned_card(from, cid);
        ctx.core.player_mut(to).hand.push(cid);
        ctx.emit(EventData::CardsMoved {
            cards: vec![cid],
            from,
            to,
        });
        ctx.out.push(LogEvent::CardsGiven {
            from,
            to,
            cards: vec![cid],
        });
    }
}

pub(super) fn handle_l25_batch_c_impl(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage != HostStage::Act
        && m.stage != HostStage::DrawSettle
        && m.stage != HostStage::CardsAsk
        && ctx.res.answer.as_ref().is_some_and(|a| a.is_empty())
    {
        refund_once_used(ctx, owner, &m.def_key);
        set_flag(ctx.core, owner, "l25_c_ask_id", 0);
        return Flow::Done;
    }
    match m.def_key.as_str() {
        "ssr_bailun" => step_ssr_bailun(m, ctx),
        "ssr_newyice" => step_ssr_newyice(m, ctx),
        "ssr_tumeng" => step_ssr_tumeng(m, ctx),
        "ssr_zhujiu" => step_ssr_zhujiu(m, ctx),
        "ssr_hujia" => step_ssr_hujia(m, ctx),
        "ssr_shenfen" => step_ssr_shenfen(m, ctx),
        "ssr_kuangbao" => step_ssr_kuangbao(m, ctx),
        "ssr_wushuang" => step_ssr_wushuang(m, ctx),
        "ssr_jiwu" => step_ssr_jiwu(m, ctx),
        "ssr_liegong" => step_ssr_liegong(m, ctx),
        "ssr_pojia" => step_ssr_pojia(m, ctx),
        _ => handle_l25_batch_c_ext(m, ctx),
    }
}

fn step_ssr_bailun(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if m.stage != HostStage::Act {
        return Flow::Done;
    }
    match m.card {
        Some(105) => {
            set_flag(ctx.core, m.owner, "l25_bailun_dying", 1);
            set_flag(ctx.core, m.owner, "l25_bailun_needdie", 1);
            if ctx.core.player(m.owner).hp <= 0 {
                ctx.core.player_mut(m.owner).hp = 1;
                ctx.core.player_mut(m.owner).dying = false;
            }
            if ctx.core.player(m.owner).hand.is_empty() {
                ctx.start_draw(m.owner, 1);
                m.stage = HostStage::CardsAsk;
                return Flow::Yield;
            }
            Flow::Done
        }
        Some(106) | Some(107) => {
            if ctx.core.player(m.owner).hand.is_empty() {
                set_flag(ctx.core, m.owner, "l25_bailun_refill", 1);
                ctx.start_draw(m.owner, 1);
                m.stage = HostStage::CardsAsk;
                return Flow::Yield;
            }
            Flow::Done
        }
        Some(108) => {
            if ctx.core.deck.is_empty() && ctx.core.discard.is_empty() {
                set_flag(ctx.core, m.owner, "l25_bailun_trick_boost", 1);
                set_flag(ctx.core, m.owner, "ssr_bailun_direct", 1);
                return Flow::Done;
            }
            let jc = ctx.draw_judge_card();
            let suit = card_suit(ctx.catalog, jc);
            ctx.out.push(LogEvent::Judged {
                player: m.owner,
                card: jc,
            });
            if suit == Suit::Spade {
                ctx.core.player_mut(m.owner).hand.push(jc);
                ctx.out.push(LogEvent::CardsDrawn {
                    player: m.owner,
                    cards: vec![jc],
                });
                set_flag(ctx.core, m.owner, "l25_bailun_spade", 1);
                ctx.emit(EventData::LoseHp {
                    player: m.owner,
                    amount: 1,
                });
                m.stage = HostStage::CardsAsk;
                return Flow::Yield;
            } else {
                ctx.to_discard(jc);
                set_flag(ctx.core, m.owner, "l25_bailun_trick_boost", 1);
                set_flag(ctx.core, m.owner, "ssr_bailun_direct", 1);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_ssr_newyice(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => match m.card {
            Some(108) | Some(110) => {
                set_flag(ctx.core, m.owner, "l25_yice_mark", 1);
                set_flag(ctx.core, m.owner, "ssr_newyice_mark", 1);
                set_flag(ctx.core, m.owner, "l25_yice_extra_uses", 0);
                set_flag(ctx.core, m.owner, "l25_yice_used", 0);
                Flow::Done
            }
            Some(101) => {
                let extra = flag(ctx.core, m.owner, "l25_yice_extra_uses") + 1;
                set_flag(ctx.core, m.owner, "l25_yice_extra_uses", extra);
                set_flag(ctx.core, m.owner, "l25_yice_mark", 1 + extra);
                set_flag(ctx.core, m.owner, "ssr_newyice_mark", 1 + extra);
                Flow::Done
            }
            Some(v) if (200..=220).contains(&v) => {
                let amt = (v - 200).max(1);
                let extra = flag(ctx.core, m.owner, "l25_yice_extra_uses") + amt;
                set_flag(ctx.core, m.owner, "l25_yice_extra_uses", extra);
                set_flag(ctx.core, m.owner, "l25_yice_mark", 1 + extra);
                set_flag(ctx.core, m.owner, "ssr_newyice_mark", 1 + extra);
                Flow::Done
            }
            Some(102) => {
                if let Some(u_player) = m.target {
                    if flag(ctx.core, u_player, "l25_yice_jian") > 0 {
                        set_flag(ctx.core, u_player, "l25_yice_jian", 0);
                        set_flag(ctx.core, u_player, "ssr_newyice_trick", 0);
                        set_flag(ctx.core, u_player, "l25_yice_jian_drawn", 1);
                        ctx.start_draw(u_player, 1);
                        m.stage = HostStage::CardsAsk;
                        return Flow::Yield;
                    }
                }
                Flow::Done
            }
            _ => wait_ask(m, ctx, HostStage::SelectCard, ASK_NEWYICE_CARD),
        },
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() else {
                set_flag(ctx.core, m.owner, "l25_c_ask_id", 0);
                return Flow::Done;
            };
            m.cards = vec![cid];
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_NEWYICE_TARGET)
        }
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                set_flag(ctx.core, m.owner, "l25_c_ask_id", 0);
                return Flow::Done;
            };
            m.target = Some(tgt);
            wait_ask(m, ctx, HostStage::SelectNature, ASK_NEWYICE_MODE)
        }
        HostStage::SelectNature => {
            let opt = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(v)) => v,
                _ => {
                    set_flag(ctx.core, m.owner, "l25_c_ask_id", 0);
                    return Flow::Done;
                }
            };
            set_flag(ctx.core, m.owner, "l25_c_ask_id", 0);
            bump_used_this_turn(ctx.core, m.owner, "l25_yice_used");
            let Some(&cid) = m.cards.first() else {
                return Flow::Done;
            };
            let Some(tgt) = m.target else {
                return Flow::Done;
            };
            if ctx.core.player(m.owner).hand.contains(&cid) {
                ctx.give_hand_card(m.owner, tgt, cid);
            }
            let jian = flag(ctx.core, tgt, "l25_yice_jian") + 1;
            set_flag(ctx.core, tgt, "l25_yice_jian", jian);
            set_flag(ctx.core, tgt, "ssr_newyice_trick", jian);
            let mode_val = u32::from(opt) + 1;
            set_flag(ctx.core, m.owner, "l25_yice_mode", mode_val);
            set_flag(ctx.core, tgt, "l25_yice_mode", mode_val);
            match opt {
                0 => {
                    set_flag(ctx.core, m.owner, "l25_yice_chuqibuyi", 1);
                    ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                        Some(m.owner),
                        tgt,
                        1,
                        DamageNature::Normal,
                        None,
                    )));
                    m.stage = HostStage::CardsAsk;
                    Flow::Yield
                }
                1 => {
                    set_flag(ctx.core, m.owner, "l25_yice_dongzhuxianji", 1);
                    ctx.start_draw(m.owner, 2);
                    m.stage = HostStage::CardsAsk;
                    Flow::Yield
                }
                2 => {
                    set_flag(ctx.core, m.owner, "l25_yice_zhujinqiyuan", 1);
                    ctx.start_draw(m.owner, 1);
                    m.stage = HostStage::CardsAsk;
                    Flow::Yield
                }
                _ => {
                    set_flag(ctx.core, m.owner, "l25_yice_shengdong", 1);
                    ctx.start_draw(m.owner, 1);
                    m.stage = HostStage::CardsAsk;
                    Flow::Yield
                }
            }
        }
        _ => Flow::Done,
    }
}

fn step_ssr_tumeng(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => {
            let mut found_weapon = None;
            if let Some(pos) = ctx.core.deck.iter().position(|&c| is_weapon_card(ctx.catalog, c)) {
                found_weapon = Some(ctx.core.deck.remove(pos));
            } else if let Some(pos) = ctx
                .core
                .discard
                .iter()
                .position(|&c| is_weapon_card(ctx.catalog, c))
            {
                found_weapon = Some(ctx.core.discard.remove(pos));
            }
            if let Some(w_cid) = found_weapon {
                ctx.core.player_mut(m.owner).hand.push(w_cid);
                ctx.out.push(LogEvent::CardsDrawn {
                    player: m.owner,
                    cards: vec![w_cid],
                });
                set_flag(ctx.core, m.owner, "l25_tumeng_fetched", w_cid + 1);
            } else {
                let cur = ctx.core.turn.current;
                let v = flag(ctx.core, cur, "l25_tumeng_hand_minus1") + 1;
                set_flag(ctx.core, cur, "l25_tumeng_hand_minus1", v);
                set_flag(ctx.core, cur, "ssr_tumeng_card", v);
            }
            if m.card == Some(101) {
                if let Some(&dc) = m.cards.first() {
                    if let Some(pos) = ctx.core.discard.iter().position(|&x| x == dc) {
                        ctx.core.discard.remove(pos);
                        ctx.core.player_mut(m.owner).hand.push(dc);
                        ctx.out.push(LogEvent::CardsDrawn {
                            player: m.owner,
                            cards: vec![dc],
                        });
                        set_flag(ctx.core, m.owner, "l25_tumeng_gained_damage_card", 1);
                    }
                }
                let has_weapon_on_field = ctx.core.players.iter().any(|p| {
                    p.alive && p.equips.iter().any(|&c| is_weapon_card(ctx.catalog, c))
                });
                if has_weapon_on_field && !alive_others(ctx.core, m.owner).is_empty() {
                    return wait_ask(m, ctx, HostStage::TargetChoice, ASK_TUMENG_TARGET);
                }
            }
            Flow::Done
        }
        HostStage::TargetChoice => {
            set_flag(ctx.core, m.owner, "l25_c_ask_id", 0);
            if let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() {
                set_flag(ctx.core, m.owner, "l25_tumeng_backlash", 1);
                set_flag(ctx.core, tgt, "l25_tumeng_backlash", 1);
                if let Some(&c) = ctx.core.player(tgt).hand.first() {
                    if tgt != m.owner {
                        ctx.give_hand_card(tgt, m.owner, c);
                    }
                }
                ctx.start_draw(m.owner, 1);
                m.stage = HostStage::CardsAsk;
                return Flow::Yield;
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_ssr_zhujiu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_ZHUJIU_DISCARD),
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() else {
                set_flag(ctx.core, m.owner, "l25_c_ask_id", 0);
                refund_once_used(ctx, m.owner, "ssr_zhujiu");
                return Flow::Done;
            };
            discard_single_he(ctx, m.owner, cid);
            bump_used_this_turn(ctx.core, m.owner, "l25_zhujiu_used");
            set_flag(ctx.core, m.owner, "l25_zhujiu_used_flag", 1);
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_ZHUJIU_TARGET)
        }
        HostStage::TargetChoice => {
            set_flag(ctx.core, m.owner, "l25_c_ask_id", 0);
            if let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() {
                ctx.core.turn.jiu_active = true;
                set_flag(ctx.core, tgt, "l25_viewas_jiu", 1);
                set_flag(ctx.core, m.owner, "l25_viewas_jiu", 1);
                if ctx.core.player(tgt).hp <= 0 {
                    ctx.recover(tgt, 1, None);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_ssr_hujia(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectNature, ASK_HUJIA_CONFIRM),
        HostStage::SelectNature => {
            set_flag(ctx.core, m.owner, "l25_c_ask_id", 0);
            if !is_confirmed(ctx) {
                return Flow::Done;
            }
            ctx.start_draw(m.owner, 1);
            m.stage = HostStage::DrawSettle;
            Flow::Yield
        }
        HostStage::DrawSettle => {
            if he_cards(ctx.core, m.owner).is_empty() {
                set_flag(ctx.core, m.owner, "l25_hujia_feiying", 1);
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::SelectCard, ASK_HUJIA_GIVE)
        }
        HostStage::SelectCard => {
            set_flag(ctx.core, m.owner, "l25_c_ask_id", 0);
            if let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() {
                let recv = m
                    .target
                    .filter(|&t| t != m.owner && ctx.core.player(t).alive)
                    .or_else(|| alive_others(ctx.core, m.owner).first().copied())
                    .unwrap_or(m.owner);
                if recv != m.owner {
                    give_single_he(ctx, m.owner, recv, cid);
                    set_flag(ctx.core, recv, "l25_hujia_feiying", 1);
                }
                set_flag(ctx.core, m.owner, "l25_hujia_feiying", 1);
                let dying_p = m.target.unwrap_or(m.owner);
                if ctx.core.player(dying_p).hp <= 0 {
                    ctx.recover(dying_p, 1, None);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_ssr_shenfen(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if m.stage != HostStage::Act || flag(ctx.core, m.owner, "l25_shenfen_awakened") > 0 {
        return Flow::Done;
    }
    set_baonu_count(ctx.core, m.owner, 6);
    set_flag(ctx.core, m.owner, "l25_shenfen_awakened", 1);
    set_flag(ctx.core, m.owner, "l25_shenwei_active", 1);
    set_flag(ctx.core, m.owner, "ssr_newshenwei", 1);
    set_flag(ctx.core, m.owner, "l25_zhanshen_lianyu", 1);
    set_flag(ctx.core, m.owner, "l25_zhanshen_xiuluo", 1);
    set_flag(ctx.core, m.owner, "l25_zhanshen_mengyan", 1);
    set_flag(ctx.core, m.owner, "l25_zhanshen_xieshen", 1);
    let others = alive_others(ctx.core, m.owner);
    if others.is_empty() {
        return Flow::Done;
    }
    for other in others {
        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
            Some(m.owner),
            other,
            1,
            DamageNature::Normal,
            None,
        )));
    }
    m.stage = HostStage::CardsAsk;
    Flow::Yield
}

fn step_ssr_kuangbao(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if m.stage != HostStage::Act {
        return Flow::Done;
    }
    match m.card {
        Some(108) | Some(110) => {
            if flag(ctx.core, m.owner, "l25_kuangbao_init") == 0 {
                set_flag(ctx.core, m.owner, "l25_kuangbao_init", 1);
                let cur = baonu_count(ctx.core, m.owner);
                set_baonu_count(ctx.core, m.owner, cur + 2);
            }
            Flow::Done
        }
        Some(101) => {
            let cur = baonu_count(ctx.core, m.owner);
            set_baonu_count(ctx.core, m.owner, cur + 1);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_ssr_wushuang(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if m.stage != HostStage::Act {
        return Flow::Done;
    }
    set_flag(ctx.core, m.owner, "l25_wushuang_unrespondable", 1);
    set_flag(ctx.core, m.owner, "l25_wushuang_triggered", 1);
    for &t in &m.targets {
        set_flag(ctx.core, t, "l25_wushuang_target", 1);
    }
    Flow::Done
}

fn step_ssr_jiwu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if m.stage != HostStage::Act {
        return Flow::Done;
    }
    let cur = baonu_count(ctx.core, m.owner);
    if cur >= 3 {
        set_baonu_count(ctx.core, m.owner, cur - 3);
        ctx.core.turn.jiu_active = true;
        set_flag(ctx.core, m.owner, "l25_jiwu_boost", 1);
        set_flag(ctx.core, m.owner, "l25_viewas_jiu", 1);
    }
    Flow::Done
}

fn step_ssr_liegong(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => match m.card {
            Some(102) => wait_ask(m, ctx, HostStage::SelectCard, ASK_LIEGONG_DISCARD),
            Some(101) => {
                let Some(&dmg_tgt) = m.targets.first() else {
                    return Flow::Done;
                };
                m.target = Some(dmg_tgt);
                if ctx.core.player(dmg_tgt).equips.is_empty() {
                    return Flow::Done;
                }
                wait_ask(m, ctx, HostStage::JiefenTargetDiscard, ASK_LIEGONG_EQUIP)
            }
            Some(108) | Some(110) | Some(112) => {
                set_flag(ctx.core, m.owner, "l25_liegong_has_exile", 0);
                let all_p = alive_all(ctx.core, m.owner);
                for p in all_p {
                    let key = (p, "ssr_liegong".to_string());
                    if let Some(exiled) = ctx.core.vm_aux.piles.remove(&key) {
                        for cid in exiled {
                            ctx.core.player_mut(p).equips.push(cid);
                            crate::kernel::grant_equip_skills(ctx, p, cid);
                        }
                        set_flag(ctx.core, p, "l25_liegong_exiled", 0);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        HostStage::SelectCard => {
            set_flag(ctx.core, m.owner, "l25_c_ask_id", 0);
            if let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() {
                discard_single_he(ctx, m.owner, cid);
                set_flag(ctx.core, m.owner, "l25_liegong_unresp", 1);
                set_flag(ctx.core, m.owner, "l20_liegong_direct_hit", 1);
            }
            Flow::Done
        }
        HostStage::JiefenTargetDiscard => {
            set_flag(ctx.core, m.owner, "l25_c_ask_id", 0);
            if let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() {
                let eq_tgt = m.targets.first().copied().or(m.target).unwrap_or(m.owner);
                if let Some(pos) = ctx.core.player(eq_tgt).equips.iter().position(|&x| x == cid) {
                    ctx.core.player_mut(eq_tgt).equips.remove(pos);
                    crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, eq_tgt, cid);
                    ctx.core
                        .vm_aux
                        .piles
                        .entry((eq_tgt, "ssr_liegong".to_string()))
                        .or_default()
                        .push(cid);
                    set_flag(ctx.core, eq_tgt, "l25_liegong_exiled", 1);
                    set_flag(ctx.core, m.owner, "l25_liegong_has_exile", 1);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_ssr_pojia(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if m.stage != HostStage::Act {
        return Flow::Done;
    }
    set_flag(ctx.core, m.owner, "l25_pojia_ignore_armor", 1);
    for &t in &m.targets {
        set_flag(ctx.core, t, "l25_pojia_broken", 1);
    }
    Flow::Done
}
