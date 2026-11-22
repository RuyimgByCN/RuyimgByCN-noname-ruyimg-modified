use super::*;
use crate::catalog::Catalog;

/// 杀的结算：询问目标闪，未闪则造成伤害子事件。
/// 裸衣伤害加成：来源本回合已发动裸衣（`once_used` 旗标，判定阶段
/// 开始时随 `reset_turn_limits` 清除）→ 杀/决斗伤害 +1。
pub(super) fn luoyi_bonus(ctx: &Ctx<'_>, source: Option<PlayerId>) -> u32 {
    let Some(s) = source else { return 0 };
    let Some(def) = ctx.catalog.skill_by_name(crate::vocabulary::names::LUO_YI) else {
        return 0;
    };
    match ctx.core.skills.get(&(s, def)) {
        Some(inst) if inst.enabled && inst.once_used => 1,
        _ => 0,
    }
}

/// 凶兽（Boss 模式共有技，规则 v208）：来源持凶兽且目标体力小于来源
/// 时，来源使用【杀】造成的伤害 +1（上游 boss_xiongshou_damage
/// `damageBegin1` filter hp< 同口径）。
pub(super) fn xiongshou_bonus(ctx: &Ctx<'_>, source: PlayerId, target: PlayerId) -> u32 {
    let Some(def) = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_XIONGSHOU)
    else {
        return 0;
    };
    let enabled = ctx
        .core
        .skills
        .get(&(source, def))
        .map(|i| i.enabled)
        .unwrap_or(false);
    if enabled && ctx.core.player(target).hp < ctx.core.player(source).hp {
        1
    } else {
        0
    }
}

/// 贪欲手牌「全场最多」判定：持有者手牌数不少于任何其他存活角色。
pub(crate) fn tanyu_is_max_hand(core: &crate::state::GameStateCore, holder: PlayerId) -> bool {
    let my = core.player(holder).hand.len();
    core.alive_players()
        .all(|o| o == holder || core.player(o).hand.len() <= my)
}

/// 覆泰2（Boss 穷奇，规则 v214）：场上存有持覆泰的穷奇、用/救桃者非
/// 穷奇且当前回合非穷奇时，桃不可用/不可救（上游 boss_futai2
/// cardSavable/cardEnabled 双 mod 同口径）。
pub fn tao_futai2_banned(
    core: &crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
    user: PlayerId,
) -> bool {
    let Some(fdef) = catalog.skill_by_name(crate::vocabulary::names::BOSS_FUTAI) else {
        return false;
    };
    let holders: Vec<PlayerId> = core
        .skills
        .iter()
        .filter(|((_, dd), i)| *dd == fdef && i.enabled)
        .map(|((o, _), _)| *o)
        .filter(|o| core.player(*o).alive && *o != user)
        .collect();
    match holders.first() {
        Some(&boss) => core.turn.current != boss,
        None => false,
    }
}

/// `L29` 放逐禁非锦囊牌旗标（`cardEnabled` + `cardSavable`，不含 `cardRespondable`）。
pub fn mbcmfangzhu_banned(
    core: &crate::state::GameStateCore,
    player: PlayerId,
) -> bool {
    core.vm_aux
        .flags
        .get(&(player, "mbcmfangzhu_ban".to_string()))
        .copied()
        .unwrap_or(0)
        > 0
}

/// `L29` 放逐白板旗标（`skillBlocker`：非持恒非夏洛特技全部失效）。
pub fn mbcmfangzhu_baiban(
    core: &crate::state::GameStateCore,
    player: PlayerId,
) -> bool {
    core.vm_aux
        .flags
        .get(&(player, "mbcmfangzhu_baiban".to_string()))
        .copied()
        .unwrap_or(0)
        > 0
}

/// L25 `configs:ssr_luoyi` (`祖安武将/extension.js:15031-15108`): 持有者所有普通/延时锦囊均视为【决斗】。
pub fn is_ssr_luoyi_trick_to_duel(
    core: &crate::state::GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    card: crate::ids::CardId,
) -> bool {
    let k = catalog.card_kind(card);
    !k.is_basic_kind()
        && k.equip_slot().is_none()
        && k.is_basic_trick_or_equip()
        && catalog
            .skill_by_name("ssr_luoyi")
            .is_some_and(|d| core.has_skill(player, d))
}

