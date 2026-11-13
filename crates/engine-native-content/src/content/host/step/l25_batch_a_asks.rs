//! L25 祖安武将 Batch A 询问构造与前半技能步进（4 张正式卡牌 + 前 13 个全局/模式/神将技能）。

use super::l25_batch_a_ext::handle_l25_batch_a_ext;
use super::*;
use crate::catalog::{CardKind, Catalog, Suit};
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::event::{DamageNature, EventData};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow, LogEvent};
use crate::snap::ExpectedAsk;
use crate::state::GameStateCore;

fn spec_ask(
    chooser: PlayerId,
    kind: ChoiceKind,
    prompt: &'static str,
    options: Vec<ChoiceOption>,
    min: usize,
    max: usize,
    allow_cancel: bool,
) -> Option<(PlayerId, ChoiceSpec)> {
    (!options.is_empty() && options.len() >= min).then_some((
        chooser,
        ChoiceSpec {
            kind,
            prompt,
            options,
            min,
            max,
            allow_cancel,
        },
    ))
}

fn tag_ask(
    chooser: PlayerId,
    prompt: &'static str,
    tags: impl IntoIterator<Item = u8>,
    allow_cancel: bool,
) -> Option<(PlayerId, ChoiceSpec)> {
    spec_ask(
        chooser,
        ChoiceKind::PickTag,
        prompt,
        tags.into_iter().map(ChoiceOption::Tag).collect(),
        1,
        1,
        allow_cancel,
    )
}

fn player_ask(
    chooser: PlayerId,
    prompt: &'static str,
    players: Vec<PlayerId>,
    min: usize,
    max: usize,
    allow_cancel: bool,
) -> Option<(PlayerId, ChoiceSpec)> {
    spec_ask(
        chooser,
        ChoiceKind::SelectPlayers,
        prompt,
        players.into_iter().map(ChoiceOption::Player).collect(),
        min,
        max,
        allow_cancel,
    )
}

fn card_ask(
    chooser: PlayerId,
    prompt: &'static str,
    cards: Vec<CardId>,
    min: usize,
    max: usize,
    allow_cancel: bool,
) -> Option<(PlayerId, ChoiceSpec)> {
    spec_ask(
        chooser,
        ChoiceKind::SelectCards,
        prompt,
        cards.into_iter().map(ChoiceOption::Card).collect(),
        min,
        max,
        allow_cancel,
    )
}

