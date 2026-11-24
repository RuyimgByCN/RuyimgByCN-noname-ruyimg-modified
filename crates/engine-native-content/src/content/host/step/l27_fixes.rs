//! L27 `选将美化` 规则语义对齐补丁（`l27_fixes.rs`）：
//! 覆盖 `xiefang`、`new_meibu`、`new_zhixi`、`clandianzhan`、`reqimou`、
//! `bolan`/`bolan_g`、`sbqingzheng`、`xingchong`、`dcxiaoxi`、`twfengji`、`twzhouzu`。

use super::l27_batch_a::{
    flag as flag_a, refund_once_used as refund_a, set_flag as set_flag_a,
    suit_bit as suit_bit_a, wait_ask as wait_ask_a, ASK_REQIMOU_CONFIRM, ASK_REQIMOU_HP,
};
use super::l27_batch_b::{
    alive_others, flag, refund_once_used as refund_b, set_flag, wait_ask as wait_ask_b,
    ASK_BOLAN_G_SKILL, ASK_BOLAN_G_TARGET, ASK_BOLAN_SKILL,
};
use super::*;
use crate::catalog::{Catalog, Suit};
use crate::choice::ChoiceOption;
use crate::event::EventData;
use crate::ids::{CardId, PlayerId, SkillDefId};
use crate::machine::{Ctx, Flow};
use crate::state::{GameStateCore, SkillInstance};

pub(super) fn has_skill_named(
    core: &GameStateCore,
    catalog: &Catalog,
    p: PlayerId,
    name: &str,
) -> bool {
    catalog
        .skill_by_name(name)
        .is_some_and(|def| core.has_skill(p, def))
}

pub(super) fn suit_tag(suit: Suit) -> u8 {
    match suit {
        Suit::Spade => 0,
        Suit::Heart => 1,
        Suit::Club => 2,
        Suit::Diamond => 3,
    }
}

/// 刷新单个存活角色持有的 `xiefang` 标记（`选将美化/extension.js:1162-1212`），
/// `excluding` 用于在 `EventData::Death` 早于 `alive=false` 前排除当前阵亡者。
pub(super) fn refresh_xiefang_mark(
    core: &mut GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    excluding: Option<PlayerId>,
) {
    let Some(def) = catalog.skill_by_name(L27_XIEFANG_KEY) else {
        return;
    };
    let count = (0..core.players.len() as PlayerId)
        .filter(|p| {
            Some(*p) != excluding
                && core.player(*p).alive
                && core.matches_gender(catalog, *p, crate::content::standard::Gender::Female)
        })
        .count() as i32;
    if count > 0 {
        core.vm_aux.marks.insert((owner, u32::from(def)), count);
    } else {
        core.vm_aux.marks.remove(&(owner, u32::from(def)));
    }
    set_flag_a(core, owner, "xiefang_game_draw_done", 1);
}

fn clear_skill_marks_by_name(core: &mut GameStateCore, catalog: &Catalog, owner: PlayerId, key: &str) {
    if let Some(def) = catalog.skill_by_name(key) {
        core.vm_aux.marks.remove(&(owner, u32::from(def)));
    }
}

pub(super) fn clear_xingchong_state(core: &mut GameStateCore, catalog: &Catalog, owner: PlayerId) {
    let keys: Vec<String> = core
        .vm_aux
        .flags
        .keys()
        .filter_map(|(p, k)| (*p == owner && k.starts_with("xingchong_card:")).then(|| k.clone()))
        .collect();
    for k in keys {
        core.vm_aux.flags.remove(&(owner, k));
    }
    set_flag(core, owner, "xingchong_tagged_count", 0);
    set_flag(core, owner, "xingchong_show_max", 0);
    clear_skill_marks_by_name(core, catalog, owner, "xingchong");
}

