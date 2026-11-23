use super::helpers::*;
use super::super::*;
use crate::catalog::{CardKind, Catalog, QueryCtx, SkillLimits, TriggerMeta};
use crate::choice::{ChoiceKind, ChoiceOption};
use crate::event::{DamageNature, EventData, EventKind, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow};
use crate::state::{GameStateCore, SkillInstance};

pub(super) const ASK_ZHENYING_DISCARD: u32 = 551;
pub(super) const ASK_QINGSHU_VCARD: u32 = 553;
pub(super) const ASK_QINGSHU_TARGET: u32 = 554;
pub(super) const ASK_YUNYOU_DISCOVER: u32 = 555;
pub(super) const ASK_XUANZHEN_CONFIRM: u32 = 556;
pub(super) const ASK_XUANZHEN_VCARD: u32 = 557;

const SHA_RELATED_POOL: [&str; 10] = [
    "paoxiao",
    "wushuang",
    "wusheng",
    "tieji",
    "jianwu",
    "zuizhan",
    "liuying",
    "xuanci",
    "kuanggu",
    "mashu",
];

const XUANZHEN_POOL: [CardKind; 42] = [
    CardKind::Wuzhong,
    CardKind::Shujinsan,
    CardKind::Liufengsan,
    CardKind::Ziyangdan,
    CardKind::Yunvyuanshen,
    CardKind::Tao,
    CardKind::Luyugeng,
    CardKind::Chunbing,
    CardKind::Gudonggeng,
    CardKind::Liyutang,
    CardKind::Mizhilianou,
    CardKind::Xiajiao,
    CardKind::Tanhuadong,
    CardKind::Mapodoufu,
    CardKind::Qingtuan,
    CardKind::Yougeng,
    CardKind::Molicha,
    CardKind::Yuanbaorou,
    CardKind::Liutouge,
    CardKind::Armor,
    CardKind::HorsePlus,
    CardKind::HorseMinus,
    CardKind::Weapon,
    CardKind::Sha,
    CardKind::FireSha,
    CardKind::LeiSha,
    CardKind::Dismantle,
    CardKind::Snatch,
    CardKind::Duel,
    CardKind::Chain,
    CardKind::Le,
    CardKind::Bing,
    CardKind::Linghunzhihuo,
    CardKind::ShouLiJian,
    CardKind::Shatang,
    CardKind::Shihuifen,
    CardKind::Feibiao,
    CardKind::Longxugou,
    CardKind::Qiankunbiao,
    CardKind::Shenhuofeiya,
    CardKind::Mianlijinzhen,
    CardKind::Bingpotong,
];

fn xuanzhen_card_eligible(
    core: &GameStateCore,
    catalog: &Catalog,
    attacker: PlayerId,
    owner: PlayerId,
    kind: CardKind,
) -> bool {
    if !catalog.cards.iter().any(|c| !c.off_deck && c.kind == kind) {
        return false;
    }
    if !can_target_card(core, catalog, attacker, owner, kind) {
        return false;
    }
    let owner_p = core.player(owner);
    match kind {
        CardKind::Tao => owner_p.hp < owner_p.max_hp as i32,
        CardKind::Dismantle => {
            !owner_p.hand.is_empty() || !owner_p.equips.is_empty() || !owner_p.judge.is_empty()
        }
        CardKind::Snatch => {
            core.computed_distance(catalog, attacker, owner) <= 1
                && (!owner_p.hand.is_empty()
                    || !owner_p.equips.is_empty()
                    || !owner_p.judge.is_empty())
        }
        CardKind::Bing => {
            core.computed_distance(catalog, attacker, owner) <= 1
                && !owner_p
                    .judge
                    .iter()
                    .any(|&c| catalog.card_kind(c) == CardKind::Bing)
        }
        CardKind::Le => !owner_p
            .judge
            .iter()
            .any(|&c| catalog.card_kind(c) == CardKind::Le),
        CardKind::Shihuifen => attacker != owner,
        CardKind::Shujinsan | CardKind::Qiankunbiao => {
            !owner_p.hand.is_empty() || !owner_p.equips.is_empty()
        }
        CardKind::Longxugou => !owner_p.equips.is_empty(),
        CardKind::Bingpotong => !owner_p.hand.is_empty() && !core.player(attacker).hand.is_empty(),
        CardKind::Mianlijinzhen => owner_p.hp >= core.player(attacker).hp,
        CardKind::Feibiao => {
            crate::rvm_bridge::modifiers::eval_card_distance_unlimited(
                core,
                catalog,
                attacker,
                0,
                CardKind::Feibiao,
                DamageNature::Normal,
            ) || core.computed_distance(catalog, attacker, owner) >= 2
        }
        CardKind::Luyugeng
        | CardKind::Chunbing
        | CardKind::Gudonggeng
        | CardKind::Liyutang
        | CardKind::Mizhilianou
        | CardKind::Xiajiao
        | CardKind::Tanhuadong
        | CardKind::Mapodoufu
        | CardKind::Qingtuan
        | CardKind::Yougeng
        | CardKind::Molicha
        | CardKind::Yuanbaorou => food_listener_key(kind)
            .is_some_and(|k| !has_skill(core, catalog, owner, k)),
        _ => true,
    }
}

