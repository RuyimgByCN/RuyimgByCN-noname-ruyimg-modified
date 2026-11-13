//! U08 英雄杀 Batch A（23 技能，覆盖 10 名武将，`英雄杀/character/skill.js:110-845`）。
//!
//! 覆盖项：
//! 1. `guimian`（兰陵王·鬼面，`110-119`）
//! 2. `lyuxue` & 3. `lyuxue2`（兰陵王·浴血，`120-175`）
//! 4. `yaoji`（妺喜·妖姬，`176-194`）
//! 5. `liebo`（妺喜·裂帛，`195-257`）
//! 6. `zhuxin`（魏忠贤·诛心，`258-283`）
//! 7. `wlianhuan`（魏忠贤·连环，`284-305`）
//! 8. `huli`（南丁格尔·护理，`306-336`）
//! 9. `yixin`（南丁格尔·医心，`337-372`）
//! 10. `xianqu`（南丁格尔·先驱，`373-381`）
//! 11. `zbudao` & 12. `zbudao2`（张三丰·布道，`382-436`）
//! 13. `taiji`（张三丰·太极，`437-447`）
//! 14. `fengliu`（唐伯虎·风流，`448-480`）
//! 15. `luobi`（唐伯虎·落笔，`481-531`）
//! 16. `yaoyi`（杨广·徭役，`532-606`）
//! 17. `shiqin`（杨广·弑亲，`607-627`）
//! 18. `zyhufu`（赵雍·胡服，`628-647`）
//! 19. `hanbei`（赵雍·捍北，`648-660`）
//! 20. `yxsdili`（鲁智深·底力，`761-798`）
//! 21. `kuangchan`（鲁智深·狂禅，`799-810`）
//! 22. `zhijie`（小乔·致洁，`749-760`）
//! 23. `chujia`（小乔·出嫁，`811-845`）

use super::super::*;
use crate::catalog::{CardKind, Catalog, EquipSlot, Faction, QueryCtx, SkillLimits, Suit, TriggerMeta};
use crate::choice::ChoiceOption;
use crate::content::standard::Gender;
use crate::event::{CardUseData, DamageNature, EventData, EventKind, LostOrigin, Phase, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow};
use crate::state::GameStateCore;

pub(in crate::content::host) use super::u08_batch_a_asks::ask_u08_batch_a;
use super::u08_batch_a_asks::{handle_u08_batch_a_ext, wait_ask};

pub(super) const ASK_YAOJI_CONFIRM: u32 = 1;
pub(super) const ASK_LIEBO_TARGET: u32 = 2;
pub(super) const ASK_ZHUXIN_TARGET: u32 = 3;
pub(super) const ASK_ZHUXIN_OWNER_CARD: u32 = 4;
pub(super) const ASK_ZHUXIN_TARGET_CARD: u32 = 5;
pub(super) const ASK_WLIANHUAN_DISCARD_EQUIP: u32 = 6;
pub(super) const ASK_HULI_CARD: u32 = 7;
pub(super) const ASK_HULI_TARGET: u32 = 8;
pub(super) const ASK_YIXIN_DISCARD_TWO: u32 = 9;
pub(super) const ASK_YIXIN_TARGET: u32 = 10;
pub(super) const ASK_ZBUDAO_CONFIRM: u32 = 11;
pub(super) const ASK_ZBUDAO2_CARD: u32 = 12;
pub(super) const ASK_ZBUDAO2_TARGET: u32 = 13;
pub(super) const ASK_TAIJI_CHOOSE_SHA: u32 = 14;
pub(super) const ASK_TAIJI_CHOOSE_TARGET: u32 = 15;
pub(super) const ASK_LUOBI_CONFIRM: u32 = 16;
pub(super) const ASK_LUOBI_GIVE_CARD: u32 = 17;
pub(super) const ASK_LUOBI_GIVE_TARGET: u32 = 18;
pub(super) const ASK_YAOYI_TARGETS: u32 = 19;
pub(super) const ASK_YAOYI_TARGET_GIVE: u32 = 20;
pub(super) const ASK_ZHIJIE_CARD: u32 = 21;
pub(super) const ASK_CHUJIA_DISCARD_TWO: u32 = 22;
pub(super) const ASK_CHUJIA_TARGET: u32 = 23;

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

