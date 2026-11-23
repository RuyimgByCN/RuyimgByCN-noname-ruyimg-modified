//! U05 杀海拾遗主动转化技（`yeying` / `guiying` / `fengze`）与实体置桌面共用辅助。

use super::super::*;
use crate::catalog::{CardKind, Catalog, Suit};
use crate::event::LostOrigin;
use crate::ids::{CardId, PlayerId};

/// 将手牌或装备区的一张实体牌移至处理区（`core.table`）等待结算/弃置，返回原区域来源。
///
/// 若原在装备区，同时撤销装备技能并清理 `declared_slots` / `card_mods`；手牌与装备均记录失牌历史。
pub(in crate::content::host) fn take_he_card_to_table(
    ctx: &mut Ctx<'_>,
    player: PlayerId,
    card: CardId,
) -> Option<LostOrigin> {
    if ctx.core.player(player).hand.contains(&card) {
        ctx.take_hand_card(player, card);
        ctx.core.table.push(card);
        ctx.note_cards_lost(player, &[card]);
        return Some(LostOrigin::Hand);
    }
    if ctx.core.player(player).equips.contains(&card) {
        ctx.core.player_mut(player).equips.retain(|x| *x != card);
        ctx.core.declared_slots.remove(&card);
        ctx.core.player_mut(player).card_mods.remove(&card);
        if ctx.catalog.card_kind(card).is_equip() {
            crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, player, card);
        }
        ctx.core.table.push(card);
        ctx.note_cards_lost(player, &[card]);
        return Some(LostOrigin::Equip);
    }
    None
}

