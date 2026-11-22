//! L28 蒲元 `th_shengong` 18 张专属装备技能（`sp.js:7634-8003`, `yingbian.js:306-458`, `guozhan.js:1688-1712`）。

use super::*;
use crate::catalog::{CardKind, Catalog, EquipSlot, QueryCtx, TriggerMeta};
use crate::event::{CardUseData, DamageNature, EventData, EventKind, LostOrigin, Phase, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{damage_data, damage_data_with, Ctx, Flow};
use crate::rvm_bridge::modifiers::{d34_l28_mods, eval_card_target_canceled, is_target_unequip2};
use crate::state::GameStateCore;

#[path = "l28_shengong_equips_asks.rs"]
mod asks;
use asks::*;
pub(super) use asks::build_equip_ask;

const SHENGONG_EQUIP_SKILLS: [&str; 19] = [
    "wushuangfangtianji_skill",
    "guilongzhanyuedao",
    "chixueqingfeng",
    "bintieshuangji_skill",
    "wutiesuolian_skill",
    "wuxinghelingshan_skill",
    "linglongshimandai_skill",
    "hongmianbaihuapao_skill",
    "qimenbagua",
    "guofengyupao",
    "huxinjing",
    "heiguangkai_skill",
    "shufazijinguan_skill",
    "xuwangzhimian",
    "tianjitu_skill",
    "taigongyinfu_skill",
    "taigongyinfu_link",
    "sanlve_skill",
    "zhaogujing_skill",
];

/// Returns whether `name` belongs to the `th_shengong` exclusive equipment skill set.
pub(super) fn handles_skill(name: &str) -> bool {
    SHENGONG_EQUIP_SKILLS.contains(&name)
}

/// Trigger metadata for interactive/triggered `th_shengong` equipment skills.
/// Purely passive/modifier skills return `None` and run via kernel modifiers.
pub(super) fn trigger_meta_shengong_equip(name: &str) -> Option<TriggerMeta> {
    let meta = |event: EventKind, extra_events: Vec<EventKind>| TriggerMeta {
        event,
        timing: Timing::Normal,
        priority: 1,
        order_selectable: false,
        optional: false,
        extra_events,
    };
    match name {
        "wushuangfangtianji_skill" => Some(meta(EventKind::Damage, Vec::new())),
        "chixueqingfeng"
        | "wutiesuolian_skill"
        | "wuxinghelingshan_skill"
        | "linglongshimandai_skill" => Some(meta(EventKind::CardUsed, Vec::new())),
        "bintieshuangji_skill" => Some(meta(EventKind::ShaDodged, Vec::new())),
        "huxinjing" => Some(meta(EventKind::DamageBefore, Vec::new())),
        "shufazijinguan_skill" | "taigongyinfu_link" => Some(meta(EventKind::PhaseBegin, Vec::new())),
        "taigongyinfu_skill" | "zhaogujing_skill" => Some(meta(EventKind::PhaseEnd, Vec::new())),
        "tianjitu_skill" => Some(meta(EventKind::Equipped, Vec::new())),
        _ => None,
    }
}

fn has_huxinjing_equipped(core: &GameStateCore, catalog: &Catalog, p: PlayerId) -> bool {
    core.player(p)
        .equips
        .iter()
        .any(|&c| catalog.cards.get(c as usize).is_some_and(|cd| cd.name == "huxinjing"))
}

fn kind_nature(kind: CardKind) -> DamageNature {
    match kind {
        CardKind::FireSha => DamageNature::Fire,
        CardKind::LeiSha => DamageNature::Thunder,
        _ => DamageNature::Normal,
    }
}

fn player_has_zone_cards(core: &GameStateCore, p: PlayerId) -> bool {
    let pl = core.player(p);
    !pl.hand.is_empty() || !pl.equips.is_empty() || !pl.judge.is_empty()
}

#[inline]
pub(self) fn zhaogujing_is_huogong(catalog: &Catalog, c: CardId) -> bool {
    catalog.cards.get(c as usize).is_some_and(|d| d.name == "huogong")
}

pub(self) fn zhaogujing_jiedao_victims(
    core: &GameStateCore,
    catalog: &Catalog,
    holder: PlayerId,
) -> Vec<PlayerId> {
    let range = core.attack_range(catalog, holder);
    alive_others(core, holder)
        .into_iter()
        .filter(|&v| {
            core.computed_distance(catalog, holder, v) <= range
                && !eval_card_target_canceled(core, catalog, holder, v, 0, CardKind::Sha, DamageNature::Normal)
        })
        .collect()
}

pub(self) fn zhaogujing_huogong_targets(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<PlayerId> {
    alive_all(core, owner)
        .into_iter()
        .filter(|&t| {
            !core.player(t).hand.is_empty()
                && !eval_card_target_canceled(core, catalog, owner, t, 0, CardKind::Linghunzhihuo, DamageNature::Fire)
        })
        .collect()
}

fn zhaogujing_needs_target(kind: CardKind) -> bool {
    matches!(
        kind,
        CardKind::Sha
            | CardKind::FireSha
            | CardKind::LeiSha
            | CardKind::Duel
            | CardKind::Dismantle
            | CardKind::Snatch
            | CardKind::Jiedao
            | CardKind::Chain
    )
}

pub(self) fn zhaogujing_targets_for(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    kind: CardKind,
) -> Vec<PlayerId> {
    let nature = kind_nature(kind);
    let range = core.attack_range(catalog, owner);
    let pool = if kind == CardKind::Chain {
        alive_all(core, owner)
    } else {
        alive_others(core, owner)
    };
    pool.into_iter()
        .filter(|&t| {
            let geom_ok = match kind {
                CardKind::Sha | CardKind::FireSha | CardKind::LeiSha => {
                    core.computed_distance(catalog, owner, t) <= range
                }
                CardKind::Snatch => {
                    core.computed_distance(catalog, owner, t) <= 1 && player_has_zone_cards(core, t)
                }
                CardKind::Dismantle => player_has_zone_cards(core, t),
                CardKind::Jiedao => {
                    core.player(t)
                        .equips
                        .iter()
                        .any(|&c| catalog.card_slot_of(core, c) == Some(EquipSlot::Weapon))
                        && !zhaogujing_jiedao_victims(core, catalog, t).is_empty()
                }
                CardKind::Duel | CardKind::Chain => true,
                _ => false,
            };
            geom_ok && !eval_card_target_canceled(core, catalog, owner, t, 0, kind, nature)
        })
        .collect()
}

fn zhaogujing_card_usable(core: &GameStateCore, catalog: &Catalog, owner: PlayerId, c: CardId) -> bool {
    if zhaogujing_is_huogong(catalog, c) {
        return !zhaogujing_huogong_targets(core, catalog, owner).is_empty();
    }
    let kind = catalog.card_kind(c);
    match kind {
        CardKind::Tao => core.player(owner).hp < core.player(owner).max_hp as i32,
        CardKind::Jiu | CardKind::Wuzhong | CardKind::Taoyuan | CardKind::Wugu => true,
        CardKind::SavageAttack | CardKind::ArcheryAttack => !alive_others(core, owner).is_empty(),
        k if zhaogujing_needs_target(k) => !zhaogujing_targets_for(core, catalog, owner, k).is_empty(),
        _ => false,
    }
}

pub(self) fn zhaogujing_eligible_cards(core: &GameStateCore, catalog: &Catalog, owner: PlayerId) -> Vec<CardId> {
    core.player(owner)
        .hand
        .iter()
        .copied()
        .filter(|&c| zhaogujing_card_usable(core, catalog, owner, c))
        .collect()
}

/// Filter callback for triggered `th_shengong` equipment skills.
pub(super) fn filter_shengong_equip(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    if !ctx.state.player(owner).alive {
        return false;
    }
    match skill.name.as_str() {
        "wushuangfangtianji_skill" => matches!(
            event,
            EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && (!d.from_chain || is_l11_taixu_context(ctx.catalog))
                    && d.amount > 0
                    && (d.card.is_some_and(|c| {
                        ctx.catalog.card_kind(c).is_sha_like()
                            || ctx
                                .state
                                .declared_kinds
                                .get(&c)
                                .is_some_and(|k| k.is_sha_like())
                    }) || (d.card.is_none() && flag(ctx.state, d.target, "l28_vdmg_kind") == 1))
        ),
        "chixueqingfeng" => match event {
            EventData::CardUsed(u) if u.player == owner && !u.cancelled => {
                let kind = u.use_as.unwrap_or_else(|| u.effective_kind(ctx.catalog));
                let n = u.targets.len();
                kind.is_sha_like()
                    && u.targets.iter().any(|&t| {
                        ctx.state.player(t).alive
                            && !d34_l28_mods::shengong_excludes_target(
                                ctx.state,
                                ctx.catalog,
                                owner,
                                t,
                                u.card,
                                kind,
                                u.nature,
                                n,
                            )
                    })
            }
            _ => false,
        },
        "bintieshuangji_skill" => matches!(
            event,
            EventData::ShaDodged { source, .. }
                if *source == owner && ctx.state.player(owner).hp > 0
        ),
        "wutiesuolian_skill" => match event {
            EventData::CardUsed(u) if u.player == owner && !u.cancelled => {
                let kind = u.use_as.unwrap_or_else(|| u.effective_kind(ctx.catalog));
                let n = u.targets.len();
                kind.is_sha_like()
                    && u.targets.iter().any(|&t| {
                        ctx.state.player(t).alive
                            && !ctx.state.player(t).linked
                            && !d34_l28_mods::shengong_excludes_target(
                                ctx.state,
                                ctx.catalog,
                                owner,
                                t,
                                u.card,
                                kind,
                                u.nature,
                                n,
                            )
                    })
            }
            _ => false,
        },
        "wuxinghelingshan_skill" => match event {
            EventData::CardUsed(u) if u.player == owner && !u.cancelled => {
                let kind = u.use_as.unwrap_or_else(|| u.effective_kind(ctx.catalog));
                kind.is_sha_like() && matches!(u.nature, DamageNature::Fire | DamageNature::Thunder)
            }
            _ => false,
        },
        "linglongshimandai_skill" => match event {
            EventData::CardUsed(u)
                if u.player != owner && !u.cancelled && u.targets.len() == 1 && u.targets[0] == owner =>
            {
                let kind = u.use_as.unwrap_or_else(|| u.effective_kind(ctx.catalog));
                !is_target_unequip2(ctx.state, owner)
                    && !d34_l28_mods::source_ignores_armor(
                        ctx.state,
                        ctx.catalog,
                        u.player,
                        owner,
                        u.card,
                        Some(kind),
                        u.nature,
                    )
                    && !d34_l28_mods::shengong_excludes_target(
                        ctx.state,
                        ctx.catalog,
                        u.player,
                        owner,
                        u.card,
                        kind,
                        u.nature,
                        1,
                    )
            }
            _ => false,
        },
        "huxinjing" => matches!(
            event,
            EventData::Damage(d)
                if d.target == owner
                    && !d.prevented
                    && d.amount > 0
                    && (d.amount > 1 || d.amount >= ctx.state.player(owner).hp)
                    && !is_target_unequip2(ctx.state, owner)
                    && !d.source.is_some_and(|s| {
                        d34_l28_mods::source_ignores_armor(
                            ctx.state,
                            ctx.catalog,
                            s,
                            owner,
                            d.card,
                            None,
                            d.nature,
                        )
                    })
                    && has_huxinjing_equipped(ctx.state, ctx.catalog, owner)
        ),
        "shufazijinguan_skill" => matches!(
            event,
            EventData::Phase { player, phase: Phase::Prepare, begin: true }
                if *player == owner && !alive_others(ctx.state, owner).is_empty()
        ),
        "tianjitu_skill" => matches!(
            event,
            EventData::Equipped { player, card, .. }
                if *player == owner
                    && ctx.catalog.cards.get(*card as usize).is_some_and(|d| d.name == "tianjitu")
                    && hand_and_equips(ctx.state, owner).into_iter().any(|c| c != *card)
        ),
        "taigongyinfu_link" => matches!(
            event,
            EventData::Phase { player, phase: Phase::Play, begin: true } if *player == owner
        ),
        "taigongyinfu_skill" => matches!(
            event,
            EventData::Phase { player, phase: Phase::Play, begin: false }
                if *player == owner && !ctx.state.player(owner).hand.is_empty()
        ),
        "zhaogujing_skill" => matches!(
            event,
            EventData::Phase { player, phase: Phase::Play, begin: false }
                if *player == owner && !zhaogujing_eligible_cards(ctx.state, ctx.catalog, owner).is_empty()
        ),
        _ => false,
    }
}

/// `tianjitu_skill` on-lose hook (`yingbian.js:389-406`): draw up to 5 hand cards.
pub(super) fn step_tianjitu_lose(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if ctx.core.player(owner).alive {
        let len = ctx.core.player(owner).hand.len();
        if len < 5 {
            ctx.start_draw(owner, (5 - len) as u32);
        }
    }
    Flow::Done
}

/// Returns whether the active catalog runs in L11 Taixu context.
pub(self) fn is_l11_taixu_context(catalog: &Catalog) -> bool {
    catalog.skill_by_name("buff_txhj_fuchendangmo").is_some()
}

/// Dispatches step execution for triggered `th_shengong` equipment skills.
pub(super) fn step_shengong_equip(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.def_key.as_str() {
        "wushuangfangtianji_skill" => step_wushuangfangtianji(m, ctx),
        "chixueqingfeng" => step_chixueqingfeng(m, ctx),
        "bintieshuangji_skill" => step_bintieshuangji(m, ctx),
        "wutiesuolian_skill" => step_wutiesuolian(m, ctx),
        "wuxinghelingshan_skill" => step_wuxinghelingshan(m, ctx),
        "linglongshimandai_skill" => step_linglongshimandai(m, ctx),
        "huxinjing" => step_huxinjing(m, ctx),
        "shufazijinguan_skill" => step_shufazijinguan(m, ctx),
        "tianjitu_skill" => step_tianjitu(m, ctx),
        "taigongyinfu_link" => step_taigong_link(m, ctx),
        "taigongyinfu_skill" => step_taigong_recast(m, ctx),
        "zhaogujing_skill" => step_zhaogujing(m, ctx),
        _ => Flow::Done,
    }
}

fn step_wushuangfangtianji(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if is_l11_taixu_context(ctx.catalog)
                && m.target
                    .is_none_or(|t| !ctx.core.player(t).alive || hand_and_equips(ctx.core, t).is_empty())
            {
                ctx.start_draw(owner, 1);
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::SelectNature, ASK_SG_WUSHUANG_MODE)
        }
        HostStage::SelectNature => {
            let ans = ctx.answer().first().copied();
            set_flag(ctx.core, owner, "l28_ask_id", 0);
            match ans {
                Some(ChoiceOption::Tag(0)) => {
                    ctx.start_draw(owner, 1);
                    Flow::Done
                }
                Some(ChoiceOption::Tag(1)) => {
                    wait_ask(m, ctx, HostStage::SelectCard, ASK_SG_WUSHUANG_DISCARD)
                }
                _ => Flow::Done,
            }
        }
        HostStage::SelectCard | HostStage::TargetChoice => {
            let ans = ctx.answer().first().copied();
            set_flag(ctx.core, owner, "l28_ask_id", 0);
            if let (Some(t), Some(ChoiceOption::Card(c))) = (m.target, ans) {
                let first_pass = m.stage == HostStage::SelectCard;
                discard_cards(ctx, t, &[c]);
                if super::step::l11_common_b4::post_wushuang_discard_l11_b4(
                    ctx, owner, t, c, first_pass,
                ) {
                    return wait_ask(m, ctx, HostStage::TargetChoice, ASK_SG_WUSHUANG_DISCARD);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_chixueqingfeng(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let frame_id = ctx.current_event_frame_id();
    let frame_u = ctx.res.frame(frame_id).and_then(|f| match &f.data {
        EventData::CardUsed(u) => Some((
            u.card,
            u.use_as.unwrap_or_else(|| u.effective_kind(ctx.catalog)),
            u.nature,
            u.targets.clone(),
        )),
        _ => None,
    });
    let (card, kind, nature, list) = frame_u.unwrap_or((m.card, CardKind::Sha, DamageNature::Normal, m.targets.clone()));
    let n = list.len();
    for t in list {
        if ctx.core.player(t).alive
            && !d34_l28_mods::shengong_excludes_target(
                ctx.core,
                ctx.catalog,
                owner,
                t,
                card,
                kind,
                nature,
                n,
            )
        {
            set_flag(ctx.core, t, &format!("chixueqingfeng2_frame:{frame_id}"), 1);
            let cur = flag(ctx.core, t, "chixueqingfeng2_count");
            set_flag(ctx.core, t, "chixueqingfeng2_count", cur + 1);
        }
    }
    Flow::Done
}

fn step_bintieshuangji(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectNature, ASK_SG_BINTIE_CONFIRM),
        HostStage::SelectNature => {
            let confirmed = matches!(ctx.answer().first().copied(), Some(ChoiceOption::Confirm));
            set_flag(ctx.core, owner, "l28_ask_id", 0);
            if !confirmed {
                return Flow::Done;
            }
            ctx.emit(EventData::LoseHp { player: owner, amount: 1 });
            m.stage = HostStage::DrawSettle;
            Flow::Yield
        }
        HostStage::DrawSettle => {
            if !ctx.core.player(owner).alive {
                return Flow::Done;
            }
            if let Some(c) = m.card {
                if ctx.core.table.contains(&c) {
                    ctx.core.table.retain(|&x| x != c);
                    ctx.core.player_mut(owner).hand.push(c);
                } else if ctx.core.discard.contains(&c) {
                    ctx.core.discard.retain(|&x| x != c);
                    ctx.core.player_mut(owner).hand.push(c);
                }
            }
            ctx.start_draw(owner, 1);
            let stamp = d34_l28_mods::l28_turn_stamp(ctx.core);
            let prev = if flag(ctx.core, owner, "bintieshuangji_sha_stamp") == stamp {
                flag(ctx.core, owner, "bintieshuangji_sha_bonus")
            } else {
                0
            };
            set_flag(ctx.core, owner, "bintieshuangji_sha_stamp", stamp);
            set_flag(ctx.core, owner, "bintieshuangji_sha_bonus", prev + 1);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_wutiesuolian(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let frame_u = ctx.res.frame(ctx.current_event_frame_id()).and_then(|f| match &f.data {
        EventData::CardUsed(u) => Some((
            u.card,
            u.use_as.unwrap_or_else(|| u.effective_kind(ctx.catalog)),
            u.nature,
            u.targets.clone(),
        )),
        _ => None,
    });
    let (card, kind, nature, list) = frame_u.unwrap_or((m.card, CardKind::Sha, DamageNature::Normal, m.targets.clone()));
    let n = list.len();
    for t in list {
        if ctx.core.player(t).alive
            && !ctx.core.player(t).linked
            && !d34_l28_mods::shengong_excludes_target(
                ctx.core,
                ctx.catalog,
                owner,
                t,
                card,
                kind,
                nature,
                n,
            )
        {
            ctx.emit(EventData::Linked { player: t, linked: true });
        }
    }
    Flow::Done
}

fn step_wuxinghelingshan(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            set_flag(ctx.core, owner, "l28_wuxing_frame", ctx.current_event_frame_id());
            wait_ask(m, ctx, HostStage::SelectNature, ASK_SG_WUXING_NATURE)
        }
        HostStage::SelectNature => {
            let confirmed = matches!(ctx.answer().first().copied(), Some(ChoiceOption::Confirm));
            set_flag(ctx.core, owner, "l28_ask_id", 0);
            let frame_id = flag(ctx.core, owner, "l28_wuxing_frame");
            set_flag(ctx.core, owner, "l28_wuxing_frame", 0);
            if confirmed && frame_id > 0 {
                if let Some(f) = ctx.res.frame_mut(frame_id) {
                    if let EventData::CardUsed(u) = &mut f.data {
                        u.nature = if u.nature == DamageNature::Fire {
                            DamageNature::Thunder
                        } else {
                            DamageNature::Fire
                        };
                        let is_converted_or_virtual = u
                            .card
                            .is_none_or(|c| u.use_as.is_some_and(|k| k != ctx.catalog.card_kind(c)));
                        if is_converted_or_virtual
                            && matches!(u.use_as, Some(CardKind::FireSha | CardKind::LeiSha))
                        {
                            u.use_as = Some(if u.nature == DamageNature::Fire {
                                CardKind::FireSha
                            } else {
                                CardKind::LeiSha
                            });
                        }
                    }
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_linglongshimandai(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            set_flag(ctx.core, owner, "l28_linglong_frame", ctx.current_event_frame_id());
            wait_ask(m, ctx, HostStage::SelectNature, ASK_SG_LINGLONG_CONFIRM)
        }
        HostStage::SelectNature => {
            let confirmed = matches!(ctx.answer().first().copied(), Some(ChoiceOption::Confirm));
            set_flag(ctx.core, owner, "l28_ask_id", 0);
            let frame_id = flag(ctx.core, owner, "l28_linglong_frame");
            set_flag(ctx.core, owner, "l28_linglong_frame", 0);
            if confirmed && frame_id > 0 && (!ctx.core.deck.is_empty() || !ctx.core.discard.is_empty())
            {
                let judged = ctx.draw_judge_card_with_elo(
                    owner,
                    engine_native_kernel::kernel::EloJudgePref::HeartPositive,
                );
                let (suit, _) = ctx.inline_judge_suit_rank(owner, judged);
                let is_heart = suit.is_heart();
                let _ = ctx.settle_inline_judge_card(owner, suit, judged);
                if is_heart {
                    if let Some(f) = ctx.res.frame_mut(frame_id) {
                        if let EventData::CardUsed(u) = &mut f.data {
                            u.targets.retain(|&t| t != owner);
                            if u.targets.is_empty() {
                                u.cancelled = true;
                            }
                        }
                    }
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_huxinjing(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            set_flag(ctx.core, owner, "l28_huxinjing_frame", ctx.current_event_frame_id());
            wait_ask(m, ctx, HostStage::SelectNature, ASK_SG_HUXINJING_CONFIRM)
        }
        HostStage::SelectNature => {
            let confirmed = matches!(ctx.answer().first().copied(), Some(ChoiceOption::Confirm));
            set_flag(ctx.core, owner, "l28_ask_id", 0);
            let frame_id = flag(ctx.core, owner, "l28_huxinjing_frame");
            set_flag(ctx.core, owner, "l28_huxinjing_frame", 0);
            if confirmed && frame_id > 0 {
                if let Some(f) = ctx.res.frame_mut(frame_id) {
                    if let EventData::Damage(d) = &mut f.data {
                        d.prevented = true;
                    }
                }
                let hx_cards: Vec<CardId> = ctx
                    .core
                    .player(owner)
                    .equips
                    .iter()
                    .copied()
                    .filter(|&c| {
                        ctx.catalog
                            .cards
                            .get(c as usize)
                            .is_some_and(|cd| cd.name == "huxinjing")
                    })
                    .collect();
                if !hx_cards.is_empty() {
                    discard_cards(ctx, owner, &hx_cards);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_shufazijinguan(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_SG_SHUFA_TARGET),
        HostStage::TargetChoice => {
            let ans = ctx.answer().first().copied();
            set_flag(ctx.core, owner, "l28_ask_id", 0);
            if let Some(ChoiceOption::Player(t)) = ans {
                if ctx.core.player(t).alive {
                    ctx.emit(EventData::Damage(damage_data(Some(owner), t, 1)));
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_tianjitu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_SG_TIANJITU_DISCARD),
        HostStage::SelectCard => {
            let ans = ctx.answer().first().copied();
            set_flag(ctx.core, owner, "l28_ask_id", 0);
            if let Some(ChoiceOption::Card(c)) = ans {
                discard_cards(ctx, owner, &[c]);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_taigong_link(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_SG_TAIGONG_TARGET),
        HostStage::TargetChoice => {
            let ans = ctx.answer().first().copied();
            set_flag(ctx.core, owner, "l28_ask_id", 0);
            if let Some(ChoiceOption::Player(t)) = ans {
                if ctx.core.player(t).alive {
                    let next = !ctx.core.player(t).linked;
                    ctx.emit(EventData::Linked { player: t, linked: next });
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_taigong_recast(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_SG_TAIGONG_RECAST),
        HostStage::SelectCard => {
            let ans = ctx.answer().first().copied();
            set_flag(ctx.core, owner, "l28_ask_id", 0);
            if let Some(ChoiceOption::Card(c)) = ans {
                if ctx.core.player(owner).hand.contains(&c) {
                    ctx.take_hand_card(owner, c);
                    ctx.to_discard(c);
                    ctx.emit(EventData::CardsLost {
                        player: owner,
                        cards: vec![c],
                        origins: vec![LostOrigin::Hand],
                    });
                    ctx.start_draw(owner, 1);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_zhaogujing(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_SG_ZHAOGUJING_CARD),
        HostStage::SelectCard => {
            let ans = ctx.answer().first().copied();
            set_flag(ctx.core, owner, "l28_ask_id", 0);
            let Some(ChoiceOption::Card(c)) = ans else {
                return Flow::Done;
            };
            m.card = Some(c);
            if zhaogujing_is_huogong(ctx.catalog, c) {
                if zhaogujing_huogong_targets(ctx.core, ctx.catalog, owner).is_empty() {
                    return Flow::Done;
                }
                return wait_ask(m, ctx, HostStage::TargetChoice, ASK_SG_ZHAOGUJING_TARGET);
            }
            let kind = ctx.catalog.card_kind(c);
            if zhaogujing_needs_target(kind) {
                if zhaogujing_targets_for(ctx.core, ctx.catalog, owner, kind).is_empty() {
                    return Flow::Done;
                }
                return wait_ask(m, ctx, HostStage::TargetChoice, ASK_SG_ZHAOGUJING_TARGET);
            }
            let targets = match kind {
                CardKind::Tao | CardKind::Jiu | CardKind::Wuzhong => vec![owner],
                CardKind::SavageAttack | CardKind::ArcheryAttack => alive_others(ctx.core, owner),
                CardKind::Taoyuan | CardKind::Wugu => Vec::new(),
                _ => return Flow::Done,
            };
            ctx.emit(EventData::CardUsed(CardUseData {
                card: None,
                player: owner,
                targets,
                nature: kind_nature(kind),
                use_as: Some(kind),
                cancelled: false,
            }));
            Flow::Done
        }
        HostStage::TargetChoice => {
            let ans = ctx.answer().clone();
            set_flag(ctx.core, owner, "l28_ask_id", 0);
            let Some(c) = m.card else {
                return Flow::Done;
            };
            if zhaogujing_is_huogong(ctx.catalog, c) {
                let Some(t) = ans.iter().find_map(|o| match *o {
                    ChoiceOption::Player(t) if ctx.core.player(t).alive => Some(t),
                    _ => None,
                }) else {
                    return Flow::Done;
                };
                m.target = Some(t);
                let Some(&shown) = ctx.core.player(t).hand.first() else {
                    return Flow::Done;
                };
                let suit = ctx.catalog.effective_card_suit(ctx.core, t, shown);
                let has_match = ctx
                    .core
                    .player(owner)
                    .hand
                    .iter()
                    .any(|&hc| ctx.catalog.effective_card_suit(ctx.core, owner, hc) == suit);
                return if has_match {
                    wait_ask(m, ctx, HostStage::SelectNature, ASK_SG_ZHAOGUJING_HUOGONG_DISCARD)
                } else {
                    Flow::Done
                };
            }
            let kind = ctx.catalog.card_kind(c);
            if kind == CardKind::Jiedao {
                let Some(holder) = ans.iter().find_map(|o| match *o {
                    ChoiceOption::Player(t) if ctx.core.player(t).alive => Some(t),
                    _ => None,
                }) else {
                    return Flow::Done;
                };
                m.target = Some(holder);
                return if zhaogujing_jiedao_victims(ctx.core, ctx.catalog, holder).is_empty() {
                    Flow::Done
                } else {
                    wait_ask(m, ctx, HostStage::SelectNature, ASK_SG_ZHAOGUJING_JIEDAO_VICTIM)
                };
            }
            let targets: Vec<PlayerId> = ans
                .into_iter()
                .filter_map(|o| match o {
                    ChoiceOption::Player(t) if ctx.core.player(t).alive => Some(t),
                    _ => None,
                })
                .collect();
            if targets.is_empty() {
                return Flow::Done;
            }
            ctx.emit(EventData::CardUsed(CardUseData {
                card: None,
                player: owner,
                targets,
                nature: kind_nature(kind),
                use_as: Some(kind),
                cancelled: false,
            }));
            Flow::Done
        }
        HostStage::SelectNature => {
            let ans = ctx.answer().first().copied();
            set_flag(ctx.core, owner, "l28_ask_id", 0);
            let Some(c) = m.card else {
                return Flow::Done;
            };
            if zhaogujing_is_huogong(ctx.catalog, c) {
                if let (Some(t), Some(ChoiceOption::Card(disc))) = (m.target, ans) {
                    if ctx.core.player(owner).hand.contains(&disc) && ctx.core.player(t).alive {
                        discard_cards(ctx, owner, &[disc]);
                        ctx.emit(EventData::Damage(damage_data_with(
                            Some(owner),
                            t,
                            1,
                            DamageNature::Fire,
                            None,
                        )));
                    }
                }
                return Flow::Done;
            }
            if ctx.catalog.card_kind(c) == CardKind::Jiedao {
                if let (Some(holder), Some(ChoiceOption::Player(victim))) = (m.target, ans) {
                    if ctx.core.player(holder).alive && ctx.core.player(victim).alive {
                        ctx.emit(EventData::CardUsed(CardUseData {
                            card: None,
                            player: owner,
                            targets: vec![holder, victim],
                            nature: DamageNature::Normal,
                            use_as: Some(CardKind::Jiedao),
                            cancelled: false,
                        }));
                    }
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}