pub(super) fn alive_others(core: &GameStateCore, owner: PlayerId) -> Vec<PlayerId> {
    core.seat_order_from(owner)
        .into_iter()
        .filter(|&p| p != owner && core.player(p).alive)
        .collect()
}

pub(super) fn alive_all(core: &GameStateCore, owner: PlayerId) -> Vec<PlayerId> {
    core.seat_order_from(owner)
        .into_iter()
        .filter(|&p| core.player(p).alive)
        .collect()
}

pub(super) fn is_sha_card(core: &GameStateCore, catalog: &Catalog, cid: CardId) -> bool {
    core.declared_kinds
        .get(&cid)
        .copied()
        .unwrap_or_else(|| catalog.card_kind(cid))
        .is_sha_like()
}

pub(super) fn is_sha_use(catalog: &Catalog, u: &CardUseData) -> bool {
    u.use_as
        .or_else(|| u.card.map(|c| catalog.card_kind(c)))
        .unwrap_or(CardKind::Sha)
        .is_sha_like()
}

pub(super) fn is_card_red(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    cid: CardId,
) -> bool {
    matches!(
        catalog.effective_card_suit(core, owner, cid),
        Suit::Heart | Suit::Diamond
    )
}

pub(super) fn has_slot_equipped(
    core: &GameStateCore,
    catalog: &Catalog,
    p: PlayerId,
    slot: EquipSlot,
) -> bool {
    core.player(p)
        .equips
        .iter()
        .any(|&c| catalog.card_slot_of(core, c) == Some(slot))
}

pub(super) fn has_lyuxue2(core: &GameStateCore, catalog: &Catalog, p: PlayerId) -> bool {
    flag(core, p, "lyuxue2_mark") > 0
        || catalog
            .skill_by_name("lyuxue2")
            .is_some_and(|def| core.has_skill(p, def))
}

pub(super) fn has_le_in_judge(core: &GameStateCore, catalog: &Catalog, p: PlayerId) -> bool {
    core.player(p).judge.iter().any(|&c| {
        core.declared_kinds
            .get(&c)
            .copied()
            .unwrap_or_else(|| catalog.card_kind(c))
            == CardKind::Le
    })
}

pub(super) fn liebo_targets(core: &GameStateCore, owner: PlayerId) -> Vec<PlayerId> {
    let my_len = core.player(owner).hand.len() as isize;
    alive_others(core, owner)
        .into_iter()
        .filter(|&p| (core.player(p).hand.len() as isize - my_len).abs() <= 1)
        .collect()
}

