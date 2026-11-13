//! L20 标记补充 Batch A 询问派生（`l20_batch_a_asks.rs`）：
//! 负责还原 `HostBehavior::L20MarkBatchA` 在挂起等待玩家输入时的 `ExpectedAsk`。

use super::*;
use crate::catalog::Catalog;
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::event::{EventData, LostOrigin};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow};
use crate::snap::{ask, ExpectedAsk};
use crate::state::{GameStateCore, KingdomIdentity};

pub(super) fn flag(core: &GameStateCore, p: PlayerId, key: &str) -> u32 {
    core.vm_aux
        .flags
        .get(&(p, key.to_string()))
        .copied()
        .unwrap_or(0)
}

pub(super) fn set_flag(core: &mut GameStateCore, p: PlayerId, key: &str, val: u32) {
    if val == 0 {
        core.vm_aux.flags.remove(&(p, key.to_string()));
    } else {
        core.vm_aux.flags.insert((p, key.to_string()), val);
    }
}

pub(super) fn alive_others(core: &GameStateCore, owner: PlayerId) -> Vec<PlayerId> {
    core.seat_order_from(owner)
        .into_iter()
        .filter(|&p| p != owner && core.player(p).alive)
        .collect()
}

pub(super) fn hand_and_equips(core: &GameStateCore, p: PlayerId) -> Vec<CardId> {
    let mut out = core.player(p).hand.clone();
    out.extend(core.player(p).equips.iter().copied());
    out
}

pub(super) fn refund_once_used(ctx: &mut Ctx<'_>, p: PlayerId, name: &str) {
    if let Some(def) = ctx.catalog.skill_by_name(name) {
        if let Some(inst) = ctx.core.skills.get_mut(&(p, def)) {
            inst.once_used = false;
        }
    }
}

pub(super) fn gz_mark_count(
    core: &GameStateCore,
    catalog: &Catalog,
    p: PlayerId,
    key: &str,
) -> u32 {
    let f = flag(core, p, key);
    if f > 0 {
        return f;
    }
    catalog
        .skill_by_name(key)
        .and_then(|d| core.vm_aux.marks.get(&(p, u32::from(d))).copied())
        .unwrap_or(0)
        .max(0) as u32
}

pub(super) fn has_gz_mark(
    core: &GameStateCore,
    catalog: &Catalog,
    p: PlayerId,
    key: &str,
) -> bool {
    gz_mark_count(core, catalog, p, key) > 0
}

fn dec_gz_mark_single(core: &mut GameStateCore, catalog: &Catalog, p: PlayerId, key: &str) -> bool {
    let f = flag(core, p, key);
    if f > 0 {
        set_flag(core, p, key, f - 1);
        if let Some(d) = catalog.skill_by_name(key) {
            let k = (p, u32::from(d));
            if let Some(m) = core.vm_aux.marks.get_mut(&k) {
                *m = (*m - 1).max(0);
                if *m == 0 {
                    core.vm_aux.marks.remove(&k);
                }
            }
        }
        return true;
    }
    if let Some(d) = catalog.skill_by_name(key) {
        let k = (p, u32::from(d));
        let cur = core.vm_aux.marks.get(&k).copied().unwrap_or(0);
        if cur > 0 {
            if cur <= 1 {
                core.vm_aux.marks.remove(&k);
            } else {
                core.vm_aux.marks.insert(k, cur - 1);
            }
            return true;
        }
    }
    false
}

pub(super) fn consume_gz_mark_or_yexinjia(
    core: &mut GameStateCore,
    catalog: &Catalog,
    p: PlayerId,
    primary_key: &str,
) -> bool {
    if dec_gz_mark_single(core, catalog, p, primary_key) {
        return true;
    }
    dec_gz_mark_single(core, catalog, p, "yexinjia_mark")
}

