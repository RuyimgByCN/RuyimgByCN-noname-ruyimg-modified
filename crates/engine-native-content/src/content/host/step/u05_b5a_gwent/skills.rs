//! `card/gwent/skill.js` 与 `character/gwent/skill.js` 卡牌关联技、状态技与全局技。

use super::super::super::*;
use super::{discard_he_card, enemies_of, friends_of, pull_card_matching, GOLD_SPELL_PICK_NAMES, SILVER_SPELL_PICK_NAMES};
use crate::event::Phase;
use crate::state::GameStateCore;

pub(in crate::content::host) fn trigger_meta_u05_gwent_skill(
    name: &str,
) -> Option<crate::catalog::TriggerMeta> {
    use crate::catalog::TriggerMeta;
    use crate::event::{EventKind, Timing};
    match name {
        "gw_aerdeyin" => Some(TriggerMeta {
            event: EventKind::TurnBegin,
            timing: Timing::Normal,
            priority: 0,
            order_selectable: false,
            optional: false,
            extra_events: Vec::new(),
        }),
        "gw_baobaoshu" => Some(TriggerMeta {
            event: EventKind::CardUsed,
            timing: Timing::Normal,
            priority: 0,
            order_selectable: false,
            optional: false,
            extra_events: Vec::new(),
        }),
        "gw_nuhaifengbao" | "gw_dieyi" => Some(TriggerMeta {
            event: EventKind::TurnPhaseEnd,
            timing: Timing::Normal,
            priority: 0,
            order_selectable: false,
            optional: false,
            extra_events: Vec::new(),
        }),
        "gw_youer" => Some(TriggerMeta {
            event: EventKind::TurnPhaseEnd,
            timing: Timing::Normal,
            priority: 0,
            order_selectable: false,
            optional: false,
            extra_events: vec![EventKind::Death],
        }),
        "gw_leizhoushu" => Some(TriggerMeta {
            event: EventKind::PhaseBegin,
            timing: Timing::Normal,
            priority: 0,
            order_selectable: false,
            optional: false,
            extra_events: Vec::new(),
        }),
        "_gainspell" => Some(TriggerMeta {
            event: EventKind::DrawPhaseBegin,
            timing: Timing::Normal,
            priority: -11,
            order_selectable: false,
            optional: false,
            extra_events: Vec::new(),
        }),
        "gw_xianzumaijiu" => Some(TriggerMeta {
            event: EventKind::DamageEnd,
            timing: Timing::Normal,
            priority: 0,
            order_selectable: false,
            optional: false,
            extra_events: vec![
                EventKind::TurnPhaseAfter,
                EventKind::DamageCancelled,
                EventKind::DamageZero,
            ],
        }),
        "gwmaoxian_hengsaite_sha" => Some(TriggerMeta {
            event: EventKind::DamageSource,
            timing: Timing::Normal,
            priority: 0,
            order_selectable: false,
            optional: false,
            extra_events: Vec::new(),
        }),
        _ => None,
    }
}

pub(in crate::content::host) fn filter_u05_gwent_skill_with_frame(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    frame: &crate::event::Frame,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    let state = ctx.state;
    if !state.player(owner).alive {
        return false;
    }
    match skill.name.as_str() {
        "gw_xianzumaijiu" => match frame.kind {
            crate::event::EventKind::TurnPhaseAfter => {
                matches!(
                    event,
                    EventData::TurnEndWindow { player, after: true } if *player == owner
                )
            }
            crate::event::EventKind::DamageEnd
            | crate::event::EventKind::DamageCancelled
            | crate::event::EventKind::DamageZero => {
                let EventData::DamageWindow { root, .. } = frame.data else {
                    return false;
                };
                matches!(event, EventData::Damage(d) if d.source == Some(owner))
                    && state
                        .vm_aux
                        .flags
                        .get(&(owner, format!("sha_damage_frame:{root}")))
                        .copied()
                        .unwrap_or(0)
                        > 0
            }
            _ => false,
        },
        "gw_baobaoshu" => {
            let EventData::CardUsed(u) = event else {
                return false;
            };
            if u.player != owner {
                return false;
            }
            if state.player(owner).hand.is_empty() && state.player(owner).equips.is_empty() {
                return false;
            }
            let eff = u.use_as.unwrap_or_else(|| u.effective_kind(ctx.catalog));
            if eff.equip_slot().is_some() {
                return false;
            }
            if let Some(cid) = u.card {
                let cname = super::clean_gwent_name(ctx.catalog.cards[cid as usize].name);
                if matches!(cname, "gw_fuyuan" | "gw_xianzumaijiu" | "gwjinli_jiu")
                    && state
                        .vm_aux
                        .flags
                        .contains_key(&(u.player, format!("dying_save_frame:{}", frame.id)))
                {
                    return false;
                }
                if is_gwent_non_trick_name(cname) && u.use_as.is_none() {
                    return false;
                }
            }
            eff.is_basic_kind() || eff.is_trick_kind()
        }
        _ => filter_u05_gwent_skill(skill, ctx, owner, event),
    }
}

