//! L10 `reguhuo` (`史诗卡牌/extension.js:297-420`, `refresh.js:10639-10783`)。

use super::super::*;
use crate::catalog::{CardKind, Catalog};
use crate::ids::{CardId, PlayerId, SkillDefId};
use crate::snap::{ask, ExpectedAsk};
use crate::state::GameStateCore;

pub(in crate::content::host) fn has_unlimited_sha(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> bool {
    let paoxiao_std = !catalog.is_rechanyuan_sealed_name(core, owner, crate::vocabulary::names::PAO_XIAO)
        && catalog
            .skill_by_name(crate::vocabulary::names::PAO_XIAO)
            .is_some_and(|d| core.has_skill(owner, d));
    let paoxiao_ext = !catalog.is_rechanyuan_sealed_name(core, owner, "paoxiao")
        && catalog
            .skill_by_name("paoxiao")
            .is_some_and(|d| core.has_skill(owner, d));
    let zhuge = core.has_active_equip_kind(catalog, owner, CardKind::Zhuge);
    let baolei = !catalog.is_rechanyuan_sealed_name(core, owner, WUDIAN_BAOLEI_KEY)
        && catalog
            .skill_by_name(WUDIAN_BAOLEI_KEY)
            .is_some_and(|d| core.has_skill(owner, d) && core.player(owner).baolei_mode == 1);
    let sbzx = !catalog.is_rechanyuan_sealed_name(core, owner, L27_SBZHAXIANG_KEY)
        && catalog
            .skill_by_name(L27_SBZHAXIANG_KEY)
            .is_some_and(|d| core.has_skill(owner, d))
        && sbzhaxiang_active(core, owner);
    let yuanbaorou = catalog
        .skill_by_name(GJ_FOOD_YUANBAOROU_KEY)
        .is_some_and(|d| core.skills.get(&(owner, d)).is_some_and(|i| i.enabled && !i.once_used));
    let huihuo = !catalog.is_rechanyuan_sealed_name(core, owner, crate::vocabulary::names::BOSS_HUIHUO)
        && catalog
            .skill_by_name(crate::vocabulary::names::BOSS_HUIHUO)
            .is_some_and(|d| core.skills.get(&(owner, d)).is_some_and(|i| i.enabled && !i.once_used));
    let lingshi = !catalog.is_rechanyuan_sealed_name(core, owner, "lingshi")
        && catalog
            .skill_by_name("lingshi")
            .is_some_and(|d| core.skills.get(&(owner, d)).is_some_and(|i| i.enabled && !i.once_used))
        && core
            .player(owner)
            .equips
            .iter()
            .any(|c| catalog.card_slot_of(core, *c) == Some(crate::catalog::EquipSlot::Treasure));
    paoxiao_std || paoxiao_ext || zhuge || baolei || sbzx || yuanbaorou || huihuo || lingshi
}

pub(in crate::content::host) fn reguhuo_all_kinds(catalog: &Catalog) -> Vec<CardKind> {
    let mut kinds: Vec<CardKind> = REGUHUO_KINDS.to_vec();
    for card in &catalog.cards {
        let k = card.kind;
        if !kinds.contains(&k)
            && !k.is_equip()
            && k.is_basic_trick_or_equip()
            && !matches!(
                k,
                CardKind::Le
                    | CardKind::Bing
                    | CardKind::ShanDian
                    | CardKind::HongShui
                    | CardKind::Guiyoujie
                    | CardKind::Hslingjian
            )
        {
            kinds.push(k);
        }
        if kinds.len() >= u8::MAX as usize {
            break;
        }
    }
    kinds
}

pub(in crate::content::host) fn reguhuo_kind_at_catalog(
    catalog: &Catalog,
    idx: u8,
) -> Option<CardKind> {
    reguhuo_all_kinds(catalog).get(idx as usize).copied()
}

/// 上游 `filterCard`（`史诗卡牌/extension.js:306-332`）：按目录与当前上下文
/// （0=出牌、1=闪、2=杀、3=无懈、4=濒死桃、5=濒死自救桃/酒）动态过滤牌名。
pub(in crate::content::host) fn reguhuo_name_options(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    resp_ctx: u32,
) -> Vec<ChoiceOption> {
    let blocked = if resp_ctx == 0 || resp_ctx >= 3 {
        catalog.blocks_card_use_and_save(core, owner)
    } else {
        catalog.blocks_all_cards(core, owner)
    };
    if blocked {
        return Vec::new();
    }
    let all = reguhuo_all_kinds(catalog);
    let has_in_catalog = |kind: CardKind| catalog.cards.iter().any(|c| c.kind == kind);
    let fangzhu_basic_ban = core
        .vm_aux
        .flags
        .get(&(owner, "mbcmfangzhu_ban".to_string()))
        .copied()
        .unwrap_or(0)
        > 0;
    let tao_futai2 = crate::kernel::tao_futai2_banned(core, catalog, owner);

    let mut out = Vec::new();
    for (idx, &kind) in all.iter().enumerate() {
        if !has_in_catalog(kind) {
            continue;
        }
        if fangzhu_basic_ban && kind.is_basic_kind() {
            continue;
        }
        let ok = match resp_ctx {
            1 => kind == CardKind::Shan,
            2 => kind.is_sha_like(),
            3 => kind == CardKind::Nullify,
            4 => kind == CardKind::Tao && !tao_futai2,
            5 => (kind == CardKind::Tao && !tao_futai2) || kind == CardKind::Jiu,
            _ => {
                // Phase::Play 主动使用：排除闪/无懈/延时/仅响应牌，校验体力/限次/目标合法性
                if matches!(
                    kind,
                    CardKind::Shan
                        | CardKind::Nullify
                        | CardKind::Le
                        | CardKind::Bing
                        | CardKind::ShanDian
                        | CardKind::HongShui
                        | CardKind::Guiyoujie
                        | CardKind::Hufu
                        | CardKind::Youdishenru
                        | CardKind::Jinlianzhu
                        | CardKind::Shenmiguo
                        | CardKind::Jiguan
                        | CardKind::Hslingjian
                ) {
                    false
                } else if kind.is_sha_like() {
                    let sha_ok = !core.turn.sha_used_play_phase || has_unlimited_sha(core, catalog, owner);
                    let (min, _, tgts) = core.legal_targets(catalog, kind, owner);
                    sha_ok && tgts.len() >= min as usize
                } else if kind == CardKind::Tao {
                    !tao_futai2 && core.player(owner).hp < core.player(owner).max_hp as i32
                } else if kind == CardKind::Dinvxuanshuang {
                    core.player(owner).hp < core.player(owner).max_hp as i32
                } else if matches!(kind, CardKind::Jiu | CardKind::Tianxianjiu) {
                    !core.turn.jiu_used
                } else if kind == CardKind::Changshezhen {
                    core.inline(catalog, owner)
                        || core
                            .alive_players()
                            .any(|p| p != owner && core.is_friend_of(catalog, owner, p))
                } else if kind == CardKind::Dizaizhen {
                    core.alive_players().any(|p| !core.is_major_faction(catalog, p))
                } else if matches!(
                    kind,
                    CardKind::Tianfuzhen | CardKind::Qixingzhen | CardKind::Niaoxiangzhen
                ) {
                    core.players.len() >= 4
                } else if matches!(
                    kind,
                    CardKind::Wugu
                        | CardKind::Taoyuan
                        | CardKind::SavageAttack
                        | CardKind::ArcheryAttack
                        | CardKind::Jingleishan
                        | CardKind::Chiyuxi
                        | CardKind::Xianluhui
                ) {
                    !core.legal_targets(catalog, kind, owner).2.is_empty()
                } else {
                    let (min, _, tgts) = core.legal_targets(catalog, kind, owner);
                    min == 0 || tgts.len() >= min as usize
                }
            }
        };
        if ok {
            out.push(ChoiceOption::Tag(idx as u8));
        }
    }
    out
}

pub(crate) fn reguhuo_response_skill(
    core: &GameStateCore,
    catalog: &Catalog,
    responder: PlayerId,
    resp_ctx: u8,
) -> Option<SkillDefId> {
    if !core.player(responder).alive
        || core.player(responder).hand.is_empty()
        || catalog.blocks_all_cards(core, responder)
    {
        return None;
    }
    let def = catalog.skill_by_name(EPIC_REGUHUO_KEY)?;
    if !core.has_skill(responder, def)
        || catalog.is_tieji_sealed(core, responder)
        || catalog.is_rechanyuan_sealed_def(core, responder, def)
    {
        return None;
    }
    if core
        .vm_aux
        .once
        .contains(&(responder, EPIC_REGUHUO_KEY.to_string(), 1))
    {
        return None;
    }
    if reguhuo_name_options(core, catalog, responder, u32::from(resp_ctx)).is_empty() {
        return None;
    }
    Some(def)
}

pub(crate) fn reguhuo_spawn_response(
    ctx: &mut Ctx<'_>,
    responder: PlayerId,
    resp_ctx: u8,
    dying: PlayerId,
    skill_def: SkillDefId,
    steal_target: PlayerId,
) {
    let _ = ctx.core.vm_aux.flags.remove(&(responder, "l10_reguhuo_resp_done".to_string()));
    let _ = ctx.core.vm_aux.flags.remove(&(responder, "l10_reguhuo_resp_card".to_string()));
    let _ = ctx.core.vm_aux.flags.remove(&(responder, "l10_reguhuo_resp_kind".to_string()));
    if ctx.catalog.skill_by_name("scsanruo") == Some(skill_def) {
        super::l10_scs::scsanruo_spawn_response(ctx, responder, resp_ctx, dying, steal_target);
        return;
    }
    if ctx.catalog.skill_by_name("jingxiang") == Some(skill_def) {
        super::u05_hearth::u05_b6b_hearth::jingxiang_spawn_response(
            ctx, responder, resp_ctx, dying,
        );
        return;
    }
    let machine = Box::new(HostMachine {
        def_key: EPIC_REGUHUO_KEY.to_string(),
        behavior: HostBehavior::Reguhuo,
        owner: responder,
        target: (resp_ctx == 4 || resp_ctx == 5).then_some(dying),
        card: None,
        cards: vec![u32::from(resp_ctx)],
        targets: Vec::new(),
        stage: HostStage::Act,
    });
    ctx.res.parks.push(crate::resolution::Park {
        resume_depth: ctx.res.stack.len(),
        slot: crate::resolution::Slot::Trigger {
            frame: crate::resolution::NO_FRAME,
        },
        machine,
        awaiting_answer: false,
    });
}

pub(crate) fn reguhuo_consume_resp(core: &mut GameStateCore, responder: PlayerId) -> Option<CardId> {
    let done = core
        .vm_aux
        .flags
        .remove(&(responder, "l10_reguhuo_resp_done".to_string()))
        .unwrap_or(0);
    let card = core
        .vm_aux
        .flags
        .remove(&(responder, "l10_reguhuo_resp_card".to_string()));
    if done == 1 {
        card
    } else {
        None
    }
}

pub(crate) fn reguhuo_has_pending_final(core: &GameStateCore, responder: PlayerId) -> bool {
    core.vm_aux
        .flags
        .get(&(responder, "l10_reguhuo_final_cnt".to_string()))
        .copied()
        .unwrap_or(0)
        > 0
}

pub(crate) fn reguhuo_spawn_final(ctx: &mut Ctx<'_>, responder: PlayerId, resume_depth: usize) {
    let cnt = ctx
        .core
        .vm_aux
        .flags
        .remove(&(responder, "l10_reguhuo_final_cnt".to_string()))
        .unwrap_or(0);
    let mut betrays = Vec::with_capacity(cnt as usize);
    for i in 0..cnt {
        if let Some(p) = ctx
            .core
            .vm_aux
            .flags
            .remove(&(responder, format!("l10_reguhuo_final:{i}")))
        {
            betrays.push(p as PlayerId);
        }
    }
    if betrays.is_empty() {
        return;
    }
    let machine = Box::new(HostMachine {
        def_key: EPIC_REGUHUO_KEY.to_string(),
        behavior: HostBehavior::Reguhuo,
        owner: responder,
        target: None,
        card: None,
        cards: Vec::new(),
        targets: betrays,
        stage: HostStage::ReguhuoFinalPick,
    });
    ctx.res.parks.push(crate::resolution::Park {
        resume_depth,
        slot: crate::resolution::Slot::Trigger {
            frame: crate::resolution::NO_FRAME,
        },
        machine,
        awaiting_answer: false,
    });
}

fn grant_rechanyuan(ctx: &mut Ctx<'_>, target: PlayerId) {
    ctx.core
        .vm_aux
        .flags
        .insert((target, "l10_rechanyuan".to_string()), 1);
    if let Some(def) = ctx.catalog.skill_by_name("rechanyuan") {
        ctx.insert_skill(
            (target, def),
            crate::state::SkillInstance {
                owner: target,
                enabled: true,
                once_used: false,
                expires_end_of_turn: false,
                rounds: 0,
            },
        );
    }
}

pub(super) fn step_act(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let options: Vec<ChoiceOption> = ctx
        .core
        .player(m.owner)
        .hand
        .iter()
        .copied()
        .map(ChoiceOption::Card)
        .collect();
    if options.is_empty() {
        return Flow::Done;
    }
    if m.cards.is_empty() {
        m.cards = vec![0];
    }
    m.stage = HostStage::SelectCard;
    Flow::WaitChoice {
        observer: m.owner,
        spec: ChoiceSpec {
            kind: ChoiceKind::SelectCards,
            prompt: "reguhuo_card",
            options,
            min: 1,
            max: 1,
            allow_cancel: true,
        },
    }
}

pub(super) fn step_select_card(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let Some(ChoiceOption::Card(card)) = ctx.answer().first().copied() else {
        // 取消选牌不消耗回合限一次（refresh.js:10653）
        return Flow::Done;
    };
    if !ctx.core.player(m.owner).hand.contains(&card) {
        return Flow::Done;
    }
    let resp_ctx = m.cards.first().copied().unwrap_or(0);
    let name_opts = reguhuo_name_options(ctx.core, ctx.catalog, m.owner, resp_ctx);
    if name_opts.is_empty() {
        return Flow::Done;
    }
    m.card = Some(card);
    m.stage = HostStage::SelectNature;
    Flow::WaitChoice {
        observer: m.owner,
        spec: ChoiceSpec {
            kind: ChoiceKind::PickTag,
            prompt: "reguhuo_name",
            options: name_opts,
            min: 1,
            max: 1,
            allow_cancel: true,
        },
    }
}

pub(super) fn step_select_nature(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let Some(ChoiceOption::Tag(idx)) = ctx.answer().first().copied() else {
        // 取消声明牌名不消耗回合限一次（refresh.js:10653）
        return Flow::Done;
    };
    let Some(card) = m.card else {
        return Flow::Done;
    };
    let Some(kind) = reguhuo_kind_at_catalog(ctx.catalog, idx) else {
        return Flow::Done;
    };
    let resp_ctx = m.cards.first().copied().unwrap_or(0);
    if resp_ctx != 0 {
        return begin_guess(m, ctx, card, idx, kind, resp_ctx, Vec::new());
    }
    let (min, max, targets) = ctx.core.legal_targets(ctx.catalog, kind, m.owner);
    m.cards = vec![u32::from(idx), resp_ctx];
    m.target = None;
    if min == 0 && max == 0 {
        return begin_guess(m, ctx, card, idx, kind, 0, targets);
    }
    if matches!(
        kind,
        CardKind::Tao | CardKind::Jiu | CardKind::Tianxianjiu | CardKind::Dinvxuanshuang
    ) {
        if targets.is_empty() {
            return Flow::Done;
        }
        return begin_guess(m, ctx, card, idx, kind, 0, targets);
    }
    if targets.len() < min as usize {
        return Flow::Done;
    }
    m.stage = HostStage::TargetChoice;
    Flow::WaitChoice {
        observer: m.owner,
        spec: ChoiceSpec {
            kind: ChoiceKind::SelectPlayers,
            prompt: "reguhuo_targets",
            options: targets.into_iter().map(ChoiceOption::Player).collect(),
            min: min as usize,
            max: max as usize,
            allow_cancel: true,
        },
    }
}

pub(super) fn step_target_choice(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let Some(card) = m.card else {
        return Flow::Done;
    };
    let Some(kind_tag) = m.cards.first().copied() else {
        return Flow::Done;
    };
    let Some(kind) = reguhuo_kind_at_catalog(ctx.catalog, kind_tag as u8) else {
        return Flow::Done;
    };
    if kind == CardKind::Jiedao {
        if let Some(holder) = m.target {
            let Some(ChoiceOption::Player(victim)) = ctx.answer().first().copied() else {
                // 取消借刀第二目标不消耗回合限一次（refresh.js:10653）
                m.target = None;
                return Flow::Done;
            };
            m.target = None;
            if !ctx
                .core
                .legal_jiedao_victims(ctx.catalog, holder)
                .contains(&victim)
            {
                return Flow::Done;
            }
            return begin_guess(m, ctx, card, kind_tag as u8, kind, 0, vec![holder, victim]);
        }
        let Some(ChoiceOption::Player(holder)) = ctx.answer().first().copied() else {
            return Flow::Done;
        };
        let victims = ctx.core.legal_jiedao_victims(ctx.catalog, holder);
        if victims.is_empty() {
            return Flow::Done;
        }
        m.target = Some(holder);
        return Flow::WaitChoice {
            observer: m.owner,
            spec: ChoiceSpec {
                kind: ChoiceKind::SelectPlayers,
                prompt: "reguhuo_jiedao_victim",
                options: victims.into_iter().map(ChoiceOption::Player).collect(),
                min: 1,
                max: 1,
                allow_cancel: true,
            },
        };
    }
    let targets: Vec<PlayerId> = ctx
        .answer()
        .iter()
        .filter_map(|o| match o {
            ChoiceOption::Player(p) => Some(*p),
            _ => None,
        })
        .collect();
    let (min, max, _) = ctx.core.legal_targets(ctx.catalog, kind, m.owner);
    if targets.len() < min as usize || targets.len() > max as usize {
        // 取消选目标不消耗回合限一次（refresh.js:10653）
        return Flow::Done;
    }
    begin_guess(m, ctx, card, kind_tag as u8, kind, 0, targets)
}

fn begin_guess(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    card: CardId,
    kind_tag: u8,
    kind: CardKind,
    resp_ctx: u32,
    use_targets: Vec<PlayerId>,
) -> Flow {
    // 真正确认后才记入本回合已使用（reguhuo_phase / vm_aux.once scope=1）
    ctx.core
        .vm_aux
        .once
        .insert((m.owner, m.def_key.clone(), 1));
    if ctx.core.player(m.owner).hand.contains(&card) {
        ctx.take_hand_card(m.owner, card);
    }
    if !ctx.core.table.contains(&card) {
        ctx.core.table.push(card);
    }
    ctx.log(LogEvent::SkillTriggered {
        skill: "蛊惑",
        owner: m.owner,
        presentation: None,
    });

    let guessers: Vec<PlayerId> = ctx
        .core
        .seat_order_from(ctx.core.next_alive(m.owner))
        .into_iter()
        .filter(|&p| {
            p != m.owner && ctx.core.player(p).alive && !ctx.catalog.has_rechanyuan(ctx.core, p)
        })
        .collect();

    m.card = Some(card);
    m.cards = vec![u32::from(kind_tag), resp_ctx, use_targets.len() as u32];
    m.cards.extend(use_targets.iter().copied().map(u32::from));
    m.targets = guessers;
    if m.targets.is_empty() {
        return finish_guess(m, ctx, card, kind, resp_ctx, use_targets, Vec::new());
    }
    let observer = m.targets[0];
    m.stage = HostStage::ReguhuoGuessPick;
    Flow::WaitChoice {
        observer,
        spec: ChoiceSpec {
            kind: ChoiceKind::PickTag,
            prompt: "reguhuo_guess_pick",
            options: vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
            min: 1,
            max: 1,
            allow_cancel: false,
        },
    }
}

pub(super) fn step_guess_pick(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if !m.targets.is_empty() {
        let voter = m.targets.remove(0);
        if matches!(ctx.answer().first(), Some(ChoiceOption::Tag(1))) {
            m.cards.push(u32::from(voter));
        }
    }
    if let Some(&next_obs) = m.targets.first() {
        m.stage = HostStage::ReguhuoGuessPick;
        return Flow::WaitChoice {
            observer: next_obs,
            spec: ChoiceSpec {
                kind: ChoiceKind::PickTag,
                prompt: "reguhuo_guess_pick",
                options: vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
                min: 1,
                max: 1,
                allow_cancel: false,
            },
        };
    }
    let Some(card) = m.card else {
        return Flow::Done;
    };
    let kind_tag = m.cards.first().copied().unwrap_or(0) as u8;
    let resp_ctx = m.cards.get(1).copied().unwrap_or(0);
    let n_tgts = m.cards.get(2).copied().unwrap_or(0) as usize;
    let use_targets: Vec<PlayerId> = m
        .cards
        .iter()
        .skip(3)
        .take(n_tgts)
        .map(|&p| p as PlayerId)
        .collect();
    let betrays: Vec<PlayerId> = m
        .cards
        .iter()
        .skip(3 + n_tgts)
        .map(|&p| p as PlayerId)
        .collect();
    let Some(kind) = reguhuo_kind_at_catalog(ctx.catalog, kind_tag) else {
        return Flow::Done;
    };
    finish_guess(m, ctx, card, kind, resp_ctx, use_targets, betrays)
}

fn finish_guess(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    card: CardId,
    kind: CardKind,
    resp_ctx: u32,
    use_targets: Vec<PlayerId>,
    betrays: Vec<PlayerId>,
) -> Flow {
    let fake = ctx.catalog.card_kind(card) != kind;
    if !betrays.is_empty() && fake {
        // 假牌被质疑：载体入弃牌堆，质疑者各摸 1 张，作废不结算（refresh.js:10726-10730）
        ctx.core.table.retain(|&c| c != card);
        ctx.to_discard(card);
        ctx.log(LogEvent::CardDiscarded {
            player: m.owner,
            cards: vec![card],
        });
        m.card = None;
        m.targets = betrays;
        m.stage = HostStage::DrawSettle;
        let b = m.targets.remove(0);
        ctx.start_draw(b, 1);
        return Flow::Yield;
    }

    if resp_ctx != 0 {
        ctx.core
            .vm_aux
            .flags
            .insert((m.owner, "l10_reguhuo_resp_done".to_string()), 1);
        ctx.core
            .vm_aux
            .flags
            .insert((m.owner, "l10_reguhuo_resp_card".to_string()), card);
        if resp_ctx == 4 || resp_ctx == 5 {
            ctx.core.vm_aux.flags.insert(
                (m.owner, "l10_reguhuo_resp_kind".to_string()),
                if kind == CardKind::Jiu { 5 } else { 4 },
            );
        }
        if !betrays.is_empty() {
            ctx.core.vm_aux.flags.insert(
                (m.owner, "l10_reguhuo_final_cnt".to_string()),
                betrays.len() as u32,
            );
            for (i, b) in betrays.iter().enumerate() {
                ctx.core.vm_aux.flags.insert(
                    (m.owner, format!("l10_reguhuo_final:{i}")),
                    u32::from(*b),
                );
            }
        }
        return Flow::Done;
    }

    let use_data = if kind.is_sha_like() {
        super::sha_card_use_data(ctx, m.owner, card, use_targets, Some(kind))
    } else {
        crate::machine::card_use_as(card, m.owner, use_targets, Some(kind))
    };
    ctx.emit(EventData::CardUsed(use_data));
    if betrays.is_empty() {
        return Flow::Done;
    }
    m.card = None;
    m.target = None;
    m.targets = betrays;
    m.stage = HostStage::ReguhuoFinalPick;
    Flow::Yield
}

pub(super) fn step_draw_settle(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if let Some(b) = (!m.targets.is_empty()).then(|| m.targets.remove(0)) {
        ctx.start_draw(b, 1);
        return Flow::Yield;
    }
    Flow::Done
}

pub(super) fn step_final_pick(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if let Some(t) = m.target.take() {
        if ctx.core.player(t).alive {
            grant_rechanyuan(ctx, t);
        }
    }
    if let Some(t) = m.card.take().map(|v| v as PlayerId) {
        if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
            if ctx.core.player(t).hand.contains(&c) {
                ctx.discard_from_hand(t, c);
            }
            grant_rechanyuan(ctx, t);
        } else {
            m.target = Some(t);
            ctx.emit(EventData::LoseHp {
                player: t,
                amount: 1,
            });
            return Flow::Yield;
        }
    }
    while !m.targets.is_empty() {
        let t = m.targets.remove(0);
        if !ctx.core.player(t).alive {
            continue;
        }
        if ctx.core.player(t).hand.is_empty() {
            m.target = Some(t);
            ctx.emit(EventData::LoseHp {
                player: t,
                amount: 1,
            });
            return Flow::Yield;
        }
        m.card = Some(u32::from(t));
        let options: Vec<ChoiceOption> = ctx
            .core
            .player(t)
            .hand
            .iter()
            .copied()
            .map(ChoiceOption::Card)
            .collect();
        return Flow::WaitChoice {
            observer: t,
            spec: ChoiceSpec {
                kind: ChoiceKind::SelectCards,
                prompt: "reguhuo_final",
                options,
                min: 0,
                max: 1,
                allow_cancel: true,
            },
        };
    }
    Flow::Done
}