/// `gameDrawEnd` 开局钩子：为建局即持 `xiefang` 的角色初始化存活女性计数标记，并初始化 U05-2 古剑局内标记。
pub(crate) fn host_on_game_start(core: &mut GameStateCore, catalog: &Catalog) {
    super::u05_ow::host_on_game_start_ow(core, catalog);
    let seats = core.players.len() as PlayerId;
    for owner in 0..seats {
        if core.player(owner).alive && has_skill_named(core, catalog, owner, L27_XIEFANG_KEY) {
            refresh_xiefang_mark(core, catalog, owner, None);
        }
    }
    super::l01_rules::l01_on_game_start(core, catalog);
    super::u05_b2_gujian::on_game_start_gujian(core, catalog);
    super::u05_hearth::host_on_game_start_hearth(core, catalog);
    if core
        .boss_mode()
        .is_some_and(|(boss, _)| has_skill_named(core, catalog, boss, "boss_baonu"))
    {
        set_flag(core, 0, "q4_boss_baonuwash_armed", 1);
    }
}

/// `Death` 执行前钩子：当前用于 `yange`/`humeng` 子武将阵亡时退出子将并恢复主将，阻止角色死亡。
pub(crate) fn host_on_death_before(ctx: &mut Ctx<'_>, dying: PlayerId) -> bool {
    super::u05_b2_gujian::on_death_before_gujian(ctx, dying)
}

/// `TurnEndWindow { after: true }`（`phaseAfter`）前钩子：`yange` 子将在拷贝技 `phaseEnd` 之后、`lianjing` 之前退出。
pub(crate) fn host_on_turn_phase_after(ctx: &mut Ctx<'_>, player: PlayerId) {
    super::u05_b2_gujian::on_turn_phase_after_gujian(ctx, player);
}

/// `dieEnd` 死亡结算后钩子（`alive=false` 落账后）：刷新全场存活 `xiefang` 计数，并登记 `lingyong` / 清理 `mhuanyi` 与 U05 `ow` / `hearth` / `gwent` 死亡监听。
pub(crate) fn host_on_player_died(
    ctx: &mut Ctx<'_>,
    dead: PlayerId,
    killer: Option<PlayerId>,
) {
    super::u05_ow::host_on_player_died_ow(ctx, dead);
    super::u05_hearth::host_on_player_died_hearth(ctx, dead, killer);
    crate::content::host::step::u05_b5b_gwent::u05_b5b_on_player_died(ctx, dead);
    let seats = ctx.core.players.len() as PlayerId;
    for owner in 0..seats {
        if !ctx.core.player(owner).alive {
            continue;
        }
        if has_skill_named(ctx.core, ctx.catalog, owner, L27_XIEFANG_KEY) {
            refresh_xiefang_mark(ctx.core, ctx.catalog, owner, None);
        }
        if flag(ctx.core, owner, "u05_mhuanyi_tgt") == u32::from(dead) + 1 {
            set_flag(ctx.core, owner, "u05_mhuanyi_tgt", 0);
            set_flag(ctx.core, owner, "u05_mhuanyi_cls", 0);
        }
        if has_skill_named(ctx.core, ctx.catalog, owner, "mtongnian")
            && super::u05_b1_mtg::alive_enemies_hand_empty(ctx.core, owner)
        {
            super::u05_b1_mtg::return_mtongnian_links(ctx.core, owner);
        }
        super::u05_b1_mtg_subplayer::capture_lingyong_subplayer(ctx, owner, dead);
    }
}

