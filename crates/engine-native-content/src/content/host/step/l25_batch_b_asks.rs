//! L25 祖安武将 Batch B 询问构造与前半技能步进（技能 1–11：`ssr_qianxun` 至 `ssr_newmubing`）。

use super::l25_batch_b_ext::handle_l25_batch_b_ext;
use super::*;
use crate::catalog::{CardKind, Catalog, Faction};
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::event::{CardUseData, DamageNature, EventData};
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

pub(super) fn tag_ask(
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

pub(super) fn player_ask(
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

pub(super) fn card_ask(
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
        ASK_QIANXUN_CONFIRM => tag_ask(owner, "ssr_qianxun:confirm", 0..=1, true),
        ASK_QIANXUN_DISCARD => {
            let hand = core.player(owner).hand.clone();
            let n = hand.len().saturating_sub(1);
            if n == 0 {
                return None;
            }
            card_ask(owner, "ssr_qianxun:discard_cards", hand, n, n, true)
        }
        ASK_JUEJI_MODE => tag_ask(owner, "ssr_jueji:choose_mode", 0..=10, true),
        ASK_JUEJI_TARGET => {
            player_ask(owner, "ssr_jueji:choose_target", alive_others(core, owner), 1, 1, true)
        }
        ASK_ZHIHENG_DISCARD => {
            let he = he_cards(core, owner);
            let max = he.len();
            card_ask(owner, "ssr_zhiheng:discard_cards", he, 1, max, true)
        }
        ASK_ZHIHENG_TARGET => player_ask(
            owner,
            "ssr_zhiheng:choose_target",
            zhiheng_targets(core, owner),
            1,
            1,
            false,
        ),
        ASK_ZHIHENG_MODE => tag_ask(owner, "ssr_zhiheng:choose_mode", 0..=1, false),
        ASK_ZHIHENG_TARGET_CHOICE => {
            let tgt = target.unwrap_or(owner);
            tag_ask(tgt, "ssr_zhiheng:target_choice", 0..=1, false)
        }
        ASK_ZHIHENG_TARGET_DISCARD => {
            let tgt = target.unwrap_or(owner);
            let x = card.unwrap_or(1) as usize;
            let he = he_cards(core, tgt);
            card_ask(tgt, "ssr_zhiheng:target_discard", he, x, x, true)
        }
        ASK_JIUYUAN_CONFIRM => {
            let resp = target.unwrap_or(owner);
            tag_ask(resp, "ssr_jiuyuan:confirm", 0..=1, true)
        }
        ASK_LONGDAN_CARD => {
            let cands: Vec<CardId> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| {
                    matches!(
                        catalog.card_kind(c),
                        CardKind::Sha
                            | CardKind::FireSha
                            | CardKind::LeiSha
                            | CardKind::Shan
                            | CardKind::Tao
                            | CardKind::Jiu
                    )
                })
                .collect();
            card_ask(owner, "ssr_newlongdan:choose_card", cands, 1, 1, true)
        }
        ASK_LONGDAN_TARGET => player_ask(
            owner,
            "ssr_newlongdan:choose_target",
            alive_others(core, owner),
            1,
            1,
            false,
        ),
        ASK_POZHEN_TARGET_DISCARD => {
            let opp = target.unwrap_or(owner);
            let basics: Vec<CardId> = core
                .player(opp)
                .hand
                .iter()
                .copied()
                .filter(|&c| catalog.card_kind(c).is_basic_kind())
                .collect();
            card_ask(opp, "ssr_newpozhen:target_discard", basics, 1, 1, true)
        }
        ASK_POZHEN_GAIN_CARD => {
            let opp = target.unwrap_or(owner);
            card_ask(
                owner,
                "ssr_newpozhen:choose_card",
                core.player(opp).hand.clone(),
                1,
                1,
                true,
            )
        }
        ASK_JINNANG_CARD => {
            let tricks: Vec<CardId> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| {
                    let k = catalog.card_kind(c);
                    is_normal_trick_kind(k) || is_delayed_trick_kind(k)
                })
                .collect();
            card_ask(owner, "ssr_jinnang:choose_card", tricks, 1, 1, true)
        }
        ASK_JINNANG_TARGET => player_ask(
            owner,
            "ssr_jinnang:choose_target",
            alive_all(core, owner),
            1,
            1,
            false,
        ),
        ASK_JINNANG_MODE => tag_ask(owner, "ssr_jinnang:choose_mode", 0..=7, false),
        ASK_BAZHEN_CARD => card_ask(
            owner,
            "ssr_newbazhen:choose_card",
            bazhen_pile(core, owner),
            1,
            1,
            true,
        ),
        ASK_BAZHEN_TARGET => player_ask(
            owner,
            "ssr_newbazhen:choose_target",
            alive_others(core, owner),
            1,
            1,
            false,
        ),
        ASK_TUXI_TARGET => {
            let cands: Vec<PlayerId> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| !he_cards(core, p).is_empty())
                .collect();
            player_ask(owner, "ssr_tuxi:choose_target", cands, 1, 1, true)
        }
        ASK_MUBING_CARDS => {
            let max_pick = ((card.unwrap_or(2) as usize).saturating_sub(1)).clamp(1, cards.len());
            card_ask(
                owner,
                "ssr_newmubing:choose_cards",
                cards.to_vec(),
                1,
                max_pick,
                true,
            )
        }
        ASK_JINFAN_DISCARD => card_ask(
            owner,
            "ssr_jinfan:discard_card",
            jinfan_eligible_cards(core, catalog, owner),
            1,
            1,
            true,
        ),
        ASK_QIXI_TARGET => player_ask(
            owner,
            "ssr_qixi:choose_target",
            alive_others(core, owner),
            1,
            1,
            false,
        ),
        ASK_ZHUIJI_CARD => {
            let tgt = target.unwrap_or(owner);
            card_ask(
                owner,
                "ssr_zhuiji:choose_card",
                he_cards(core, tgt),
                1,
                1,
                false,
            )
        }
        ASK_TIEJI_CONFIRM => tag_ask(owner, "ssr_tieji:confirm", 0..=1, true),
        ASK_FANJIAN_CARD => card_ask(
            owner,
            "ssr_newfanjian:choose_card",
            core.player(owner).hand.clone(),
            1,
            1,
            true,
        ),
        ASK_FANJIAN_TARGET => {
            let cands: Vec<PlayerId> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| !core.player(p).hand.is_empty())
                .collect();
            player_ask(owner, "ssr_newfanjian:choose_target", cands, 1, 1, false)
        }
        ASK_FANJIAN_SHOW => {
            let tgt = target.unwrap_or(owner);
            card_ask(
                tgt,
                "ssr_newfanjian:show_card",
                core.player(tgt).hand.clone(),
                1,
                1,
                false,
            )
        }
        ASK_SHICHOU_CONFIRM => tag_ask(owner, "ssr_shichou:confirm", 0..=1, true),
        ASK_RENDE_CARD => card_ask(
            owner,
            "ssr_newrende:give_card",
            core.player(owner).hand.clone(),
            1,
            1,
            true,
        ),
        ASK_RENDE_TARGET => player_ask(
            owner,
            "ssr_newrende:choose_target",
            alive_others(core, owner),
            1,
            1,
            false,
        ),
        ASK_RENDE_TARGET_CHOICE => {
            let tgt = target.unwrap_or(owner);
            tag_ask(tgt, "ssr_newrende:target_choice", 0..=1, false)
        }
        ASK_JIJIANG_TARGET => {
            let cands: Vec<PlayerId> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| {
                    core.effective_faction(catalog, p) == Some(Faction::Shu)
                        && flag(core, p, "l25_jijiang_sha_plus1") == 0
                })
                .collect();
            player_ask(owner, "ssr_jijiang:choose_target", cands, 1, 1, true)
        }
        ASK_QIANGXI_TARGET => player_ask(
            owner,
            "ssr_qiangxi:choose_target",
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
    set_flag(ctx.core, m.owner, "l25_batch_b_ask_id", ask_id);
    if let Some((observer, spec)) =
        build_ask(ask_id, m.owner, m.target, m.card, &m.cards, ctx.core, ctx.catalog)
    {
        m.stage = stage;
        Flow::WaitChoice { observer, spec }
    } else {
        set_flag(ctx.core, m.owner, "l25_batch_b_ask_id", 0);
        Flow::Done
    }
}

