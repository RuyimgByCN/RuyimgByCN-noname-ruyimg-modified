//! L28 雷霆万钧 Batch B 询问构造，以及神周瑜 `th_yeyan` / `qinyin` 步进（`l28_batch_b_asks.rs`）。

use super::*;
use crate::catalog::{Catalog, Suit};
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::event::{DamageNature, EventData};
use crate::snap::ExpectedAsk;
use crate::state::GameStateCore;

fn tag_ask(
    chooser: PlayerId,
    prompt: &'static str,
    tags: impl IntoIterator<Item = u8>,
    cancel: bool,
) -> Option<(PlayerId, ChoiceSpec)> {
    let options: Vec<ChoiceOption> = tags.into_iter().map(ChoiceOption::Tag).collect();
    if options.is_empty() {
        return None;
    }
    Some((
        chooser,
        ChoiceSpec {
            kind: ChoiceKind::PickTag,
            prompt,
            options,
            min: 1,
            max: 1,
            allow_cancel: cancel,
        },
    ))
}

fn player_ask(
    chooser: PlayerId,
    prompt: &'static str,
    players: Vec<PlayerId>,
    max: usize,
    cancel: bool,
) -> Option<(PlayerId, ChoiceSpec)> {
    let options: Vec<ChoiceOption> = players.into_iter().map(ChoiceOption::Player).collect();
    if options.is_empty() {
        return None;
    }
    let capped_max = max.clamp(1, options.len());
    Some((
        chooser,
        ChoiceSpec {
            kind: ChoiceKind::SelectPlayers,
            prompt,
            options,
            min: 1,
            max: capped_max,
            allow_cancel: cancel,
        },
    ))
}

/// 选牌：`count` 张（`exact` 时恰好 `count` 张，否则 1..=count 张）。
fn card_ask(
    chooser: PlayerId,
    prompt: &'static str,
    cards: Vec<CardId>,
    count: usize,
    exact: bool,
    cancel: bool,
) -> Option<(PlayerId, ChoiceSpec)> {
    let options: Vec<ChoiceOption> = cards.into_iter().map(ChoiceOption::Card).collect();
    if options.is_empty() {
        return None;
    }
    let max = count.clamp(1, options.len());
    Some((
        chooser,
        ChoiceSpec {
            kind: ChoiceKind::SelectCards,
            prompt,
            options,
            min: if exact { max } else { 1 },
            max,
            allow_cancel: cancel,
        },
    ))
}

/// 四种花色各一张（大业炎的弃牌条件）。
pub(super) fn has_four_suits(core: &GameStateCore, catalog: &Catalog, p: PlayerId) -> bool {
    let mask = core
        .player(p)
        .hand
        .iter()
        .fold(0u8, |m, &c| m | suit_bit(catalog.cards[c as usize].suit));
    mask == 0b1111
}

fn suit_bit(s: Suit) -> u8 {
    match s {
        Suit::Spade => 1,
        Suit::Heart => 2,
        Suit::Club => 4,
        Suit::Diamond => 8,
    }
}