/// `L29` 酒诗（`mbcmjiushi_1` `chooseToUse`）：正面向上濒死本人可用虚拟【酒】自救。
pub(crate) fn mbcmjiushi_dying_skill(
    core: &crate::state::GameStateCore,
    catalog: &Catalog,
    responder: PlayerId,
    dying: PlayerId,
) -> Option<crate::ids::SkillDefId> {
    if responder != dying
        || !core.player(responder).alive
        || core.player(responder).face_down
        || mbcmfangzhu_banned(core, responder)
        || catalog.blocks_all_cards(core, responder)
    {
        return None;
    }
    let def = catalog.skill_by_name(crate::vocabulary::host::HLSS_MBCMJIUSHI_KEY)?;
    core.has_skill(responder, def).then_some(def)
}

/// P1/N08 急救（华佗）：持有者濒死结算时红色手牌可当【桃】使用。
pub(super) fn is_tao_like(
    ctx: &Ctx<'_>,
    responder: PlayerId,
    card: crate::ids::CardId,
    dying: PlayerId,
) -> bool {
    if mbcmfangzhu_banned(ctx.core, responder)
        || jinyan_blocks_black_trick(ctx.core, ctx.catalog, responder, card)
    {
        return false;
    }
    if crate::rvm_bridge::modifiers::eval_dying_peach_target_canceled(
        ctx.core,
        ctx.catalog,
        responder,
        dying,
        card,
    ) || crate::rvm_bridge::modifiers::eval_card_use_canceled(
        ctx.core,
        ctx.catalog,
        responder,
        card,
        ctx.catalog.card_kind(card),
        crate::event::DamageNature::Normal,
    ) {
        return false;
    }
    if ctx.catalog.card_kind(card) == CardKind::Tao {
        // 覆泰2（规则 v214）：全场禁桃时真桃不可救
        return !tao_futai2_banned(ctx.core, ctx.catalog, responder);
    }
    // 酒（G1 cardpile 前置）：濒死时对自身使用回复 1 点体力
    if ctx.catalog.card_kind(card) == CardKind::Jiu {
        return true;
    }
    // 护符（U05 swd 首切片，规则 v85）：仅濒死者本人的桃形救援
    //（上游 savable: dying === player）
    if ctx.catalog.card_kind(card) == CardKind::Hufu {
        return responder == dying;
    }
    // 天仙酒（U05 swd 第五批，规则 v89）：酒的 swd 变体——濒死自救
    if ctx.catalog.card_kind(card) == CardKind::Tianxianjiu {
        return true;
    }
    // 急救（v202 对齐上游 viewAsFilter）：限持有者回合外——自己是当前
    // 回合角色时不可发动
    let jijiu = !ctx.catalog.is_rechanyuan_sealed_name(
        ctx.core,
        responder,
        crate::vocabulary::names::JI_JIU,
    ) && ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::JI_JIU)
        .map(|d| ctx.core.has_skill(responder, d) && ctx.core.turn.current != responder)
        .unwrap_or(false);
    if jijiu
        && matches!(
            ctx.catalog.cards[card as usize].suit,
            crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
        )
    {
        return true;
    }
    // 复燃（Boss 焰灵，规则 v241）：焰灵濒死时其敌方可用红色牌当桃救
    //（上游 boss_furan2 dying chooseToUse viewAs tao 同口径）
    let furan = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_FURAN)
        .map(|d| ctx.core.has_skill(dying, d))
        .unwrap_or(false);
    if furan
        && ctx.core.is_boss_side(responder) != ctx.core.is_boss_side(dying)
        && matches!(
            ctx.catalog.cards[card as usize].suit,
            crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
        )
    {
        return true;
    }
    // 蜜链藕（G1 古剑食物）：持有者红桃手牌可当桃（濒死响应）
    let mizhilianou = ctx
        .catalog
        .skill_by_name(crate::vocabulary::host::GJ_FOOD_MIZHLIANOU_KEY)
        .map(|d| ctx.core.has_skill(responder, d))
        .unwrap_or(false);
    if mizhilianou && ctx.catalog.cards[card as usize].suit == crate::catalog::Suit::Heart {
        return true;
    }
    if ctx
        .catalog
        .find_player_vm_view_as(ctx.core, responder, card, false, Some(CardKind::Tao))
        .is_some()
        && !tao_futai2_banned(ctx.core, ctx.catalog, responder)
    {
        return true;
    }
    if responder == dying
        && ctx
            .catalog
            .find_player_vm_view_as(ctx.core, responder, card, false, Some(CardKind::Jiu))
            .is_some()
    {
        return true;
    }
    // 帝女玄霜（G1 古剑，濒死可救基本牌）：按桃结算回复 1 体力
    //（弃摸骑乘由 DyingSaveMachine rider 承担）
    ctx.catalog.card_kind(card) == CardKind::Dinvxuanshuang
}

