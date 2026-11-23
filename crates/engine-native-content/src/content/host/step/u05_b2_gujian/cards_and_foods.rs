use super::helpers::*;
use super::super::*;
use crate::catalog::{CardKind, Catalog, QueryCtx, SkillLimits, TriggerMeta};
use crate::choice::{ChoiceKind, ChoiceOption};
use crate::event::{DetachedKind, EventData, EventKind, Phase, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow, LogEvent};
use crate::state::{GameStateCore, SkillInstance};

const YANJIADAN_VCARDS: [(&str, [CardKind; 3]); 4] = [
    (
        "yanjiadan_heart",
        [CardKind::Yunvyuanshen, CardKind::Ziyangdan, CardKind::Shatang],
    ),
    (
        "yanjiadan_diamond",
        [CardKind::Liufengsan, CardKind::Shujinsan, CardKind::Shenhuofeiya],
    ),
    (
        "yanjiadan_club",
        [CardKind::Bingpotong, CardKind::Liutouge, CardKind::Mianlijinzhen],
    ),
    (
        "yanjiadan_spade",
        [CardKind::Feibiao, CardKind::Qiankunbiao, CardKind::Longxugou],
    ),
];

pub(super) fn yanjiadan_triplet(def_key: &str) -> &'static [CardKind; 3] {
    for &(name, ref trio) in &YANJIADAN_VCARDS {
        if def_key == name || def_key.ends_with(name) {
            return trio;
        }
    }
    &YANJIADAN_VCARDS[0].1
}

pub(super) fn yanjiadan_targets_for(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    kind: CardKind,
) -> (Vec<PlayerId>, usize, usize) {
    let candidates: Vec<PlayerId> = core
        .alive_players()
        .filter(|&t| {
            if !can_target_card(core, catalog, owner, t, kind) {
                return false;
            }
            match kind {
                CardKind::Yunvyuanshen
                | CardKind::Tanhuadong
                | CardKind::Yougeng
                | CardKind::Qingtuan
                | CardKind::Yuanbaorou
                | CardKind::Gudonggeng
                | CardKind::Mapodoufu
                | CardKind::Chunbing
                | CardKind::Liyutang
                | CardKind::Molicha
                | CardKind::Xiajiao
                | CardKind::Mizhilianou
                | CardKind::Luyugeng => {
                    let key = super::mod_food_key(kind);
                    !key.is_some_and(|k| {
                        catalog
                            .skill_by_name(k)
                            .is_some_and(|d| core.skills.contains_key(&(t, d)))
                    })
                }
                CardKind::Ziyangdan | CardKind::Shatang | CardKind::Liutouge | CardKind::Liufengsan => true,
                CardKind::Shenhuofeiya => t != owner,
                CardKind::Shujinsan => !core.player(t).hand.is_empty() || !core.player(t).equips.is_empty(),
                CardKind::Bingpotong => !core.player(t).hand.is_empty(),
                CardKind::Mianlijinzhen => core.player(t).hp >= core.player(owner).hp,
                CardKind::Feibiao => {
                    t != owner
                        && (crate::rvm_bridge::modifiers::eval_card_distance_unlimited(
                            core,
                            catalog,
                            owner,
                            0,
                            CardKind::Feibiao,
                            crate::event::DamageNature::Normal,
                        ) || core.computed_distance(catalog, owner, t) >= 2)
                }
                CardKind::Qiankunbiao => {
                    t != owner && (!core.player(t).hand.is_empty() || !core.player(t).equips.is_empty())
                }
                CardKind::Longxugou => t != owner && !core.player(t).equips.is_empty(),
                _ => false,
            }
        })
        .collect();
    if kind == CardKind::Bingpotong {
        let max = candidates.len().min(3);
        (candidates, 1.min(max), max)
    } else {
        let max = if candidates.is_empty() { 0 } else { 1 };
        (candidates, max, max)
    }
}

pub(in crate::content::host) fn trigger_meta_card_batch(name: &str) -> Option<TriggerMeta> {
    let meta = |event: EventKind| TriggerMeta {
        event,
        timing: Timing::Normal,
        priority: 0,
        order_selectable: false,
        optional: false,
        extra_events: Vec::new(),
    };
    match name {
        "g_shihuifen" => Some(meta(EventKind::PhaseBegin)),
        "xiajiao2" => Some(meta(EventKind::PhaseEnd)),
        _ => None,
    }
}