fn build_ask(
    ask_id: u32,
    owner: PlayerId,
    target: Option<PlayerId>,
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<(PlayerId, ChoiceSpec)> {
    let t = target.unwrap_or(owner);
    match ask_id {
        ASK_JINGXIE_CARD => card_ask(
            owner,
            "th_jingxie:choose_card",
            jingxie_candidates(core, catalog, owner, target == Some(owner)),
            1,
            false,
            true,
        ),
        ASK_QIAOSI_COUNT => tag_ask(owner, "th_qiaosi:choose_count", 1..=3, true),
        ASK_QIAOSI_MODE => tag_ask(owner, "th_qiaosi:choose_mode", 0..=1, false),
        ASK_QIAOSI_TARGET => player_ask(
            owner,
            "th_qiaosi:choose_target",
            alive_others(core, owner),
            1,
            false,
        ),
        ASK_TIANSUAN_LOT => tag_ask(owner, "th_tiansuan:choose_lot", 0..=4, true),
        ASK_TIANSUAN_TARGET => player_ask(
            owner,
            "th_tiansuan:choose_target",
            alive_all(core, owner),
            1,
            false,
        ),
        ASK_TIANSUAN_GAIN => card_ask(
            owner,
            "th_tiansuan:gain_card",
            tiansuan_gain_pool(core, owner, t),
            1,
            true,
            false,
        ),
        ASK_GUSHE_TARGETS => player_ask(
            owner,
            "th_gushe:choose_targets",
            l28_batch_b_gushe::gushe_targets(core, owner),
            3,
            true,
        ),
        ASK_GUSHE_OWNER_CARD => card_ask(
            owner,
            "th_gushe:pindian_card",
            core.player(owner).hand.clone(),
            1,
            true,
            false,
        ),
        ASK_GUSHE_TARGET_CARD => card_ask(
            t,
            "th_gushe:target_pindian_card",
            core.player(t).hand.clone(),
            1,
            true,
            false,
        ),
        // 「弃置一张牌，或令王朗摸一张牌」：取消 = 令其摸牌。
        ASK_GUSHE_TARGET_DISCARD => card_ask(
            t,
            "th_gushe:target_discard",
            hand_and_equips(core, t),
            1,
            true,
            true,
        ),
        // 「弃置一张牌，或摸一张牌」：取消 = 摸牌。
        ASK_GUSHE_OWNER_DISCARD => card_ask(
            owner,
            "th_gushe:owner_discard",
            hand_and_equips(core, owner),
            1,
            true,
            true,
        ),
        ASK_REJICI_KILLER_DISCARD => card_ask(
            t,
            "rejici:killer_discard",
            hand_and_equips(core, t),
            l28_batch_b_gushe::rejici_discard_count(core, owner) as usize,
            true,
            false,
        ),
        ASK_GEYUAN_MAGICA_TARGETS => player_ask(
            owner,
            "th_geyuan:magica_targets",
            alive_all(core, owner),
            3,
            false,
        ),
        ASK_GEYUAN_MAGICA_DISCARD => card_ask(
            t,
            "th_geyuan:magica_discard",
            hand_and_equips(core, t),
            4,
            true,
            false,
        ),
        ASK_YEYAN_MODE => {
            let great = has_four_suits(core, catalog, owner);
            tag_ask(
                owner,
                "th_yeyan:choose_mode",
                (0..=1u8).filter(|&m| m == 0 || great),
                true,
            )
        }
        ASK_YEYAN_TARGETS => {
            let great = flag(core, owner, "th_yeyan_mode") == 1;
            player_ask(
                owner,
                "th_yeyan:choose_targets",
                alive_all(core, owner),
                if great { 2 } else { 3 },
                true,
            )
        }
        ASK_YEYAN_DISCARD => card_ask(
            owner,
            "th_yeyan:discard_cards",
            core.player(owner).hand.clone(),
            4,
            true,
            true,
        ),
        ASK_QINYIN_MODE => tag_ask(owner, "qinyin:choose_mode", 0..=1, true),
        _ => None,
    }
}

pub(super) fn wait_ask(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    stage: HostStage,
    ask_id: u32,
) -> Flow {
    set_flag(ctx.core, m.owner, "l28_ask_id", ask_id);
    if let Some((observer, spec)) = build_ask(ask_id, m.owner, m.target, ctx.core, ctx.catalog) {
        m.stage = stage;
        Flow::WaitChoice { observer, spec }
    } else {
        Flow::Done
    }
}

pub(in crate::content::host) fn ask_l28_batch_b(
    _stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    _targets: &[PlayerId],
    _card: Option<CardId>,
    _cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    let ask_id = flag(core, owner, "l28_ask_id");
    let (observer, spec) = build_ask(ask_id, owner, target, core, catalog)?;
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

/// 琴音：弃牌阶段内弃牌只计数；阶段结束且本阶段弃置 ≥2 张时选择全体回复或失去 1 点体力。
pub(super) fn step_qinyin(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act if !m.cards.is_empty() => {
            let stamp = turn_stamp(ctx.core);
            let prev = if flag(ctx.core, owner, "qinyin_discard_stamp") == stamp {
                flag(ctx.core, owner, "qinyin_discard_count")
            } else {
                0
            };
            set_flag(ctx.core, owner, "qinyin_discard_stamp", stamp);
            set_flag(ctx.core, owner, "qinyin_discard_count", prev + m.cards.len() as u32);
            Flow::Done
        }
        HostStage::Act => {
            set_flag(ctx.core, owner, "qinyin_discard_count", 0);
            wait_ask(m, ctx, HostStage::SelectNature, ASK_QINYIN_MODE)
        }
        HostStage::SelectNature => {
            let ans = ctx.answer().clone();
            let Some(ChoiceOption::Tag(mode)) = ans.first().copied() else {
                return Flow::Done;
            };
            set_flag(ctx.core, owner, "qinyin_triggered", 1);
            set_flag(ctx.core, owner, "qinyin_mode", u32::from(mode) + 1);
            for p in alive_all(ctx.core, owner) {
                if mode == 0 {
                    if ctx.core.player(p).hp < ctx.core.player(p).max_hp as i32 {
                        ctx.recover(p, 1, None);
                    }
                } else {
                    ctx.emit(EventData::LoseHp {
                        player: p,
                        amount: 1,
                    });
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

/// 业炎（限定技，`extension.js:8636-8935`）：对至多三名角色分配共计至多 3 点火焰伤害；
/// 若有角色分配到 ≥2 点，须先弃置四张花色各不相同的手牌并失去 3 点体力。
/// 小业炎（mode 0）：1~3 名角色各 1 点；大业炎（mode 1）：1 名 3 点，或首选 2 点、次选 1 点。
pub(super) fn step_th_yeyan(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if flag(ctx.core, owner, "th_yeyan_used") > 0 {
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::SelectNature, ASK_YEYAN_MODE)
        }
        HostStage::SelectNature => {
            let ans = ctx.answer().clone();
            let Some(ChoiceOption::Tag(mode)) = ans.first().copied() else {
                return Flow::Done;
            };
            set_flag(ctx.core, owner, "th_yeyan_mode", u32::from(mode));
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_YEYAN_TARGETS)
        }
        HostStage::TargetChoice => {
            m.targets = ctx
                .answer()
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Player(p) => Some(p),
                    _ => None,
                })
                .collect();
            if flag(ctx.core, owner, "th_yeyan_mode") == 1 {
                return wait_ask(m, ctx, HostStage::SelectCard, ASK_YEYAN_DISCARD);
            }
            resolve_yeyan(m, ctx, &[])
        }
        HostStage::SelectCard => {
            let cards: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Card(c) => Some(c),
                    _ => None,
                })
                .collect();
            let mask = cards
                .iter()
                .fold(0u8, |acc, &c| acc | suit_bit(ctx.catalog.cards[c as usize].suit));
            if cards.len() != 4 || mask != 0b1111 {
                // 花色不各不相同：重新选择（不产生任何效果）。
                return wait_ask(m, ctx, HostStage::SelectCard, ASK_YEYAN_DISCARD);
            }
            resolve_yeyan(m, ctx, &cards)
        }
        _ => Flow::Done,
    }
}