/// `phaseUseEnd`（出牌阶段结束）钩子：清理 `new_meibu` 与 `bolan` 授予的临时技能及 `mtongnian` 镜像牌。
pub(crate) fn host_on_phase_use_end(ctx: &mut Ctx<'_>, player: PlayerId) {
    super::u05_b1_mtg::return_mtongnian_links(ctx.core, player);
    crate::content::host::step::u05_b5b_gwent::u05_b5b_on_phase_use_end(ctx, player);
    for (flag_key, sk_name) in [
        ("l27_temp_phase_use:new_zhixi", L27_NEW_ZHIXI_KEY),
        ("l27_temp_phase_use:new_meibu_range", L27_NEW_MEIBU_RANGE_KEY),
        ("l27_temp_phase_use:new_zhixi_recover", "new_zhixi_recover"),
    ] {
        if flag(ctx.core, player, flag_key) > 0 {
            set_flag(ctx.core, player, flag_key, 0);
            if let Some(def) = ctx.catalog.skill_by_name(sk_name) {
                ctx.core.skills.remove(&(player, def));
            }
            let reclaimed =
                (ctx.catalog.hooks.on_skill_removed)(ctx.core, ctx.catalog, player, sk_name);
            for c in reclaimed {
                ctx.to_discard(c);
            }
        }
    }
    let temp_count = flag(ctx.core, player, "l27_temp_phase_use_count");
    for i in 0..temp_count {
        let key = format!("l27_temp_phase_use_def:{i}");
        let def_plus1 = flag(ctx.core, player, &key);
        set_flag(ctx.core, player, &key, 0);
        if def_plus1 > 0 {
            let def = (def_plus1 - 1) as u16;
            ctx.core.skills.remove(&(player, def));
            if let Some(sdef) = ctx.catalog.skills.get(def as usize) {
                let reclaimed =
                    (ctx.catalog.hooks.on_skill_removed)(ctx.core, ctx.catalog, player, sdef.name);
                for c in reclaimed {
                    ctx.to_discard(c);
                }
            }
        }
    }
    set_flag(ctx.core, player, "l27_temp_phase_use_count", 0);
    let legacy_plus1 = flag(ctx.core, player, "l27_temp_phase_use_def");
    if legacy_plus1 > 0 {
        let def = (legacy_plus1 - 1) as u16;
        ctx.core.skills.remove(&(player, def));
        set_flag(ctx.core, player, "l27_temp_phase_use_def", 0);
        if let Some(sdef) = ctx.catalog.skills.get(def as usize) {
            let reclaimed =
                (ctx.catalog.hooks.on_skill_removed)(ctx.core, ctx.catalog, player, sdef.name);
            for c in reclaimed {
                ctx.to_discard(c);
            }
        }
    }
    set_flag(ctx.core, player, "bolan_granted_def", 0);
    set_flag(ctx.core, player, "bolan_temp_skill", 0);
    super::u05_hearth::host_on_phase_use_end_hearth(ctx, player);
}

/// `roundStart` 轮次开始钩子：清除 `xingchong` 上轮展示牌与标记、`clandianzhan` 本轮花色记录。
pub(crate) fn host_on_round_start(ctx: &mut Ctx<'_>) {
    let seats = ctx.core.players.len() as PlayerId;
    for p in 0..seats {
        clear_xingchong_state(ctx.core, ctx.catalog, p);
        set_flag(ctx.core, p, "clandianzhan_suit_mask", 0);
        set_flag(ctx.core, p, "clandianzhan_mask_round", 0);
        set_flag(ctx.core, p, "clandianzhan_used_round_mask", 0);
        set_flag(ctx.core, p, "clandianzhan_used_round", 0);
    }
    super::u05_b2_gujian::on_round_start_gujian(ctx);
}

/// 角色回合开始（`phaseBegin` / `TurnBegin` Commit 段）钩子：清理 `mhuanyi_target` 等跨回合临时状态。
pub(crate) fn host_on_turn_begin(ctx: &mut Ctx<'_>, player: PlayerId) {
    set_flag(ctx.core, player, "u05_mhuanyi_tgt", 0);
    set_flag(ctx.core, player, "u05_mhuanyi_cls", 0);
}

/// 手牌移除钩子：处理 `mtg_du_*` 离手失血（非交出/保留在他人手牌），以及 `xingchong` 展示牌离手摸 2。
pub(crate) fn host_on_hand_card_removed(ctx: &mut Ctx<'_>, player: PlayerId, card: CardId) {
    if ctx
        .catalog
        .cards
        .get(card as usize)
        .is_some_and(|cd| cd.name.starts_with("mtg_du_"))
        && ctx.core.player(player).alive
        && !ctx
            .core
            .players
            .iter()
            .any(|p| p.hand.contains(&card) || p.off_court.contains(&card))
    {
        ctx.emit(EventData::LoseHp { player, amount: 1 });
    }
    super::u05_hearth::host_on_hand_card_removed_hearth(ctx, player, card);
    if ctx.catalog.skill_by_name("xingchong").is_none() {
        return;
    }
    let key = format!("xingchong_card:{card}");
    if flag(ctx.core, player, &key) == 0 {
        return;
    }
    set_flag(ctx.core, player, &key, 0);
    super::l27_fixes_b::sync_xingchong_mark(ctx.core, ctx.catalog, player);
    if ctx.core.player(player).alive
        && has_skill_named(ctx.core, ctx.catalog, player, "xingchong")
        && !ctx.catalog.is_tieji_sealed(ctx.core, player)
    {
        ctx.start_draw(player, 2);
    }
}

