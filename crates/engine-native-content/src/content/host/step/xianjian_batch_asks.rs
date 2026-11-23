//! U05 仙剑（`xianjian`）Batch A..E 共享原语、询问构造与内核钩子（`xianjian_batch_asks.rs`）。

use super::*;
use crate::catalog::{CardKind, Catalog, Suit};
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::event::{DamageData, DamageNature, EventData, LostOrigin};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow, LogEvent};
use crate::snap::{ask, ExpectedAsk};
use crate::state::{GameStateCore, SkillInstance};

pub(super) const ASK_QIONGGUANG_CONFIRM: u32 = 1;
pub(super) const ASK_HUANLEI_CONFIRM: u32 = 2;
pub(super) const ASK_TIANWU_CONFIRM: u32 = 3;
pub(super) const ASK_YUJIA_CONFIRM: u32 = 4;
pub(super) const ASK_YUJIA_PICK: u32 = 5;
pub(super) const ASK_JIANZHEN_SHA: u32 = 6;
pub(super) const ASK_FEIZHUA_CONFIRM: u32 = 7;
pub(super) const ASK_QIJIAN_TARGETS: u32 = 8;
pub(super) const ASK_DANQING_TARGETS: u32 = 9;
pub(super) const ASK_LEIYU_CARD: u32 = 10;
pub(super) const ASK_LINGDI_CARD: u32 = 11;
pub(super) const ASK_LINGDI_TARGET: u32 = 12;
pub(super) const ASK_XJZONGYU_CARD: u32 = 13;
pub(super) const ASK_DUJIANG_CARD: u32 = 15;
pub(super) const ASK_YANZHAN_CARD: u32 = 16;
pub(super) const ASK_YANZHAN_TARGET: u32 = 17;
pub(super) const ASK_FEIXIA_CARD: u32 = 18;
pub(super) const ASK_HUXI_TARGET: u32 = 19;
pub(super) const ASK_GUIJIANG_CARD: u32 = 20;
pub(super) const ASK_GUIJIANG_TARGET: u32 = 21;
pub(super) const ASK_DUXINSHU_TARGET: u32 = 22;
pub(super) const ASK_DUXINSHU_STEAL: u32 = 23;
pub(super) const ASK_DUXINSHU_GIVE: u32 = 24;
pub(super) const ASK_TUOQIAO_CARD: u32 = 25;
pub(super) const ASK_TUOQIAO_TARGET: u32 = 26;

pub(super) fn flag(core: &GameStateCore, p: PlayerId, key: &str) -> u32 {
    core.vm_aux.flags.get(&(p, key.to_string())).copied().unwrap_or(0)
}

pub(super) fn set_flag(core: &mut GameStateCore, p: PlayerId, key: &str, val: u32) {
    if val == 0 {
        core.vm_aux.flags.remove(&(p, key.to_string()));
    } else {
        core.vm_aux.flags.insert((p, key.to_string()), val);
    }
}

pub(super) fn suit_bit(suit: Suit) -> u32 {
    match suit {
        Suit::Spade => 1 << 0,
        Suit::Heart => 1 << 1,
        Suit::Club => 1 << 2,
        Suit::Diamond => 1 << 3,
    }
}

pub(super) fn suit_tag(suit: Suit) -> u32 {
    match suit {
        Suit::Spade => 1,
        Suit::Heart => 2,
        Suit::Club => 3,
        Suit::Diamond => 4,
    }
}

pub(super) fn is_red_card(core: &GameStateCore, catalog: &Catalog, owner: PlayerId, c: CardId) -> bool {
    matches!(catalog.effective_card_suit(core, owner, c), Suit::Heart | Suit::Diamond)
}

pub(super) fn is_black_card(core: &GameStateCore, catalog: &Catalog, owner: PlayerId, c: CardId) -> bool {
    matches!(catalog.effective_card_suit(core, owner, c), Suit::Spade | Suit::Club)
}

