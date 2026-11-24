//! Phase 3 P3-C1a `L11` 跨包 NPC Part A 引擎生命周期钩子（`lifecycle.rs`）。
//!
//! 承载开局、装备变更、回合开始/结束、出牌/弃牌/摸牌/造成伤害后与【杀】结算等生命周期钩子。

use super::helpers::*;
use super::l11_common::*;
use super::*;
use crate::catalog::{CardKind, Catalog};
use crate::event::{EventData, Phase};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, LogEvent};
use crate::state::GameStateCore;

#[inline]
pub(super) fn catalog_has_npc_a(catalog: &Catalog) -> bool {
    catalog
        .skill_by_behavior(HostBehavior::L11CrossNpcA)
        .is_some()
}

#[inline]
fn any_alive_has_npc_a(core: &GameStateCore, catalog: &Catalog, name: &str) -> bool {
    core.alive_players()
        .any(|p| has_npc_a_skill(core, catalog, p, name))
}

#[inline]
fn has_discard_batch_npc_a(core: &GameStateCore, catalog: &Catalog, p: PlayerId) -> bool {
    has_npc_a_skill(core, catalog, p, "lirang")
        || has_npc_a_skill(core, catalog, p, "relianying")
        || has_npc_a_skill(core, catalog, p, "retuntian")
        || has_npc_a_skill(core, catalog, p, "rexuanfeng")
}

/// `gameDrawEnd` 钩子：初始化所有 `reqizhou` 装备花色联动。
pub(in crate::content::host) fn on_game_start_l11_npc_a(
    core: &mut GameStateCore,
    catalog: &Catalog,
) {
    if !catalog_has_npc_a(catalog) {
        return;
    }
    let seats = core.players.len() as PlayerId;
    for seat in 0..seats {
        sync_reqizhou_skills(core, catalog, seat);
    }
}

/// `Equipped` / `EquipRevoked` 钩子。
pub(in crate::content::host) fn on_equip_changed_l11_npc_a(
    core: &mut GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    card: CardId,
    revoked: bool,
) {
    if !catalog_has_npc_a(catalog) {
        return;
    }
    core.vm_aux
        .flags
        .remove(&(player, format!("l11_discard_batched:{card}")));
    if revoked
        && core.turn.phase == Phase::Play
        && player != core.turn.current
        && has_npc_a_skill(core, catalog, player, "diancai")
    {
        inc_flag(core, player, "l11_diancai_play_lost", 1);
    }
    sync_reqizhou_skills(core, catalog, player);
}

/// 回合开始钩子：恢复本回合 `mingjian` 加成并清除上回合延续的 `reluoyi` / 伤害状态。
pub(in crate::content::host) fn on_turn_begin_l11_npc_a(ctx: &mut Ctx<'_>, player: PlayerId) {
    if !catalog_has_npc_a(ctx.catalog) {
        return;
    }
    set_flag(ctx.core, player, "l11_mod:reluoyi_dmg_plus1", 0);
    let mj = flag(ctx.core, player, "l11_mingjian_pending");
    if mj > 0 {
        set_flag(ctx.core, player, "l11_mingjian_pending", 0);
        set_flag(ctx.core, player, "l11_mod:mingjian_buff", mj);
    }
    let seats = ctx.core.players.len() as PlayerId;
    for p in 0..seats {
        set_flag(ctx.core, p, "l11_lost_card_turn", 0);
        set_flag(ctx.core, p, "l11_rezaiqi_red_discard_turn", 0);
        set_flag(ctx.core, p, "l11_kuizhu_discard_cnt", 0);
        set_flag(ctx.core, p, "l11_xuanfeng_discard_cnt", 0);
        set_flag(ctx.core, p, "l11_diancai_play_lost", 0);
        set_flag(ctx.core, p, "l11_source_dmg_turn", 0);
        set_flag(ctx.core, p, "l11_shicai_seen_turn", 0);
        set_flag(ctx.core, p, "l11_reluanji_suits_turn", 0);
        set_flag(ctx.core, p, "l11_mozhi_hist_len", 0);
        set_flag(ctx.core, p, "l11_olniepan_saving", 0);
        set_flag(ctx.core, p, "l11_lihun_used_play", 0);
        set_flag(ctx.core, p, "l11_refenxun_used_play", 0);
    }
}

