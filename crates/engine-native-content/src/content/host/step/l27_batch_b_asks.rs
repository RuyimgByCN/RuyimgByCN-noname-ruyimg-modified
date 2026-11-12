//! L27 选将美化 Batch B 询问构造（`l27_batch_b_asks.rs`）。

use super::*;
use crate::catalog::{Catalog, Suit};
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::snap::ExpectedAsk;
use crate::state::GameStateCore;

fn spec_ask(
    chooser: PlayerId,
    kind: ChoiceKind,
    prompt: &'static str,
    options: Vec<ChoiceOption>,
    min: usize,
    max: usize,
) -> Option<(PlayerId, ChoiceSpec)> {
    (!options.is_empty() && options.len() >= min).then_some((
        chooser,
        ChoiceSpec {
            kind,
            prompt,
            options,
            min,
            max,
            allow_cancel: false,
        },
    ))
}

fn tag_ask(
    chooser: PlayerId,
    prompt: &'static str,
    tags: impl IntoIterator<Item = u8>,
) -> Option<(PlayerId, ChoiceSpec)> {
    spec_ask(
        chooser,
        ChoiceKind::PickTag,
        prompt,
        tags.into_iter().map(ChoiceOption::Tag).collect(),
        1,
        1,
    )
}

fn player_ask(
    chooser: PlayerId,
    prompt: &'static str,
    players: Vec<PlayerId>,
    min: usize,
    max: usize,
) -> Option<(PlayerId, ChoiceSpec)> {
    spec_ask(
        chooser,
        ChoiceKind::SelectPlayers,
        prompt,
        players.into_iter().map(ChoiceOption::Player).collect(),
        min,
        max,
    )
}

fn card_ask(
    chooser: PlayerId,
    prompt: &'static str,
    cards: Vec<CardId>,
    min: usize,
    max: usize,
) -> Option<(PlayerId, ChoiceSpec)> {
    spec_ask(
        chooser,
        ChoiceKind::SelectCards,
        prompt,
        cards.into_iter().map(ChoiceOption::Card).collect(),
        min,
        max,
    )
}

const CANCELLABLE_ASKS: [u32; 17] = [
    ASK_BOLAN_SKILL,
    ASK_DUORUI_SLOT,
    ASK_PRO_DUORUI_SLOT,
    ASK_XIONGHUO_TARGET,
    ASK_MBWULING_TARGET,
    ASK_SBQINGZHENG_SUITS,
    ASK_SBQINGZHENG_GAIN_MARK,
    ASK_SPYANJIAO_SUIT,
    ASK_TWXIAFENG_COUNT,
    ASK_TWXINGZHUI_TURNS,
    ASK_XINGCHONG_DRAW,
    ASK_XINGCHONG_SHOW,
    ASK_TWFENGJI_CARD,
    ASK_TWHARVESTINORI_CARD,
    ASK_TWZHOUHU_CARD,
    ASK_TWZUHUO_CARD,
    ASK_TWZHOUZU_TARGET,
];

const FIRST_CANCELLABLE_ASKS: [u32; 15] = [
    ASK_BOLAN_SKILL,
    ASK_DUORUI_SLOT,
    ASK_PRO_DUORUI_SLOT,
    ASK_XIONGHUO_TARGET,
    ASK_MBWULING_TARGET,
    ASK_SBQINGZHENG_SUITS,
    ASK_SPYANJIAO_SUIT,
    ASK_TWXIAFENG_COUNT,
    ASK_TWXINGZHUI_TURNS,
    ASK_XINGCHONG_DRAW,
    ASK_TWFENGJI_CARD,
    ASK_TWHARVESTINORI_CARD,
    ASK_TWZHOUHU_CARD,
    ASK_TWZUHUO_CARD,
    ASK_TWZHOUZU_TARGET,
];

pub(super) fn is_cancellable(ask_id: u32) -> bool {
    CANCELLABLE_ASKS.contains(&ask_id)
}

pub(super) fn is_first_cancellable(ask_id: u32) -> bool {
    FIRST_CANCELLABLE_ASKS.contains(&ask_id)
}

pub(super) fn has_custom_cancel(ask_id: u32) -> bool {
    matches!(ask_id, ASK_SBQINGZHENG_GAIN_MARK | ASK_XINGCHONG_SHOW)
}

pub(super) fn hand_suit_tags(core: &GameStateCore, catalog: &Catalog, p: PlayerId) -> Vec<u8> {
    let mut mask = 0u8;
    for &c in &core.player(p).hand {
        if let Some(cd) = catalog.cards.get(c as usize) {
            let bit = match cd.suit {
                Suit::Spade => 0,
                Suit::Heart => 1,
                Suit::Club => 2,
                Suit::Diamond => 3,
            };
            mask |= 1u8 << bit;
        }
    }
    let out: Vec<u8> = (0..4).filter(|&i| (mask & (1u8 << i)) != 0).collect();
    if out.is_empty() && !core.player(p).hand.is_empty() {
        vec![0, 1, 2, 3]
    } else {
        out
    }
}

