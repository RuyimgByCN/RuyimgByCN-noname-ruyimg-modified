//! Batch U05-6a (`shahai-hearth` Part 1): `card/hearth/{card,skill}.js` 13 牌 + 2(4) 技与
//! `character/hearth/card.js` 44 张衍生卡的真实核心玩法结算（仅形式包 `hearth_formal_enabled` 生效）。

use super::u05_hearth::{
    base_card_name, change_max_hp, discard_card_from_zone, equip_card_to_player, exact_base_name,
    finish_table_card_hearth, flag, gain_card_to_hand, grant_skill, has_any_skill,
    hearth_formal_enabled, is_hearth_enemy, make_damage, meta, remove_skill, rng_pick, set_flag,
    spawn_card_by_kind,
};
use super::u05_hearth_asks::{
    set_active_ask, wait_hs, ASK_HS_CHUANSONGMEN_TARGET, ASK_HS_CHUANSONGMEN_USE,
    ASK_HS_DUNPAIGEDANG_DISCARD, ASK_HS_SHANDIANJIAN_DISCARD, ASK_HS_SHANDIANJIAN_SHOW,
    ASK_HS_SHIJIESHU_DISCARD, ASK_HS_ZHAOMINGDAN_DISCARD,
};
use super::*;
use crate::catalog::{CardKind, EquipSlot, QueryCtx, SkillMachine, TriggerMeta};
use crate::choice::ChoiceOption;
use crate::event::{CardUseData, DamageNature, EventData, EventKind, Timing};
use crate::ids::{CardId, PlayerId};
use crate::kernel::wrap_nullify_window;
use crate::machine::{Ctx, Flow, LogEvent};

pub(super) fn trigger_meta_cards(name: &str) -> Option<TriggerMeta> {
    match name {
        "hsmengjing_mengye" => Some(meta(
            EventKind::PhaseBegin,
            Timing::Normal,
            -1,
            false,
            Vec::new(),
        )),
        _ => None,
    }
}

pub(super) fn filter_cards(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    match skill.name.as_str() {
        "hsmengjing_mengye" => matches!(
            event,
            EventData::Phase {
                player,
                phase: crate::event::Phase::End,
                begin: true,
            } if *player == owner
                && (!ctx.state.player(owner).hand.is_empty()
                    || !ctx.state.player(owner).equips.is_empty())
        ),
        _ => false,
    }
}

fn spawn_machine(
    def_key: &str,
    u: &CardUseData,
    card: CardId,
    stage: HostStage,
    target: Option<PlayerId>,
    targets: Vec<PlayerId>,
) -> Box<dyn SkillMachine> {
    Box::new(HostMachine {
        def_key: def_key.to_string(),
        behavior: HostBehavior::U05HearthCards,
        owner: u.player,
        target,
        card: (card != u32::MAX).then_some(card),
        cards: Vec::new(),
        targets,
        stage,
    })
}

fn spawn_wrapped(
    ctx: &mut Ctx<'_>,
    def_key: &str,
    u: &CardUseData,
    card: CardId,
    stage: HostStage,
    target: Option<PlayerId>,
    targets: Vec<PlayerId>,
) -> Option<Box<dyn SkillMachine>> {
    let eff = spawn_machine(def_key, u, card, stage, target, targets);
    if card == u32::MAX {
        Some(eff)
    } else {
        Some(Box::new(wrap_nullify_window(ctx, card, u.player, eff)))
    }
}

fn is_basic_non_du(catalog: &crate::catalog::Catalog, cid: CardId) -> bool {
    let def = &catalog.cards[cid as usize];
    exact_base_name(def.name) != "du" && def.name != "毒" && def.kind.is_basic_kind()
}