/// `phaseUseEnd` 钩子：清理出牌阶段内限时修饰与暂存计数。
pub(in crate::content::host) fn on_phase_use_end_l11_npc_a(
    ctx: &mut Ctx<'_>,
    player: PlayerId,
) {
    if !catalog_has_npc_a(ctx.catalog) {
        return;
    }
    set_flag(ctx.core, player, "l11_mod:dcnutao_sha_plus", 0);
    set_flag(ctx.core, player, "l11_new_reqingnang_ban", 0);
    let keys: Vec<(PlayerId, String)> = ctx
        .core
        .vm_aux
        .flags
        .keys()
        .filter(|(_, k)| k == "l11_new_reqingnang_t" || k.starts_with("l11_new_reqingnang_t:"))
        .cloned()
        .collect();
    for k in keys {
        ctx.core.vm_aux.flags.remove(&k);
    }
}

/// `DamageEnd` 钩子：累计本回合伤害与 `reluanji` 乱击次数。
pub(in crate::content::host) fn on_damage_end_l11_npc_a(
    ctx: &mut Ctx<'_>,
    d: &crate::event::DamageData,
) {
    if !catalog_has_npc_a(ctx.catalog) || d.prevented || d.amount <= 0 {
        return;
    }
    let amt = d.amount as u32;
    if let Some(src) = d.source {
        if any_alive_has_npc_a(ctx.core, ctx.catalog, "nsxianhai")
            || any_alive_has_npc_a(ctx.core, ctx.catalog, "jigong")
        {
            inc_flag(ctx.core, src, "l11_source_dmg_turn", amt);
        }
        if flag(ctx.core, src, &format!("l11_mod:fenxun_to:{}", d.target)) > 0 {
            set_flag(ctx.core, src, &format!("l11_refenxun_dealt:{}", d.target), 1);
        }
    }
    let lj_fids: Vec<(PlayerId, crate::ids::EventId)> = ctx
        .res
        .stack
        .iter()
        .filter_map(|f| match &f.data {
            EventData::CardUsed(u)
                if u.effective_kind(ctx.catalog) == CardKind::ArcheryAttack
                    && flag(ctx.core, u.player, &format!("l11_reluanji_fid:{}", f.id)) > 0 =>
            {
                Some((u.player, f.id))
            }
            _ => None,
        })
        .collect();
    for (p, fid) in lj_fids {
        inc_flag(ctx.core, p, &format!("l11_reluanji_dmg:{fid}"), amt);
    }
}

/// `phaseAfter` 整回合结束钩子：归还 `repojun` 扣置牌、收回无主 `jinglve` 死士牌、清理单回合修饰。
pub(in crate::content::host) fn on_turn_phase_after_l11_npc_a(
    ctx: &mut Ctx<'_>,
    player: PlayerId,
) {
    if !catalog_has_npc_a(ctx.catalog) {
        return;
    }
    // 1. `nsxianhai`: 记录本回合造成伤害作下次他人比较基准，并恢复觉醒过的 `nsxingchu`（如已记录）
    let cur_dmg = flag(ctx.core, player, "l11_source_dmg_turn");
    if cur_dmg > 0 || any_alive_has_npc_a(ctx.core, ctx.catalog, "nsxianhai") {
        set_flag(ctx.core, player, "l11_last_turn_dmg", cur_dmg);
    }
    set_flag(ctx.core, player, "l11_source_dmg_turn", 0);
    set_flag(ctx.core, player, "l11_mod:jigong_active", 0);
    set_flag(ctx.core, player, "l11_jigong_turn_dmg", 0);
    set_flag(ctx.core, player, "l11_mod:mingjian_buff", 0);
    set_flag(ctx.core, player, "l11_nzry_chenglve_suits", 0);
    set_flag(ctx.core, player, "l11_mod:reshuangxiong_duel", 0);
    set_flag(ctx.core, player, "shuangxiong_color", 0);
    let seats = ctx.core.players.len() as PlayerId;
    for seat in 0..seats {
        set_flag(ctx.core, seat, "l11_new_yijue_hand_ban", 0);
        if flag(ctx.core, seat, "l11_yijue_set_xj") > 0 {
            ctx.core.vm_aux.flags.remove(&(seat, "l11_yijue_set_xj".to_string()));
            ctx.core.vm_aux.flags.remove(&(seat, "xj_fengyin_sealed".to_string()));
        }
    }
    ctx.core
        .vm_aux
        .flags
        .retain(|(seat, k), _| {
            if *seat == player
                && (k.starts_with("l11_mod:fenxun_to:")
                    || k.starts_with("l11_refenxun_dealt:")
                    || k.starts_with("l11_yijue_heart_target:"))
            {
                return false;
            }
            if *seat == player && k == "l11_mod:fenxun_dist1" {
                return false;
            }
            true
        });

    // 2. `repojun`: 任意角色回合结束时，归还场上全部 `repojun:{t}` 扣置牌（`refresh.js:9743`）
    let seats = ctx.core.players.len() as PlayerId;
    for owner in 0..seats {
        for t in 0..seats {
            if let Some(cards) = ctx
                .core
                .vm_aux
                .piles
                .remove(&(owner, format!("l11_repojun:{t}")))
            {
                if ctx.core.player(t).alive && !cards.is_empty() {
                    let delivered =
                        (ctx.catalog.hooks.on_before_gain_hand)(ctx, None, t, &cards);
                    ctx.core
                        .player_mut(t)
                        .hand
                        .extend(delivered.iter().copied());
                } else {
                    for c in cards {
                        ctx.to_discard(c);
                    }
                }
            }
        }
        // 3. `jinglve`: 目标回合结束时若仍持 `sishi` 牌，由经略持有者获得之
        if let Some(sishi_cid) = ctx
            .core
            .vm_aux
            .flags
            .get(&(player, format!("l11_jinglve_sishi_from:{owner}")))
            .copied()
            .and_then(|v| v.checked_sub(1))
        {
            ctx.core
                .vm_aux
                .flags
                .remove(&(player, format!("l11_jinglve_sishi_from:{owner}")));
            ctx.core
                .vm_aux
                .flags
                .remove(&(player, format!("l11_jinglve_sishi_card:{sishi_cid}")));
            ctx.core
                .vm_aux
                .flags
                .remove(&(owner, "l11_jinglve_active_target".to_string()));
            if ctx.core.player(owner).alive {
                if ctx.core.player(player).hand.contains(&sishi_cid)
                    || ctx.core.player(player).equips.contains(&sishi_cid)
                    || ctx.core.player(player).judge.contains(&sishi_cid)
                {
                    move_cards_to_hand(ctx, player, owner, &[sishi_cid]);
                } else if take_from_table_discard_or_equip(ctx, sishi_cid) {
                    ctx.core.player_mut(owner).hand.push(sishi_cid);
                }
            }
        }
    }
}

