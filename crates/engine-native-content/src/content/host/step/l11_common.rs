//! L11（太虚幻境）共用宿主辅助与生命周期回调（`l11_common.rs`）。
//!
//! 为 `l11_cards`、`l11_equips`、`l11_servants` 与 `l11_skills_{a,b,c}` 提供
//! 统一的状态旗标、摸弃牌、出牌/判定/死亡辅助、以及真实生命周期钩子：
//! - `on_draw_begin_l11`: 摸牌阶段 `DP::Begin`（`txhj_xuwangzhimian` 额外 +2 摸牌）；
//! - `on_card_use_after_l11`: 用牌后钩子（`txhj_feilong_skill_blank` 等辅助清理）。

use super::super::*;
use crate::catalog::{CardKind, Catalog, EquipSlot, Suit};
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::event::{CardUseData, DamageNature, EventData};
use crate::ids::{CardId, PlayerId};
use crate::machine::{damage_data_with, Ctx, LogEvent};
use crate::state::{GameStateCore, SkillInstance};

pub(super) const VIRTUAL_HUOGONG_FLAG: &str = "l11_virtual_huogong";
pub(super) const VIRTUAL_TRICK_CID_FLAG: &str = "l11_virtual_trick_cid";

pub(super) fn with_current_damage_mut(
    ctx: &mut Ctx<'_>,
    f: impl FnOnce(&mut crate::event::DamageData),
) {
    let cur = ctx.current_event_frame_id();
    let root_id = match ctx.res.frame(cur).map(|fr| &fr.data) {
        Some(EventData::DamageWindow { root, .. }) => *root,
        _ => cur,
    };
    if let Some(frame) = ctx.res.frame_mut(root_id) {
        if let EventData::Damage(dm) = &mut frame.data {
            f(dm);
        }
    }
}

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

pub(super) fn turn_stamp(core: &GameStateCore) -> u32 {
    core.turn
        .round
        .saturating_mul(64)
        .saturating_add(u32::from(core.turn.current) + 1)
}

pub(super) fn alive_others(core: &GameStateCore, seat: PlayerId) -> Vec<PlayerId> {
    core.seat_order_from(seat)
        .into_iter()
        .filter(|&p| p != seat && core.player(p).alive)
        .collect()
}

pub(super) fn alive_all(core: &GameStateCore, start: PlayerId) -> Vec<PlayerId> {
    core.seat_order_from(start)
        .into_iter()
        .filter(|&p| core.player(p).alive)
        .collect()
}

pub(super) fn he_cards(core: &GameStateCore, p: PlayerId) -> Vec<CardId> {
    let pl = core.player(p);
    pl.hand.iter().chain(pl.equips.iter()).copied().collect()
}

pub(super) fn hej_cards(core: &GameStateCore, p: PlayerId) -> Vec<CardId> {
    let pl = core.player(p);
    pl.hand
        .iter()
        .chain(pl.equips.iter())
        .chain(pl.judge.iter())
        .copied()
        .collect()
}

pub(super) fn has_named_skill(
    core: &GameStateCore,
    catalog: &Catalog,
    p: PlayerId,
    name: &str,
) -> bool {
    catalog
        .skill_by_name(name)
        .is_some_and(|def| core.has_skill(p, def))
}

pub(super) fn ensure_perm_skill(ctx: &mut Ctx<'_>, p: PlayerId, name: &str) {
    let Some(def) = ctx.catalog.skill_by_name(name) else {
        return;
    };
    ctx.ensure_skill(
        (p, def),
        SkillInstance {
            owner: p,
            enabled: true,
            once_used: false,
            expires_end_of_turn: false,
            rounds: 0,
        },
    );
}

pub(super) fn remove_named_skill(ctx: &mut Ctx<'_>, p: PlayerId, name: &str) {
    if let Some(def) = ctx.catalog.skill_by_name(name) {
        ctx.remove_skill(p, def);
    }
}

pub(super) fn refund_once_used(ctx: &mut Ctx<'_>, owner: PlayerId, name: &str) {
    if let Some(def) = ctx.catalog.skill_by_name(name) {
        if let Some(inst) = ctx.core.skills.get_mut(&(owner, def)) {
            inst.once_used = false;
        }
    }
}

pub(super) fn has_equip_named(
    core: &GameStateCore,
    catalog: &Catalog,
    p: PlayerId,
    name: &str,
) -> bool {
    core.player(p)
        .equips
        .iter()
        .any(|&c| catalog.cards.get(c as usize).is_some_and(|cd| cd.name == name))
}