pub(in crate::content::host) fn trigger_meta_char_e2(name: &str) -> Option<TriggerMeta> {
    let meta = |event: EventKind, timing: Timing, priority: i32, extra_events: Vec<EventKind>| TriggerMeta {
        event,
        timing,
        priority,
        order_selectable: false,
        optional: false,
        extra_events,
    };
    match name {
        "zhenying" => Some(meta(
            EventKind::CardUsed,
            Timing::Normal,
            0,
            vec![EventKind::CardResponded, EventKind::TurnPhaseAfter],
        )),
        "lingnu" => Some(meta(
            EventKind::DamageSource,
            Timing::Normal,
            0,
            vec![EventKind::TurnPhaseAfter],
        )),
        "shahun" => Some(meta(EventKind::Dying, Timing::Normal, 0, Vec::new())),
        "shahun2" => Some(meta(EventKind::TurnPhaseAfter, Timing::Normal, 0, Vec::new())),
        "qingshu" => Some(meta(EventKind::TurnPhaseEnd, Timing::Normal, 0, Vec::new())),
        "yunyou" => Some(meta(EventKind::PhaseBegin, Timing::Normal, 0, Vec::new())),
        "xuanzhen" => Some(meta(EventKind::CardUsed, Timing::First, 10, Vec::new())),
        _ => None,
    }
}

pub(in crate::content::host) fn limits_char_e2(_name: &str) -> SkillLimits {
    SkillLimits::default()
}