pub(super) fn is_friendly(core: &GameStateCore, a: PlayerId, b: PlayerId) -> bool {
    use crate::state::{GameMode, Role};
    if a == b {
        return true;
    }
    if flag(core, a, &format!("tongling_ally:{b}")) > 0 || core.player(a).yexinjia_friends.contains(&b) {
        return true;
    }
    match &core.mode {
        GameMode::Identity { roles } => {
            let ra = roles.get(a as usize);
            let rb = roles.get(b as usize);
            matches!(
                (ra, rb),
                (Some(Role::Lord | Role::Loyalist), Some(Role::Lord | Role::Loyalist))
                    | (Some(Role::Rebel), Some(Role::Rebel))
            )
        }
        GameMode::Versus { teams, .. } => teams.iter().any(|t| t.contains(&a) && t.contains(&b)),
        GameMode::Boss { boss, fellows, .. } => {
            let sa = a == *boss || fellows.contains(&a);
            let sb = b == *boss || fellows.contains(&b);
            sa == sb
        }
        GameMode::Doudizhu { lord } => (a == *lord) == (b == *lord),
        GameMode::Prototype | GameMode::Kingdoms | GameMode::Kof { .. } => false,
    }
}

pub(super) fn alive_others(core: &GameStateCore, owner: PlayerId) -> Vec<PlayerId> {
    core.seat_order_from(owner).into_iter().filter(|&p| p != owner && core.player(p).alive).collect()
}

pub(super) fn alive_enemies(core: &GameStateCore, owner: PlayerId) -> Vec<PlayerId> {
    alive_others(core, owner).into_iter().filter(|&p| !is_friendly(core, owner, p)).collect()
}

pub(super) fn hand_and_equips(core: &GameStateCore, p: PlayerId) -> Vec<CardId> {
    let pl = core.player(p);
    pl.hand.iter().chain(pl.equips.iter()).copied().collect()
}

pub(super) fn has_skill_named(core: &GameStateCore, catalog: &Catalog, p: PlayerId, name: &str) -> bool {
    catalog.skill_by_name(name).is_some_and(|d| core.has_skill(p, d))
}

pub(super) fn ensure_temp_skill(ctx: &mut Ctx<'_>, owner: PlayerId, name: &str, expires_end_of_turn: bool) {
    if let Some(def) = ctx.catalog.skill_by_name(name) {
        if expires_end_of_turn {
            ctx.add_skill_until_turn_end(owner, def);
        } else {
            ctx.ensure_skill(
                (owner, def),
                SkillInstance { owner, enabled: true, once_used: false, expires_end_of_turn: false, rounds: 0 },
            );
        }
    }
}

pub(super) fn remove_skill_named(ctx: &mut Ctx<'_>, owner: PlayerId, name: &str) {
    if let Some(def) = ctx.catalog.skill_by_name(name) {
        ctx.remove_skill(owner, def);
    }
}

pub(super) fn has_once_turn(core: &GameStateCore, owner: PlayerId, skill: &str) -> bool {
    core.vm_aux.once.contains(&(owner, skill.to_string(), 1))
}

pub(super) fn mark_once_turn(core: &mut GameStateCore, owner: PlayerId, skill: &str) {
    core.vm_aux.once.insert((owner, skill.to_string(), 1));
}

pub(super) fn discard_from_he(ctx: &mut Ctx<'_>, actor: PlayerId, card: CardId) {
    if ctx.core.player(actor).hand.contains(&card) {
        ctx.discard_from_hand(actor, card);
    } else if ctx.core.player(actor).equips.contains(&card) {
        ctx.core.player_mut(actor).equips.retain(|&x| x != card);
        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, actor, card);
        ctx.to_discard(card);
        ctx.log(LogEvent::CardDiscarded { player: actor, cards: vec![card] });
        ctx.note_cards_lost(actor, &[card]);
        ctx.emit(EventData::CardsLost { player: actor, cards: vec![card], origins: vec![LostOrigin::Equip] });
        (ctx.catalog.hooks.on_cards_to_discard)(ctx, actor, &[card], false);
    }
}

