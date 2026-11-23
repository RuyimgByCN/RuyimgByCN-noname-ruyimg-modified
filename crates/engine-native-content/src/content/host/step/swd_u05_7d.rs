//! U05-7d 杀海拾遗 swd 人物补充与共享原语（`swd_u05_7d.rs`）。

use super::super::*;
use super::swd_u05_7d_duijue_tanlin::step_duijue_tanlin;

/// 通用弃置己方牌（手牌或装备区）：确保清理 `declared_slots`、装备技与离场事件。
pub(super) fn discard_owned_card_once(ctx: &mut Ctx<'_>, owner: PlayerId, card: CardId) -> bool {
    if ctx.core.player(owner).hand.contains(&card) {
        ctx.discard_from_hand(owner, card);
        return true;
    }
    if ctx.core.player(owner).equips.contains(&card) {
        ctx.core.player_mut(owner).equips.retain(|x| *x != card);
        ctx.core.declared_slots.remove(&card);
        if ctx.catalog.card_kind(card).is_equip() {
            crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, owner, card);
        }
        ctx.to_discard(card);
        ctx.note_cards_lost(owner, &[card]);
        ctx.emit(EventData::CardsLost {
            player: owner,
            cards: vec![card],
            origins: vec![crate::event::LostOrigin::Equip],
        });
        ctx.log(LogEvent::CardDiscarded {
            player: owner,
            cards: vec![card],
        });
        return true;
    }
    false
}

/// 对多目标 `CardUsed` 仅移除单个目标 `target`（仅当 `targets` 清空时整体置 `cancelled = true`）。
pub(super) fn cancel_target_or_frame(ctx: &mut Ctx<'_>, fid: crate::ids::EventId, target: PlayerId) {
    if let Some(frame) = ctx.res.frame_mut(fid) {
        if let EventData::CardUsed(u) = &mut frame.data {
            u.targets.retain(|&t| t != target);
            if u.targets.is_empty() {
                u.cancelled = true;
            }
        }
    }
}

/// 退还主动技 `once_used` 次数（选目标/选牌前取消不白耗限次）。
pub(super) fn refund_once_used(ctx: &mut Ctx<'_>, owner: PlayerId, name: &str) {
    if let Some(def) = ctx.catalog.skill_by_name(name) {
        if let Some(inst) = ctx.core.skills.get_mut(&(owner, def)) {
            inst.once_used = false;
        }
    }
}

/// 空牌堆安全虚拟杀（不从牌堆抽载体、不占出牌阶段出杀次数）。
pub(super) fn virtual_sha_safe(
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    as_kind: CardKind,
    nature: crate::event::DamageNature,
    targets: Vec<PlayerId>,
) {
    ctx.core
        .vm_aux
        .flags
        .insert((owner, "vm_va_sha_no_count".to_string()), 1);
    ctx.emit(EventData::CardUsed(crate::event::CardUseData {
        card: None,
        player: owner,
        targets,
        nature,
        use_as: Some(as_kind),
        cancelled: false,
    }));
}

/// 发起一次走判定机（支持鬼才/鬼道/天妒等改判与响应）的常规判定。
pub(super) fn start_ordinary_judge(
    ctx: &mut Ctx<'_>,
    judge_owner: PlayerId,
    pref: Option<engine_native_kernel::kernel::EloJudgePref>,
) {
    for k in ["last_judge_suit", "last_judge_rank", "last_judge_card"] {
        ctx.core.vm_aux.flags.remove(&(judge_owner, k.to_string()));
    }
    engine_native_kernel::kernel::set_ordinary_judge_elo_pref(ctx.core, judge_owner, pref);
    ctx.emit(EventData::OrdinaryJudge {
        owner: judge_owner,
        invocation: u32::MAX,
    });
}

/// 读取并清理最近一次 `OrdinaryJudge` 结果 `(suit, rank, card_id)`。
pub(super) fn finish_ordinary_judge(
    ctx: &mut Ctx<'_>,
    judge_owner: PlayerId,
) -> (crate::catalog::Suit, u8, Option<CardId>) {
    engine_native_kernel::kernel::clear_ordinary_judge_elo_pref(ctx.core, judge_owner);
    let suit_code = ctx
        .core
        .vm_aux
        .flags
        .remove(&(judge_owner, "last_judge_suit".to_string()))
        .unwrap_or(1);
    let rank = ctx
        .core
        .vm_aux
        .flags
        .remove(&(judge_owner, "last_judge_rank".to_string()))
        .unwrap_or(1) as u8;
    let card = ctx
        .core
        .vm_aux
        .flags
        .remove(&(judge_owner, "last_judge_card".to_string()));
    let suit = match suit_code {
        1 => crate::catalog::Suit::Spade,
        2 => crate::catalog::Suit::Heart,
        3 => crate::catalog::Suit::Club,
        4 => crate::catalog::Suit::Diamond,
        _ => crate::catalog::Suit::Spade,
    };
    (suit, rank, card)
}