pub(super) fn huli_targets(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<PlayerId> {
    alive_all(core, owner)
        .into_iter()
        .filter(|&p| {
            core.player(p).hp < core.player(p).max_hp as i32
                && core.computed_distance(catalog, owner, p) <= 1
        })
        .collect()
}

pub(super) fn damaged_alive_players(core: &GameStateCore, owner: PlayerId) -> Vec<PlayerId> {
    alive_all(core, owner)
        .into_iter()
        .filter(|&p| core.player(p).hp < core.player(p).max_hp as i32)
        .collect()
}

pub(super) fn taiji_targets(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    sha_cid: Option<CardId>,
) -> Vec<PlayerId> {
    let atk = core.attack_range(catalog, owner);
    alive_others(core, owner)
        .into_iter()
        .filter(|&p| {
            core.computed_distance(catalog, owner, p) <= atk
                && !crate::rvm_bridge::modifiers::eval_card_target_canceled(
                    core,
                    catalog,
                    owner,
                    p,
                    sha_cid.unwrap_or(0),
                    CardKind::Sha,
                    DamageNature::Normal,
                )
        })
        .collect()
}

pub(super) fn count_alive_females(core: &GameStateCore, catalog: &Catalog) -> u32 {
    (0..core.players.len() as PlayerId)
        .filter(|&p| {
            if !core.player(p).alive {
                return false;
            }
            let g = core.player(p).general_slots[0]
                .and_then(|def| catalog.general_meta(def).map(|(_, _, _, _, gender)| gender))
                .unwrap_or_else(|| core.effective_gender(p));
            g == Gender::Female
        })
        .count() as u32
}

pub(super) fn yaoyi_candidates(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<PlayerId> {
    alive_others(core, owner)
        .into_iter()
        .filter(|&p| {
            core.effective_faction(catalog, p) != Some(Faction::Qun)
                && !core.player(p).hand.is_empty()
        })
        .collect()
}

pub(super) fn chujia_eligible_cards(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<CardId> {
    let hand = &core.player(owner).hand;
    let red_cnt = hand
        .iter()
        .filter(|&&c| is_card_red(core, catalog, owner, c))
        .count();
    let black_cnt = hand.len().saturating_sub(red_cnt);
    hand.iter()
        .copied()
        .filter(|&c| {
            if is_card_red(core, catalog, owner, c) {
                red_cnt >= 2
            } else {
                black_cnt >= 2
            }
        })
        .collect()
}

pub(super) fn draw_direct(ctx: &mut Ctx<'_>, p: PlayerId, n: u32) -> Vec<CardId> {
    let mut drawn = Vec::new();
    for _ in 0..n {
        if let Some(c) = ctx.core.deck.pop() {
            ctx.core.player_mut(p).hand.push(c);
            drawn.push(c);
        }
    }
    if !drawn.is_empty() {
        ctx.emit(EventData::CardsDrawn {
            player: p,
            cards: drawn.clone(),
        });
    } else if n > 0 {
        ctx.start_draw(p, n);
    }
    drawn
}

pub(super) fn discard_cards(ctx: &mut Ctx<'_>, p: PlayerId, cards: &[CardId]) {
    let mut discarded = Vec::new();
    for &c in cards {
        if ctx.core.player(p).hand.contains(&c) {
            ctx.discard_from_hand(p, c);
            discarded.push(c);
        } else if let Some(pos) = ctx.core.player(p).equips.iter().position(|&x| x == c) {
            ctx.core.player_mut(p).equips.remove(pos);
            crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, p, c);
            ctx.note_cards_lost(p, &[c]);
            ctx.to_discard(c);
            ctx.emit(EventData::CardsLost {
                player: p,
                cards: vec![c],
                origins: vec![LostOrigin::Equip],
            });
            discarded.push(c);
        }
    }
    if !discarded.is_empty() {
        ctx.emit(EventData::CardsDiscarded {
            player: p,
            cards: discarded,
        });
    }
}

pub(super) fn move_cards_to_hand(
    ctx: &mut Ctx<'_>,
    from: PlayerId,
    to: PlayerId,
    cards: &[CardId],
) {
    if from == to || cards.is_empty() {
        return;
    }
    let mut moved = Vec::new();
    let mut origins = Vec::new();
    for &c in cards {
        if let Some(pos) = ctx.core.player(from).hand.iter().position(|&x| x == c) {
            ctx.core.player_mut(from).hand.remove(pos);
            moved.push(c);
            origins.push(LostOrigin::Hand);
        } else if let Some(pos) = ctx.core.player(from).equips.iter().position(|&x| x == c) {
            ctx.core.player_mut(from).equips.remove(pos);
            crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, from, c);
            moved.push(c);
            origins.push(LostOrigin::Equip);
        }
    }
    if !moved.is_empty() {
        ctx.note_cards_lost(from, &moved);
        ctx.core.player_mut(to).hand.extend(moved.iter().copied());
        ctx.emit(EventData::CardsLost {
            player: from,
            cards: moved.clone(),
            origins,
        });
        ctx.emit(EventData::CardsMoved {
            cards: moved,
            from,
            to,
        });
    }
}

pub(super) fn grant_skill_named(ctx: &mut Ctx<'_>, p: PlayerId, name: &str) {
    if let Some(def) = ctx.catalog.skill_by_name(name) {
        ctx.core.skills.insert(
            (p, def),
            crate::state::SkillInstance {
                owner: p,
                enabled: true,
                once_used: false,
                expires_end_of_turn: false,
                rounds: 0,
            },
        );
    }
}

pub(super) fn remove_skill_named(ctx: &mut Ctx<'_>, p: PlayerId, name: &str) {
    if let Some(def) = ctx.catalog.skill_by_name(name) {
        ctx.core.remove_skill_and_r05_mark(ctx.catalog, p, def);
    }
}

pub(super) fn refund_once_used(ctx: &mut Ctx<'_>, p: PlayerId, name: &str) {
    if let Some(def) = ctx.catalog.skill_by_name(name) {
        if let Some(inst) = ctx.core.skills.get_mut(&(p, def)) {
            inst.once_used = false;
        }
    }
}

pub(in crate::content::host) fn trigger_meta_u08_batch_a(name: &str) -> Option<TriggerMeta> {
    let meta = |event: EventKind, extra_events: Vec<EventKind>| TriggerMeta {
        event,
        timing: Timing::Normal,
        priority: 1,
        order_selectable: false,
        optional: false,
        extra_events,
    };
    match name {
        "guimian" => Some(meta(EventKind::Damage, Vec::new())),
        "lyuxue" => Some(meta(
            EventKind::Damage,
            vec![EventKind::TurnBegin, EventKind::PhaseBegin, EventKind::Death],
        )),
        "yaoji" | "yaoyi" => Some(meta(EventKind::Damage, Vec::new())),
        "wlianhuan" => Some(meta(EventKind::DamageBefore, Vec::new())),
        "zbudao" | "fengliu" | "luobi" | "zyhufu" | "yxsdili" => {
            Some(meta(EventKind::PhaseBegin, Vec::new()))
        }
        "zbudao2" => Some(meta(EventKind::PhaseEnd, Vec::new())),
        "taiji" => Some(meta(EventKind::CardResponded, vec![EventKind::CardUsed])),
        "shiqin" => Some(TriggerMeta {
            event: EventKind::Dying,
            timing: Timing::First,
            priority: 9,
            order_selectable: false,
            optional: false,
            extra_events: Vec::new(),
        }),
        "hanbei" => Some(meta(EventKind::CardUsed, Vec::new())),
        "kuangchan" => Some(meta(EventKind::TurnBegin, vec![EventKind::PhaseBegin])),
        _ => None,
    }
}

pub(in crate::content::host) fn limits_u08_batch_a(name: &str) -> SkillLimits {
    match name {
        "liebo" | "zhuxin" | "yixin" | "zhijie" | "chujia" => SkillLimits {
            once_per_turn: true,
            ..Default::default()
        },
        _ => SkillLimits::default(),
    }
}

pub(in crate::content::host) fn filter_u08_batch_a(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    match skill.name.as_str() {
        "guimian" => matches!(
            event,
            EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && ctx.state.turn.current == owner
                    && d.card.is_some_and(|cid| is_sha_card(ctx.state, ctx.catalog, cid))
        ),
        "lyuxue" => match event {
            EventData::Damage(d) => {
                d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && ctx.state.player(d.target).alive
                    && !has_lyuxue2(ctx.state, ctx.catalog, d.target)
            }
            EventData::TurnBegin { player }
            | EventData::Phase {
                player,
                phase: Phase::Prepare | Phase::Judge,
                begin: true,
            } => {
                if *player != owner {
                    return false;
                }
                let alive = alive_all(ctx.state, owner);
                let marked = alive
                    .iter()
                    .filter(|&&p| has_lyuxue2(ctx.state, ctx.catalog, p))
                    .count();
                marked > 0 && marked >= alive.len() / 2
            }
            EventData::Death { player, .. } => {
                *player == owner
                    && alive_all(ctx.state, owner)
                        .iter()
                        .any(|&p| has_lyuxue2(ctx.state, ctx.catalog, p))
            }
            _ => false,
        },
        "yaoji" => matches!(
            event,
            EventData::Damage(d)
                if d.target == owner
                    && !d.prevented
                    && d.amount > 0
                    && d.source.is_some_and(|src| {
                        src != owner
                            && ctx.state.player(src).alive
                            && !has_le_in_judge(ctx.state, ctx.catalog, src)
                    })
        ),
        "wlianhuan" => matches!(
            event,
            EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && d.card.is_some_and(|cid| is_sha_card(ctx.state, ctx.catalog, cid))
                    && !ctx.state.player(owner).equips.is_empty()
        ),
        "zbudao" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::Draw,
                begin: true,
            } if *player == owner
        ),
        "zbudao2" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::Draw,
                begin: false,
            } if *player == owner
                && flag(ctx.state, owner, "zbudao2_active") > 0
                && !ctx.state.player(owner).hand.is_empty()
                && !alive_others(ctx.state, owner).is_empty()
        ),
        "taiji" => {
            let is_shan = match event {
                EventData::CardResponded { player, card } => {
                    *player == owner && ctx.catalog.card_kind(*card) == CardKind::Shan
                }
                EventData::CardUsed(u) => {
                    u.player == owner
                        && u.use_as
                            .or_else(|| u.card.map(|c| ctx.catalog.card_kind(c)))
                            == Some(CardKind::Shan)
                }
                _ => false,
            };
            is_shan
                && ctx
                    .state
                    .player(owner)
                    .hand
                    .iter()
                    .any(|&c| is_sha_card(ctx.state, ctx.catalog, c))
                && !taiji_targets(ctx.state, ctx.catalog, owner, None).is_empty()
        }
        "fengliu" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::Draw,
                begin: true,
            } if *player == owner && count_alive_females(ctx.state, ctx.catalog) > 0
        ),
        "luobi" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::End,
                begin: true,
            } if *player == owner
                && ctx.state.player(owner).hp < ctx.state.player(owner).max_hp as i32
        ),
        "yaoyi" => matches!(
            event,
            EventData::Damage(d)
                if d.target == owner
                    && !d.prevented
                    && d.amount > 0
                    && !yaoyi_candidates(ctx.state, ctx.catalog, owner).is_empty()
        ),
        "shiqin" => matches!(
            event,
            EventData::Dying { player, .. }
                if *player != owner
                    && ctx.state.player(*player).alive
                    && ctx.state.player(*player).hp <= 0
                    && ctx.state.effective_faction(ctx.catalog, *player) == Some(Faction::Qun)
        ),
        "zyhufu" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::Draw,
                begin: true,
            } if *player == owner
                && !has_slot_equipped(ctx.state, ctx.catalog, owner, EquipSlot::Armor)
        ),
        "hanbei" => matches!(
            event,
            EventData::CardUsed(u)
                if u.player == owner
                    && is_sha_use(ctx.catalog, u)
                    && (has_slot_equipped(ctx.state, ctx.catalog, owner, EquipSlot::HorsePlus)
                        || has_slot_equipped(ctx.state, ctx.catalog, owner, EquipSlot::HorseMinus))
        ),
        "yxsdili" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::Draw,
                begin: true,
            } if *player == owner
                && ctx.state.player(owner).hp < ctx.state.player(owner).max_hp as i32
        ),
        "kuangchan" => {
            let is_lord = ctx
                .state
                .identity_roles()
                .and_then(|r| r.get(owner as usize))
                == Some(&crate::state::Role::Lord)
                || flag(ctx.state, owner, "is_zhu") > 0;
            is_lord
                && flag(ctx.state, owner, "kuangchan_init_done") == 0
                && ctx.state.player(owner).max_hp > 4
                && matches!(
                    event,
                    EventData::TurnBegin { player }
                        | EventData::Phase {
                            player,
                            begin: true,
                            ..
                        } if *player == owner
                )
        }
        _ => false,
    }
}