pub(super) fn u05_apply_buff(ctx: &mut Ctx<'_>, target: PlayerId) {
    if !ctx.core.player(target).alive {
        return;
    }
    let pl = ctx.core.player(target);
    let mut pool = vec![1u8, 3];
    if pl.hp < pl.max_hp as i32 {
        pool.push(2);
    }
    if !pl.yinshen_active {
        pool.push(4);
    }
    if !pl.judge.is_empty() {
        pool.push(5);
    }
    if pl.linked || pl.face_down {
        pool.push(6);
    }
    let idx = ctx.core.rng.gen_range(pool.len() as u32) as usize;
    match pool[idx] {
        1 => ctx.start_draw(target, 1),
        2 => {
            ctx.recover(target, 1, None);
        }
        3 => {
            let cur = i32::from(ctx.core.player(target).hujia);
            ctx.core.player_mut(target).hujia = (cur + 1).min(99) as u8;
        }
        4 => {
            ctx.core.player_mut(target).yinshen_active = true;
        }
        5 => {
            let judges: Vec<CardId> = ctx.core.player_mut(target).judge.drain(..).collect();
            if !judges.is_empty() {
                for &c in &judges {
                    ctx.to_discard(c);
                }
                ctx.log(LogEvent::CardDiscarded { player: target, cards: judges.clone() });
                ctx.emit(EventData::CardsLost {
                    player: target,
                    origins: vec![LostOrigin::Judge; judges.len()],
                    cards: judges,
                });
            }
        }
        _ => {
            if ctx.core.player(target).linked {
                ctx.core.player_mut(target).linked = false;
                ctx.log(LogEvent::Linked { player: target, linked: false });
            }
            if ctx.core.player(target).face_down {
                ctx.turn_over(target);
            }
        }
    }
}

pub(super) fn u05_apply_debuff(ctx: &mut Ctx<'_>, target: PlayerId) {
    if !ctx.core.player(target).alive {
        return;
    }
    let pl = ctx.core.player(target);
    let mut pool = Vec::new();
    if !pl.hand.is_empty() || !pl.equips.is_empty() {
        pool.push(1u8);
    }
    pool.push(2u8);
    if pl.hp != 1 {
        pool.push(3u8);
    }
    if !pl.linked {
        pool.push(4u8);
    }
    if !pl.fengyin_active && flag(ctx.core, target, "xj_fengyin_sealed") == 0 {
        pool.push(5u8);
    }
    let delay_avail = ctx
        .core
        .deck
        .iter()
        .chain(ctx.core.discard.iter())
        .copied()
        .any(|c| {
            let k = ctx.catalog.card_kind(c);
            matches!(k, CardKind::Le | CardKind::Bing | CardKind::ShanDian | CardKind::HongShui | CardKind::Guiyoujie)
                && !pl.judge.iter().any(|&jc| ctx.catalog.card_kind(jc) == k)
        });
    if delay_avail {
        pool.push(6u8);
    }
    if pl.hp == 1 && pool.len() > 1 {
        pool.retain(|&x| x != 2);
    }
    if pool.is_empty() {
        return;
    }
    let idx = ctx.core.rng.gen_range(pool.len() as u32) as usize;
    match pool[idx] {
        1 => {
            let he = hand_and_equips(ctx.core, target);
            if !he.is_empty() {
                let ci = ctx.core.rng.gen_range(he.len() as u32) as usize;
                discard_from_he(ctx, target, he[ci]);
            }
        }
        2 => ctx.emit(EventData::LoseHp { player: target, amount: 1 }),
        3 => ctx.emit(EventData::Damage(crate::machine::damage_data(None, target, 1))),
        4 => {
            ctx.core.player_mut(target).linked = true;
            ctx.log(LogEvent::Linked { player: target, linked: true });
        }
        5 => {
            set_flag(ctx.core, target, "xj_fengyin_sealed", 1);
            let to_disable: Vec<u16> = ctx
                .core
                .skills
                .iter()
                .filter_map(|(&(p, def), inst)| {
                    (p == target && inst.enabled && ctx.catalog.is_xj_fengyin_blocked(def))
                        .then_some(def)
                })
                .collect();
            for def in to_disable {
                if let Some(inst) = ctx.core.skills.get_mut(&(target, def)) {
                    inst.enabled = false;
                }
                set_flag(ctx.core, target, &format!("xj_fengyin_disabled:{def}"), 1);
            }
        }
        _ => {
            let candidates: Vec<CardId> = ctx
                .core
                .deck
                .iter()
                .chain(ctx.core.discard.iter())
                .copied()
                .filter(|&c| {
                    let k = ctx.catalog.card_kind(c);
                    matches!(k, CardKind::Le | CardKind::Bing | CardKind::ShanDian | CardKind::HongShui | CardKind::Guiyoujie)
                        && !ctx.core.player(target).judge.iter().any(|&jc| ctx.catalog.card_kind(jc) == k)
                })
                .collect();
            if !candidates.is_empty() {
                let ci = ctx.core.rng.gen_range(candidates.len() as u32) as usize;
                let picked = candidates[ci];
                if let Some(pos) = ctx.core.deck.iter().position(|&c| c == picked) {
                    ctx.core.deck.remove(pos);
                } else if let Some(pos) = ctx.core.discard.iter().position(|&c| c == picked) {
                    ctx.core.discard.remove(pos);
                }
                ctx.core.player_mut(target).judge.push(picked);
            }
        }
    }
}

