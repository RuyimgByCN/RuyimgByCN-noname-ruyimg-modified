//! L20 标记补充 Batch B 后半部分技能实现（`l20_batch_b_ext.rs`）：
//! 覆盖 `fz_new_longdan`、`fz_new_tieji`、`fz_liegong`、`fz_xinkuanggu`、`rekuanshi`、
//! `kuanshi`、`kuanshi2`、`rechuhai`；后续 `xinfu_lingren`、`sbtieji`、`sbduanliang`、`shouye`、`mbmowang` 见 `l20_batch_b_mouyi.rs`。

use super::*;
use crate::catalog::{CardKind, Catalog, QueryCtx};
use crate::choice::ChoiceOption;
use crate::event::{CardUseData, DamageNature, EventData, Phase};
use crate::ids::PlayerId;
use crate::machine::{Ctx, Flow};

pub(super) fn is_lingren_damage_card(catalog: &Catalog, u: &CardUseData) -> bool {
    let kind = u
        .use_as
        .or_else(|| u.card.map(|c| catalog.card_kind(c)))
        .unwrap_or(CardKind::Sha);
    let is_named_dmg = u.card.is_some_and(|c| {
        matches!(
            catalog.cards[c as usize].name,
            "huogong" | "shandian" | "juedou" | "nanman" | "wanjian" | "sha" | "huosha" | "leisha"
        )
    });
    is_named_dmg
        || kind.is_sha_like()
        || matches!(
            kind,
            CardKind::Duel
                | CardKind::SavageAttack
                | CardKind::ArcheryAttack
                | CardKind::HongShui
                | CardKind::ShanDian
                | CardKind::ShouLiJian
                | CardKind::Linghunzhihuo
                | CardKind::Liuxinghuoyu
                | CardKind::Shatang
                | CardKind::Yuansuhuimie
                | CardKind::Aozuzhilei
                | CardKind::Jingleishan
                | CardKind::Chiyuxi
        )
}

/// 凌人 +1 伤害只属于被猜中的那张牌：拥有者再用下一张牌时清除残留标记。
pub(super) fn lingren_bonus_pending(state: &crate::state::GameStateCore, owner: PlayerId) -> bool {
    (0..state.players.len() as PlayerId)
        .any(|p| flag(state, p, "xinfu_lingren_vuln_from") == u32::from(owner) + 1)
}