pub(in crate::content::host) fn activate_char_e2(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<Box<dyn crate::catalog::SkillMachine>> {
    match skill.name.as_str() {
        "yunyou" => {
            if ctx.state.turn.round < flag(ctx.state, owner, "gj_yunyou_ready_round")
                || ctx.state.turn.land_used
                || yunyou_unused_lands(ctx.state, owner).is_empty()
            {
                return None;
            }
            Some(Box::new(HostMachine {
                def_key: skill.name.clone(),
                behavior: HostBehavior::U05GujianCharBatchE,
                owner,
                target: None,
                card: None,
                cards: Vec::new(),
                targets: Vec::new(),
                stage: HostStage::Act,
            }))
        }
        _ => None,
    }
}

pub(in crate::content::host) fn filter_char_e2(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    if !ctx.state.player(owner).alive {
        return false;
    }
    match skill.name.as_str() {
        "zhenying" => match event {
            EventData::CardUsed(u) if u.player == owner && !u.cancelled => {
                if u.use_as.is_some() {
                    return false;
                }
                let Some(cid) = u.card else {
                    return false;
                };
                if flag(ctx.state, owner, &format!("gj_zhenying_card:{cid}")) > 0 {
                    return false;
                }
                let kind = ctx.catalog.card_kind(cid);
                is_black(ctx.catalog, cid)
                    && kind.equip_slot().is_none()
                    && !matches!(kind, CardKind::Le | CardKind::Bing | CardKind::ShanDian)
            }
            EventData::CardResponded { player, card } if *player == owner => {
                if flag(ctx.state, owner, &format!("gj_zhenying_card:{card}")) > 0 {
                    return true;
                }
                let kind = ctx.catalog.card_kind(*card);
                is_black(ctx.catalog, *card)
                    && kind.equip_slot().is_none()
                    && !matches!(kind, CardKind::Le | CardKind::Bing | CardKind::ShanDian)
            }
            EventData::TurnEndWindow { after: true, .. } => ctx
                .state
                .player(owner)
                .hand
                .iter()
                .any(|&c| flag(ctx.state, owner, &format!("gj_zhenying_card:{c}")) > 0),
            _ => false,
        },
        "lingnu" => match event {
            EventData::Damage(d) => {
                d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && lingnu_slot_count(ctx.state, owner) < 3
            }
            EventData::TurnEndWindow { player, after: true } if *player == owner => {
                lingnu_slot_count(ctx.state, owner) > 0
            }
            _ => false,
        },
        "shahun2" => {
            matches!(event, EventData::TurnEndWindow { player, after: true } if *player == owner)
                && flag(ctx.state, owner, "gj_shahun_timer") > 0
        }
        "qingshu" => {
            matches!(event, EventData::TurnEndWindow { player, after: false } if *player == owner)
                && !qingshu_available_lands(ctx.state, owner).is_empty()
        }
        "xuanzhen" => match event {
            EventData::CardUsed(u)
                if !u.cancelled
                    && u.player != owner
                    && u.targets.len() == 1
                    && u.targets[0] == owner =>
            {
                ctx.state.turn.round >= flag(ctx.state, owner, "gj_xuanzhen_ready_round")
            }
            _ => false,
        },
        _ => false,
    }
}

pub(super) fn yunyou_unused_lands(core: &GameStateCore, owner: PlayerId) -> Vec<CardKind> {
    LAND_KINDS
        .iter()
        .copied()
        .filter(|&k| {
            let idx = land_kind_to_idx(k).unwrap_or(0);
            flag(core, owner, &format!("gj_yunyou_used:{idx}")) == 0
        })
        .collect()
}

pub(super) fn qingshu_available_lands(core: &GameStateCore, owner: PlayerId) -> Vec<CardKind> {
    LAND_KINDS
        .iter()
        .copied()
        .filter(|&k| {
            let idx = land_kind_to_idx(k).unwrap_or(0);
            flag(core, owner, &format!("gj_qingshu_pool:{idx}")) > 0 && core.current_land != Some(k)
        })
        .collect()
}

pub(super) fn lingnu_slot_count(core: &GameStateCore, owner: PlayerId) -> usize {
    (0..3)
        .filter(|&s| flag(core, owner, &format!("gj_lingnu_def:{s}")) > 0)
        .count()
}

pub(super) fn shahun_can_save(core: &GameStateCore, owner: PlayerId) -> bool {
    core.player(owner).hp <= 0 && flag(core, owner, "gj_shahun_used") == 0
}

pub(super) fn step_char_e2(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match machine.def_key.as_str() {
        "zhenying" => step_zhenying(machine, ctx),
        "lingnu" => step_lingnu(machine, ctx),
        "shahun" => step_shahun(machine, ctx),
        "shahun2" => step_shahun2(machine, ctx),
        "qingshu" => step_qingshu(machine, ctx),
        "yunyou" => step_yunyou(machine, ctx),
        "xuanzhen" => step_xuanzhen(machine, ctx),
        _ => Flow::Done,
    }
}

fn step_zhenying(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if matches!(machine.stage, HostStage::Act) && machine.card.is_none() {
        let to_drop: Vec<CardId> = ctx
            .core
            .player(machine.owner)
            .hand
            .iter()
            .copied()
            .filter(|&c| flag(ctx.core, machine.owner, &format!("gj_zhenying_card:{c}")) > 0)
            .collect();
        for c in to_drop {
            ctx.core.player_mut(machine.owner).hand.retain(|x| *x != c);
            set_flag(ctx.core, machine.owner, &format!("gj_zhenying_card:{c}"), 0);
            ctx.to_discard(c);
        }
        return Flow::Done;
    }
    if matches!(machine.stage, HostStage::Act) {
        if let Some(cid) = machine.card {
            if cid != u32::MAX {
                let zkey = format!("gj_zhenying_card:{cid}");
                if flag(ctx.core, machine.owner, &zkey) > 0 {
                    set_flag(ctx.core, machine.owner, &zkey, 0);
                    machine.card = Some(u32::MAX);
                }
            }
        }
    }
    if machine.card == Some(u32::MAX) {
        // Triggered on using/responding with a zhenying_link card -> forced discard 1 "he"
        match machine.stage {
            HostStage::Act => {
                let opts: Vec<ChoiceOption> = hand_and_equips(ctx.core, machine.owner)
                    .into_iter()
                    .map(ChoiceOption::Card)
                    .collect();
                if opts.is_empty() {
                    return Flow::Done;
                }
                wait_ask(
                    machine,
                    ctx,
                    HostStage::SelectCard,
                    machine.owner,
                    ASK_ZHENYING_DISCARD,
                    ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "gj_zhenying_discard",
                        options: opts,
                        min: 1,
                        max: 1,
                        allow_cancel: false,
                    },
                )
            }
            HostStage::SelectCard => {
                if let ChoiceOption::Card(cid) = answer_first_or_pass(ctx) {
                    discard_card_from_he(ctx, machine.owner, cid);
                }
                Flow::Done
            }
            _ => Flow::Done,
        }
    } else {
        let Some(cid) = machine.card else {
            return Flow::Done;
        };
        let kind = ctx.catalog.card_kind(cid);
        let match_pos = |pool: &[CardId]| {
            pool.iter()
                .rposition(|&c| c != cid && ctx.catalog.card_kind(c) == kind && is_black(ctx.catalog, c))
                .or_else(|| {
                    pool.iter().rposition(|&c| {
                        c != cid
                            && ctx.catalog.card_kind(c) == kind
                    })
                })
        };
        let copy = if let Some(i) = match_pos(&ctx.core.deck) {
            Some(ctx.core.deck.remove(i))
        } else if let Some(i) = match_pos(&ctx.core.discard) {
            Some(ctx.core.discard.remove(i))
        } else {
            None
        };
        if let Some(fk) = copy {
            set_flag(ctx.core, machine.owner, &format!("gj_zhenying_card:{fk}"), 1);
            ctx.core.player_mut(machine.owner).hand.push(fk);
            ctx.emit(EventData::CardsDrawn {
                player: machine.owner,
                cards: vec![fk],
            });
        }
        Flow::Done
    }
}