pub(super) fn sha_user_blocked(core: &GameStateCore, catalog: &Catalog, owner: PlayerId, target: PlayerId) -> bool {
    crate::kernel::sha_user_or_target_blocked(core, catalog, owner, target)
}

pub(super) fn sha_target_legal(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    target: PlayerId,
    check_dist: bool,
) -> bool {
    if sha_user_blocked(core, catalog, owner, target) {
        return false;
    }
    if check_dist && core.computed_distance(catalog, owner, target) > core.attack_range(catalog, owner) {
        return false;
    }
    !crate::rvm_bridge::modifiers::eval_card_target_canceled(core, catalog, owner, target, 0, CardKind::Sha, DamageNature::Normal)
}

pub(super) fn can_owner_play_sha(core: &GameStateCore, catalog: &Catalog, owner: PlayerId) -> bool {
    let has_paoxiao = has_skill_named(core, catalog, owner, crate::vocabulary::names::PAO_XIAO);
    let has_zhuge = core.player(owner).equips.iter().any(|&c| catalog.card_kind(c) == CardKind::Zhuge);
    let (limit, mut unlimited) = crate::rvm_bridge::modifiers::eval_sha_use_limit(
        core,
        catalog,
        owner,
        None,
        CardKind::Sha,
        DamageNature::Normal,
        1,
    );
    if has_paoxiao || has_zhuge {
        unlimited = true;
    }
    let used = u32::from(core.turn.sha_used_play_phase) + flag(core, owner, "vm_sha_extra_used");
    unlimited || used < limit
}

pub(super) fn answered_confirm(ctx: &Ctx<'_>) -> bool {
    ctx.answer().first().copied() == Some(ChoiceOption::Confirm)
}

pub(super) fn answered_cards(ctx: &Ctx<'_>) -> Vec<CardId> {
    ctx.answer()
        .iter()
        .filter_map(|o| match *o {
            ChoiceOption::Card(c) => Some(c),
            _ => None,
        })
        .collect()
}

pub(super) fn answered_players(ctx: &Ctx<'_>) -> Vec<PlayerId> {
    ctx.answer()
        .iter()
        .filter_map(|o| match *o {
            ChoiceOption::Player(p) => Some(p),
            _ => None,
        })
        .collect()
}

pub(super) fn wait_ask(m: &mut HostMachine, ctx: &mut Ctx<'_>, stage: HostStage, ask_id: u32) -> Flow {
    m.stage = stage;
    m.card = Some(ask_id);
    if let Some((observer, spec)) = build_spec(m.owner, m.target, &m.targets, m.card, &m.cards, ctx.core, ctx.catalog) {
        Flow::WaitChoice { observer, spec }
    } else {
        m.stage = HostStage::Act;
        m.card = None;
        Flow::Done
    }
}

pub(super) fn ask_xianjian_batch_impl(
    _stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    card: Option<u32>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    let (observer, spec) = build_spec(owner, target, targets, card, cards, core, catalog)?;
    Some(ask(observer, spec.prompt, spec.kind, spec.options, spec.min, spec.max, spec.allow_cancel))
}