/// 枚举 `owner` 手牌或装备区中有效花色为黑色的牌（`yeying`/`guiying`/`fengze`）。
pub(in crate::content::host) fn he_black_options(
    core: &crate::state::GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<ChoiceOption> {
    core.player(owner)
        .hand
        .iter()
        .chain(core.player(owner).equips.iter())
        .copied()
        .filter(|c| {
            matches!(
                catalog.effective_card_suit(core, owner, *c),
                Suit::Spade | Suit::Club
            )
        })
        .map(ChoiceOption::Card)
        .collect()
}

/// 夜影（乾坤镖）合法目标：其他存活且 `he` 非空。
pub(in crate::content::host) fn yeying_target_options(
    core: &crate::state::GameStateCore,
    owner: PlayerId,
) -> Vec<ChoiceOption> {
    core.alive_players()
        .filter(|p| {
            *p != owner && {
                let pl = core.player(*p);
                !pl.hand.is_empty() || !pl.equips.is_empty()
            }
        })
        .map(ChoiceOption::Player)
        .collect()
}

/// 归影（偷梁换柱）合法目标：其他存活且有手牌。
pub(in crate::content::host) fn guiying_target_options(
    core: &crate::state::GameStateCore,
    owner: PlayerId,
) -> Vec<ChoiceOption> {
    core.alive_players()
        .filter(|p| *p != owner && !core.player(*p).hand.is_empty())
        .map(ChoiceOption::Player)
        .collect()
}

/// 枚举 `owner` 手牌或装备区中有效花色为红色的牌（`huanxia`）。
pub(in crate::content::host) fn he_red_options(
    core: &crate::state::GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<ChoiceOption> {
    if catalog.blocks_all_cards(core, owner) {
        return Vec::new();
    }
    let hand_blocked =
        catalog.blocks_hand_cards(core, owner) || core.player(owner).fengyin_active;
    let hand_iter = if hand_blocked {
        [].as_slice().iter()
    } else {
        core.player(owner).hand.as_slice().iter()
    };
    hand_iter
        .chain(core.player(owner).equips.iter())
        .copied()
        .filter(|c| {
            matches!(
                catalog.effective_card_suit(core, owner, *c),
                Suit::Heart | Suit::Diamond
            )
        })
        .map(ChoiceOption::Card)
        .collect()
}

/// 枚举 `owner` 手牌中可用于妖童转杀的牌（`yaotong`）。
pub(in crate::content::host) fn hand_all_options(
    core: &crate::state::GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<ChoiceOption> {
    if catalog.blocks_all_cards(core, owner)
        || catalog.blocks_hand_cards(core, owner)
        || core.player(owner).fengyin_active
    {
        return Vec::new();
    }
    core.player(owner)
        .hand
        .iter()
        .copied()
        .map(ChoiceOption::Card)
        .collect()
}

/// 还侠 / 妖童转化为【杀】的合法目标列表。
pub(in crate::content::host) fn sha_target_options(
    core: &crate::state::GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<ChoiceOption> {
    core.legal_targets(catalog, CardKind::Sha, owner)
        .2
        .into_iter()
        .map(ChoiceOption::Player)
        .collect()
}

/// 激活入口：`yeying`（`usable:1`）、`guiying`、`fengze`（`usable:1`）、`huanxia`、`yaotong`。
pub(in crate::content::host) fn activate_u05_viewas(
    behavior: HostBehavior,
    def_key: &str,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<Box<dyn crate::catalog::SkillMachine>> {
    let can_use = match behavior {
        HostBehavior::SwdYeying => {
            !he_black_options(&ctx.state, ctx.catalog, owner).is_empty()
                && !yeying_target_options(&ctx.state, owner).is_empty()
        }
        HostBehavior::SwdGuiying => {
            !he_black_options(&ctx.state, ctx.catalog, owner).is_empty()
                && !guiying_target_options(&ctx.state, owner).is_empty()
        }
        HostBehavior::SwdFengze => {
            !he_black_options(&ctx.state, ctx.catalog, owner).is_empty()
                && ctx.state.alive_players().count() > 0
        }
        HostBehavior::SwdHuanxia => {
            super::skills_virtual_use::can_activate_sha_viewas(&ctx.state, ctx.catalog, owner)
                && !he_red_options(&ctx.state, ctx.catalog, owner).is_empty()
        }
        HostBehavior::SwdYaotong => {
            ctx.state.player(owner).hand.len() % 2 == 1
                && super::skills_virtual_use::can_activate_sha_viewas(&ctx.state, ctx.catalog, owner)
                && !hand_all_options(&ctx.state, ctx.catalog, owner).is_empty()
        }
        _ => return None,
    };
    if !can_use {
        return None;
    }
    Some(Box::new(HostMachine {
        def_key: def_key.to_string(),
        behavior,
        owner,
        target: None,
        card: None,
        cards: Vec::new(),
        targets: Vec::new(),
        stage: HostStage::Act,
    }))
}

fn finish_viewas_use(
    m: &HostMachine,
    ctx: &mut Ctx<'_>,
    card: CardId,
    as_kind: CardKind,
    targets: Vec<PlayerId>,
    refund_on_err: bool,
) -> Flow {
    let suit = ctx.catalog.effective_card_suit(ctx.core, m.owner, card);
    let card_ok = match m.behavior {
        HostBehavior::SwdHuanxia => {
            matches!(suit, Suit::Heart | Suit::Diamond)
                && super::skills_virtual_use::can_activate_sha_viewas(ctx.core, ctx.catalog, m.owner)
                && (!ctx.core.player(m.owner).hand.contains(&card)
                    || (!ctx.catalog.blocks_hand_cards(ctx.core, m.owner)
                        && !ctx.core.player(m.owner).fengyin_active))
        }
        HostBehavior::SwdYaotong => {
            ctx.core.player(m.owner).hand.len() % 2 == 1
                && ctx.core.player(m.owner).hand.contains(&card)
                && super::skills_virtual_use::can_activate_sha_viewas(ctx.core, ctx.catalog, m.owner)
                && !ctx.catalog.blocks_hand_cards(ctx.core, m.owner)
                && !ctx.core.player(m.owner).fengyin_active
        }
        _ => matches!(suit, Suit::Spade | Suit::Club),
    };
    if !card_ok {
        if refund_on_err {
            super::u05_7c_skills::refund_once_used(ctx, m.owner, &m.def_key);
        }
        return Flow::Done;
    }
    let Some(orig) = take_he_card_to_table(ctx, m.owner, card) else {
        if refund_on_err {
            super::u05_7c_skills::refund_once_used(ctx, m.owner, &m.def_key);
        }
        return Flow::Done;
    };
    if m.behavior == HostBehavior::SwdHuanxia {
        ctx.core.turn.huanxia_track.push((card, false));
    }
    ctx.emit(EventData::CardUsed(crate::event::CardUseData {
        card: Some(card),
        player: m.owner,
        targets,
        nature: crate::event::DamageNature::Normal,
        use_as: Some(as_kind),
        cancelled: false,
    }));
    // LIFO：CardsLost 后压先结算，匹配上游 useCard 前先离手/装备区语义
    ctx.emit(EventData::CardsLost {
        player: m.owner,
        cards: vec![card],
        origins: vec![orig],
    });
    Flow::Done
}

pub(super) fn step_u05_viewas(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let refund = matches!(
        m.behavior,
        HostBehavior::SwdYeying | HostBehavior::SwdFengze
    );
    let (pick_prompt, target_prompt, as_kind) = match m.behavior {
        HostBehavior::SwdYeying => ("yeying_pick", Some("yeying_target"), CardKind::Qiankunbiao),
        HostBehavior::SwdGuiying => ("guiying_pick", Some("guiying_target"), CardKind::Toulianghuanzhu),
        HostBehavior::SwdFengze => ("fengze_pick", None, CardKind::Taoyuan),
        HostBehavior::SwdHuanxia => ("huanxia_pick", Some("huanxia_target"), CardKind::Sha),
        HostBehavior::SwdYaotong => ("yaotong_pick", Some("yaotong_target"), CardKind::Sha),
        _ => return Flow::Done,
    };
    match m.stage {
        HostStage::Act => {
            let options = match m.behavior {
                HostBehavior::SwdHuanxia => he_red_options(ctx.core, ctx.catalog, m.owner),
                HostBehavior::SwdYaotong => hand_all_options(ctx.core, ctx.catalog, m.owner),
                _ => he_black_options(ctx.core, ctx.catalog, m.owner),
            };
            if options.is_empty() {
                if refund {
                    super::u05_7c_skills::refund_once_used(ctx, m.owner, &m.def_key);
                }
                return Flow::Done;
            }
            m.stage = HostStage::SelectCard;
            Flow::WaitChoice {
                observer: m.owner,
                spec: ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: pick_prompt,
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            }
        }
        HostStage::SelectCard => {
            let Some(c) = ctx.answer().iter().find_map(|o| match o {
                ChoiceOption::Card(c) => Some(*c),
                _ => None,
            }) else {
                if refund {
                    super::u05_7c_skills::refund_once_used(ctx, m.owner, &m.def_key);
                }
                return Flow::Done;
            };
            if target_prompt.is_none() {
                return finish_viewas_use(m, ctx, c, as_kind, Vec::new(), refund);
            }
            let options = match m.behavior {
                HostBehavior::SwdYeying => yeying_target_options(ctx.core, m.owner),
                HostBehavior::SwdGuiying => guiying_target_options(ctx.core, m.owner),
                HostBehavior::SwdHuanxia | HostBehavior::SwdYaotong => {
                    sha_target_options(ctx.core, ctx.catalog, m.owner)
                }
                _ => Vec::new(),
            };
            if options.is_empty() {
                if refund {
                    super::u05_7c_skills::refund_once_used(ctx, m.owner, &m.def_key);
                }
                return Flow::Done;
            }
            m.card = Some(c);
            m.stage = HostStage::TargetChoice;
            Flow::WaitChoice {
                observer: m.owner,
                spec: ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: target_prompt.expect("checked"),
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            }
        }
        HostStage::TargetChoice => {
            let Some(t) = ctx.answer().first().and_then(|o| match o {
                ChoiceOption::Player(p) => Some(*p),
                _ => None,
            }) else {
                if refund {
                    super::u05_7c_skills::refund_once_used(ctx, m.owner, &m.def_key);
                }
                return Flow::Done;
            };
            let Some(c) = m.card.take() else {
                if refund {
                    super::u05_7c_skills::refund_once_used(ctx, m.owner, &m.def_key);
                }
                return Flow::Done;
            };
            let target_ok = match m.behavior {
                HostBehavior::SwdHuanxia | HostBehavior::SwdYaotong => {
                    sha_target_options(ctx.core, ctx.catalog, m.owner)
                        .contains(&ChoiceOption::Player(t))
                }
                _ => ctx.core.player(t).alive && t != m.owner,
            };
            if !target_ok {
                if refund {
                    super::u05_7c_skills::refund_once_used(ctx, m.owner, &m.def_key);
                }
                return Flow::Done;
            }
            finish_viewas_use(m, ctx, c, as_kind, vec![t], refund)
        }
        _ => Flow::Done,
    }
}