pub(super) fn dying_tao_cards(
    ctx: &Ctx<'_>,
    responder: PlayerId,
    dying: PlayerId,
) -> Vec<crate::ids::CardId> {
    if ctx.catalog.blocks_all_cards(ctx.core, responder) {
        return Vec::new();
    }
    let hand_locked = crate::rvm_bridge::modifiers::d34_flags::is_u01_hand_locked(
        ctx.core,
        ctx.catalog,
        responder,
    );
    let p = ctx.core.player(responder);
    let mut out: Vec<crate::ids::CardId> = if hand_locked {
        Vec::new()
    } else {
        p.hand
            .iter()
            .copied()
            .filter(|&c| is_tao_like(ctx, responder, c, dying))
            .collect()
    };
    for c in ctx
        .catalog
        .player_vm_view_as_extra_cards(ctx.core, responder, false, None)
    {
        if (!hand_locked || !p.hand.contains(&c))
            && is_tao_like(ctx, responder, c, dying)
            && !out.contains(&c)
        {
            out.push(c);
        }
    }
    out.retain(|&c| {
        !crate::rvm_bridge::modifiers::d34_flags::is_ducai3_forbidden(
            ctx.core,
            ctx.catalog,
            responder,
            c,
        )
    });
    out
}

/// 通用响应/打出候选项（kernel 与 snap/asks 共用，含禁言 `jinyan` 禁用与黑锦囊转【杀】）。
pub fn response_kind_cards_core(
    core: &crate::state::GameStateCore,
    catalog: &Catalog,
    responder: PlayerId,
    need: CardKind,
) -> Vec<crate::ids::CardId> {
    if catalog.blocks_all_cards(core, responder) {
        return Vec::new();
    }
    let hand_locked =
        crate::rvm_bridge::modifiers::d34_flags::is_u01_hand_locked(core, catalog, responder);
    let sealed = catalog.is_tieji_sealed(core, responder)
        || (core.player(responder).hp <= 1 && catalog.has_rechanyuan(core, responder));
    let rw_lanyin = !hand_locked
        && need == CardKind::Shan
        && crate::rvm_bridge::modifiers::d34_l28_mods::has_rewritten_lanyinjia(
            core, catalog, responder,
        );
    let p = core.player(responder);
    let mut out: Vec<crate::ids::CardId> = if hand_locked {
        Vec::new()
    } else {
        p.hand
            .iter()
            .copied()
            .filter(|&c| {
                if need == CardKind::Nullify
                    && is_ssr_luoyi_trick_to_duel(core, catalog, responder, c)
                {
                    return false;
                }
                if jinyan_blocks_black_trick(core, catalog, responder, c) {
                    return need == CardKind::Sha && !sealed;
                }
                rw_lanyin
                    || catalog.card_kind(c) == need
                    || catalog
                        .find_player_vm_view_as(core, responder, c, false, Some(need))
                        .is_some()
            })
            .collect()
    };
    for c in catalog.player_vm_view_as_extra_cards(core, responder, false, Some(need)) {
        if (!hand_locked || !p.hand.contains(&c))
            && !jinyan_blocks_black_trick(core, catalog, responder, c)
            && !out.contains(&c)
        {
            out.push(c);
        }
    }
    out.retain(|&c| {
        !crate::rvm_bridge::modifiers::d34_flags::is_ducai3_forbidden(
            core,
            catalog,
            responder,
            c,
        )
    });
    out
}

pub(super) fn has_u01_respond_park(ctx: &Ctx<'_>) -> bool {
    let Some(fid) = ctx.res.stack.last().map(|f| f.id) else {
        return false;
    };
    let depth = ctx.res.stack.len();
    ctx.res.parks.iter().any(|p| {
        p.slot == crate::resolution::Slot::Trigger { frame: fid }
            && p.resume_depth == depth
            && p.machine.trigger_order().is_none()
    })
}

