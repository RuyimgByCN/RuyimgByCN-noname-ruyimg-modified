//! L25 祖安武将 Batch A 后半技能步进（神将技能 14–23 + 77 个被动展示/语音钩子）。

use super::l25_batch_a_asks::wait_ask;
use super::*;
use crate::catalog::{CardKind, QueryCtx, TriggerMeta};
use crate::choice::ChoiceOption;
use crate::event::{DamageNature, EventData, EventKind, Phase, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow, LogEvent};

pub(super) fn is_l25_ui_hook(name: &str) -> bool {
    name.starts_with('_')
        && !matches!(
            name,
            "_zuansaidskill"
                | "_yongjinskill"
                | "_kongchengskill"
                | "_anyeskill"
                | "_taohuaskill"
                | "_lunjianskill"
                | "_jianghunskill"
                | "_thunderjudgemodeskill"
                | "_thundermodeskill"
                | "_goodluckjuedgeskill"
        )
}

pub(super) fn ui_hook_trigger_meta(name: &str) -> Option<TriggerMeta> {
    if !is_l25_ui_hook(name) {
        return None;
    }
    let silent = |event: EventKind, extra_events: Vec<EventKind>| TriggerMeta {
        event,
        timing: Timing::Normal,
        priority: 1,
        order_selectable: false,
        optional: false,
        extra_events,
    };
    let m = match name {
        "_kaishitexiao" | "_gameStart" | "_huihekaishitexiao" | "_zuanRoundTX"
        | "_zuanSTARTsayskill" | "_startgamesay" | "_newBackgroundAtart" => {
            silent(EventKind::TurnBegin, vec![EventKind::PhaseBegin])
        }
        "_phaseStart" | "_phaseJudge" | "_phaseDraw" | "_phaseUse" | "_phaseDiscard"
        | "_phaseEnd" | "_guanxingtexiao" | "_guanxing" | "_luoshentexiao" | "_luoshen" => {
            silent(EventKind::PhaseBegin, Vec::new())
        }
        "_biyue" | "_phaseendskill" => silent(EventKind::PhaseEnd, vec![EventKind::PhaseBegin]),
        "_pandingtexiao" | "_guidaotexiao" | "_shandian" | "_basicjuedgeskill"
        | "_bingleAddJudge" | "_bingleLoseJudge" | "_extrajudgeskill" | "_judgeskill" => {
            silent(EventKind::OrdinaryJudge, vec![EventKind::PhaseBegin])
        }
        "_zhuangbeitexiao" | "_equip" | "_equipuseskill" => {
            silent(EventKind::Equipped, vec![EventKind::PhaseBegin])
        }
        "_jisha" | "_win" | "_die" | "_dieskill" | "_extradieskill" | "_diesay"
        | "_zuan_killSendFlower" | "_zuanidentitydieafterskill" | "_zuanidentitytodieskill"
        | "_binglediecls" => silent(EventKind::Death, Vec::new()),
        "_damage" | "_damage2" | "_bahuangtexiao" | "_bahuang" | "_jianxiong" | "_fankui"
        | "_ganglie" | "_yiji" | "_zuan_damageSendEgg" | "_zuan_damageSendFlower"
        | "_extradamageskill" | "_damsayskill" | "_damagesay" | "_zuanidentitydamskill" => {
            silent(EventKind::Damage, vec![EventKind::PhaseBegin])
        }
        "_huifutexiao" | "_recover" | "_qingnang" | "_recoversay" | "_zuanidentityrecskill" => {
            silent(EventKind::Recover, vec![EventKind::PhaseBegin])
        }
        "_loseHp" | "_loseMaxHp" | "_kurou" | "_extradlosehpskill" | "_ssrchangehpskill"
        | "_zuanCHANGEHPsayskill" | "_bingletexiaochangehp" => {
            silent(EventKind::LoseHp, vec![EventKind::PhaseBegin])
        }
        "_dying" | "_bingleDying" | "_zuanidentitydyingskill" | "_saveskill" => {
            silent(EventKind::Dying, vec![EventKind::PhaseBegin])
        }
        "_turnover" | "_extraturnoverskill" => {
            silent(EventKind::TurnOvered, vec![EventKind::TurnOverBefore, EventKind::PhaseBegin])
        }
        "_link" | "_extralinkskill" | "_linkskill" => {
            silent(EventKind::Linked, vec![EventKind::PhaseBegin])
        }
        "_getcard" | "_extragainskill" => {
            silent(EventKind::CardsDrawn, vec![EventKind::PhaseBegin])
        }
        "_discard" | "_zhiheng" | "_extraloseCardskill" | "_extraloseskill" => {
            silent(EventKind::CardsDiscarded, vec![EventKind::PhaseBegin])
        }
        "_kongcheng" => silent(EventKind::CardsLost, vec![EventKind::CardsDiscarded]),
        "_yuyin_skill" | "_voicewhenusecard" | "_useCard" | "_shaungxiongtexiao"
        | "_jijiangtexiao" | "_wushengtexiao" | "_qixitexiao" | "_rende" | "_liuli"
        | "_tieji" | "_guose" | "_paoxiao" | "_jizhi" | "_longdan" | "_wushuang"
        | "_qingguo" | "_lijian" | "_huogong" | "_tiesuo" | "_sha" | "_shan" | "_tao"
        | "_jiu" | "_wuzhong" | "_juedou" | "_nanman" | "_wanjian" | "_taoyuan" | "_wugu"
        | "_shunshou" | "_guohe" | "_jiedao" | "_wuxie" | "_lebusishu" | "_bingliang" => {
            silent(EventKind::CardUsed, vec![EventKind::CardResponded])
        }
        _ => silent(
            EventKind::PhaseBegin,
            vec![EventKind::TurnBegin, EventKind::CardUsed, EventKind::Damage],
        ),
    };
    Some(m)
}