pub(in crate::content::host) fn on_phase_end_l11_npc_a(ctx: &mut Ctx<'_>) {
    if !catalog_has_npc_a(ctx.catalog) {
        return;
    }
    let seats = ctx.core.players.len() as PlayerId;
    for seat in 0..seats {
        set_flag(ctx.core, seat, "l11_diancai_play_lost", 0);
    }
}

pub(in crate::content::host) fn on_hand_card_removed_l11_npc_a(
    ctx: &mut Ctx<'_>,
    player: PlayerId,
    card: CardId,
) {
    if !catalog_has_npc_a(ctx.catalog) {
        return;
    }
    ctx.core
        .vm_aux
        .flags
        .remove(&(player, format!("l11_discard_batched:{card}")));
    if any_alive_has_npc_a(ctx.core, ctx.catalog, "rejuece") {
        set_flag(ctx.core, player, "l11_lost_card_turn", 1);
    }
    if has_discard_batch_npc_a(ctx.core, ctx.catalog, player) {
        inc_flag(ctx.core, player, "l11_batch_lost_hand", 1);
    }
    if ctx.core.turn.phase == Phase::Play
        && player != ctx.core.turn.current
        && has_npc_a_skill(ctx.core, ctx.catalog, player, "diancai")
    {
        inc_flag(ctx.core, player, "l11_diancai_play_lost", 1);
    }
    if ctx.core.turn.phase == Phase::Discard
        && player == ctx.core.turn.current
        && has_npc_a_skill(ctx.core, ctx.catalog, player, "rexuanfeng")
    {
        inc_flag(ctx.core, player, "l11_xuanfeng_discard_cnt", 1);
    }
}

