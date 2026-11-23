//! U05-5b（`杀海拾遗 · gwent` 52 武将与 67 角色/衍生/全局技）宿主实现总入口与公共原语。
//!
//! 拆分为职责清晰的 8 个子模块（每文件 < 600 行）：
//! - `group_a` / `group_a_asks`：Batch A（`sqlong*`, `jielue`, `gwhuanshuang*`, `gwjinli*`,
//!   `gwliaotian`, `gwhuanbi`, `gwminxiang`, `gwlangshi`, `gwjingtian*`, `gwjingshi`, `gwweitu*`,
//!   `gwgouhun`, `gwfutian`, `gwzhongmo`, `gwyewu`, `g_gw_yewu`）
//! - `group_b` / `group_b_asks`：Batch B（`shuangxi`, `yangfan`, `gwfengshi`, `gwchenshui`,
//!   `gwliedi`, `julian`, `gwfusheng`, `gwqinwu*`, `huanshu*`, `gwjieyin`, `zhengjun`,
//!   `gwxuezhan`, `bolang`, `gwjushi*`, `gwfengchi`）
//! - `group_c` / `group_c_asks`：Batch C（`lingji`, `gwjinyan`, `gwshenyu`, `junchi`, `hupeng*`,
//!   `hunmo*`, `shuijian`, `yunhuo`, `yinzhang`, `gwtianbian`, `gwxiaoshou`, `kuanglie*`, `gwjiquan`）
//! - `group_d` / `group_d_asks`：Batch D（`nuhou`, `shewu*`, `gwzhanjiang*`, `gwchuanxin`,
//!   `fengjian`, `huandie*`, `xuezhou*`, `fayin`, `gwbaquan`, `huihun`, `lanquan`）

use super::super::*;
use super::u05_b5a_gwent::{
    apply_buff, apply_debuff, apply_gw_baishuang_to_targets, clean_gwent_name, discard_he_card,
    eligible_bronze_tags, eligible_bronze_targets, emit_huoge_use, enemies_of, friends_of,
    has_named_skill, pull_gwent_or_vanished, BRONZE_SPELL_NAMES, GOLD_SPELL_PICK_NAMES,
    SILVER_SPELL_PICK_NAMES,
};
use crate::catalog::{Catalog, SkillLimits, Suit, TriggerMeta};
use crate::event::Phase;
use crate::ids::{EventId, SkillDefId};
use crate::snap::ExpectedAsk;
use crate::state::GameStateCore;

mod group_a;
mod group_a_asks;
mod group_a_steps;
mod group_b;
mod group_b_asks;
mod group_b_steps;
mod group_c;
mod group_c_asks;
mod group_c_steps;
mod group_d;
mod group_d_asks;
mod group_d_steps;
mod targets;

use targets::*;

pub(super) const ASK_ID_KEY: &str = "u05_b5b_ask_id";

pub(super) fn flag(core: &GameStateCore, p: PlayerId, key: &str) -> u32 {
    core.vm_aux
        .flags
        .get(&(p, key.to_string()))
        .copied()
        .unwrap_or(0)
}

pub(super) fn set_flag(core: &mut GameStateCore, p: PlayerId, key: &str, val: u32) {
    if val == 0 {
        core.vm_aux.flags.remove(&(p, key.to_string()));
    } else {
        core.vm_aux.flags.insert((p, key.to_string()), val);
    }
}

pub(super) fn sync_mark(
    core: &mut GameStateCore,
    catalog: &Catalog,
    p: PlayerId,
    skill: &str,
    val: i32,
) {
    if let Some(def) = catalog.skill_by_name(skill) {
        if val > 0 {
            core.vm_aux.marks.insert((p, u32::from(def)), val);
        } else {
            core.vm_aux.marks.remove(&(p, u32::from(def)));
        }
    }
}

pub(super) fn is_red_suit(suit: Suit) -> bool {
    matches!(suit, Suit::Heart | Suit::Diamond)
}

pub(super) fn is_red_card(catalog: &Catalog, c: CardId) -> bool {
    catalog
        .cards
        .get(c as usize)
        .is_some_and(|cd| is_red_suit(cd.suit))
}

pub(super) fn hand_and_equips(core: &GameStateCore, p: PlayerId) -> Vec<CardId> {
    let mut out = core.player(p).hand.clone();
    out.extend(core.player(p).equips.iter().copied());
    out
}

pub(super) fn discardable_hand(core: &GameStateCore, catalog: &Catalog, p: PlayerId) -> Vec<CardId> {
    core.player(p)
        .hand
        .iter()
        .copied()
        .filter(|&c| !crate::kernel::is_gw_dudayuanshuai2_locked(core, catalog, p, c))
        .collect()
}