pub(super) fn filter_ui_hook(
    name: &str,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    match name {
        "_kaishitexiao" | "_gameStart" | "_huihekaishitexiao" => {
            matches!(event, EventData::TurnBegin { player } if *player == owner)
        }
        "_phaseStart" | "_guanxingtexiao" | "_guanxing" | "_luoshentexiao" | "_luoshen" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } if *player == owner
        ),
        "_phaseJudge" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::Judge,
                begin: true,
            } if *player == owner
        ),
        "_phaseDraw" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::Draw,
                begin: true,
            } if *player == owner
        ),
        "_phaseUse" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::Play,
                begin: true,
            } if *player == owner
        ),
        "_phaseDiscard" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::Discard,
                begin: true,
            } if *player == owner
        ),
        "_phaseEnd" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::End,
                begin: true,
            } if *player == owner
        ),
        "_biyue" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::End,
                begin: false,
            } if *player == owner
        ),
        "_pandingtexiao" | "_guidaotexiao" | "_shandian" => matches!(
            event,
            EventData::OrdinaryJudge {
                owner: judge_owner,
                ..
            } if *judge_owner == owner
        ),
        "_zhuangbeitexiao" | "_equip" => matches!(
            event,
            EventData::Equipped { player, .. } if *player == owner
        ),
        "_jisha" | "_zuan_killSendFlower" => matches!(
            event,
            EventData::Death { killer: Some(k), .. } if *k == owner
        ),
        "_win" | "_die" | "_dieskill" | "_extradieskill" | "_diesay"
        | "_zuanidentitydieafterskill" | "_zuanidentitytodieskill" | "_binglediecls" => {
            matches!(event, EventData::Death { player, .. } if *player == owner)
        }
        "_damage" | "_damage2" | "_bahuangtexiao" | "_bahuang" => matches!(
            event,
            EventData::Damage(d)
                if !d.prevented && d.amount > 0 && (d.source == Some(owner) || d.target == owner)
        ),
        "_jianxiong" | "_fankui" | "_ganglie" | "_yiji" => matches!(
            event,
            EventData::Damage(d) if !d.prevented && d.amount > 0 && d.target == owner
        ),
        "_huifutexiao" | "_recover" | "_qingnang" => matches!(
            event,
            EventData::Recover { player, .. } if *player == owner
        ),
        "_loseHp" | "_loseMaxHp" | "_kurou" => matches!(
            event,
            EventData::LoseHp { player, .. } if *player == owner
        ),
        "_dying" => matches!(event, EventData::Dying { player, .. } if *player == owner),
        "_turnover" => matches!(
            event,
            EventData::TurnOvered { player, .. } | EventData::TurnOverBefore { player, .. }
                if *player == owner
        ),
        "_link" => matches!(event, EventData::Linked { player, .. } if *player == owner),
        "_getcard" => matches!(event, EventData::CardsDrawn { player, .. } if *player == owner),
        "_discard" | "_zhiheng" => matches!(
            event,
            EventData::CardsDiscarded { player, .. } if *player == owner
        ),
        "_kongcheng" => match event {
            EventData::CardsLost { player, .. } | EventData::CardsDiscarded { player, .. } => {
                *player == owner && ctx.state.player(owner).hand.is_empty()
            }
            _ => false,
        },
        _ => match event {
            EventData::CardUsed(u) if u.player == owner => {
                let kind = u
                    .use_as
                    .or_else(|| u.card.map(|c| ctx.catalog.card_kind(c)));
                match name {
                    "_sha" | "_tieji" | "_paoxiao" | "_wushuang" | "_wushengtexiao"
                    | "_jijiangtexiao" => matches!(
                        kind,
                        Some(CardKind::Sha | CardKind::FireSha | CardKind::LeiSha)
                    ),
                    "_shan" | "_qingguo" => matches!(kind, Some(CardKind::Shan)),
                    "_tao" => matches!(kind, Some(CardKind::Tao)),
                    "_jiu" => matches!(kind, Some(CardKind::Jiu)),
                    "_wuzhong" => matches!(kind, Some(CardKind::Wuzhong)),
                    "_juedou" | "_shaungxiongtexiao" | "_lijian" => {
                        matches!(kind, Some(CardKind::Duel))
                    }
                    "_nanman" => matches!(kind, Some(CardKind::SavageAttack)),
                    "_wanjian" => matches!(kind, Some(CardKind::ArcheryAttack)),
                    "_taoyuan" => matches!(kind, Some(CardKind::Taoyuan)),
                    "_wugu" => matches!(kind, Some(CardKind::Wugu)),
                    "_shunshou" => matches!(kind, Some(CardKind::Snatch)),
                    "_guohe" | "_qixitexiao" => matches!(kind, Some(CardKind::Dismantle)),
                    "_jiedao" => matches!(kind, Some(CardKind::Jiedao)),
                    "_wuxie" => matches!(kind, Some(CardKind::Nullify)),
                    "_lebusishu" | "_guose" => matches!(kind, Some(CardKind::Le)),
                    "_bingliang" => matches!(kind, Some(CardKind::Bing)),
                    "_tiesuo" => matches!(kind, Some(CardKind::Chain)),
                    _ => true,
                }
            }
            EventData::CardResponded { player, card } if *player == owner => {
                let kind = ctx.catalog.card_kind(*card);
                match name {
                    "_sha" | "_longdan" => {
                        matches!(kind, CardKind::Sha | CardKind::FireSha | CardKind::LeiSha | CardKind::Shan)
                    }
                    "_shan" | "_qingguo" => kind == CardKind::Shan,
                    "_yuyin_skill" | "_voicewhenusecard" | "_useCard" => true,
                    _ => false,
                }
            }
            EventData::Phase { player, .. }
            | EventData::TurnBegin { player }
            | EventData::OrdinaryJudge { owner: player, .. }
            | EventData::Equipped { player, .. }
            | EventData::Recover { player, .. }
            | EventData::LoseHp { player, .. }
            | EventData::Dying { player, .. }
            | EventData::TurnOvered { player, .. }
            | EventData::Linked { player, .. }
            | EventData::CardsDrawn { player, .. }
            | EventData::CardsDiscarded { player, .. }
                if *player == owner
                    && !matches!(
                        name,
                        "_yuyin_skill"
                            | "_voicewhenusecard"
                            | "_useCard"
                            | "_shaungxiongtexiao"
                            | "_jijiangtexiao"
                            | "_wushengtexiao"
                            | "_qixitexiao"
                            | "_rende"
                            | "_liuli"
                            | "_tieji"
                            | "_guose"
                            | "_paoxiao"
                            | "_jizhi"
                            | "_longdan"
                            | "_wushuang"
                            | "_qingguo"
                            | "_lijian"
                            | "_huogong"
                            | "_tiesuo"
                            | "_sha"
                            | "_shan"
                            | "_tao"
                            | "_jiu"
                            | "_wuzhong"
                            | "_juedou"
                            | "_nanman"
                            | "_wanjian"
                            | "_taoyuan"
                            | "_wugu"
                            | "_shunshou"
                            | "_guohe"
                            | "_jiedao"
                            | "_wuxie"
                            | "_lebusishu"
                            | "_bingliang"
                    ) =>
            {
                true
            }
            EventData::Damage(d)
                if !d.prevented
                    && d.amount > 0
                    && (d.target == owner || d.source == Some(owner)) =>
            {
                true
            }
            _ => false,
        },
    }
}

