//! U05 `杀海拾遗 · swd` Batch 7b 灵剑与机关锻造子模块（`hslingjian_*` 37 技 + `jiguanyaoshu_skill` + `lingjianduanzao` / `_lingjianduanzao`）。

use super::asks::{wait_confirm, wait_select_cards, wait_select_players};
use super::super::swd_u05_7a::{discard_he_card, hand_or_equip};
use super::super::super::*;
use crate::catalog::{Catalog, EquipSlot, QueryCtx};
use crate::event::{EventData, LostOrigin, Phase};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow};
use crate::state::GameStateCore;

pub(in crate::content::host) fn is_lingjian_equip5_active(name: &str) -> bool {
    matches!(
        name,
        "hslingjian_xuanfengzhiren_equip5"
            | "hslingjian_zhongxinghujia_equip5"
            | "hslingjian_shengxiuhaojiao_equip5"
            | "hslingjian_shijianhuisu_equip5"
    )
}

pub(in crate::content::host) fn can_activate_equip5(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> bool {
    let he = hand_or_equip(ctx.state, owner);
    match skill.name.as_str() {
        "hslingjian_xuanfengzhiren_equip5" => {
            !he.is_empty()
                && ctx
                    .state
                    .alive_players()
                    .any(|p| !hand_or_equip(ctx.state, p).is_empty())
        }
        "hslingjian_zhongxinghujia_equip5" => {
            he.len() >= 2 && find_armor_in_piles(ctx.state, ctx.catalog).is_some()
        }
        "hslingjian_shengxiuhaojiao_equip5" => {
            he.len() >= 2 && ctx.state.alive_players().next().is_some()
        }
        "hslingjian_shijianhuisu_equip5" => {
            !he.is_empty()
                && ctx
                    .state
                    .alive_players()
                    .any(|p| p != owner && !hand_or_equip(ctx.state, p).is_empty())
        }
        _ => false,
    }
}

pub(in crate::content::host) fn can_activate_duanzao(ctx: &QueryCtx<'_>, owner: PlayerId) -> bool {
    let has_mat = ctx
        .state
        .player(owner)
        .hand
        .iter()
        .any(|c| is_forge_material(ctx.catalog, *c));
    let has_eq = !forgeable_equips(ctx.state, ctx.catalog, owner).is_empty();
    has_mat && has_eq
}

pub(super) fn is_forge_material(catalog: &Catalog, card: CardId) -> bool {
    let kind = catalog.card_kind(card);
    if kind == CardKind::Hslingjian {
        return true;
    }
    let name = catalog.cards.get(card as usize).map(|c| c.name).unwrap_or("");
    matches!(
        kind,
        CardKind::Qinglongzhigui
            | CardKind::Zhuquezhizhang
            | CardKind::Baishouzhihu
            | CardKind::Xuanwuzhihuang
            | CardKind::Huanglinzhicong
            | CardKind::Cangchizhibi
            | CardKind::Yuchanqian
            | CardKind::Yuchankun
            | CardKind::Yuchanzhen
            | CardKind::Yuchanxun
            | CardKind::Yuchankan
            | CardKind::Yuchanli
            | CardKind::Yuchangen
            | CardKind::Yuchandui
    ) || name.starts_with("hslingjian_")
}

fn is_unforgeable_equip(core: &GameStateCore, catalog: &Catalog, owner: PlayerId, card: CardId) -> bool {
    if core.player(owner).card_mods.contains_key(&card) {
        return true;
    }
    let kind = catalog.card_kind(card);
    let name = catalog.cards.get(card as usize).map(|c| c.name).unwrap_or("");
    matches!(
        kind,
        CardKind::Donghuangzhong
            | CardKind::Xuanyuanjian
            | CardKind::Pangufu
            | CardKind::Lianyaohu
            | CardKind::Haotianta
            | CardKind::Shennongding
            | CardKind::Kongdongyin
            | CardKind::Kunlunjingc
            | CardKind::Nvwashi
            | CardKind::Gjyuheng
    ) || matches!(
        name,
        "fuxiqin"
            | "swd_fuxiqin"
            | "donghuangzhong"
            | "xuanyuanjian"
            | "pangufu"
            | "lianyaohu"
            | "haotianta"
            | "shennongding"
            | "kongdongyin"
            | "kunlunjingc"
            | "nvwashi"
            | "liulongcanjia"
            | "dinglanyemingzhu"
            | "feilongduofeng"
            | "taipingyaoshu"
            | "jilinqianyi"
            | "mengchong"
    ) || name.starts_with("swd_fuxiqin")
        || name.starts_with("swd_donghuangzhong")
        || name.starts_with("swd_xuanyuanjian")
        || name.starts_with("swd_pangufu")
        || name.starts_with("swd_lianyaohu")
        || name.starts_with("swd_haotianta")
        || name.starts_with("swd_shennongding")
        || name.starts_with("swd_kongdongyin")
        || name.starts_with("swd_kunlunjingc")
        || name.starts_with("swd_nvwashi")
        || name.starts_with("gjyuheng")
        || name.starts_with("gj_gjyuheng")
        || name.contains("liulongcanjia")
        || name.contains("dinglanyemingzhu")
        || name.contains("feilongduofeng")
        || name.contains("taipingyaoshu")
        || name.contains("jilinqianyi")
        || name.contains("mengchong")
}

pub(super) fn forgeable_equips(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<CardId> {
    hand_or_equip(core, owner)
        .into_iter()
        .filter(|c| catalog.card_slot_of(core, *c).is_some() && !is_unforgeable_equip(core, catalog, owner, *c))
        .collect()
}

pub(super) fn filter(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    match skill.behavior {
        HostBehavior::U05SwdLingjianDmgSrc => filter_dmg_src(skill.name.as_str(), ctx, owner, event),
        HostBehavior::U05SwdLingjianDmgTgt => filter_dmg_tgt(skill.name.as_str(), ctx, owner, event),
        HostBehavior::U05SwdLingjianLoseTemp => {
            if !is_lose_event(ctx.state, event, owner) {
                return false;
            }
            match skill.name.as_str() {
                "hslingjian_xuanfengzhiren_equip3" => {
                    ctx.state.turn.current != owner
                        && !ctx
                            .catalog
                            .skill_by_name("hslingjian_xuanfengzhiren_equip3_dist")
                            .is_some_and(|d| ctx.state.has_skill(owner, d))
                }
                "hslingjian_xuanfengzhiren_equip4" => {
                    ctx.state.turn.current == owner
                        && !ctx
                            .catalog
                            .skill_by_name("hslingjian_xuanfengzhiren_equip4_dist")
                            .is_some_and(|d| ctx.state.has_skill(owner, d))
                }
                _ => false,
            }
        }
        HostBehavior::U05SwdLingjianEquipped => match event {
            EventData::Equipped { player, card, .. } if *player == owner => {
                let slot = ctx.catalog.card_slot_of(ctx.state, *card);
                match skill.name.as_str() {
                    "hslingjian_shijianhuisu_equip1" => slot == Some(EquipSlot::Armor),
                    "hslingjian_shijianhuisu_equip2" => slot == Some(EquipSlot::Weapon),
                    _ => false,
                }
            }
            _ => false,
        },
        HostBehavior::U05SwdLingjianDrawPhase => false,
        HostBehavior::U05SwdLingjianPhaseEnd => match skill.name.as_str() {
            "hslingjian_xingtigaizao" => matches!(
                event,
                EventData::Phase { player, phase: Phase::End, begin: false } if *player == owner
            ),
            "hslingjian_jinjilengdong_equip5" => matches!(
                event,
                EventData::TurnEndWindow { player, after: false }
                    if *player == owner && !ctx.state.player(owner).face_down
            ),
            _ => false,
        },
        HostBehavior::U05SwdJiguanyaoshuSkill => {
            if ctx.state.turn.current == owner {
                return false;
            }
            jiguanyaoshu_lost_equip_count(ctx.state, ctx.catalog, owner, event) > 0
        }
        _ => false,
    }
}

fn filter_dmg_src(name: &str, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
    let EventData::Damage(d) = event else {
        return false;
    };
    if d.source != Some(owner)
        || d.prevented
        || !d.card.is_some_and(|c| ctx.catalog.card_kind(c).is_sha_like())
    {
        return false;
    }
    match name {
        "hslingjian_xuanfengzhiren_equip1" => {
            ctx.state.player(d.target).alive && !hand_or_equip(ctx.state, d.target).is_empty()
        }
        "hslingjian_zhongxinghujia_equip1" => {
            find_armor_in_piles(ctx.state, ctx.catalog).is_some()
        }
        "hslingjian_jinjilengdong_equip1" => ctx.state.player(d.target).alive,
        "hslingjian_yinmilichang_equip1" => ctx
            .state
            .alive_players()
            .any(|p| p != owner && !ctx.state.player(p).yinmi_active),
        "hslingjian_xingtigaizao_equip1" => true,
        _ => false,
    }
}

fn filter_dmg_tgt(name: &str, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
    let EventData::Damage(d) = event else {
        return false;
    };
    if d.target != owner || d.prevented {
        return false;
    }
    let sha = d.card.is_some_and(|c| ctx.catalog.card_kind(c).is_sha_like());
    match name {
        "hslingjian_yinmilichang_equip2" => !ctx.state.player(owner).yinmi_active,
        "hslingjian_xingtigaizao_equip2" => sha,
        "hslingjian_xuanfengzhiren_equip2" => {
            sha && d
                .source
                .is_some_and(|s| ctx.state.player(s).alive && !hand_or_equip(ctx.state, s).is_empty())
        }
        "hslingjian_zhongxinghujia_equip2" => sha
            && d.source.is_some_and(|s| {
                ctx.state.player(s).alive
                    && ctx
                        .state
                        .player(s)
                        .equips
                        .iter()
                        .any(|c| ctx.catalog.card_slot_of(ctx.state, *c) == Some(EquipSlot::Armor))
            }),
        "hslingjian_jinjilengdong_equip2" => {
            sha && d.source.is_some_and(|s| ctx.state.player(s).alive)
        }
        _ => false,
    }
}

fn is_lose_event(state: &GameStateCore, event: &EventData, owner: PlayerId) -> bool {
    matches!(
        event,
        EventData::CardsLost { player, cards, .. } if *player == owner && !cards.is_empty() && state.player(owner).alive
    ) || matches!(
        event,
        EventData::CardsDiscarded { player, cards } if *player == owner && !cards.is_empty() && state.player(owner).alive
    )
}

fn jiguanyaoshu_lost_equip_count(
    state: &GameStateCore,
    _catalog: &Catalog,
    owner: PlayerId,
    event: &EventData,
) -> usize {
    match event {
        EventData::CardsLost {
            player, origins, ..
        } if *player == owner && state.player(owner).alive => origins
            .iter()
            .filter(|o| matches!(o, LostOrigin::Equip))
            .count(),
        _ => 0,
    }
}

pub(super) fn step(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.behavior {
        HostBehavior::U05SwdLingjianDmgSrc => step_dmg_src(m, ctx),
        HostBehavior::U05SwdLingjianDmgTgt => step_dmg_tgt(m, ctx),
        HostBehavior::U05SwdLingjianLoseTemp => {
            let grant = match m.def_key.as_str() {
                "hslingjian_xuanfengzhiren_equip3" => "hslingjian_xuanfengzhiren_equip3_dist",
                "hslingjian_xuanfengzhiren_equip4" => "hslingjian_xuanfengzhiren_equip4_dist",
                _ => return Flow::Done,
            };
            if let Some(def) = ctx.catalog.skill_by_name(grant) {
                ctx.ensure_skill(
                    (m.owner, def),
                    crate::state::SkillInstance {
                        owner: m.owner,
                        enabled: true,
                        once_used: false,
                        expires_end_of_turn: true,
                        rounds: 0,
                    },
                );
            }
            Flow::Done
        }
        HostBehavior::U05SwdLingjianEquipped => {
            ctx.start_draw(m.owner, 1);
            Flow::Done
        }
        HostBehavior::U05SwdLingjianDrawPhase | HostBehavior::U05SwdLingjianShaBegin => Flow::Done,
        HostBehavior::U05SwdLingjianPhaseEnd => step_phase_end(m, ctx),
        HostBehavior::U05SwdLingjianPhaseUse => step_equip5_active(m, ctx),
        HostBehavior::U05SwdLingjianPassive | HostBehavior::U05SwdLingjianduanzao => Flow::Done,
        HostBehavior::U05SwdJiguanyaoshuSkill => step_jiguanyaoshu_skill(m, ctx),
        HostBehavior::U05SwdForgeAction => step_forge_action(m, ctx),
        _ => Flow::Done,
    }
}

fn step_dmg_src(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.def_key.as_str() {
        "hslingjian_xuanfengzhiren_equip1" => {
            if let Some(t) = m.target {
                random_discard_he(ctx, t);
            }
            Flow::Done
        }
        "hslingjian_xingtigaizao_equip1" => {
            ctx.start_draw(owner, 1);
            Flow::Done
        }
        "hslingjian_zhongxinghujia_equip1" => match m.stage {
            HostStage::Act => {
                m.stage = HostStage::SelectNature;
                wait_confirm(owner, "u05_zhongxing_equip1_confirm")
            }
            HostStage::SelectNature => {
                if matches!(ctx.answer().first(), Some(ChoiceOption::Confirm)) {
                    equip_random_armor_from_piles(ctx, owner);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "hslingjian_jinjilengdong_equip1" => match m.stage {
            HostStage::Act => {
                m.stage = HostStage::DrawSettle;
                wait_confirm(owner, "u05_jileng_equip1_confirm")
            }
            HostStage::DrawSettle => {
                if matches!(ctx.answer().first(), Some(ChoiceOption::Confirm)) {
                    if let Some(t) = m.target {
                        if ctx.core.player(t).alive {
                            // LIFO 栈：先压 TurnOverBefore，再压 draw(2)，确保摸 2 先结算后翻面
                            ctx.emit(EventData::TurnOverBefore {
                                player: t,
                                cancelled: false,
                            });
                            ctx.start_draw(t, 2);
                        }
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "hslingjian_yinmilichang_equip1" => match m.stage {
            HostStage::Act => {
                let opts: Vec<PlayerId> = ctx
                    .core
                    .alive_players()
                    .filter(|&p| p != owner && !ctx.core.player(p).yinmi_active)
                    .collect();
                if opts.is_empty() {
                    return Flow::Done;
                }
                m.stage = HostStage::TargetChoice;
                wait_select_players(owner, "u05_yinmi_equip1_target", opts, true)
            }
            HostStage::TargetChoice => {
                if let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() {
                    if ctx.core.player(t).alive {
                        ctx.core.player_mut(t).yinmi_active = true;
                        ctx.core
                            .vm_aux
                            .flags
                            .insert((t, "u05_yinmi_until_phase_begin".to_string()), 1);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        _ => Flow::Done,
    }
}

fn step_dmg_tgt(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.def_key.as_str() {
        "hslingjian_yinmilichang_equip2" => {
            ctx.core.player_mut(owner).yinmi_active = true;
            ctx.core
                .vm_aux
                .flags
                .insert((owner, "u05_yinmi_until_phase_after".to_string()), 1);
            Flow::Done
        }
        "hslingjian_xingtigaizao_equip2" => {
            ctx.start_draw(owner, 1);
            Flow::Done
        }
        "hslingjian_xuanfengzhiren_equip2" => {
            if let Some(EventData::Damage(d)) =
                ctx.res.frame(ctx.current_event_frame_id()).map(|f| &f.data)
            {
                if let Some(src) = d.source {
                    random_discard_he(ctx, src);
                }
            }
            Flow::Done
        }
        "hslingjian_zhongxinghujia_equip2" => match m.stage {
            HostStage::Act => {
                let src = match ctx.res.frame(ctx.current_event_frame_id()).map(|f| &f.data) {
                    Some(EventData::Damage(d)) => d.source,
                    _ => None,
                };
                m.target = src;
                if src.is_none() {
                    return Flow::Done;
                }
                m.stage = HostStage::SelectNature;
                wait_confirm(owner, "u05_zhongxing_equip2_confirm")
            }
            HostStage::SelectNature => {
                if matches!(ctx.answer().first(), Some(ChoiceOption::Confirm)) {
                    if let Some(src) = m.target {
                        let armor = ctx
                            .core
                            .player(src)
                            .equips
                            .iter()
                            .copied()
                            .find(|c| {
                                ctx.catalog.card_slot_of(ctx.core, *c) == Some(EquipSlot::Armor)
                            });
                        if let Some(a) = armor {
                            discard_he_card(ctx, src, a);
                        }
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "hslingjian_jinjilengdong_equip2" => match m.stage {
            HostStage::Act => {
                let src = match ctx.res.frame(ctx.current_event_frame_id()).map(|f| &f.data) {
                    Some(EventData::Damage(d)) => d.source,
                    _ => None,
                };
                m.target = src;
                if src.is_none() {
                    return Flow::Done;
                }
                m.stage = HostStage::DrawSettle;
                wait_confirm(owner, "u05_jileng_equip2_confirm")
            }
            HostStage::DrawSettle => {
                if matches!(ctx.answer().first(), Some(ChoiceOption::Confirm)) {
                    if let Some(src) = m.target {
                        if ctx.core.player(src).alive {
                            // LIFO 栈：先压 TurnOverBefore，再压 draw(2)，确保摸 2 先结算后翻面
                            ctx.emit(EventData::TurnOverBefore {
                                player: src,
                                cancelled: false,
                            });
                            ctx.start_draw(src, 2);
                        }
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        _ => Flow::Done,
    }
}

fn step_phase_end(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.def_key == "hslingjian_xingtigaizao" {
        ctx.core
            .vm_aux
            .flags
            .remove(&(owner, "u05_xingtigaizao_skill_n".to_string()));
        if let Some(def) = ctx.catalog.skill_by_name("hslingjian_xingtigaizao") {
            ctx.core.skills.remove(&(owner, def));
        }
        return Flow::Done;
    }
    if m.def_key == "hslingjian_jinjilengdong_equip5" {
        return match m.stage {
            HostStage::Act => {
                if ctx.core.player(owner).face_down {
                    return Flow::Done;
                }
                let opts: Vec<PlayerId> = ctx
                    .core
                    .alive_players()
                    .filter(|&p| p != owner && !ctx.core.player(p).face_down)
                    .collect();
                if opts.is_empty() {
                    return Flow::Done;
                }
                m.stage = HostStage::TargetChoice;
                wait_select_players(owner, "u05_jileng_equip5_target", opts, true)
            }
            HostStage::TargetChoice => {
                if let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() {
                    if ctx.core.player(t).alive && !ctx.core.player(t).face_down {
                        m.target = Some(t);
                        ctx.emit(EventData::TurnOverBefore {
                            player: t,
                            cancelled: false,
                        });
                        ctx.emit(EventData::TurnOverBefore {
                            player: owner,
                            cancelled: false,
                        });
                        m.stage = HostStage::DrawSettle;
                        return Flow::Yield;
                    }
                }
                Flow::Done
            }
            HostStage::DrawSettle => {
                if let Some(t) = m.target {
                    if ctx.core.player(t).alive {
                        ctx.start_draw(t, 2);
                    }
                }
                if ctx.core.player(owner).alive {
                    ctx.start_draw(owner, 2);
                }
                Flow::Done
            }
            _ => Flow::Done,
        };
    }
    Flow::Done
}

fn step_equip5_active(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let cost_n = if matches!(
        m.def_key.as_str(),
        "hslingjian_zhongxinghujia_equip5" | "hslingjian_shengxiuhaojiao_equip5"
    ) {
        2
    } else {
        1
    };
    let target_stage = match m.def_key.as_str() {
        "hslingjian_shijianhuisu_equip5" => HostStage::TargetChoice,
        "hslingjian_xuanfengzhiren_equip5" => HostStage::CompareTargetPick,
        _ => HostStage::ComparePlayerPick,
    };
    match m.stage {
        HostStage::Act => {
            let pool = hand_or_equip(ctx.core, owner);
            if pool.len() < cost_n {
                return Flow::Done;
            }
            m.stage = if cost_n == 2 {
                HostStage::CardsAsk
            } else {
                HostStage::SelectCard
            };
            wait_select_cards(owner, "u05_lingjian_e5_cost", pool, cost_n, cost_n, true)
        }
        HostStage::SelectCard | HostStage::CardsAsk => {
            let picked: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Card(c) => Some(*c),
                    _ => None,
                })
                .collect();
            if picked.len() < cost_n {
                refund_host_once_used(ctx, owner, &m.def_key);
                return Flow::Done;
            }
            let opts = equip5_targets(ctx.core, owner, m.def_key.as_str());
            if opts.is_empty() {
                refund_host_once_used(ctx, owner, &m.def_key);
                return Flow::Done;
            }
            m.cards = picked;
            m.stage = target_stage;
            wait_select_players(owner, "u05_lingjian_e5_target", opts, true)
        }
        HostStage::TargetChoice | HostStage::CompareTargetPick | HostStage::ComparePlayerPick => {
            let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                refund_host_once_used(ctx, owner, &m.def_key);
                return Flow::Done;
            };
            for c in std::mem::take(&mut m.cards) {
                discard_he_card(ctx, owner, c);
            }
            if !ctx.core.player(t).alive {
                return Flow::Done;
            }
            match m.def_key.as_str() {
                "hslingjian_xuanfengzhiren_equip5" => random_discard_he(ctx, t),
                "hslingjian_zhongxinghujia_equip5" => equip_random_armor_from_piles(ctx, t),
                "hslingjian_shengxiuhaojiao_equip5" => toggle_chaofeng(ctx, t),
                "hslingjian_shijianhuisu_equip5" => return_all_equips_to_hand(ctx, t),
                _ => {}
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn equip5_targets(core: &GameStateCore, owner: PlayerId, key: &str) -> Vec<PlayerId> {
    core.alive_players()
        .filter(|&p| match key {
            "hslingjian_xuanfengzhiren_equip5" => !hand_or_equip(core, p).is_empty(),
            "hslingjian_shijianhuisu_equip5" => {
                p != owner && !hand_or_equip(core, p).is_empty()
            }
            _ => true,
        })
        .collect()
}

pub(in crate::content::host) fn take_random_hslingjian_from_piles(
    ctx: &mut Ctx<'_>,
    receiver: PlayerId,
    count: usize,
) -> Vec<CardId> {
    let mut pool: Vec<CardId> = ctx
        .core
        .deck
        .iter()
        .chain(ctx.core.discard.iter())
        .copied()
        .filter(|x| ctx.catalog.card_kind(*x) == CardKind::Hslingjian)
        .collect();
    let n = pool.len();
    for i in (1..n).rev() {
        let j = ctx.core.rng.gen_range(i as u32 + 1) as usize;
        pool.swap(i, j);
    }
    let mut got = Vec::new();
    for x in pool.into_iter().take(count) {
        if let Some(pos) = ctx.core.deck.iter().position(|y| *y == x) {
            ctx.core.deck.remove(pos);
        } else if let Some(pos) = ctx.core.discard.iter().position(|y| *y == x) {
            ctx.core.discard.remove(pos);
        }
        ctx.core.player_mut(receiver).hand.push(x);
        got.push(x);
    }
    if !got.is_empty() {
        ctx.log(LogEvent::CardsDrawn {
            player: receiver,
            cards: got.clone(),
        });
    }
    got
}

fn step_jiguanyaoshu_skill(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let fid = ctx.top_frame_id();
    let count = match ctx.res.frame(fid).map(|f| &f.data) {
        Some(d) => jiguanyaoshu_lost_equip_count(ctx.core, ctx.catalog, m.owner, d),
        None => 0,
    };
    if count > 0 {
        let _ = take_random_hslingjian_from_piles(ctx, m.owner, count);
    }
    Flow::Done
}

fn step_forge_action(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            let eqs = forgeable_equips(ctx.core, ctx.catalog, owner);
            if eqs.is_empty() {
                return Flow::Done;
            }
            m.stage = HostStage::SelectCard;
            wait_select_cards(owner, "u05_duanzao_equip", eqs, 1, 1, true)
        }
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(eq)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            if !forgeable_equips(ctx.core, ctx.catalog, owner).contains(&eq) {
                return Flow::Done;
            }
            let mats: Vec<CardId> = ctx
                .core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|c| *c != eq && is_forge_material(ctx.catalog, *c))
                .collect();
            if mats.is_empty() {
                return Flow::Done;
            }
            m.card = Some(eq);
            m.stage = HostStage::CardsAsk;
            wait_select_cards(owner, "u05_duanzao_mat", mats, 1, 1, true)
        }
        HostStage::CardsAsk => {
            let Some(ChoiceOption::Card(mat)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            if !ctx.core.player(owner).hand.contains(&mat) {
                return Flow::Done;
            }
            m.cards = vec![mat];
            let opts: Vec<PlayerId> = ctx
                .core
                .alive_players()
                .filter(|&p| ctx.core.computed_distance(ctx.catalog, owner, p) <= 1)
                .collect();
            if opts.is_empty() {
                return Flow::Done;
            }
            m.stage = HostStage::TargetChoice;
            wait_select_players(owner, "u05_duanzao_target", opts, true)
        }
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            let Some(eq) = m.card.take() else {
                return Flow::Done;
            };
            let Some(mat) = m.cards.first().copied() else {
                return Flow::Done;
            };
            if !ctx.core.player(t).alive {
                return Flow::Done;
            }
            if ctx.core.player(owner).hand.contains(&mat) {
                ctx.discard_from_hand(owner, mat);
            }
            apply_forged_equip(ctx, owner, t, eq, mat);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(in crate::content::host) fn sync_forged_equip_skills(ctx: &mut Ctx<'_>, player: PlayerId) {
    let eqs = ctx.core.player(player).equips.clone();
    for eq in eqs {
        if let Some(sk) = crate::kernel::forged_lingjian_skill_name(ctx.core, ctx.catalog, player, eq) {
            if let Some(def) = ctx.catalog.skill_by_name(sk) {
                ctx.ensure_skill(
                    (player, def),
                    crate::state::SkillInstance {
                        owner: player,
                        enabled: true,
                        once_used: false,
                        expires_end_of_turn: false,
                        rounds: 0,
                    },
                );
            }
        }
    }
}

fn apply_forged_equip(
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    target: PlayerId,
    eq: CardId,
    mat: CardId,
) {
    let Some(slot) = ctx.catalog.card_slot_of(ctx.core, eq) else {
        return;
    };
    if ctx.core.player(owner).hand.contains(&eq) {
        ctx.take_hand_card(owner, eq);
    } else if let Some(pos) = ctx.core.player(owner).equips.iter().position(|x| *x == eq) {
        ctx.core.player_mut(owner).equips.remove(pos);
        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, owner, eq);
        ctx.note_cards_lost(owner, &[eq]);
        ctx.emit(EventData::CardsLost {
            player: owner,
            cards: vec![eq],
            origins: vec![LostOrigin::Equip],
        });
    } else {
        return;
    }
    let replaced = ctx
        .core
        .player(target)
        .equips
        .iter()
        .copied()
        .find(|c| ctx.catalog.card_slot_of(ctx.core, *c) == Some(slot));
    if let Some(old) = replaced {
        ctx.core.player_mut(target).equips.retain(|c| *c != old);
        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, target, old);
        ctx.note_cards_lost(target, &[old]);
        ctx.to_discard(old);
        ctx.log(LogEvent::CardDiscarded {
            player: target,
            cards: vec![old],
        });
    }
    ctx.core.player_mut(target).equips.push(eq);
    ctx.core.player_mut(target).card_mods.insert(eq, mat);
    crate::kernel::grant_equip_skills(ctx, target, eq);
    ctx.log(LogEvent::CardsGiven {
        from: owner,
        to: target,
        cards: vec![eq],
    });
}

fn find_armor_in_piles(core: &GameStateCore, catalog: &Catalog) -> Option<CardId> {
    core.deck
        .iter()
        .chain(core.discard.iter())
        .copied()
        .find(|x| catalog.card_slot_of(core, *x) == Some(EquipSlot::Armor))
}

fn equip_random_armor_from_piles(ctx: &mut Ctx<'_>, target: PlayerId) {
    let mut pool: Vec<CardId> = ctx
        .core
        .deck
        .iter()
        .chain(ctx.core.discard.iter())
        .copied()
        .filter(|x| ctx.catalog.card_slot_of(ctx.core, *x) == Some(EquipSlot::Armor))
        .collect();
    if pool.is_empty() {
        return;
    }
    let idx = ctx.core.rng.gen_range(pool.len() as u32) as usize;
    let armor = pool.swap_remove(idx);
    if let Some(pos) = ctx.core.deck.iter().position(|x| *x == armor) {
        ctx.core.deck.remove(pos);
    } else if let Some(pos) = ctx.core.discard.iter().position(|x| *x == armor) {
        ctx.core.discard.remove(pos);
    }
    ctx.core.player_mut(target).hand.push(armor);
    ctx.emit(EventData::Equipped {
        player: target,
        card: armor,
        replaced: None,
    });
}

fn random_discard_he(ctx: &mut Ctx<'_>, target: PlayerId) {
    let pool = hand_or_equip(ctx.core, target);
    if pool.is_empty() {
        return;
    }
    let idx = ctx.core.rng.gen_range(pool.len() as u32) as usize;
    discard_he_card(ctx, target, pool[idx]);
}

fn toggle_chaofeng(ctx: &mut Ctx<'_>, target: PlayerId) {
    let now = !ctx.core.player(target).chaofeng_active;
    ctx.core.player_mut(target).chaofeng_active = now;
    if let Some(def) = ctx.catalog.skill_by_name("hslingjian_chaofeng") {
        if now {
            ctx.ensure_skill(
                (target, def),
                crate::state::SkillInstance {
                    owner: target,
                    enabled: true,
                    once_used: false,
                    expires_end_of_turn: false,
                    rounds: 0,
                },
            );
        } else {
            ctx.core.skills.remove(&(target, def));
        }
    }
}

fn return_all_equips_to_hand(ctx: &mut Ctx<'_>, target: PlayerId) {
    let es = ctx.core.player(target).equips.clone();
    for c in es {
        if let Some(pos) = ctx.core.player(target).equips.iter().position(|x| *x == c) {
            ctx.core.player_mut(target).equips.remove(pos);
            crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, target, c);
            ctx.note_cards_lost(target, &[c]);
            ctx.emit(EventData::CardsLost {
                player: target,
                cards: vec![c],
                origins: vec![LostOrigin::Equip],
            });
            ctx.core.player_mut(target).hand.push(c);
        }
    }
}