pub(super) fn discardable_hand_and_equips(
    core: &GameStateCore,
    catalog: &Catalog,
    p: PlayerId,
) -> Vec<CardId> {
    hand_and_equips(core, p)
        .into_iter()
        .filter(|&c| !crate::kernel::is_gw_dudayuanshuai2_locked(core, catalog, p, c))
        .collect()
}

pub(super) fn off_court_put(core: &mut GameStateCore, holder: PlayerId, c: CardId) {
    let oc = &mut core.player_mut(holder).off_court;
    if !oc.contains(&c) {
        oc.push(c);
    }
}

pub(super) fn off_court_take(core: &mut GameStateCore, holder: PlayerId, c: CardId) -> bool {
    let oc = &mut core.player_mut(holder).off_court;
    if let Some(pos) = oc.iter().position(|&x| x == c) {
        oc.remove(pos);
        true
    } else {
        false
    }
}

pub(super) fn vanish_card_from_hand(ctx: &mut Ctx<'_>, owner: PlayerId, c: CardId) {
    if ctx.core.player(owner).hand.contains(&c) {
        ctx.take_hand_card(owner, c);
        ctx.note_cards_lost(owner, &[c]);
        ctx.core
            .vm_aux
            .piles
            .entry((0, "u05_vanished".to_string()))
            .or_default()
            .push(c);
        ctx.emit(EventData::CardsLost {
            player: owner,
            cards: vec![c],
            origins: vec![crate::event::LostOrigin::Hand],
        });
    }
}

pub(super) fn give_or_draw_card_named(ctx: &mut Ctx<'_>, owner: PlayerId, name: &str) -> bool {
    if !ctx.core.player(owner).alive {
        return false;
    }
    if let Some(c) =
        pull_gwent_or_vanished(ctx, |cid| clean_gwent_name(ctx.catalog.cards[cid as usize].name) == name)
    {
        ctx.core.player_mut(owner).hand.push(c);
        ctx.emit(EventData::CardsDrawn {
            player: owner,
            cards: vec![c],
        });
        return true;
    }
    false
}

pub(super) fn give_random_spell_of_tier(ctx: &mut Ctx<'_>, owner: PlayerId, tier: u8) -> bool {
    let pool: &[&str] = match tier {
        0 => &[
            "gw_qinpendayu",
            "gw_birinongwu",
            "gw_ciguhanshuang",
            "gw_baoxueyaoshui",
            "gw_zhihuanjun",
            "gw_zumoshoukao",
            "gw_aozuzhilei",
            "gw_poxiao",
            "gw_kunenfayin",
            "gw_yanziyaoshui",
            "gw_wenyi",
            "gw_shanbengshu",
        ],
        1 => SILVER_SPELL_PICK_NAMES,
        _ => GOLD_SPELL_PICK_NAMES,
    };
    let mut found = pull_gwent_or_vanished(ctx, |cid| {
        let n = clean_gwent_name(ctx.catalog.cards[cid as usize].name);
        pool.contains(&n) || (tier >= 2 && n == "gw_huangjiashenpan")
    });
    if found.is_none() && tier >= 2 {
        found = pull_gwent_or_vanished(ctx, |cid| {
            clean_gwent_name(ctx.catalog.cards[cid as usize].name) == "gw_huangjiashenpan"
        });
    }
    if let Some(c) = found {
        if ctx.core.player(owner).alive {
            ctx.core.player_mut(owner).hand.push(c);
            ctx.emit(EventData::CardsDrawn {
                player: owner,
                cards: vec![c],
            });
            return true;
        }
        ctx.to_discard(c);
    }
    false
}

pub(super) fn ask_spec(
    observer: PlayerId,
    prompt: &'static str,
    kind: ChoiceKind,
    options: Vec<ChoiceOption>,
    min: usize,
    max: usize,
    allow_cancel: bool,
) -> ExpectedAsk {
    ExpectedAsk {
        observer,
        prompt,
        kind,
        options,
        min,
        max,
        allow_cancel,
        source_skill: None,
    }
}

