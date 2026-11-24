//! Active-phase and respond-window skills for `HostBehavior::L11VmFixC`.

use super::helpers::*;
use super::*;
use crate::catalog::{CardKind, Catalog, EquipSlot, Suit};
use crate::choice::ChoiceOption;
use crate::event::{DamageNature, EventData};
use crate::ids::{CardId, EventId, PlayerId, SkillDefId};
use crate::machine::{card_use_as, Ctx, Flow};
use crate::state::GameStateCore;

pub(super) fn has_phase_use_l11_vm_c(name: &str) -> bool {
    matches!(
        name,
        "huoji"
            | "longhun"
            | "qice"
            | "relianji"
            | "reqiangxi"
            | "rezhiheng"
            | "twjiange"
            | "twkujian"
            | "xiongyi"
    )
}

pub(super) fn activate_l11_vm_c(
    skill: &HostSkillBehavior,
    state: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Option<HostMachine> {
    if !state.player(owner).alive
        || !is_vm_c_unblocked(state, catalog, owner, false)
        || catalog.blocks_card_use_and_save(state, owner) && matches!(skill.name.as_str(), "huoji" | "longhun" | "qice" | "twjiange")
    {
        return None;
    }
    let pstamp = phase_stamp(state);
    let tstamp = turn_stamp(state);
    let can = match skill.name.as_str() {
        "huoji" => {
            !crate::rvm_bridge::modifiers::eval_card_use_canceled(
                state,
                catalog,
                owner,
                0,
                CardKind::Linghunzhihuo,
                DamageNature::Fire,
            ) && hes_cards(state, owner)
                .into_iter()
                .any(|c| is_red_suit(card_suit(state, catalog, owner, c)))
                && !single_targets_for_virtual_trick(state, catalog, owner, CardKind::Linghunzhihuo, false)
                    .is_empty()
        }
        "longhun" => {
            let need = longhun_need(state, owner);
            let hes = hes_cards(state, owner);
            let tao_ok = state.player(owner).hp < state.player(owner).max_hp as i32
                && hes.iter().filter(|&&c| card_matches_suit(state, catalog, owner, c, Suit::Heart)).count() >= need;
            let sha_ok = hes.iter().filter(|&&c| card_matches_suit(state, catalog, owner, c, Suit::Diamond)).count() >= need
                && can_use_sha_limit_for_card(state, catalog, owner, None, CardKind::FireSha, false)
                && !sha_targets_for_card(state, catalog, owner, None, CardKind::FireSha, false).is_empty();
            tao_ok || sha_ok
        }
        "qice" => qice_hand_usable(state, catalog, owner),
        "relianji" => {
            flag(state, owner, "l11c_relianji_phase") != pstamp
                && alive_others(state, owner).len() >= 2
        }
        "reqiangxi" => alive_others(state, owner).into_iter().any(|p| {
            state.in_attack_range(catalog, owner, p)
                && flag(state, p, &format!("l11c_reqiangxi_off:{owner}")) != tstamp
        }),
        "rezhiheng" => {
            flag(state, owner, "l11c_rezhiheng_phase") != pstamp && !he_cards(state, owner).is_empty()
        }
        "twjiange" => {
            flag(state, owner, "l11c_twjiange_turn") != tstamp
                && hes_cards(state, owner)
                    .into_iter()
                    .any(|c| !catalog.resolved_card_kind(state, c).is_basic_kind())
                && can_use_sha_limit_for_card(state, catalog, owner, None, CardKind::Sha, true)
                && !sha_targets_for_card(state, catalog, owner, None, CardKind::Sha, true).is_empty()
        }
        "twkujian" => {
            flag(state, owner, "l11c_twkujian_phase") != pstamp
                && !state.player(owner).hand.is_empty()
                && !alive_others(state, owner).is_empty()
        }
        "xiongyi" => flag(state, owner, "l11c_xiongyi_used") == 0,
        _ => false,
    };
    can.then(|| HostMachine {
        def_key: skill.name.clone(),
        behavior: HostBehavior::L11VmFixC,
        owner,
        target: None,
        card: None,
        cards: Vec::new(),
        targets: Vec::new(),
        stage: HostStage::Act,
    })
}

pub(super) fn step_active(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let pstamp = phase_stamp(ctx.core);
    let tstamp = turn_stamp(ctx.core);
    match m.def_key.as_str() {
        "huoji" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_HUOJI_CARD),
            HostStage::SelectCard => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                m.card = Some(c);
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_HUOJI_TARGET)
            }
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                if let Some(c) = m.card {
                    log_skill(ctx, owner, "huoji");
                    set_flag(ctx.core, owner, &format!("l11c_huogong_use:{c}"), 1);
                    ctx.emit(EventData::CardUsed(card_use_as(
                        c,
                        owner,
                        vec![t],
                        Some(CardKind::Linghunzhihuo),
                    )));
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "longhun" => match m.stage {
            HostStage::Act => {
                let need = longhun_need(ctx.core, owner);
                let hes = hes_cards(ctx.core, owner);
                let tao_ok = ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32
                    && hes.iter().filter(|&&c| card_matches_suit(ctx.core, ctx.catalog, owner, c, Suit::Heart)).count() >= need;
                let sha_ok = hes.iter().filter(|&&c| card_matches_suit(ctx.core, ctx.catalog, owner, c, Suit::Diamond)).count() >= need
                    && can_use_sha_limit_for_card(ctx.core, ctx.catalog, owner, None, CardKind::FireSha, false)
                    && !sha_targets_for_card(ctx.core, ctx.catalog, owner, None, CardKind::FireSha, false).is_empty();
                match (tao_ok, sha_ok) {
                    (true, true) => wait_ask(m, ctx, HostStage::SelectNature, ASK_LONGHUN_KIND),
                    (true, false) => {
                        m.cards = vec![0];
                        wait_ask(m, ctx, HostStage::CardsAsk, ASK_LONGHUN_CARDS)
                    }
                    (false, true) => {
                        m.cards = vec![1];
                        wait_ask(m, ctx, HostStage::CardsAsk, ASK_LONGHUN_CARDS)
                    }
                    _ => Flow::Done,
                }
            }
            HostStage::SelectNature => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Tag(tag)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                m.cards = vec![u32::from(tag)];
                wait_ask(m, ctx, HostStage::CardsAsk, ASK_LONGHUN_CARDS)
            }
            HostStage::CardsAsk => {
                clear_ask(ctx, owner);
                let chosen: Vec<CardId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    })
                    .collect();
                if chosen.len() < longhun_need(ctx.core, owner) {
                    return Flow::Done;
                }
                let kind_tag = m.cards.first().copied().unwrap_or(0);
                m.cards = chosen;
                if kind_tag == 0 {
                    log_skill(ctx, owner, "longhun");
                    if m.cards.len() == 1 {
                        let c = m.cards[0];
                        ctx.emit(EventData::CardUsed(card_use_as(c, owner, Vec::new(), Some(CardKind::Tao))));
                    } else {
                        let to_disc = m.cards.clone();
                        discard_hes_cards(ctx, owner, &to_disc);
                        ctx.emit(EventData::CardUsed(card_use_virtual(
                            owner,
                            Vec::new(),
                            DamageNature::Normal,
                            CardKind::Tao,
                        )));
                    }
                    Flow::Done
                } else {
                    wait_ask(m, ctx, HostStage::TargetChoice, ASK_LONGHUN_TARGET)
                }
            }
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                log_skill(ctx, owner, "longhun");
                if m.cards.len() == 1 {
                    let c = m.cards[0];
                    ctx.emit(EventData::CardUsed(card_use_as(c, owner, vec![t], Some(CardKind::FireSha))));
                } else {
                    let to_disc = m.cards.clone();
                    discard_hes_cards(ctx, owner, &to_disc);
                    ctx.emit(EventData::CardUsed(card_use_virtual(
                        owner,
                        vec![t],
                        DamageNature::Fire,
                        CardKind::FireSha,
                    )));
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "qice" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectNature, ASK_QICE_TRICK),
            HostStage::SelectNature => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Tag(tag)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                let idx = (tag as usize).min(QICE_KINDS.len() - 1);
                let kind = QICE_KINDS[idx];
                m.card = Some(idx as u32);
                match kind {
                    CardKind::Wuzhong | CardKind::Taoyuan | CardKind::Wugu => {
                        let ts = if kind == CardKind::Wuzhong {
                            Vec::new()
                        } else {
                            alive_all(ctx.core, owner)
                                .into_iter()
                                .filter(|&t| {
                                    !crate::rvm_bridge::modifiers::eval_card_target_canceled(
                                        ctx.core, ctx.catalog, owner, t, 0, kind, DamageNature::Normal,
                                    )
                                })
                                .collect()
                        };
                        finish_qice(ctx, owner, pstamp, kind, ts);
                        Flow::Done
                    }
                    CardKind::SavageAttack | CardKind::ArcheryAttack => {
                        let ts = alive_others(ctx.core, owner)
                            .into_iter()
                            .filter(|&t| {
                                !crate::rvm_bridge::modifiers::eval_card_target_canceled(
                                    ctx.core, ctx.catalog, owner, t, 0, kind, DamageNature::Normal,
                                )
                            })
                            .collect();
                        finish_qice(ctx, owner, pstamp, kind, ts);
                        Flow::Done
                    }
                    _ => wait_ask(m, ctx, HostStage::TargetChoice, ASK_QICE_TARGETS),
                }
            }
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                let idx = (m.card.unwrap_or(0) as usize).min(QICE_KINDS.len() - 1);
                let kind = QICE_KINDS[idx];
                let picked: Vec<PlayerId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    })
                    .collect();
                if picked.is_empty() {
                    return Flow::Done;
                }
                if kind == CardKind::Jiedao && m.targets.is_empty() {
                    m.targets = vec![picked[0]];
                    return wait_ask(m, ctx, HostStage::TargetChoice, ASK_QICE_TARGETS);
                }
                let final_ts = if kind == CardKind::Jiedao {
                    vec![m.targets[0], picked[0]]
                } else {
                    picked
                };
                finish_qice(ctx, owner, pstamp, kind, final_ts);
                Flow::Done
            }
            _ => Flow::Done,
        },
        "relianji" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_RELIANJI_TARGETS),
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                let ts: Vec<PlayerId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    })
                    .collect();
                if ts.len() != 2 {
                    return Flow::Done;
                }
                set_flag(ctx.core, owner, "l11c_relianji_phase", pstamp);
                log_skill(ctx, owner, "relianji");
                let t0 = ts[0];
                let t1 = ts[1];
                m.targets = vec![t0, t1];
                if crate::rvm_ops::has_enabled_equip_slot(ctx.core, t0, EquipSlot::Weapon) {
                    let eq = ctx
                        .core
                        .deck
                        .iter()
                        .rev()
                        .copied()
                        .find(|&c| is_weapon_card(ctx.catalog, ctx.core, c));
                    let Some(wcard) = eq else {
                        return Flow::Done;
                    };
                    let equipped = equip_card_onto_player(ctx, t0, wcard);
                    if !equipped {
                        return Flow::Done;
                    }
                    m.stage = HostStage::DrawSettle;
                    return Flow::Yield;
                }
                relianji_step2(ctx, owner, t0, t1);
                Flow::Done
            }
            HostStage::DrawSettle => {
                if m.targets.len() == 2 {
                    relianji_step2(ctx, owner, m.targets[0], m.targets[1]);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "reqiangxi" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_REQIANGXI_TARGET),
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                m.target = Some(t);
                let has_weapon = he_cards(ctx.core, owner)
                    .into_iter()
                    .any(|c| is_weapon_card(ctx.catalog, ctx.core, c));
                if has_weapon {
                    wait_ask(m, ctx, HostStage::SelectCard, ASK_REQIANGXI_WEAPON)
                } else {
                    set_flag(ctx.core, t, &format!("l11c_reqiangxi_off:{owner}"), tstamp);
                    log_skill(ctx, owner, "reqiangxi");
                    emit_lose_hp(ctx, owner, 1);
                    if ctx.core.player(t).alive {
                        emit_damage(ctx, Some(owner), t, 1, DamageNature::Normal, None);
                    }
                    Flow::Done
                }
            }
            HostStage::SelectCard => {
                clear_ask(ctx, owner);
                let t = m.target.unwrap_or(owner);
                set_flag(ctx.core, t, &format!("l11c_reqiangxi_off:{owner}"), tstamp);
                log_skill(ctx, owner, "reqiangxi");
                if let Some(ChoiceOption::Card(w)) = ctx.answer().first().copied() {
                    discard_player_cards(ctx, owner, &[w]);
                } else {
                    emit_lose_hp(ctx, owner, 1);
                }
                if ctx.core.player(t).alive {
                    emit_damage(ctx, Some(owner), t, 1, DamageNature::Normal, None);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "rezhiheng" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::CardsAsk, ASK_REZHIHENG_CARDS),
            HostStage::CardsAsk => {
                clear_ask(ctx, owner);
                let chosen: Vec<CardId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    })
                    .collect();
                if chosen.is_empty() {
                    return Flow::Done;
                }
                set_flag(ctx.core, owner, "l11c_rezhiheng_phase", pstamp);
                log_skill(ctx, owner, "rezhiheng");
                let hand = ctx.core.player(owner).hand.clone();
                let all_hand = !hand.is_empty() && hand.iter().all(|c| chosen.contains(c));
                let draw_n = chosen.len() as u32 + u32::from(all_hand);
                discard_player_cards(ctx, owner, &chosen);
                if ctx.core.player(owner).alive {
                    ctx.start_draw(owner, draw_n);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "twjiange" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_TWJIANGE_CARD),
            HostStage::SelectCard => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                m.card = Some(c);
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_TWJIANGE_TARGET)
            }
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                set_flag(ctx.core, owner, "l11c_twjiange_turn", tstamp);
                log_skill(ctx, owner, "twjiange");
                if ctx.core.turn.current != owner {
                    ctx.start_draw(owner, 1);
                }
                emit_sha_card_use(ctx, owner, m.card, vec![t], CardKind::Sha, true);
                Flow::Done
            }
            _ => Flow::Done,
        },
        "twkujian" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::CardsAsk, ASK_TWKUJIAN_CARDS),
            HostStage::CardsAsk => {
                clear_ask(ctx, owner);
                let chosen: Vec<CardId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    })
                    .collect();
                if chosen.is_empty() {
                    return Flow::Done;
                }
                m.cards = chosen;
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_TWKUJIAN_TARGET)
            }
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                set_flag(ctx.core, owner, "l11c_twkujian_phase", pstamp);
                log_skill(ctx, owner, "twkujian");
                let to_give = m.cards.clone();
                move_cards_to_hand(ctx, owner, t, &to_give);
                for c in to_give {
                    if ctx.core.player(t).hand.contains(&c) {
                        set_flag(ctx.core, t, &format!("l11c_twkujian_card:{c}"), u32::from(owner) + 1);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "xiongyi" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_XIONGYI_TARGETS),
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                let ts: Vec<PlayerId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    })
                    .collect();
                if ts.is_empty() {
                    return Flow::Done;
                }
                set_flag(ctx.core, owner, "l11c_xiongyi_used", 1);
                log_skill(ctx, owner, "xiongyi");
                let min_hp = alive_all(ctx.core, owner)
                    .into_iter()
                    .map(|p| ctx.core.player(p).hp)
                    .min()
                    .unwrap_or(i32::MAX);
                let should_recover = ctx.core.player(owner).hp == min_hp
                    && ctx.core.player(owner).hp < (ctx.core.player(owner).max_hp as i32);
                for p in ts {
                    if ctx.core.player(p).alive {
                        ctx.start_draw(p, 3);
                    }
                }
                if should_recover && ctx.core.player(owner).alive {
                    ctx.recover(owner, 1, None);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        _ => Flow::Done,
    }
}