pub(in crate::content::host) fn limits_card_batch(name: &str) -> SkillLimits {
    match name {
        "gjyuheng_plus_skill" => SkillLimits {
            once_per_turn: true,
            ..Default::default()
        },
        "gjyuheng_pro_skill" => SkillLimits::default(),
        _ => SkillLimits::default(),
    }
}

pub(in crate::content::host) fn activate_card_batch(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<Box<dyn crate::catalog::SkillMachine>> {
    match skill.name.as_str() {
        "gjyuheng_plus_skill" | "gjyuheng_pro_skill" => {
            if !yuheng_can_activate(ctx.state, ctx.catalog, owner, true) {
                return None;
            }
            Some(Box::new(HostMachine {
                def_key: skill.name.clone(),
                behavior: HostBehavior::U05GujianCardBatch,
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

pub(crate) fn yuheng_can_activate(
    state: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    black_cost: bool,
) -> bool {
    let has_cost = state.player(owner).hand.iter().any(|&c| {
        if black_cost {
            is_black(catalog, c)
        } else {
            catalog.cards[c as usize].suit == crate::catalog::Suit::Spade
        }
    });
    if !has_cost {
        return false;
    }
    let enemies = seat_order_enemies(state, catalog, owner);
    enemies.into_iter().any(|e| {
        state
            .player(e)
            .hand
            .iter()
            .any(|&c| catalog.cards[c as usize].suit == crate::catalog::Suit::Spade)
    })
}

pub(in crate::content::host) fn filter_card_batch(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    if !ctx.state.player(owner).alive {
        return false;
    }
    match skill.name.as_str() {
        "g_shihuifen" => match event {
            EventData::Phase {
                player,
                phase: Phase::Play,
                begin: true,
            } => {
                *player != owner
                    && ctx.state.player(*player).alive
                    && !has_skill(ctx.state, ctx.catalog, *player, GJ_FOOD_SHIHUIFEN_BAN_KEY)
                    && !has_skill(ctx.state, ctx.catalog, *player, "shihuifen")
                    && ctx
                        .state
                        .player(owner)
                        .hand
                        .iter()
                        .any(|&c| ctx.catalog.card_kind(c) == CardKind::Shihuifen)
                    && can_target_card(ctx.core_or_state(), ctx.catalog, owner, *player, CardKind::Shihuifen)
            }
            _ => false,
        },
        _ => false,
    }
}

trait QueryCtxStateExt {
    fn core_or_state(&self) -> &GameStateCore;
}
impl QueryCtxStateExt for QueryCtx<'_> {
    fn core_or_state(&self) -> &GameStateCore {
        self.state
    }
}

pub(in crate::content::host) fn machine_card_batch(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    let target = match event {
        EventData::Phase { player, .. } => Some(*player),
        _ => None,
    };
    HostMachine {
        def_key: skill.name.clone(),
        behavior: HostBehavior::U05GujianCardBatch,
        owner,
        target,
        card: None,
        cards: Vec::new(),
        targets: Vec::new(),
        stage: HostStage::Act,
    }
}

pub(super) fn step_card_batch(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if machine.def_key.contains("yanjiadan") {
        return step_yanjiadan(machine, ctx);
    }
    match machine.def_key.as_str() {
        "g_shihuifen" => step_g_shihuifen(machine, ctx),
        "gjyuheng_plus_skill" | "gjyuheng_pro_skill" => {
            step_yuheng_active(machine, ctx, true)
        }
        _ => Flow::Done,
    }
}

fn step_yanjiadan(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let trio = yanjiadan_triplet(&machine.def_key);
    match machine.stage {
        HostStage::Act => {
            let valid: Vec<ChoiceOption> = trio
                .iter()
                .enumerate()
                .filter_map(|(idx, &k)| {
                    let (cands, _, _) = yanjiadan_targets_for(ctx.core, ctx.catalog, machine.owner, k);
                    if cands.is_empty() {
                        None
                    } else {
                        Some(ChoiceOption::Tag(idx as u8))
                    }
                })
                .collect();
            if valid.is_empty() {
                if let Some(c) = machine.card {
                    if ctx.core.table.contains(&c) {
                        ctx.core.table.retain(|x| *x != c);
                        ctx.core.detached_cards.push((c, DetachedKind::Unclaimed));
                    }
                }
                return Flow::Done;
            }
            wait_ask(
                machine,
                ctx,
                HostStage::SelectCard,
                machine.owner,
                ASK_YANJIADAN_VCARD,
                ChoiceSpec {
                    kind: ChoiceKind::PickTag,
                    prompt: "gj_yanjiadan_vcard",
                    options: valid,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            )
        }
        HostStage::SelectCard => {
            let idx = match answer_first_or_pass(ctx) {
                ChoiceOption::Tag(i) if (i as usize) < 3 => i as usize,
                _ => {
                    if let Some(c) = machine.card {
                        ctx.core.table.retain(|x| *x != c);
                        ctx.core.detached_cards.push((c, DetachedKind::Unclaimed));
                    }
                    return Flow::Done;
                }
            };
            let vkind = trio[idx];
            machine.cards = vec![idx as u32];
            let (cands, min, max) = yanjiadan_targets_for(ctx.core, ctx.catalog, machine.owner, vkind);
            if cands.is_empty() || min == 0 {
                if let Some(c) = machine.card {
                    ctx.core.table.retain(|x| *x != c);
                    ctx.core.detached_cards.push((c, DetachedKind::Unclaimed));
                }
                return Flow::Done;
            }
            let opts = cands.into_iter().map(ChoiceOption::Player).collect();
            wait_ask(
                machine,
                ctx,
                HostStage::TargetChoice,
                machine.owner,
                ASK_YANJIADAN_TARGET,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "gj_yanjiadan_target",
                    options: opts,
                    min,
                    max,
                    allow_cancel: false,
                },
            )
        }
        HostStage::TargetChoice => {
            let idx = machine.cards.first().copied().unwrap_or(0) as usize;
            let vkind = trio[idx.min(2)];
            let targets: Vec<PlayerId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Player(p) => Some(*p),
                    _ => None,
                })
                .collect();
            if let Some(c) = machine.card.take() {
                ctx.core.table.retain(|x| *x != c);
                ctx.core.detached_cards.push((c, DetachedKind::Unclaimed));
            }
            if targets.is_empty() {
                return Flow::Done;
            }
            let behavior = match vkind {
                CardKind::Yunvyuanshen | CardKind::Ziyangdan => HostBehavior::GujianFoodGrant,
                CardKind::Shatang => HostBehavior::JiguanShatangCard,
                CardKind::Liufengsan => HostBehavior::GujianLiufengsanCard,
                CardKind::Shujinsan => HostBehavior::GujianShujinsanCard,
                CardKind::Shenhuofeiya => HostBehavior::JiguanShenhuoCard,
                CardKind::Bingpotong => HostBehavior::JiguanBingpoCard,
                CardKind::Liutouge => HostBehavior::JiguanLiutougeCard,
                CardKind::Mianlijinzhen => HostBehavior::GujianMianliCard,
                CardKind::Feibiao => HostBehavior::GujianFeibiaoCard,
                CardKind::Qiankunbiao => HostBehavior::JiguanQiankunCard,
                CardKind::Longxugou => HostBehavior::JiguanLongxuCard,
                _ => return Flow::Done,
            };
            let stage = match behavior {
                HostBehavior::GujianShujinsanCard
                | HostBehavior::GujianFeibiaoCard
                | HostBehavior::JiguanLongxuCard => HostStage::SelectCard,
                HostBehavior::JiguanBingpoCard => HostStage::ComparePlayerPick,
                _ => HostStage::Act,
            };
            let fake_card_key = match vkind {
                CardKind::Yunvyuanshen => GJ_YUNVYUANSHEN_CARD_KEY,
                CardKind::Ziyangdan => GJ_ZIYANGDAN_CARD_KEY,
                _ => machine.def_key.as_str(),
            };
            if vkind == CardKind::Yunvyuanshen || vkind == CardKind::Ziyangdan {
                let (listener, rounds) = if vkind == CardKind::Yunvyuanshen {
                    (GJ_YUNVYUANSHEN_LISTENER_KEY, GJ_FOOD_ROUNDS_DEFAULT)
                } else {
                    (GJ_FOOD_ZIYANGDAN_KEY, GJ_FOOD_ROUNDS_DEFAULT)
                };
                if let Some(t) = targets.first().copied() {
                    if let Some(def) = ctx.catalog.skill_by_name(listener) {
                        if vkind == CardKind::Ziyangdan {
                            if let Some(inst) = ctx.core.skills.get_mut(&(t, def)) {
                                inst.rounds = inst.rounds.saturating_add(rounds);
                                inst.enabled = true;
                            } else {
                                ctx.insert_skill(
                                    (t, def),
                                    SkillInstance {
                                        owner: t,
                                        enabled: true,
                                        once_used: false,
                                        expires_end_of_turn: false,
                                        rounds,
                                    },
                                );
                            }
                        } else {
                            ctx.insert_skill(
                                (t, def),
                                SkillInstance {
                                    owner: t,
                                    enabled: true,
                                    once_used: false,
                                    expires_end_of_turn: false,
                                    rounds,
                                },
                            );
                        }
                    }
                    if vkind == CardKind::Ziyangdan {
                        ctx.change_hujia(t, GJ_ZIYANGDAN_HUJIA as i32);
                    }
                }
                let _ = fake_card_key;
                return Flow::Done;
            }
            let spawned_targets = if vkind == CardKind::Shenhuofeiya {
                let t = targets[0];
                let mut all = vec![t];
                for p in ctx.core.seat_order_from(t) {
                    if p != t
                        && ctx.core.player(p).alive
                        && ctx.core.seat_distance_alive(t, p) == 1
                    {
                        all.push(p);
                    }
                }
                all
            } else {
                targets.clone()
            };
            ctx.res.parks.push(crate::resolution::Park {
                resume_depth: ctx.res.stack.len(),
                slot: crate::resolution::Slot::Intrinsic {
                    frame: crate::resolution::NO_FRAME,
                },
                awaiting_answer: false,
                machine: Box::new(HostMachine {
                    def_key: behavior.key().to_string(),
                    behavior,
                    owner: machine.owner,
                    target: targets.first().copied(),
                    card: None,
                    cards: Vec::new(),
                    targets: spawned_targets,
                    stage,
                }),
            });
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_g_shihuifen(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let Some(t) = machine.target else {
        return Flow::Done;
    };
    match machine.stage {
        HostStage::Act => {
            if !ctx.core.player(t).alive || !ctx.core.player(machine.owner).alive {
                return Flow::Done;
            }
            let mut opts: Vec<ChoiceOption> = vec![ChoiceOption::Pass];
            opts.extend(
                ctx.core
                    .player(machine.owner)
                    .hand
                    .iter()
                    .copied()
                    .filter(|&c| ctx.catalog.card_kind(c) == CardKind::Shihuifen)
                    .map(ChoiceOption::Card),
            );
            if opts.len() <= 1 {
                return Flow::Done;
            }
            wait_ask(
                machine,
                ctx,
                HostStage::SelectCard,
                machine.owner,
                ASK_G_SHIHUIFEN_USE,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "gj_shihuifen_use",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            )
        }
        HostStage::SelectCard => {
            let ChoiceOption::Card(cid) = answer_first_or_pass(ctx) else {
                return Flow::Done;
            };
            if !ctx.core.player(machine.owner).hand.contains(&cid)
                || ctx.catalog.card_kind(cid) != CardKind::Shihuifen
            {
                return Flow::Done;
            }
            ctx.core.player_mut(machine.owner).hand.retain(|x| *x != cid);
            ctx.core.table.push(cid);
            ctx.log(LogEvent::CardPlayed {
                card: cid,
                player: machine.owner,
                targets: vec![t],
                presentation: None,
            });
            ctx.res.parks.push(crate::resolution::Park {
                resume_depth: ctx.res.stack.len(),
                slot: crate::resolution::Slot::Intrinsic {
                    frame: crate::resolution::NO_FRAME,
                },
                awaiting_answer: false,
                machine: Box::new(HostMachine {
                    def_key: GJ_SHIHUIFEN_CARD_KEY.to_string(),
                    behavior: HostBehavior::GujianShihuifenCard,
                    owner: machine.owner,
                    target: Some(t),
                    card: Some(cid),
                    cards: Vec::new(),
                    targets: vec![t],
                    stage: HostStage::Act,
                }),
            });
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(crate) fn step_yuheng_active(
    machine: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    force_black: bool,
) -> Flow {
    let eq_card = find_yuheng_equip_card(ctx.core, ctx.catalog, machine.owner);
    let tier = eq_card
        .map(|c| yuheng_tier(ctx.core, ctx.catalog, c))
        .unwrap_or(0);
    let allow_black = force_black || tier >= 1;
    match machine.stage {
        HostStage::Act => {
            let opts: Vec<ChoiceOption> = ctx
                .core
                .player(machine.owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| {
                    if allow_black {
                        is_black(ctx.catalog, c)
                    } else {
                        ctx.catalog.cards[c as usize].suit == crate::catalog::Suit::Spade
                    }
                })
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
                ASK_YUHENG_COST,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "gj_yuheng_cost",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            )
        }
        HostStage::SelectCard => {
            let ChoiceOption::Card(cost) = answer_first_or_pass(ctx) else {
                // Refund once_used when cancelled
                if let Some(def) = ctx.catalog.skill_by_name(&machine.def_key) {
                    if let Some(inst) = ctx.core.skills.get_mut(&(machine.owner, def)) {
                        inst.once_used = false;
                    }
                }
                return Flow::Done;
            };
            ctx.discard_from_hand(machine.owner, cost);
            ctx.log(LogEvent::CardDiscarded {
                player: machine.owner,
                cards: vec![cost],
            });
            let enemies = seat_order_enemies(ctx.core, ctx.catalog, machine.owner);
            let mut cands: Vec<(PlayerId, Vec<CardId>)> = Vec::new();
            for e in enemies {
                let sp: Vec<CardId> = ctx
                    .core
                    .player(e)
                    .hand
                    .iter()
                    .copied()
                    .filter(|&c| ctx.catalog.cards[c as usize].suit == crate::catalog::Suit::Spade)
                    .collect();
                if !sp.is_empty() {
                    cands.push((e, sp));
                }
            }
            if !cands.is_empty() {
                let ei = ctx.core.rng.gen_range(cands.len() as u32) as usize;
                let (victim, spades) = &cands[ei];
                let si = ctx.core.rng.gen_range(spades.len() as u32) as usize;
                let stolen = spades[si];
                ctx.core.player_mut(*victim).hand.retain(|x| *x != stolen);
                ctx.core.player_mut(machine.owner).hand.push(stolen);
                ctx.log(LogEvent::CardsGiven {
                    from: *victim,
                    to: machine.owner,
                    cards: vec![stolen],
                });
            }
            // Increment uses & upgrade gjyuheng -> gjyuheng_plus (>=3) -> gjyuheng_pro (>=7)
            if let Some(old_card) = eq_card {
                let uses_key = format!("gj_yuheng_uses:{old_card}");
                let uses = flag(ctx.core, 0, &uses_key) + 1;
                set_flag(ctx.core, 0, &uses_key, uses);
                let target_prefix = if uses >= 7 {
                    Some(("gjyuheng_pro", 2u32))
                } else if uses >= 3 {
                    Some(("gjyuheng_plus", 1u32))
                } else {
                    None
                };
                if let Some((new_name, new_tier)) = target_prefix {
                    let cur_tier = yuheng_tier(ctx.core, ctx.catalog, old_card);
                    if new_tier > u32::from(cur_tier) {
                        if let Some(pos) = ctx
                            .core
                            .detached_cards
                            .iter()
                            .position(|&(c, _)| ctx.catalog.cards[c as usize].name.starts_with(new_name))
                        {
                            let new_card = ctx.core.detached_cards.remove(pos).0;
                            ctx.core.player_mut(machine.owner).equips.retain(|&c| c != old_card);
                            crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, machine.owner, old_card);
                            ctx.core.detached_cards.push((old_card, DetachedKind::Unclaimed));
                            ctx.core.player_mut(machine.owner).equips.push(new_card);
                            set_flag(ctx.core, 0, &format!("gj_yuheng_uses:{new_card}"), uses);
                            set_flag(ctx.core, 0, &format!("gj_yuheng_tier:{new_card}"), new_tier);
                            crate::kernel::grant_equip_skills(ctx, machine.owner, new_card);
                            if new_tier >= 2 {
                                if let Some(d) = ctx.catalog.skill_by_name("gjyuheng_pro_skill") {
                                    ctx.insert_skill(
                                        (machine.owner, d),
                                        SkillInstance {
                                            owner: machine.owner,
                                            enabled: true,
                                            once_used: false,
                                            expires_end_of_turn: false,
                                            rounds: 0,
                                        },
                                    );
                                }
                            } else if let Some(d) = ctx.catalog.skill_by_name("gjyuheng_plus_skill") {
                                ctx.insert_skill(
                                    (machine.owner, d),
                                    SkillInstance {
                                        owner: machine.owner,
                                        enabled: true,
                                        once_used: false,
                                        expires_end_of_turn: false,
                                        rounds: 0,
                                    },
                                );
                            }
                        } else {
                            set_flag(ctx.core, 0, &format!("gj_yuheng_tier:{old_card}"), new_tier);
                            if new_tier >= 2 {
                                if let Some(def) = ctx.catalog.skill_by_name(&machine.def_key) {
                                    if let Some(inst) = ctx.core.skills.get_mut(&(machine.owner, def)) {
                                        inst.once_used = false;
                                    }
                                }
                            }
                        }
                    }
                } else if tier >= 2 {
                    if let Some(def) = ctx.catalog.skill_by_name(&machine.def_key) {
                        if let Some(inst) = ctx.core.skills.get_mut(&(machine.owner, def)) {
                            inst.once_used = false;
                        }
                    }
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}