pub(super) fn target_skills_list(core: &GameStateCore, target: PlayerId) -> Vec<u16> {
    let mut defs: Vec<u16> = core
        .skills
        .iter()
        .filter(|&(&(p, _), inst)| p == target && inst.enabled)
        .map(|(&(_, def), _)| def)
        .collect();
    defs.sort_unstable();
    defs
}

fn build_ask(
    ask_id: u32,
    owner: PlayerId,
    target: Option<PlayerId>,
    _cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<(PlayerId, ChoiceSpec)> {
    let mut built = build_ask_spec(ask_id, owner, target, core, catalog)?;
    built.1.allow_cancel = is_cancellable(ask_id);
    Some(built)
}

fn build_ask_spec(
    ask_id: u32,
    owner: PlayerId,
    target: Option<PlayerId>,
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<(PlayerId, ChoiceSpec)> {
    match ask_id {
        ASK_BOLAN_SKILL | ASK_BOLAN_G_SKILL => {
            let chooser = if ask_id == ASK_BOLAN_G_SKILL {
                target.unwrap_or(owner)
            } else {
                owner
            };
            tag_ask(chooser, "bolan:choose_skill", 0..3)
        }
        ASK_DUORUI_SLOT | ASK_PRO_DUORUI_SLOT => {
            let mask = flag(core, owner, "drlt_duorui_sealed_slots_mask");
            let avail: Vec<u8> = (0..5).filter(|&i| (mask & (1u32 << i)) == 0).collect();
            let prompt = if ask_id == ASK_PRO_DUORUI_SLOT {
                "pro_drlt_duorui:choose_slot"
            } else {
                "drlt_duorui:choose_slot"
            };
            tag_ask(owner, prompt, avail)
        }
        ASK_DUORUI_SKILL | ASK_PRO_DUORUI_SKILL => {
            let t = target.unwrap_or(owner);
            let count = target_skills_list(core, t).len().max(3).min(8) as u8;
            let prompt = if ask_id == ASK_PRO_DUORUI_SKILL {
                "pro_drlt_duorui:choose_skill"
            } else {
                "drlt_duorui:choose_skill"
            };
            tag_ask(owner, prompt, 0..count)
        }
        ASK_XIONGHUO_TARGET => {
            let cands: Vec<PlayerId> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| flag(core, p, "xinfu_xionghuo_marked") == 0)
                .collect();
            player_ask(owner, "xinfu_xionghuo:choose_target", cands, 1, 1)
        }
        ASK_MBWULING_TARGET => {
            let cands: Vec<PlayerId> = alive_all(core, owner)
                .into_iter()
                .filter(|&p| flag(core, p, "mbwuling_spirit") == 0)
                .collect();
            player_ask(owner, "mbwuling:choose_target", cands, 1, 1)
        }
        ASK_MBWULING_ORDER => tag_ask(owner, "mbwuling:choose_order", 0..=4),
        ASK_SBQINGZHENG_SUITS => {
            let suits = hand_suit_tags(core, catalog, owner);
            if suits.is_empty() {
                return None;
            }
            let marks = flag(core, owner, "sbjianxiong_marks")
                .max(flag(core, owner, "sbjianxiong"))
                .min(2);
            let need = ((3 - marks).clamp(1, 3) as usize).min(suits.len());
            spec_ask(
                owner,
                ChoiceKind::PickTag,
                "sbqingzheng:choose_suits",
                suits.into_iter().map(ChoiceOption::Tag).collect(),
                need,
                need,
            )
        }
        ASK_SBQINGZHENG_TARGET => {
            let cands: Vec<PlayerId> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| !core.player(p).hand.is_empty())
                .collect();
            player_ask(owner, "sbqingzheng:choose_target", cands, 1, 1)
        }
        ASK_SBQINGZHENG_TARGET_SUIT => tag_ask(owner, "sbqingzheng:choose_target_suit", 0..4),
        ASK_SBQINGZHENG_GAIN_MARK => tag_ask(owner, "sbqingzheng:gain_mark", 0..=1),
        ASK_SPYANJIAO_SUIT => {
            let suits = hand_suit_tags(core, catalog, owner);
            tag_ask(owner, "spyanjiao:choose_suit", suits)
        }
        ASK_SPYANJIAO_TARGET => {
            player_ask(owner, "spyanjiao:choose_target", alive_others(core, owner), 1, 1)
        }
        ASK_TWXIAFENG_COUNT => {
            let marks = baonvezhi_count(core, owner).min(3) as u8;
            if marks == 0 {
                return None;
            }
            tag_ask(owner, "twxiafeng:choose_count", 1..=marks)
        }
        ASK_TWXINGZHUI_TURNS => tag_ask(owner, "twxingzhui:choose_turns", 1..=3),
        ASK_TWXINGZHUI_TARGET => {
            player_ask(owner, "twxingzhui:choose_target", alive_others(core, owner), 1, 1)
        }
        ASK_XINGCHONG_DRAW => {
            let limit = (core.player(owner).max_hp as u8).min(5);
            tag_ask(owner, "xingchong:choose_draw", 0..=limit)
        }
        ASK_XINGCHONG_SHOW => {
            let hand = core.player(owner).hand.clone();
            let rem = flag(core, owner, "xingchong_show_max").max(1) as usize;
            let max = hand.len().min(rem);
            if max == 0 {
                return None;
            }
            card_ask(owner, "xingchong:show_cards", hand, 1, max)
        }
        ASK_DCXIAOXI_COUNT => {
            let max_lose = core.player(owner).max_hp.saturating_sub(1).min(2) as u8;
            if max_lose == 0 {
                return None;
            }
            tag_ask(owner, "dcxiaoxi:choose_count", 1..=max_lose)
        }
        ASK_DCXIAOXI_TARGET => {
            let others = alive_others(core, owner);
            let range = core.attack_range(catalog, owner);
            let in_range: Vec<PlayerId> = others
                .iter()
                .copied()
                .filter(|&p| core.computed_distance(catalog, owner, p) <= range)
                .collect();
            let cands = if in_range.is_empty() { others } else { in_range };
            player_ask(owner, "dcxiaoxi:choose_target", cands, 1, 1)
        }
        ASK_DCXIAOXI_MODE => tag_ask(owner, "dcxiaoxi:choose_mode", 0..=1),
        ASK_TWFENGJI_CARD => {
            card_ask(owner, "twfengji:choose_card", hand_and_equips(core, owner), 1, 1)
        }
        ASK_TWFENGJI_TURNS => tag_ask(owner, "twfengji:choose_turns", 1..=3),
        ASK_TWHARVESTINORI_CARD => {
            let blacks: Vec<CardId> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| {
                    catalog
                        .cards
                        .get(c as usize)
                        .is_some_and(|cd| matches!(cd.suit, Suit::Spade | Suit::Club))
                })
                .collect();
            card_ask(owner, "twharvestinori:discard_black", blacks, 1, 1)
        }
        ASK_TWHARVESTINORI_TURNS => tag_ask(owner, "twharvestinori:choose_turns", 1..=3),
        ASK_TWZHOUHU_CARD => {
            let reds: Vec<CardId> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| {
                    catalog
                        .cards
                        .get(c as usize)
                        .is_some_and(|cd| matches!(cd.suit, Suit::Heart | Suit::Diamond))
                })
                .collect();
            card_ask(owner, "twzhouhu:discard_red", reds, 1, 1)
        }
        ASK_TWZHOUHU_TURNS => tag_ask(owner, "twzhouhu:choose_turns", 1..=3),
        ASK_TWZUHUO_CARD => {
            let nonbasics: Vec<CardId> = hand_and_equips(core, owner)
                .into_iter()
                .filter(|&c| !catalog.card_kind(c).is_basic_kind())
                .collect();
            card_ask(owner, "twzuhuo:discard_nonbasic", nonbasics, 1, 1)
        }
        ASK_TWZUHUO_TURNS => tag_ask(owner, "twzuhuo:choose_turns", 1..=3),
        ASK_TWZHOUZU_TARGET => {
            player_ask(owner, "twzhouzu:choose_target", alive_others(core, owner), 1, 1)
        }
        ASK_TWZHOUZU_TURNS => tag_ask(owner, "twzhouzu:choose_turns", 1..=3),
        _ => None,
    }
}

pub(super) fn wait_ask(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    stage: HostStage,
    ask_id: u32,
) -> Flow {
    set_flag(ctx.core, m.owner, "l27_b_ask_id", ask_id);
    if let Some((observer, spec)) =
        build_ask(ask_id, m.owner, m.target, &m.cards, ctx.core, ctx.catalog)
    {
        m.stage = stage;
        Flow::WaitChoice { observer, spec }
    } else {
        set_flag(ctx.core, m.owner, "l27_b_ask_id", 0);
        Flow::Done
    }
}

pub(in crate::content::host) fn ask_l27_batch_b(
    stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    _targets: &[PlayerId],
    _card: Option<CardId>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    if stage == HostStage::Act {
        return None;
    }
    let ask_id = flag(core, owner, "l27_b_ask_id");
    let (observer, spec) = build_ask(ask_id, owner, target, cards, core, catalog)?;
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