fn finish_qice(ctx: &mut Ctx<'_>, owner: PlayerId, pstamp: u32, kind: CardKind, targets: Vec<PlayerId>) {
    set_flag(ctx.core, owner, "l11c_qice_phase", pstamp);
    log_skill(ctx, owner, "qice");
    let hs = ctx.core.player(owner).hand.clone();
    discard_player_cards(ctx, owner, &hs);
    if !ctx.core.player(owner).alive {
        return;
    }
    let nature = if kind == CardKind::Linghunzhihuo {
        set_flag(ctx.core, owner, super::super::l11_common::VIRTUAL_HUOGONG_FLAG, 1);
        DamageNature::Fire
    } else {
        DamageNature::Normal
    };
    ctx.emit(EventData::CardUsed(card_use_virtual(
        owner,
        targets,
        nature,
        kind,
    )));
}

fn relianji_step2(ctx: &mut Ctx<'_>, owner: PlayerId, t0: PlayerId, t1: PlayerId) {
    if !ctx.core.player(t0).alive || !ctx.core.player(t1).alive {
        return;
    }
    let candidates = [
        CardKind::SavageAttack,
        CardKind::ArcheryAttack,
        CardKind::Linghunzhihuo,
        CardKind::Duel,
        CardKind::Sha,
    ];
    let mut usable = Vec::new();
    for kind in candidates {
        if card_usable_on_target_with_card(ctx.core, ctx.catalog, t0, t1, 0, kind, true) {
            usable.push(kind);
        }
    }
    if usable.is_empty() {
        return;
    }
    let pick = usable[ctx.core.rng.gen_range(usable.len() as u32) as usize];
    let ts = match pick {
        CardKind::SavageAttack | CardKind::ArcheryAttack => alive_others(ctx.core, t0)
            .into_iter()
            .filter(|&p| {
                p != owner
                    && !crate::rvm_bridge::modifiers::eval_card_target_canceled(
                        ctx.core,
                        ctx.catalog,
                        t0,
                        p,
                        0,
                        pick,
                        DamageNature::Normal,
                    )
            })
            .collect(),
        _ => vec![t1],
    };
    if ts.is_empty() {
        return;
    }
    let nature = if pick == CardKind::Linghunzhihuo {
        set_flag(ctx.core, t0, super::super::l11_common::VIRTUAL_HUOGONG_FLAG, 1);
        DamageNature::Fire
    } else {
        DamageNature::Normal
    };
    ctx.emit(EventData::CardUsed(card_use_virtual(
        t0,
        ts,
        nature,
        pick,
    )));
}