pub fn ask_l25_batch_b(
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
    let ask_id = flag(core, owner, "l25_batch_b_ask_id");
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

pub(super) fn handle_l25_batch_b_impl(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let cur_ask = flag(ctx.core, owner, "l25_batch_b_ask_id");
    if m.stage != HostStage::Act
        && m.stage != HostStage::DrawSettle
        && m.stage != HostStage::CardsAsk
        && cur_ask != ASK_ZHIHENG_TARGET_DISCARD
        && cur_ask != ASK_POZHEN_TARGET_DISCARD
        && ctx.res.answer.as_ref().is_some_and(|a| a.is_empty())
    {
        refund_once_used(ctx, owner, &m.def_key);
        set_flag(ctx.core, owner, "l25_batch_b_ask_id", 0);
        return Flow::Done;
    }
    match m.def_key.as_str() {
        "ssr_qianxun" => step_qianxun(m, ctx),
        "ssr_jueji" => step_jueji(m, ctx),
        "ssr_zhiheng" => step_zhiheng(m, ctx),
        "ssr_jianye" => step_jianye(m, ctx),
        "ssr_jiuyuan" => step_jiuyuan(m, ctx),
        "ssr_newlongdan" => step_newlongdan(m, ctx),
        "ssr_newpozhen" => step_newpozhen(m, ctx),
        _ => handle_l25_batch_b_ext(m, ctx),
    }
}

fn step_qianxun(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if ctx.core.player(owner).hand.is_empty() {
                wait_ask(m, ctx, HostStage::SelectNature, ASK_QIANXUN_CONFIRM)
            } else if ctx.core.player(owner).hand.len() > 1 {
                wait_ask(m, ctx, HostStage::SelectCard, ASK_QIANXUN_DISCARD)
            } else {
                Flow::Done
            }
        }
        HostStage::SelectNature => {
            let ok = matches!(
                ctx.answer().first().copied(),
                Some(ChoiceOption::Tag(1)) | Some(ChoiceOption::Confirm)
            );
            set_flag(ctx.core, owner, "l25_batch_b_ask_id", 0);
            if ok {
                set_flag(ctx.core, owner, "l25_qianxun_triggered", 1);
                ctx.start_draw(owner, 1);
                m.stage = HostStage::CardsAsk;
                return Flow::Yield;
            }
            Flow::Done
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
            set_flag(ctx.core, owner, "l25_batch_b_ask_id", 0);
            if !picked.is_empty() {
                set_flag(ctx.core, owner, "l25_qianxun_triggered", picked.len() as u32);
                discard_he_cards(ctx, owner, &picked);
                m.stage = HostStage::CardsAsk;
                return Flow::Yield;
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn jueji_kind_from_tag(tag: u8) -> CardKind {
    match tag {
        0 => CardKind::Sha,
        1 => CardKind::FireSha,
        2 => CardKind::LeiSha,
        3 => CardKind::Tao,
        4 => CardKind::Jiu,
        5 => CardKind::Wuzhong,
        6 => CardKind::Duel,
        7 => CardKind::Dismantle,
        8 => CardKind::Snatch,
        9 => CardKind::SavageAttack,
        _ => CardKind::ArcheryAttack,
    }
}

fn execute_jueji_use(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    as_kind: CardKind,
    targets: Vec<PlayerId>,
) -> Flow {
    let owner = m.owner;
    let Some(&c) = ctx.core.player(owner).hand.first() else {
        return Flow::Done;
    };
    let cnt = inc_used_this_turn(ctx.core, owner, "ssr_jueji");
    set_flag(ctx.core, owner, "l25_jueji_used", cnt);
    let nature = match as_kind {
        CardKind::FireSha => DamageNature::Fire,
        CardKind::LeiSha => DamageNature::Thunder,
        _ => DamageNature::Normal,
    };
    ctx.emit(EventData::CardUsed(CardUseData {
        card: Some(c),
        player: owner,
        targets,
        nature,
        use_as: Some(as_kind),
        cancelled: false,
    }));
    m.stage = HostStage::DrawSettle;
    Flow::Yield
}

fn step_jueji(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(100) {
        let cnt = inc_used_this_turn(ctx.core, owner, "ssr_jueji");
        set_flag(ctx.core, owner, "l25_jueji_used", cnt);
        ctx.start_draw(owner, 1);
        m.card = Some(101);
        m.stage = HostStage::CardsAsk;
        return Flow::Yield;
    }
    if m.card == Some(101) {
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectNature, ASK_JUEJI_MODE),
        HostStage::SelectNature => {
            let Some(ChoiceOption::Tag(tag)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            set_flag(ctx.core, owner, "l25_batch_b_ask_id", 0);
            let as_kind = jueji_kind_from_tag(tag);
            m.card = Some(u32::from(tag));
            match as_kind {
                CardKind::Tao | CardKind::Jiu | CardKind::Wuzhong => {
                    execute_jueji_use(m, ctx, as_kind, vec![owner])
                }
                CardKind::SavageAttack | CardKind::ArcheryAttack => {
                    let tgts = alive_others(ctx.core, owner);
                    execute_jueji_use(m, ctx, as_kind, tgts)
                }
                _ => wait_ask(m, ctx, HostStage::TargetChoice, ASK_JUEJI_TARGET),
            }
        }
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            set_flag(ctx.core, owner, "l25_batch_b_ask_id", 0);
            let as_kind = jueji_kind_from_tag(m.card.unwrap_or(0) as u8);
            execute_jueji_use(m, ctx, as_kind, vec![tgt])
        }
        HostStage::DrawSettle => {
            if ctx.core.player(owner).alive && ctx.core.player(owner).hand.is_empty() {
                ctx.start_draw(owner, 1);
                m.stage = HostStage::CardsAsk;
                return Flow::Yield;
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_zhiheng(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_ZHIHENG_DISCARD),
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
                set_flag(ctx.core, owner, "l25_batch_b_ask_id", 0);
                return Flow::Done;
            }
            let x = picked.len() as u32;
            m.card = Some(x);
            discard_he_cards(ctx, owner, &picked);
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_ZHIHENG_TARGET)
        }
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            m.target = Some(tgt);
            set_flag(
                ctx.core,
                owner,
                &format!("l25_zhiheng_forbid:{tgt}"),
                turn_stamp(ctx.core),
            );
            set_flag(ctx.core, owner, "l25_zhiheng_used", m.card.unwrap_or(1));
            wait_ask(m, ctx, HostStage::SelectNature, ASK_ZHIHENG_MODE)
        }
        HostStage::SelectNature => {
            let Some(ChoiceOption::Tag(mode)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            let tgt = m.target.unwrap_or(owner);
            let x = m.card.unwrap_or(1);
            if mode == 0 {
                wait_ask(m, ctx, HostStage::JiefenTarget, ASK_ZHIHENG_TARGET_CHOICE)
            } else if he_cards(ctx.core, tgt).len() < x as usize {
                set_flag(ctx.core, owner, "l25_batch_b_ask_id", 0);
                ctx.emit(EventData::LoseHp {
                    player: tgt,
                    amount: 1,
                });
                m.stage = HostStage::CardsAsk;
                Flow::Yield
            } else {
                wait_ask(
                    m,
                    ctx,
                    HostStage::JiefenOwnerDiscard,
                    ASK_ZHIHENG_TARGET_DISCARD,
                )
            }
        }
        HostStage::JiefenTarget => {
            let tgt = m.target.unwrap_or(owner);
            let x = m.card.unwrap_or(1);
            let choice = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(v)) => v,
                _ => 0,
            };
            set_flag(ctx.core, owner, "l25_batch_b_ask_id", 0);
            if choice == 0 {
                ctx.start_draw(tgt, x);
            } else {
                ctx.emit(EventData::LoseHp {
                    player: tgt,
                    amount: 1,
                });
            }
            m.stage = HostStage::CardsAsk;
            Flow::Yield
        }
        HostStage::JiefenOwnerDiscard => {
            let tgt = m.target.unwrap_or(owner);
            let x = m.card.unwrap_or(1) as usize;
            let picked: Vec<CardId> = ctx
                .res
                .answer
                .as_ref()
                .map(|ans| {
                    ans.iter()
                        .filter_map(|o| match *o {
                            ChoiceOption::Card(c) => Some(c),
                            _ => None,
                        })
                        .collect()
                })
                .unwrap_or_default();
            set_flag(ctx.core, owner, "l25_batch_b_ask_id", 0);
            if picked.len() < x {
                ctx.emit(EventData::LoseHp {
                    player: tgt,
                    amount: 1,
                });
                m.stage = HostStage::CardsAsk;
                Flow::Yield
            } else {
                discard_he_cards(ctx, tgt, &picked);
                m.stage = HostStage::CardsAsk;
                Flow::Yield
            }
        }
        _ => Flow::Done,
    }
}