pub(crate) fn is_hearth_named_wuzhong_unwrapped(
    catalog: &crate::catalog::Catalog,
    card: CardId,
) -> bool {
    if !hearth_formal_enabled(catalog) {
        return false;
    }
    let Some(raw) = catalog.cards.get(card as usize).map(|c| c.name) else {
        return false;
    };
    let b = exact_base_name(raw);
    if b == "wuzhong" || b == "无中生有" {
        return false;
    }
    // `card/hearth/card.js` trick cards (`shandianjian`, `shenenshu`) ARE wuxieable (NOT unwrapped)!
    // Non-trick `character/hearth/card.js` & generated `hsyaoshui` / `hsnitai_card` ARE unwrapped.
    b == "shihuawuqi"
        || b == "hsyaoshui"
        || b == "hsnitai_card"
        || b == "hsfashu_buwendingyibian"
        || b == "hsjixie_zhadan"
        || b.starts_with("hsqizhou_")
        || b.starts_with("hsqingyu_")
        || b.starts_with("hsdusu_")
        || b.starts_with("hsbaowu_")
        || b.starts_with("hsshenqi_")
        || b.starts_with("hsmengjing_")
        || b.starts_with("hszuzhou_")
}

pub(crate) fn hearth_try_named_ext_card(
    ctx: &mut Ctx<'_>,
    u: &CardUseData,
    card: CardId,
    effective_kind: CardKind,
) -> Option<Option<Box<dyn SkillMachine>>> {
    if !hearth_formal_enabled(ctx.catalog) {
        return None;
    }
    let raw = ctx
        .catalog
        .cards
        .get(card as usize)
        .map(|c| c.name)
        .unwrap_or("");
    let exact = exact_base_name(raw);
    let virtual_src = flag(ctx.core, u.player, "hs_virtual_card_src");
    let has_virtual_src = virtual_src > 0 && effective_kind == CardKind::Wuzhong;
    let mut dispatch_raw = raw;
    let mut shadow_copied_wuzhong = false;
    // Check `wxuying` transformed `hsfashu_anyingjingxiang` (keep copied card identity!)
    if card != u32::MAX && exact == "hsfashu_anyingjingxiang" {
        let kind_code = flag(ctx.core, u.player, &format!("hs_wxuying_kind:{card}"));
        let was_shadow = flag(ctx.core, u.player, &format!("hs_wxuying_was_shadow:{card}")) > 0
            || kind_code > 0;
        if was_shadow {
            let src_plus = flag(ctx.core, u.player, &format!("hs_wxuying_src_card:{card}"));
            if src_plus > 0 {
                if let Some(cdef) = ctx.catalog.cards.get((src_plus - 1) as usize) {
                    dispatch_raw = cdef.name;
                }
            }
            set_flag(ctx.core, u.player, &format!("hs_wxuying_kind:{card}"), 0);
            set_flag(ctx.core, u.player, &format!("hs_wxuying_shadow:{card}"), 0);
            set_flag(ctx.core, u.player, &format!("hs_wxuying_was_shadow:{card}"), 1);
            shadow_copied_wuzhong = true;
        } else if u.use_as.is_none() {
            return Some(None);
        }
    }
    if has_virtual_src {
        set_flag(ctx.core, u.player, "hs_virtual_card_src", 0);
        if let Some(cdef) = ctx.catalog.cards.get((virtual_src - 1) as usize) {
            dispatch_raw = cdef.name;
        }
    }
    let dispatch_exact = exact_base_name(dispatch_raw);
    let base = base_card_name(dispatch_raw);
    // Check `shihuawuqi` first (`type: "basic"`, uses CardKind::Tao in Catalog)
    if (u.use_as.is_none() || shadow_copied_wuzhong)
        && base == "shihuawuqi"
        && effective_kind == CardKind::Tao
    {
        return Some(Some(spawn_machine(
            "hs_card:shihuawuqi",
            u,
            card,
            HostStage::Act,
            Some(u.player),
            Vec::new(),
        )));
    }
    if shadow_copied_wuzhong
        && effective_kind == CardKind::Wuzhong
        && (dispatch_exact == "hsfashu_anyingjingxiang"
            || dispatch_exact == "wuzhong"
            || dispatch_exact == "无中生有"
            || dispatch_raw.starts_with("wuzhong_"))
    {
        // Delegate plain Wuzhong copy cleanly to kernel `WuzhongMachine` (`CardKind::Wuzhong`)
        return None;
    }
    // Intercept standard `card/hearth/card.js` CardKinds ONLY in formal hearth so `bridge_pack` tests remain exact
    match effective_kind {
        CardKind::Linghunzhihuo => {
            return Some(spawn_wrapped(
                ctx,
                "hs_card:linghunzhihuo",
                u,
                card,
                HostStage::Act,
                u.targets.first().copied(),
                Vec::new(),
            ));
        }
        CardKind::Zhaomingdan => {
            return Some(spawn_wrapped(
                ctx,
                "hs_card:zhaomingdan",
                u,
                card,
                HostStage::Act,
                u.targets.first().copied(),
                Vec::new(),
            ));
        }
        CardKind::Dunpaigedang => {
            return Some(spawn_wrapped(
                ctx,
                "hs_card:dunpaigedang",
                u,
                card,
                HostStage::Act,
                Some(u.player),
                Vec::new(),
            ));
        }
        CardKind::Chuansongmen => {
            return Some(spawn_wrapped(
                ctx,
                "hs_card:chuansongmen",
                u,
                card,
                HostStage::Act,
                Some(u.player),
                Vec::new(),
            ));
        }
        CardKind::Tanshezhiren => {
            return Some(spawn_wrapped(
                ctx,
                "hs_card:tanshezhiren",
                u,
                card,
                HostStage::Act,
                Some(u.player),
                Vec::new(),
            ));
        }
        CardKind::Shijieshu => {
            return Some(spawn_wrapped(
                ctx,
                "hs_card:shijieshu",
                u,
                card,
                HostStage::Act,
                u.targets.first().copied(),
                Vec::new(),
            ));
        }
        CardKind::Yuansuhuimie => {
            // `reverseOrder: true` on `filterTarget: true` => reverse seat order from caster
            let mut order: Vec<PlayerId> = if u.targets.is_empty() {
                ctx.core.seat_order_from(u.player)
            } else {
                ctx.core
                    .seat_order_from(u.player)
                    .into_iter()
                    .filter(|p| u.targets.contains(p))
                    .collect()
            };
            order.reverse();
            return Some(spawn_wrapped(
                ctx,
                "hs_card:yuansuhuimie",
                u,
                card,
                HostStage::Act,
                None,
                order,
            ));
        }
        CardKind::Xingjiegoutong => {
            return Some(spawn_wrapped(
                ctx,
                "hs_card:xingjiegoutong",
                u,
                card,
                HostStage::Act,
                Some(u.player),
                Vec::new(),
            ));
        }
        CardKind::Jihuocard if base == "shenenshu" => {
            return Some(spawn_wrapped(
                ctx,
                "hs_card:shenenshu",
                u,
                card,
                HostStage::Act,
                Some(u.player),
                Vec::new(),
            ));
        }
        CardKind::Jihuocard => {
            return Some(spawn_wrapped(
                ctx,
                "hs_card:jihuocard",
                u,
                card,
                HostStage::Act,
                Some(u.player),
                Vec::new(),
            ));
        }
        _ => {
            if u.use_as.is_some() && !has_virtual_src && !shadow_copied_wuzhong {
                return None;
            }
        }
    }
    // Check named trick `shandianjian` + `character/hearth/card.js`
    if base == "shandianjian" {
        return Some(spawn_wrapped(
            ctx,
            "hs_card:shandianjian",
            u,
            card,
            HostStage::Act,
            u.targets.first().copied(),
            Vec::new(),
        ));
    }
    if base == "shenenshu" {
        return Some(spawn_wrapped(
            ctx,
            "hs_card:shenenshu",
            u,
            card,
            HostStage::Act,
            Some(u.player),
            Vec::new(),
        ));
    }
    if base == "huogong" && (has_virtual_src || shadow_copied_wuzhong) {
        crate::content::host::intrinsic::resolve_huogong_effect(ctx, u);
        if card != u32::MAX {
            finish_table_card_hearth(ctx, card, false);
        }
        return Some(None);
    }
    if dispatch_exact == "hsfashu_buwendingyibian"
        || dispatch_exact == "hsjixie_zhadan"
        || dispatch_exact == "hsyaoshui"
        || dispatch_exact == "hsnitai_card"
        || dispatch_exact.starts_with("hsqizhou_")
        || dispatch_exact.starts_with("hsqingyu_")
        || dispatch_exact.starts_with("hsdusu_")
        || dispatch_exact.starts_with("hsbaowu_")
        || dispatch_exact.starts_with("hsshenqi_")
        || dispatch_exact.starts_with("hsmengjing_")
        || dispatch_exact.starts_with("hszuzhou_")
    {
        return Some(Some(spawn_machine(
            &format!("hs_card:{dispatch_exact}"),
            u,
            card,
            HostStage::Act,
            u.targets.first().copied(),
            u.targets.clone(),
        )));
    }
    None
}