pub(super) fn filter_l20_batch_b_ext(
    name: &str,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    match name {
        "fz_new_longdan" => match event {
            EventData::CardResponded { player, card } => {
                if *player != owner || !ctx.catalog.card_kind(*card).is_sha_like() {
                    return false;
                }
                let is_converted = flag(ctx.state, owner, "responding_to_sha_from") > 0
                    || flag(ctx.state, owner, "event_view_as_skill") == 1_926_900_022
                    || flag(ctx.state, owner, "l20_longdan_converted") > 0;
                if !is_converted {
                    return false;
                }
                if has_shouyue_on_table(ctx.state, ctx.catalog) {
                    return true;
                }
                let src_enc = flag(ctx.state, owner, "responding_to_sha_from");
                let sha_src = if src_enc > 0 {
                    Some((src_enc - 1) as PlayerId)
                } else {
                    None
                };
                alive_others(ctx.state, owner).into_iter().any(|p| {
                    Some(p) != sha_src && ctx.state.player(p).hp < ctx.state.player(p).max_hp as i32
                })
            }
            EventData::CardUsed(u) => {
                if u.player != owner {
                    return false;
                }
                if is_sha_use(ctx.catalog, u)
                    && u.card.is_some_and(|c| ctx.catalog.card_kind(c) == CardKind::Shan)
                {
                    return true;
                }
                if u.use_as != Some(CardKind::Shan)
                    || !u.card.is_some_and(|c| ctx.catalog.card_kind(c).is_sha_like())
                {
                    return false;
                }
                if has_shouyue_on_table(ctx.state, ctx.catalog) {
                    return true;
                }
                let src_enc = flag(ctx.state, owner, "responding_to_sha_from");
                let sha_src = if src_enc > 0 {
                    Some((src_enc - 1) as PlayerId)
                } else {
                    None
                };
                alive_others(ctx.state, owner).into_iter().any(|p| {
                    Some(p) != sha_src && ctx.state.player(p).hp < ctx.state.player(p).max_hp as i32
                })
            }
            EventData::ShaDodged {
                source,
                target,
                card,
            } => {
                *source == owner
                    && card.is_some_and(|c| {
                        ctx.catalog.card_kind(c) == CardKind::Shan
                            || flag(ctx.state, owner, &format!("fz_new_longdan_converted_sha:{c}"))
                                > 0
                            || flag(ctx.state, owner, "l20_longdan_converted") > 0
                    })
                    && alive_others(ctx.state, owner)
                        .into_iter()
                        .any(|p| p != *target)
            }
            _ => false,
        },
        "fz_new_tieji" => match event {
            EventData::CardUsed(u) => {
                u.player == owner && is_sha_use(ctx.catalog, u) && !u.targets.is_empty()
            }
            _ => false,
        },
        "fz_liegong" => match event {
            EventData::CardUsed(u) => {
                if u.player != owner || !is_sha_use(ctx.catalog, u) || u.targets.is_empty() {
                    return false;
                }
                let hp = ctx.state.player(owner).hp;
                let atk_range = ctx.state.attack_range(ctx.catalog, owner) as usize;
                u.targets.iter().any(|&t| {
                    let hand_len = ctx.state.player(t).hand.len();
                    hand_len as i32 >= hp || hand_len <= atk_range
                })
            }
            _ => false,
        },
        "fz_xinkuanggu" => match event {
            EventData::Damage(d) => {
                d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && (d.target == owner
                        || ctx.state.computed_distance(ctx.catalog, owner, d.target) <= 1)
            }
            _ => false,
        },
        "rekuanshi" => match event {
            EventData::Phase {
                player,
                phase: Phase::End,
                begin: true,
            } => *player == owner && !alive_all(ctx.state, owner).is_empty(),
            EventData::TurnBegin { .. } => (0..ctx.state.players.len() as PlayerId)
                .any(|p| flag(ctx.state, p, "rekuanshi_turn_dmg") > 0),
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } => *player == owner && flag(ctx.state, owner, "rekuanshi_target") > 0,
            EventData::Damage(d) => {
                !d.prevented
                    && d.amount > 0
                    && flag(ctx.state, owner, "rekuanshi_target") == u32::from(d.target) + 1
            }
            _ => false,
        },
        "kuanshi" => match event {
            EventData::Phase {
                player,
                phase: Phase::End,
                begin: true,
            } => *player == owner && !alive_all(ctx.state, owner).is_empty(),
            EventData::Phase {
                player,
                phase: Phase::Prepare | Phase::Judge,
                begin: true,
            } => {
                *player == owner
                    && (flag(ctx.state, owner, "kuanshi_target") > 0
                        || flag(ctx.state, owner, "kuanshi_skip_draw") > 0)
            }
            EventData::Death { player, .. } => {
                *player == owner && flag(ctx.state, owner, "kuanshi_target") > 0
            }
            EventData::Damage(d) => {
                !d.prevented
                    && d.amount > 1
                    && d.settlement.committed_damage.is_none()
                    && flag(ctx.state, owner, "kuanshi_target") == u32::from(d.target) + 1
            }
            _ => false,
        },
        "kuanshi2" => match event {
            EventData::Damage(d) => {
                d.target == owner
                    && !d.prevented
                    && d.amount > 1
                    && d.settlement.committed_damage.is_none()
                    && flag(ctx.state, owner, "kuanshi_protected_by") > 0
            }
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            }
            | EventData::Death { player, .. } => {
                let prot = flag(ctx.state, owner, "kuanshi_protected_by");
                prot == 0
                    || prot == u32::from(*player) + 1
                    || flag(ctx.state, *player, "kuanshi_skip_draw") > 0
            }
            _ => false,
        },
        "rechuhai" => match event {
            EventData::Damage(d) => {
                d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && flag(ctx.state, owner, "rechuhai_damage_target") == u32::from(d.target) + 1
            }
            EventData::Equipped { player, .. } => {
                *player == owner
                    && ctx.state.player(owner).equips.len() > 2
                    && flag(ctx.state, owner, "rechuhai_resolved") == 0
            }
            EventData::Phase {
                player,
                phase: Phase::Play,
                begin: false,
            }
            | EventData::TurnBegin { player }
            | EventData::TurnEnded { player } => {
                *player == owner && flag(ctx.state, owner, "rechuhai_damage_target") > 0
            }
            _ => false,
        },
        "xinfu_lingren" => match event {
            EventData::CardUsed(u) => {
                u.player == owner
                    && ((is_lingren_damage_card(ctx.catalog, u)
                        && !u.targets.is_empty()
                        && flag(ctx.state, owner, "xinfu_lingren_used_turn") == 0)
                        || lingren_bonus_pending(ctx.state, owner))
            }
            EventData::Phase { begin: false, .. } | EventData::TurnEnded { .. } => {
                lingren_bonus_pending(ctx.state, owner)
            }
            EventData::TurnBegin { player }
            | EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } => {
                *player == owner
                    && (flag(ctx.state, owner, "xinfu_lingren_used_turn") > 0
                        || flag(ctx.state, owner, "xinfu_lingren_jianxiong") > 0
                        || flag(ctx.state, owner, "xinfu_lingren_xingshang") > 0
                        || flag(ctx.state, owner, "xinfu_lingren_dmg_bonus") > 0)
            }
            _ => false,
        },
        "sbtieji" => match event {
            EventData::CardUsed(u) => {
                u.player == owner
                    && is_sha_use(ctx.catalog, u)
                    && u.targets
                        .iter()
                        .any(|&t| t != owner && ctx.state.player(t).alive)
            }
            EventData::TurnBegin { .. } | EventData::TurnEnded { .. } => {
                (0..ctx.state.players.len() as PlayerId)
                    .any(|p| flag(ctx.state, p, "sbtieji_target_sealed") > 0)
            }
            _ => false,
        },
        "shouye" => match event {
            EventData::CardUsed(u) => {
                u.player != owner
                    && u.targets.len() == 1
                    && u.targets[0] == owner
                    && ctx.state.player(u.player).alive
                    && flag(ctx.state, owner, "shouye_used_turn") == 0
            }
            EventData::TurnBegin { .. } | EventData::TurnEnded { .. } => {
                flag(ctx.state, owner, "shouye_used_turn") > 0
            }
            _ => false,
        },
        "mbmowang" => match event {
            EventData::Death { player, .. } => {
                let has_danggu = flag(ctx.state, owner, "mbdanggu_empty") == 0
                    && (flag(ctx.state, owner, "mbdanggu_count") > 0
                        || flag(ctx.state, owner, "mbdanggu_init") == 0);
                *player == owner
                    && has_danggu
                    && flag(ctx.state, owner, "mbmowang_used") == 0
                    && ctx.state.player(owner).max_hp > 0
            }
            EventData::TurnBegin { player }
            | EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } => *player == owner && flag(ctx.state, owner, "mbmowang_resting") > 0,
            EventData::TurnEnded { player }
            | EventData::Phase {
                player,
                phase: Phase::End,
                begin: false,
            } => *player == owner && flag(ctx.state, owner, "mbmowang_doomed") > 0,
            _ => false,
        },
        "xinhongyan" | "xinfu_langxi" | "fz_gzpaoxiao" => {
            super::l20_batch_b_asks::filter_l20_extra_four(name, ctx, owner, event)
        }
        _ => false,
    }
}