pub(super) fn handle_l25_batch_a_ext(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if is_l25_ui_hook(&m.def_key) {
        let key = format!("l25_ui_hook:{}", m.def_key);
        let cnt = flag(ctx.core, owner, &key) + 1;
        set_flag(ctx.core, owner, &key, cnt);
        let vfx_key = format!("l25_vfx_fired:{}", m.def_key);
        set_flag(ctx.core, owner, &vfx_key, cnt);
        let total = flag(ctx.core, owner, "l25_ui_hook_invoked") + 1;
        set_flag(ctx.core, owner, "l25_ui_hook_invoked", total);
        return Flow::Done;
    }
    match m.def_key.as_str() {
        "wumou" => step_wumou(m, ctx),
        "ol_wuqian" => step_ol_wuqian(m, ctx),
        "new_guixin" => step_new_guixin(m, ctx),
        "kuangfeng" => step_kuangfeng(m, ctx),
        "dawu" => step_dawu(m, ctx),
        "dawu2" => step_dawu2(m, ctx),
        "qinyin" => step_qinyin(m, ctx),
        "xinjuejing" => step_xinjuejing(m, ctx),
        "renjie" => {
            let add = m.cards.first().copied().unwrap_or(1);
            let cur = flag(ctx.core, owner, "mark:ren") + add;
            set_flag(ctx.core, owner, "mark:ren", cur);
            set_flag(ctx.core, owner, "renjie_marks", cur);
            Flow::Done
        }
        "wushen" => {
            set_flag(ctx.core, owner, "wushen_unblockable_sha", 1);
            let cnt = flag(ctx.core, owner, "l25_wushen_triggered") + 1;
            set_flag(ctx.core, owner, "l25_wushen_triggered", cnt);
            Flow::Done
        }
        "new_wuhun" => step_new_wuhun(m, ctx),
        "g_nzry_jieying" => step_jieying(m, ctx),
        "nzry_longnu" => step_longnu(m, ctx),
        _ => Flow::Done,
    }
}

