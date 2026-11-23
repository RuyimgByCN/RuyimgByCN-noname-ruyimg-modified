//! 【杀】的使用判定、目标合法性与额外【杀】问询候选模块。

use super::*;
use crate::catalog::Catalog;

/// 判定一张牌能否在主动出杀询问中作为真实【杀】使用（含转化技），返回 `Some((use_as, is_mutou))`。
pub fn sha_card_use_as(
    core: &crate::state::GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    card: crate::ids::CardId,
) -> Option<(Option<CardKind>, bool)> {
    let in_hand = core.player(owner).hand.contains(&card);
    if in_hand
        && (core.player(owner).fengyin_active
            || catalog.blocks_hand_cards(core, owner)
            || is_gw_dudayuanshuai2_locked(core, catalog, owner, card))
    {
        return None;
    }
    let kind = catalog.resolved_card_kind(core, card);
    if in_hand
        && (kind.is_sha_like() || catalog.cards[card as usize].name == "gw_wuyao")
        && !core.player(owner).wangshi_ban
    {
        return Some((
            (catalog.cards[card as usize].name == "gw_wuyao").then_some(CardKind::Sha),
            false,
        ));
    }
    let sealed = catalog.is_tieji_sealed(core, owner)
        || (core.player(owner).hp <= 1 && catalog.has_rechanyuan(core, owner));
    let suit = catalog.effective_card_suit(core, owner, card);
    let red = matches!(suit, crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond);
    let has_skill = |k: &str| catalog.skill_by_name(k).is_some_and(|d| core.has_skill(owner, d));
    if !sealed && in_hand {
        if core.player(owner).hp <= 2
            && has_skill(crate::vocabulary::host::WUDIAN_JINYAN_KEY)
            && is_black_trick_card(core, catalog, owner, card)
        {
            return Some((Some(CardKind::Sha), false));
        }
        if (has_skill(crate::vocabulary::names::WU_SHENG) && red)
            || (has_skill(crate::vocabulary::names::LONG_DAN) && kind == CardKind::Shan)
            || (has_skill("huanxia") && red)
            || (has_skill("yaotong") && core.player(owner).hand.len() % 2 == 1)
            || (has_ow_mujing(core, catalog, owner)
                && matches!(suit, crate::catalog::Suit::Spade | crate::catalog::Suit::Club))
        {
            return Some((Some(CardKind::Sha), false));
        }
        let muto = catalog
            .skill_by_name(crate::vocabulary::host::GJ_MUTOU_SKILL_KEY)
            .is_some_and(|d| {
                core.skills
                    .get(&(owner, d))
                    .is_some_and(|i| i.enabled && !i.once_used)
            });
        if muto {
            return Some((Some(CardKind::Sha), true));
        }
    }
    if (!in_hand || !sealed)
        && catalog
            .find_player_vm_view_as(core, owner, card, true, Some(CardKind::Sha))
            .is_some()
    {
        return Some((Some(CardKind::Sha), false));
    }
    if !in_hand
        && !sealed
        && core.player(owner).equips.contains(&card)
        && (has_skill("poxiao")
            || (has_skill("huanxia") && red)
            || (has_ow_mujing(core, catalog, owner)
                && matches!(
                    catalog.effective_card_suit(core, owner, card),
                    crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                )))
    {
        return Some((Some(CardKind::Sha), false));
    }
    None
}