fn step_jianye(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(1) {
        let tgt = m.target.unwrap_or(owner);
        let n = m.cards.len();
        if n >= 2 || flag(ctx.core, tgt, "l25_jianye_temp") > 0 {
            set_flag(ctx.core, tgt, "l25_jianye_temp", 0);
            set_flag(ctx.core, tgt, "l25_jianye_tongye", 1);
        } else if flag(ctx.core, tgt, "l25_jianye_tongye") == 0 {
            set_flag(ctx.core, tgt, "l25_jianye_temp", 1);
        }
        return Flow::Done;
    }
    if m.card == Some(2) && m.stage == HostStage::Act {
        let mut cnt = 0u32;
        for p in 0..ctx.core.players.len() as PlayerId {
            if ctx.core.player(p).alive && flag(ctx.core, p, "l25_jianye_tongye") > 0 {
                cnt += 1;
            }
            set_flag(ctx.core, p, "l25_jianye_tongye", 0);
            set_flag(ctx.core, p, "l25_jianye_temp", 0);
        }
        if cnt > 0 {
            set_flag(ctx.core, owner, "l25_jianye_drawn", cnt);
            ctx.start_draw(owner, cnt);
            m.stage = HostStage::CardsAsk;
            return Flow::Yield;
        }
    }
    Flow::Done
}