pub(super) fn guozhan_mark_options(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<ChoiceOption> {
    let has_yexin = has_gz_mark(core, catalog, owner, "yexinjia_mark");
    let mut opts = Vec::new();
    if has_gz_mark(core, catalog, owner, "xianqu_mark") || has_yexin {
        opts.push(ChoiceOption::Tag(0));
    }
    if has_gz_mark(core, catalog, owner, "zhulianbihe_mark") || has_yexin {
        opts.push(ChoiceOption::Tag(1));
    }
    if has_gz_mark(core, catalog, owner, "yinyang_mark") || has_yexin {
        opts.push(ChoiceOption::Tag(2));
    }
    if (has_gz_mark(core, catalog, owner, "zhulianbihe_mark") || has_yexin)
        && core.player(owner).hp < core.player(owner).max_hp as i32
    {
        opts.push(ChoiceOption::Tag(3));
    }
    opts
}

pub(super) fn xianqu_target_options(core: &GameStateCore, p: PlayerId) -> Vec<ChoiceOption> {
    let mut opts = Vec::new();
    let unseen_main = flag(core, p, "general_unrevealed") > 0
        || (matches!(core.mode, crate::state::GameMode::Kingdoms)
            && !core.player(p).revealed_generals[0]);
    if unseen_main {
        opts.push(ChoiceOption::Tag(1));
    }
    let unseen_deputy = flag(core, p, "deputy_unrevealed") > 0
        || (matches!(core.mode, crate::state::GameMode::Kingdoms)
            && !core.player(p).revealed_generals[1]
            && core.player(p).general_slots[1].is_some());
    if unseen_deputy {
        opts.push(ChoiceOption::Tag(2));
    }
    opts
}

pub(super) fn xianqu_valid_targets(core: &GameStateCore, owner: PlayerId) -> Vec<PlayerId> {
    alive_others(core, owner)
        .into_iter()
        .filter(|&p| !xianqu_target_options(core, p).is_empty())
        .collect()
}

pub(super) fn find_wengua_owner(
    core: &GameStateCore,
    catalog: &Catalog,
    giver: PlayerId,
) -> Option<PlayerId> {
    let explicit = flag(core, giver, "wengua_owner_seat");
    if explicit > 0 {
        let seat = (explicit - 1) as PlayerId;
        if (seat as usize) < core.players.len() && seat != giver && core.player(seat).alive {
            return Some(seat);
        }
    }
    for sk in ["wengua", "wengua5"] {
        if let Some(wg_def) = catalog.skill_by_name(sk) {
            for p in core.seat_order_from(giver) {
                if p != giver
                    && core.player(p).alive
                    && ((core
                        .skills
                        .get(&(p, wg_def))
                        .is_some_and(|i| i.enabled && !i.expires_end_of_turn)
                        && flag(core, p, "wengua2_active") == 0)
                        || flag(core, p, "has_wengua") > 0)
                {
                    return Some(p);
                }
            }
        }
    }
    None
}

pub(super) fn take_card_from_player(ctx: &mut Ctx<'_>, p: PlayerId, cid: CardId) -> bool {
    if let Some(pos) = ctx.core.player(p).hand.iter().position(|&c| c == cid) {
        ctx.core.player_mut(p).hand.remove(pos);
        ctx.note_cards_lost(p, &[cid]);
        ctx.emit(EventData::CardsLost {
            player: p,
            cards: vec![cid],
            origins: vec![LostOrigin::Hand],
        });
        return true;
    }
    if let Some(pos) = ctx.core.player(p).equips.iter().position(|&c| c == cid) {
        ctx.core.player_mut(p).equips.remove(pos);
        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, p, cid);
        ctx.note_cards_lost(p, &[cid]);
        ctx.emit(EventData::CardsLost {
            player: p,
            cards: vec![cid],
            origins: vec![LostOrigin::Equip],
        });
        return true;
    }
    false
}

fn ensure_deck_nonempty(ctx: &mut Ctx<'_>) {
    if ctx.core.deck.is_empty() && !ctx.core.discard.is_empty() {
        let mut recycle = std::mem::take(&mut ctx.core.discard);
        ctx.core.rng.shuffle(&mut recycle);
        ctx.core.deck = recycle;
    }
}