pub(super) fn step_hearth_cards(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let key = m.def_key.as_str();
    let owner = m.owner;
    match key {
        "hs_card:shihuawuqi" => {
            let used = flag(ctx.core, owner, "hs_shihuawuqi_used_turn") + 1;
            set_flag(ctx.core, owner, "hs_shihuawuqi_used_turn", used);
            grant_skill(ctx, owner, "shihuawuqi", true);
            let has_sha = ctx
                .core
                .player(owner)
                .hand
                .iter()
                .any(|&c| ctx.catalog.card_kind(c).is_sha_like());
            if !has_sha {
                if let Some(sha_c) = ctx
                    .core
                    .deck
                    .iter()
                    .rposition(|&c| ctx.catalog.card_kind(c).is_sha_like())
                    .map(|idx| ctx.core.deck.remove(idx))
                    .or_else(|| {
                        ctx.core
                            .discard
                            .iter()
                            .rposition(|&c| ctx.catalog.card_kind(c).is_sha_like())
                            .map(|idx| ctx.core.discard.remove(idx))
                    })
                {
                    gain_card_to_hand(ctx, owner, sha_c);
                }
            }
            if let Some(c) = m.card {
                finish_table_card_hearth(ctx, c, false);
            }
            Flow::Done
        }
        "hs_card:jihuocard" => {
            if ctx.core.turn.current == owner {
                grant_skill(ctx, owner, "jihuocard2", true);
            }
            ctx.start_draw(owner, 1);
            if let Some(c) = m.card {
                finish_table_card_hearth(ctx, c, false);
            }
            Flow::Done
        }
        "hsmengjing_mengye" => {
            let mut discarded = ctx.core.player(owner).hand.clone();
            discarded.extend(ctx.core.player(owner).equips.iter().copied());
            for &dc in &discarded {
                discard_card_from_zone(ctx, owner, dc);
            }
            if !discarded.is_empty() {
                ctx.log(LogEvent::CardDiscarded {
                    player: owner,
                    cards: discarded,
                });
            }
            remove_skill(ctx, owner, "hsmengjing_mengye");
            Flow::Done
        }
        "hs_card:linghunzhihuo" => step_linghunzhihuo(m, ctx),
        "hs_card:zhaomingdan" => step_zhaomingdan(m, ctx),
        "hs_card:dunpaigedang" => step_dunpaigedang(m, ctx),
        "hs_card:chuansongmen" => step_chuansongmen(m, ctx),
        "hs_card:shijieshu" => step_shijieshu(m, ctx),
        "hs_card:shandianjian" => step_shandianjian(m, ctx),
        "hs_card:yuansuhuimie" => step_yuansuhuimie(m, ctx),
        "hs_card:xingjiegoutong" => {
            change_max_hp(ctx, owner, 1);
            ctx.recover(owner, 1, None);
            let hand = ctx.core.player(owner).hand.clone();
            for &hc in &hand {
                discard_card_from_zone(ctx, owner, hc);
            }
            if !hand.is_empty() {
                ctx.log(LogEvent::CardDiscarded {
                    player: owner,
                    cards: hand,
                });
            }
            if let Some(c) = m.card {
                finish_table_card_hearth(ctx, c, false);
            }
            Flow::Done
        }
        "hs_card:tanshezhiren" => step_tanshezhiren(m, ctx),
        "hs_card:shenenshu" => step_shenenshu(m, ctx),
        _ => super::u05_hearth_derived_cards::step_hearth_derived_card(m, ctx),
    }
}