/// 乐学（`lexue1` 回合开始 / `lexue2` 回合结束）切换临时技能并清理前值。
pub(super) fn lexue_apply_pool(ctx: &mut Ctx<'_>, owner: PlayerId, pool: &[&str]) {
    if let Some(prev) = ctx
        .core
        .vm_aux
        .flags
        .remove(&(owner, "lexue_granted_def".to_string()))
    {
        if let Some(prev_def) = prev.checked_sub(1).map(|v| crate::ids::SkillDefId::from(v as u16)) {
            if let Some((orig, prev_name)) = ctx
                .catalog
                .skills
                .get(prev_def as usize)
                .map(|d| (prev_def, d.name))
            {
                let _ = orig;
                let keep_owned = ctx
                    .core
                    .player(owner)
                    .general_slots
                    .iter()
                    .flatten()
                    .any(|slot| {
                        ctx.catalog
                            .general_meta(*slot)
                            .is_some_and(|(_, _, sks, _, _)| sks.contains(&prev_name))
                    });
                if !keep_owned {
                    ctx.core.skills.remove(&(owner, prev_def));
                }
            }
        }
    }
    let defs: Vec<crate::ids::SkillDefId> = pool
        .iter()
        .filter_map(|n| ctx.catalog.skill_by_name(n))
        .filter(|d| !ctx.core.skills.contains_key(&(owner, *d)))
        .collect();
    if defs.is_empty() {
        return;
    }
    let j = ctx.core.rng.gen_range(defs.len() as u32) as usize;
    let chosen = defs[j];
    ctx.insert_skill(
        (owner, chosen),
        crate::state::SkillInstance {
            owner,
            enabled: true,
            once_used: false,
            expires_end_of_turn: false,
            rounds: 0,
        },
    );
    ctx.core
        .vm_aux
        .flags
        .insert((owner, "lexue_granted_def".to_string()), u32::from(u16::from(chosen)) + 1);
}

/// 手牌中是否存在至少两张同有效花色的牌。
pub(in crate::content::host) fn same_suit_hand_candidates(
    catalog: &crate::catalog::Catalog,
    core: &crate::state::GameStateCore,
    owner: PlayerId,
) -> Vec<CardId> {
    let hand = &core.player(owner).hand;
    hand.iter()
        .copied()
        .filter(|&c| {
            let s = catalog.effective_card_suit(core, owner, c);
            hand.iter()
                .any(|&other| other != c && catalog.effective_card_suit(core, owner, other) == s)
        })
        .collect()
}