pub(super) fn build_ask(
    ask_id: u32,
    owner: PlayerId,
    target: Option<PlayerId>,
    card: Option<CardId>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<(PlayerId, ChoiceSpec)> {
    match ask_id {
        ASK_XICHENG_TARGET => player_ask(
            owner,
            "ssr_newsance_xicheng:choose_target",
            alive_others(core, owner),
            1,
            1,
            true,
        ),
        ASK_XICHENG_SHOW_CARD => {
            let tgt = target.unwrap_or(owner);
            card_ask(
                tgt,
                "ssr_newsance_xicheng:show_card",
                core.player(tgt).hand.clone(),
                1,
                1,
                false,
            )
        }
        ASK_WEICHENG_TARGET => player_ask(
            owner,
            "ssr_newsance_weicheng:choose_target",
            alive_others(core, owner),
            1,
            1,
            true,
        ),
        ASK_WEICHENG_GIVE_CARD => {
            let tgt = target.unwrap_or(owner);
            card_ask(
                tgt,
                "ssr_newsance_weicheng:give_card",
                core.player(tgt).hand.clone(),
                1,
                1,
                false,
            )
        }
        ASK_WEICHENG_CONFIRM_SHA => {
            tag_ask(owner, "ssr_newsance_weicheng:confirm_sha", 0..=1, true)
        }
        ASK_SHOUCHENG_CHOOSE_CARD => card_ask(
            owner,
            "ssr_newsance_shoucheng:choose_card",
            cards.to_vec(),
            1,
            1,
            false,
        ),
        ASK_YUXI_TARGET => {
            let cands: Vec<PlayerId> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| !core.player(p).hand.is_empty() || !core.player(owner).hand.is_empty())
                .collect();
            player_ask(owner, "ssr_yongsi_yuxi:choose_target", cands, 1, 1, true)
        }
        ASK_YUXI_ACTION => {
            let tgt = target.unwrap_or(owner);
            let mut tags = Vec::new();
            if !core.player(tgt).hand.is_empty() {
                tags.push(0);
            }
            if !core.player(owner).hand.is_empty() {
                tags.push(1);
            }
            tag_ask(owner, "ssr_yongsi_yuxi:choose_action", tags, true)
        }
        ASK_YUXI_DISCARD_TARGET => {
            let tgt = target.unwrap_or(owner);
            card_ask(
                owner,
                "ssr_yongsi_yuxi:discard_target_card",
                core.player(tgt).hand.clone(),
                1,
                1,
                false,
            )
        }
        ASK_YUXI_GIVE_CARD => card_ask(
            owner,
            "ssr_yongsi_yuxi:give_card",
            core.player(owner).hand.clone(),
            1,
            1,
            false,
        ),
        ASK_G_DU_TARGET => {
            player_ask(owner, "g_du:choose_target", alive_others(core, owner), 1, 1, true)
        }
        ASK_YONGJIN_SRC => {
            let srcs: Vec<PlayerId> = alive_all(core, owner)
                .into_iter()
                .filter(|&p| !yongjin_movable_equips(core, catalog, p).is_empty())
                .collect();
            player_ask(owner, "_yongjinskill:choose_target", srcs, 1, 1, true)
        }
        ASK_YONGJIN_EQUIP => {
            let src = target.unwrap_or(owner);
            card_ask(
                owner,
                "_yongjinskill:choose_equip",
                yongjin_movable_equips(core, catalog, src),
                1,
                1,
                false,
            )
        }
        ASK_YONGJIN_DST => {
            let src = target.unwrap_or(owner);
            let eq = card.unwrap_or(0);
            let dsts = yongjin_destinations(core, catalog, src, eq);
            player_ask(owner, "_yongjinskill:choose_dest", dsts, 1, 1, false)
        }
        ASK_KONGCHENG_CONFIRM => tag_ask(owner, "_kongchengskill:confirm", 0..=1, true),
        ASK_LUNJIAN_TARGET => player_ask(
            owner,
            "_lunjianskill:choose_target",
            alive_others(core, owner),
            1,
            1,
            true,
        ),
        ASK_WUMOU_MODE => tag_ask(owner, "wumou:choose_mode", 0..=1, false),
        ASK_NEW_GUIXIN_CONFIRM => tag_ask(owner, "new_guixin:confirm", 0..=1, true),
        ASK_KUANGFENG_TARGET => player_ask(
            owner,
            "kuangfeng:choose_target",
            alive_all(core, owner),
            1,
            1,
            true,
        ),
        ASK_DAWU_TARGETS => {
            let cands = alive_all(core, owner);
            let max_n = core
                .player(owner)
                .off_court
                .len()
                .max(flag(core, owner, "mark:qixing") as usize)
                .max(1)
                .min(cands.len());
            player_ask(owner, "dawu:choose_targets", cands, 1, max_n, true)
        }
        ASK_QINYIN_MODE => tag_ask(owner, "qinyin:choose_mode", 0..=1, false),
        ASK_NEW_WUHUN_TARGET => {
            let others = alive_others(core, owner);
            let max_mark = others
                .iter()
                .map(|&p| flag(core, p, "mark:mengyan"))
                .max()
                .unwrap_or(0);
            let cands: Vec<PlayerId> = others
                .into_iter()
                .filter(|&p| flag(core, p, "mark:mengyan") == max_mark)
                .collect();
            player_ask(owner, "new_wuhun:choose_target", cands, 1, 1, false)
        }
        ASK_JIEYING_TARGET => player_ask(
            owner,
            "g_nzry_jieying:choose_target",
            alive_others(core, owner),
            1,
            1,
            true,
        ),
        _ => None,
    }
}

pub(super) fn wait_ask(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    stage: HostStage,
    ask_id: u32,
) -> Flow {
    set_flag(ctx.core, m.owner, "l25_ask_id", ask_id);
    if let Some((observer, spec)) =
        build_ask(ask_id, m.owner, m.target, m.card, &m.cards, ctx.core, ctx.catalog)
    {
        m.stage = stage;
        Flow::WaitChoice { observer, spec }
    } else {
        set_flag(ctx.core, m.owner, "l25_ask_id", 0);
        Flow::Done
    }
}