pub(in crate::content::host) fn on_cards_to_discard_l11_npc_a(
    ctx: &mut Ctx<'_>,
    actor: PlayerId,
    cards: &[CardId],
) {
    if !catalog_has_npc_a(ctx.catalog) || cards.is_empty() {
        return;
    }
    if has_discard_batch_npc_a(ctx.core, ctx.catalog, actor) {
        let batch_fid = ctx.res.stack.last().map_or(1, |f| f.id);
        for &c in cards {
            if flag(ctx.core, actor, &format!("l11_discard_batched:{c}")) == 0 {
                set_flag(ctx.core, actor, &format!("l11_discard_batched:{c}"), batch_fid);
            }
        }
    }
    let cur = ctx.core.turn.current;
    if ctx.core.turn.phase == Phase::Discard
        && actor == cur
        && has_npc_a_skill(ctx.core, ctx.catalog, actor, "nzry_kuizhu")
    {
        inc_flag(ctx.core, actor, "l11_kuizhu_discard_cnt", cards.len() as u32);
    }
    if any_alive_has_npc_a(ctx.core, ctx.catalog, "rezaiqi") {
        let red_cnt = cards
            .iter()
            .filter(|&&c| is_red_card(ctx.catalog, ctx.core, actor, c))
            .count() as u32;
        if red_cnt > 0 {
            inc_flag(ctx.core, cur, "l11_rezaiqi_red_discard_turn", red_cnt);
        }
    }
    // `jinglve` (`sishi`): 若死士牌进入弃牌堆，经略持有者立即获得之
    let seats = ctx.core.players.len() as PlayerId;
    for &cid in cards {
        if !ctx.core.discard.contains(&cid) {
            continue;
        }
        let mut hit: Option<(PlayerId, PlayerId)> = None;
        for target in 0..seats {
            if flag(ctx.core, target, &format!("l11_jinglve_sishi_card:{cid}")) > 0 {
                for owner in 0..seats {
                    if flag(ctx.core, target, &format!("l11_jinglve_sishi_from:{owner}")) == cid + 1
                    {
                        hit = Some((owner, target));
                        break;
                    }
                }
            }
        }
        if let Some((owner, target)) = hit {
            ctx.core
                .vm_aux
                .flags
                .remove(&(target, format!("l11_jinglve_sishi_from:{owner}")));
            ctx.core
                .vm_aux
                .flags
                .remove(&(target, format!("l11_jinglve_sishi_card:{cid}")));
            ctx.core
                .vm_aux
                .flags
                .remove(&(owner, "l11_jinglve_active_target".to_string()));
            if ctx.core.player(owner).alive {
                ctx.core.discard.retain(|&c| c != cid);
                ctx.core.player_mut(owner).hand.push(cid);
                ctx.out.push(LogEvent::CardsDrawn {
                    player: owner,
                    cards: vec![cid],
                });
            }
        }
    }
}