fn resolve_yeyan(m: &mut HostMachine, ctx: &mut Ctx<'_>, discard: &[CardId]) -> Flow {
    let owner = m.owner;
    let great = !discard.is_empty();
    let flames: Vec<(PlayerId, i32)> = m
        .targets
        .iter()
        .enumerate()
        .map(|(i, &t)| {
            let n = match (great, m.targets.len(), i) {
                (false, _, _) => 1,
                (true, 1, _) => 3,
                (true, _, 0) => 2,
                _ => 1,
            };
            (t, n)
        })
        .collect();
    if great {
        discard_cards(ctx, owner, discard);
        ctx.emit(EventData::LoseHp {
            player: owner,
            amount: 3,
        });
    }
    set_flag(ctx.core, owner, "th_yeyan_used", 1);
    set_flag(ctx.core, owner, "th_yeyan_triggered", 1);
    set_flag(ctx.core, owner, "th_yeyan_fire_damage", if great { 3 } else { 1 });
    // `targets.sortBySeat()`：按座次（自业炎角色起）依次结算。
    for p in ctx.core.seat_order_from(owner) {
        if let Some(&(t, n)) = flames.iter().find(|(t, _)| *t == p) {
            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                Some(owner),
                t,
                n,
                DamageNature::Fire,
                None,
            )));
        }
    }
    Flow::Done
}