pub fn ask_l25_batch_a(
    stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    _targets: &[PlayerId],
    card: Option<CardId>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    if stage == HostStage::Act || stage == HostStage::DrawSettle || stage == HostStage::CardsAsk {
        return None;
    }
    let ask_id = flag(core, owner, "l25_ask_id");
    let (observer, spec) = build_ask(ask_id, owner, target, card, cards, core, catalog)?;
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

fn refund_once_used(ctx: &mut Ctx<'_>, owner: PlayerId, name: &str) {
    if let Some(def) = ctx.catalog.skill_by_name(name) {
        if let Some(inst) = ctx.core.skills.get_mut(&(owner, def)) {
            inst.once_used = false;
        }
    }
}

pub(super) fn handle_l25_batch_a_impl(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage != HostStage::Act
        && m.stage != HostStage::DrawSettle
        && m.stage != HostStage::CardsAsk
        && ctx.res.answer.as_ref().is_some_and(|a| a.is_empty())
    {
        refund_once_used(ctx, owner, &m.def_key);
        set_flag(ctx.core, owner, "l25_ask_id", 0);
        return Flow::Done;
    }
    match m.def_key.as_str() {
        "ssr_newsance_xicheng" => step_xicheng(m, ctx),
        "ssr_newsance_weicheng" => step_weicheng(m, ctx),
        "ssr_newsance_shoucheng" => step_shoucheng(m, ctx),
        "ssr_yongsi_yuxi_card" | "ssr_yongsi_yuxi" => step_yongsi_yuxi(m, ctx),
        "g_du" => step_g_du(m, ctx),
        "_yongjinskill" => step_yongjin(m, ctx),
        "_kongchengskill" => step_kongcheng(m, ctx),
        "_anyeskill" => {
            if let Ok(d) = ctx.current_damage() {
                if d.target == owner
                    && d.nature == DamageNature::Fire
                    && !d.prevented
                    && d.amount > 0
                {
                    ctx.modify_damage_amount(1);
                    let cnt = flag(ctx.core, owner, "l25_anye_triggered") + 1;
                    set_flag(ctx.core, owner, "l25_anye_triggered", cnt);
                }
            }
            Flow::Done
        }
        "_taohuaskill" => {
            set_flag(ctx.core, owner, "l25_taohua_active", 1);
            Flow::Done
        }
        "_lunjianskill" => step_lunjian(m, ctx),
        "_jianghunskill" => {
            set_flag(ctx.core, owner, "l25_jianghun_granted", 1);
            if let Some(def) = ctx
                .catalog
                .skill_by_name("new_wuhun")
                .or_else(|| ctx.catalog.skill_by_name("wuhun2"))
            {
                ctx.ensure_skill(
                    (owner, def),
                    crate::state::SkillInstance {
                        owner,
                        enabled: true,
                        once_used: false,
                        expires_end_of_turn: false,
                        rounds: 0,
                    },
                );
            }
            Flow::Done
        }
        "_thunderjudgemodeskill" => step_thunder_judge_mode(m, ctx),
        "_thundermodeskill" => step_thunder_mode(m, ctx),
        "_goodluckjuedgeskill" => {
            let cnt = flag(ctx.core, owner, "l25_goodluck_judge") + 1;
            set_flag(ctx.core, owner, "l25_goodluck_judge", cnt);
            Flow::Done
        }
        _ => handle_l25_batch_a_ext(m, ctx),
    }
}

fn resolve_xicheng_shown(m: &mut HostMachine, ctx: &mut Ctx<'_>, tgt: PlayerId, shown: CardId) -> Flow {
    let owner = m.owner;
    set_flag(ctx.core, tgt, "l25_xicheng_shown_card", u32::from(shown) + 1);
    ctx.out.push(LogEvent::CardsRevealed {
        cards: vec![shown],
        to: Vec::new(),
    });
    if is_sha_card(ctx.core, ctx.catalog, tgt, shown) && ctx.core.player(owner).alive {
        let as_kind = match effective_sha_kind(ctx.core, ctx.catalog, tgt, shown) {
            CardKind::FireSha => CardKind::FireSha,
            CardKind::LeiSha => CardKind::LeiSha,
            _ => CardKind::Sha,
        };
        let nature = match as_kind {
            CardKind::FireSha => DamageNature::Fire,
            CardKind::LeiSha => DamageNature::Thunder,
            _ => DamageNature::Normal,
        };
        ctx.emit(EventData::CardUsed(crate::event::CardUseData {
            card: Some(shown),
            player: tgt,
            targets: vec![owner],
            nature,
            use_as: Some(as_kind),
            cancelled: false,
        }));
        m.stage = HostStage::CardsAsk;
        Flow::Yield
    } else if ctx.core.player(tgt).alive {
        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
            Some(owner),
            tgt,
            1,
            DamageNature::Normal,
            m.card,
        )));
        m.stage = HostStage::CardsAsk;
        Flow::Yield
    } else {
        Flow::Done
    }
}