pub(super) fn is_unblocked_l11_owner(
    state: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> bool {
    state.player(owner).alive
        && !catalog.is_tieji_sealed(state, owner)
        && !has_named_skill(state, catalog, owner, "txhj_feilong_skill_blank")
}

/// 锁定技 / `forced: true` 仅受存活与 `txhj_feilong_skill_blank` 影响，不受 `is_tieji_sealed` 封印。
pub(super) fn is_locked_unblocked_l11_owner(
    state: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> bool {
    state.player(owner).alive
        && !has_named_skill(state, catalog, owner, "txhj_feilong_skill_blank")
}

pub(super) fn discard_player_cards(ctx: &mut Ctx<'_>, p: PlayerId, cards: &[CardId]) {
    let mut from_hand_discarded = Vec::new();
    let mut non_hand_discarded = Vec::new();
    for &c in cards {
        let in_hand = ctx.core.player(p).hand.contains(&c);
        let in_equip = ctx.core.player(p).equips.contains(&c);
        let in_judge = ctx.core.player(p).judge.contains(&c);
        if !in_hand && !in_equip && !in_judge {
            continue;
        }
        if in_hand {
            ctx.discard_from_hand(p, c);
            from_hand_discarded.push(c);
        } else {
            ctx.take_owned_card(p, c);
            ctx.to_discard(c);
            non_hand_discarded.push(c);
        }
    }
    if !non_hand_discarded.is_empty()
        && (ctx.catalog.vm_card_history
            || ctx.catalog.skill_by_name("txhj_xuanwuSkill2").is_some())
    {
        for &c in &non_hand_discarded {
            set_flag(ctx.core, p, &format!("l11_discard_non_hand:{c}"), 1);
        }
    }
    let mut all_discarded = from_hand_discarded;
    all_discarded.extend(non_hand_discarded);
    if !all_discarded.is_empty() {
        ctx.out.push(LogEvent::CardDiscarded {
            player: p,
            cards: all_discarded.clone(),
        });
        ctx.emit(EventData::CardsDiscarded {
            player: p,
            cards: all_discarded,
        });
    }
}

pub(super) fn move_cards_to_hand(
    ctx: &mut Ctx<'_>,
    from: PlayerId,
    to: PlayerId,
    cards: &[CardId],
) {
    if from == to || cards.is_empty() || !ctx.core.player(to).alive {
        return;
    }
    let mut moved = Vec::new();
    for &c in cards {
        let owns = ctx.core.player(from).hand.contains(&c)
            || ctx.core.player(from).equips.contains(&c)
            || ctx.core.player(from).judge.contains(&c);
        if !owns {
            continue;
        }
        ctx.take_owned_card(from, c);
        moved.push(c);
    }
    if !moved.is_empty() {
        ctx.note_cards_lost(from, &moved);
        let delivered = (ctx.catalog.hooks.on_before_gain_hand)(ctx, Some(from), to, &moved);
        ctx.core.player_mut(to).hand.extend(delivered.iter().copied());
        if !delivered.is_empty() {
            ctx.out.push(LogEvent::CardsGiven {
                from,
                to,
                cards: delivered.clone(),
            });
            ctx.emit(EventData::CardsMoved {
                from,
                to,
                cards: delivered,
            });
        }
    }
}

pub(super) fn draw_direct_cards(ctx: &mut Ctx<'_>, p: PlayerId, count: usize) -> Vec<CardId> {
    if !ctx.core.player(p).alive || count == 0 {
        return Vec::new();
    }
    let mut gained = Vec::new();
    for _ in 0..count {
        if ctx.core.deck.is_empty() && !ctx.core.discard.is_empty() {
            let mut reshuffled = std::mem::take(&mut ctx.core.discard);
            ctx.core.rng.shuffle(&mut reshuffled);
            ctx.core.deck = reshuffled;
        }
        if let Some(c) = ctx.core.deck.pop() {
            ctx.core.player_mut(p).hand.push(c);
            gained.push(c);
        } else {
            break;
        }
    }
    if !gained.is_empty() {
        ctx.out.push(LogEvent::CardsDrawn {
            player: p,
            cards: gained.clone(),
        });
        ctx.emit(EventData::CardsDrawn {
            player: p,
            cards: gained.clone(),
        });
    }
    gained
}

pub(super) fn gain_card_from_deck_or_discard(
    ctx: &mut Ctx<'_>,
    p: PlayerId,
    cid: CardId,
) -> bool {
    if !ctx.core.player(p).alive {
        return false;
    }
    let found = if let Some(pos) = ctx.core.deck.iter().position(|&c| c == cid) {
        ctx.core.deck.remove(pos);
        true
    } else if let Some(pos) = ctx.core.discard.iter().position(|&c| c == cid) {
        ctx.core.discard.remove(pos);
        true
    } else {
        false
    };
    if found {
        ctx.core.player_mut(p).hand.push(cid);
        ctx.out.push(LogEvent::CardsDrawn {
            player: p,
            cards: vec![cid],
        });
        ctx.emit(EventData::CardsDrawn {
            player: p,
            cards: vec![cid],
        });
    }
    found
}

pub(super) fn emit_damage(
    ctx: &mut Ctx<'_>,
    source: Option<PlayerId>,
    target: PlayerId,
    amount: u32,
    nature: DamageNature,
    card: Option<CardId>,
) {
    if amount == 0 || !ctx.core.player(target).alive {
        return;
    }
    ctx.emit(EventData::Damage(match card {
        Some(c) => damage_data_with(source, target, amount as i32, nature, Some(c)),
        None => damage_data_with(source, target, amount as i32, nature, None),
    }));
}

pub(super) fn emit_lose_hp(ctx: &mut Ctx<'_>, target: PlayerId, amount: u32) {
    if amount == 0 || !ctx.core.player(target).alive {
        return;
    }
    ctx.emit(EventData::LoseHp {
        player: target,
        amount,
    });
}

pub(super) fn would_death_end_game(
    core: &GameStateCore,
    _catalog: &Catalog,
    victim: PlayerId,
) -> bool {
    match &core.mode {
        crate::state::GameMode::Prototype => {
            core.alive_players().filter(|&p| p != victim).count() <= 1
        }
        crate::state::GameMode::Identity { roles } => {
            if roles.get(victim as usize).copied() == Some(crate::state::Role::Lord) {
                return true;
            }
            !roles.iter().enumerate().any(|(i, r)| {
                let p = i as PlayerId;
                p != victim
                    && core.player(p).alive
                    && matches!(r, crate::state::Role::Rebel | crate::state::Role::Traitor)
            })
        }
        crate::state::GameMode::Versus { teams, substitutes } => {
            let mut teams_left = 0usize;
            for (ti, team) in teams.iter().enumerate() {
                let has_alive = team.iter().any(|&p| p != victim && core.player(p).alive);
                let has_sub = team.contains(&victim)
                    && substitutes.get(ti).is_some_and(|s| !s.is_empty());
                if has_alive || has_sub {
                    teams_left += 1;
                }
            }
            teams_left <= 1
        }
        crate::state::GameMode::Doudizhu { lord } => {
            if victim == *lord {
                return true;
            }
            !core
                .alive_players()
                .any(|p| p != victim && p != *lord)
        }
        crate::state::GameMode::Boss { boss, fellows, .. } => {
            if victim == *boss {
                return true;
            }
            !core
                .alive_players()
                .any(|p| p != victim && p != *boss && !fellows.contains(&p))
        }
        _ => false,
    }
}

pub(super) fn revive_as_ally_3_3(ctx: &mut Ctx<'_>, caster: PlayerId, victim: PlayerId) {
    match &mut ctx.core.mode {
        crate::state::GameMode::Identity { roles } => {
            if let Some(cr) = roles.get(caster as usize).copied() {
                if let Some(vr) = roles.get_mut(victim as usize) {
                    *vr = if cr == crate::state::Role::Lord {
                        crate::state::Role::Loyalist
                    } else {
                        cr
                    };
                }
            }
        }
        crate::state::GameMode::Versus { teams, .. } => {
            let caster_team = teams.iter().position(|t| t.contains(&caster));
            if let Some(ct) = caster_team {
                for team in teams.iter_mut() {
                    team.retain(|&p| p != victim);
                }
                teams[ct].push(victim);
            }
        }
        crate::state::GameMode::Boss { boss, fellows, .. } => {
            if caster == *boss && !fellows.contains(&victim) {
                fellows.push(victim);
            } else if caster != *boss {
                fellows.retain(|&p| p != victim);
            }
        }
        _ => {}
    }
    {
        let vp = ctx.core.player_mut(victim);
        vp.alive = true;
        vp.dying = false;
        vp.death_cancelled = false;
        vp.max_hp = 3;
        vp.hp = 3;
    }
    if ctx
        .res
        .stack
        .iter()
        .any(|f| matches!(f.data, EventData::Death { player, .. } if player == victim))
    {
        ctx.core.player_mut(victim).death_cancelled = true;
    }
    if let Some(blank_def) = ctx.catalog.skill_by_name("txhj_feilong_skill_blank") {
        ctx.ensure_skill(
            (victim, blank_def),
            SkillInstance {
                owner: victim,
                enabled: true,
                once_used: false,
                expires_end_of_turn: false,
                rounds: 0,
            },
        );
        set_flag(ctx.core, victim, "l11_feilong_blank", 1);
        let in_damage = ctx
            .res
            .stack
            .iter()
            .any(|f| matches!(f.data, EventData::Damage(_)));
        if in_damage {
            set_flag(ctx.core, victim, "l11_feilong_just_revived", 1);
        }
    }
    set_flag(ctx.core, victim, "l11_feilong_revived_ally", 1);
    crate::resolution::sync_trigger_owners(ctx.core, ctx.res);
}

pub(super) fn spec_ask(
    chooser: PlayerId,
    kind: ChoiceKind,
    prompt: &'static str,
    options: Vec<ChoiceOption>,
    min: usize,
    max: usize,
    allow_cancel: bool,
) -> Option<(PlayerId, ChoiceSpec)> {
    (!options.is_empty() && options.len() >= min).then_some((
        chooser,
        ChoiceSpec {
            kind,
            prompt,
            options,
            min,
            max,
            allow_cancel,
        },
    ))
}

pub(super) fn confirm_ask(chooser: PlayerId, prompt: &'static str) -> Option<(PlayerId, ChoiceSpec)> {
    Some((
        chooser,
        ChoiceSpec {
            kind: ChoiceKind::Confirm,
            prompt,
            options: vec![ChoiceOption::Confirm, ChoiceOption::Pass],
            min: 1,
            max: 1,
            allow_cancel: true,
        },
    ))
}

pub(super) fn card_ask(
    chooser: PlayerId,
    prompt: &'static str,
    cards: Vec<CardId>,
    min: usize,
    max: usize,
    allow_cancel: bool,
) -> Option<(PlayerId, ChoiceSpec)> {
    spec_ask(
        chooser,
        ChoiceKind::SelectCards,
        prompt,
        cards.into_iter().map(ChoiceOption::Card).collect(),
        min,
        max,
        allow_cancel,
    )
}

pub(super) fn player_ask(
    chooser: PlayerId,
    prompt: &'static str,
    players: Vec<PlayerId>,
    min: usize,
    max: usize,
    allow_cancel: bool,
) -> Option<(PlayerId, ChoiceSpec)> {
    spec_ask(
        chooser,
        ChoiceKind::SelectPlayers,
        prompt,
        players.into_iter().map(ChoiceOption::Player).collect(),
        min,
        max,
        allow_cancel,
    )
}

pub(super) fn tag_ask(
    chooser: PlayerId,
    prompt: &'static str,
    tags: impl IntoIterator<Item = u8>,
    allow_cancel: bool,
) -> Option<(PlayerId, ChoiceSpec)> {
    spec_ask(
        chooser,
        ChoiceKind::PickTag,
        prompt,
        tags.into_iter().map(ChoiceOption::Tag).collect(),
        1,
        1,
        allow_cancel,
    )
}

pub(super) fn is_confirmed(ctx: &Ctx<'_>) -> bool {
    matches!(
        ctx.answer().first().copied(),
        Some(ChoiceOption::Confirm) | Some(ChoiceOption::Tag(1))
    )
}

pub(super) fn answered_cards(ctx: &Ctx<'_>) -> Vec<CardId> {
    ctx.answer()
        .iter()
        .filter_map(|o| match o {
            ChoiceOption::Card(c) => Some(*c),
            _ => None,
        })
        .collect()
}

pub(super) fn answered_players(ctx: &Ctx<'_>) -> Vec<PlayerId> {
    ctx.answer()
        .iter()
        .filter_map(|o| match o {
            ChoiceOption::Player(p) => Some(*p),
            _ => None,
        })
        .collect()
}

pub(super) fn active_skill_machine(
    def_key: &str,
    behavior: HostBehavior,
    owner: PlayerId,
) -> Option<Box<dyn crate::catalog::SkillMachine>> {
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

pub(super) fn effective_card_kind(
    state: &GameStateCore,
    catalog: &Catalog,
    c: CardId,
) -> CardKind {
    state
        .declared_kinds
        .get(&c)
        .copied()
        .unwrap_or_else(|| catalog.card_kind(c))
}

pub(super) fn effective_card_rank(catalog: &Catalog, c: CardId) -> u8 {
    catalog.cards.get(c as usize).map_or(0, |cd| cd.rank)
}

pub(super) fn is_sha_use(catalog: &Catalog, u: &CardUseData) -> bool {
    u.use_as
        .or_else(|| u.card.map(|c| catalog.card_kind(c)))
        .is_some_and(|k| k.is_sha_like())
}

pub(super) fn effective_kind_of_use(catalog: &Catalog, u: &CardUseData) -> Option<CardKind> {
    u.use_as.or_else(|| u.card.map(|c| catalog.card_kind(c)))
}

pub(super) fn effective_color_is_red(
    state: &GameStateCore,
    catalog: &Catalog,
    p: PlayerId,
    card: CardId,
) -> bool {
    matches!(
        catalog.effective_card_suit(state, p, card),
        Suit::Heart | Suit::Diamond
    )
}

pub(super) fn finish_table_card(ctx: &mut Ctx<'_>, cid: CardId) {
    if let Some(pos) = ctx.core.table.iter().position(|&c| c == cid) {
        ctx.core.table.remove(pos);
        ctx.to_discard(cid);
    }
}

pub(super) fn current_use_card_mut<'a>(ctx: &'a mut Ctx<'_>) -> Option<&'a mut CardUseData> {
    ctx.res
        .stack
        .iter_mut()
        .rev()
        .find_map(|f| match &mut f.data {
            EventData::CardUsed(u) => Some(u),
            _ => None,
        })
}

pub(super) fn is_single_target_trick_use(catalog: &Catalog, u: &CardUseData) -> bool {
    if u.targets.len() != 1 {
        return false;
    }
    let Some(kind) = effective_kind_of_use(catalog, u) else {
        return false;
    };
    !kind.is_basic_kind() && kind.equip_slot().is_none()
}

pub(super) fn equip_slot_card(
    core: &GameStateCore,
    catalog: &Catalog,
    p: PlayerId,
    slot: EquipSlot,
) -> Option<CardId> {
    core.player(p)
        .equips
        .iter()
        .copied()
        .find(|&c| catalog.card_slot_of(core, c) == Some(slot))
}

pub(super) fn clear_skill_blocked_flag_on_damage(ctx: &mut Ctx<'_>, victim: PlayerId) {
    if has_named_skill(ctx.core, ctx.catalog, victim, "txhj_feilong_skill_blank") {
        remove_named_skill(ctx, victim, "txhj_feilong_skill_blank");
        set_flag(ctx.core, victim, "l11_feilong_blank", 0);
    }
}

/// `DP::Begin` 钩子：L11 `txhj_xuwangzhimian`（`extension_card.js:1703`：摸牌阶段额外摸 2 张）
/// 与 `cxyRuiQi`（`extension.js:4095`：全局己方摸牌阶段额外摸 1 张）。
pub(super) fn on_draw_begin_l11(ctx: &mut Ctx<'_>, player: PlayerId, pending: &mut i32) {
    if !ctx.core.player(player).alive {
        return;
    }
    if !ctx.catalog.is_tieji_sealed(ctx.core, player)
        && (has_named_skill(ctx.core, ctx.catalog, player, "txhj_xuwangzhimian")
            || has_equip_named(ctx.core, ctx.catalog, player, "txhj_xuwangzhimian"))
    {
        if let Some(def) = ctx.catalog.skill_by_name("txhj_xuwangzhimian") {
            ctx.out.push(LogEvent::SkillTriggered {
                skill: ctx.catalog.skills[def as usize].name,
                owner: player,
                presentation: None,
            });
        }
        *pending += 2;
    }
    if let Some(rui_def) = ctx.catalog.skill_by_name("cxyRuiQi") {
        for holder in alive_all(ctx.core, player) {
            if ctx.core.has_skill(holder, rui_def)
                && is_unblocked_l11_owner(ctx.core, ctx.catalog, holder)
                && ctx.core.is_friend_of(ctx.catalog, holder, player)
            {
                ctx.out.push(LogEvent::SkillTriggered {
                    skill: ctx.catalog.skills[rui_def as usize].name,
                    owner: holder,
                    presentation: None,
                });
                *pending += 1;
            }
        }
    }
}

/// `useCardAfter` 弹出钩子：处理 L11 `txshixin_red`（红判后获牌）、`txlieji`（单目标牌后再结算）
/// 与 `cxyFanGong`（成为敌方牌目标结算后反杀）。
pub(super) fn on_card_use_after_l11(ctx: &mut Ctx<'_>, u: &CardUseData) {
    // 1. `txshixin_red`: 红判成功后于该牌结算完成时获得之
    if let Some(cid) = u.card {
        let holders = alive_all(ctx.core, u.player);
        for p in holders {
            if !has_named_skill(ctx.core, ctx.catalog, p, "txshixin_red") {
                continue;
            }
            if flag(ctx.core, p, "l11_shixin_steal_cid") == cid + 1 {
                set_flag(ctx.core, p, "l11_shixin_steal_cid", 0);
                remove_named_skill(ctx, p, "txshixin_red");
                let stolen = if let Some(pos) = ctx.core.discard.iter().position(|&c| c == cid) {
                    ctx.core.discard.remove(pos);
                    true
                } else if let Some(pos) = ctx.core.table.iter().position(|&c| c == cid) {
                    ctx.core.table.remove(pos);
                    true
                } else {
                    false
                };
                if stolen {
                    ctx.core.player_mut(p).hand.push(cid);
                    ctx.out.push(LogEvent::CardsDrawn {
                        player: p,
                        cards: vec![cid],
                    });
                    ctx.emit(EventData::CardsDrawn {
                        player: p,
                        cards: vec![cid],
                    });
                }
            }
        }
    }

    if u.cancelled {
        return;
    }

    // 2. `txlieji`: 使用【杀】或锦囊牌指定唯一目标后，摸 1 并再结算一次
    let owner = u.player;
    if ctx.core.player(owner).alive
        && has_named_skill(ctx.core, ctx.catalog, owner, "txlieji")
        && is_unblocked_l11_owner(ctx.core, ctx.catalog, owner)
        && u.targets.len() == 1
        && ctx.core.player(u.targets[0]).alive
        && flag(ctx.core, owner, "l11_lieji_ts") != turn_stamp(ctx.core)
    {
        let eff_kind = effective_kind_of_use(ctx.catalog, u);
        if eff_kind.is_some_and(|k| {
            k.is_sha_like()
                || (k.is_trick_kind()
                    && !matches!(
                        k,
                        CardKind::Le
                            | CardKind::Bing
                            | CardKind::ShanDian
                            | CardKind::HongShui
                            | CardKind::Guiyoujie
                    ))
        }) {
            let kind_code = match eff_kind {
                Some(CardKind::FireSha) => 2,
                Some(CardKind::LeiSha) => 3,
                Some(CardKind::Sha) => 1,
                _ => 0,
            };
            ctx.res.parks.push(crate::resolution::Park {
                resume_depth: ctx.res.stack.len(),
                slot: crate::resolution::Slot::Intrinsic {
                    frame: crate::resolution::NO_FRAME,
                },
                awaiting_answer: false,
                machine: Box::new(HostMachine {
                    def_key: "txlieji".to_string(),
                    behavior: HostBehavior::L11GeneralSkillsA,
                    owner,
                    target: Some(u.targets[0]),
                    card: None,
                    cards: match u.card {
                        Some(cid) => vec![cid, kind_code],
                        None => vec![kind_code],
                    },
                    targets: Vec::new(),
                    stage: HostStage::Act,
                }),
            });
        }
    }

    // 3. `cxyFanGong`: 成为敌方使用牌的目标且结算完成后，可对其使用一张【杀】
    let src = u.player;
    if ctx.core.player(src).alive {
        for &t in &u.targets {
            if !ctx.core.player(t).alive
                || ctx.core.is_friend_of(ctx.catalog, t, src)
                || !has_named_skill(ctx.core, ctx.catalog, t, "cxyFanGong")
                || !is_unblocked_l11_owner(ctx.core, ctx.catalog, t)
            {
                continue;
            }
            let has_sha = ctx
                .core
                .player(t)
                .hand
                .iter()
                .any(|&c| effective_card_kind(ctx.core, ctx.catalog, c).is_sha_like());
            if !has_sha {
                continue;
            }
            ctx.res.parks.push(crate::resolution::Park {
                resume_depth: ctx.res.stack.len(),
                slot: crate::resolution::Slot::Intrinsic {
                    frame: crate::resolution::NO_FRAME,
                },
                awaiting_answer: false,
                machine: Box::new(HostMachine {
                    def_key: "cxyFanGong".to_string(),
                    behavior: HostBehavior::L11GeneralSkillsC,
                    owner: t,
                    target: Some(src),
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::Act,
                }),
            });
        }
    }
}