pub(super) fn take_u01_respond_result(ctx: &mut Ctx<'_>, player: PlayerId) -> Option<bool> {
    let fid = ctx.res.stack.last().map(|f| f.id)?;
    ctx.core
        .vm_aux
        .flags
        .remove(&(player, format!("u01_respond_result:{fid}")))
        .map(|v| v > 0)
}

pub(super) fn response_kind_cards(
    ctx: &Ctx<'_>,
    responder: PlayerId,
    need: CardKind,
) -> Vec<crate::ids::CardId> {
    response_kind_cards_core(ctx.core, ctx.catalog, responder, need)
}

pub fn discard_response_card(
    ctx: &mut Ctx<'_>,
    player: PlayerId,
    card: crate::ids::CardId,
    target_kind: CardKind,
) {
    let in_hand = ctx.core.player(player).hand.contains(&card);
    let mut from_none = false;
    if ctx.catalog.card_kind(card) != target_kind || !in_hand {
        if let Some(spec) = ctx
            .catalog
            .find_player_vm_view_as(ctx.core, player, card, false, Some(target_kind))
            .cloned()
        {
            let extra = spec.consume_and_record(ctx.core, ctx.catalog, player, card);
            if spec.cost == Some(crate::rvm_bridge::VmViewAsCost::TurnOver) {
                ctx.emit(EventData::TurnOverBefore {
                    player,
                    cancelled: false,
                });
            }
            for c2 in extra {
                ctx.to_discard(c2);
            }
            if spec.zone == crate::rvm_bridge::VmViewAsZone::None {
                from_none = true;
            }
        }
    } else {
        ctx.core
            .vm_aux
            .flags
            .remove(&(player, "event_view_as_skill".to_string()));
    }
    if from_none {
        return;
    }
    if ctx.core.player(player).hand.contains(&card) {
        ctx.discard_from_hand(player, card);
    } else if ctx.core.player(player).equips.contains(&card) {
        ctx.core.player_mut(player).equips.retain(|x| *x != card);
        ctx.core.declared_slots.remove(&card);
        revoke_equip_skills(ctx.core, ctx.catalog, player, card);
        ctx.to_discard(card);
        ctx.log(LogEvent::CardDiscarded {
            player,
            cards: vec![card],
        });
        ctx.emit(EventData::CardsLost {
            player,
            cards: vec![card],
            origins: vec![crate::event::LostOrigin::Equip],
        });
    } else if ctx
        .core
        .vm_aux
        .piles
        .iter_mut()
        .filter(|((owner, _), _)| *owner == player)
        .any(|(_, pile)| {
            let len_before = pile.len();
            pile.retain(|&x| x != card);
            pile.len() != len_before
        })
    {
        ctx.to_discard(card);
        ctx.log(LogEvent::CardDiscarded {
            player,
            cards: vec![card],
        });
    } else {
        ctx.core.table.retain(|x| *x != card);
        ctx.to_discard(card);
    }
}

/// 黑锦囊判定（禁言/反击【杀】可用性检查共用）。
pub fn is_black_trick_card(
    core: &crate::state::GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    card: crate::ids::CardId,
) -> bool {
    let kind = catalog.card_kind(card);
    !kind.is_basic_kind()
        && kind.equip_slot().is_none()
        && kind.is_basic_trick_or_equip()
        && matches!(
            catalog.effective_card_suit(core, owner, card),
            crate::catalog::Suit::Spade | crate::catalog::Suit::Club
        )
}

/// 玩点论杀 `jinyan`（character/skill.js:448-507）：体力 <= 2 时禁用黑色锦囊（含延时锦囊；非锁定技，被铁骑封印时失效）。
pub fn jinyan_blocks_black_trick(
    core: &crate::state::GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    card: crate::ids::CardId,
) -> bool {
    core.player(player).hp <= 2
        && !catalog.is_tieji_sealed(core, player)
        && !catalog.is_rechanyuan_sealed_name(
            core,
            player,
            crate::vocabulary::host::WUDIAN_JINYAN_KEY,
        )
        && catalog
            .skill_by_name(crate::vocabulary::host::WUDIAN_JINYAN_KEY)
            .is_some_and(|d| core.has_skill(player, d))
        && is_black_trick_card(core, catalog, player, card)
}