fn step_xicheng(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            set_flag(ctx.core, owner, "l25_sance_xicheng", 1);
            let tgt = m.target.or_else(|| m.targets.first().copied());
            let Some(tgt) = tgt else {
                let others = alive_others(ctx.core, owner);
                if others.is_empty() {
                    return Flow::Done;
                }
                if others.len() == 1 {
                    m.target = Some(others[0]);
                    return step_xicheng(m, ctx);
                }
                return wait_ask(m, ctx, HostStage::TargetChoice, ASK_XICHENG_TARGET);
            };
            m.target = Some(tgt);
            if !ctx.core.player(tgt).alive {
                return Flow::Done;
            }
            if ctx.core.player(tgt).hand.is_empty() {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    tgt,
                    1,
                    DamageNature::Normal,
                    m.card,
                )));
                m.stage = HostStage::CardsAsk;
                return Flow::Yield;
            }
            if ctx.core.player(tgt).hand.len() == 1 {
                let shown = ctx.core.player(tgt).hand[0];
                return resolve_xicheng_shown(m, ctx, tgt, shown);
            }
            wait_ask(m, ctx, HostStage::SelectCard, ASK_XICHENG_SHOW_CARD)
        }
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            set_flag(ctx.core, owner, "l25_ask_id", 0);
            m.target = Some(tgt);
            m.stage = HostStage::Act;
            step_xicheng(m, ctx)
        }
        HostStage::SelectCard => {
            let tgt = m.target.unwrap_or(owner);
            let shown = match ctx.answer().first().copied() {
                Some(ChoiceOption::Card(c)) => c,
                _ => return Flow::Done,
            };
            set_flag(ctx.core, owner, "l25_ask_id", 0);
            resolve_xicheng_shown(m, ctx, tgt, shown)
        }
        _ => Flow::Done,
    }
}

