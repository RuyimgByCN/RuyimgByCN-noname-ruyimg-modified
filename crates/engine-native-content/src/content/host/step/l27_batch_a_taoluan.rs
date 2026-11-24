//! `taoluan`（张让·滔乱，`yijiang.js:5073-5247`）：出牌阶段/濒死转化、真实 `CardUsed` 结算、`taoluan:choose_targets`、严格异类别交牌与罚血失能。

use super::super::l11_npc_b::helpers::{
    card_type_group, card_usable_on_target_with_card, is_weapon_card, pile_cards,
};
use super::*;
use crate::catalog::{CardKind, Catalog};
use crate::choice::ChoiceOption;
use crate::event::{DamageNature, EventData};
use crate::ids::{CardId, EventId, PlayerId, SkillDefId};
use crate::machine::{card_use_as, Ctx, Flow};
use crate::state::GameStateCore;

pub(super) fn taoluan_hes_cards(core: &GameStateCore, p: PlayerId) -> Vec<CardId> {
    let mut out = hand_and_equips(core, p);
    out.extend(pile_cards(core, p, "gnjinfan"));
    out
}

pub(super) fn taoluan_kind_by_bit(bit: usize) -> CardKind {
    match bit {
        14 => CardKind::Shan,
        15 => CardKind::Nullify,
        _ => TAOLUAN_KINDS[bit.min(TAOLUAN_KINDS.len() - 1)],
    }
}

pub(super) fn taoluan_diff_type_cards(
    core: &GameStateCore,
    catalog: &Catalog,
    helper: PlayerId,
    virt_kind: CardKind,
) -> Vec<CardId> {
    let virt_group = card_type_group(virt_kind);
    hand_and_equips(core, helper)
        .into_iter()
        .filter(|&c| {
            card_type_group(crate::kernel::effective_card_kind_for(core, catalog, helper, c))
                != virt_group
        })
        .collect()
}

pub(super) fn taoluan_needs_target_ask(kind: CardKind) -> bool {
    matches!(
        kind,
        CardKind::Sha
            | CardKind::Dismantle
            | CardKind::Snatch
            | CardKind::Duel
            | CardKind::Chain
            | CardKind::Linghunzhihuo
            | CardKind::Jiedao
    )
}

pub(super) fn taoluan_targets_for(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    card: CardId,
    kind: CardKind,
) -> Vec<PlayerId> {
    let kongcheng_def = catalog.skill_by_name(crate::content::standard::KONG_CHENG);
    let qianxun_def = catalog.skill_by_name(crate::content::standard::QIAN_XUN);
    let pool = if kind == CardKind::Chain {
        alive_all(core, owner)
    } else {
        alive_others(core, owner)
    };
    pool.into_iter()
        .filter(|&t| {
            if (kind == CardKind::Duel || kind.is_sha_like())
                && kongcheng_def.is_some_and(|d| core.has_skill(t, d))
                && core.player(t).hand.is_empty()
            {
                return false;
            }
            if kind == CardKind::Snatch && qianxun_def.is_some_and(|d| core.has_skill(t, d)) {
                return false;
            }
            card_usable_on_target_with_card(core, catalog, owner, t, card, kind, false)
        })
        .collect()
}