fn step_jiuyuan(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(2) {
        if let Some(p) = m.target {
            set_flag(ctx.core, p, "l25_jiuyuan_jibing", 0);
        }
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => {
            let Some(resp) = alive_others(ctx.core, owner).into_iter().find(|&p| {
                ctx.core.effective_faction(ctx.catalog, p) == Some(Faction::Wu)
                    && flag(ctx.core, p, "l25_jiuyuan_jibing") == 0
            }) else {
                return Flow::Done;
            };
            m.target = Some(resp);
            wait_ask(m, ctx, HostStage::SelectNature, ASK_JIUYUAN_CONFIRM)
        }
        HostStage::SelectNature => {
            let resp = m.target.unwrap_or(owner);
            let ok = matches!(
                ctx.answer().first().copied(),
                Some(ChoiceOption::Tag(1)) | Some(ChoiceOption::Confirm)
            );
            set_flag(ctx.core, owner, "l25_batch_b_ask_id", 0);
            if ok {
                set_flag(ctx.core, resp, "l25_jiuyuan_jibing", 1);
                set_flag(ctx.core, owner, "l25_jiuyuan_triggered", u32::from(resp) + 1);
                ctx.start_draw(owner, 1);
                m.stage = HostStage::CardsAsk;
                return Flow::Yield;
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_newlongdan(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(100) {
        set_flag(ctx.core, owner, "l25_longdan_viewas", 1);
        return Flow::Done;
    }
    if m.card == Some(101) {
        let cnt = inc_used_this_turn(ctx.core, owner, "ssr_newlongdan");
        set_flag(ctx.core, owner, "l25_longdan_used", cnt);
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_LONGDAN_CARD),
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            set_flag(ctx.core, owner, "l25_batch_b_ask_id", 0);
            m.cards = vec![c];
            let orig = ctx.catalog.card_kind(c);
            match orig {
                CardKind::Shan => wait_ask(m, ctx, HostStage::TargetChoice, ASK_LONGDAN_TARGET),
                CardKind::Jiu | CardKind::Tao => {
                    let as_kind = if orig == CardKind::Jiu {
                        CardKind::Tao
                    } else {
                        CardKind::Jiu
                    };
                    let cnt = inc_used_this_turn(ctx.core, owner, "ssr_newlongdan");
                    set_flag(ctx.core, owner, "l25_longdan_used", cnt);
                    set_flag(ctx.core, owner, "l25_pozhen_chongfeng", 1);
                    ctx.emit(EventData::CardUsed(CardUseData {
                        card: Some(c),
                        player: owner,
                        targets: vec![owner],
                        nature: DamageNature::Normal,
                        use_as: Some(as_kind),
                        cancelled: false,
                    }));
                    m.stage = HostStage::CardsAsk;
                    Flow::Yield
                }
                _ => Flow::Done,
            }
        }
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            set_flag(ctx.core, owner, "l25_batch_b_ask_id", 0);
            let Some(&c) = m.cards.first() else {
                return Flow::Done;
            };
            let cnt = inc_used_this_turn(ctx.core, owner, "ssr_newlongdan");
            set_flag(ctx.core, owner, "l25_longdan_used", cnt);
            set_flag(ctx.core, owner, "l25_pozhen_chongfeng", 1);
            ctx.emit(EventData::CardUsed(CardUseData {
                card: Some(c),
                player: owner,
                targets: vec![tgt],
                nature: DamageNature::Normal,
                use_as: Some(CardKind::Sha),
                cancelled: false,
            }));
            m.stage = HostStage::CardsAsk;
            Flow::Yield
        }
        _ => Flow::Done,
    }
}

fn step_newpozhen(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(100) {
        set_flag(ctx.core, owner, "l25_pozhen_zhanyi", 0);
        set_flag(ctx.core, owner, "l25_pozhen_chongfeng", 0);
        set_flag(ctx.core, owner, "l25_pozhen_unresp", 0);
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => {
            let mode = m.card.unwrap_or(101);
            let had_zhanyi = flag(ctx.core, owner, "l25_pozhen_zhanyi") > 0;
            let had_chongfeng = flag(ctx.core, owner, "l25_pozhen_chongfeng") > 0;
            if mode == 101 || mode == 104 {
                set_flag(ctx.core, owner, "l25_pozhen_zhanyi", 1);
            } else if mode == 102 {
                set_flag(ctx.core, owner, "l25_pozhen_chongfeng", 1);
            }
            let Some(opp) = m.target.filter(|&p| p != owner && ctx.core.player(p).alive) else {
                return Flow::Done;
            };
            if (had_zhanyi || mode == 103) && (mode == 101 || mode == 102 || mode == 103) {
                let has_basic = ctx
                    .core
                    .player(opp)
                    .hand
                    .iter()
                    .any(|&c| ctx.catalog.card_kind(c).is_basic_kind());
                if !has_basic {
                    if mode == 103 {
                        if ctx.current_card_use().is_ok() {
                            ctx.current_card_use_mut().cancelled = true;
                        }
                    } else {
                        set_flag(ctx.core, owner, "l25_pozhen_unresp", 1);
                    }
                    return Flow::Done;
                }
                return wait_ask(m, ctx, HostStage::SelectCard, ASK_POZHEN_TARGET_DISCARD);
            }
            if had_chongfeng && !ctx.core.player(opp).hand.is_empty() {
                return wait_ask(m, ctx, HostStage::JiefenOwnerDiscard, ASK_POZHEN_GAIN_CARD);
            }
            Flow::Done
        }
        HostStage::SelectCard => {
            let opp = m.target.unwrap_or(owner);
            let mode = m.card.unwrap_or(101);
            let picked = ctx
                .res
                .answer
                .as_ref()
                .and_then(|ans| ans.first().copied())
                .and_then(|o| match o {
                    ChoiceOption::Card(c) => Some(c),
                    _ => None,
                });
            set_flag(ctx.core, owner, "l25_batch_b_ask_id", 0);
            if let Some(c) = picked {
                discard_he_cards(ctx, opp, &[c]);
                m.stage = HostStage::CardsAsk;
                Flow::Yield
            } else {
                if mode == 103 {
                    if ctx.current_card_use().is_ok() {
                        ctx.current_card_use_mut().cancelled = true;
                    }
                } else {
                    set_flag(ctx.core, owner, "l25_pozhen_unresp", 1);
                }
                Flow::Done
            }
        }
        HostStage::JiefenOwnerDiscard => {
            let opp = m.target.unwrap_or(owner);
            let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            set_flag(ctx.core, owner, "l25_batch_b_ask_id", 0);
            if ctx.core.player(opp).hand.contains(&c) {
                ctx.take_owned_card(opp, c);
                ctx.core.player_mut(owner).hand.push(c);
                ctx.out.push(LogEvent::CardsTaken {
                    from: opp,
                    to: owner,
                    cards: vec![c],
                    public: false,
                });
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