pub(super) fn draw_from_bottom(ctx: &mut Ctx<'_>, p: PlayerId) -> Option<CardId> {
    ensure_deck_nonempty(ctx);
    if ctx.core.deck.is_empty() {
        return None;
    }
    let cid = ctx.core.deck.remove(0);
    ctx.core.player_mut(p).hand.push(cid);
    *ctx.core.vm_aux.drawn.entry(p).or_insert(0) += 1;
    ctx.emit(EventData::CardsDrawn {
        player: p,
        cards: vec![cid],
    });
    Some(cid)
}

pub(super) fn draw_from_top_n(ctx: &mut Ctx<'_>, p: PlayerId, count: u32) -> Vec<CardId> {
    let mut drawn = Vec::new();
    for _ in 0..count {
        ensure_deck_nonempty(ctx);
        let Some(cid) = ctx.core.deck.pop() else {
            break;
        };
        ctx.core.player_mut(p).hand.push(cid);
        drawn.push(cid);
    }
    if !drawn.is_empty() {
        *ctx.core.vm_aux.drawn.entry(p).or_insert(0) += drawn.len() as u32;
        ctx.emit(EventData::CardsDrawn {
            player: p,
            cards: drawn.clone(),
        });
    }
    drawn
}

pub(super) fn give_hand_card(ctx: &mut Ctx<'_>, from: PlayerId, to: PlayerId, cid: CardId) -> bool {
    if from == to {
        return false;
    }
    if let Some(pos) = ctx.core.player(from).hand.iter().position(|&c| c == cid) {
        ctx.core.player_mut(from).hand.remove(pos);
        ctx.note_cards_lost(from, &[cid]);
        ctx.core.player_mut(to).hand.push(cid);
        ctx.emit(EventData::CardsLost {
            player: from,
            cards: vec![cid],
            origins: vec![LostOrigin::Hand],
        });
        ctx.emit(EventData::CardsMoved {
            cards: vec![cid],
            from,
            to,
        });
        true
    } else {
        false
    }
}

fn aozhan_player_faction(
    core: &GameStateCore,
    catalog: &Catalog,
    p: PlayerId,
) -> Option<String> {
    if flag(core, p, "general_unrevealed") > 0 {
        return None;
    }
    match &core.player(p).kingdom_identity {
        KingdomIdentity::Declared(s) if !s.is_empty() && s != "unknown" => {
            if s == "ye" {
                Some(format!("ye:{p}"))
            } else {
                let norm = match s.as_str() {
                    "jun_caocao" | "gz_jun_caocao" | "pe_jun_caocao" | "jun_wei" | "君曹操" => "wei",
                    "jun_liubei" | "gz_jun_liubei" | "pe_jun_liubei" | "jun_shu" | "君刘备" => "shu",
                    "jun_sunquan" | "gz_jun_sunquan" | "pe_jun_sunquan" | "jun_wu" | "君孙权" => "wu",
                    "jun_zhangjiao" | "gz_jun_zhangjiao" | "pe_jun_zhangjiao" | "jun_qun" | "君张角" => "qun",
                    other => other,
                };
                Some(format!("f:{norm}"))
            }
        }
        KingdomIdentity::Ye => Some(format!("ye:{p}")),
        _ => {
            let f = core.effective_faction(catalog, p)?;
            let name = match f {
                crate::catalog::Faction::Wei => "wei",
                crate::catalog::Faction::Shu => "shu",
                crate::catalog::Faction::Wu => "wu",
                crate::catalog::Faction::Qun => "qun",
            };
            Some(format!("f:{name}"))
        }
    }
}