/// 合并 `L28` 与 `L27` 的 `onremove` 技能移除清理回调。
pub(crate) fn on_skill_removed_all(
    core: &mut GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    skill_name: &str,
) -> Vec<CardId> {
    let mut cards = super::on_skill_removed_l28(core, catalog, owner, skill_name);
    cards.extend(super::u05_ow::on_skill_removed_ow(
        core, catalog, owner, skill_name,
    ));
    match skill_name {
        L27_XIEFANG_KEY => {
            clear_skill_marks_by_name(core, catalog, owner, L27_XIEFANG_KEY);
            set_flag_a(core, owner, "xiefang_game_draw_done", 0);
        }
        L27_NEW_ZHIXI_KEY => {
            clear_skill_marks_by_name(core, catalog, owner, L27_NEW_ZHIXI_KEY);
            clear_skill_marks_by_name(core, catalog, owner, L27_NEW_ZHIXI_MARK2_KEY);
            set_flag(core, owner, "new_zhixi", 0);
            set_flag(core, owner, "new_zhixi_mark2", 0);
            set_flag(core, owner, "l27_temp_phase_use:new_zhixi", 0);
        }
        L27_NEW_MEIBU_RANGE_KEY => {
            clear_skill_marks_by_name(core, catalog, owner, L27_NEW_MEIBU_RANGE_KEY);
            set_flag(core, owner, "l27_temp_phase_use:new_meibu_range", 0);
        }
        "new_zhixi_recover" => {
            set_flag(core, owner, "l27_temp_phase_use:new_zhixi_recover", 0);
        }
        "xingchong" => {
            clear_xingchong_state(core, catalog, owner);
        }
        "twfengji" => {
            set_flag(core, owner, "twfengji_rem", 0);
            set_flag(core, owner, "twfengji_x", 0);
            set_flag(core, owner, "twfengji_kind", 0);
            if let Some(exp) = core.vm_aux.piles.remove(&(owner, "twfengji".to_string())) {
                cards.extend(exp);
            }
        }
        "lingyong" | "subplayer" => {
            cards.extend(super::u05_b1_mtg_subplayer::cleanup_lingyong_on_remove(
                core, owner, skill_name,
            ));
        }
        _ => {}
    }
    let hearth_cards =
        super::u05_hearth::on_skill_removed_hearth(core, catalog, owner, skill_name);
    cards.extend(hearth_cards);
    cards
}

fn card_has_chongzhuable_mod(core: &GameStateCore, owner: PlayerId, card: CardId) -> bool {
    flag_a(core, owner, &format!("card_chongzhuable:{card}")) > 0
        || flag_a(core, owner, "card_chongzhuable") > 0
}