pub(super) fn wait_ask(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    next_stage: HostStage,
    ask_id: u32,
    build: fn(
        u32,
        PlayerId,
        Option<PlayerId>,
        &[PlayerId],
        Option<u32>,
        &[CardId],
        &GameStateCore,
        &Catalog,
    ) -> Option<ExpectedAsk>,
) -> Flow {
    set_flag(ctx.core, m.owner, ASK_ID_KEY, ask_id);
    m.stage = next_stage;
    let Some(ea) = build(
        ask_id,
        m.owner,
        m.target,
        &m.targets,
        m.card,
        &m.cards,
        ctx.core,
        ctx.catalog,
    ) else {
        set_flag(ctx.core, m.owner, ASK_ID_KEY, 0);
        return Flow::Done;
    };
    if ea.options.is_empty() && !ea.allow_cancel && ea.min > 0 {
        set_flag(ctx.core, m.owner, ASK_ID_KEY, 0);
        return Flow::Done;
    }
    Flow::WaitChoice {
        observer: ea.observer,
        spec: ChoiceSpec {
            kind: ea.kind,
            prompt: ea.prompt,
            options: ea.options,
            min: ea.min,
            max: ea.max,
            allow_cancel: ea.allow_cancel,
        },
    }
}

pub(super) fn take_top_deck_cards(core: &mut GameStateCore, count: usize) -> Vec<CardId> {
    let mut out = Vec::with_capacity(count);
    for _ in 0..count {
        if core.deck.is_empty() && !core.discard.is_empty() {
            let mut d = std::mem::take(&mut core.discard);
            crate::kernel::clear_u05_discard_vanishtags(core, &d);
            core.rng.shuffle(&mut d);
            core.deck = d;
        }
        if let Some(c) = core.deck.pop() {
            out.push(c);
        } else {
            break;
        }
    }
    out
}

pub(super) fn return_off_court_to_deck_top(
    core: &mut GameStateCore,
    holder: PlayerId,
    cards: &[CardId],
) {
    for &c in cards.iter().rev() {
        if off_court_take(core, holder, c) {
            core.deck.push(c);
        }
    }
}

/// `get.type(card, "trick") === "trick"`: true for standard/delayed tricks, false for basics/equips/Gwent spells/gwmaoxian/special.
pub(super) fn is_upstream_trick_use(catalog: &Catalog, u: &crate::event::CardUseData) -> bool {
    let eff = u.use_as.unwrap_or_else(|| u.effective_kind(catalog));
    if !eff.is_trick_kind() {
        return false;
    }
    if u.use_as.is_none() && eff == CardKind::Wuzhong {
        if let Some(cid) = u.card {
            let nm = clean_gwent_name(catalog.cards[cid as usize].name);
            if nm.starts_with("gw") || nm.starts_with("tuteng") {
                return false;
            }
        }
    }
    true
}

pub(super) fn is_sha_or_wuyao(catalog: &Catalog, card: CardId) -> bool {
    catalog.card_kind(card).is_sha_like()
        || clean_gwent_name(catalog.cards[card as usize].name) == "gw_wuyao"
}

pub(super) fn mark_sha_no_count_if_sha(
    core: &mut GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    card: CardId,
) {
    if is_sha_or_wuyao(catalog, card) {
        set_flag(core, player, "vm_va_sha_no_count", 1);
    }
}

pub(super) fn is_target_hidden(core: &GameStateCore, p: PlayerId) -> bool {
    let pl = core.player(p);
    pl.jidong_active || pl.yinshen_active || pl.jileng_active || pl.yinmi_active
}

pub(super) fn clear_ask_id(core: &mut GameStateCore, owner: PlayerId) {
    set_flag(core, owner, ASK_ID_KEY, 0);
}

pub(in crate::content::host) fn trigger_meta_u05_gwent_char(
    name: &str,
) -> Option<TriggerMeta> {
    group_a::trigger_meta_group_a(name)
        .or_else(|| group_b::trigger_meta_group_b(name))
        .or_else(|| group_c::trigger_meta_group_c(name))
        .or_else(|| group_d::trigger_meta_group_d(name))
}

pub(in crate::content::host) fn limits_u05_gwent_char(name: &str) -> SkillLimits {
    match name {
        "sqlonghuo" | "gwjinli" | "gwhuanbi" | "gwminxiang" | "gwjingshi" | "gwgouhun"
        | "gwyewu" | "gwqinwu" | "gwjushi" | "lingji" | "junchi" | "hupeng" | "yinzhang"
        | "gwjiquan" | "shewu" | "gwbaquan" | "lanquan" | "bolang_gain" | "gwjingtian"
        | "gwjingtian2" => SkillLimits {
            once_per_turn: true,
            ..Default::default()
        },
        // `gwliaotian` / `gwxiaoshou` 每回合限两次通过 `gw_use2_cnt:*` 旗标强制。
        _ => SkillLimits::default(),
    }
}