pub(super) fn aozhan_condition_met(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> bool {
    if flag(core, owner, "aozhan_active") > 0 || flag(core, owner, "cfg_aozhan_off") > 0 {
        return false;
    }
    let alive: Vec<PlayerId> = core.alive_players().collect();
    let total_seats = core.players.len();
    if alive.is_empty() || alive.len() > 4 || (alive.len() > 3 && total_seats <= 7) {
        return false;
    }
    let mut factions: Vec<(PlayerId, String)> = Vec::with_capacity(alive.len());
    for &p in &alive {
        let Some(f) = aozhan_player_faction(core, catalog, p) else {
            return false;
        };
        for &(prev_p, ref prev_f) in &factions {
            if prev_f == &f || core.kingdom_friends(prev_p, p) {
                return false;
            }
        }
        factions.push((p, f));
    }
    true
}

fn build_spec(
    stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    card: Option<u32>,
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<(PlayerId, ChoiceSpec)> {
    if stage != HostStage::SelectCard && stage != HostStage::TargetChoice {
        return None;
    }
    let ask_code = card?;
    match ask_code {
        ASK_WENGUA_CHOOSE_CARD => {
            let options: Vec<ChoiceOption> = hand_and_equips(core, owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            if options.is_empty() {
                return None;
            }
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "wengua:choose_card",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        ASK_WENGUA_CHOOSE_SIDE => Some((
            owner,
            ChoiceSpec {
                kind: ChoiceKind::PickTag,
                prompt: "wengua:choose_side",
                options: vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
                min: 1,
                max: 1,
                allow_cancel: true,
            },
        )),
        ASK_WENGUA5_GIVE_CARD => {
            let options: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            if options.is_empty() {
                return None;
            }
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "wengua5:give_card",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        ASK_WENGUA5_CHOOSE_SIDE => {
            let wg_owner = target.unwrap_or(owner);
            Some((
                wg_owner,
                ChoiceSpec {
                    kind: ChoiceKind::PickTag,
                    prompt: "wengua5:choose_side",
                    options: vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        ASK_GUOZHAN_CHOOSE_MARK => {
            let options = guozhan_mark_options(core, catalog, owner);
            if options.is_empty() {
                return None;
            }
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::PickTag,
                    prompt: "_guozhan_marks:choose_mark",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        ASK_XIANQU_CHOOSE_TARGET => {
            let options: Vec<ChoiceOption> = xianqu_valid_targets(core, owner)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            if options.is_empty() {
                return None;
            }
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "xianqu_mark:choose_target",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        ASK_XIANQU_CHOOSE_OPTION => {
            let t = target.unwrap_or(owner);
            let options = xianqu_target_options(core, t);
            if options.is_empty() {
                return None;
            }
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::PickTag,
                    prompt: "xianqu_mark:choose_option",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        ASK_ZHULIANBIHE_TAO_CONFIRM => Some((
            owner,
            ChoiceSpec {
                kind: ChoiceKind::Confirm,
                prompt: "_zhulianbihe_mark_tao:confirm",
                options: vec![ChoiceOption::Confirm, ChoiceOption::Pass],
                min: 1,
                max: 1,
                allow_cancel: true,
            },
        )),
        ASK_YINYANG_ADD_CONFIRM => Some((
            owner,
            ChoiceSpec {
                kind: ChoiceKind::Confirm,
                prompt: "_yinyang_mark_add:confirm",
                options: vec![ChoiceOption::Confirm, ChoiceOption::Pass],
                min: 1,
                max: 1,
                allow_cancel: true,
            },
        )),
        _ => None,
    }
}

#[allow(clippy::too_many_arguments)]
pub(in crate::content::host) fn ask_l20_batch_a(
    stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    _targets: &[PlayerId],
    card: Option<u32>,
    _cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    let (observer, spec) = build_spec(stage, owner, target, card, core, catalog)?;
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

pub(super) fn wait_ask(m: &mut HostMachine, ctx: &mut Ctx<'_>, ask_code: u32) -> Flow {
    m.card = Some(ask_code);
    m.stage = HostStage::SelectCard;
    if let Some((observer, spec)) =
        build_spec(m.stage, m.owner, m.target, m.card, ctx.core, ctx.catalog)
    {
        Flow::WaitChoice { observer, spec }
    } else {
        Flow::Done
    }
}