/// 判定一张牌能否在主动出杀询问中作为真实【杀】使用（含转化技），返回 `Some((use_as, is_mutou))`。
pub fn sha_card_use_as(
    core: &crate::state::GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    card: crate::ids::CardId,
) -> Option<(Option<CardKind>, bool)> {
    let in_hand = core.player(owner).hand.contains(&card);
    if in_hand && core.player(owner).fengyin_active {
        return None;
    }
    let kind = catalog.card_kind(card);
    if in_hand && kind.is_sha_like() && !core.player(owner).wangshi_ban {
        return Some((None, false));
    }
    let sealed = catalog.is_tieji_sealed(core, owner)
        || (core.player(owner).hp <= 1 && catalog.has_rechanyuan(core, owner));
    if !sealed && in_hand {
        let suit = catalog.effective_card_suit(core, owner, card);
        let red = matches!(suit, crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond);
        let has_skill = |k: &str| catalog.skill_by_name(k).is_some_and(|d| core.has_skill(owner, d));
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
        && catalog
            .skill_by_name("poxiao")
            .is_some_and(|d| core.has_skill(owner, d))
    {
        return Some((Some(CardKind::Sha), false));
    }
    None
}

fn sha_user_or_target_blocked(
    core: &crate::state::GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    target: PlayerId,
) -> bool {
    if !core.player(owner).alive || !core.player(target).alive || owner == target {
        return true;
    }
    let has = |p: PlayerId, k: &str| catalog.skill_by_name(k).is_some_and(|d| core.has_skill(p, d));
    if catalog.blocks_all_cards(core, owner)
        || has(owner, crate::vocabulary::host::WUDIAN_SHOUHU_KEY)
        || has(owner, crate::vocabulary::host::GW_BIRIN_STATE_KEY)
        || has(owner, crate::vocabulary::host::GJ_FOOD_SHIHUIFEN_BAN_KEY)
    {
        return true;
    }
    if core.turn.current == owner
        && has(owner, crate::vocabulary::names::BOSS_QINGZHU)
        && !has(owner, crate::vocabulary::names::BOSS_JIDING)
    {
        return true;
    }
    if has(owner, crate::vocabulary::names::BOSS_MINWAN)
        && !core.turn.minwan_targets.is_empty()
        && !core.turn.minwan_targets.contains(&target)
    {
        return true;
    }
    if (has(target, crate::vocabulary::names::KONG_CHENG) && core.player(target).hand.is_empty())
        || core.player(target).jidong_active
        || core.player(target).yinshen_active
        || core.player(target).jileng_active
        || core.player(target).yinmi_active
    {
        return true;
    }
    let seats = core.players.len() as u8;
    let adj_chaofeng = (0..seats).find(|&p| {
        p != owner && core.player(p).alive && core.player(p).chaofeng_active && {
            let d = (u16::from(p) + u16::from(seats) - u16::from(owner)) % u16::from(seats);
            d == 1 || d == u16::from(seats) - 1
        }
    });
    adj_chaofeng.is_some_and(|h| target != h)
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
        .is_some_and(|d| core.skills.contains_key(&(target, d)));
    let mut candidates: Vec<crate::ids::CardId> = core.player(owner).hand.clone();
    for c in catalog.player_vm_view_as_extra_cards(core, owner, true, Some(CardKind::Sha)) {
        if !candidates.contains(&c) {
            candidates.push(c);
        }
    }
    if catalog
        .skill_by_name("poxiao")
        .is_some_and(|d| core.has_skill(owner, d))
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
            let kind = use_as.unwrap_or_else(|| catalog.card_kind(c));
            let nature = match kind {
                CardKind::FireSha => crate::event::DamageNature::Fire,
                CardKind::LeiSha => crate::event::DamageNature::Thunder,
                _ => crate::event::DamageNature::Normal,
            };
            let black = matches!(
                catalog.cards[c as usize].suit,
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

/// 响应窗（1=闪, 2=杀, 3=无懈, 4=濒死桃, 5=自救桃/酒）可用宿主转化技能列表。
pub(crate) fn host_response_skills(
    core: &crate::state::GameStateCore,
    catalog: &Catalog,
    responder: PlayerId,
    resp_ctx: u8,
) -> Vec<crate::ids::SkillDefId> {
    let mut out = Vec::new();
    if catalog.blocks_all_cards(core, responder) {
        return out;
    }
    if let Some(def) = (catalog.hooks.reguhuo_response_skill)(core, catalog, responder, resp_ctx) {
        out.push(def);
    }
    if let Some(def) = (catalog.hooks.scsanruo_response_skill)(core, catalog, responder, resp_ctx) {
        if !out.contains(&def) {
            out.push(def);
        }
    }
    out
}