fn step_lingnu(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = machine.owner;
    if machine.card.is_none() {
        // TurnEndWindow { player: owner, after: true } decrement rounds
        for s in 0..3 {
            let def_key = format!("gj_lingnu_def:{s}");
            let rem_key = format!("gj_lingnu_rem:{s}");
            let def_plus = flag(ctx.core, owner, &def_key);
            if def_plus == 0 {
                continue;
            }
            let rem = flag(ctx.core, owner, &rem_key).saturating_sub(1);
            if rem == 0 {
                let def = (def_plus - 1) as u16;
                ctx.core.skills.remove(&(owner, def));
                set_flag(ctx.core, owner, &def_key, 0);
                set_flag(ctx.core, owner, &rem_key, 0);
            } else {
                set_flag(ctx.core, owner, &rem_key, rem);
            }
        }
        return Flow::Done;
    }
    let dmg = machine.card.unwrap_or(1) as usize;
    for _ in 0..dmg {
        if lingnu_slot_count(ctx.core, owner) >= 3 {
            break;
        }
        let cands: Vec<u16> = SHA_RELATED_POOL
            .iter()
            .filter_map(|&sk| ctx.catalog.skill_by_name(sk))
            .filter(|&d| !ctx.core.skills.contains_key(&(owner, d)))
            .collect();
        if cands.is_empty() {
            break;
        }
        let pick = ctx.core.rng.gen_range(cands.len() as u32) as usize;
        let def = cands[pick];
        for s in 0..3 {
            let def_key = format!("gj_lingnu_def:{s}");
            if flag(ctx.core, owner, &def_key) == 0 {
                set_flag(ctx.core, owner, &def_key, u32::from(def) + 1);
                set_flag(ctx.core, owner, &format!("gj_lingnu_rem:{s}"), 3);
                ctx.insert_skill(
                    (owner, def),
                    SkillInstance {
                        owner,
                        enabled: true,
                        once_used: false,
                        expires_end_of_turn: false,
                        rounds: 3,
                    },
                );
                break;
            }
        }
    }
    Flow::Done
}

pub(super) fn step_shahun(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = machine.owner;
    if !shahun_can_save(ctx.core, owner) {
        return Flow::Done;
    }
    set_flag(ctx.core, owner, "gj_shahun_used", 1);
    set_flag(ctx.core, owner, "gj_shahun_timer", 2);
    let hand: Vec<CardId> = ctx.core.player_mut(owner).hand.drain(..).collect();
    let equips: Vec<CardId> = ctx.core.player_mut(owner).equips.drain(..).collect();
    let judge: Vec<CardId> = ctx.core.player_mut(owner).judge.drain(..).collect();
    for &e in &equips {
        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, owner, e);
    }
    for &c in hand.iter().chain(equips.iter()).chain(judge.iter()) {
        ctx.to_discard(c);
    }
    ctx.core.player_mut(owner).linked = false;
    ctx.core.player_mut(owner).face_down = false;
    ctx.start_draw(owner, 3);
    let need = (1 - ctx.core.player(owner).hp).max(0) as u32;
    if need > 0 {
        ctx.recover(owner, need, None);
    }
    if let Some(def) = ctx.catalog.skill_by_name("fanshi") {
        ctx.core.skills.remove(&(owner, def));
    }
    set_flag(ctx.core, owner, "u05_mod:gj_juejing", 1);
    for sk in ["juejing", "shahun2"] {
        if let Some(def) = ctx.catalog.skill_by_name(sk) {
            ctx.insert_skill(
                (owner, def),
                SkillInstance {
                    owner,
                    enabled: true,
                    once_used: false,
                    expires_end_of_turn: false,
                    rounds: 0,
                },
            );
        }
    }
    Flow::Done
}