fn confirm_spec(owner: PlayerId, prompt: &'static str) -> (PlayerId, ChoiceSpec) {
    (
        owner,
        ChoiceSpec {
            kind: ChoiceKind::Confirm,
            prompt,
            options: vec![ChoiceOption::Confirm],
            min: 0,
            max: 1,
            allow_cancel: true,
        },
    )
}

fn cards_spec(
    observer: PlayerId,
    prompt: &'static str,
    cards: Vec<CardId>,
    min: usize,
    max: usize,
    allow_cancel: bool,
) -> Option<(PlayerId, ChoiceSpec)> {
    if cards.len() < min || cards.is_empty() {
        return None;
    }
    Some((
        observer,
        ChoiceSpec {
            kind: ChoiceKind::SelectCards,
            prompt,
            options: cards.into_iter().map(ChoiceOption::Card).collect(),
            min,
            max,
            allow_cancel,
        },
    ))
}

fn players_spec(
    observer: PlayerId,
    prompt: &'static str,
    players: Vec<PlayerId>,
    min: usize,
    max: usize,
    allow_cancel: bool,
) -> Option<(PlayerId, ChoiceSpec)> {
    if players.len() < min || players.is_empty() {
        return None;
    }
    Some((
        observer,
        ChoiceSpec {
            kind: ChoiceKind::SelectPlayers,
            prompt,
            options: players.into_iter().map(ChoiceOption::Player).collect(),
            min,
            max,
            allow_cancel,
        },
    ))
}