pub(in crate::content::host) fn ask_reguhuo(
    stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    card: Option<CardId>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    match stage {
        HostStage::SelectCard => {
            let options: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            (!options.is_empty()).then(|| {
                ask(
                    owner,
                    "reguhuo_card",
                    ChoiceKind::SelectCards,
                    options,
                    1,
                    1,
                    true,
                )
            })
        }
        HostStage::SelectNature => {
            let resp_ctx = cards.first().copied().unwrap_or(0);
            let options = reguhuo_name_options(core, catalog, owner, resp_ctx);
            (!options.is_empty()).then(|| {
                ask(
                    owner,
                    "reguhuo_name",
                    ChoiceKind::PickTag,
                    options,
                    1,
                    1,
                    true,
                )
            })
        }
        HostStage::TargetChoice => {
            let kind_tag = cards.first().copied()?;
            let kind = reguhuo_kind_at_catalog(catalog, kind_tag as u8)?;
            if kind == CardKind::Jiedao {
                if let Some(holder) = target {
                    let victims = core.legal_jiedao_victims(catalog, holder);
                    return (!victims.is_empty()).then(|| {
                        ask(
                            owner,
                            "reguhuo_jiedao_victim",
                            ChoiceKind::SelectPlayers,
                            victims.into_iter().map(ChoiceOption::Player).collect(),
                            1,
                            1,
                            true,
                        )
                    });
                }
            }
            let (min, max, tgts) = core.legal_targets(catalog, kind, owner);
            if tgts.len() < min as usize {
                return None;
            }
            Some(ask(
                owner,
                "reguhuo_targets",
                ChoiceKind::SelectPlayers,
                tgts.into_iter().map(ChoiceOption::Player).collect(),
                min as usize,
                max as usize,
                true,
            ))
        }
        HostStage::ReguhuoGuessPick => {
            let observer = *targets.first()?;
            Some(ask(
                observer,
                "reguhuo_guess_pick",
                ChoiceKind::PickTag,
                vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
                1,
                1,
                false,
            ))
        }
        HostStage::ReguhuoFinalPick => {
            let t = card? as PlayerId;
            let options: Vec<ChoiceOption> = core
                .player(t)
                .hand
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            Some(ask(
                t,
                "reguhuo_final",
                ChoiceKind::SelectCards,
                options,
                0,
                1,
                true,
            ))
        }
        _ => None,
    }
}