pub(super) fn respond_skill_options_l11_vm_c(
    core: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    need: CardKind,
    _is_use: bool,
    out: &mut Vec<ChoiceOption>,
) {
    if !core.player(player).alive || !is_vm_c_unblocked(core, catalog, player, false) {
        return;
    }
    let hes = hes_cards(core, player);
    if has_vm_c_skill(core, catalog, player, "longhun") {
        let count = longhun_need(core, player);
        let suit_opt = match need {
            CardKind::Tao => Some(Suit::Heart),
            CardKind::Sha => Some(Suit::Diamond),
            CardKind::Nullify => Some(Suit::Spade),
            CardKind::Shan => Some(Suit::Club),
            _ => None,
        };
        if let Some(suit) = suit_opt {
            if hes.iter().filter(|&&c| card_matches_suit(core, catalog, player, c, suit)).count() >= count {
                if let Some(def) = catalog.skill_by_name("longhun") {
                    if !out.contains(&ChoiceOption::Skill(def)) {
                        out.push(ChoiceOption::Skill(def));
                    }
                }
            }
        }
    }
    if need == CardKind::Sha && has_vm_c_skill(core, catalog, player, "twjiange") {
        let tstamp = turn_stamp(core);
        if flag(core, player, "l11c_twjiange_turn") != tstamp
            && hes.iter().any(|&c| !catalog.resolved_card_kind(core, c).is_basic_kind())
        {
            if let Some(def) = catalog.skill_by_name("twjiange") {
                if !out.contains(&ChoiceOption::Skill(def)) {
                    out.push(ChoiceOption::Skill(def));
                }
            }
        }
    }
}