pub(super) fn build_spec(
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    card: Option<u32>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<(PlayerId, ChoiceSpec)> {
    let ask_id = card?;
    match ask_id {
        ASK_QIONGGUANG_CONFIRM => Some(confirm_spec(owner, "xj_u05_qiongguang_confirm")),
        ASK_HUANLEI_CONFIRM => Some(confirm_spec(owner, "xj_u05_huanlei_confirm")),
        ASK_TIANWU_CONFIRM => Some(confirm_spec(owner, "xj_u05_tianwu_confirm")),
        ASK_YUJIA_CONFIRM => Some(confirm_spec(owner, "xj_u05_yujia_confirm")),
        ASK_FEIZHUA_CONFIRM => Some(confirm_spec(owner, "xj_u05_feizhua_confirm")),
        ASK_YUJIA_PICK => cards_spec(owner, "xj_u05_yujia_pick", cards.to_vec(), 1, 1, false),
        ASK_JIANZHEN_SHA => {
            let cand = *targets.first()?;
            let shas: Vec<CardId> = core
                .player(cand)
                .hand
                .iter()
                .copied()
                .filter(|&c| catalog.card_kind(c).is_sha_like())
                .collect();
            cards_spec(cand, "xj_u05_jianzhen_sha", shas, 1, 1, true)
        }
        ASK_QIJIAN_TARGETS => {
            let limit = core
                .turn
                .discard_phase_discarded
                .max(flag(core, owner, "xj_last_discard_count")) as usize;
            let elig: Vec<PlayerId> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| sha_target_legal(core, catalog, owner, p, false))
                .collect();
            let max = limit.min(elig.len()).max(1);
            players_spec(owner, "xj_u05_qijian_targets", elig, 1, max, true)
        }
        ASK_DANQING_TARGETS => {
            let all: Vec<PlayerId> = core.seat_order_from(owner).into_iter().filter(|&p| core.player(p).alive).collect();
            let max = all.len().min(4);
            players_spec(owner, "xj_u05_danqing_targets", all, 1, max, true)
        }
        ASK_LEIYU_CARD => {
            let blacks: Vec<CardId> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| is_black_card(core, catalog, owner, c))
                .collect();
            cards_spec(owner, "xj_u05_leiyu_card", blacks, 1, 1, true)
        }
        ASK_LINGDI_CARD => {
            let mask = flag(core, owner, "xj_lingdi_suits_mask");
            let opts: Vec<CardId> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| (mask & suit_bit(catalog.effective_card_suit(core, owner, c))) == 0)
                .collect();
            cards_spec(owner, "xj_u05_lingdi_card", opts, 1, 1, true)
        }
        ASK_LINGDI_TARGET => {
            let need = 1 + flag(core, owner, "xj_lingdi_uses_turn");
            let opts: Vec<PlayerId> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| core.computed_distance(catalog, owner, p).max(1) == need)
                .collect();
            players_spec(owner, "xj_u05_lingdi_target", opts, 1, 1, true)
        }
        ASK_XJZONGYU_CARD => {
            let opts: Vec<CardId> = hand_and_equips(core, owner)
                .into_iter()
                .filter(|&c| is_black_card(core, catalog, owner, c))
                .collect();
            cards_spec(owner, "xj_u05_xjzongyu_card", opts, 1, 1, true)
        }
        ASK_DUJIANG_CARD => {
            let opts: Vec<CardId> = hand_and_equips(core, owner)
                .into_iter()
                .filter(|&c| is_black_card(core, catalog, owner, c))
                .collect();
            cards_spec(owner, "xj_u05_dujiang_card", opts, 1, 1, true)
        }
        ASK_YANZHAN_CARD => {
            let opts: Vec<CardId> = hand_and_equips(core, owner)
                .into_iter()
                .filter(|&c| is_red_card(core, catalog, owner, c))
                .collect();
            cards_spec(owner, "xj_u05_yanzhan_card", opts, 1, 1, true)
        }
        ASK_YANZHAN_TARGET | ASK_HUXI_TARGET => {
            let prompt = if ask_id == ASK_YANZHAN_TARGET {
                "xj_u05_yanzhan_target"
            } else {
                "xj_u05_huxi_target"
            };
            let opts: Vec<PlayerId> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| sha_target_legal(core, catalog, owner, p, true))
                .collect();
            players_spec(owner, prompt, opts, 1, 1, true)
        }
        ASK_FEIXIA_CARD => {
            let opts: Vec<CardId> = hand_and_equips(core, owner)
                .into_iter()
                .filter(|&c| is_red_card(core, catalog, owner, c))
                .collect();
            cards_spec(owner, "xj_u05_feixia_card", opts, 1, 1, true)
        }
        ASK_GUIJIANG_CARD => {
            let opts: Vec<CardId> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| is_black_card(core, catalog, owner, c))
                .collect();
            cards_spec(owner, "xj_u05_guijiang_card", opts, 1, 1, true)
        }
        ASK_GUIJIANG_TARGET => {
            let opts: Vec<PlayerId> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| !has_skill_named(core, catalog, p, "guijiang2") && flag(core, p, "u05_mod:guijiang2") == 0)
                .collect();
            players_spec(owner, "xj_u05_guijiang_target", opts, 1, 1, true)
        }
        ASK_DUXINSHU_TARGET => {
            let opts: Vec<PlayerId> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| !core.player(p).hand.is_empty())
                .collect();
            players_spec(owner, "xj_u05_duxinshu_target", opts, 1, 1, true)
        }
        ASK_DUXINSHU_STEAL => {
            let t = target?;
            let opts = core.player(t).hand.clone();
            cards_spec(owner, "xj_u05_duxinshu_steal", opts, 1, 1, true)
        }
        ASK_DUXINSHU_GIVE => {
            let opts = core.player(owner).hand.clone();
            cards_spec(owner, "xj_u05_duxinshu_give", opts, 1, 1, false)
        }
        ASK_TUOQIAO_CARD => {
            let fixed_target = target;
            let opts: Vec<CardId> = hand_and_equips(core, owner)
                .into_iter()
                .filter(|&c| {
                    is_black_card(core, catalog, owner, c)
                        && fixed_target.is_none_or(|t| {
                            !crate::rvm_bridge::modifiers::eval_card_target_canceled(
                                core,
                                catalog,
                                owner,
                                t,
                                c,
                                CardKind::Shihuifen,
                                DamageNature::Normal,
                            )
                        })
                })
                .collect();
            cards_spec(owner, "xj_u05_tuoqiao_card", opts, 1, 1, true)
        }
        ASK_TUOQIAO_TARGET => {
            let chosen_c = cards.first().copied().unwrap_or(0);
            let opts: Vec<PlayerId> = core
                .seat_order_from(owner)
                .into_iter()
                .filter(|&p| {
                    p != owner
                        && core.player(p).alive
                        && !has_skill_named(core, catalog, p, crate::vocabulary::host::GJ_FOOD_SHIHUIFEN_BAN_KEY)
                        && !crate::rvm_bridge::modifiers::eval_card_target_canceled(core, catalog, owner, p, chosen_c, CardKind::Shihuifen, DamageNature::Normal)
                })
                .collect();
            players_spec(owner, "xj_u05_tuoqiao_target", opts, 1, 1, true)
        }
        _ => None,
    }
}