/// `clandianzhan` `useCardAfter` 弹出钩子（`选将美化/extension.js:1867-1927`）。
pub(super) fn host_card_use_after_clandianzhan(
    ctx: &mut Ctx<'_>,
    u: &crate::event::CardUseData,
) {
    if u.cancelled || ctx.catalog.skill_by_name("clandianzhan").is_none() {
        return;
    }
    let Some(cid) = u.card else {
        return;
    };
    let Some(cd) = ctx.catalog.cards.get(cid as usize) else {
        return;
    };
    let owner = u.player;
    let round = ctx.core.turn.round.max(1);
    let used_mask = if flag_a(ctx.core, owner, "clandianzhan_used_round") == round {
        flag_a(ctx.core, owner, "clandianzhan_used_round_mask")
    } else {
        0
    };
    let s_bit = suit_bit_a(cd.suit);
    set_flag_a(ctx.core, owner, "clandianzhan_used_round", round);
    set_flag_a(
        ctx.core,
        owner,
        "clandianzhan_used_round_mask",
        used_mask | s_bit,
    );
    if (used_mask & s_bit) != 0 || u.targets.len() != 1 {
        return;
    }
    if !ctx.core.player(owner).alive
        || !has_skill_named(ctx.core, ctx.catalog, owner, "clandianzhan")
        || ctx.catalog.is_tieji_sealed(ctx.core, owner)
    {
        return;
    }
    let t = u.targets[0];
    let need_link = ctx.core.player(t).alive && !ctx.core.player(t).linked;
    // 上游 `extension.js:1885-1889`：目标未横置 OR 同花色手牌中满足 `cardChongzhuable != 'unchanged'` 的张数为 0。
    let has_mod_recast = ctx.core.player(owner).hand.iter().any(|&c| {
        ctx.catalog
            .cards
            .get(c as usize)
            .is_some_and(|hc| hc.suit == cd.suit)
            && card_has_chongzhuable_mod(ctx.core, owner, c)
    });
    if !need_link && has_mod_recast {
        return;
    }
    ctx.res.parks.push(crate::resolution::Park {
        resume_depth: ctx.res.stack.len(),
        slot: crate::resolution::Slot::Intrinsic {
            frame: crate::resolution::NO_FRAME,
        },
        awaiting_answer: false,
        machine: Box::new(HostMachine {
            def_key: "clandianzhan".to_string(),
            behavior: HostBehavior::L27BeautifyBatchA,
            owner,
            target: Some(t),
            card: Some(cid),
            cards: vec![cid],
            targets: u.targets.clone(),
            stage: HostStage::Act,
        }),
    });
}