fn step_linghunzhihuo(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            m.stage = HostStage::DrawSettle;
            if let Some(t) = m.target {
                if ctx.core.player(t).alive {
                    ctx.emit(EventData::Damage(make_damage(
                        Some(owner),
                        t,
                        1,
                        DamageNature::Fire,
                        m.card,
                    )));
                    return Flow::Yield;
                }
            }
            step_linghunzhihuo(m, ctx)
        }
        HostStage::DrawSettle => {
            if ctx.core.player(owner).alive {
                if let Some(rand_c) = rng_pick(ctx, &ctx.core.player(owner).hand.clone()) {
                    ctx.discard_from_hand(owner, rand_c);
                    ctx.log(LogEvent::CardDiscarded {
                        player: owner,
                        cards: vec![rand_c],
                    });
                }
            }
            if let Some(c) = m.card {
                finish_table_card_hearth(ctx, c, false);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_zhaomingdan(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let Some(t) = m.target else {
        if let Some(c) = m.card {
            finish_table_card_hearth(ctx, c, false);
        }
        return Flow::Done;
    };
    match m.stage {
        HostStage::Act => {
            let pl = ctx.core.player(t);
            if pl.hand.is_empty() && pl.equips.is_empty() && pl.judge.is_empty() {
                m.stage = HostStage::DrawSettle;
                return step_zhaomingdan(m, ctx);
            }
            wait_hs(m, ctx, HostStage::SelectCard, ASK_HS_ZHAOMINGDAN_DISCARD)
        }
        HostStage::SelectCard => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::SelectCard, 0);
            if let Some(ChoiceOption::Card(picked)) = ctx.answer().first().copied() {
                discard_card_from_zone(ctx, t, picked);
                ctx.log(LogEvent::CardDiscarded {
                    player: t,
                    cards: vec![picked],
                });
            }
            m.stage = HostStage::DrawSettle;
            step_zhaomingdan(m, ctx)
        }
        HostStage::DrawSettle => {
            m.stage = HostStage::CardsAsk;
            if ctx.core.player(t).alive {
                ctx.start_draw(t, 1);
                return Flow::Yield;
            }
            step_zhaomingdan(m, ctx)
        }
        HostStage::CardsAsk => {
            m.stage = HostStage::TargetChoice;
            if ctx.core.player(m.owner).alive {
                ctx.start_draw(m.owner, 1);
                return Flow::Yield;
            }
            step_zhaomingdan(m, ctx)
        }
        HostStage::TargetChoice => {
            if let Some(c) = m.card {
                finish_table_card_hearth(ctx, c, false);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_dunpaigedang(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => {
            ctx.change_hujia(m.owner, 1);
            ctx.start_draw(m.owner, 1);
            m.stage = HostStage::DrawSettle;
            Flow::Yield
        }
        HostStage::DrawSettle => {
            let pl = ctx.core.player(m.owner);
            if pl.hand.is_empty() && pl.equips.is_empty() {
                if let Some(c) = m.card {
                    finish_table_card_hearth(ctx, c, false);
                }
                return Flow::Done;
            }
            wait_hs(m, ctx, HostStage::SelectCard, ASK_HS_DUNPAIGEDANG_DISCARD)
        }
        HostStage::SelectCard => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::SelectCard, 0);
            if let Some(ChoiceOption::Card(picked)) = ctx.answer().first().copied() {
                discard_card_from_zone(ctx, m.owner, picked);
                ctx.log(LogEvent::CardDiscarded {
                    player: m.owner,
                    cards: vec![picked],
                });
            }
            if let Some(c) = m.card {
                finish_table_card_hearth(ctx, c, false);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_chuansongmen(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => {
            m.cards = ctx.core.player(m.owner).hand.clone();
            ctx.start_draw(m.owner, 1);
            m.stage = HostStage::DrawSettle;
            Flow::Yield
        }
        HostStage::DrawSettle => {
            let gained = ctx
                .core
                .player(m.owner)
                .hand
                .iter()
                .copied()
                .rfind(|c| !m.cards.contains(c));
            m.cards.clear();
            let Some(gained) = gained else {
                if let Some(c) = m.card {
                    finish_table_card_hearth(ctx, c, false);
                }
                return Flow::Done;
            };
            let in_play = ctx.core.turn.current == m.owner
                && ctx.core.turn.phase == crate::event::Phase::Play
                && super::u05_hearth_asks::is_tanmi_card_usable(
                    ctx.core,
                    ctx.catalog,
                    m.owner,
                    gained,
                );
            if !in_play {
                if let Some(c) = m.card {
                    finish_table_card_hearth(ctx, c, false);
                }
                return Flow::Done;
            }
            m.cards = vec![gained];
            wait_hs(m, ctx, HostStage::SelectCard, ASK_HS_CHUANSONGMEN_USE)
        }
        HostStage::SelectCard => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::SelectCard, 0);
            let Some(ChoiceOption::Card(picked)) = ctx.answer().first().copied() else {
                if let Some(c) = m.card {
                    finish_table_card_hearth(ctx, c, false);
                }
                return Flow::Done;
            };
            if super::u05_hearth_asks::hearth_card_needs_target(
                ctx.core,
                ctx.catalog,
                m.owner,
                picked,
            ) {
                return wait_hs(m, ctx, HostStage::TargetChoice, ASK_HS_CHUANSONGMEN_TARGET);
            }
            finish_chuansongmen_use(m, ctx, picked, Vec::new())
        }
        HostStage::TargetChoice => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::TargetChoice, 0);
            let Some(&picked) = m.cards.first() else {
                if let Some(c) = m.card {
                    finish_table_card_hearth(ctx, c, false);
                }
                return Flow::Done;
            };
            let picked_targets = super::u05_hearth_asks::collect_player_answers(ctx.answer());
            if picked_targets.is_empty() {
                if let Some(c) = m.card {
                    finish_table_card_hearth(ctx, c, false);
                }
                return Flow::Done;
            }
            finish_chuansongmen_use(m, ctx, picked, picked_targets)
        }
        _ => Flow::Done,
    }
}