fn step_weicheng(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            set_flag(ctx.core, owner, "l25_sance_weicheng", 1);
            let tgt = m.target.or_else(|| m.targets.first().copied());
            let Some(tgt) = tgt else {
                let others = alive_others(ctx.core, owner);
                if others.is_empty() {
                    return Flow::Done;
                }
                if others.len() == 1 {
                    m.target = Some(others[0]);
                    return step_weicheng(m, ctx);
                }
                return wait_ask(m, ctx, HostStage::TargetChoice, ASK_WEICHENG_TARGET);
            };
            m.target = Some(tgt);
            if !ctx.core.player(tgt).alive {
                return Flow::Done;
            }
            ctx.start_draw(tgt, 1);
            m.stage = HostStage::DrawSettle;
            Flow::Yield
        }
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            set_flag(ctx.core, owner, "l25_ask_id", 0);
            m.target = Some(tgt);
            ctx.start_draw(tgt, 1);
            m.stage = HostStage::DrawSettle;
            Flow::Yield
        }
        HostStage::DrawSettle => {
            let Some(tgt) = m.target.or_else(|| m.targets.first().copied()) else {
                return Flow::Done;
            };
            if !ctx.core.player(tgt).alive
                || !ctx.core.player(owner).alive
                || ctx.core.player(tgt).hand.is_empty()
            {
                return Flow::Done;
            }
            if ctx.core.player(tgt).hand.len() == 1 {
                let c = ctx.core.player(tgt).hand[0];
                ctx.give_hand_card(tgt, owner, c);
                m.cards = vec![c];
                set_flag(ctx.core, owner, "l25_weicheng_given_card", u32::from(c) + 1);
                if ctx.core.player(owner).hand.contains(&c) && ctx.core.player(tgt).alive {
                    return wait_ask(m, ctx, HostStage::SelectNature, ASK_WEICHENG_CONFIRM_SHA);
                }
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::SelectCard, ASK_WEICHENG_GIVE_CARD)
        }
        HostStage::SelectCard => {
            let tgt = m.target.unwrap_or(owner);
            let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            set_flag(ctx.core, owner, "l25_ask_id", 0);
            if ctx.core.player(tgt).hand.contains(&c) && ctx.core.player(owner).alive {
                ctx.give_hand_card(tgt, owner, c);
                m.cards = vec![c];
                set_flag(ctx.core, owner, "l25_weicheng_given_card", u32::from(c) + 1);
                if ctx.core.player(owner).hand.contains(&c) && ctx.core.player(tgt).alive {
                    return wait_ask(m, ctx, HostStage::SelectNature, ASK_WEICHENG_CONFIRM_SHA);
                }
            }
            Flow::Done
        }
        HostStage::SelectNature => {
            let tgt = m.target.unwrap_or(owner);
            let ok = matches!(
                ctx.answer().first().copied(),
                Some(ChoiceOption::Tag(1)) | Some(ChoiceOption::Confirm)
            );
            set_flag(ctx.core, owner, "l25_ask_id", 0);
            if ok {
                if let Some(&c) = m.cards.first() {
                    if ctx.core.player(owner).hand.contains(&c) {
                        ctx.discard_from_hand(owner, c);
                    }
                }
                if ctx.core.player(owner).alive
                    && ctx.core.player(tgt).alive
                    && !ctx.core.deck.is_empty()
                {
                    ctx.use_virtual_card(owner, CardKind::Sha, vec![tgt]);
                    m.stage = HostStage::CardsAsk;
                    return Flow::Yield;
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_shoucheng(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            set_flag(ctx.core, owner, "l25_sance_shoucheng", 1);
            if ctx.core.deck.len() < 3 && !ctx.core.discard.is_empty() {
                let mut extra = std::mem::take(&mut ctx.core.discard);
                ctx.core.rng.shuffle(&mut extra);
                extra.append(&mut ctx.core.deck);
                ctx.core.deck = extra;
                ctx.out.push(LogEvent::DeckReshuffled);
            }
            let top3: Vec<CardId> = ctx.core.deck.iter().rev().take(3).copied().collect();
            if top3.is_empty() {
                return Flow::Done;
            }
            m.cards = top3;
            wait_ask(m, ctx, HostStage::SelectCard, ASK_SHOUCHENG_CHOOSE_CARD)
        }
        HostStage::SelectCard => {
            let chosen = match ctx.answer().first().copied() {
                Some(ChoiceOption::Card(c)) if m.cards.contains(&c) => c,
                _ => m.cards.first().copied().unwrap_or(0),
            };
            set_flag(ctx.core, owner, "l25_ask_id", 0);
            set_flag(ctx.core, owner, "l25_shoucheng_chosen", u32::from(chosen) + 1);
            for &c in &m.cards {
                ctx.core.deck.retain(|&x| x != c);
            }
            let is_heart = ctx
                .catalog
                .cards
                .get(chosen as usize)
                .is_some_and(|d| d.suit == Suit::Heart);
            if is_heart {
                for &c in &m.cards {
                    ctx.to_discard(c);
                }
                ctx.recover(owner, 1, m.card);
            } else {
                for &c in &m.cards {
                    if c == chosen && ctx.core.player(owner).alive {
                        ctx.core.player_mut(owner).hand.push(c);
                        ctx.out.push(LogEvent::CardsDrawn {
                            player: owner,
                            cards: vec![c],
                        });
                    } else {
                        ctx.to_discard(c);
                    }
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_yongsi_yuxi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(100) {
        let draw_n = (ctx.core.player(owner).max_hp as i32 - ctx.core.player(owner).hp).max(1) as u32;
        set_flag(ctx.core, owner, "l25_yongsi_yuxi_prepare", draw_n);
        ctx.start_draw(owner, draw_n);
        m.card = Some(101);
        m.stage = HostStage::DrawSettle;
        return Flow::Yield;
    }
    if m.card == Some(101) {
        if ctx.core.player(owner).alive {
            ctx.emit(EventData::LoseHp {
                player: owner,
                amount: 1,
            });
            m.card = Some(102);
            m.stage = HostStage::CardsAsk;
            return Flow::Yield;
        }
        return Flow::Done;
    }
    if m.card == Some(102) {
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_YUXI_TARGET),
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            m.target = Some(tgt);
            set_flag(ctx.core, owner, "l25_yongsi_yuxi_viewed", u32::from(tgt) + 1);
            wait_ask(m, ctx, HostStage::SelectNature, ASK_YUXI_ACTION)
        }
        HostStage::SelectNature => {
            let Some(ChoiceOption::Tag(act)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            let tgt = m.target.unwrap_or(owner);
            if act == 0 {
                if ctx.core.player(tgt).hand.len() <= 1 {
                    if let Some(&c) = ctx.core.player(tgt).hand.first() {
                        ctx.discard_from_hand(tgt, c);
                    }
                    set_flag(ctx.core, owner, "l25_ask_id", 0);
                    return Flow::Done;
                }
                wait_ask(m, ctx, HostStage::SelectCard, ASK_YUXI_DISCARD_TARGET)
            } else {
                if ctx.core.player(owner).hand.len() <= 1 {
                    if let Some(&c) = ctx.core.player(owner).hand.first() {
                        ctx.give_hand_card(owner, tgt, c);
                        ctx.start_draw(owner, 2);
                        set_flag(ctx.core, owner, "l25_ask_id", 0);
                        m.stage = HostStage::CardsAsk;
                        return Flow::Yield;
                    }
                    set_flag(ctx.core, owner, "l25_ask_id", 0);
                    return Flow::Done;
                }
                wait_ask(m, ctx, HostStage::JiefenOwnerDiscard, ASK_YUXI_GIVE_CARD)
            }
        }
        HostStage::SelectCard => {
            let tgt = m.target.unwrap_or(owner);
            if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                if ctx.core.player(tgt).hand.contains(&c) {
                    ctx.discard_from_hand(tgt, c);
                }
            }
            set_flag(ctx.core, owner, "l25_ask_id", 0);
            Flow::Done
        }
        HostStage::JiefenOwnerDiscard => {
            let tgt = m.target.unwrap_or(owner);
            if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                if ctx.core.player(owner).hand.contains(&c) && ctx.core.player(tgt).alive {
                    ctx.give_hand_card(owner, tgt, c);
                    ctx.start_draw(owner, 2);
                    set_flag(ctx.core, owner, "l25_ask_id", 0);
                    m.stage = HostStage::CardsAsk;
                    return Flow::Yield;
                }
            }
            set_flag(ctx.core, owner, "l25_ask_id", 0);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_g_du(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(1) {
        let cnt = m
            .cards
            .iter()
            .filter(|&&c| is_du_card(ctx.core, ctx.catalog, owner, c))
            .count()
            .max(1) as u32;
        set_flag(ctx.core, owner, "l25_g_du_triggered", cnt);
        ctx.emit(EventData::LoseHp {
            player: owner,
            amount: cnt,
        });
        m.card = Some(10);
        m.stage = HostStage::CardsAsk;
        return Flow::Yield;
    }
    if m.card == Some(10) {
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_G_DU_TARGET),
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            set_flag(ctx.core, owner, "l25_ask_id", 0);
            if let Some(c) = ctx
                .core
                .player(owner)
                .hand
                .iter()
                .copied()
                .find(|&c| is_du_card(ctx.core, ctx.catalog, owner, c))
            {
                set_flag(ctx.core, tgt, &format!("is_du_card:{c}"), 1);
                ctx.give_hand_card(owner, tgt, c);
                set_flag(ctx.core, owner, "l25_g_du_given", u32::from(tgt) + 1);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn move_equip_between_players(ctx: &mut Ctx<'_>, src: PlayerId, dst: PlayerId, eq: CardId) {
    if let Some(pos) = ctx.core.player(src).equips.iter().position(|&x| x == eq) {
        ctx.core.player_mut(src).equips.remove(pos);
        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, src, eq);
        ctx.core.player_mut(dst).equips.push(eq);
        crate::kernel::grant_equip_skills(ctx, dst, eq);
    }
}

fn step_yongjin(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_YONGJIN_SRC),
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(src)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            m.target = Some(src);
            let eqs = yongjin_movable_equips(ctx.core, ctx.catalog, src);
            if eqs.is_empty() {
                set_flag(ctx.core, owner, "l25_ask_id", 0);
                return Flow::Done;
            }
            if eqs.len() == 1 {
                m.card = Some(eqs[0]);
                return wait_ask(m, ctx, HostStage::JiefenTarget, ASK_YONGJIN_DST);
            }
            wait_ask(m, ctx, HostStage::SelectCard, ASK_YONGJIN_EQUIP)
        }
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(eq)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            m.card = Some(eq);
            wait_ask(m, ctx, HostStage::JiefenTarget, ASK_YONGJIN_DST)
        }
        HostStage::JiefenTarget => {
            let Some(ChoiceOption::Player(dst)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            set_flag(ctx.core, owner, "l25_ask_id", 0);
            let src = m.target.unwrap_or(owner);
            if let Some(eq) = m.card {
                move_equip_between_players(ctx, src, dst, eq);
                set_flag(ctx.core, owner, "l25_yongjin_moved", 1);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_kongcheng(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectNature, ASK_KONGCHENG_CONFIRM),
        HostStage::SelectNature => {
            let ok = matches!(
                ctx.answer().first().copied(),
                Some(ChoiceOption::Tag(1)) | Some(ChoiceOption::Confirm)
            );
            set_flag(ctx.core, owner, "l25_ask_id", 0);
            if ok {
                set_flag(ctx.core, owner, "l25_kongcheng_used", turn_stamp(ctx.core));
                ctx.start_draw(owner, 1);
                m.stage = HostStage::CardsAsk;
                return Flow::Yield;
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_lunjian(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_LUNJIAN_TARGET),
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            set_flag(ctx.core, owner, "l25_ask_id", 0);
            set_flag(ctx.core, owner, "l25_lunjian_target", u32::from(tgt) + 1);
            let target_skills: Vec<_> = ctx
                .core
                .skills
                .iter()
                .filter(|&(&(p, def), inst)| {
                    p == tgt
                        && inst.enabled
                        && !ctx.catalog.skill_name(def).starts_with('_')
                        && !ctx.core.has_skill(owner, def)
                })
                .map(|(&(_, def), _)| def)
                .collect();
            if let Some(&def) = target_skills.first() {
                ctx.add_skill_until_turn_end(owner, def);
                set_flag(ctx.core, owner, "l25_lunjian_copied", u32::from(def) + 1);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_thunder_judge_mode(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage != HostStage::Act {
        return Flow::Done;
    }
    let jc = ctx.draw_judge_card();
    let hit = ctx
        .catalog
        .cards
        .get(jc as usize)
        .is_some_and(|d| d.suit == Suit::Spade && (2..=9).contains(&d.rank));
    ctx.to_discard(jc);
    set_flag(ctx.core, owner, "l25_thunder_judge_card", u32::from(jc) + 1);
    if hit {
        set_flag(ctx.core, owner, "l25_thunder_judge_hit", 1);
        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
            None,
            owner,
            3,
            DamageNature::Thunder,
            Some(jc),
        )));
        m.stage = HostStage::CardsAsk;
        return Flow::Yield;
    }
    Flow::Done
}

fn step_thunder_mode(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage != HostStage::Act {
        return Flow::Done;
    }
    let preset = flag(ctx.core, owner, "l25_thundermode_target");
    let tgt = if preset > 0 && ((preset - 1) as usize) < ctx.core.players.len() {
        (preset - 1) as PlayerId
    } else {
        owner
    };
    if ctx.core.player(tgt).alive {
        set_flag(ctx.core, owner, "l25_thundermode_triggered", u32::from(tgt) + 1);
        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
            None,
            tgt,
            1,
            DamageNature::Thunder,
            None,
        )));
        m.stage = HostStage::CardsAsk;
        return Flow::Yield;
    }
    Flow::Done
}