pub(in crate::content::host) fn activate_u08_batch_a(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<Box<dyn crate::catalog::SkillMachine>> {
    if !ctx.state.player(owner).alive
        || ctx.state.turn.current != owner
        || ctx.state.turn.phase != Phase::Play
        || ctx.catalog.has_diaohulishan(ctx.state, owner)
    {
        return None;
    }
    let can_activate = match skill.name.as_str() {
        "liebo" => !liebo_targets(ctx.state, owner).is_empty(),
        "zhuxin" => {
            !ctx.state.player(owner).hand.is_empty()
                && alive_others(ctx.state, owner)
                    .iter()
                    .any(|&p| !ctx.state.player(p).hand.is_empty())
        }
        "huli" => {
            ctx.state
                .player(owner)
                .hand
                .iter()
                .any(|&c| ctx.catalog.effective_card_suit(ctx.state, owner, c) == Suit::Heart)
                && !huli_targets(ctx.state, ctx.catalog, owner).is_empty()
        }
        "yixin" => {
            flag(ctx.state, owner, "yixin_used") == 0
                && ctx.state.player(owner).hand.len() + ctx.state.player(owner).equips.len() > 2
                && !damaged_alive_players(ctx.state, owner).is_empty()
        }
        "zhijie" => ctx
            .state
            .player(owner)
            .hand
            .iter()
            .any(|&c| ctx.catalog.effective_card_suit(ctx.state, owner, c) == Suit::Heart),
        "chujia" => {
            chujia_eligible_cards(ctx.state, ctx.catalog, owner).len() >= 2
                && !damaged_alive_players(ctx.state, owner).is_empty()
        }
        _ => false,
    };
    if !can_activate {
        return None;
    }
    Some(Box::new(HostMachine {
        def_key: skill.name.clone(),
        behavior: skill.behavior,
        owner,
        target: None,
        card: None,
        cards: Vec::new(),
        targets: Vec::new(),
        stage: HostStage::Act,
    }))
}

pub(in crate::content::host) fn machine_u08_batch_a(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    let mut target = None;
    let mut card = None;
    let mut targets = Vec::new();
    match (skill.name.as_str(), event) {
        ("lyuxue", EventData::Damage(d)) => {
            target = Some(d.target);
            card = Some(1);
        }
        ("lyuxue", _) => {
            card = Some(2);
        }
        ("yaoji", EventData::Damage(d)) => {
            target = d.source;
        }
        ("wlianhuan", EventData::Damage(d)) => {
            target = Some(d.target);
        }
        ("shiqin", EventData::Dying { player, .. }) => {
            target = Some(*player);
        }
        ("hanbei", EventData::CardUsed(u)) => {
            targets = u.targets.clone();
            card = u.card.map(u32::from);
        }
        _ => {}
    }
    HostMachine {
        def_key: skill.name.clone(),
        behavior: skill.behavior,
        owner,
        target,
        card,
        cards: Vec::new(),
        targets,
        stage: HostStage::Act,
    }
}

pub(in crate::content::host) fn handle_u08_batch_a(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
) -> Flow {
    let owner = m.owner;
    match m.def_key.as_str() {
        "guimian" => {
            ctx.core.turn.sha_used_play_phase = false;
            let cur = flag(ctx.core, owner, "yxs_guimian_sha_limit_bonus");
            set_flag(ctx.core, owner, "yxs_guimian_sha_limit_bonus", cur + 1);
            Flow::Done
        }
        "lyuxue" => {
            if m.card == Some(1) {
                if let Some(t) = m.target {
                    if ctx.core.player(t).alive && !has_lyuxue2(ctx.core, ctx.catalog, t) {
                        grant_skill_named(ctx, t, "lyuxue2");
                        set_flag(ctx.core, t, "lyuxue2_mark", 1);
                    }
                }
            } else if m.card == Some(2) {
                let victims: Vec<PlayerId> = alive_all(ctx.core, owner)
                    .into_iter()
                    .filter(|&p| has_lyuxue2(ctx.core, ctx.catalog, p))
                    .collect();
                for p in victims {
                    remove_skill_named(ctx, p, "lyuxue2");
                    set_flag(ctx.core, p, "lyuxue2_mark", 0);
                    ctx.emit(EventData::LoseHp {
                        player: p,
                        amount: 1,
                    });
                }
            }
            Flow::Done
        }
        "yaoji" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_YAOJI_CONFIRM),
            HostStage::SelectCard => {
                m.stage = HostStage::Act;
                if ctx.answer().first() == Some(&ChoiceOption::Confirm) {
                    if let Some(src) = m.target {
                        if ctx.core.player(src).alive
                            && !has_le_in_judge(ctx.core, ctx.catalog, src)
                        {
                            let le_cid = ctx
                                .core
                                .discard
                                .iter()
                                .position(|&c| ctx.catalog.card_kind(c) == CardKind::Le)
                                .map(|i| ctx.core.discard.remove(i))
                                .or_else(|| {
                                    ctx.core
                                        .deck
                                        .iter()
                                        .position(|&c| ctx.catalog.card_kind(c) == CardKind::Le)
                                        .map(|i| ctx.core.deck.remove(i))
                                })
                                .or_else(|| ctx.core.deck.pop())
                                .or_else(|| ctx.core.discard.pop());
                            if let Some(cid) = le_cid {
                                ctx.core.declared_kinds.insert(cid, CardKind::Le);
                                ctx.core.player_mut(src).judge.push(cid);
                            }
                        }
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "liebo" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_LIEBO_TARGET),
            HostStage::TargetChoice => {
                m.stage = HostStage::Act;
                if let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() {
                    let my_hand = std::mem::take(&mut ctx.core.player_mut(owner).hand);
                    let t_hand = std::mem::take(&mut ctx.core.player_mut(t).hand);
                    ctx.core.player_mut(owner).hand = t_hand;
                    ctx.core.player_mut(t).hand = my_hand;
                } else {
                    refund_once_used(ctx, owner, "liebo");
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "zhuxin" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_ZHUXIN_TARGET),
            HostStage::TargetChoice => {
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    refund_once_used(ctx, owner, "zhuxin");
                    m.stage = HostStage::Act;
                    return Flow::Done;
                };
                m.target = Some(t);
                wait_ask(m, ctx, HostStage::SelectCard, ASK_ZHUXIN_OWNER_CARD)
            }
            HostStage::SelectCard => {
                let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() else {
                    m.stage = HostStage::Act;
                    return Flow::Done;
                };
                m.cards = vec![cid];
                wait_ask(m, ctx, HostStage::CompareTargetPick, ASK_ZHUXIN_TARGET_CARD)
            }
            HostStage::CompareTargetPick => {
                m.stage = HostStage::Act;
                let (Some(t), Some(&owner_cid), Some(ChoiceOption::Card(target_cid))) =
                    (m.target, m.cards.first(), ctx.answer().first())
                else {
                    return Flow::Done;
                };
                let target_cid = *target_cid;
                let owner_rank = ctx.catalog.cards[owner_cid as usize].rank;
                let target_rank = ctx.catalog.cards[target_cid as usize].rank;
                discard_cards(ctx, owner, &[owner_cid]);
                discard_cards(ctx, t, &[target_cid]);
                if owner_rank > target_rank && ctx.core.player(t).alive {
                    ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                        Some(owner),
                        t,
                        1,
                        DamageNature::Normal,
                        None,
                    )));
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "wlianhuan" => match m.stage {
            HostStage::Act => {
                wait_ask(m, ctx, HostStage::SelectCard, ASK_WLIANHUAN_DISCARD_EQUIP)
            }
            HostStage::SelectCard => {
                m.stage = HostStage::Act;
                if let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() {
                    if ctx.core.player(owner).equips.contains(&cid) {
                        if let Some(frame) = ctx.res.frame_mut(ctx.current_event_frame_id()) {
                            if let EventData::Damage(dm) = &mut frame.data {
                                dm.amount = dm.amount.saturating_add(1);
                            }
                        }
                        discard_cards(ctx, owner, &[cid]);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        _ => handle_u08_batch_a_ext(m, ctx),
    }
}