fn finish_chuansongmen_use(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    picked: CardId,
    targets: Vec<PlayerId>,
) -> Flow {
    super::u05_hearth_a::use_gained_card(ctx, m.owner, picked, targets);
    if let Some(c) = m.card {
        ctx.core.table.retain(|&x| x != c);
        if !has_any_skill(ctx.core, ctx.catalog, m.owner, "chuansongmen3") {
            if has_any_skill(ctx.core, ctx.catalog, m.owner, "chuansongmen2") {
                grant_skill(ctx, m.owner, "chuansongmen3", true);
            } else {
                grant_skill(ctx, m.owner, "chuansongmen2", true);
            }
            gain_card_to_hand(ctx, m.owner, c);
        } else {
            finish_table_card_hearth(ctx, c, false);
        }
    }
    Flow::Done
}

fn step_shijieshu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let Some(t) = m.target else {
        if let Some(c) = m.card {
            finish_table_card_hearth(ctx, c, false);
        }
        return Flow::Done;
    };
    match m.stage {
        HostStage::Act => {
            let mut picked = Vec::new();
            let mut seen_slots: Vec<EquipSlot> = Vec::new();
            let mut i = ctx.core.deck.len();
            while i > 0 && picked.len() < 2 {
                i -= 1;
                let cid = ctx.core.deck[i];
                let Some(slot) = ctx.catalog.card_kind(cid).equip_slot() else {
                    continue;
                };
                if seen_slots.contains(&slot) {
                    continue;
                }
                seen_slots.push(slot);
                picked.push(cid);
                ctx.core.deck.remove(i);
            }
            m.cards = picked;
            m.stage = HostStage::TargetChoice;
            step_shijieshu(m, ctx)
        }
        HostStage::TargetChoice => {
            if !ctx.core.player(t).alive {
                for ec in m.cards.drain(..) {
                    finish_table_card_hearth(ctx, ec, false);
                }
                if let Some(c) = m.card {
                    finish_table_card_hearth(ctx, c, false);
                }
                return Flow::Done;
            }
            if !m.cards.is_empty() {
                let ec = m.cards.remove(0);
                equip_card_to_player(ctx, t, ec);
                return Flow::Yield;
            }
            let pl = ctx.core.player(t);
            if pl.hand.is_empty() && pl.equips.is_empty() {
                if let Some(c) = m.card {
                    finish_table_card_hearth(ctx, c, false);
                }
                return Flow::Done;
            }
            wait_hs(m, ctx, HostStage::SelectCard, ASK_HS_SHIJIESHU_DISCARD)
        }
        HostStage::SelectCard => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::SelectCard, 0);
            if let Some(ChoiceOption::Card(picked)) = ctx.answer().first().copied() {
                discard_card_from_zone(ctx, t, picked);
                ctx.log(LogEvent::CardDiscarded {
                    player: t,
                    cards: vec![picked],
                });
            }
            if let Some(c) = m.card {
                finish_table_card_hearth(ctx, c, false);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_shandianjian(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let Some(t) = m.target else {
        if let Some(c) = m.card {
            finish_table_card_hearth(ctx, c, false);
        }
        return Flow::Done;
    };
    match m.stage {
        HostStage::Act => {
            let hand_len = ctx.core.player(t).hand.len();
            if hand_len == 0 {
                if let Some(c) = m.card {
                    finish_table_card_hearth(ctx, c, false);
                }
                return Flow::Done;
            }
            if hand_len == 1 {
                m.cards = vec![ctx.core.player(t).hand[0]];
                return shandianjian_prompt_discard(m, ctx, t);
            }
            wait_hs(m, ctx, HostStage::SelectCard, ASK_HS_SHANDIANJIAN_SHOW)
        }
        HostStage::SelectCard => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::SelectCard, 0);
            let Some(ChoiceOption::Card(shown)) = ctx.answer().first().copied() else {
                if let Some(c) = m.card {
                    finish_table_card_hearth(ctx, c, false);
                }
                return Flow::Done;
            };
            m.cards = vec![shown];
            shandianjian_prompt_discard(m, ctx, t)
        }
        HostStage::TargetChoice => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::TargetChoice, 0);
            if let Some(ChoiceOption::Card(discarded)) = ctx.answer().first().copied() {
                discard_card_from_zone(ctx, m.owner, discarded);
                ctx.log(LogEvent::CardDiscarded {
                    player: m.owner,
                    cards: vec![discarded],
                });
                ctx.emit(EventData::Damage(make_damage(
                    Some(m.owner),
                    t,
                    1,
                    DamageNature::Thunder,
                    m.card,
                )));
            } else {
                set_flag(ctx.core, t, "hs_temp_huogong2", 1);
            }
            if let Some(c) = m.card {
                finish_table_card_hearth(ctx, c, false);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn shandianjian_prompt_discard(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    t: PlayerId,
) -> Flow {
    let shown = m.cards[0];
    let suit = ctx.catalog.effective_card_suit(ctx.core, t, shown);
    let can_match = ctx
        .core
        .player(m.owner)
        .hand
        .iter()
        .any(|&c| ctx.catalog.effective_card_suit(ctx.core, m.owner, c) == suit);
    if !can_match {
        set_flag(ctx.core, t, "hs_temp_huogong2", 1);
        if let Some(c) = m.card {
            finish_table_card_hearth(ctx, c, false);
        }
        return Flow::Done;
    }
    wait_hs(m, ctx, HostStage::TargetChoice, ASK_HS_SHANDIANJIAN_DISCARD)
}

fn step_yuansuhuimie(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    loop {
        match m.stage {
            HostStage::Act => {
                let Some(t) = m.targets.first().copied() else {
                    if let Some(c) = m.card {
                        finish_table_card_hearth(ctx, c, false);
                    }
                    return Flow::Done;
                };
                m.targets.remove(0);
                if !ctx.core.player(t).alive {
                    continue;
                }
                m.target = Some(t);
                let opts: Vec<ChoiceOption> = ctx
                    .core
                    .player(t)
                    .hand
                    .iter()
                    .chain(ctx.core.player(t).equips.iter())
                    .copied()
                    .map(ChoiceOption::Card)
                    .collect();
                if opts.is_empty() {
                    ctx.emit(EventData::Damage(make_damage(
                        Some(m.owner),
                        t,
                        2,
                        DamageNature::Thunder,
                        m.card,
                    )));
                    continue;
                }
                let n = opts.len().min(2);
                m.stage = HostStage::TargetChoice;
                return Flow::WaitChoice {
                    observer: t,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: crate::content::host::HH_YUANSU_DISCARD,
                        options: opts,
                        min: 1,
                        max: n,
                        allow_cancel: true,
                    },
                };
            }
            HostStage::TargetChoice => {
                m.stage = HostStage::Act;
                let Some(t) = m.target else {
                    return Flow::Done;
                };
                let picked: Vec<CardId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    })
                    .collect();
                for &dc in &picked {
                    discard_card_from_zone(ctx, t, dc);
                }
                if !picked.is_empty() {
                    ctx.log(LogEvent::CardDiscarded {
                        player: t,
                        cards: picked.clone(),
                    });
                }
                let dmg = 2i32.saturating_sub(picked.len() as i32);
                if dmg > 0 {
                    ctx.emit(EventData::Damage(make_damage(
                        Some(m.owner),
                        t,
                        dmg,
                        DamageNature::Thunder,
                        m.card,
                    )));
                }
            }
            _ => return Flow::Done,
        }
    }
}

