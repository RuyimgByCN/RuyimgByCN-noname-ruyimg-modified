//! Batch U05-6a (`shahai-hearth` Part 1): 生命周期与全局事件钩子。

use super::u05_hearth::{
    exact_base_name, flag, gain_card_to_hand, grant_skill, has_any_skill, has_skill_hearth,
    hearth_formal_enabled, is_du_card_hearth, is_hearth_unique_equip, is_hearth_vanish_card_name,
    remove_skill, rng_pick, set_flag, spawn_card_by_base, spawn_card_by_kind,
};
use super::*;
use crate::catalog::{CardKind, Catalog, EquipSlot, Suit};
use crate::event::{CardUseData, EventData};
use crate::ids::{CardId, PlayerId};
use crate::machine::Ctx;
use crate::state::GameStateCore;

pub(crate) fn host_on_game_start_hearth(core: &mut GameStateCore, catalog: &Catalog) {
    if hearth_formal_enabled(catalog) {
        let seats = core.players.len() as PlayerId;
        for p in 0..seats {
            if has_skill_hearth(core, catalog, p, "qingzun") {
                set_flag(core, p, "hs_qingzun_count", 0);
            }
        }
    }
    super::u05_b6b_hearth::b6b_on_game_start(core, catalog);
}

pub(crate) fn host_on_round_start_hearth(_ctx: &mut Ctx<'_>) {}

pub(crate) fn host_on_phase_use_end_hearth(ctx: &mut Ctx<'_>, player: PlayerId) {
    if hearth_formal_enabled(ctx.catalog) {
        set_flag(ctx.core, player, "hs_buwendingyibian_used_turn", 0);
        set_flag(ctx.core, player, "hs_temp_huogong2", 0);
        set_flag(ctx.core, player, "hs_azaowu_used_turn", 0);
        set_flag(ctx.core, player, "hs_lieyang_turn_used", 0);
        set_flag(ctx.core, player, "hs_nuyan2_used_turn", 0);
        set_flag(ctx.core, player, "hs_maoxian_used_turn", 0);
        set_flag(ctx.core, player, "hs_mobao_used_turn", 0);
        set_flag(ctx.core, player, "hs_yingxi_sha_active", 0);
        let seats = ctx.core.players.len() as PlayerId;
        for p in 0..seats {
            set_flag(ctx.core, p, "hs_hllingxi_used_turn", 0);
        }
    }
    super::u05_b6b_hearth::b6b_on_phase_use_end(ctx, player);
}

pub(crate) fn host_on_phase_end_hearth(ctx: &mut Ctx<'_>) {
    if hearth_formal_enabled(ctx.catalog) {
        let seats = ctx.core.players.len() as PlayerId;
        for p in 0..seats {
            set_flag(ctx.core, p, "hs_shihuawuqi_used_turn", 0);
            set_flag(ctx.core, p, "hs_shouji_begin_used", 0);
            set_flag(ctx.core, p, "hs_shouji_miss_used", 0);
        }
    }
    super::u05_b6b_hearth::b6b_on_phase_end(ctx);
}

pub(crate) fn is_hearth_vanish_card_hook(catalog: &Catalog, card: CardId) -> bool {
    hearth_formal_enabled(catalog)
        && catalog
            .cards
            .get(card as usize)
            .is_some_and(|c| is_hearth_vanish_card_name(c.name))
}

pub(crate) fn host_on_player_died_hearth(
    ctx: &mut Ctx<'_>,
    dead: PlayerId,
    killer_opt: Option<PlayerId>,
) {
    super::u05_b6b_hearth::b6b_on_player_died(ctx, dead);
    if !hearth_formal_enabled(ctx.catalog) {
        return;
    }

    let seats = ctx.core.players.len() as PlayerId;
    for p in 0..seats {
        if !ctx.core.player(p).alive {
            continue;
        }
        // `fuhua2_remove` (`dieAfter`): if `fuhua` owner dies, remove `fuhua2`
        if flag(ctx.core, p, "hs_fuhua2_source") == u32::from(dead) + 1 {
            remove_skill(ctx, p, "fuhua2");
            set_flag(ctx.core, p, "hs_fuhua2_source", 0);
        }
        // `bzhuiji` (`dieAfter`, `character/hearth/skill.js:2980`):
        // when someone dies and killer is alive & != owner -> draw 2 + use Duel on killer
        if has_skill_hearth(ctx.core, ctx.catalog, p, "bzhuiji")
            && !ctx.catalog.is_tieji_sealed(ctx.core, p)
        {
            if let Some(killer) = killer_opt {
                if killer != p && ctx.core.player(killer).alive {
                    set_flag(ctx.core, p, "hs_bzhuiji_killer", u32::from(killer) + 1);
                    let m = Box::new(HostMachine {
                        def_key: "bzhuiji".to_string(),
                        behavior: HostBehavior::U05HearthBatchD,
                        owner: p,
                        target: Some(killer),
                        card: None,
                        cards: Vec::new(),
                        targets: Vec::new(),
                        stage: HostStage::Act,
                    });
                    ctx.res.parks.push(crate::resolution::Park {
                        resume_depth: ctx.res.stack.len(),
                        slot: crate::resolution::Slot::Trigger {
                            frame: ctx.res.stack.last().map_or(crate::resolution::NO_FRAME, |f| f.id),
                        },
                        machine: m,
                        awaiting_answer: false,
                    });
                }
            }
        }
    }
}

pub(crate) fn host_draw_begin_step_hearth(
    ctx: &mut Ctx<'_>,
    player: PlayerId,
    pending: &mut i32,
) {
    super::u05_b6b_hearth::b6b_on_draw_begin(ctx, player, pending);
    if !hearth_formal_enabled(ctx.catalog) {
        return;
    }
    // `ysjqisha_wang` (`character/hearth/skill.js:1246`): draw -1
    if flag(ctx.core, player, "hs_ysjqisha_wang") > 0 {
        *pending = (*pending - 1).max(0);
    }
    // `ylyuchu_draw`: subplayer extra turn draw -1
    if flag(ctx.core, player, "hs_ylyuchu_extra_draw_minus") > 0
        && flag(ctx.core, 0, "extra_turn_in_progress") > 0
    {
        set_flag(ctx.core, player, "hs_ylyuchu_extra_draw_minus", 0);
        *pending = (*pending - 1).max(0);
    }
}

pub(crate) fn host_on_draw_machine_begin_hearth(ctx: &mut Ctx<'_>, player: PlayerId) {
    if !hearth_formal_enabled(ctx.catalog) {
        return;
    }
    // `hsyanxin` (`character/hearth/skill.js:939`): if top of deck is black, move nearest red card to top
    if has_skill_hearth(ctx.core, ctx.catalog, player, "hsyanxin")
        && !ctx.catalog.is_tieji_sealed(ctx.core, player)
        && !ctx.core.deck.is_empty()
    {
        let top_idx = ctx.core.deck.len() - 1;
        let top_cid = ctx.core.deck[top_idx];
        if matches!(
            ctx.catalog.cards[top_cid as usize].suit,
            Suit::Spade | Suit::Club
        ) {
            if let Some(red_idx) = ctx.core.deck[..top_idx]
                .iter()
                .rposition(|&c| matches!(ctx.catalog.cards[c as usize].suit, Suit::Heart | Suit::Diamond))
            {
                let red = ctx.core.deck.remove(red_idx);
                ctx.core.deck.push(red);
            }
        }
    }
}

pub(super) fn wxuying_effective_src(
    core: &GameStateCore,
    catalog: &Catalog,
    user: PlayerId,
    cid: CardId,
) -> CardId {
    if exact_base_name(catalog.cards[cid as usize].name) == "hsfashu_anyingjingxiang" {
        let src_plus = flag(core, user, &format!("hs_wxuying_src_card:{cid}"));
        if src_plus > 0 {
            return (src_plus - 1) as CardId;
        }
    }
    cid
}

pub(super) fn wxuying_apply_hide(ctx: &mut Ctx<'_>, user: PlayerId, cid: CardId) {
    if flag(ctx.core, user, &format!("hs_wxuying_was_shadow:{cid}")) > 0 {
        set_flag(ctx.core, user, &format!("hs_wxuying_was_shadow:{cid}"), 0);
        set_flag(ctx.core, user, &format!("hs_wxuying_kind:{cid}"), 0);
        set_flag(ctx.core, user, &format!("hs_wxuying_shadow:{cid}"), 0);
        set_flag(ctx.core, user, &format!("hs_wxuying_src_card:{cid}"), 0);
        ctx.core.declared_kinds.remove(&cid);
        if ctx.core.player(user).alive && has_skill_hearth(ctx.core, ctx.catalog, user, "wxuying") {
            if ctx.core.turn.current == user {
                ctx.start_draw(user, 1);
            } else {
                ctx.core.player_mut(user).yinshen_active = true;
            }
        }
    }
}

pub(super) fn wxuying_transform_hand(
    ctx: &mut Ctx<'_>,
    user: PlayerId,
    used_kind: CardKind,
    src_card: Option<CardId>,
) {
    if !has_skill_hearth(ctx.core, ctx.catalog, user, "wxuying") {
        return;
    }
    let code = crate::archive::kind_code(used_kind) as u32;
    let src_val = src_card
        .filter(|&sc| ctx.catalog.card_kind(sc) == used_kind)
        .map(|sc| u32::from(sc) + 1)
        .unwrap_or(0);
    let shadows: Vec<CardId> = ctx
        .core
        .player(user)
        .hand
        .iter()
        .copied()
        .filter(|&cid| {
            exact_base_name(ctx.catalog.cards[cid as usize].name) == "hsfashu_anyingjingxiang"
        })
        .collect();
    for sc in shadows {
        ctx.core.declared_kinds.insert(sc, used_kind);
        set_flag(ctx.core, user, &format!("hs_wxuying_kind:{sc}"), code);
        set_flag(ctx.core, user, &format!("hs_wxuying_shadow:{sc}"), 1);
        set_flag(ctx.core, user, &format!("hs_wxuying_src_card:{sc}"), src_val);
    }
}

pub(crate) fn host_card_use_after_hearth(ctx: &mut Ctx<'_>, u: &CardUseData) {
    super::u05_b6b_hearth::b6b_on_card_use_after(ctx, u);
    if !hearth_formal_enabled(ctx.catalog) {
        return;
    }
    let user = u.player;
    let src_card = u
        .card
        .map(|cid| wxuying_effective_src(ctx.core, ctx.catalog, user, cid));
    // `qingzun`: count `hsqingyu_*` cards used
    if let Some(cid) = u.card {
        let cname = exact_base_name(ctx.catalog.cards[cid as usize].name);
        if cname.starts_with("hsqingyu_")
            && has_skill_hearth(ctx.core, ctx.catalog, user, "qingzun")
            && !ctx.catalog.is_tieji_sealed(ctx.core, user)
        {
            let next = flag(ctx.core, user, "hs_qingzun_count") + 1;
            set_flag(ctx.core, user, "hs_qingzun_count", next);
            if let Some(def) = ctx.catalog.skill_by_name("qingzun") {
                ctx.core.vm_aux.marks.insert((user, u32::from(def)), next as i32);
            }
        }
        // `wxuying_hide`: shadow card consumed via UseCard
        wxuying_apply_hide(ctx, user, cid);
    }
    // `wxuying_change`: transform `hsfashu_anyingjingxiang` cards in user's hand into used card kind + copied identity
    let used_kind = u.effective_kind(ctx.catalog);
    wxuying_transform_hand(ctx, user, used_kind, src_card);
}

pub(crate) fn host_sha_begin_step_hearth(
    ctx: &mut Ctx<'_>,
    source: PlayerId,
    target: PlayerId,
    card: Option<CardId>,
    priority_ge_6: bool,
) {
    if !hearth_formal_enabled(ctx.catalog) || priority_ge_6 {
        return;
    }
    // `yingxi2`: sha used via `yingxi` against full-HP target has directHit (cannot Shan)
    if flag(ctx.core, source, "hs_yingxi_sha_active") > 0 {
        set_flag(ctx.core, source, "hs_yingxi_sha_active", 0);
        if ctx.core.player(target).hp >= ctx.core.player(target).max_hp as i32
            && has_skill_hearth(ctx.core, ctx.catalog, source, "yingxi2")
        {
            let cid = card.unwrap_or(0);
            ctx.core
                .vm_aux
                .flags
                .insert((source, format!("sha_unrespondable:{cid}:{target}")), 1);
        }
    }
    // `shouji_begin` (`shaBegin`, usable 1): if target has hand cards, gain a copy (`createCard`)
    if has_skill_hearth(ctx.core, ctx.catalog, source, "shouji")
        && !ctx.catalog.is_tieji_sealed(ctx.core, source)
        && flag(ctx.core, source, "hs_shouji_begin_used") == 0
        && !ctx.core.player(target).hand.is_empty()
    {
        set_flag(ctx.core, source, "hs_shouji_begin_used", 1);
        let target_hand = ctx.core.player(target).hand.clone();
        if let Some(sample) = rng_pick(ctx, &target_hand) {
            let k = ctx.catalog.card_kind(sample);
            if let Some(copy) = spawn_card_by_kind(ctx, k) {
                gain_card_to_hand(ctx, source, copy);
            }
        }
    }
}

pub(crate) fn host_sha_end_step_hearth(
    ctx: &mut Ctx<'_>,
    source: PlayerId,
    target: PlayerId,
    is_damage: bool,
) {
    if !hearth_formal_enabled(ctx.catalog) {
        return;
    }
    let missed = !is_damage;
    if missed {
        if flag(ctx.core, source, "hs_zhaochao_sha_pending") > 0 {
            set_flag(ctx.core, source, "hs_zhaochao_sha_pending", 0);
            set_flag(ctx.core, source, "hs_zhaochao2_missed", 1);
        }
        // `shouji_miss` (`shaMiss`, usable 1): gain copy of target's non-unique equip
        let non_unique_eq: Vec<CardId> = ctx
            .core
            .player(target)
            .equips
            .iter()
            .copied()
            .filter(|&c| !is_hearth_unique_equip(ctx.catalog, c))
            .collect();
        if has_skill_hearth(ctx.core, ctx.catalog, source, "shouji")
            && !ctx.catalog.is_tieji_sealed(ctx.core, source)
            && flag(ctx.core, source, "hs_shouji_miss_used") == 0
            && !non_unique_eq.is_empty()
        {
            if let Some(sample) = rng_pick(ctx, &non_unique_eq) {
                set_flag(ctx.core, source, "hs_shouji_miss_used", 1);
                let k = ctx.catalog.card_kind(sample);
                if let Some(copy) = spawn_card_by_kind(ctx, k) {
                    gain_card_to_hand(ctx, source, copy);
                }
            }
        }
    }
}

pub(crate) fn host_on_hand_card_removed_hearth(
    ctx: &mut Ctx<'_>,
    player: PlayerId,
    card: CardId,
) {
    super::u05_b6b_hearth::b6b_on_hand_card_removed(ctx, player, card);
    if !hearth_formal_enabled(ctx.catalog) {
        return;
    }
    // `muyin`: if hand becomes empty after losing a hand card -> tempHide (`yinshen_active = true`)
    if ctx.core.player(player).alive
        && ctx.core.player(player).hand.is_empty()
        && has_skill_hearth(ctx.core, ctx.catalog, player, "muyin")
        && !ctx.catalog.is_tieji_sealed(ctx.core, player)
    {
        ctx.core.player_mut(player).yinshen_active = true;
    }
    // `wxuying_lose` (`loseAfter`): move `_wxuying` mark to `was_shadow` so only real UseCard/Respond triggers hide/draw
    if flag(ctx.core, player, &format!("hs_wxuying_shadow:{card}")) > 0 {
        set_flag(ctx.core, player, &format!("hs_wxuying_shadow:{card}"), 0);
        set_flag(ctx.core, player, &format!("hs_wxuying_was_shadow:{card}"), 1);
    }
    // `du` + `moxie` (`character/hearth/skill.js:2412`): losing `"du"` from hand
    if ctx.core.player(player).alive
        && is_du_card_hearth(ctx.core, ctx.catalog, player, card)
        && flag(ctx.core, player, "g_du_gifting") == 0
        && flag(ctx.core, player, "hs_du_silent_lose") == 0
    {
        if has_skill_hearth(ctx.core, ctx.catalog, player, "moxie")
            && !ctx.catalog.is_tieji_sealed(ctx.core, player)
        {
            ctx.start_draw(player, 2);
        } else if !has_any_skill(ctx.core, ctx.catalog, player, "g_du") {
            ctx.emit(EventData::LoseHp {
                player,
                amount: 1,
            });
        }
    }
}

pub(crate) fn host_on_recover_before_hearth(
    _ctx: &mut Ctx<'_>,
    _player: PlayerId,
    _amount: &mut u32,
) {
}

pub(crate) fn host_on_equip_gained_hearth(ctx: &mut Ctx<'_>, player: PlayerId, card: CardId) {
    if !hearth_formal_enabled(ctx.catalog) {
        return;
    }
    let cname = exact_base_name(ctx.catalog.cards[card as usize].name);
    if cname == "hsqingyu_zhao" {
        ctx.start_draw(player, 1);
    } else if cname == "hstianqi_nazigelin" {
        ctx.change_hujia(player, 1);
    }
    // `xiubu` (`character/hearth/skill.js:3661`): equip weapon (`equip1`) -> gain `1 - attackFrom` (= range) hslingjian
    if ctx.catalog.card_kind(card).equip_slot() == Some(EquipSlot::Weapon)
        && has_skill_hearth(ctx.core, ctx.catalog, player, "xiubu")
        && !ctx.catalog.is_tieji_sealed(ctx.core, player)
        && !cname.starts_with("hstianqi_")
    {
        let num = (ctx.catalog.cards[card as usize].range as usize).max(1);
        for _ in 0..num {
            if let Some(lc) = spawn_card_by_kind(ctx, CardKind::Hslingjian)
                .or_else(|| spawn_card_by_base(ctx, "hs_lingjian"))
            {
                gain_card_to_hand(ctx, player, lc);
            }
        }
    }
}

pub(crate) fn host_on_equip_revoked_hearth(
    core: &mut GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    card: CardId,
) {
    if !hearth_formal_enabled(catalog) || !core.player(player).alive {
        return;
    }
    let cname = exact_base_name(catalog.cards[card as usize].name);
    if cname == "hstianqi_suolasi" {
        let cur = flag(core, player, "hs_suolasi_pending_lose");
        set_flag(core, player, "hs_suolasi_pending_lose", cur + 1);
    }
    if has_skill_hearth(core, catalog, player, "yinzong") && !catalog.is_tieji_sealed(core, player)
    {
        let cur = flag(core, player, "hs_yinzong_pending_lose");
        set_flag(core, player, "hs_yinzong_pending_lose", cur + 1);
    }
}

pub(crate) fn host_on_outer_settled_hearth(ctx: &mut Ctx<'_>) {
    super::u05_b6b_hearth::b6b_on_outer_settled(ctx);
    if !hearth_formal_enabled(ctx.catalog) {
        return;
    }
    let seats = ctx.core.players.len() as PlayerId;
    for p in 0..seats {
        let suolasi = flag(ctx.core, p, "hs_suolasi_pending_lose");
        if suolasi > 0 {
            set_flag(ctx.core, p, "hs_suolasi_pending_lose", 0);
            if ctx.core.player(p).alive {
                for _ in 0..suolasi {
                    if ctx.core.player(p).hp < ctx.core.player(p).max_hp as i32 {
                        ctx.recover(p, 1, None);
                    }
                }
            }
        }
        let yinzong = flag(ctx.core, p, "hs_yinzong_pending_lose");
        if yinzong > 0 {
            set_flag(ctx.core, p, "hs_yinzong_pending_lose", 0);
            if ctx.core.player(p).alive {
                for _ in 0..yinzong {
                    if let Some(sc) = spawn_card_by_kind(ctx, CardKind::Shan) {
                        gain_card_to_hand(ctx, p, sc);
                    }
                }
            }
        }
    }
}

pub(crate) fn host_turn_before_skip_hearth(ctx: &mut Ctx<'_>, player: PlayerId) -> bool {
    super::u05_b6b_hearth::b6b_on_turn_begin(ctx, player);
    if !hearth_formal_enabled(ctx.catalog) {
        return false;
    }
    has_skill_hearth(ctx.core, ctx.catalog, player, "qianfu2")
}

pub(crate) fn hearth_dying_save_skills(
    core: &GameStateCore,
    catalog: &Catalog,
    responder: PlayerId,
    dying: PlayerId,
) -> Vec<crate::ids::SkillDefId> {
    let mut out = super::u05_b6b_hearth::b6b_dying_save_skills(core, catalog, responder, dying);
    if !hearth_formal_enabled(catalog)
        || !core.player(responder).alive
        || catalog.is_tieji_sealed(core, responder)
        || catalog.blocks_all_cards(core, responder)
    {
        return out;
    }
    if !has_skill_hearth(core, catalog, dying, "shouwang2") {
        if let Some(def) = catalog.skill_by_name("shouwang") {
            if core.has_skill(responder, def) {
                out.push(def);
            }
        }
    }
    let has_black = core
        .player(responder)
        .hand
        .iter()
        .chain(core.player(responder).equips.iter())
        .any(|&c| matches!(catalog.effective_card_suit(core, responder, c), Suit::Spade | Suit::Club));
    if has_black {
        if let Some(def) = catalog.skill_by_name("yuelu") {
            if core.has_skill(responder, def) {
                out.push(def);
            }
        }
    }
    out
}

pub(crate) fn resolve_hearth_dying_save(
    ctx: &mut Ctx<'_>,
    responder: PlayerId,
    dying: PlayerId,
    def: crate::ids::SkillDefId,
) -> bool {
    let sname = ctx.catalog.skill_name(def);
    match sname {
        "shouwang" => {
            ctx.recover(dying, 1, None);
            ctx.change_hujia(dying, 1);
            grant_skill(ctx, dying, "shouwang2", false);
            true
        }
        "yuelu" => {
            set_flag(ctx.core, responder, "hs_yuelu_saved", 0);
            let machine = Box::new(HostMachine {
                def_key: "yuelu".to_string(),
                behavior: HostBehavior::U05HearthBatchD,
                owner: responder,
                target: Some(dying),
                card: None,
                cards: Vec::new(),
                targets: Vec::new(),
                stage: HostStage::Act,
            });
            ctx.res.parks.push(crate::resolution::Park {
                resume_depth: ctx.res.stack.len(),
                slot: crate::resolution::Slot::Trigger {
                    frame: ctx.res.stack.last().map_or(crate::resolution::NO_FRAME, |f| f.id),
                },
                machine,
                awaiting_answer: false,
            });
            false
        }
        _ => super::u05_b6b_hearth::b6b_resolve_dying_save(ctx, responder, dying, def),
    }
}

pub(crate) fn on_skill_removed_hearth(
    core: &mut GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    skill_name: &str,
) -> Vec<CardId> {
    let mut ret = Vec::new();
    match skill_name {
        "nsaiqi" => {
            if let Some(cards) = core.vm_aux.piles.remove(&(owner, "hs_nsaiqi".to_string())) {
                ret.extend(cards);
            }
            if let Some(def) = catalog.skill_by_name("nsaiqi") {
                core.vm_aux.marks.remove(&(owner, u32::from(def)));
            }
        }
        "hshuanling" => {
            core.vm_aux.piles.remove(&(owner, "hs_hshuanling".to_string()));
        }
        "fuhua2" => {
            set_flag(core, owner, "hs_fuhua2_source", 0);
        }
        "hstianqi" => {
            set_flag(core, owner, "hs_hstianqi_mask", 0);
        }
        "hsnitai" => {
            let active = flag(core, owner, "hs_nitai_active_def");
            if active > 0 {
                if let Some(d) = catalog.skills.get((active - 1) as usize) {
                    let _ = core.skills.remove(&(owner, (active - 1) as u16));
                    let _ = d;
                }
                set_flag(core, owner, "hs_nitai_active_def", 0);
            }
        }
        "qianfu2" => {
            set_flag(core, owner, "hs_qianfu2_active", 0);
        }
        _ => {}
    }
    ret.extend(super::u05_b6b_hearth::b6b_on_skill_removed(
        core, catalog, owner, skill_name,
    ));
    ret
}