pub(super) fn step_swd_u05_7d(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.behavior {
        HostBehavior::SwdDuijue
        | HostBehavior::SwdDuijueOut
        | HostBehavior::SwdDuijueTrigger
        | HostBehavior::SwdTanlin
        | HostBehavior::SwdTanlinTarget
        | HostBehavior::SwdTanlinOwner
        | HostBehavior::SwdTanlinRange
        | HostBehavior::SwdTanlinDefence => step_duijue_tanlin(m, ctx),
        HostBehavior::SwdOldLiaoyuan => match m.stage {
            HostStage::Act => {
                let cands = same_suit_hand_candidates(ctx.catalog, ctx.core, m.owner);
                if cands.len() < 2 {
                    refund_once_used(ctx, m.owner, &m.def_key);
                    return Flow::Done;
                }
                m.stage = HostStage::SelectCard;
                Flow::WaitChoice {
                    observer: m.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "oldliaoyuan_cards",
                        options: cands.into_iter().map(ChoiceOption::Card).collect(),
                        min: 2,
                        max: 2,
                        allow_cancel: true,
                    },
                }
            }
            HostStage::SelectCard => {
                let picks: Vec<CardId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    })
                    .collect();
                if picks.len() != 2 || picks[0] == picks[1] {
                    refund_once_used(ctx, m.owner, &m.def_key);
                    return Flow::Done;
                }
                let s0 = ctx.catalog.effective_card_suit(ctx.core, m.owner, picks[0]);
                let s1 = ctx.catalog.effective_card_suit(ctx.core, m.owner, picks[1]);
                if s0 != s1
                    || !ctx.core.player(m.owner).hand.contains(&picks[0])
                    || !ctx.core.player(m.owner).hand.contains(&picks[1])
                {
                    let cands = same_suit_hand_candidates(ctx.catalog, ctx.core, m.owner);
                    if cands.len() < 2 {
                        refund_once_used(ctx, m.owner, &m.def_key);
                        return Flow::Done;
                    }
                    return Flow::WaitChoice {
                        observer: m.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "oldliaoyuan_cards",
                            options: cands.into_iter().map(ChoiceOption::Card).collect(),
                            min: 2,
                            max: 2,
                            allow_cancel: true,
                        },
                    };
                }
                m.cards = picks;
                let targets: Vec<ChoiceOption> = ctx
                    .core
                    .alive_players()
                    .filter(|p| *p != m.owner && !ctx.catalog.has_undist(ctx.core, *p))
                    .map(ChoiceOption::Player)
                    .collect();
                if targets.is_empty() {
                    refund_once_used(ctx, m.owner, &m.def_key);
                    return Flow::Done;
                }
                m.stage = HostStage::TargetChoice;
                Flow::WaitChoice {
                    observer: m.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectPlayers,
                        prompt: "oldliaoyuan_target",
                        options: targets,
                        min: 1,
                        max: 1,
                        allow_cancel: true,
                    },
                }
            }
            HostStage::TargetChoice => {
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    refund_once_used(ctx, m.owner, &m.def_key);
                    return Flow::Done;
                };
                if !ctx.core.player(t).alive || t == m.owner {
                    refund_once_used(ctx, m.owner, &m.def_key);
                    return Flow::Done;
                }
                let cost = std::mem::take(&mut m.cards);
                for c in cost {
                    discard_owned_card_once(ctx, m.owner, c);
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "燎原",
                    owner: m.owner,
                    presentation: None,
                });
                m.target = Some(t);
                let mut he: Vec<CardId> = ctx.core.player(t).hand.iter().copied().collect();
                he.extend(ctx.core.player(t).equips.iter().copied());
                if !he.is_empty() {
                    m.stage = HostStage::CardsAsk;
                    return Flow::WaitChoice {
                        observer: m.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "oldliaoyuan_target_discard",
                            options: he.into_iter().map(ChoiceOption::Card).collect(),
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    };
                }
                virtual_sha_safe(
                    ctx,
                    m.owner,
                    CardKind::FireSha,
                    crate::event::DamageNature::Fire,
                    vec![t],
                );
                Flow::Done
            }
            HostStage::CardsAsk => {
                let Some(t) = m.target else {
                    return Flow::Done;
                };
                if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                    discard_owned_card_once(ctx, t, c);
                }
                if ctx.core.player(t).alive {
                    virtual_sha_safe(
                        ctx,
                        m.owner,
                        CardKind::FireSha,
                        crate::event::DamageNature::Fire,
                        vec![t],
                    );
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        HostBehavior::SwdOldLiaoyuan2 => match m.stage {
            HostStage::Act => {
                let cands = same_suit_hand_candidates(ctx.catalog, ctx.core, m.owner);
                if cands.len() < 2 {
                    refund_once_used(ctx, m.owner, &m.def_key);
                    return Flow::Done;
                }
                m.stage = HostStage::SelectCard;
                Flow::WaitChoice {
                    observer: m.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "oldliaoyuan2_cards",
                        options: cands.into_iter().map(ChoiceOption::Card).collect(),
                        min: 2,
                        max: 2,
                        allow_cancel: true,
                    },
                }
            }
            HostStage::SelectCard => {
                let picks: Vec<CardId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    })
                    .collect();
                if picks.len() != 2 || picks[0] == picks[1] {
                    refund_once_used(ctx, m.owner, &m.def_key);
                    return Flow::Done;
                }
                let s0 = ctx.catalog.effective_card_suit(ctx.core, m.owner, picks[0]);
                let s1 = ctx.catalog.effective_card_suit(ctx.core, m.owner, picks[1]);
                if s0 != s1
                    || !ctx.core.player(m.owner).hand.contains(&picks[0])
                    || !ctx.core.player(m.owner).hand.contains(&picks[1])
                {
                    let cands = same_suit_hand_candidates(ctx.catalog, ctx.core, m.owner);
                    if cands.len() < 2 {
                        refund_once_used(ctx, m.owner, &m.def_key);
                        return Flow::Done;
                    }
                    return Flow::WaitChoice {
                        observer: m.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "oldliaoyuan2_cards",
                            options: cands.into_iter().map(ChoiceOption::Card).collect(),
                            min: 2,
                            max: 2,
                            allow_cancel: true,
                        },
                    };
                }
                m.cards = picks;
                let targets: Vec<ChoiceOption> = ctx
                    .core
                    .alive_players()
                    .filter(|p| *p != m.owner && !ctx.catalog.has_undist(ctx.core, *p))
                    .map(ChoiceOption::Player)
                    .collect();
                if targets.is_empty() {
                    refund_once_used(ctx, m.owner, &m.def_key);
                    return Flow::Done;
                }
                m.stage = HostStage::TargetChoice;
                Flow::WaitChoice {
                    observer: m.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectPlayers,
                        prompt: "oldliaoyuan2_target",
                        options: targets,
                        min: 1,
                        max: 1,
                        allow_cancel: true,
                    },
                }
            }
            HostStage::TargetChoice => {
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    refund_once_used(ctx, m.owner, &m.def_key);
                    return Flow::Done;
                };
                if !ctx.core.player(t).alive || t == m.owner || m.cards.len() != 2 {
                    refund_once_used(ctx, m.owner, &m.def_key);
                    return Flow::Done;
                }
                let c0 = m.cards[0];
                let c1 = m.cards[1];
                if !ctx.core.player(m.owner).hand.contains(&c0) {
                    refund_once_used(ctx, m.owner, &m.def_key);
                    return Flow::Done;
                }
                m.target = Some(t);
                m.card = Some(c1);
                m.cards.clear();
                ctx.core
                    .vm_aux
                    .flags
                    .insert((m.owner, "vm_va_sha_no_count".to_string()), 1);
                ctx.emit(EventData::CardUsed(crate::event::CardUseData {
                    card: Some(c0),
                    player: m.owner,
                    targets: vec![t],
                    nature: crate::event::DamageNature::Normal,
                    use_as: Some(CardKind::Sha),
                    cancelled: false,
                }));
                m.stage = HostStage::ShuchuiTargetLoop;
                Flow::Yield
            }
            HostStage::ShuchuiTargetLoop => {
                let (Some(t), Some(c1)) = (m.target, m.card) else {
                    return Flow::Done;
                };
                if ctx.core.player(m.owner).alive
                    && ctx.core.player(t).alive
                    && ctx.core.player(m.owner).hand.contains(&c1)
                {
                    ctx.core
                        .vm_aux
                        .flags
                        .insert((m.owner, "vm_va_sha_no_count".to_string()), 1);
                    ctx.emit(EventData::CardUsed(crate::event::CardUseData {
                        card: Some(c1),
                        player: m.owner,
                        targets: vec![t],
                        nature: crate::event::DamageNature::Normal,
                        use_as: Some(CardKind::Sha),
                        cancelled: false,
                    }));
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        HostBehavior::SwdHuanling2 => {
            if ctx.core.player(m.owner).face_down {
                let fid = ctx.current_event_frame_id();
                if let Some(frame) = ctx.res.frame_mut(fid) {
                    if let EventData::Damage(d) = &mut frame.data {
                        d.prevented = true;
                    }
                }
            }
            Flow::Done
        }
        HostBehavior::SwdLexue2 => {
            lexue_apply_pool(ctx, m.owner, &["遗计", "急救", "鬼道", "反馈"]);
            Flow::Done
        }
        HostBehavior::SwdTianshuRemove => {
            let p = ctx.core.player_mut(m.owner);
            let old_from = p.swdtianshu_from.take();
            let old_granted = p.swdtianshu_granted.take();
            if let (Some(_), Some(raw_id)) = (old_from, old_granted) {
                let def = crate::ids::SkillDefId::from(raw_id);
                ctx.core.skills.remove(&(m.owner, def));
            }
            Flow::Done
        }
        HostBehavior::SwdMoyu => {
            let p = ctx.core.player(m.owner);
            if p.alive && p.hp < p.max_hp as i32 {
                let need = (p.max_hp as i32 - p.hp) as u32;
                ctx.recover(m.owner, need, None);
            }
            Flow::Done
        }
        HostBehavior::SwdYinguo2 | HostBehavior::SwdSusheng2 | HostBehavior::SwdU05_7dSub => {
            Flow::Done
        }
        _ => Flow::Done,
    }
}