fn step_tanshezhiren(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let mut current = m.owner;
    let alive = ctx.core.alive_count();
    let mut times = if alive % 2 == 0 { alive - 1 } else { alive };
    while times > 0 {
        let enemies: Vec<PlayerId> = ctx
            .core
            .alive_players()
            .filter(|&p| {
                p != m.owner
                    && !ctx.core.player(p).hand.is_empty()
                    && is_hearth_enemy(ctx.core, ctx.catalog, current, p)
            })
            .collect();
        let Some(enemy) = rng_pick(ctx, &enemies) else {
            break;
        };
        if let Some(rc) = rng_pick(ctx, &ctx.core.player(enemy).hand.clone()) {
            ctx.discard_from_hand(enemy, rc);
            ctx.log(LogEvent::CardDiscarded {
                player: enemy,
                cards: vec![rc],
            });
        }
        current = enemy;
        times -= 1;
    }
    if let Some(c) = m.card {
        finish_table_card_hearth(ctx, c, false);
    }
    Flow::Done
}

fn step_shenenshu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let t = m.target.unwrap_or(m.owner);
    let old_hand = ctx.core.player(t).hand.clone();
    let need = 1 + old_hand.len();
    for hc in old_hand {
        ctx.core.player_mut(t).hand.retain(|&x| x != hc);
        finish_table_card_hearth(ctx, hc, false);
    }
    let mut gained = Vec::with_capacity(need);
    let kinds = [CardKind::Sha, CardKind::Shan, CardKind::Tao, CardKind::Jiu];
    for _ in 0..need {
        let pick_kind = rng_pick(ctx, &kinds).unwrap_or(CardKind::Sha);
        let card = spawn_card_by_kind(ctx, pick_kind).or_else(|| {
            ctx.core
                .deck
                .iter()
                .position(|&c| is_basic_non_du(ctx.catalog, c))
                .map(|idx| ctx.core.deck.remove(idx))
        });
        if let Some(c) = card {
            ctx.core.player_mut(t).hand.push(c);
            gained.push(c);
        }
    }
    if !gained.is_empty() {
        ctx.log(LogEvent::CardsDrawn {
            player: t,
            cards: gained,
        });
    }
    if let Some(c) = m.card {
        finish_table_card_hearth(ctx, c, false);
    }
    Flow::Done
}