pub(super) fn handle_fz_new_longdan(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if m.card == Some(3) {
                if let Some(&cid) = m.cards.first() {
                    set_flag(
                        ctx.core,
                        owner,
                        &format!("fz_new_longdan_converted_sha:{cid}"),
                        1,
                    );
                }
                set_flag(ctx.core, owner, "fz_new_longdan_converted", 1);
                if has_shouyue_on_table(ctx.core, ctx.catalog) {
                    draw_direct(ctx, owner, 1);
                    set_flag(ctx.core, owner, "fz_new_longdan_shouyue_drew", 1);
                }
                return Flow::Done;
            }
            if m.card == Some(1) {
                set_flag(ctx.core, owner, "fz_new_longdan_converted", 1);
                if has_shouyue_on_table(ctx.core, ctx.catalog) {
                    draw_direct(ctx, owner, 1);
                    set_flag(ctx.core, owner, "fz_new_longdan_shouyue_drew", 1);
                }
                let src_enc = flag(ctx.core, owner, "responding_to_sha_from");
                set_flag(ctx.core, owner, "responding_to_sha_from", 0);
                let sha_src = if src_enc > 0 {
                    Some((src_enc - 1) as PlayerId)
                } else {
                    None
                };
                m.targets = alive_others(ctx.core, owner)
                    .into_iter()
                    .filter(|&p| {
                        Some(p) != sha_src
                            && ctx.core.player(p).hp < ctx.core.player(p).max_hp as i32
                    })
                    .collect();
                if m.targets.is_empty() {
                    return Flow::Done;
                }
                set_flag(ctx.core, owner, "fz_new_longdan_is_shan", 1);
                wait_ask(m, ctx, ASK_FZ_NEW_LONGDAN_TARGET)
            } else {
                let sha_target = m.target;
                m.targets = alive_others(ctx.core, owner)
                    .into_iter()
                    .filter(|&p| Some(p) != sha_target)
                    .collect();
                if m.targets.is_empty() {
                    return Flow::Done;
                }
                set_flag(ctx.core, owner, "fz_new_longdan_is_shan", 0);
                wait_ask(m, ctx, ASK_FZ_NEW_LONGDAN_TARGET)
            }
        }
        HostStage::SelectCard => {
            let Some(t) = ctx.answer().iter().find_map(|o| match o {
                ChoiceOption::Player(p) => Some(*p),
                _ => None,
            }) else {
                return Flow::Done;
            };
            set_flag(ctx.core, owner, "fz_new_longdan_fired", 1);
            if flag(ctx.core, owner, "fz_new_longdan_is_shan") > 0 {
                if ctx.core.player(t).hp < ctx.core.player(t).max_hp as i32 {
                    ctx.recover(t, 1, None);
                }
            } else {
                set_flag(ctx.core, owner, "fz_new_longdan_in_damage", 1);
                emit_damage(ctx, Some(owner), t, 1, DamageNature::Normal);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn advance_tieji_target(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if !m.targets.is_empty() {
        m.targets.remove(0);
    }
    if let Some(&next_t) = m.targets.first() {
        m.target = Some(next_t);
        let sha_v = flag(ctx.core, owner, "fz_new_tieji_sha_cid");
        m.cards = if sha_v > 0 {
            vec![sha_v - 1]
        } else {
            Vec::new()
        };
        return wait_ask(m, ctx, ASK_FZ_NEW_TIEJI_CONFIRM);
    }
    Flow::Done
}

pub(super) fn handle_fz_new_tieji(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if m.card == Some(99) {
                let t = m.target.unwrap_or(owner);
                let sha_v = flag(ctx.core, owner, "fz_new_tieji_sha_cid");
                let sha_cid = if sha_v > 0 { Some(sha_v - 1) } else { None };
                let suit_val = flag(ctx.core, owner, "last_judge_suit");
                let jsuit = match suit_val {
                    2 => crate::catalog::Suit::Heart,
                    3 => crate::catalog::Suit::Club,
                    4 => crate::catalog::Suit::Diamond,
                    _ => crate::catalog::Suit::Spade,
                };
                set_flag(ctx.core, owner, "fz_new_tieji_judge_suit", suit_code(jsuit));
                let matching: Vec<_> = hand_and_equips(ctx.core, t)
                    .into_iter()
                    .filter(|&c| ctx.catalog.effective_card_suit(ctx.core, t, c) == jsuit)
                    .collect();
                if matching.is_empty() {
                    set_flag(ctx.core, owner, "fz_new_tieji_direct_hit", 1);
                    set_flag(ctx.core, t, "fz_new_tieji_direct_hit", 1);
                    mark_sha_unrespondable(ctx, owner, t, sha_cid);
                    return advance_tieji_target(m, ctx);
                }
                m.cards = matching;
                return wait_ask(m, ctx, ASK_FZ_NEW_TIEJI_TARGET_DISCARD);
            }
            set_flag(ctx.core, owner, "fz_new_tieji_direct_hit", 0);
            set_flag(
                ctx.core,
                owner,
                "fz_new_tieji_sha_cid",
                m.cards.first().map(|&c| c + 1).unwrap_or(0),
            );
            if m.targets.is_empty() {
                if let Some(t) = m.target {
                    m.targets.push(t);
                }
            }
            m.target = m.targets.first().copied();
            wait_ask(m, ctx, ASK_FZ_NEW_TIEJI_CONFIRM)
        }
        HostStage::SelectCard => {
            let ask_code = m.card.unwrap_or(0);
            let ans = ctx.answer();
            let t = m.target.unwrap_or(owner);
            let sha_v = flag(ctx.core, owner, "fz_new_tieji_sha_cid");
            let sha_cid = if sha_v > 0 { Some(sha_v - 1) } else { None };
            if ask_code == ASK_FZ_NEW_TIEJI_CONFIRM {
                if !ans.contains(&ChoiceOption::Confirm) {
                    return advance_tieji_target(m, ctx);
                }
                let has_deputy = flag(ctx.core, t, "deputy_general") > 0
                    || ctx.core.player(t).general_slots[1].is_some();
                if has_deputy && !has_shouyue_on_table(ctx.core, ctx.catalog) {
                    return wait_ask(m, ctx, ASK_FZ_NEW_TIEJI_CHOOSE_SEAL);
                }
                set_flag(ctx.core, owner, "fz_new_tieji_sealed", 1);
                set_flag(ctx.core, t, "fz_new_tieji_sealed", 1);
                if t != owner {
                    set_flag(ctx.core, t, "fz_new_tieji_target_sealed", 1);
                }
                m.stage = HostStage::Act;
                m.card = Some(99);
                ctx.emit(EventData::OrdinaryJudge {
                    owner,
                    invocation: u32::MAX,
                });
                Flow::Yield
            } else if ask_code == ASK_FZ_NEW_TIEJI_CHOOSE_SEAL {
                let tag = ans
                    .iter()
                    .find_map(|o| match o {
                        ChoiceOption::Tag(tag) => Some(*tag),
                        _ => None,
                    })
                    .unwrap_or(0);
                set_flag(ctx.core, t, "fz_new_tieji_sealed_slot", u32::from(tag) + 1);
                set_flag(ctx.core, owner, "fz_new_tieji_sealed", 1);
                set_flag(ctx.core, t, "fz_new_tieji_sealed", 1);
                if t != owner {
                    set_flag(ctx.core, t, "fz_new_tieji_target_sealed", 1);
                }
                m.stage = HostStage::Act;
                m.card = Some(99);
                ctx.emit(EventData::OrdinaryJudge {
                    owner,
                    invocation: u32::MAX,
                });
                Flow::Yield
            } else {
                if let Some(cid) = ans.iter().find_map(|o| match o {
                    ChoiceOption::Card(c) => Some(*c),
                    _ => None,
                }) {
                    discard_cards(ctx, t, &[cid]);
                    set_flag(ctx.core, owner, "fz_new_tieji_discarded", 1);
                } else {
                    set_flag(ctx.core, owner, "fz_new_tieji_direct_hit", 1);
                    set_flag(ctx.core, t, "fz_new_tieji_direct_hit", 1);
                    mark_sha_unrespondable(ctx, owner, t, sha_cid);
                }
                advance_tieji_target(m, ctx)
            }
        }
        _ => Flow::Done,
    }
}

