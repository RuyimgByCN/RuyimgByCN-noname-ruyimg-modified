//! U08 英雄杀 Batch A 后半部分步骤状态机与询问构造（`u08_batch_a_asks.rs`）。

use super::super::*;
use super::u08_batch_a::*;
use crate::catalog::{CardKind, Catalog, EquipSlot, Suit};
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::event::{CardUseData, DamageNature, EventData, LostOrigin};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow};
use crate::snap::{ask, ExpectedAsk};
use crate::state::GameStateCore;

pub(super) fn wait_ask(
    m: &mut HostMachine,
    ctx: &Ctx<'_>,
    next_stage: HostStage,
    ask_id: u32,
) -> Flow {
    m.stage = next_stage;
    m.card = Some(ask_id);
    if let Some((observer, spec)) = build_spec(
        m.stage,
        m.owner,
        m.target,
        &m.targets,
        m.card,
        &m.cards,
        ctx.core,
        ctx.catalog,
    ) {
        Flow::WaitChoice { observer, spec }
    } else {
        m.stage = HostStage::Act;
        Flow::Done
    }
}

pub(super) fn handle_u08_batch_a_ext(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.def_key.as_str() {
        "huli" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_HULI_CARD),
            HostStage::SelectCard => {
                let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() else {
                    m.stage = HostStage::Act;
                    return Flow::Done;
                };
                m.cards = vec![cid];
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_HULI_TARGET)
            }
            HostStage::TargetChoice => {
                m.stage = HostStage::Act;
                if let (Some(&cid), Some(ChoiceOption::Player(t))) =
                    (m.cards.first(), ctx.answer().first().copied())
                {
                    discard_cards(ctx, owner, &[cid]);
                    if ctx.core.player(t).alive
                        && ctx.core.player(t).hp < ctx.core.player(t).max_hp as i32
                    {
                        ctx.recover(t, 1, Some(cid));
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "yixin" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_YIXIN_DISCARD_TWO),
            HostStage::SelectCard => {
                let cards: Vec<CardId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    })
                    .collect();
                if cards.len() != 2 {
                    refund_once_used(ctx, owner, "yixin");
                    m.stage = HostStage::Act;
                    return Flow::Done;
                }
                m.cards = cards;
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_YIXIN_TARGET)
            }
            HostStage::TargetChoice => {
                m.stage = HostStage::Act;
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    refund_once_used(ctx, owner, "yixin");
                    return Flow::Done;
                };
                set_flag(ctx.core, owner, "yixin_used", 1);
                let chosen = m.cards.clone();
                discard_cards(ctx, owner, &chosen);
                if ctx.core.player(t).alive {
                    let lost =
                        (ctx.core.player(t).max_hp as i32 - ctx.core.player(t).hp).max(0) as u32;
                    let num = lost.min(4);
                    if num > 0 {
                        ctx.recover(t, num, None);
                    }
                    if num < 4 {
                        draw_direct(ctx, t, 4 - num);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "zbudao" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_ZBUDAO_CONFIRM),
            HostStage::SelectCard => {
                m.stage = HostStage::Act;
                if ctx.answer().first() == Some(&ChoiceOption::Confirm) {
                    draw_direct(ctx, owner, 1);
                    set_flag(ctx.core, owner, "zbudao2_active", 1);
                    grant_skill_named(ctx, owner, "zbudao2");
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "zbudao2" => match m.stage {
            HostStage::Act => {
                set_flag(ctx.core, owner, "zbudao2_active", 0);
                wait_ask(m, ctx, HostStage::SelectCard, ASK_ZBUDAO2_CARD)
            }
            HostStage::SelectCard => {
                let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() else {
                    m.stage = HostStage::Act;
                    return Flow::Done;
                };
                m.cards = vec![cid];
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_ZBUDAO2_TARGET)
            }
            HostStage::TargetChoice => {
                m.stage = HostStage::Act;
                if let (Some(&cid), Some(ChoiceOption::Player(t))) =
                    (m.cards.first(), ctx.answer().first().copied())
                {
                    move_cards_to_hand(ctx, owner, t, &[cid]);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "taiji" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_TAIJI_CHOOSE_SHA),
            HostStage::SelectCard => {
                let Some(ChoiceOption::Card(sha_cid)) = ctx.answer().first().copied() else {
                    m.stage = HostStage::Act;
                    return Flow::Done;
                };
                m.cards = vec![sha_cid];
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_TAIJI_CHOOSE_TARGET)
            }
            HostStage::TargetChoice => {
                m.stage = HostStage::Act;
                let (Some(&sha_cid), Some(ChoiceOption::Player(t))) =
                    (m.cards.first(), ctx.answer().first().copied())
                else {
                    return Flow::Done;
                };
                if let Some(pos) = ctx.core.player(owner).hand.iter().position(|&c| c == sha_cid) {
                    ctx.core.player_mut(owner).hand.remove(pos);
                    ctx.note_cards_lost(owner, &[sha_cid]);
                    ctx.core.table.push(sha_cid);
                    ctx.emit(EventData::CardsLost {
                        player: owner,
                        cards: vec![sha_cid],
                        origins: vec![LostOrigin::Hand],
                    });
                    set_flag(ctx.core, owner, "vm_va_sha_no_count", 1);
                    let kind = ctx.catalog.card_kind(sha_cid);
                    let nature = match kind {
                        CardKind::FireSha => DamageNature::Fire,
                        CardKind::LeiSha => DamageNature::Thunder,
                        _ => DamageNature::Normal,
                    };
                    ctx.emit(EventData::CardUsed(CardUseData {
                        card: Some(sha_cid),
                        player: owner,
                        targets: vec![t],
                        nature,
                        use_as: Some(kind),
                        cancelled: false,
                    }));
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "fengliu" => {
            let n = count_alive_females(ctx.core, ctx.catalog).min(2);
            if n > 0 {
                draw_direct(ctx, owner, n);
            }
            Flow::Done
        }
        "luobi" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectNature, ASK_LUOBI_CONFIRM),
            HostStage::SelectNature => {
                if ctx.answer().first() != Some(&ChoiceOption::Confirm) {
                    m.stage = HostStage::Act;
                    return Flow::Done;
                }
                let lost =
                    (ctx.core.player(owner).max_hp as i32 - ctx.core.player(owner).hp).max(0) as u32;
                let drawn = draw_direct(ctx, owner, lost);
                m.cards = drawn;
                if m.cards
                    .iter()
                    .any(|&c| ctx.core.player(owner).hand.contains(&c))
                    && !alive_others(ctx.core, owner).is_empty()
                {
                    wait_ask(m, ctx, HostStage::SelectCard, ASK_LUOBI_GIVE_CARD)
                } else {
                    m.stage = HostStage::Act;
                    Flow::Done
                }
            }
            HostStage::SelectCard => {
                let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() else {
                    set_flag(ctx.core, owner, "luobi_selected_card", 0);
                    m.stage = HostStage::Act;
                    return Flow::Done;
                };
                set_flag(ctx.core, owner, "luobi_selected_card", u32::from(cid) + 1);
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_LUOBI_GIVE_TARGET)
            }
            HostStage::TargetChoice => {
                let sel = flag(ctx.core, owner, "luobi_selected_card");
                set_flag(ctx.core, owner, "luobi_selected_card", 0);
                if let (Some(ChoiceOption::Player(t)), true) =
                    (ctx.answer().first().copied(), sel > 0)
                {
                    let cid = (sel - 1) as CardId;
                    m.cards.retain(|&c| c != cid);
                    move_cards_to_hand(ctx, owner, t, &[cid]);
                    if m.cards
                        .iter()
                        .any(|&c| ctx.core.player(owner).hand.contains(&c))
                        && !alive_others(ctx.core, owner).is_empty()
                    {
                        return wait_ask(m, ctx, HostStage::SelectCard, ASK_LUOBI_GIVE_CARD);
                    }
                }
                m.stage = HostStage::Act;
                Flow::Done
            }
            _ => Flow::Done,
        },
        "yaoyi" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_YAOYI_TARGETS),
            HostStage::TargetChoice => {
                let chosen: Vec<PlayerId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    })
                    .collect();
                let mut ordered: Vec<PlayerId> = alive_others(ctx.core, owner)
                    .into_iter()
                    .filter(|p| chosen.contains(p) && !ctx.core.player(*p).hand.is_empty())
                    .take(2)
                    .collect();
                if ordered.is_empty() {
                    m.stage = HostStage::Act;
                    return Flow::Done;
                }
                let first = ordered.remove(0);
                m.target = Some(first);
                m.targets = ordered;
                wait_ask(m, ctx, HostStage::CompareTargetPick, ASK_YAOYI_TARGET_GIVE)
            }
            HostStage::CompareTargetPick => {
                if let (Some(t), Some(ChoiceOption::Card(cid))) =
                    (m.target, ctx.answer().first().copied())
                {
                    move_cards_to_hand(ctx, t, owner, &[cid]);
                }
                while let Some(next) = m.targets.first().copied() {
                    m.targets.remove(0);
                    if ctx.core.player(next).alive && !ctx.core.player(next).hand.is_empty() {
                        m.target = Some(next);
                        return wait_ask(
                            m,
                            ctx,
                            HostStage::CompareTargetPick,
                            ASK_YAOYI_TARGET_GIVE,
                        );
                    }
                }
                m.stage = HostStage::Act;
                Flow::Done
            }
            _ => Flow::Done,
        },
        "shiqin" => {
            if let Some(t) = m.target {
                if !ctx.core.turn.ciqiu_dying.contains(&t) {
                    ctx.core.turn.ciqiu_dying.push(t);
                }
            }
            Flow::Done
        }
        "zyhufu" => {
            if !has_slot_equipped(ctx.core, ctx.catalog, owner, EquipSlot::Armor) {
                draw_direct(ctx, owner, 1);
            }
            Flow::Done
        }
        "hanbei" => {
            if let Some(cid) = m.card {
                for &t in &m.targets {
                    ctx.core
                        .vm_aux
                        .flags
                        .insert((owner, format!("sha_unrespondable:{cid}:{t}")), 1);
                }
            }
            Flow::Done
        }
        "yxsdili" => {
            let lost =
                (ctx.core.player(owner).max_hp as i32 - ctx.core.player(owner).hp).max(0) as u32;
            let n = ((lost + 1) / 2).min(2);
            if n > 0 {
                draw_direct(ctx, owner, n);
            }
            Flow::Done
        }
        "kuangchan" => {
            set_flag(ctx.core, owner, "kuangchan_init_done", 1);
            let p = ctx.core.player_mut(owner);
            if p.max_hp > 1 && p.max_hp != crate::state::INFINITE_HP {
                p.max_hp -= 1;
                if p.hp > p.max_hp as i32 {
                    p.hp = p.max_hp as i32;
                }
            }
            Flow::Done
        }
        "zhijie" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_ZHIJIE_CARD),
            HostStage::SelectCard => {
                m.stage = HostStage::Act;
                if let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() {
                    discard_cards(ctx, owner, &[cid]);
                    draw_direct(ctx, owner, 2);
                } else {
                    refund_once_used(ctx, owner, "zhijie");
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "chujia" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_CHUJIA_DISCARD_TWO),
            HostStage::SelectCard => {
                let cards: Vec<CardId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    })
                    .collect();
                if cards.len() != 2
                    || is_card_red(ctx.core, ctx.catalog, owner, cards[0])
                        != is_card_red(ctx.core, ctx.catalog, owner, cards[1])
                {
                    refund_once_used(ctx, owner, "chujia");
                    m.stage = HostStage::Act;
                    return Flow::Done;
                }
                m.cards = cards;
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_CHUJIA_TARGET)
            }
            HostStage::TargetChoice => {
                m.stage = HostStage::Act;
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    refund_once_used(ctx, owner, "chujia");
                    return Flow::Done;
                };
                let chosen = m.cards.clone();
                discard_cards(ctx, owner, &chosen);
                let n = (ctx.core.player(t).max_hp as i32 - ctx.core.player(t).hp).max(0) as u32;
                if n > 0 {
                    draw_direct(ctx, t, n);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        _ => {
            m.stage = HostStage::Act;
            Flow::Done
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(in crate::content::host) fn ask_u08_batch_a(
    stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    card: Option<u32>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    let (observer, spec) = build_spec(
        stage, owner, target, targets, card, cards, core, catalog,
    )?;
    Some(ask(
        observer,
        spec.prompt,
        spec.kind,
        spec.options,
        spec.min,
        spec.max,
        spec.allow_cancel,
    ))
}

#[allow(clippy::too_many_arguments)]
fn build_spec(
    _stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    _targets: &[PlayerId],
    card: Option<u32>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<(PlayerId, ChoiceSpec)> {
    let ask_id = card?;
    let build = |observer: PlayerId,
                 prompt: &'static str,
                 kind: ChoiceKind,
                 mut options: Vec<ChoiceOption>,
                 min: usize,
                 max: usize,
                 allow_cancel: bool|
     -> Option<(PlayerId, ChoiceSpec)> {
        if options.is_empty() {
            return None;
        }
        if allow_cancel && !options.contains(&ChoiceOption::Pass) {
            options.push(ChoiceOption::Pass);
        }
        Some((
            observer,
            ChoiceSpec {
                prompt,
                kind,
                options,
                min,
                max,
                allow_cancel,
            },
        ))
    };
    let confirm = |observer: PlayerId, prompt: &'static str| -> Option<(PlayerId, ChoiceSpec)> {
        Some((
            observer,
            ChoiceSpec {
                prompt,
                kind: ChoiceKind::Confirm,
                options: vec![ChoiceOption::Confirm, ChoiceOption::Pass],
                min: 1,
                max: 1,
                allow_cancel: true,
            },
        ))
    };

    match ask_id {
        ASK_YAOJI_CONFIRM => confirm(owner, "yaoji:confirm"),
        ASK_LIEBO_TARGET => {
            let opts: Vec<ChoiceOption> = liebo_targets(core, owner)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            build(owner, "liebo:choose_target", ChoiceKind::SelectPlayers, opts, 1, 1, true)
        }
        ASK_ZHUXIN_TARGET => {
            let opts: Vec<ChoiceOption> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| !core.player(p).hand.is_empty())
                .map(ChoiceOption::Player)
                .collect();
            build(owner, "zhuxin:choose_target", ChoiceKind::SelectPlayers, opts, 1, 1, true)
        }
        ASK_ZHUXIN_OWNER_CARD => {
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            build(owner, "zhuxin:owner_card", ChoiceKind::SelectCards, opts, 1, 1, false)
        }
        ASK_ZHUXIN_TARGET_CARD => {
            let t = target?;
            let opts: Vec<ChoiceOption> = core
                .player(t)
                .hand
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            build(t, "zhuxin:target_card", ChoiceKind::SelectCards, opts, 1, 1, false)
        }
        ASK_WLIANHUAN_DISCARD_EQUIP => {
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .equips
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            build(owner, "wlianhuan:discard_equip", ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        ASK_HULI_CARD => {
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| catalog.effective_card_suit(core, owner, c) == Suit::Heart)
                .map(ChoiceOption::Card)
                .collect();
            build(owner, "huli:choose_card", ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        ASK_HULI_TARGET => {
            let opts: Vec<ChoiceOption> = huli_targets(core, catalog, owner)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            build(owner, "huli:choose_target", ChoiceKind::SelectPlayers, opts, 1, 1, true)
        }
        ASK_YIXIN_DISCARD_TWO => {
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .chain(core.player(owner).equips.iter())
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            if opts.len() < 2 {
                return None;
            }
            build(owner, "yixin:discard_two", ChoiceKind::SelectCards, opts, 2, 2, true)
        }
        ASK_YIXIN_TARGET => {
            let opts: Vec<ChoiceOption> = damaged_alive_players(core, owner)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            build(owner, "yixin:choose_target", ChoiceKind::SelectPlayers, opts, 1, 1, true)
        }
        ASK_ZBUDAO_CONFIRM => confirm(owner, "zbudao:confirm"),
        ASK_ZBUDAO2_CARD => {
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            build(owner, "zbudao2:choose_card", ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        ASK_ZBUDAO2_TARGET => {
            let opts: Vec<ChoiceOption> = alive_others(core, owner)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            build(owner, "zbudao2:choose_target", ChoiceKind::SelectPlayers, opts, 1, 1, true)
        }
        ASK_TAIJI_CHOOSE_SHA => {
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| is_sha_card(core, catalog, c))
                .map(ChoiceOption::Card)
                .collect();
            build(owner, "taiji:choose_sha", ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        ASK_TAIJI_CHOOSE_TARGET => {
            let sha_cid = cards.first().copied();
            let opts: Vec<ChoiceOption> = taiji_targets(core, catalog, owner, sha_cid)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            build(owner, "taiji:choose_target", ChoiceKind::SelectPlayers, opts, 1, 1, true)
        }
        ASK_LUOBI_CONFIRM => confirm(owner, "luobi:confirm"),
        ASK_LUOBI_GIVE_CARD => {
            let opts: Vec<ChoiceOption> = cards
                .iter()
                .copied()
                .filter(|c| core.player(owner).hand.contains(c))
                .map(ChoiceOption::Card)
                .collect();
            build(owner, "luobi:give_card", ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        ASK_LUOBI_GIVE_TARGET => {
            let opts: Vec<ChoiceOption> = alive_others(core, owner)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            build(owner, "luobi:give_target", ChoiceKind::SelectPlayers, opts, 1, 1, true)
        }
        ASK_YAOYI_TARGETS => {
            let opts: Vec<ChoiceOption> = yaoyi_candidates(core, catalog, owner)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            let max = opts.len().min(2);
            build(owner, "yaoyi:choose_targets", ChoiceKind::SelectPlayers, opts, 1, max, true)
        }
        ASK_YAOYI_TARGET_GIVE => {
            let t = target?;
            let opts: Vec<ChoiceOption> = core
                .player(t)
                .hand
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            build(t, "yaoyi:target_give", ChoiceKind::SelectCards, opts, 1, 1, false)
        }
        ASK_ZHIJIE_CARD => {
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| catalog.effective_card_suit(core, owner, c) == Suit::Heart)
                .map(ChoiceOption::Card)
                .collect();
            build(owner, "zhijie:choose_card", ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        ASK_CHUJIA_DISCARD_TWO => {
            let opts: Vec<ChoiceOption> = chujia_eligible_cards(core, catalog, owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            if opts.len() < 2 {
                return None;
            }
            build(owner, "chujia:discard_two", ChoiceKind::SelectCards, opts, 2, 2, true)
        }
        ASK_CHUJIA_TARGET => {
            let opts: Vec<ChoiceOption> = damaged_alive_players(core, owner)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            build(owner, "chujia:choose_target", ChoiceKind::SelectPlayers, opts, 1, 1, true)
        }
        _ => None,
    }
}