pub(super) fn taoluan_jiedao_pairs(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<(PlayerId, PlayerId)> {
    let mut pairs = Vec::new();
    for h in alive_others(core, owner) {
        if !core.player(h).equips.iter().any(|&c| is_weapon_card(catalog, core, c)) {
            continue;
        }
        if crate::rvm_bridge::modifiers::eval_card_target_canceled(
            core, catalog, owner, h, 0, CardKind::Jiedao, DamageNature::Normal,
        ) {
            continue;
        }
        for v in alive_all(core, owner) {
            if v != h && core.in_attack_range(catalog, h, v) {
                pairs.push((h, v));
            }
        }
    }
    pairs
}

pub(super) fn is_taoluan_kind_usable(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    card: CardId,
    kind: CardKind,
) -> bool {
    let nature = if kind == CardKind::Linghunzhihuo {
        DamageNature::Fire
    } else {
        DamageNature::Normal
    };
    if crate::rvm_bridge::modifiers::eval_card_use_canceled(
        core, catalog, owner, card, kind, nature,
    ) {
        return false;
    }
    match kind {
        CardKind::Tao => core.player(owner).hp < (core.player(owner).max_hp as i32),
        CardKind::Jiu => !core.turn.jiu_active,
        CardKind::Wuzhong => true,
        CardKind::Sha => {
            let unlimited = crate::rvm_bridge::modifiers::eval_card_use_unlimited(
                core,
                catalog,
                owner,
                Some(card),
                CardKind::Sha,
                DamageNature::Normal,
            ) || crate::rvm_bridge::modifiers::eval_sha_no_use_count(
                core,
                catalog,
                owner,
                Some(card),
                CardKind::Sha,
            );
            let used = if core.turn.sha_used_play_phase {
                flag(core, owner, "sha_used_count").max(1)
            } else {
                0
            };
            let (sha_limit, rvm_unlim) = crate::rvm_bridge::modifiers::eval_sha_use_limit(
                core,
                catalog,
                owner,
                Some(card),
                CardKind::Sha,
                DamageNature::Normal,
                1,
            );
            (unlimited || rvm_unlim || used < sha_limit)
                && !taoluan_targets_for(core, catalog, owner, card, CardKind::Sha).is_empty()
        }
        CardKind::Taoyuan | CardKind::Wugu => alive_all(core, owner).into_iter().any(|t| {
            !crate::rvm_bridge::modifiers::eval_card_target_canceled(
                core, catalog, owner, t, card, kind, DamageNature::Normal,
            )
        }),
        CardKind::SavageAttack | CardKind::ArcheryAttack => {
            alive_others(core, owner).into_iter().any(|t| {
                !crate::rvm_bridge::modifiers::eval_card_target_canceled(
                    core, catalog, owner, t, card, kind, DamageNature::Normal,
                )
            })
        }
        CardKind::Jiedao => !taoluan_jiedao_pairs(core, catalog, owner).is_empty(),
        CardKind::Chain => !alive_all(core, owner).is_empty(),
        _ => !taoluan_targets_for(core, catalog, owner, card, kind).is_empty(),
    }
}

pub(super) fn taoluan_legal_kind_tags(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    card: CardId,
) -> Vec<u8> {
    let mask = flag(core, owner, "taoluan_used_mask");
    TAOLUAN_KINDS
        .iter()
        .enumerate()
        .filter_map(|(i, &k)| {
            ((mask & (1u32 << i)) == 0 && is_taoluan_kind_usable(core, catalog, owner, card, k))
                .then_some(i as u8)
        })
        .collect()
}

pub(in crate::content::host) fn taoluan_dying_respond_options(
    core: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    need: CardKind,
    is_use: bool,
    out: &mut Vec<ChoiceOption>,
) {
    let bit_idx = match need {
        CardKind::Tao => 1,
        CardKind::Jiu => 2,
        CardKind::Shan => 14,
        CardKind::Nullify => 15,
        _ => return,
    };
    if !is_use
        || !core.player(player).alive
        || flag(core, player, "taoluan_disabled_turn") > 0
        || catalog.is_tieji_sealed(core, player)
        || catalog.is_chanyuan_sealed(core, player)
        || taoluan_hes_cards(core, player).is_empty()
    {
        return;
    }
    if (flag(core, player, "taoluan_used_mask") & (1u32 << bit_idx)) != 0 {
        return;
    }
    if let Some(def) = catalog.skill_by_name("taoluan") {
        if core.has_skill(player, def) && !out.contains(&ChoiceOption::Skill(def)) {
            out.push(ChoiceOption::Skill(def));
        }
    }
}

pub(in crate::content::host) fn try_activate_taoluan_dying(
    ctx: &mut Ctx<'_>,
    player: PlayerId,
    def: SkillDefId,
    need: CardKind,
    fid: EventId,
    is_use: bool,
) -> Option<bool> {
    let mode = match need {
        CardKind::Tao => 101,
        CardKind::Jiu => 102,
        CardKind::Shan => 103,
        CardKind::Nullify => 104,
        _ => return None,
    };
    if !is_use
        || ctx.catalog.skill_name(def) != "taoluan"
        || flag(ctx.core, player, "taoluan_disabled_turn") > 0
    {
        return None;
    }
    set_flag(ctx.core, player, "u01_respond_fid", fid);
    ctx.res.parks.push(crate::resolution::Park {
        resume_depth: ctx.res.stack.len(),
        slot: crate::resolution::Slot::Trigger { frame: fid },
        awaiting_answer: false,
        machine: Box::new(HostMachine {
            def_key: "taoluan".to_string(),
            behavior: HostBehavior::L27BeautifyBatchA,
            owner: player,
            target: None,
            card: Some(mode),
            cards: Vec::new(),
            targets: Vec::new(),
            stage: HostStage::Act,
        }),
    });
    Some(false)
}

pub(crate) fn on_card_use_after_taoluan(ctx: &mut Ctx<'_>, u: &crate::event::CardUseData) {
    let post_plus1 = flag(ctx.core, u.player, "l27_taoluan_post_virt");
    if post_plus1 == 0 {
        return;
    }
    set_flag(ctx.core, u.player, "l27_taoluan_post_virt", 0);
    if !ctx.core.player(u.player).alive || alive_others(ctx.core, u.player).is_empty() {
        return;
    }
    ctx.res.parks.push(crate::resolution::Park {
        resume_depth: ctx.res.stack.len(),
        slot: crate::resolution::Slot::Intrinsic {
            frame: crate::resolution::NO_FRAME,
        },
        awaiting_answer: false,
        machine: Box::new(HostMachine {
            def_key: "taoluan".to_string(),
            behavior: HostBehavior::L27BeautifyBatchA,
            owner: u.player,
            target: None,
            card: Some(999),
            cards: vec![post_plus1 - 1],
            targets: Vec::new(),
            stage: HostStage::Act,
        }),
    });
}

pub(super) fn step_taoluan(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if m.card == Some(16) {
                set_flag(ctx.core, owner, "taoluan_disabled_turn", 0);
                return Flow::Done;
            }
            if m.card == Some(999) {
                return wait_ask(m, ctx, HostStage::TargetChoice, ASK_TAOLUAN_HELPER);
            }
            wait_ask(m, ctx, HostStage::SelectCard, ASK_TAOLUAN_CARD)
        }
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                return finish_respond_if_dying(m, ctx, false);
            };
            if matches!(m.card, Some(101..=104)) {
                let bit = match m.card {
                    Some(102) => 2usize,
                    Some(103) => 14usize,
                    Some(104) => 15usize,
                    _ => 1usize,
                };
                let mask = flag(ctx.core, owner, "taoluan_used_mask") | (1u32 << bit);
                set_flag(ctx.core, owner, "taoluan_used_mask", mask);
                set_flag(ctx.core, owner, "taoluan_last_kind", (bit as u32) + 1);
                set_flag(ctx.core, owner, "l27_taoluan_post_virt", (bit as u32) + 1);
                set_flag(ctx.core, owner, "u01_respond_card_override", c + 1);
                set_flag(ctx.core, owner, "u01_respond_emit_use", 1);
                set_flag(ctx.core, owner, "l27_ask_id", 0);
                return finish_respond_if_dying(m, ctx, true);
            }
            m.card = Some(c);
            wait_ask(m, ctx, HostStage::SelectNature, ASK_TAOLUAN_VIRTUAL)
        }
        HostStage::SelectNature => {
            let Some(ChoiceOption::Tag(tag)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            let bit = (tag as usize).min(TAOLUAN_KINDS.len() - 1);
            let kind = TAOLUAN_KINDS[bit];
            m.cards = vec![bit as u32];
            if taoluan_needs_target_ask(kind) {
                return wait_ask(m, ctx, HostStage::CompareTargetPick, ASK_TAOLUAN_TARGETS);
            }
            let cid = m.card.unwrap_or(0);
            let ts = match kind {
                CardKind::Taoyuan | CardKind::Wugu => alive_all(ctx.core, owner)
                    .into_iter()
                    .filter(|&t| {
                        !crate::rvm_bridge::modifiers::eval_card_target_canceled(
                            ctx.core, ctx.catalog, owner, t, cid, kind, DamageNature::Normal,
                        )
                    })
                    .collect(),
                CardKind::SavageAttack | CardKind::ArcheryAttack => alive_others(ctx.core, owner)
                    .into_iter()
                    .filter(|&t| {
                        !crate::rvm_bridge::modifiers::eval_card_target_canceled(
                            ctx.core, ctx.catalog, owner, t, cid, kind, DamageNature::Normal,
                        )
                    })
                    .collect(),
                _ => Vec::new(),
            };
            let ts = if ts.is_empty() && matches!(kind, CardKind::Tao | CardKind::Jiu | CardKind::Wuzhong) {
                vec![owner]
            } else {
                ts
            };
            emit_taoluan_card(m, ctx, bit, ts)
        }
        HostStage::CompareTargetPick => {
            let bit = (m.cards.first().copied().unwrap_or(0) as usize).min(TAOLUAN_KINDS.len() - 1);
            let kind = TAOLUAN_KINDS[bit];
            let picked: Vec<PlayerId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Player(p) => Some(*p),
                    _ => None,
                })
                .collect();
            if picked.is_empty() {
                set_flag(ctx.core, owner, "l27_ask_id", 0);
                return Flow::Done;
            }
            if kind == CardKind::Jiedao && m.targets.is_empty() {
                m.targets = vec![picked[0]];
                return wait_ask(m, ctx, HostStage::CompareTargetPick, ASK_TAOLUAN_TARGETS);
            }
            let ts = if kind == CardKind::Jiedao {
                vec![m.targets[0], picked[0]]
            } else {
                picked
            };
            m.targets.clear();
            emit_taoluan_card(m, ctx, bit, ts)
        }
        HostStage::DrawSettle => {
            if !ctx.core.player(owner).alive || alive_others(ctx.core, owner).is_empty() {
                set_flag(ctx.core, owner, "l27_ask_id", 0);
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_TAOLUAN_HELPER)
        }
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(helper)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            m.target = Some(helper);
            let bit = m.cards.first().copied().unwrap_or(0) as usize;
            let virt_kind = taoluan_kind_by_bit(bit);
            if taoluan_diff_type_cards(ctx.core, ctx.catalog, helper, virt_kind).is_empty() {
                set_flag(ctx.core, owner, "taoluan_disabled_turn", 1);
                ctx.emit(EventData::LoseHp {
                    player: owner,
                    amount: 1,
                });
                set_flag(ctx.core, owner, "l27_ask_id", 0);
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::CardsAsk, ASK_TAOLUAN_HELPER_GIVE)
        }
        HostStage::CardsAsk => {
            let helper = m.target.unwrap_or(owner);
            match ctx.answer().first().copied() {
                Some(ChoiceOption::Card(c)) => {
                    move_cards_to_hand(ctx, helper, owner, &[c]);
                }
                _ => {
                    set_flag(ctx.core, owner, "taoluan_disabled_turn", 1);
                    ctx.emit(EventData::LoseHp {
                        player: owner,
                        amount: 1,
                    });
                }
            }
            set_flag(ctx.core, owner, "l27_ask_id", 0);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn finish_respond_if_dying(m: &HostMachine, ctx: &mut Ctx<'_>, ok: bool) -> Flow {
    if matches!(m.card, Some(101..=104)) {
        let fid = flag(ctx.core, m.owner, "u01_respond_fid");
        set_flag(ctx.core, m.owner, "u01_respond_fid", 0);
        set_flag(
            ctx.core,
            m.owner,
            &format!("u01_respond_result:{fid}"),
            u32::from(ok),
        );
    }
    set_flag(ctx.core, m.owner, "l27_ask_id", 0);
    Flow::Done
}

fn emit_taoluan_card(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    bit: usize,
    targets: Vec<PlayerId>,
) -> Flow {
    let owner = m.owner;
    let kind = TAOLUAN_KINDS[bit];
    let mask = flag(ctx.core, owner, "taoluan_used_mask") | (1u32 << bit);
    set_flag(ctx.core, owner, "taoluan_used_mask", mask);
    set_flag(ctx.core, owner, "taoluan_last_kind", (bit as u32) + 1);
    set_flag(ctx.core, owner, "l27_ask_id", 0);
    if let Some(c) = m.card {
        m.stage = HostStage::DrawSettle;
        if kind == CardKind::Linghunzhihuo {
            set_flag(ctx.core, owner, &format!("l11c_huogong_use:{c}"), 1);
        }
        ctx.emit(EventData::CardUsed(card_use_as(c, owner, targets, Some(kind))));
        return Flow::Yield;
    }
    Flow::Done
}