fn step_wumou(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            set_flag(ctx.core, owner, "l25_wumou_triggered", 1);
            if flag(ctx.core, owner, "mark:baonu") == 0 {
                ctx.emit(EventData::LoseHp {
                    player: owner,
                    amount: 1,
                });
                m.stage = HostStage::CardsAsk;
                return Flow::Yield;
            }
            wait_ask(m, ctx, HostStage::SelectNature, ASK_WUMOU_MODE)
        }
        HostStage::SelectNature => {
            let tag = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(v)) => v,
                _ => 0,
            };
            set_flag(ctx.core, owner, "l25_ask_id", 0);
            if tag == 0 {
                let cur = flag(ctx.core, owner, "mark:baonu");
                set_flag(ctx.core, owner, "mark:baonu", cur.saturating_sub(1));
                Flow::Done
            } else {
                ctx.emit(EventData::LoseHp {
                    player: owner,
                    amount: 1,
                });
                m.stage = HostStage::CardsAsk;
                Flow::Yield
            }
        }
        _ => Flow::Done,
    }
}

fn step_ol_wuqian(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.targets.is_empty() {
        let cur = flag(ctx.core, owner, "mark:baonu");
        set_flag(ctx.core, owner, "mark:baonu", cur.saturating_sub(2));
        let stamp = turn_stamp(ctx.core);
        set_flag(ctx.core, owner, "ol_wuqian_used_turn", stamp);
        set_flag(ctx.core, owner, "ol_wuqian_active_turn", stamp);
        if let Some(def) = ctx.catalog.skill_by_name("wushuang") {
            ctx.add_skill_until_turn_end(owner, def);
        }
        for p in alive_others(ctx.core, owner) {
            set_flag(ctx.core, p, "qinggang_disable_armor", 1);
        }
    } else {
        set_flag(
            ctx.core,
            owner,
            "l25_ol_wuqian_unblockable",
            m.targets.len() as u32,
        );
        for &p in &m.targets {
            set_flag(ctx.core, p, "qinggang_disable_armor", 1);
        }
    }
    Flow::Done
}