pub(super) fn try_activate_respond_skill_l11_vm_c(
    ctx: &mut Ctx<'_>,
    player: PlayerId,
    def: SkillDefId,
    need: CardKind,
    fid: EventId,
    is_use: bool,
) -> Option<bool> {
    let name = ctx.catalog.skill_name(def);
    if ctx.catalog.skills[def as usize].behavior.as_host_behavior() != Some(HostBehavior::L11VmFixC)
    {
        return None;
    }
    let key = match (name, need) {
        ("longhun", CardKind::Tao) => "respond:longhun:tao",
        ("longhun", CardKind::Sha) => "respond:longhun:sha",
        ("longhun", CardKind::Nullify) => "respond:longhun:nullify",
        ("longhun", CardKind::Shan) => "respond:longhun:shan",
        ("twjiange", CardKind::Sha) => "respond:twjiange:sha",
        _ => return None,
    };
    set_flag(ctx.core, player, "u01_respond_fid", fid);
    set_flag(ctx.core, player, "l11c_respond_is_use", u32::from(is_use));
    ctx.res.parks.push(crate::resolution::Park {
        resume_depth: ctx.res.stack.len(),
        slot: crate::resolution::Slot::Trigger { frame: fid },
        awaiting_answer: false,
        machine: Box::new(HostMachine {
            def_key: key.to_string(),
            behavior: HostBehavior::L11VmFixC,
            owner: player,
            target: None,
            card: Some(if need == CardKind::Shan { 3 } else if need == CardKind::Nullify { 2 } else if need == CardKind::Sha { 1 } else { 0 }),
            cards: Vec::new(),
            targets: Vec::new(),
            stage: HostStage::Act,
        }),
    });
    Some(false)
}