pub(in crate::content::host) fn yujia_jiguan_pool(core: &GameStateCore, catalog: &Catalog) -> Vec<CardId> {
    let mut out = Vec::new();
    for &c in core.deck.iter().chain(core.discard.iter()) {
        if catalog.card_kind(c).is_jiguan_kind() {
            out.push(c);
        }
    }
    if out.is_empty() {
        for &(c, kind) in &core.detached_cards {
            if kind == crate::event::DetachedKind::Unclaimed
                && catalog.card_kind(c).is_jiguan_kind()
            {
                out.push(c);
            }
        }
    }
    out
}

pub(crate) fn host_on_hp_changed_xianjian(ctx: &mut Ctx<'_>, player: PlayerId, delta: i32) {
    if delta == 0 || !ctx.core.player(player).alive || ctx.catalog.is_tieji_sealed(ctx.core, player) {
        return;
    }
    if let Some(def) = ctx.catalog.skill_by_name("xfenxin") {
        if ctx.core.has_skill(player, def) {
            ctx.out.push(LogEvent::SkillTriggered {
                skill: ctx.catalog.skills[def as usize].name,
                owner: player,
                presentation: None,
            });
            ctx.start_draw(player, delta.unsigned_abs());
        }
    }
}

pub(crate) fn host_on_choose_respond_begin_xianjian(ctx: &mut Ctx<'_>, responder: PlayerId, need_tag: u8) {
    if ctx.core.turn.current == responder || !ctx.core.player(responder).alive || ctx.catalog.is_tieji_sealed(ctx.core, responder) {
        return;
    }
    let Some(def) = ctx.catalog.skill_by_name("longxi") else {
        return;
    };
    if !ctx.core.has_skill(responder, def) {
        return;
    }
    let self_dying = ctx.res.stack.iter().rev().any(|f| matches!(f.data, EventData::Dying { player, .. } if player == responder));
    let n = ctx.core.deck.len().min(2);
    let mut gained = Vec::new();
    for i in 0..n {
        let cid = ctx.core.deck[ctx.core.deck.len() - 1 - i];
        let kind = ctx.catalog.card_kind(cid);
        let matches = match need_tag {
            1 => kind == CardKind::Shan,
            2 => kind.is_sha_like(),
            3 => kind == CardKind::Nullify,
            4 => kind == CardKind::Tao || (kind == CardKind::Jiu && self_dying),
            _ => false,
        };
        if matches {
            gained.push(cid);
        }
    }
    if gained.is_empty() {
        return;
    }
    for &cid in &gained {
        if let Some(pos) = ctx.core.deck.iter().rposition(|&c| c == cid) {
            ctx.core.deck.remove(pos);
        }
        ctx.core.player_mut(responder).hand.push(cid);
    }
    ctx.out.push(LogEvent::SkillTriggered {
        skill: ctx.catalog.skills[def as usize].name,
        owner: responder,
        presentation: None,
    });
    ctx.log(LogEvent::CardsDrawn { player: responder, cards: gained });
}

pub(crate) fn host_on_damage_end_xianjian(ctx: &mut Ctx<'_>, d: &DamageData) {
    if d.prevented || d.amount <= 0 || d.settlement.notrigger {
        return;
    }
    let cur = ctx.core.turn.current;
    if flag(ctx.core, cur, "xj_fenglue_active") > 0
        && flag(ctx.core, cur, "xj_fenglue_target") == u32::from(d.target) + 1
    {
        let cnt = flag(ctx.core, cur, "xj_fenglue_draw_count");
        set_flag(ctx.core, cur, "xj_fenglue_draw_count", cnt + 1);
    }
    let Some(src) = d.source else {
        return;
    };
    if let Some(cid) = d.card {
        if flag(ctx.core, src, &format!("xj_tianjian_card:{cid}")) > 0 && ctx.core.player(d.target).alive {
            let he = hand_and_equips(ctx.core, d.target);
            if !he.is_empty() {
                let i = ctx.core.rng.gen_range(he.len() as u32) as usize;
                discard_from_he(ctx, d.target, he[i]);
            }
        }
        if flag(ctx.core, src, &format!("xj_qianfang_card:{cid}")) > 0
            && ctx.core.player(src).alive
            && !is_friendly(ctx.core, src, d.target)
        {
            ctx.start_draw(src, 1);
        }
    }
    if ctx.core.turn.current == src && flag(ctx.core, src, "ywuhun_active") > 0 && ctx.core.player(src).alive {
        set_flag(ctx.core, src, "xj_ywuhun_abort", 1);
    }
}

