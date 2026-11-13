//! L27 选将美化 Batch A 询问构造与技能 9–16 步进（`l27_batch_a_asks.rs`）。
//!
//! 覆盖技能 9–16：
//! 9. `shouxi`（曹节·守玺，`2480`）
//! 10. `dinghan`（荀谌·定汉，`2562`）
//! 11. `quanji`（钟会·权计，`2645`）
//! 12. `potzhuangshi`（势魏延·壮誓，`3218`）
//! 13. `qimou`（魏延·奇谋，`5902`）
//! 14. `reqimou`（界魏延·奇谋，`6059`）
//! 15. `sbjianxiong`（谋曹操·奸雄，`6216`）
//! 16. `jinghe`（南华老仙·经合，`7568`）

use super::*;
use crate::catalog::Catalog;
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::event::EventData;
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

const CANCELLABLE_ASKS: [u32; 16] = [
    ASK_REGUSHE_TARGETS,
    ASK_REGUSHE_TARGET_DISCARD,
    ASK_REGUSHE_CONFIRM_DRAW,
    ASK_REHUASHEN_ACTION,
    ASK_TAOLUAN_CARD,
    ASK_TAOLUAN_VIRTUAL,
    ASK_TAOLUAN_HELPER_GIVE,
    ASK_SHOUXI_DECLARE,
    ASK_SHOUXI_ATTACKER_DISCARD,
    ASK_SHOUXI_ATTACKER_GAIN,
    ASK_DINGHAN_ADJUST,
    ASK_QUANJI_CONFIRM,
    ASK_POTZHUANGSHI_DISCARD,
    ASK_POTZHUANGSHI_LOSE_HP,
    ASK_SBJIANXIONG_CONFIRM,
    ASK_JINGHE_SHOW,
];

const FIRST_CANCELLABLE_ASKS: [u32; 8] = [
    ASK_REGUSHE_TARGETS,
    ASK_REHUASHEN_ACTION,
    ASK_TAOLUAN_CARD,
    ASK_SHOUXI_DECLARE,
    ASK_DINGHAN_ADJUST,
    ASK_QUANJI_CONFIRM,
    ASK_SBJIANXIONG_CONFIRM,
    ASK_JINGHE_SHOW,
];

const CUSTOM_CANCEL_ASKS: [u32; 6] = [
    ASK_REGUSHE_TARGET_DISCARD,
    ASK_REGUSHE_CONFIRM_DRAW,
    ASK_TAOLUAN_HELPER_GIVE,
    ASK_SHOUXI_ATTACKER_DISCARD,
    ASK_POTZHUANGSHI_DISCARD,
    ASK_POTZHUANGSHI_LOSE_HP,
];

pub(super) fn is_cancellable(ask_id: u32) -> bool {
    CANCELLABLE_ASKS.contains(&ask_id) || ask_id == ASK_SBJIANXIONG_REMOVE_MARK
}

pub(super) fn is_first_cancellable(ask_id: u32) -> bool {
    FIRST_CANCELLABLE_ASKS.contains(&ask_id)
}

pub(super) fn has_custom_cancel(ask_id: u32) -> bool {
    CUSTOM_CANCEL_ASKS.contains(&ask_id) || ask_id == ASK_SBJIANXIONG_REMOVE_MARK
}