fn step_shahun2(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let t = flag(ctx.core, machine.owner, "gj_shahun_timer");
    if t > 1 {
        set_flag(ctx.core, machine.owner, "gj_shahun_timer", t - 1);
    } else if t == 1 {
        set_flag(ctx.core, machine.owner, "gj_shahun_timer", 0);
        die_player(ctx, machine.owner, None);
    }
    Flow::Done
}

fn step_qingshu(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match machine.stage {
        HostStage::Act => {
            let lands = qingshu_available_lands(ctx.core, machine.owner);
            if lands.is_empty() {
                return Flow::Done;
            }
            let opts: Vec<ChoiceOption> = lands
                .into_iter()
                .filter_map(|k| land_kind_to_idx(k).map(|idx| ChoiceOption::Tag(idx as u8)))
                .collect();
            wait_ask(
                machine,
                ctx,
                HostStage::SelectCard,
                machine.owner,
                ASK_QINGSHU_VCARD,
                ChoiceSpec {
                    kind: ChoiceKind::PickTag,
                    prompt: "gj_qingshu_vcard",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            )
        }
        HostStage::SelectCard => {
            let ChoiceOption::Tag(idx) = answer_first_or_pass(ctx) else {
                return Flow::Done;
            };
            let Some(kind) = idx_to_land_kind(u32::from(idx)) else {
                return Flow::Done;
            };
            machine.card = Some(u32::from(idx));
            let keys = land_skill_keys(kind);
            let opts: Vec<ChoiceOption> = ctx
                .core
                .alive_players()
                .filter(|&p| {
                    keys.first()
                        .is_some_and(|k| !has_skill(ctx.core, ctx.catalog, p, k))
                })
                .map(ChoiceOption::Player)
                .collect();
            if opts.is_empty() {
                return Flow::Done;
            }
            wait_ask(
                machine,
                ctx,
                HostStage::TargetChoice,
                machine.owner,
                ASK_QINGSHU_TARGET,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "gj_qingshu_target",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            )
        }
        HostStage::TargetChoice => {
            let ChoiceOption::Player(t) = answer_first_or_pass(ctx) else {
                return Flow::Done;
            };
            let Some(idx) = machine.card else {
                return Flow::Done;
            };
            let Some(kind) = idx_to_land_kind(idx) else {
                return Flow::Done;
            };
            set_flag(ctx.core, machine.owner, &format!("gj_qingshu_pool:{idx}"), 0);
            for &sk in land_skill_keys(kind) {
                if let Some(def) = ctx.catalog.skill_by_name(sk) {
                    ctx.insert_skill(
                        (t, def),
                        SkillInstance {
                            owner: t,
                            enabled: true,
                            once_used: false,
                            expires_end_of_turn: false,
                            rounds: 0,
                        },
                    );
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_yunyou(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match machine.stage {
        HostStage::Act => {
            let mut pool = yunyou_unused_lands(ctx.core, machine.owner);
            if pool.is_empty() {
                return Flow::Done;
            }
            let mut choices = Vec::new();
            for _ in 0..3 {
                if pool.is_empty() {
                    break;
                }
                let i = ctx.core.rng.gen_range(pool.len() as u32) as usize;
                let k = pool.remove(i);
                if let Some(idx) = land_kind_to_idx(k) {
                    choices.push(idx as u8);
                }
            }
            if choices.is_empty() {
                return Flow::Done;
            }
            machine.cards = choices.iter().copied().map(u32::from).collect();
            wait_ask(
                machine,
                ctx,
                HostStage::SelectCard,
                machine.owner,
                ASK_YUNYOU_DISCOVER,
                ChoiceSpec {
                    kind: ChoiceKind::PickTag,
                    prompt: "gj_yunyou_discover",
                    options: choices.into_iter().map(ChoiceOption::Tag).collect(),
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            )
        }
        HostStage::SelectCard => {
            let ChoiceOption::Tag(idx) = answer_first_or_pass(ctx) else {
                return Flow::Done;
            };
            let Some(kind) = idx_to_land_kind(u32::from(idx)) else {
                return Flow::Done;
            };
            set_flag(
                ctx.core,
                machine.owner,
                "gj_yunyou_ready_round",
                ctx.core.turn.round + 2,
            );
            ctx.core.turn.land_used = true;
            set_flag(ctx.core, machine.owner, &format!("gj_yunyou_used:{idx}"), 1);
            if flag(ctx.core, machine.owner, &format!("gj_qingshu_all:{idx}")) == 0 {
                set_flag(ctx.core, machine.owner, &format!("gj_qingshu_all:{idx}"), 1);
                set_flag(ctx.core, machine.owner, &format!("gj_qingshu_pool:{idx}"), 1);
            }
            sync_land_skill(ctx, machine.owner, kind);
            ctx.core.current_land = Some(kind);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_xuanzhen(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match machine.stage {
        HostStage::Act => wait_ask(
            machine,
            ctx,
            HostStage::SelectCard,
            machine.owner,
            ASK_XUANZHEN_CONFIRM,
            ChoiceSpec {
                kind: ChoiceKind::Confirm,
                prompt: "gj_xuanzhen_confirm",
                options: vec![ChoiceOption::Confirm, ChoiceOption::Pass],
                min: 1,
                max: 1,
                allow_cancel: false,
            },
        ),
        HostStage::SelectCard => {
            if !matches!(answer_first_or_pass(ctx), ChoiceOption::Confirm) {
                return Flow::Done;
            }
            let Ok(attacker) = ctx.current_card_use().map(|u| u.player) else {
                return Flow::Done;
            };
            let mut list1: Vec<u32> = Vec::new();
            let mut list2: Vec<u32> = Vec::new();
            for (i, &k) in XUANZHEN_POOL.iter().enumerate() {
                if xuanzhen_card_eligible(ctx.core, ctx.catalog, attacker, machine.owner, k) {
                    list1.push(i as u32);
                    if k.equip_slot().is_none() {
                        list2.push(i as u32);
                    }
                }
            }
            let mut discovered: Vec<u32> = Vec::new();
            let mut equipped = false;
            for _ in 0..3 {
                if equipped && !list2.is_empty() {
                    let pos = ctx.core.rng.gen_range(list2.len() as u32) as usize;
                    let pick = list2.remove(pos);
                    list1.retain(|&x| x != pick);
                    discovered.push(pick);
                } else if !list1.is_empty() {
                    let pos = ctx.core.rng.gen_range(list1.len() as u32) as usize;
                    let pick = list1.remove(pos);
                    list2.retain(|&x| x != pick);
                    if XUANZHEN_POOL[pick as usize].equip_slot().is_some() {
                        equipped = true;
                    }
                    discovered.push(pick);
                }
            }
            if discovered.is_empty() {
                return Flow::Done;
            }
            machine.cards = discovered;
            let opts: Vec<ChoiceOption> = machine
                .cards
                .iter()
                .copied()
                .map(|v| ChoiceOption::Tag(v as u8))
                .collect();
            wait_ask(
                machine,
                ctx,
                HostStage::TargetChoice,
                machine.owner,
                ASK_XUANZHEN_VCARD,
                ChoiceSpec {
                    kind: ChoiceKind::PickTag,
                    prompt: "gj_xuanzhen_vcard",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            )
        }
        HostStage::TargetChoice => {
            let ChoiceOption::Tag(idx) = answer_first_or_pass(ctx) else {
                return Flow::Done;
            };
            let kind = XUANZHEN_POOL.get(idx as usize).copied().unwrap_or(CardKind::Wuzhong);
            set_flag(
                ctx.core,
                machine.owner,
                "gj_xuanzhen_ready_round",
                ctx.core.turn.round + 1,
            );
            let u = ctx.current_card_use_mut();
            u.use_as = Some(kind);
            if matches!(kind, CardKind::FireSha | CardKind::LeiSha) {
                u.nature = if kind == CardKind::FireSha {
                    DamageNature::Fire
                } else {
                    DamageNature::Thunder
                };
            } else {
                u.nature = DamageNature::Normal;
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xuanzhen_pool_excludes_dinvxuanshuang() {
        assert!(!XUANZHEN_POOL.contains(&CardKind::Dinvxuanshuang));
    }
}