fn step_new_guixin(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectNature, ASK_NEW_GUIXIN_CONFIRM),
        HostStage::SelectNature => {
            let ok = matches!(
                ctx.answer().first().copied(),
                Some(ChoiceOption::Tag(1)) | Some(ChoiceOption::Confirm)
            );
            set_flag(ctx.core, owner, "l25_ask_id", 0);
            if !ok {
                return Flow::Done;
            }
            for p in alive_others(ctx.core, owner) {
                let cands = hej_cards(ctx.core, p);
                if let Some(&c) = cands.first() {
                    ctx.take_owned_card(p, c);
                    ctx.core.player_mut(owner).hand.push(c);
                    ctx.out.push(LogEvent::CardsTaken {
                        from: p,
                        to: owner,
                        cards: vec![c],
                        public: false,
                    });
                }
            }
            let flipped = {
                let p = ctx.core.player_mut(owner);
                p.face_down = !p.face_down;
                p.face_down
            };
            ctx.emit(EventData::TurnOvered {
                player: owner,
                face_down: flipped,
            });
            let cnt = flag(ctx.core, owner, "l25_new_guixin_count") + 1;
            set_flag(ctx.core, owner, "l25_new_guixin_count", cnt);
            let rem = m.cards.first().copied().unwrap_or(1).saturating_sub(1);
            if rem > 0
                && alive_others(ctx.core, owner)
                    .iter()
                    .any(|&p| !hej_cards(ctx.core, p).is_empty())
            {
                m.cards = vec![rem];
                return wait_ask(m, ctx, HostStage::SelectNature, ASK_NEW_GUIXIN_CONFIRM);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn consume_qixing(ctx: &mut Ctx<'_>, owner: PlayerId, count: usize) {
    for _ in 0..count {
        if let Some(&c) = ctx.core.player(owner).off_court.first() {
            ctx.off_court_discard(owner, c);
        } else {
            let cur = flag(ctx.core, owner, "mark:qixing");
            set_flag(ctx.core, owner, "mark:qixing", cur.saturating_sub(1));
        }
    }
}

fn step_kuangfeng(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(2) {
        for p in 0..ctx.core.players.len() as PlayerId {
            set_flag(ctx.core, p, "kuangfeng_marked", 0);
        }
        set_flag(ctx.core, owner, "kuangfeng_target", 0);
        return Flow::Done;
    }
    if m.card == Some(3) {
        if let Ok(d) = ctx.current_damage() {
            if !d.prevented
                && d.amount > 0
                && d.nature == DamageNature::Fire
                && flag(ctx.core, d.target, "kuangfeng_marked") > 0
            {
                ctx.modify_damage_amount(1);
                let cnt = flag(ctx.core, owner, "l25_kuangfeng_boosted") + 1;
                set_flag(ctx.core, owner, "l25_kuangfeng_boosted", cnt);
            }
        }
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_KUANGFENG_TARGET),
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            set_flag(ctx.core, owner, "l25_ask_id", 0);
            consume_qixing(ctx, owner, 1);
            set_flag(ctx.core, tgt, "kuangfeng_marked", 1);
            set_flag(ctx.core, owner, "kuangfeng_target", u32::from(tgt) + 1);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_dawu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(2) {
        for p in 0..ctx.core.players.len() as PlayerId {
            set_flag(ctx.core, p, "dawu_marked", 0);
        }
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_DAWU_TARGETS),
        HostStage::TargetChoice => {
            let picked: Vec<PlayerId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Player(p) => Some(p),
                    _ => None,
                })
                .collect();
            set_flag(ctx.core, owner, "l25_ask_id", 0);
            if picked.is_empty() {
                return Flow::Done;
            }
            consume_qixing(ctx, owner, picked.len());
            for &p in &picked {
                set_flag(ctx.core, p, "dawu_marked", 1);
            }
            set_flag(ctx.core, owner, "l25_dawu_count", picked.len() as u32);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_dawu2(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if let Ok(d) = ctx.current_damage() {
        if !d.prevented
            && d.amount > 0
            && d.nature != DamageNature::Thunder
            && (d.target == owner || flag(ctx.core, d.target, "dawu_marked") > 0)
        {
            ctx.current_damage_mut().prevented = true;
            ctx.current_damage_mut().amount = 0;
            let cnt = flag(ctx.core, owner, "l25_dawu2_prevented") + 1;
            set_flag(ctx.core, owner, "l25_dawu2_prevented", cnt);
        }
    }
    Flow::Done
}

fn step_qinyin(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act if !m.cards.is_empty() => {
            let stamp = turn_stamp(ctx.core);
            let prev = if flag(ctx.core, owner, "qinyin_discard_stamp") == stamp {
                flag(ctx.core, owner, "qinyin_discard_cnt")
                    .max(flag(ctx.core, owner, "qinyin_discard_count"))
            } else {
                0
            };
            let next = prev + m.cards.len() as u32;
            set_flag(ctx.core, owner, "qinyin_discard_stamp", stamp);
            set_flag(ctx.core, owner, "qinyin_discard_cnt", next);
            set_flag(ctx.core, owner, "qinyin_discard_count", next);
            Flow::Done
        }
        HostStage::Act => {
            set_flag(ctx.core, owner, "qinyin_discard_cnt", 0);
            set_flag(ctx.core, owner, "qinyin_discard_count", 0);
            wait_ask(m, ctx, HostStage::SelectNature, ASK_QINYIN_MODE)
        }
        HostStage::SelectNature => {
            let mode = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(v)) => v,
                _ => return Flow::Done,
            };
            set_flag(ctx.core, owner, "l25_ask_id", 0);
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

fn step_xinjuejing(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage != HostStage::Act {
        return Flow::Done;
    }
    if m.card == Some(1) {
        let cnt = flag(ctx.core, owner, "l25_xinjuejing_dying") + 1;
        set_flag(ctx.core, owner, "l25_xinjuejing_dying", cnt);
    } else {
        let cnt = flag(ctx.core, owner, "l25_xinjuejing_draw") + 1;
        set_flag(ctx.core, owner, "l25_xinjuejing_draw", cnt);
    }
    ctx.start_draw(owner, 1);
    m.stage = HostStage::CardsAsk;
    Flow::Yield
}

fn resolve_new_wuhun_death(m: &mut HostMachine, ctx: &mut Ctx<'_>, tgt: PlayerId) -> Flow {
    let owner = m.owner;
    for p in 0..ctx.core.players.len() as PlayerId {
        set_flag(ctx.core, p, "mark:mengyan", 0);
    }
    let jc = ctx.draw_judge_card();
    ctx.to_discard(jc);
    let safe = matches!(ctx.catalog.card_kind(jc), CardKind::Tao | CardKind::Taoyuan);
    set_flag(ctx.core, owner, "l25_new_wuhun_target", u32::from(tgt) + 1);
    set_flag(ctx.core, owner, "l25_new_wuhun_judge_card", u32::from(jc) + 1);
    if !safe && ctx.core.player(tgt).alive {
        ctx.core.player_mut(tgt).hp = 0;
        ctx.core.player_mut(tgt).alive = false;
        ctx.emit(EventData::Death {
            player: tgt,
            killer: Some(owner),
        });
        ctx.log(LogEvent::PlayerDied {
            player: tgt,
            killer: Some(owner),
            presentation: None,
        });
        m.stage = HostStage::CardsAsk;
        return Flow::Yield;
    }
    Flow::Done
}

fn step_new_wuhun(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(1) {
        if let Some(src) = m.target {
            let amt = m.cards.first().copied().unwrap_or(1);
            let cur = flag(ctx.core, src, "mark:mengyan") + amt;
            set_flag(ctx.core, src, "mark:mengyan", cur);
        }
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => {
            let others = alive_others(ctx.core, owner);
            let max_mark = others
                .iter()
                .map(|&p| flag(ctx.core, p, "mark:mengyan"))
                .max()
                .unwrap_or(0);
            if max_mark == 0 {
                return Flow::Done;
            }
            let cands: Vec<PlayerId> = others
                .into_iter()
                .filter(|&p| flag(ctx.core, p, "mark:mengyan") == max_mark)
                .collect();
            if cands.len() == 1 {
                return resolve_new_wuhun_death(m, ctx, cands[0]);
            }
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_NEW_WUHUN_TARGET)
        }
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            set_flag(ctx.core, owner, "l25_ask_id", 0);
            resolve_new_wuhun_death(m, ctx, tgt)
        }
        _ => Flow::Done,
    }
}