pub(in crate::content::host) fn on_card_use_after_l11_npc_a(
    ctx: &mut Ctx<'_>,
    fid: crate::ids::EventId,
    u: &crate::event::CardUseData,
) {
    if !catalog_has_npc_a(ctx.catalog) {
        return;
    }
    let owner = u.player;
    set_flag(ctx.core, owner, "l11_mod:kyou_zhidian_dmg", 0);
    set_flag(ctx.core, owner, "l11_mod:kyou_zhidian_unresp", 0);
    set_flag(ctx.core, owner, "l11_mod:kyou_zhidian_pierce", 0);
    if u.effective_kind_in(ctx.core, ctx.catalog).is_sha_like() {
        set_flag(ctx.core, owner, "l11_mod:gnsheque_pierce", 0);
        set_flag(ctx.core, owner, "l11_zhenqiao_recast_guard", 0);
    }
    if u.card.is_none() && u.use_as == Some(CardKind::Wuzhong) {
        set_flag(ctx.core, owner, "l11_olhuoji_active", 0);
    }
    // `mozhi`: 记录本回合出牌阶段已使用的基本/锦囊牌（含酒；回放停机逻辑由 mozhi_next 判停）
    if !u.cancelled
        && owner == ctx.core.turn.current
        && ctx.core.turn.phase == Phase::Play
        && has_npc_a_skill(ctx.core, ctx.catalog, owner, "mozhi")
    {
        let k = u.effective_kind_in(ctx.core, ctx.catalog);
        if k.is_basic_kind() || is_ordinary_trick_kind(k) {
            let idx = flag(ctx.core, owner, "l11_mozhi_hist_len");
            let Some(tag) = CardKind::ALL.iter().position(|&x| x == k) else {
                return;
            };
            let nat = match u.nature {
                crate::event::DamageNature::Fire => 1u32,
                crate::event::DamageNature::Thunder => 2u32,
                _ => 0u32,
            };
            set_flag(
                ctx.core,
                owner,
                &format!("l11_mozhi_hist:{idx}"),
                (tag as u32) | (nat << 16),
            );
            set_flag(ctx.core, owner, "l11_mozhi_hist_len", idx + 1);
        }
    }
    // `reluanji`: 若乱击未造成伤害，摸等于目标数的牌
    let lj_key = format!("l11_reluanji_fid:{fid}");
    if flag(ctx.core, owner, &lj_key) > 0 {
        set_flag(ctx.core, owner, &lj_key, 0);
        let dealt = flag(ctx.core, owner, &format!("l11_reluanji_dmg:{fid}"));
        set_flag(ctx.core, owner, &format!("l11_reluanji_dmg:{fid}"), 0);
        if dealt == 0 && !u.targets.is_empty() && ctx.core.player(owner).alive {
            ctx.start_draw(owner, u.targets.len() as u32);
        }
    }
    ctx.core
        .vm_aux
        .flags
        .remove(&(owner, format!("l11_duel_shas_fid:{fid}")));
    if u.cancelled {
        return;
    }
    if let Some(cid) = u.card {
        ctx.core
            .vm_aux
            .flags
            .remove(&(owner, format!("l11_card_ignore_armor:{cid}")));
        ctx.core
            .vm_aux
            .flags
            .remove(&(owner, format!("l11_duwu_dmg_card:{cid}")));
        ctx.core
            .vm_aux
            .flags
            .remove(&(owner, format!("l11_kyou_dmg_card:{cid}")));
    }
    // `nzry_shicai` (`shenhua.js:2208`): 每回合首次使用该类别牌结算后，可将仍在处理区/弃牌堆/装备区的牌置于牌堆顶并摸 1
    if let Some(cid) = u.card {
        if has_npc_a_skill(ctx.core, ctx.catalog, owner, "nzry_shicai")
            && is_unblocked_npc_a(ctx.core, ctx.catalog, owner)
        {
            let k = u.effective_kind_in(ctx.core, ctx.catalog);
            let bit = 1u32 << (card_category(k) - 1);
            let seen = flag(ctx.core, owner, "l11_shicai_seen_turn");
            if (seen & bit) == 0 {
                set_flag(ctx.core, owner, "l11_shicai_seen_turn", seen | bit);
                let on_field = ctx.core.discard.contains(&cid)
                    || ctx.core.table.contains(&cid)
                    || (0..ctx.core.players.len() as PlayerId)
                        .any(|s| ctx.core.player(s).equips.contains(&cid));
                if on_field {
                    push_npc_a_park(
                        ctx,
                        HostMachine {
                            def_key: "nzry_shicai".to_string(),
                            behavior: HostBehavior::L11CrossNpcA,
                            owner,
                            target: None,
                            card: None,
                            cards: vec![cid],
                            targets: Vec::new(),
                            stage: HostStage::Act,
                        },
                    );
                }
            }
        }
    }
    // `rexingluan` (`sp2.js:4592`): 出牌阶段每限一次，单目标有点数牌结算后可拿场上同点数或牌堆同点数
    if let Some(cid) = u.card {
        if owner == ctx.core.turn.current
            && ctx.core.turn.phase == Phase::Play
            && u.targets.len() == 1
            && has_npc_a_skill(ctx.core, ctx.catalog, owner, "rexingluan")
            && is_unblocked_npc_a(ctx.core, ctx.catalog, owner)
            && !has_used_skill(ctx.core, ctx.catalog, owner, "rexingluan")
            && ctx.catalog.cards.get(cid as usize).is_some_and(|d| d.rank > 0)
        {
            push_npc_a_park(
                ctx,
                HostMachine {
                    def_key: "rexingluan".to_string(),
                    behavior: HostBehavior::L11CrossNpcA,
                    owner,
                    target: None,
                    card: None,
                    cards: vec![cid],
                    targets: Vec::new(),
                    stage: HostStage::Act,
                },
            );
        }
    }
}

/// `sha_end_step` 钩子：处理 `dcmoukui` 双方模式在【杀】未命中时令目标弃自己 1 张牌。
pub(in crate::content::host) fn on_sha_end_step_l11_npc_a(
    ctx: &mut Ctx<'_>,
    sha_fid: crate::ids::EventId,
    source: PlayerId,
    target: PlayerId,
    is_damage: bool,
) -> bool {
    if !catalog_has_npc_a(ctx.catalog) {
        return false;
    }
    let key = format!("l11_dcmoukui_both:{sha_fid}:{target}");
    if flag(ctx.core, source, &key) == 0 {
        return false;
    }
    set_flag(ctx.core, source, &key, 0);
    if is_damage
        || !ctx.core.player(target).alive
        || !ctx.core.player(source).alive
        || (ctx.core.player(source).hand.is_empty() && ctx.core.player(source).equips.is_empty())
    {
        return false;
    }
    push_npc_a_park(
        ctx,
        HostMachine {
            def_key: "dcmoukui_dodged".to_string(),
            behavior: HostBehavior::L11CrossNpcA,
            owner: source,
            target: Some(target),
            card: None,
            cards: Vec::new(),
            targets: Vec::new(),
            stage: HostStage::Act,
        },
    );
    true
}