pub(in crate::content::host) fn has_phase_use_u05_gwent_char(
    behavior: HostBehavior,
    name: &str,
) -> bool {
    match behavior {
        HostBehavior::U05GwentCharBatchA => matches!(
            name,
            "sqlonghuo"
                | "gwjinli"
                | "gwliaotian"
                | "gwhuanbi"
                | "gwminxiang"
                | "gwjingtian"
                | "gwjingshi"
                | "gwgouhun"
                | "gwyewu"
        ),
        HostBehavior::U05GwentCharBatchB => matches!(name, "shuangxi" | "gwjieyin" | "gwjushi"),
        HostBehavior::U05GwentCharBatchC => matches!(
            name,
            "lingji" | "hupeng" | "hunmo" | "yinzhang" | "gwxiaoshou" | "gwjiquan"
        ),
        HostBehavior::U05GwentCharBatchD => {
            matches!(name, "shewu" | "xuezhou_hp" | "gwbaquan" | "lanquan")
        }
        _ => false,
    }
}

pub(in crate::content::host) fn activate_u05_gwent_char(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<HostMachine> {
    match skill.behavior {
        HostBehavior::U05GwentCharBatchA => group_a::activate_group_a(skill, ctx, owner),
        HostBehavior::U05GwentCharBatchB => group_b::activate_group_b(skill, ctx, owner),
        HostBehavior::U05GwentCharBatchC => group_c::activate_group_c(skill, ctx, owner),
        HostBehavior::U05GwentCharBatchD => group_d::activate_group_d(skill, ctx, owner),
        _ => None,
    }
}

pub(in crate::content::host) fn filter_u05_gwent_char(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    match skill.behavior {
        HostBehavior::U05GwentCharBatchA => group_a::filter_group_a(skill, ctx, owner, event),
        HostBehavior::U05GwentCharBatchB => group_b::filter_group_b(skill, ctx, owner, event),
        HostBehavior::U05GwentCharBatchC => group_c::filter_group_c(skill, ctx, owner, event),
        HostBehavior::U05GwentCharBatchD => group_d::filter_group_d(skill, ctx, owner, event),
        _ => false,
    }
}

pub(in crate::content::host) fn filter_u05_gwent_char_with_frame(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    _frame: &crate::event::Frame,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    filter_u05_gwent_char(skill, ctx, owner, event)
}

pub(in crate::content::host) fn machine_u05_gwent_char(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    match skill.behavior {
        HostBehavior::U05GwentCharBatchA => group_a::machine_group_a(skill, owner, event),
        HostBehavior::U05GwentCharBatchB => group_b::machine_group_b(skill, owner, event),
        HostBehavior::U05GwentCharBatchC => group_c::machine_group_c(skill, owner, event),
        HostBehavior::U05GwentCharBatchD => group_d::machine_group_d(skill, owner, event),
        _ => HostMachine {
            def_key: skill.name.clone(),
            behavior: skill.behavior,
            owner,
            target: None,
            card: None,
            cards: Vec::new(),
            targets: Vec::new(),
            stage: HostStage::Act,
        },
    }
}

pub(in crate::content::host) fn handle_u05_gwent_char(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
) -> Flow {
    match m.behavior {
        HostBehavior::U05GwentCharBatchA => group_a::step_group_a(m, ctx),
        HostBehavior::U05GwentCharBatchB => group_b_steps::step_group_b(m, ctx),
        HostBehavior::U05GwentCharBatchC => group_c_steps::step_group_c(m, ctx),
        HostBehavior::U05GwentCharBatchD => group_d_steps::step_group_d(m, ctx),
        _ => Flow::Done,
    }
}

#[allow(clippy::too_many_arguments)]
pub(in crate::content::host) fn ask_u05_gwent_char(
    behavior: HostBehavior,
    _stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    card: Option<u32>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    let ask_id = flag(core, owner, ASK_ID_KEY);
    if ask_id == 0 {
        return None;
    }
    match behavior {
        HostBehavior::U05GwentCharBatchA => group_a_asks::build_ask_group_a(
            ask_id, owner, target, targets, card, cards, core, catalog,
        ),
        HostBehavior::U05GwentCharBatchB => group_b_asks::build_ask_group_b(
            ask_id, owner, target, targets, card, cards, core, catalog,
        ),
        HostBehavior::U05GwentCharBatchC => group_c_asks::build_ask_group_c(
            ask_id, owner, target, targets, card, cards, core, catalog,
        ),
        HostBehavior::U05GwentCharBatchD => group_d_asks::build_ask_group_d(
            ask_id, owner, target, targets, card, cards, core, catalog,
        ),
        _ => None,
    }
}

/// `phaseDrawBefore` 钩子：处理 `gwjingtian`（弃跳过正常摸牌 +3 镜标）与 `gwzhongmo`（跳过摸牌改获 2 异阶法术）。
pub(crate) fn u05_b5b_on_phase_draw_before(
    ctx: &mut Ctx<'_>,
    player: PlayerId,
) -> bool {
    if !ctx.core.player(player).alive {
        return false;
    }
    if has_named_skill(ctx.core, ctx.catalog, player, "gwjingtian") {
        ctx.log(LogEvent::SkillTriggered {
            skill: "gwjingtian",
            owner: player,
            presentation: None,
        });
        let next = flag(ctx.core, player, "gwjingtian_marks") + 3;
        set_flag(ctx.core, player, "gwjingtian_marks", next);
        sync_mark(ctx.core, ctx.catalog, player, "gwjingtian", next as i32);
        return true;
    }
    if has_named_skill(ctx.core, ctx.catalog, player, "gwzhongmo") {
        ctx.log(LogEvent::SkillTriggered {
            skill: "gwzhongmo",
            owner: player,
            presentation: None,
        });
        let drop_tier = ctx.core.rng.gen_range(3) as u8;
        for t in [0u8, 1, 2] {
            if t != drop_tier {
                give_random_spell_of_tier(ctx, player, t);
            }
        }
        return true;
    }
    false
}

/// `changeHujiaAfter` 钩子：处理 `gwweitu` / `gwweitu_gain` 每累计失去 3 点护甲获得 1 张随机银法术。
pub(crate) fn u05_b5b_on_hujia_changed(
    ctx: &mut Ctx<'_>,
    player: PlayerId,
    delta: i32,
) {
    if delta >= 0 || !ctx.core.player(player).alive {
        return;
    }
    if !has_named_skill(ctx.core, ctx.catalog, player, "gwweitu")
        && !has_named_skill(ctx.core, ctx.catalog, player, "gwweitu_gain")
    {
        return;
    }
    let mut acc = flag(ctx.core, player, "gwweitu_loss") + 1;
    while acc >= 3 {
        acc -= 3;
        let _ = give_random_spell_of_tier(ctx, player, 1);
    }
    set_flag(ctx.core, player, "gwweitu_loss", acc);
    sync_mark(ctx.core, ctx.catalog, player, "gwweitu", acc as i32);
}

/// `phaseUseEnd` 钩子：清理 `hunmo2`（`addTempSkill` 出牌阶段临时禁用标记）。
pub(in crate::content::host) fn u05_b5b_on_phase_use_end(ctx: &mut Ctx<'_>, _player: PlayerId) {
    let seats = ctx.core.players.len() as PlayerId;
    let hunmo2_def = ctx.catalog.skill_by_name("hunmo2");
    for s in 0..seats {
        if let Some(def) = hunmo2_def {
            ctx.core.skills.remove(&(s, def));
        }
        set_flag(ctx.core, s, "gw_hunmo2_banned", 0);
    }
}

/// `dieEnd` 钩子：持有 `off_court` 暂存牌的角色阵亡时将未结算牌移入弃牌堆，守恒全区牌数。
pub(in crate::content::host) fn u05_b5b_on_player_died(ctx: &mut Ctx<'_>, dead: PlayerId) {
    let leftover = std::mem::take(&mut ctx.core.player_mut(dead).off_court);
    for c in leftover {
        ctx.to_discard(c);
    }
    if ctx.core.vm_aux.flags.get(&(0, "gw_xuezhou_owner".to_string())).copied()
        == Some(u32::from(dead) + 1)
    {
        ctx.core.vm_aux.flags.remove(&(0, "gw_xuezhou_owner".to_string()));
    }
}

/// `shaBegin` 钩子：分派 `fayin`。
pub(in crate::content::host) fn u05_b5b_sha_begin_step(
    ctx: &mut Ctx<'_>,
    sha_frame_id: EventId,
    source: PlayerId,
    target: PlayerId,
    _sha_card: Option<CardId>,
) -> bool {
    group_d::try_spawn_fayin_sha_begin(ctx, sha_frame_id, source, target)
}

/// `shaAfter` 钩子：分派 `gwchuanxin` 与 `junchi`。
pub(in crate::content::host) fn u05_b5b_sha_end_step(
    ctx: &mut Ctx<'_>,
    sha_frame_id: EventId,
    source: PlayerId,
    target: PlayerId,
    _sha_card: Option<CardId>,
    _is_damage: bool,
) -> bool {
    if group_d::try_spawn_chuanxin_sha_end(ctx, sha_frame_id, source, target) {
        return true;
    }
    group_c::try_spawn_junchi_sha_end(ctx, sha_frame_id, source, target)
}