fn build_ask(
    ask_id: u32,
    owner: PlayerId,
    target: Option<PlayerId>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<(PlayerId, ChoiceSpec)> {
    let mut built = build_ask_spec(ask_id, owner, target, cards, core, catalog)?;
    built.1.allow_cancel = is_cancellable(ask_id);
    Some(built)
}

fn build_ask_spec(
    ask_id: u32,
    owner: PlayerId,
    target: Option<PlayerId>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<(PlayerId, ChoiceSpec)> {
    match ask_id {
        ASK_REGUSHE_TARGETS => {
            let cands: Vec<PlayerId> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| !core.player(p).hand.is_empty())
                .collect();
            let max = cands.len().min(3);
            player_ask(owner, "regushe:choose_targets", cands, 1, max)
        }
        ASK_REGUSHE_PINDIAN_CARD => {
            card_ask(owner, "regushe:pindian_card", core.player(owner).hand.clone(), 1, 1)
        }
        ASK_REGUSHE_TARGET_PINDIAN_CARD => {
            let t = target.unwrap_or(owner);
            card_ask(t, "regushe:target_pindian_card", core.player(t).hand.clone(), 1, 1)
        }
        ASK_REGUSHE_TARGET_DISCARD => {
            let t = target.unwrap_or(owner);
            card_ask(t, "regushe:target_discard", hand_and_equips(core, t), 1, 1)
        }
        ASK_REGUSHE_CONFIRM_DRAW => tag_ask(owner, "regushe:confirm_draw", 0..=1),
        ASK_SBTONGYE_PREDICT => tag_ask(owner, "sbtongye:predict", 0..=1),
        ASK_REHUASHEN_SKILL => {
            let pool = flag(core, owner, "rehuashen_pool_count")
                .max(3)
                .min(REHUASHEN_SKILLS.len() as u32) as u8;
            tag_ask(owner, "rehuashen:choose_skill", 0..pool)
        }
        ASK_REHUASHEN_ACTION => tag_ask(owner, "rehuashen:choose_action", 0..=2),
        ASK_TAOLUAN_CARD => {
            card_ask(owner, "taoluan:choose_card", hand_and_equips(core, owner), 1, 1)
        }
        ASK_TAOLUAN_VIRTUAL => {
            let mask = flag(core, owner, "taoluan_used_mask");
            let avail: Vec<u8> = (0..TAOLUAN_KINDS.len() as u8)
                .filter(|&i| (mask & (1u32 << i)) == 0)
                .collect();
            tag_ask(owner, "taoluan:choose_virtual", avail)
        }
        ASK_TAOLUAN_HELPER => {
            player_ask(owner, "taoluan:choose_helper", alive_others(core, owner), 1, 1)
        }
        ASK_TAOLUAN_HELPER_GIVE => {
            let helper = target.unwrap_or(owner);
            let virt_idx = cards.first().copied().unwrap_or(0) as usize;
            let virt_is_basic = virt_idx < 3;
            let all_he = hand_and_equips(core, helper);
            let diff_type: Vec<CardId> = all_he
                .iter()
                .copied()
                .filter(|&c| catalog.card_kind(c).is_basic_kind() != virt_is_basic)
                .collect();
            let opts = if diff_type.is_empty() { all_he } else { diff_type };
            card_ask(helper, "taoluan:helper_give", opts, 1, 1)
        }
        ASK_SHOUXI_DECLARE => {
            let mask = flag(core, owner, "shouxi_declared_mask");
            let avail: Vec<u8> = (0..SHOUXI_KINDS.len() as u8)
                .filter(|&i| (mask & (1u32 << i)) == 0)
                .collect();
            tag_ask(owner, "shouxi:declare_kind", avail)
        }
        ASK_SHOUXI_ATTACKER_DISCARD => {
            let attacker = target.unwrap_or(owner);
            let decl_idx = (flag(core, owner, "shouxi_last_declared").saturating_sub(1) as usize)
                .min(SHOUXI_KINDS.len() - 1);
            let decl_kind = SHOUXI_KINDS[decl_idx];
            let all_he = hand_and_equips(core, attacker);
            let mut opts: Vec<ChoiceOption> = all_he
                .iter()
                .copied()
                .filter(|&c| {
                    let k = catalog.card_kind(c);
                    k == decl_kind || (decl_kind == CardKind::Sha && k.is_sha_like())
                })
                .map(ChoiceOption::Card)
                .collect();
            opts.push(ChoiceOption::Pass);
            Some((
                attacker,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "shouxi:attacker_discard",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        ASK_SHOUXI_ATTACKER_GAIN => {
            let attacker = target.unwrap_or(owner);
            card_ask(
                attacker,
                "shouxi:attacker_gain",
                hand_and_equips(core, owner),
                0,
                1,
            )
        }
        ASK_DINGHAN_ADJUST => tag_ask(owner, "dinghan:adjust_record", 0..15),
        ASK_QUANJI_CONFIRM => tag_ask(owner, "quanji:confirm", 0..=1),
        ASK_QUANJI_PUT_CARD => {
            card_ask(owner, "quanji:put_card", core.player(owner).hand.clone(), 1, 1)
        }
        ASK_POTZHUANGSHI_DISCARD => {
            let hand = core.player(owner).hand.clone();
            if hand.is_empty() {
                return None;
            }
            let max = hand.len();
            let opts = hand.into_iter().map(ChoiceOption::Card).collect();
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "potzhuangshi:discard_cards",
                    options: opts,
                    min: 0,
                    max,
                    allow_cancel: true,
                },
            ))
        }
        ASK_POTZHUANGSHI_LOSE_HP => {
            let hp = core.player(owner).hp.max(0) as u8;
            if hp == 0 {
                return None;
            }
            tag_ask(owner, "potzhuangshi:lose_hp", 0..=hp)
        }
        ASK_QIMOU_HP => {
            let hp = core.player(owner).hp.max(1) as u8;
            tag_ask(owner, "qimou:choose_hp", 1..=hp)
        }
        ASK_REQIMOU_HP => {
            let hp = core.player(owner).hp.max(1) as u8;
            tag_ask(owner, "reqimou:choose_hp", 1..=hp)
        }
        ASK_SBJIANXIONG_INIT => tag_ask(owner, "sbjianxiong:init_marks", 0..=2),
        ASK_SBJIANXIONG_CONFIRM => tag_ask(owner, "sbjianxiong:confirm", 0..=1),
        ASK_SBJIANXIONG_REMOVE_MARK => tag_ask(owner, "sbjianxiong:remove_mark", 0..=1),
        ASK_JINGHE_SHOW => {
            let hand = core.player(owner).hand.clone();
            let alive_cnt = alive_all(core, owner).len();
            let max = hand.len().min(alive_cnt).min(4).max(1);
            card_ask(owner, "jinghe:show_cards", hand, 1, max)
        }
        ASK_JINGHE_TARGETS => {
            let cnt = cards.len().max(1);
            let cands = alive_all(core, owner);
            let need = cnt.min(cands.len()).max(1);
            player_ask(owner, "jinghe:choose_targets", cands, need, need)
        }
        _ => None,
    }
}

pub(super) fn wait_ask(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    stage: HostStage,
    ask_id: u32,
) -> Flow {
    set_flag(ctx.core, m.owner, "l27_ask_id", ask_id);
    if let Some((observer, spec)) =
        build_ask(ask_id, m.owner, m.target, &m.cards, ctx.core, ctx.catalog)
    {
        m.stage = stage;
        Flow::WaitChoice { observer, spec }
    } else {
        set_flag(ctx.core, m.owner, "l27_ask_id", 0);
        Flow::Done
    }
}

pub(in crate::content::host) fn ask_l27_batch_a(
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
    let ask_id = flag(core, owner, "l27_ask_id");
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

pub(super) fn step_shouxi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            set_flag(ctx.core, owner, "shouxi_nullified", 0);
            set_flag(ctx.core, owner, "l27_shouxi_nullified", 0);
            wait_ask(m, ctx, HostStage::SelectNature, ASK_SHOUXI_DECLARE)
        }
        HostStage::SelectNature => {
            let Some(ChoiceOption::Tag(tag)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            let bit = (tag as usize).min(SHOUXI_KINDS.len() - 1);
            let mask = flag(ctx.core, owner, "shouxi_declared_mask") | (1u32 << bit);
            set_flag(ctx.core, owner, "shouxi_declared_mask", mask);
            set_flag(ctx.core, owner, "shouxi_last_declared", (bit as u32) + 1);
            let attacker = m.target.unwrap_or(owner);
            if !ctx.core.player(attacker).alive || hand_and_equips(ctx.core, attacker).is_empty() {
                set_flag(ctx.core, owner, "shouxi_nullified", 1);
                if !nullify_current_card_on_owner(ctx, owner, m.card) {
                    set_flag(ctx.core, owner, "l27_shouxi_nullified", 1);
                }
                set_flag(ctx.core, owner, "l27_ask_id", 0);
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::CardsAsk, ASK_SHOUXI_ATTACKER_DISCARD)
        }
        HostStage::CardsAsk => {
            let attacker = m.target.unwrap_or(owner);
            match ctx.answer().first().copied() {
                Some(ChoiceOption::Card(c)) => {
                    discard_cards(ctx, attacker, &[c]);
                    if ctx.core.player(attacker).alive
                        && !hand_and_equips(ctx.core, owner).is_empty()
                    {
                        return wait_ask(m, ctx, HostStage::TargetChoice, ASK_SHOUXI_ATTACKER_GAIN);
                    }
                }
                _ => {
                    set_flag(ctx.core, owner, "shouxi_nullified", 1);
                    if !nullify_current_card_on_owner(ctx, owner, m.card) {
                        set_flag(ctx.core, owner, "l27_shouxi_nullified", 1);
                    }
                }
            }
            set_flag(ctx.core, owner, "l27_ask_id", 0);
            Flow::Done
        }
        HostStage::TargetChoice => {
            let attacker = m.target.unwrap_or(owner);
            if let Some(ChoiceOption::Card(steal_cid)) = ctx.answer().first().copied() {
                move_cards_to_hand(ctx, owner, attacker, &[steal_cid]);
            }
            set_flag(ctx.core, owner, "l27_ask_id", 0);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn step_dinghan(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if m.card == Some(1) && m.cards.is_empty() {
                set_flag(ctx.core, owner, "dinghan_nullified", 0);
                set_flag(ctx.core, owner, "l27_dinghan_negated", 0);
                wait_ask(m, ctx, HostStage::SelectNature, ASK_DINGHAN_ADJUST)
            } else {
                let kind_disc = m.cards.first().copied().unwrap_or(0);
                let kind = m
                    .card
                    .map(|c| ctx.catalog.card_kind(c))
                    .unwrap_or_else(|| {
                        if kind_disc == CardKind::Dismantle as u32 {
                            CardKind::Dismantle
                        } else if kind_disc == CardKind::Snatch as u32 {
                            CardKind::Snatch
                        } else if kind_disc == CardKind::Duel as u32 {
                            CardKind::Duel
                        } else {
                            CardKind::Wuzhong
                        }
                    });
                let bit = dinghan_trick_bit(kind);
                let mask = flag(ctx.core, owner, "dinghan_recorded_mask") | (1u32 << bit);
                set_flag(ctx.core, owner, "dinghan_recorded_mask", mask);
                set_flag(ctx.core, owner, "dinghan_nullified", 1);
                if !nullify_current_card_on_owner(ctx, owner, m.card) {
                    set_flag(ctx.core, owner, "l27_dinghan_negated", 1);
                }
                Flow::Done
            }
        }
        HostStage::SelectNature => {
            if let Some(ChoiceOption::Tag(tag)) = ctx.answer().first().copied() {
                let bit = (tag as u32).min(30);
                let mask = flag(ctx.core, owner, "dinghan_recorded_mask") ^ (1u32 << bit);
                set_flag(ctx.core, owner, "dinghan_recorded_mask", mask);
            }
            set_flag(ctx.core, owner, "l27_ask_id", 0);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn step_quanji(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if m.card == Some(4) && m.targets.is_empty() {
                set_flag(ctx.core, owner, "quanji_hp_decreased_play", 0);
                set_flag(
                    ctx.core,
                    owner,
                    "quanji_play_start_hp",
                    ctx.core.player(owner).hp.max(0) as u32,
                );
                return Flow::Done;
            }
            if m.card.unwrap_or(0) >= 30 && m.cards.is_empty() {
                set_flag(ctx.core, owner, "quanji_hp_decreased_play", 1);
                return Flow::Done;
            }
            if !m.targets.is_empty()
                && ctx.core.turn.current == owner
                && ctx.core.turn.phase == Phase::Play
            {
                set_flag(ctx.core, owner, "quanji_hp_decreased_play", 1);
            }
            if m.cards.is_empty() {
                m.cards.push(1);
            }
            wait_ask(m, ctx, HostStage::DrawSettle, ASK_QUANJI_CONFIRM)
        }
        HostStage::DrawSettle => {
            if let Some(c) = ctx.core.deck.pop() {
                ctx.core.player_mut(owner).hand.push(c);
                ctx.emit(EventData::CardsDrawn {
                    player: owner,
                    cards: vec![c],
                });
            } else {
                ctx.start_draw(owner, 1);
            }
            if ctx.core.player(owner).hand.is_empty() {
                return advance_quanji_loop(m, ctx);
            }
            wait_ask(m, ctx, HostStage::SelectCard, ASK_QUANJI_PUT_CARD)
        }
        HostStage::SelectCard => {
            if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                if let Some(pos) = ctx.core.player(owner).hand.iter().position(|&x| x == c) {
                    ctx.core.player_mut(owner).hand.remove(pos);
                }
                ctx.core
                    .vm_aux
                    .piles
                    .entry((owner, "quanji".to_string()))
                    .or_default()
                    .push(c);
                let cnt = ctx
                    .core
                    .vm_aux
                    .piles
                    .get(&(owner, "quanji".to_string()))
                    .map_or(1, |p| p.len() as u32)
                    .max(flag(ctx.core, owner, "quanji_count") + 1);
                set_flag(ctx.core, owner, "quanji_count", cnt);
                set_flag(ctx.core, owner, "quanji", cnt);
            }
            advance_quanji_loop(m, ctx)
        }
        _ => Flow::Done,
    }
}

fn advance_quanji_loop(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let rem = m.cards.first().copied().unwrap_or(1).saturating_sub(1);
    if rem > 0 && ctx.core.player(m.owner).alive {
        m.cards[0] = rem;
        wait_ask(m, ctx, HostStage::DrawSettle, ASK_QUANJI_CONFIRM)
    } else {
        set_flag(ctx.core, m.owner, "l27_ask_id", 0);
        Flow::Done
    }
}

pub(super) fn step_potzhuangshi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if m.card == Some(14) {
                set_flag(ctx.core, owner, "potzhuangshi_unblockable_x", 0);
                set_flag(ctx.core, owner, "potzhuangshi_damage_y", 0);
                set_flag(ctx.core, owner, "l27_potzhuangshi_unblockable", 0);
                return Flow::Done;
            }
            if !ctx.core.player(owner).hand.is_empty() {
                wait_ask(m, ctx, HostStage::SelectCard, ASK_POTZHUANGSHI_DISCARD)
            } else if ctx.core.player(owner).hp > 0 {
                wait_ask(m, ctx, HostStage::SelectNature, ASK_POTZHUANGSHI_LOSE_HP)
            } else {
                Flow::Done
            }
        }
        HostStage::SelectCard => {
            let picked: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Card(c) => Some(c),
                    _ => None,
                })
                .collect();
            if !picked.is_empty() {
                let x = picked.len() as u32;
                discard_cards(ctx, owner, &picked);
                set_flag(ctx.core, owner, "potzhuangshi_unblockable_x", x);
                set_flag(ctx.core, owner, "l27_potzhuangshi_unblockable", x);
            }
            if ctx.core.player(owner).hp > 0 {
                wait_ask(m, ctx, HostStage::SelectNature, ASK_POTZHUANGSHI_LOSE_HP)
            } else {
                set_flag(ctx.core, owner, "l27_ask_id", 0);
                Flow::Done
            }
        }
        HostStage::SelectNature => {
            if let Some(ChoiceOption::Tag(y)) = ctx.answer().first().copied() {
                if y > 0 {
                    let y_u32 = u32::from(y);
                    set_flag(ctx.core, owner, "potzhuangshi_damage_y", y_u32);
                    ctx.emit(EventData::LoseHp {
                        player: owner,
                        amount: u32::from(y),
                    });
                }
            }
            set_flag(ctx.core, owner, "l27_ask_id", 0);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn step_qimou(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if m.card == Some(16) {
                set_flag(ctx.core, owner, "qimou_x", 0);
                set_flag(ctx.core, owner, "l27_qimou_range_bonus", 0);
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::SelectNature, ASK_QIMOU_HP)
        }
        HostStage::SelectNature => {
            let x = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(n)) => u32::from(n).max(1),
                _ => 1,
            };
            set_flag(ctx.core, owner, "qimou_used", 1);
            set_flag(ctx.core, owner, "qimou_x", x);
            set_flag(ctx.core, owner, "l27_qimou_range_bonus", x);
            ctx.emit(EventData::LoseHp {
                player: owner,
                amount: x,
            });
            set_flag(ctx.core, owner, "l27_ask_id", 0);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn step_reqimou(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if m.card == Some(16) {
                set_flag(ctx.core, owner, "reqimou_x", 0);
                set_flag(ctx.core, owner, "l27_qimou_range_bonus", 0);
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::SelectNature, ASK_REQIMOU_HP)
        }
        HostStage::SelectNature => {
            let x = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(n)) => u32::from(n).max(1),
                _ => 1,
            };
            set_flag(ctx.core, owner, "reqimou_used", 1);
            set_flag(ctx.core, owner, "reqimou_x", x);
            set_flag(ctx.core, owner, "l27_qimou_range_bonus", x);
            ctx.emit(EventData::LoseHp {
                player: owner,
                amount: x,
            });
            ctx.start_draw(owner, x);
            set_flag(ctx.core, owner, "l27_ask_id", 0);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn step_sbjianxiong(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if m.card == Some(20) && m.cards.is_empty() {
                set_flag(ctx.core, owner, "sbjianxiong_init", 1);
                wait_ask(m, ctx, HostStage::SelectNature, ASK_SBJIANXIONG_INIT)
            } else {
                wait_ask(m, ctx, HostStage::DrawSettle, ASK_SBJIANXIONG_CONFIRM)
            }
        }
        HostStage::SelectNature => {
            let n = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(t)) => u32::from(t).min(2),
                _ => 0,
            };
            set_flag(ctx.core, owner, "sbjianxiong", n);
            set_flag(ctx.core, owner, "sbjianxiong_marks", n);
            set_flag(ctx.core, owner, "l27_ask_id", 0);
            Flow::Done
        }
        HostStage::DrawSettle => {
            if let Some(cid) = m.card {
                if let Some(pos) = ctx.core.table.iter().position(|&x| x == cid) {
                    ctx.core.table.remove(pos);
                    ctx.core.player_mut(owner).hand.push(cid);
                } else if let Some(pos) = ctx.core.discard.iter().position(|&x| x == cid) {
                    ctx.core.discard.remove(pos);
                    ctx.core.player_mut(owner).hand.push(cid);
                }
            }
            let marks = flag(ctx.core, owner, "sbjianxiong")
                .max(flag(ctx.core, owner, "sbjianxiong_marks"));
            let draw_n = (1i32 - marks as i32).max(0) as u32;
            if draw_n > 0 {
                ctx.start_draw(owner, draw_n);
            }
            if marks > 0 {
                wait_ask(m, ctx, HostStage::HuanrenPick, ASK_SBJIANXIONG_REMOVE_MARK)
            } else {
                set_flag(ctx.core, owner, "l27_ask_id", 0);
                Flow::Done
            }
        }
        HostStage::HuanrenPick => {
            if !ctx.answer().is_empty() {
                let marks = flag(ctx.core, owner, "sbjianxiong")
                    .max(flag(ctx.core, owner, "sbjianxiong_marks"))
                    .saturating_sub(1);
                set_flag(ctx.core, owner, "sbjianxiong", marks);
                set_flag(ctx.core, owner, "sbjianxiong_marks", marks);
            }
            set_flag(ctx.core, owner, "l27_ask_id", 0);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn step_jinghe(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if m.card == Some(20) {
                set_flag(ctx.core, owner, "jinghe_active_round", 0);
                for pid in 0..(ctx.core.players.len() as u8) {
                    let g_def = flag(ctx.core, pid, "jinghe_granted_def");
                    if g_def > 0 {
                        ctx.core.skills.remove(&(pid, (g_def - 1) as u16));
                        set_flag(ctx.core, pid, "jinghe_granted_def", 0);
                    }
                    set_flag(ctx.core, pid, "jinghe_granted_skill", 0);
                    set_flag(ctx.core, pid, "l27_jinghe_shared", 0);
                }
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::SelectCard, ASK_JINGHE_SHOW)
        }
        HostStage::SelectCard => {
            let picked: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Card(c) => Some(c),
                    _ => None,
                })
                .collect();
            if picked.is_empty() {
                return Flow::Done;
            }
            m.cards = picked;
            set_flag(ctx.core, owner, "jinghe_shown_count", m.cards.len() as u32);
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_JINGHE_TARGETS)
        }
        HostStage::TargetChoice => {
            let targets: Vec<PlayerId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Player(p) => Some(p),
                    _ => None,
                })
                .collect();
            set_flag(ctx.core, owner, "jinghe_active_round", 1);
            for (i, &t) in targets.iter().enumerate() {
                let s_name = JINGHE_SKILL_POOL[i % JINGHE_SKILL_POOL.len()];
                if let Some(def) = grant_skill_named(ctx, t, s_name) {
                    set_flag(ctx.core, t, "jinghe_granted_def", u32::from(def) + 1);
                }
                set_flag(ctx.core, t, "jinghe_granted_skill", (i as u32) + 1);
                set_flag(ctx.core, t, "l27_jinghe_shared", 1);
            }
            set_flag(ctx.core, owner, "l27_ask_id", 0);
            Flow::Done
        }
        _ => Flow::Done,
    }
}