pub(super) fn handle_fz_liegong(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            set_flag(ctx.core, owner, "fz_liegong_direct_hit", 0);
            if m.targets.is_empty() {
                if let Some(t) = m.target {
                    m.targets.push(t);
                }
            }
            let hp = ctx.core.player(owner).hp;
            let atk_range = ctx.core.attack_range(ctx.catalog, owner) as usize;
            m.targets.retain(|&t| {
                let hand_len = ctx.core.player(t).hand.len();
                hand_len as i32 >= hp || hand_len <= atk_range
            });
            let Some(&first) = m.targets.first() else {
                return Flow::Done;
            };
            m.target = Some(first);
            wait_ask(m, ctx, ASK_FZ_LIEGONG_CONFIRM)
        }
        HostStage::SelectCard => {
            let t = m.target.unwrap_or(owner);
            let sha_cid = m.cards.first().copied();
            if ctx.answer().contains(&ChoiceOption::Confirm) {
                set_flag(ctx.core, owner, "fz_liegong_direct_hit", 1);
                set_flag(ctx.core, t, "fz_liegong_direct_hit", 1);
                mark_sha_unrespondable(ctx, owner, t, sha_cid);
            }
            if !m.targets.is_empty() {
                m.targets.remove(0);
            }
            if let Some(&next_t) = m.targets.first() {
                m.target = Some(next_t);
                return wait_ask(m, ctx, ASK_FZ_LIEGONG_CONFIRM);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn handle_fz_xinkuanggu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if m.cards.is_empty() {
                m.cards.push(m.card.unwrap_or(1).max(1));
            }
            wait_ask(m, ctx, ASK_FZ_XINKUANGGU_MODE)
        }
        HostStage::SelectCard => {
            let Some(mode) = ctx.answer().iter().find_map(|o| match o {
                ChoiceOption::Tag(t) => Some(*t),
                _ => None,
            }) else {
                return Flow::Done;
            };
            let rem = m.cards.first().copied().unwrap_or(1).saturating_sub(1);
            ctx.log(crate::machine::LogEvent::SkillTriggered {
                skill: "狂骨",
                owner,
                presentation: None,
            });
            if mode == 0 && ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32 {
                ctx.recover(owner, 1, None);
                set_flag(ctx.core, owner, "fz_xinkuanggu_recovered", 1);
            } else {
                set_flag(ctx.core, owner, "fz_xinkuanggu_drew", 1);
                if rem == 0 {
                    ctx.start_draw(owner, 1);
                } else {
                    draw_direct(ctx, owner, 1);
                }
            }
            if rem > 0 && ctx.core.player(owner).alive {
                m.cards = vec![rem];
                return wait_ask(m, ctx, ASK_FZ_XINKUANGGU_MODE);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn handle_rekuanshi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => match m.card {
            Some(1) => {
                for p in 0..ctx.core.players.len() as PlayerId {
                    set_flag(ctx.core, p, "rekuanshi_turn_dmg", 0);
                }
                Flow::Done
            }
            Some(3) => {
                if m.target == Some(owner) {
                    let old_t = flag(ctx.core, owner, "rekuanshi_target");
                    if old_t > 0 {
                        set_flag(ctx.core, (old_t - 1) as PlayerId, "rekuanshi_marked", 0);
                    }
                    set_flag(ctx.core, owner, "rekuanshi_target", 0);
                }
                Flow::Done
            }
            Some(2) => {
                let t = m.target.unwrap_or(owner);
                let dmg = m.cards.first().copied().unwrap_or(1).max(1);
                let prev = flag(ctx.core, t, "rekuanshi_turn_dmg");
                let new_total = prev + dmg;
                set_flag(ctx.core, t, "rekuanshi_turn_dmg", new_total);
                set_flag(ctx.core, owner, "rekuanshi_turn_dmg", new_total);
                if prev <= 1
                    && new_total > 1
                    && ctx.core.player(t).alive
                    && ctx.core.player(t).hp < ctx.core.player(t).max_hp as i32
                {
                    set_flag(ctx.core, owner, "rekuanshi_target", 0);
                    set_flag(ctx.core, t, "rekuanshi_marked", 0);
                    set_flag(ctx.core, owner, "rekuanshi_triggered", 1);
                    set_flag(ctx.core, t, "rekuanshi_triggered", 1);
                    ctx.recover(t, 1, None);
                }
                Flow::Done
            }
            _ => wait_ask(m, ctx, ASK_REKUANSHI_TARGET),
        },
        HostStage::SelectCard => {
            let ans = ctx.answer();
            let chosen = ans.iter().find_map(|o| match o {
                ChoiceOption::Player(p) => Some(*p),
                _ => None,
            });
            let Some(t) = chosen else {
                return Flow::Done;
            };
            set_flag(ctx.core, owner, "rekuanshi_target", u32::from(t) + 1);
            set_flag(ctx.core, t, "rekuanshi_marked", 1);
            set_flag(ctx.core, t, "rekuanshi_turn_dmg", 0);
            set_flag(ctx.core, owner, "rekuanshi_turn_dmg", 0);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn cleanup_kuanshi_protection(ctx: &mut Ctx<'_>, ks_owner: PlayerId) {
    let t_enc = flag(ctx.core, ks_owner, "kuanshi_target");
    set_flag(ctx.core, ks_owner, "kuanshi_target", 0);
    let kuanshi2_def = ctx.catalog.skill_by_name("kuanshi2");
    for p in 0..ctx.core.players.len() as PlayerId {
        if flag(ctx.core, p, "kuanshi_protected_by") == u32::from(ks_owner) + 1
            || (t_enc > 0 && p == (t_enc - 1) as PlayerId)
        {
            set_flag(ctx.core, p, "kuanshi_protected_by", 0);
            if let Some(def) = kuanshi2_def {
                ctx.core.skills.remove(&(p, def));
            }
        }
    }
}

pub(super) fn handle_kuanshi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => match m.card {
            Some(1) => {
                cleanup_kuanshi_protection(ctx, owner);
                if flag(ctx.core, owner, "kuanshi_skip_draw") > 0 {
                    ctx.core.player_mut(owner).skip_next_draw = true;
                    set_flag(ctx.core, owner, "kuanshi_skip_draw", 0);
                }
                Flow::Done
            }
            Some(3) => {
                cleanup_kuanshi_protection(ctx, owner);
                set_flag(ctx.core, owner, "kuanshi_skip_draw", 0);
                Flow::Done
            }
            Some(2) => {
                let t = m.target.unwrap_or(owner);
                ctx.current_damage_mut().prevented = true;
                cleanup_kuanshi_protection(ctx, owner);
                set_flag(ctx.core, owner, "kuanshi_prevented", 1);
                set_flag(ctx.core, t, "kuanshi_prevented", 1);
                set_flag(ctx.core, owner, "kuanshi_skip_draw", 1);
                ctx.core.player_mut(owner).skip_next_draw = true;
                Flow::Done
            }
            _ => wait_ask(m, ctx, ASK_KUANSHI_TARGET),
        },
        HostStage::SelectCard => {
            let ans = ctx.answer();
            let chosen = ans.iter().find_map(|o| match o {
                ChoiceOption::Player(p) => Some(*p),
                _ => None,
            });
            let Some(t) = chosen else {
                return Flow::Done;
            };
            cleanup_kuanshi_protection(ctx, owner);
            set_flag(ctx.core, owner, "kuanshi_target", u32::from(t) + 1);
            set_flag(ctx.core, t, "kuanshi_protected_by", u32::from(owner) + 1);
            grant_skill_named(ctx, t, "kuanshi2");
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn handle_kuanshi2(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let target = m.owner;
    if m.card == Some(1) {
        if let Some(p) = m.target {
            if flag(ctx.core, p, "kuanshi_skip_draw") > 0 {
                ctx.core.player_mut(p).skip_next_draw = true;
                set_flag(ctx.core, p, "kuanshi_skip_draw", 0);
            }
        }
        let src_val = flag(ctx.core, target, "kuanshi_protected_by");
        if src_val > 0 {
            let ks_owner = (src_val - 1) as PlayerId;
            if m.target == Some(ks_owner) {
                cleanup_kuanshi_protection(ctx, ks_owner);
            }
        } else if flag(ctx.core, target, "kuanshi_skip_draw") == 0 {
            if let Some(def) = ctx.catalog.skill_by_name("kuanshi2") {
                ctx.core.skills.remove(&(target, def));
            }
        }
        return Flow::Done;
    }
    let src_val = flag(ctx.core, target, "kuanshi_protected_by");
    if src_val == 0 {
        return Flow::Done;
    }
    let ks_owner = (src_val - 1) as PlayerId;
    ctx.current_damage_mut().prevented = true;
    cleanup_kuanshi_protection(ctx, ks_owner);
    set_flag(ctx.core, ks_owner, "kuanshi_prevented", 1);
    set_flag(ctx.core, target, "kuanshi_prevented", 1);
    set_flag(ctx.core, ks_owner, "kuanshi_skip_draw", 1);
    ctx.core.player_mut(ks_owner).skip_next_draw = true;
    Flow::Done
}

fn check_rechuhai_achieve(ctx: &mut Ctx<'_>, owner: PlayerId) {
    if ctx.core.player(owner).equips.len() > 2 && flag(ctx.core, owner, "rechuhai_resolved") == 0 {
        set_flag(ctx.core, owner, "rechuhai_resolved", 1);
        set_flag(ctx.core, owner, "rechuhai_succeeded", 1);
        let heal = (ctx.core.player(owner).max_hp as i32 - ctx.core.player(owner).hp).max(0) as u32;
        if heal > 0 {
            ctx.recover(owner, heal, None);
        }
        if let Some(def) = ctx.catalog.skill_by_name("xianghai") {
            ctx.core.skills.remove(&(owner, def));
        }
        grant_skill_named(ctx, owner, "zhangming");
        set_flag(ctx.core, owner, "zhangming_granted", 1);
    }
}

pub(super) fn handle_rechuhai(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => match m.card {
            Some(2) => {
                let occupied: Vec<_> = ctx
                    .core
                    .player(owner)
                    .equips
                    .iter()
                    .filter_map(|&c| ctx.catalog.card_kind(c).equip_slot())
                    .collect();
                // 上游 chuhai2：`get.cardPile2(equip && player.canEquip(card))`——仅取
                // 能放入空栏的装备牌（牌堆优先、再弃牌堆），无则不置入。
                let fits = |c: crate::ids::CardId| {
                    ctx.catalog.card_kind(c).equip_slot().is_some_and(|s| !occupied.contains(&s))
                };
                let from_deck = ctx.core.deck.iter().rposition(|&c| fits(c));
                let from_discard = ctx.core.discard.iter().rposition(|&c| fits(c));
                let placed = if let Some(idx) = from_deck {
                    Some(ctx.core.deck.remove(idx))
                } else {
                    from_discard.map(|idx| ctx.core.discard.remove(idx))
                };
                if let Some(eq_cid) = placed {
                    ctx.core.player_mut(owner).hand.push(eq_cid);
                    ctx.emit(EventData::Equipped {
                        player: owner,
                        card: eq_cid,
                        replaced: None,
                    });
                }
                refund_once_used(ctx, owner, "rechuhai");
                Flow::Done
            }
            Some(3) => {
                check_rechuhai_achieve(ctx, owner);
                refund_once_used(ctx, owner, "rechuhai");
                Flow::Done
            }
            Some(4) => {
                set_flag(ctx.core, owner, "rechuhai_damage_target", 0);
                refund_once_used(ctx, owner, "rechuhai");
                Flow::Done
            }
            _ => wait_ask(m, ctx, ASK_RECHUHAI_TARGET),
        },
        HostStage::SelectCard => {
            let ask_code = m.card.unwrap_or(0);
            let ans = ctx.answer();
            match ask_code {
                ASK_RECHUHAI_TARGET => {
                    let chosen = ans.iter().find_map(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    });
                    let Some(t) = chosen else {
                        refund_once_used(ctx, owner, "rechuhai");
                        return Flow::Done;
                    };
                    m.target = Some(t);
                    let stamp =
                        (ctx.core.turn.round.max(1) << 8) | u32::from(ctx.core.turn.current);
                    set_flag(ctx.core, owner, "rechuhai_play_stamp", stamp);
                    draw_direct(ctx, owner, 1);
                    if ctx.core.player(owner).hand.is_empty() || ctx.core.player(t).hand.is_empty()
                    {
                        refund_once_used(ctx, owner, "rechuhai");
                        return Flow::Done;
                    }
                    wait_ask(m, ctx, ASK_RECHUHAI_PINDIAN_CARD)
                }
                ASK_RECHUHAI_PINDIAN_CARD => {
                    let chosen = ans.iter().find_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    });
                    let Some(c_owner) = chosen else {
                        refund_once_used(ctx, owner, "rechuhai");
                        return Flow::Done;
                    };
                    m.cards = vec![c_owner];
                    wait_ask(m, ctx, ASK_RECHUHAI_TARGET_PINDIAN_CARD)
                }
                ASK_RECHUHAI_TARGET_PINDIAN_CARD => {
                    let chosen = ans.iter().find_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    });
                    let Some(c_target) = chosen else {
                        refund_once_used(ctx, owner, "rechuhai");
                        return Flow::Done;
                    };
                    let t = m.target.unwrap_or(owner);
                    let c_owner = m.cards.first().copied().unwrap_or(c_target);
                    let bonus = 4usize.saturating_sub(ctx.core.player(owner).equips.len()) as u32;
                    let r_owner = (card_rank(ctx.catalog, c_owner) + bonus).min(13);
                    let r_target = card_rank(ctx.catalog, c_target);
                    set_flag(ctx.core, owner, "rechuhai_owner_num", r_owner);
                    discard_cards(ctx, owner, &[c_owner]);
                    discard_cards(ctx, t, &[c_target]);
                    if r_owner > r_target {
                        set_flag(ctx.core, owner, "rechuhai_won", 1);
                        set_flag(ctx.core, owner, "rechuhai_damage_target", u32::from(t) + 1);
                        let mut type_mask = 0u32;
                        for &hc in &ctx.core.player(t).hand {
                            let k = ctx.catalog.card_kind(hc);
                            if k.is_basic_kind() {
                                type_mask |= 1;
                            } else if k.equip_slot().is_some() {
                                type_mask |= 4;
                            } else {
                                type_mask |= 2;
                            }
                        }
                        // 上游 `get.cardPile(type2==i)`：先牌堆后弃牌堆，找不到该类别则不补。
                        let type_bit = |ck: CardKind| {
                            if ck.is_basic_kind() {
                                1
                            } else if ck.equip_slot().is_some() {
                                4
                            } else {
                                2
                            }
                        };
                        let mut gained = Vec::new();
                        for want_bit in [1u32, 2, 4] {
                            if (type_mask & want_bit) == 0 {
                                continue;
                            }
                            let cat = ctx.catalog;
                            if let Some(idx) =
                                ctx.core.deck.iter().rposition(|&c| type_bit(cat.card_kind(c)) == want_bit)
                            {
                                gained.push(ctx.core.deck.remove(idx));
                            } else if let Some(idx) = ctx
                                .core
                                .discard
                                .iter()
                                .rposition(|&c| type_bit(cat.card_kind(c)) == want_bit)
                            {
                                gained.push(ctx.core.discard.remove(idx));
                            }
                        }
                        ctx.core.player_mut(owner).hand.extend(gained.iter().copied());
                        if !gained.is_empty() {
                            ctx.emit(EventData::CardsDrawn {
                                player: owner,
                                cards: gained,
                            });
                        }
                    } else {
                        set_flag(ctx.core, owner, "rechuhai_won", 0);
                        if r_owner < 7 {
                            set_flag(ctx.core, owner, "rechuhai_resolved", 2);
                            set_flag(ctx.core, owner, "rechuhai_failed", 1);
                        }
                    }
                    refund_once_used(ctx, owner, "rechuhai");
                    Flow::Done
                }
                _ => Flow::Done,
            }
        }
        _ => Flow::Done,
    }
}