pub(super) fn step_respond(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let tstamp = turn_stamp(ctx.core);
    let is_use = flag(ctx.core, owner, "l11c_respond_is_use") > 0;
    match m.def_key.as_str() {
        "respond:longhun:tao"
        | "respond:longhun:sha"
        | "respond:longhun:nullify"
        | "respond:longhun:shan" => match m.stage {
            HostStage::Act => {
                let tag = m.card.unwrap_or(0);
                m.cards = vec![tag];
                wait_ask(m, ctx, HostStage::CardsAsk, ASK_LONGHUN_CARDS)
            }
            HostStage::CardsAsk => {
                clear_ask(ctx, owner);
                let chosen: Vec<CardId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    })
                    .collect();
                if chosen.len() < longhun_need(ctx.core, owner) {
                    finish_respond_result(ctx, owner, false);
                    return Flow::Done;
                }
                log_skill(ctx, owner, "longhun");
                if chosen.len() == 1 {
                    set_flag(ctx.core, owner, "u01_respond_card_override", chosen[0] + 1);
                } else {
                    ctx.core
                        .vm_aux
                        .flags
                        .remove(&(owner, "u01_respond_card_override".to_string()));
                    discard_hes_cards(ctx, owner, &chosen);
                }
                if is_use {
                    set_flag(ctx.core, owner, "u01_respond_emit_use", 1);
                }
                finish_respond_result(ctx, owner, true);
                Flow::Done
            }
            _ => Flow::Done,
        },
        "respond:twjiange:sha" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_TWJIANGE_CARD),
            HostStage::SelectCard => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                    finish_respond_result(ctx, owner, false);
                    return Flow::Done;
                };
                set_flag(ctx.core, owner, "l11c_twjiange_turn", tstamp);
                log_skill(ctx, owner, "twjiange");
                if ctx.core.turn.current != owner {
                    ctx.start_draw(owner, 1);
                }
                set_flag(ctx.core, owner, "u01_respond_card_override", c + 1);
                if is_use {
                    set_flag(ctx.core, owner, "u01_respond_emit_use", 1);
                }
                finish_respond_result(ctx, owner, true);
                Flow::Done
            }
            _ => Flow::Done,
        },
        _ => Flow::Done,
    }
}