pub(in crate::content::host) fn filter_u05_gwent_skill(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    let state = ctx.state;
    if !state.player(owner).alive {
        return false;
    }
    match (skill.name.as_str(), event) {
        ("gw_aerdeyin", EventData::TurnBegin { player }) => {
            *player == owner
                && state
                    .vm_aux
                    .flags
                    .get(&(owner, "gw_aerdeyin_pending_extra_turn".to_string()))
                    .copied()
                    .unwrap_or(0)
                    == 1
        }
        ("gw_baobaoshu", EventData::CardUsed(u)) => {
            if u.player != owner {
                return false;
            }
            if state.player(owner).hand.is_empty() && state.player(owner).equips.is_empty() {
                return false;
            }
            let eff = u.use_as.unwrap_or_else(|| u.effective_kind(ctx.catalog));
            if eff.equip_slot().is_some() {
                return false;
            }
            if let Some(cid) = u.card {
                let cname = super::clean_gwent_name(ctx.catalog.cards[cid as usize].name);
                if is_gwent_non_trick_name(cname) && u.use_as.is_none() {
                    return false;
                }
            }
            eff.is_basic_kind() || eff.is_trick_kind()
        }
        ("gw_nuhaifengbao", EventData::TurnEndWindow { player, after: false }) => {
            *player == owner
        }
        ("gw_youer", EventData::TurnEndWindow { after: false, .. }) => true,
        ("gw_youer", EventData::Death { player, .. }) => *player == owner,
        ("gw_dieyi", EventData::TurnEndWindow { after: false, .. }) => true,
        (
            "gw_leizhoushu",
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            },
        ) => {
            *player == owner
                && state
                    .alive_players()
                    .map(|p| state.player(p).hand.len())
                    .max()
                    .is_some_and(|max_h| {
                        state
                            .alive_players()
                            .any(|p| p != owner && state.player(p).hand.len() == max_h)
                    })
        }
        ("_gainspell", EventData::DrawPhase { player, .. }) => {
            *player == owner
                && state.turn.phase == Phase::Draw
                && !(state.deck.is_empty() && state.discard.is_empty())
        }
        ("gw_xianzumaijiu", EventData::Damage(d)) => {
            d.source == Some(owner)
                && !d.prevented
                && d.amount > 0
                && d.card.is_some_and(|c| {
                    ctx.catalog.card_kind(c).is_sha_like()
                        || super::clean_gwent_name(ctx.catalog.cards[c as usize].name) == "gw_wuyao"
                })
        }
        ("gw_xianzumaijiu", EventData::TurnEndWindow { player, after: true }) => {
            *player == owner
        }
        ("gwmaoxian_hengsaite_sha", EventData::Damage(d)) => {
            d.source == Some(owner)
                && !d.prevented
                && d.amount > 0
                && state
                    .vm_aux
                    .flags
                    .get(&(owner, "gwmaoxian_hengsaite_active".to_string()))
                    .copied()
                    .unwrap_or(0)
                    == 1
        }
        _ => false,
    }
}

fn is_gwent_non_trick_name(name: &str) -> bool {
    GOLD_SPELL_PICK_NAMES.contains(&name)
        || name == "gw_huangjiashenpan"
        || SILVER_SPELL_PICK_NAMES.contains(&name)
        || super::BRONZE_SPELL_NAMES.iter().any(|&(bn, _)| bn == name)
        || matches!(
            name,
            "gwjinli_jiu" | "gw_xianzumaijiu" | "gw_wuyao" | "gw_lang" | "gw_dudayuanshuai1" | "gw_dudayuanshuai2" | "gw_dieyi"
        )
        || name.starts_with("gwmaoxian_")
}