pub(crate) fn host_on_recover_before_xianjian(ctx: &mut Ctx<'_>, player: PlayerId, amount: &mut u32) {
    if *amount == 0 || !ctx.core.player(player).alive || ctx.core.player(player).hp <= 0 || ctx.catalog.is_tieji_sealed(ctx.core, player) {
        return;
    }
    if let Some(def) = ctx.catalog.skill_by_name("tanhua") {
        if ctx.core.has_skill(player, def) {
            let orig = *amount;
            *amount = 0;
            ctx.out.push(LogEvent::SkillTriggered {
                skill: ctx.catalog.skills[def as usize].name,
                owner: player,
                presentation: None,
            });
            ctx.start_draw(player, orig * 2);
        }
    }
}

pub(crate) fn host_draw_begin_xianjian(ctx: &mut Ctx<'_>, player: PlayerId, pending: &mut i32) {
    if let Some(def) = ctx.catalog.skill_by_name("qianfang2") {
        if ctx.core.has_skill(player, def) {
            *pending += 1;
        }
    }
}

pub(crate) fn host_on_hand_card_removed_xianjian(ctx: &mut Ctx<'_>, player: PlayerId, card: CardId) {
    let is_catalog_du = ctx
        .catalog
        .cards
        .get(card as usize)
        .is_some_and(|d| d.name == "du" || d.name == "毒" || d.name.starts_with("xj_du_"))
        && ctx.catalog.skill_by_name("dujiang").is_some();
    let held_du = flag(ctx.core, player, &format!("xj_du_holder:{card}")) > 0;
    if held_du || is_catalog_du {
        set_flag(ctx.core, player, &format!("xj_du_holder:{card}"), 0);
        set_flag(ctx.core, player, &format!("is_du_card:{card}"), 0);
        if ctx.core.player(player).alive && !has_skill_named(ctx.core, ctx.catalog, player, "g_du") {
            ctx.emit(EventData::LoseHp { player, amount: 1 });
        }
    }
    if has_skill_named(ctx.core, ctx.catalog, player, "zhangmu_ai")
        && !ctx.core.player(player).hand.iter().any(|&c| ctx.catalog.card_kind(c) == CardKind::Shan)
    {
        remove_skill_named(ctx, player, "zhangmu_ai");
    }
}

pub(crate) fn host_on_phase_use_end_xianjian(ctx: &mut Ctx<'_>, player: PlayerId) {
    let seats = ctx.core.players.len() as PlayerId;
    for p in 0..seats {
        remove_skill_named(ctx, p, "yuexing2");
    }
    if flag(ctx.core, player, "u05_mod:yanzhan4") > 0 {
        set_flag(ctx.core, player, "u05_mod:yanzhan4", 0);
        remove_skill_named(ctx, player, "yanzhan4");
    }
}

pub(crate) fn host_on_phase_end_xianjian(ctx: &mut Ctx<'_>) {
    let cur = ctx.core.turn.current;
    if !ctx.core.player(cur).skip_next_draw {
        remove_skill_named(ctx, cur, "guijin2");
    }
    set_flag(ctx.core, cur, "xj_lingdi_suits_mask", 0);
    set_flag(ctx.core, cur, "xj_lingdi_uses_turn", 0);
    set_flag(ctx.core, cur, "xj_yanshi2_turn", 0);
    remove_skill_named(ctx, cur, "yanshi2");
    remove_skill_named(ctx, cur, "tianwu2");
}