/// 判定角色当前是否受全局/状态效果限制而不可使用【杀】。
pub fn sha_blocked_for(
    core: &crate::state::GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> bool {
    let has = |p: PlayerId, k: &str| catalog.skill_by_name(k).is_some_and(|d| core.has_skill(p, d));
    if catalog.blocks_card_use_and_save(core, owner)
        || has(owner, crate::vocabulary::host::WUDIAN_SHOUHU_KEY)
        || has(owner, crate::vocabulary::host::GW_BIRIN_STATE_KEY)
        || has(owner, "gw_birinongwu")
        || has(owner, crate::vocabulary::host::GJ_FOOD_SHIHUIFEN_BAN_KEY)
        || has(owner, "shihuifen")
        || catalog.skill_by_name("guangshu_club").is_some_and(|d| {
            catalog.skills[d as usize].behavior.as_host_behavior()
                == Some(crate::vocabulary::host::HostBehavior::U05OwBatchD)
                && core.has_skill(owner, d)
        })
    {
        return true;
    }
    core.turn.current == owner
        && has(owner, crate::vocabulary::names::BOSS_QINGZHU)
        && !has(owner, crate::vocabulary::names::BOSS_JIDING)
}

/// 判定指定来源与目标之间是否因阵营、潜行、嘲讽或封印规则禁止使用【杀】。
pub fn sha_user_or_target_blocked(
    core: &crate::state::GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    target: PlayerId,
) -> bool {
    if !core.player(owner).alive || !core.player(target).alive || owner == target {
        return true;
    }
    if sha_blocked_for(core, catalog, owner) {
        return true;
    }
    let has = |p: PlayerId, k: &str| catalog.skill_by_name(k).is_some_and(|d| core.has_skill(p, d));
    if has(owner, crate::vocabulary::names::BOSS_MINWAN)
        && !core.turn.minwan_targets.is_empty()
        && !core.turn.minwan_targets.contains(&target)
    {
        return true;
    }
    let flag = |p: PlayerId, k: &str| {
        core.vm_aux
            .flags
            .get(&(p, k.to_string()))
            .copied()
            .unwrap_or(0)
    };
    if (has(target, crate::vocabulary::names::KONG_CHENG) && core.player(target).hand.is_empty())
        || core.player(target).jidong_active
        || core.player(target).yinshen_active
        || core.player(target).jileng_active
        || core.player(target).yinmi_active
        || flag(target, "u05_mod:gj_qianxing") > 0
        || flag(target, "u05_mod:gj_lianjing_out_of_play") > 0
        || (core.player(target).face_down
            && (flag(target, "u05_mod:gj_xlqianhuan") > 0 || has(target, "xlqianhuan")))
        || catalog.skill_by_name("sheng2").is_some_and(|d| {
            catalog.skills[d as usize].behavior.as_host_behavior()
                == Some(crate::vocabulary::host::HostBehavior::U05OwBatchD)
                && core.has_skill(target, d)
        })
        || catalog.skill_by_name("yihun2").is_some_and(|d| {
            catalog.skills[d as usize].behavior.as_host_behavior()
                == Some(crate::vocabulary::host::HostBehavior::U05OwBatchC)
                && core.has_skill(target, d)
        })
        || has(target, "hslingjian_jinjilengdong")
        || has(owner, "hslingjian_jinjilengdong")
        || (has(target, "hslingjian_yinmilichang_equip5") && core.player(target).hand.is_empty())
        || (has(target, "hslingjian_shengxiuhaojiao_equip2")
            && (core.player(owner).chaofeng_active || has(owner, "hslingjian_chaofeng")))
    {
        return true;
    }
    let seats = core.players.len() as u8;
    let adj_chaofeng = (0..seats).find(|&p| {
        p != owner
            && core.player(p).alive
            && (core.player(p).chaofeng_active || has(p, "hslingjian_chaofeng"))
            && {
                let d = (u16::from(p) + u16::from(seats) - u16::from(owner)) % u16::from(seats);
                d == 1 || d == u16::from(seats) - 1
            }
    });
    !has(owner, "hslingjian_chaofeng") && adj_chaofeng.is_some_and(|h| target != h)
}

/// 询问使用真实【杀】（复仇 2 / 诱敌深入 / 诱战 共用；Step 与 Asks 同源）：忽略距离与出牌阶段次数限制。
pub fn usable_sha_options(
    core: &crate::state::GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    target: PlayerId,
) -> Vec<ChoiceOption> {
    if sha_user_or_target_blocked(core, catalog, owner, target) {
        return Vec::new();
    }
    let molicha_target = catalog
        .skill_by_name(crate::vocabulary::host::GJ_FOOD_MOLICHA_KEY)
        .is_some_and(|d| core.skills.contains_key(&(target, d)))
        || catalog
            .skill_by_name("molicha")
            .is_some_and(|d| core.skills.contains_key(&(target, d)));
    let hand_locked = catalog.blocks_hand_cards(core, owner);
    let mut candidates: Vec<crate::ids::CardId> = if hand_locked {
        Vec::new()
    } else {
        core.player(owner).hand.clone()
    };
    for c in catalog.player_vm_view_as_extra_cards(core, owner, true, Some(CardKind::Sha)) {
        if (!hand_locked || !core.player(owner).hand.contains(&c)) && !candidates.contains(&c) {
            candidates.push(c);
        }
    }
    let sealed = catalog.is_tieji_sealed(core, owner)
        || (core.player(owner).hp <= 1 && catalog.has_rechanyuan(core, owner));
    if !sealed
        && (catalog
            .skill_by_name("poxiao")
            .is_some_and(|d| core.has_skill(owner, d))
            || catalog
                .skill_by_name("huanxia")
                .is_some_and(|d| core.has_skill(owner, d))
            || has_ow_mujing(core, catalog, owner))
    {
        for &c in &core.player(owner).equips {
            if !candidates.contains(&c) {
                candidates.push(c);
            }
        }
    }
    candidates
        .into_iter()
        .filter(|&c| {
            let Some((use_as, _)) = sha_card_use_as(core, catalog, owner, c) else {
                return false;
            };
            let kind = use_as.unwrap_or_else(|| catalog.resolved_card_kind(core, c));
            let nature = match kind {
                CardKind::FireSha => crate::event::DamageNature::Fire,
                CardKind::LeiSha => crate::event::DamageNature::Thunder,
                _ => crate::event::DamageNature::Normal,
            };
            let black = matches!(
                catalog.resolved_card_suit(core, c),
                crate::catalog::Suit::Spade | crate::catalog::Suit::Club
            );
            !(molicha_target && black)
                && !crate::rvm_bridge::modifiers::eval_card_use_canceled(
                    core, catalog, owner, c, kind, nature,
                )
                && !crate::rvm_bridge::modifiers::eval_card_target_canceled(
                    core, catalog, owner, target, c, kind, nature,
                )
        })
        .map(ChoiceOption::Card)
        .collect()
}