/// 集中派发 `U05-7d` 修复覆写机（自 `step.rs` 拆出以保持 `step.rs` 单文件行数门禁）。
pub(super) fn try_step_swd_u05_7d_fixes_override(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
) -> Option<Flow> {
    match m.behavior {
        HostBehavior::SwdBusi
        | HostBehavior::SwdHuanhun
        | HostBehavior::SwdDunxing
        | HostBehavior::SwdShending
        | HostBehavior::SwdHuanyin
        | HostBehavior::SwdLqingcheng
        | HostBehavior::SwdDuanyi
        | HostBehavior::SwdToudan
        | HostBehavior::SwdGuanhu
        | HostBehavior::SwdShehun
        | HostBehavior::SwdXuanzhou
        | HostBehavior::SwdXuehuang
        | HostBehavior::Moyan
        | HostBehavior::SwdHuajian
        | HostBehavior::Lxianglong
        | HostBehavior::SwdRexue
        | HostBehavior::Kunlunjing
        | HostBehavior::SwdDaixing3
        | HostBehavior::SwdYinguo
        | HostBehavior::SwdSusheng
        | HostBehavior::SwdLexue
        | HostBehavior::SwdPozhen
        | HostBehavior::SwdGuiyan2 => Some(super::swd_u05_7d_fixes::step_swd_u05_7d_fix(m, ctx)),
        _ => None,
    }
}