pub(in crate::content::host) fn handle_u05_gwent_skill(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
) -> Flow {
    match m.def_key.as_str() {
        "gw_aerdeyin" => {
            ctx.core
                .vm_aux
                .flags
                .remove(&(m.owner, "gw_aerdeyin_pending_extra_turn".to_string()));
            if let Some(def) = ctx.catalog.skill_by_name("gw_aerdeyin") {
                ctx.core.skills.remove(&(m.owner, def));
            }
            ctx.log(LogEvent::SkillTriggered {
                skill: "gw_aerdeyin",
                owner: m.owner,
                presentation: None,
            });
            Flow::Done
        }
        "gw_baobaoshu" => match m.stage {
            HostStage::Act => {
                let opts: Vec<ChoiceOption> = ctx
                    .core
                    .player(m.owner)
                    .hand
                    .iter()
                    .chain(ctx.core.player(m.owner).equips.iter())
                    .copied()
                    .map(ChoiceOption::Card)
                    .collect();
                if opts.is_empty() {
                    return Flow::Done;
                }
                m.stage = HostStage::SelectCard;
                Flow::WaitChoice {
                    observer: m.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "gw_baobaoshu_discard",
                        options: opts,
                        min: 1,
                        max: 1,
                        allow_cancel: false,
                    },
                }
            }
            HostStage::SelectCard => {
                if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                    discard_he_card(ctx, m.owner, c);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "gw_nuhaifengbao" => {
            let mut he: Vec<CardId> = ctx.core.player(m.owner).hand.clone();
            he.extend(ctx.core.player(m.owner).equips.iter().copied());
            if !he.is_empty() {
                let pick = he[ctx.core.rng.gen_range(he.len() as u32) as usize];
                discard_he_card(ctx, m.owner, pick);
            }
            let rkey = (m.owner, "gw_nuhaifengbao_rounds".to_string());
            let left = ctx.core.vm_aux.flags.get(&rkey).copied().unwrap_or(2).saturating_sub(1);
            if left == 0 {
                ctx.core.vm_aux.flags.remove(&rkey);
                if let Some(def) = ctx.catalog.skill_by_name("gw_nuhaifengbao") {
                    ctx.core.skills.remove(&(m.owner, def));
                }
            } else {
                ctx.core.vm_aux.flags.insert(rkey, left);
                if let Some(def) = ctx.catalog.skill_by_name("gw_nuhaifengbao") {
                    if let Some(inst) = ctx.core.skills.get_mut(&(m.owner, def)) {
                        inst.rounds = left as u8;
                    }
                }
            }
            Flow::Done
        }
        "gw_youer" => {
            let is_death = ctx
                .res
                .frame(ctx.current_event_frame_id())
                .is_some_and(|f| matches!(f.data, EventData::Death { .. }));
            let cards = ctx
                .core
                .vm_aux
                .piles
                .remove(&(m.owner, "gw_youer_pile".to_string()))
                .unwrap_or_default();
            if !cards.is_empty() {
                if is_death {
                    for &c in &cards {
                        ctx.to_discard(c);
                    }
                    ctx.note_cards_lost(m.owner, &cards);
                    ctx.emit(EventData::CardsLost {
                        player: m.owner,
                        cards: cards.clone(),
                        origins: Vec::new(),
                    });
                    ctx.log(LogEvent::CardDiscarded {
                        player: m.owner,
                        cards,
                    });
                } else if ctx.core.player(m.owner).alive {
                    ctx.core.player_mut(m.owner).hand.extend(cards);
                }
            }
            if let Some(def) = ctx.catalog.skill_by_name("gw_youer") {
                ctx.core.skills.remove(&(m.owner, def));
            }
            Flow::Done
        }
        "gw_dieyi" => {
            let n = ctx
                .core
                .vm_aux
                .flags
                .remove(&(m.owner, "gw_dieyi_storage".to_string()))
                .unwrap_or(1)
                .max(1);
            for _ in 0..n {
                let mut he: Vec<CardId> = ctx.core.player(m.owner).hand.clone();
                he.extend(ctx.core.player(m.owner).equips.iter().copied());
                if he.is_empty() {
                    break;
                }
                let pick = he[ctx.core.rng.gen_range(he.len() as u32) as usize];
                discard_he_card(ctx, m.owner, pick);
            }
            if let Some(def) = ctx.catalog.skill_by_name("gw_dieyi") {
                ctx.core.skills.remove(&(m.owner, def));
            }
            Flow::Done
        }
        "gw_leizhoushu" => {
            let repeats = ctx
                .core
                .vm_aux
                .flags
                .get(&(m.owner, "gw_leizhoushu_count".to_string()))
                .copied()
                .unwrap_or(1)
                .max(1);
            for _ in 0..repeats {
                let Some(max_h) = ctx
                    .core
                    .alive_players()
                    .map(|p| ctx.core.player(p).hand.len())
                    .max()
                else {
                    break;
                };
                let mut list: Vec<PlayerId> = ctx
                    .core
                    .seat_order_from(m.owner)
                    .into_iter()
                    .filter(|&p| ctx.core.player(p).alive && ctx.core.player(p).hand.len() == max_h)
                    .collect();
                let enemies = enemies_of(ctx.core, ctx.catalog, m.owner);
                let has_enemy = enemies.iter().any(|e| list.contains(e));
                if !has_enemy && !enemies.is_empty() {
                    let pick = enemies[ctx.core.rng.gen_range(enemies.len() as u32) as usize];
                    list.push(pick);
                }
                list.retain(|&p| p != m.owner);
                for t in list {
                    let hl = ctx.core.player(t).hand.len();
                    if hl > 0 {
                        let i = ctx.core.rng.gen_range(hl as u32) as usize;
                        let c = ctx.core.player(t).hand[i];
                        ctx.discard_from_hand(t, c);
                    }
                }
            }
            Flow::Done
        }
        "_gainspell" => step_gainspell(m.owner, ctx),
        "gw_xianzumaijiu" => {
            let is_damage = ctx.current_damage().is_ok_and(|d| {
                d.settlement.outcome == crate::event::DamageOutcome::Running
                    && !d.prevented
                    && d.amount > 0
            });
            if is_damage {
                let mut draw_list = vec![m.owner];
                draw_list.extend(friends_of(ctx.core, ctx.catalog, m.owner));
                for p in draw_list.into_iter().rev() {
                    if ctx.core.player(p).alive {
                        ctx.start_draw(p, 1);
                    }
                }
            }
            if let Some(def) = ctx.catalog.skill_by_name("gw_xianzumaijiu") {
                ctx.core.skills.remove(&(m.owner, def));
            }
            Flow::Done
        }
        "gwmaoxian_hengsaite_sha" => {
            if ctx.core.player(m.owner).alive {
                if let Some(sha) =
                    pull_card_matching(ctx, |cid| ctx.catalog.card_kind(cid).is_sha_like())
                {
                    ctx.core.player_mut(m.owner).hand.push(sha);
                    ctx.emit(EventData::CardsDrawn {
                        player: m.owner,
                        cards: vec![sha],
                    });
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

/// `_gainspell` (`card/gwent/skill.js:336`): 摸牌阶段开始倒数，归零将随机金/银法术置于牌堆顶。
fn step_gainspell(owner: PlayerId, ctx: &mut Ctx<'_>) -> Flow {
    const OFFSET: u32 = 1000;
    let get_c = |core: &GameStateCore, idx: usize| -> i32 {
        let raw = core
            .vm_aux
            .flags
            .get(&(owner, format!("gw_spell_gain:{idx}")))
            .copied()
            .unwrap_or(0);
        if raw == 0 {
            -1
        } else {
            raw as i32 - OFFSET as i32
        }
    };
    let mut vals = [
        get_c(ctx.core, 0),
        get_c(ctx.core, 1),
        get_c(ctx.core, 2),
    ];
    let init_flag = ctx
        .core
        .vm_aux
        .flags
        .get(&(owner, "gw_spell_gain_init".to_string()))
        .copied()
        .unwrap_or(0);
    if init_flag == 0 || vals.iter().copied().max().unwrap_or(-1) < 0 {
        let tmp = ctx
            .core
            .vm_aux
            .flags
            .get(&(owner, "gw_spell_gain2".to_string()))
            .copied()
            .unwrap_or(0) as i32;
        let mut bag: Vec<i32> = (0..16).collect();
        for slot in &mut vals {
            let pick = ctx.core.rng.gen_range(bag.len() as u32) as usize;
            *slot = bag.swap_remove(pick);
        }
        let max_v = vals.iter().copied().max().unwrap_or(0);
        ctx.core.vm_aux.flags.insert(
            (owner, "gw_spell_gain2".to_string()),
            ((15 - max_v).max(0) / 2) as u32,
        );
        if tmp > 0 {
            for slot in &mut vals {
                *slot += tmp;
            }
        }
        ctx.core
            .vm_aux
            .flags
            .insert((owner, "gw_spell_gain_init".to_string()), 1);
    }
    for (i, v) in vals.iter_mut().enumerate() {
        if *v == 0 {
            let pool: &[&str] = if i == 0 {
                GOLD_SPELL_PICK_NAMES
            } else {
                SILVER_SPELL_PICK_NAMES
            };
            if let Some(c) = pull_card_matching(ctx, |cid| {
                let n = super::clean_gwent_name(ctx.catalog.cards[cid as usize].name);
                pool.contains(&n) || (i == 0 && n == "gw_huangjiashenpan")
            }) {
                ctx.core.deck.push(c);
            }
        }
        *v -= 1;
        ctx.core.vm_aux.flags.insert(
            (owner, format!("gw_spell_gain:{i}")),
            (*v + OFFSET as i32).max(0) as u32,
        );
    }
    Flow::Done
}

/// 轮次开始钩子：处理 `gw_aerdeyin` (`card/gwent/skill.js:6`)。
pub(in crate::content::host) fn u05_gwent_on_round_start(ctx: &mut Ctx<'_>) {
    let Some(def) = ctx.catalog.skill_by_name("gw_aerdeyin") else {
        return;
    };
    let seats = ctx.core.players.len() as PlayerId;
    for p in 0..seats {
        if !ctx.core.player(p).alive {
            continue;
        }
        if !ctx
            .core
            .skills
            .get(&(p, def))
            .is_some_and(|i| i.enabled)
        {
            continue;
        }
        if ctx.core.player(p).face_down {
            ctx.turn_over(p);
            ctx.core.skills.remove(&(p, def));
            ctx.log(LogEvent::SkillTriggered {
                skill: "gw_aerdeyin",
                owner: p,
                presentation: None,
            });
        } else {
            ctx.core
                .vm_aux
                .flags
                .insert((p, "extra_turn_insert_queued".to_string()), 1);
            ctx.core
                .vm_aux
                .flags
                .insert((p, "gw_aerdeyin_pending_extra_turn".to_string()), 1);
        }
    }
}

pub(in crate::content::host::step) fn apply_fengyin_to_target(ctx: &mut Ctx<'_>, t: PlayerId) {
    let Some(fdef) = ctx.catalog.skill_by_name(GW_FENGYIN_STATE_KEY) else {
        return;
    };
    let defs: Vec<crate::ids::SkillDefId> = ctx
        .core
        .skills
        .iter()
        .filter_map(|((p, d), inst)| (*p == t && *d != fdef && inst.enabled).then_some(*d))
        .collect();
    for def in defs {
        let skill = &ctx.catalog.skills[def as usize];
        let weather_non_locked = matches!(
            skill.name,
            GW_QINPEN_STATE_KEY | "gw_qinpendayu" | GW_BIRIN_STATE_KEY | "gw_birinongwu"
        );
        let eff_meta = if weather_non_locked {
            None
        } else {
            skill.trigger.as_ref()
        };
        let exempt = ctx.catalog.is_forever_skill(def)
            || ctx.catalog.is_charlotte_skill(def)
            || crate::catalog::Catalog::is_locked_skill_name(skill.name, eff_meta);
        if !exempt {
            if let Some(inst) = ctx.core.skills.get_mut(&(t, def)) {
                inst.enabled = false;
                ctx.core
                    .vm_aux
                    .flags
                    .insert((t, format!("gw_fengyin_dis:{def}")), 1);
            }
        }
    }
    ctx.insert_skill(
        (t, fdef),
        crate::state::SkillInstance {
            owner: t,
            enabled: true,
            once_used: false,
            expires_end_of_turn: false,
            rounds: 0,
        },
    );
}

/// `player.getBuff()` (`library/element/player.js:9235`) 6 分支精准移植。
pub(in crate::content::host::step) fn apply_buff(ctx: &mut Ctx<'_>, t: PlayerId) {
    let mut list = vec![1u8, 2, 3, 4, 5, 6];
    let p = ctx.core.player(t);
    if p.hp >= p.max_hp as i32 {
        list.retain(|&x| x != 2);
    }
    if p.judge.is_empty() {
        list.retain(|&x| x != 5);
    }
    if !p.linked && !p.face_down {
        list.retain(|&x| x != 6);
    }
    if ctx.core.player(t).yinshen_active {
        list.retain(|&x| x != 4);
    }
    let choice = list[ctx.core.rng.gen_range(list.len() as u32) as usize];
    match choice {
        1 => ctx.start_draw(t, 1),
        2 => {
            ctx.recover(t, 1, None);
        }
        3 => {
            ctx.change_hujia(t, 1);
        }
        4 => {
            if ctx.core.player(t).alive {
                ctx.core.player_mut(t).yinshen_active = true;
            }
        }
        5 => {
            let judge = std::mem::take(&mut ctx.core.player_mut(t).judge);
            for &c in &judge {
                ctx.to_discard(c);
                ctx.core
                    .vm_aux
                    .flags
                    .remove(&(t, format!("gw_dieyi_card:{c}")));
            }
            if !judge.is_empty() {
                ctx.note_cards_lost(t, &judge);
                ctx.emit(EventData::CardsLost {
                    player: t,
                    cards: judge.clone(),
                    origins: vec![crate::event::LostOrigin::Judge; judge.len()],
                });
                ctx.log(LogEvent::CardDiscarded {
                    player: t,
                    cards: judge,
                });
            }
        }
        _ => {
            if ctx.core.player(t).linked {
                ctx.core.player_mut(t).linked = false;
                ctx.log(LogEvent::Linked {
                    player: t,
                    linked: false,
                });
            }
            if ctx.core.player(t).face_down {
                ctx.turn_over(t);
            }
        }
    }
}

/// `player.getDebuff()` (`library/element/player.js:9285`) 6 分支精准移植。
pub(in crate::content::host::step) fn apply_debuff(ctx: &mut Ctx<'_>, t: PlayerId) {
    let mut list = vec![1u8, 2, 3, 4, 5, 6];
    let p = ctx.core.player(t);
    if p.hand.is_empty() && p.equips.is_empty() {
        list.retain(|&x| x != 1);
    }
    if p.linked {
        list.retain(|&x| x != 4);
    }
    if super::has_named_skill(ctx.core, ctx.catalog, t, GW_FENGYIN_STATE_KEY) {
        list.retain(|&x| x != 5);
    }
    if p.hp == 1 {
        list.retain(|&x| x != 3);
        if list.len() > 1 {
            list.retain(|&x| x != 2);
        }
    }
    if list.is_empty() {
        return;
    }
    let choice = list[ctx.core.rng.gen_range(list.len() as u32) as usize];
    match choice {
        1 => {
            let mut he: Vec<CardId> = ctx.core.player(t).hand.clone();
            he.extend(ctx.core.player(t).equips.iter().copied());
            if !he.is_empty() {
                let c = he[ctx.core.rng.gen_range(he.len() as u32) as usize];
                discard_he_card(ctx, t, c);
            }
        }
        2 => ctx.emit(EventData::LoseHp { player: t, amount: 1 }),
        3 => ctx.emit(EventData::Damage(crate::machine::damage_data_with(
            None,
            t,
            1,
            crate::event::DamageNature::Normal,
            None,
        ))),
        4 => {
            if !ctx.core.player(t).linked {
                ctx.core.player_mut(t).linked = true;
                ctx.log(LogEvent::Linked {
                    player: t,
                    linked: true,
                });
            }
        }
        5 => apply_fengyin_to_target(ctx, t),
        _ => {
            let opts = super::silver_spells::eligible_delay_options(ctx.core, ctx.catalog, t);
            if let Some(ChoiceOption::Tag(tag)) =
                opts.get(if opts.is_empty() { 0 } else { ctx.core.rng.gen_range(opts.len() as u32) as usize }).copied()
            {
                if let Some(kind) = super::silver_spells::delay_kind_for_tag(tag) {
                    if let Some(c) = pull_card_matching(ctx, |id| ctx.catalog.card_kind(id) == kind) {
                        ctx.core.player_mut(t).judge.push(c);
                        return;
                    }
                }
            }
            list.retain(|&x| x != 6);
            if !list.is_empty() {
                let fallback = list[ctx.core.rng.gen_range(list.len() as u32) as usize];
                match fallback {
                    1 => {
                        let mut he: Vec<CardId> = ctx.core.player(t).hand.clone();
                        he.extend(ctx.core.player(t).equips.iter().copied());
                        if !he.is_empty() {
                            let c = he[ctx.core.rng.gen_range(he.len() as u32) as usize];
                            discard_he_card(ctx, t, c);
                        }
                    }
                    2 => ctx.emit(EventData::LoseHp { player: t, amount: 1 }),
                    3 => ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                        None,
                        t,
                        1,
                        crate::event::DamageNature::Normal,
                        None,
                    ))),
                    4 => {
                        if !ctx.core.player(t).linked {
                            ctx.core.player_mut(t).linked = true;
                            ctx.log(LogEvent::Linked {
                                player: t,
                                linked: true,
                            });
                        }
                    }
                    _ => apply_fengyin_to_target(ctx, t),
                }
            }
        }
    }
}