/// `clandianzhan` 步进：记录本轮花色位（不当轮重置）、横置目标、重铸全部同花色手牌后再摸 1 张。
pub(super) fn step_clandianzhan_fix(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            let Some(cid) = m.card else {
                return Flow::Done;
            };
            let Some(suit) = ctx.catalog.cards.get(cid as usize).map(|cd| cd.suit) else {
                return Flow::Done;
            };
            let round = ctx.core.turn.round.max(1);
            let prev = if flag_a(ctx.core, owner, "clandianzhan_mask_round") == round {
                flag_a(ctx.core, owner, "clandianzhan_suit_mask")
            } else {
                0
            };
            set_flag_a(ctx.core, owner, "clandianzhan_mask_round", round);
            set_flag_a(
                ctx.core,
                owner,
                "clandianzhan_suit_mask",
                prev | suit_bit_a(suit),
            );
            if m.targets.len() == 1 {
                let t = m.targets[0];
                if ctx.core.player(t).alive && !ctx.core.player(t).linked {
                    ctx.emit(EventData::Linked {
                        player: t,
                        linked: true,
                    });
                }
            }
            let recast: Vec<CardId> = ctx
                .core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| {
                    ctx.catalog
                        .cards
                        .get(c as usize)
                        .is_some_and(|cd| cd.suit == suit)
                })
                .collect();
            if !recast.is_empty() {
                for &c in &recast {
                    ctx.take_owned_card(owner, c);
                    ctx.to_discard(c);
                }
                (ctx.catalog.hooks.on_cards_to_discard)(ctx, owner, &recast, false);
                m.stage = HostStage::DrawSettle;
                ctx.start_draw(owner, recast.len() as u32);
                return Flow::Yield;
            }
            ctx.start_draw(owner, 1);
            Flow::Done
        }
        HostStage::DrawSettle => {
            if ctx.core.player(owner).alive {
                ctx.start_draw(owner, 1);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

/// `reqimou` 步进（`选将美化/extension.js:6059-6214`）：选数 → 可取消 `confirm` → 先 `LoseHp` 后 `draw(X)`。
pub(super) fn step_reqimou_fix(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if m.card == Some(16) {
                set_flag_a(ctx.core, owner, "reqimou_x", 0);
                return Flow::Done;
            }
            wait_ask_a(m, ctx, HostStage::SelectNature, ASK_REQIMOU_HP)
        }
        HostStage::SelectNature => {
            let hp = ctx.core.player(owner).hp.max(1) as u32;
            let Some(ChoiceOption::Tag(n)) = ctx.answer().first().copied() else {
                refund_a(ctx, owner, "reqimou");
                set_flag_a(ctx.core, owner, "l27_ask_id", 0);
                return Flow::Done;
            };
            let x = u32::from(n).clamp(1, hp);
            m.card = Some(x);
            wait_ask_a(m, ctx, HostStage::CardsAsk, ASK_REQIMOU_CONFIRM)
        }
        HostStage::CardsAsk => {
            if !matches!(ctx.answer().first().copied(), Some(ChoiceOption::Confirm)) {
                refund_a(ctx, owner, "reqimou");
                set_flag_a(ctx.core, owner, "l27_ask_id", 0);
                return Flow::Done;
            }
            let x = m.card.unwrap_or(1).max(1);
            set_flag_a(ctx.core, owner, "reqimou_used", 1);
            set_flag_a(ctx.core, owner, "reqimou_x", x);
            set_flag_a(ctx.core, owner, "l27_ask_id", 0);
            m.stage = HostStage::DrawSettle;
            ctx.emit(EventData::LoseHp {
                player: owner,
                amount: x,
            });
            Flow::Yield
        }
        HostStage::DrawSettle => {
            let x = m.card.unwrap_or(1).max(1);
            if ctx.core.player(owner).alive {
                ctx.start_draw(owner, x);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn is_bolan_forbidden_name(name: &str) -> bool {
    matches!(
        name,
        "bolan"
            | "bolan_g"
            | "kotomi_chuanxiang"
            | "jijiang"
            | "激将"
            | "hujia"
            | "护驾"
            | "jiuyuan"
            | "救援"
            | "xueyi"
            | "血裔"
            | "huangtian"
            | "黄天"
            | "zhiba"
            | "制霸"
            | "hunzi"
            | "魂姿"
            | "ruoyu"
            | "若愚"
            | "zaoxian"
            | "凿险"
            | "zili"
            | "自立"
            | "zhiji"
            | "志继"
            | "baiyin"
            | "拜印"
            | "qieting"
            | "窃听"
            | "xianzhou"
            | "献州"
            | "benghuai"
            | "崩坏"
            | "wuyong"
            | "武勇"
            | "fuyin"
            | "伏荫"
            | "kurou"
            | "苦肉"
    )
}

/// 动态扫描当前目录内置与扩展武将的「出牌阶段限一次」技能池（`选将美化/extension.js:7751-7773`）。
pub(super) fn collect_bolan_pool(catalog: &Catalog) -> Vec<SkillDefId> {
    let mut pool = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    let mut consider = |general_name: &str, raw_sym: Option<&str>, skills: &[&str]| {
        if general_name.starts_with("gz_jun")
            || raw_sym.is_some_and(|s| s.starts_with("gz_jun"))
        {
            return;
        }
        for &sk in skills {
            if is_bolan_forbidden_name(sk) {
                continue;
            }
            let Some(def) = catalog.skill_by_name(sk) else {
                continue;
            };
            let sdef = &catalog.skills[def as usize];
            if !sdef.limits.once_per_turn || sdef.limits.active_outside_play {
                continue;
            }
            // 上游 `extension.js:7751-7773`：仅收录真实具备 `phaseUse` + `usable: 1` 主动面且非限定/主公/觉醒/使命技。
            if let Some(beh) = sdef.behavior.as_host_behavior() {
                if !super::super::activation::is_host_skill_bolan_eligible(beh, sdef.name) {
                    continue;
                }
            } else if sdef.trigger.is_some() {
                continue;
            }
            if seen.insert(def) {
                pool.push(def);
            }
        }
    };
    for g in &catalog.standard_generals {
        consider(
            g.name,
            g.original_source.as_ref().map(|o| o.raw_symbol),
            g.skills,
        );
    }
    for g in &catalog.extension_generals {
        consider(g.name, None, g.skills);
    }
    pool
}

fn prepare_bolan_options(m: &mut HostMachine, ctx: &mut Ctx<'_>, beneficiary: PlayerId) -> u32 {
    let mut pool = collect_bolan_pool(ctx.catalog);
    set_flag(ctx.core, beneficiary, "bolan_pool_size", pool.len() as u32);
    let mut count = 0u32;
    while count < 3 && !pool.is_empty() {
        let idx = ctx.core.rng.gen_range(pool.len() as u32) as usize;
        let def = pool.remove(idx);
        set_flag(
            ctx.core,
            beneficiary,
            &format!("bolan_opt_{count}"),
            u32::from(def) + 1,
        );
        count += 1;
    }
    set_flag(ctx.core, beneficiary, "bolan_opt_count", count);
    if m.owner != beneficiary {
        set_flag(ctx.core, m.owner, "bolan_opt_count", count);
        for i in 0..count {
            let v = flag(ctx.core, beneficiary, &format!("bolan_opt_{i}"));
            set_flag(ctx.core, m.owner, &format!("bolan_opt_{i}"), v);
        }
    }
    count
}

/// `bolan`/`bolan_g` 步进：动态池随机 3 选 1 授予真实技能，持续至出牌阶段结束。
pub(super) fn step_bolan_fix(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if m.def_key == "bolan_g" {
                let holders: Vec<PlayerId> = alive_others(ctx.core, owner)
                    .into_iter()
                    .filter(|&p| has_skill_named(ctx.core, ctx.catalog, p, "bolan"))
                    .collect();
                if holders.is_empty() {
                    refund_b(ctx, owner, "bolan_g");
                    return Flow::Done;
                }
                if holders.len() == 1 {
                    m.target = Some(holders[0]);
                    m.stage = HostStage::DrawSettle;
                    ctx.emit(EventData::LoseHp {
                        player: owner,
                        amount: 1,
                    });
                    return Flow::Yield;
                }
                return wait_ask_b(m, ctx, HostStage::TargetChoice, ASK_BOLAN_G_TARGET);
            }
            m.target = Some(owner);
            let cnt = prepare_bolan_options(m, ctx, owner);
            if cnt == 0 {
                return Flow::Done;
            }
            wait_ask_b(m, ctx, HostStage::SelectNature, ASK_BOLAN_SKILL)
        }
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(holder)) = ctx.answer().first().copied() else {
                refund_b(ctx, owner, "bolan_g");
                set_flag(ctx.core, owner, "l27_b_ask_id", 0);
                return Flow::Done;
            };
            set_flag(ctx.core, owner, "l27_b_ask_id", 0);
            m.target = Some(holder);
            m.stage = HostStage::DrawSettle;
            ctx.emit(EventData::LoseHp {
                player: owner,
                amount: 1,
            });
            Flow::Yield
        }
        HostStage::DrawSettle => {
            let Some(holder) = m.target else {
                return Flow::Done;
            };
            if !ctx.core.player(owner).alive || !ctx.core.player(holder).alive {
                return Flow::Done;
            }
            let cnt = prepare_bolan_options(m, ctx, owner);
            if cnt == 0 {
                return Flow::Done;
            }
            wait_ask_b(m, ctx, HostStage::SelectNature, ASK_BOLAN_G_SKILL)
        }
        HostStage::SelectNature => {
            let beneficiary = owner;
            let count = flag(ctx.core, beneficiary, "bolan_opt_count");
            let idx = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(n)) if u32::from(n) < count => u32::from(n),
                _ => {
                    set_flag(ctx.core, owner, "l27_b_ask_id", 0);
                    return Flow::Done;
                }
            };
            let def_plus1 = flag(ctx.core, beneficiary, &format!("bolan_opt_{idx}"));
            if def_plus1 > 0 {
                let def = (def_plus1 - 1) as u16;
                let already_had = ctx.core.skills.contains_key(&(beneficiary, def));
                if !already_had {
                    ctx.insert_skill(
                        (beneficiary, def),
                        SkillInstance {
                            owner: beneficiary,
                            enabled: true,
                            once_used: false,
                            expires_end_of_turn: false,
                            rounds: 0,
                        },
                    );
                    let temp_n = flag(ctx.core, beneficiary, "l27_temp_phase_use_count");
                    set_flag(
                        ctx.core,
                        beneficiary,
                        &format!("l27_temp_phase_use_def:{temp_n}"),
                        def_plus1,
                    );
                    set_flag(
                        ctx.core,
                        beneficiary,
                        "l27_temp_phase_use_count",
                        temp_n + 1,
                    );
                }
                set_flag(ctx.core, beneficiary, "bolan_granted_def", def_plus1);
                set_flag(ctx.core, beneficiary, "bolan_temp_skill", idx + 1);
            }
            set_flag(ctx.core, owner, "l27_b_ask_id", 0);
            Flow::Done
        }
        _ => Flow::Done,
    }
}