fn step_jieying(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(1) {
        set_flag(ctx.core, owner, "mark:ying", 1);
        return Flow::Done;
    }
    if m.card == Some(2) {
        if m.stage != HostStage::Act {
            return Flow::Done;
        }
        if let Some(p) = m.target {
            set_flag(ctx.core, owner, "l25_jieying_extra_draw", u32::from(p) + 1);
            ctx.start_draw(p, 1);
            m.stage = HostStage::CardsAsk;
            return Flow::Yield;
        }
        return Flow::Done;
    }
    if m.card == Some(4) {
        if let Some(t) = m.target {
            set_flag(ctx.core, t, "mark:ying", 0);
            if ctx.core.player(owner).alive {
                let hand: Vec<CardId> = ctx.core.player(t).hand.clone();
                if !hand.is_empty() {
                    ctx.give_hand_cards(t, owner, &hand);
                }
                set_flag(ctx.core, owner, "mark:ying", 1);
                set_flag(ctx.core, owner, "l25_jieying_reclaimed", u32::from(t) + 1);
            }
        }
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_JIEYING_TARGET),
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            set_flag(ctx.core, owner, "l25_ask_id", 0);
            set_flag(ctx.core, owner, "mark:ying", 0);
            set_flag(ctx.core, tgt, "mark:ying", 1);
            set_flag(ctx.core, owner, "l25_jieying_moved_to", u32::from(tgt) + 1);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_longnu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            let state = flag(ctx.core, owner, "nzry_longnu_state");
            let stamp = turn_stamp(ctx.core);
            if state == 0 {
                set_flag(ctx.core, owner, "nzry_longnu_state", 1);
                set_flag(ctx.core, owner, "nzry_longnu_fire_turn", stamp);
                set_flag(ctx.core, owner, "nzry_longnu_thunder_turn", 0);
                ctx.emit(EventData::LoseHp {
                    player: owner,
                    amount: 1,
                });
                m.card = Some(1);
                m.stage = HostStage::DrawSettle;
                Flow::Yield
            } else {
                set_flag(ctx.core, owner, "nzry_longnu_state", 0);
                set_flag(ctx.core, owner, "nzry_longnu_thunder_turn", stamp);
                set_flag(ctx.core, owner, "nzry_longnu_fire_turn", 0);
                ctx.lose_max_hp(owner, 1);
                if ctx.core.player(owner).alive {
                    ctx.start_draw(owner, 1);
                    m.stage = HostStage::CardsAsk;
                    return Flow::Yield;
                }
                Flow::Done
            }
        }
        HostStage::DrawSettle => {
            if ctx.core.player(owner).alive {
                ctx.start_draw(owner, 1);
                m.stage = HostStage::CardsAsk;
                return Flow::Yield;
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}
