//! Batch Q4 U02/L03 boss/jiange 技能家族（tag 870）。

use super::super::*;
use crate::catalog::{CardKind, Catalog, QueryCtx, SkillLimits, SkillMachine, TriggerMeta};
use crate::choice::ChoiceOption;
use crate::event::{DamageNature, EventData, EventKind, Phase, Timing};
use crate::ids::{CardId, PlayerId, SkillDefId};
use crate::machine::{damage_data_with, Ctx, Flow, LogEvent};
use crate::state::GameStateCore;

#[path = "q4_boss_asks.rs"]
pub(in crate::content::host) mod q4_boss_asks;
#[path = "q4_boss_extra.rs"]
mod q4_boss_extra;
#[path = "q4_boss_jiwu.rs"]
mod q4_boss_jiwu;
pub(crate) use q4_boss_jiwu::{q4_boss_on_deck_washed, retieji_sha_begin};

pub(super) const ASK_ZHINANG_TARGET: u32 = 1;
pub(super) const ASK_QIWU_TARGET: u32 = 2;
pub(super) const ASK_HUODI_TARGET: u32 = 3;
pub(super) const ASK_BASHI_CONFIRM: u32 = 4;
pub(super) const ASK_DANJING_CONFIRM: u32 = 5;
pub(super) const ASK_JIAOXIE_TARGET: u32 = 6;
pub(super) const ASK_JIAOXIE_GIVE: u32 = 7;
pub(super) const ASK_KEDING_CARDS: u32 = 8;
pub(super) const ASK_KEDING_TARGETS: u32 = 9;
pub(super) const ASK_BAONU_FORM: u32 = 10;
pub(super) const ASK_XIULUO_HAND: u32 = 11;
pub(super) const ASK_XIULUO_JUDGE: u32 = 12;
pub(super) const ASK_SHENQU_TAO: u32 = 13;
pub(super) const ASK_JIWU_CARD: u32 = 14;
pub(super) const ASK_JIWU_SKILL: u32 = 15;
pub(super) const ASK_LINGFENG_TARGET: u32 = 16;
pub(super) const ASK_LEILI_TARGET: u32 = 17;
pub(super) const ASK_FENGXING_TARGET: u32 = 18;
pub(super) const ASK_XUANFENG_TARGET: u32 = 19;
pub(super) const ASK_DANJING_SAVE: u32 = 20;
pub(super) const ASK_LINGFENG_GAIN: u32 = 21;
pub(super) const ASK_XUANFENG_CARD: u32 = 22;
pub(super) const ASK_TIANYUJG_NEXT: u32 = 23;
pub(super) const ASK_XUANLEI_NEXT: u32 = 24;
pub(super) const ASK_SKONGHUN_NEXT: u32 = 25;
pub(super) const ASK_JIWU_RESUME: u32 = 26;
pub(super) const ASK_JINGJIA_NEXT: u32 = 27;
pub(super) const ASK_QIANGXIX_TARGET: u32 = 28;
pub(super) const ASK_QIANGXIX_CARD: u32 = 29;
pub(super) const ASK_RETIEJI_CONFIRM: u32 = 30;
pub(super) const ASK_RETIEJI_GIVE: u32 = 31;
pub(super) const ASK_REXUANFENG_MODE: u32 = 32;
pub(super) const ASK_REXUANFENG_MOVE_FROM: u32 = 33;
pub(super) const ASK_REXUANFENG_MOVE_CARD: u32 = 34;
pub(super) const ASK_REXUANFENG_MOVE_TO: u32 = 35;
pub(super) const ASK_RETIEJI_DISCARD: u32 = 36;
pub(super) const ASK_RETIEJI_JUDGE: u32 = 37;

pub(super) fn is_mech_seat(core: &GameStateCore, catalog: &Catalog, seat: PlayerId) -> bool {
    catalog
        .skill_by_name("boss_jiguan")
        .is_some_and(|def| core.has_skill(seat, def))
        || core
            .vm_aux
            .flags
            .get(&(seat, "q4_is_mech".to_string()))
            .copied()
            .unwrap_or(0)
            > 0
}

pub(super) fn fengjian_banned(core: &GameStateCore, actor: PlayerId, target: PlayerId) -> bool {
    core.vm_aux
        .flags
        .get(&(actor, format!("q4_fengjian_ban:{target}")))
        .copied()
        .unwrap_or(0)
        > 0
}

pub(super) fn matching_judge_suit_cards(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<CardId> {
    let pl = core.player(owner);
    if pl.judge.is_empty() {
        return Vec::new();
    }
    pl.hand
        .iter()
        .copied()
        .filter(|hc| {
            let hs = catalog.effective_card_suit(core, owner, *hc);
            pl.judge
                .iter()
                .any(|jc| catalog.effective_card_suit(core, owner, *jc) == hs)
        })
        .collect()
}

fn is_q4_normal_trick(kind: CardKind) -> bool {
    kind.is_trick_kind()
        && !matches!(
            kind,
            CardKind::Le | CardKind::Bing | CardKind::ShanDian | CardKind::HongShui
        )
}

pub(super) fn q4_keding_extra_targets(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    card: Option<CardId>,
    existing_targets: &[PlayerId],
) -> Vec<PlayerId> {
    let Some(cid) = card else {
        return Vec::new();
    };
    let kind = catalog.card_kind(cid);
    if !kind.allows_multiple_targets() {
        return Vec::new();
    }
    let nature = match kind {
        CardKind::FireSha => DamageNature::Fire,
        CardKind::LeiSha => DamageNature::Thunder,
        _ => DamageNature::Normal,
    };
    let (_, _, legal) = core.legal_targets(catalog, kind, owner);
    legal
        .into_iter()
        .filter(|p| {
            !existing_targets.contains(p)
                && !fengjian_banned(core, owner, *p)
                && (!kind.is_sha_like()
                    || (!crate::kernel::sha_user_or_target_blocked(core, catalog, owner, *p)
                        && core.computed_distance(catalog, owner, *p)
                            <= core.attack_range(catalog, owner)))
                && !crate::rvm_bridge::modifiers::eval_card_target_canceled(
                    core, catalog, owner, *p, cid, kind, nature,
                )
        })
        .collect()
}

pub(super) fn q4_fengxing_targets(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<PlayerId> {
    (0..core.players.len() as PlayerId)
        .filter(|&p| {
            core.player(p).alive
                && core.is_hostile_to(catalog, owner, p)
                && !crate::kernel::sha_user_or_target_blocked(core, catalog, owner, p)
                && !crate::rvm_bridge::modifiers::eval_card_target_canceled(
                    core,
                    catalog,
                    owner,
                    p,
                    0,
                    CardKind::Sha,
                    DamageNature::Normal,
                )
        })
        .collect()
}

pub(super) fn jiwu_available_skills(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<SkillDefId> {
    const GROUPS: [&[&str]; 4] = [
        &["qiangxix", "qiangxi"],
        &["retieji", "tieji", "铁骑"],
        &["olxuanfeng", "rexuanfeng", "xuanfeng"],
        &["rewansha", "wansha", "完杀"],
    ];
    let mut out = Vec::new();
    for group in GROUPS {
        let already = group
            .iter()
            .filter_map(|n| catalog.skill_by_name(n))
            .any(|def| core.has_skill(owner, def));
        if already {
            continue;
        }
        if let Some(def) = group.iter().filter_map(|n| catalog.skill_by_name(n)).next() {
            out.push(def);
        }
    }
    out
}

pub(super) fn wait_q4_boss(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let Some(spec) = q4_boss_asks::ask_q4_boss(
        m.stage, m.owner, m.target, &m.targets, m.card, &m.cards, ctx.core, ctx.catalog,
    ) else {
        return Flow::Done;
    };
    Flow::WaitChoice {
        observer: spec.observer,
        spec: crate::choice::ChoiceSpec {
            kind: spec.kind,
            prompt: spec.prompt,
            options: spec.options,
            min: spec.min,
            max: spec.max,
            allow_cancel: spec.allow_cancel,
        },
    }
}

pub(in crate::content::host) fn trigger_meta_q4_boss(name: &str) -> Option<TriggerMeta> {
    let m = |event: EventKind, extra_events: Vec<EventKind>, optional: bool| TriggerMeta {
        event,
        timing: Timing::Normal,
        priority: 0,
        order_selectable: false,
        optional,
        extra_events,
    };
    match name {
        "boss_jizhen" | "boss_tianyujg" | "boss_fanshi" => Some(m(EventKind::PhaseBegin, Vec::new(), false)),
        "boss_gongshenjg" | "boss_huodi" => Some(m(EventKind::PhaseBegin, Vec::new(), true)),
        "boss_jingmiao" => Some(m(EventKind::CardResponded, vec![EventKind::CardUsed], true)),
        "boss_zhinang" => Some(m(EventKind::PhaseBegin, Vec::new(), true)),
        "boss_biantian" => Some(m(EventKind::PhaseBegin, vec![EventKind::Death], false)),
        "boss_qiwu" | "boss_bashi" | "boss_keding" => Some(m(EventKind::CardUsed, Vec::new(), false)),
        "boss_danjing" => Some(m(EventKind::Dying, Vec::new(), false)),
        "boss_xiaorui" | "boss_fengjian" => Some(m(EventKind::DamageSource, Vec::new(), false)),
        "boss_huchen" => Some(m(EventKind::Death, Vec::new(), false)),
        "boss_baonu" => Some(m(EventKind::DamageAfter, vec![EventKind::LoseHp], false)),
        "boss_jingjia" => Some(m(EventKind::TurnBegin, Vec::new(), false)),
        "xiuluo" => Some(m(EventKind::PhaseBegin, Vec::new(), false)),
        "shenqu" => Some(m(EventKind::PhaseBegin, vec![EventKind::DamageAfter], false)),
        "boss_lingfeng" => Some(m(EventKind::DrawPhaseBefore, Vec::new(), true)),
        "boss_jueji" => Some(m(EventKind::DrawPhaseBegin, Vec::new(), true)),
        "boss_leili" => Some(m(EventKind::DamageAfter, Vec::new(), false)),
        "boss_fengxing" | "boss_xuanlei" => Some(m(EventKind::PhaseBegin, Vec::new(), false)),
        "boss_skonghun" => Some(m(EventKind::PhaseBegin, Vec::new(), false)),
        "xuanfeng" | "rexuanfeng" => Some(m(EventKind::PhaseEnd, vec![EventKind::CardsLost], true)),
        "olxuanfeng" => Some(m(EventKind::CardsLost, vec![EventKind::CardsDiscarded], true)),
        "rewansha" => Some(m(EventKind::Dying, Vec::new(), false)),
        _ => None,
    }
}

pub(in crate::content::host) fn limits_q4_boss(_name: &str) -> SkillLimits {
    SkillLimits::default()
}

pub(in crate::content::host) fn has_phase_use_q4_boss(name: &str) -> bool {
    matches!(name, "boss_jiaoxie" | "jiwu" | "qiangxix")
}

pub(in crate::content::host) fn filter_q4_boss(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    let core = ctx.state;
    let cat = ctx.catalog;
    match (skill.name.as_str(), event) {
        ("boss_jizhen", EventData::Phase { player, phase: Phase::End, begin: true }) => {
            *player == owner
                && (0..core.players.len() as PlayerId).any(|p| {
                    let pl = core.player(p);
                    pl.alive && pl.hp < pl.max_hp as i32 && core.is_same_team_or_friend(cat, owner, p)
                })
        }
        ("boss_gongshenjg", EventData::Phase { player, phase: Phase::End, begin: true }) => {
            *player == owner
                && (0..core.players.len() as PlayerId).any(|p| {
                    let pl = core.player(p);
                    pl.alive && is_mech_seat(core, cat, p) && (core.is_hostile_to(cat, owner, p) || pl.hp < pl.max_hp as i32)
                })
        }
        ("boss_jingmiao", EventData::CardUsed(u)) => {
            !u.cancelled
                && core.is_hostile_to(cat, owner, u.player)
                && u.card.is_some_and(|c| cat.card_kind(c) == CardKind::Nullify)
        }
        ("boss_jingmiao", EventData::CardResponded { card, player }) => {
            core.is_hostile_to(cat, owner, *player) && cat.card_kind(*card) == CardKind::Nullify
        }
        ("boss_zhinang", EventData::Phase { player, phase: Phase::Prepare, begin: true }) => {
            *player == owner && !core.deck.is_empty()
        }
        ("boss_biantian", EventData::Phase { player, phase: Phase::Prepare, begin: true }) => *player == owner,
        ("boss_biantian", EventData::Death { player, .. }) => *player == owner,
        ("boss_qiwu", EventData::CardUsed(u)) => {
            !u.cancelled
                && u.player == owner
                && u.card.is_some_and(|c| cat.effective_card_suit(core, owner, c) == crate::catalog::Suit::Club)
                && (0..core.players.len() as PlayerId).any(|p| {
                    let pl = core.player(p);
                    pl.alive && pl.hp < pl.max_hp as i32 && core.is_same_team_or_friend(cat, owner, p)
                })
        }
        ("boss_tianyujg", EventData::Phase { player, phase: Phase::End, begin: true }) => {
            *player == owner
                && (0..core.players.len() as PlayerId)
                    .any(|p| core.player(p).alive && !core.player(p).linked && core.is_hostile_to(cat, owner, p))
        }
        ("boss_huodi", EventData::Phase { player, phase: Phase::End, begin: true }) => {
            *player == owner
                && (0..core.players.len() as PlayerId).any(|p| {
                    core.player(p).alive && core.player(p).face_down && core.is_same_team_or_friend(cat, owner, p)
                })
        }
        ("boss_fanshi", EventData::Phase { player, phase: Phase::End, begin: true }) => {
            *player == owner && core.player(owner).alive
        }
        ("boss_bashi", EventData::CardUsed(u)) => {
            !u.cancelled
                && u.player != owner
                && u.targets.contains(&owner)
                && !core.player(owner).face_down
                && u.card.is_some_and(|c| {
                    let k = cat.card_kind(c);
                    k.is_sha_like() || is_q4_normal_trick(k)
                })
        }
        ("boss_danjing", EventData::Dying { player, .. }) => {
            core.player(owner).hp > 1
                && core.player(*player).hp < 1
                && core.is_same_team_or_friend(cat, owner, *player)
        }
        ("boss_xiaorui", EventData::Damage(d)) => {
            d.amount > 0
                && !d.prevented
                && d.source.is_some_and(|s| s == core.turn.current && core.player(s).alive && core.is_same_team_or_friend(cat, owner, s))
                && d.card.is_some_and(|c| cat.card_kind(c).is_sha_like())
        }
        ("boss_huchen", EventData::Death { player, killer }) => {
            *killer == Some(owner) && core.is_hostile_to(cat, owner, *player)
        }
        ("boss_fengjian", EventData::Damage(d)) => {
            d.amount > 0 && !d.prevented && d.source == Some(owner) && core.player(d.target).alive
        }
        ("boss_keding", EventData::CardUsed(u)) => {
            !u.cancelled
                && u.player == owner
                && u.targets.len() == 1
                && !core.player(owner).hand.is_empty()
                && !q4_keding_extra_targets(core, cat, owner, u.card, &u.targets).is_empty()
        }
        ("boss_baonu", EventData::Damage(d)) => {
            !d.prevented && d.target == owner && core.player(owner).alive && core.player(owner).hp > 0 && core.player(owner).hp <= 4
        }
        ("boss_baonu", EventData::LoseHp { player, .. }) => {
            *player == owner && core.player(owner).alive && core.player(owner).hp > 0 && core.player(owner).hp <= 4
        }
        ("boss_jingjia", EventData::TurnBegin { player }) => {
            *player == owner && core.vm_aux.flags.get(&(owner, "q4_jingjia_done".to_string())).copied().unwrap_or(0) == 0
        }
        ("xiuluo", EventData::Phase { player, phase: Phase::Prepare, begin: true }) => {
            *player == owner && !matching_judge_suit_cards(core, cat, owner).is_empty()
        }
        ("shenqu", EventData::Phase { phase: Phase::Prepare, begin: true, .. }) => {
            core.player(owner).alive && core.player(owner).hand.len() <= core.player(owner).max_hp as usize
        }
        ("shenqu", EventData::Damage(d)) => {
            !d.prevented
                && d.amount > 0
                && d.target == owner
                && core.player(owner).alive
                && core.player(owner).hand.iter().any(|c| cat.card_kind(*c) == CardKind::Tao)
        }
        ("boss_lingfeng", EventData::DrawPhase { player, outcome: crate::event::DrawOutcome::Running, .. }) => {
            *player == owner && core.player(owner).alive && (core.deck.len() + core.discard.len()) >= 2
        }
        ("boss_jueji", EventData::DrawPhase { player, outcome: crate::event::DrawOutcome::Running, pending, .. }) => {
            *player != owner
                && *pending > 0
                && core.player(*player).alive
                && core.player(*player).hp < core.player(*player).max_hp as i32
                && core.is_hostile_to(cat, owner, *player)
        }
        ("boss_leili", EventData::Damage(d)) => {
            !d.prevented
                && d.amount > 0
                && d.source == Some(owner)
                && d.card.is_some_and(|c| cat.card_kind(c).is_sha_like())
                && (0..core.players.len() as PlayerId).any(|p| {
                    p != d.target && core.player(p).alive && core.is_hostile_to(cat, owner, p)
                })
        }
        ("boss_fengxing", EventData::Phase { player, phase: Phase::Prepare, begin: true }) => {
            *player == owner && !q4_fengxing_targets(core, cat, owner).is_empty()
        }
        ("boss_xuanlei", EventData::Phase { player, phase: Phase::Prepare, begin: true }) => {
            *player == owner
                && (0..core.players.len() as PlayerId).any(|p| {
                    core.player(p).alive && !core.player(p).judge.is_empty() && core.is_hostile_to(cat, owner, p)
                })
        }
        ("boss_skonghun", EventData::Phase { player, phase: Phase::Play, begin: true }) => {
            if *player != owner || !core.player(owner).alive {
                return false;
            }
            let enemies = (0..core.players.len() as PlayerId)
                .filter(|&p| core.player(p).alive && core.is_hostile_to(cat, owner, p))
                .count() as i32;
            let lost = (core.player(owner).max_hp as i32 - core.player(owner).hp).max(0);
            enemies > 0 && lost >= enemies
        }
        ("xuanfeng" | "rexuanfeng", EventData::Phase { player, phase: Phase::Discard, begin: false }) => {
            *player == owner
                && (core.turn.discard_phase_discarded >= 2
                    || core
                        .vm_aux
                        .flags
                        .get(&(owner, "q4_xuanfeng_discard_ready".to_string()))
                        .copied()
                        .unwrap_or(0)
                        > 0)
                && (q4_boss_jiwu::has_other_card_target(core, owner)
                    || (skill.name == "rexuanfeng"
                        && q4_boss_jiwu::can_move_other_equip(core, cat, owner)))
        }
        ("xuanfeng" | "rexuanfeng", EventData::CardsLost { player, origins, .. }) => {
            *player == owner
                && origins.contains(&crate::event::LostOrigin::Equip)
                && (q4_boss_jiwu::has_other_card_target(core, owner)
                    || (skill.name == "rexuanfeng"
                        && q4_boss_jiwu::can_move_other_equip(core, cat, owner)))
        }
        ("olxuanfeng", EventData::CardsLost { player, cards, origins }) => {
            *player == owner
                && (origins.contains(&crate::event::LostOrigin::Equip) || cards.len() >= 2)
                && q4_boss_jiwu::has_other_card_target(core, owner)
        }
        ("olxuanfeng", EventData::CardsDiscarded { player, cards }) => {
            *player == owner
                && cards.len() >= 2
                && q4_boss_jiwu::has_other_card_target(core, owner)
        }
        ("rewansha", EventData::Dying { .. }) => {
            core.turn.current == owner && core.player(owner).alive
        }
        _ => false,
    }
}

pub(in crate::content::host) fn activate_q4_boss(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<Box<dyn SkillMachine>> {
    let core = ctx.state;
    let cat = ctx.catalog;
    match skill.name.as_str() {
        "boss_jiaoxie" => {
            if core.vm_aux.once_count(owner, 1, "boss_jiaoxie") >= 1 {
                return None;
            }
            let any_tgt = (0..core.players.len() as PlayerId).any(|p| {
                let pl = core.player(p);
                pl.alive && core.is_hostile_to(cat, owner, p) && is_mech_seat(core, cat, p) && (!pl.hand.is_empty() || !pl.equips.is_empty())
            });
            any_tgt.then(|| Box::new(HostMachine {
                def_key: skill.name.clone(),
                behavior: HostBehavior::Q4BossPlaySkill,
                owner,
                target: None,
                card: Some(ASK_JIAOXIE_TARGET),
                cards: Vec::new(),
                targets: Vec::new(),
                stage: HostStage::Act,
            }) as Box<dyn SkillMachine>)
        }
        "jiwu" => {
            let has_card = !core.player(owner).hand.is_empty() || !core.player(owner).equips.is_empty();
            if !has_card || jiwu_available_skills(core, cat, owner).is_empty() {
                return None;
            }
            Some(Box::new(HostMachine {
                def_key: skill.name.clone(),
                behavior: HostBehavior::Q4BossPlaySkill,
                owner,
                target: None,
                card: Some(ASK_JIWU_CARD),
                cards: Vec::new(),
                targets: Vec::new(),
                stage: HostStage::Act,
            }))
        }
        "qiangxix" => {
            if !q4_boss_jiwu::can_activate_qiangxix(core, cat, owner) {
                return None;
            }
            Some(Box::new(HostMachine {
                def_key: skill.name.clone(),
                behavior: HostBehavior::Q4BossPlaySkill,
                owner,
                target: None,
                card: Some(ASK_QIANGXIX_TARGET),
                cards: Vec::new(),
                targets: Vec::new(),
                stage: HostStage::Act,
            }))
        }
        _ => None,
    }
}

pub(in crate::content::host) fn machine_q4_boss(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    let (card, cards, target, targets) = match (skill.name.as_str(), event) {
        ("boss_zhinang", _) => (Some(ASK_ZHINANG_TARGET), Vec::new(), None, Vec::new()),
        ("boss_qiwu", EventData::CardUsed(u)) => (Some(ASK_QIWU_TARGET), Vec::new(), Some(u.player), Vec::new()),
        ("boss_huodi", _) => (Some(ASK_HUODI_TARGET), Vec::new(), None, Vec::new()),
        ("boss_bashi", EventData::CardUsed(u)) => (Some(ASK_BASHI_CONFIRM), Vec::new(), Some(u.player), u.targets.clone()),
        ("boss_danjing", EventData::Dying { player, .. }) => (Some(ASK_DANJING_CONFIRM), Vec::new(), Some(*player), Vec::new()),
        ("boss_keding", EventData::CardUsed(u)) => (Some(ASK_KEDING_CARDS), u.card.into_iter().collect(), None, u.targets.clone()),
        ("boss_baonu", _) => (Some(ASK_BAONU_FORM), Vec::new(), None, Vec::new()),
        ("xiuluo", _) => (Some(ASK_XIULUO_HAND), Vec::new(), None, Vec::new()),
        ("shenqu", EventData::Damage(_)) => (Some(ASK_SHENQU_TAO), Vec::new(), None, Vec::new()),
        ("boss_jingmiao", EventData::CardUsed(u)) => (None, Vec::new(), Some(u.player), Vec::new()),
        ("boss_jingmiao", EventData::CardResponded { player, .. }) => (None, Vec::new(), Some(*player), Vec::new()),
        ("boss_xiaorui", EventData::Damage(d)) => (None, Vec::new(), d.source, Vec::new()),
        ("boss_fengjian", EventData::Damage(d)) => (None, Vec::new(), d.source, vec![d.target]),
        ("boss_lingfeng", EventData::DrawPhase { player, .. }) => (Some(ASK_LINGFENG_TARGET), Vec::new(), Some(*player), Vec::new()),
        ("boss_jueji", EventData::DrawPhase { player, .. }) => (None, Vec::new(), Some(*player), Vec::new()),
        ("boss_leili", EventData::Damage(d)) => (Some(ASK_LEILI_TARGET), Vec::new(), Some(d.target), Vec::new()),
        ("boss_fengxing", _) => (Some(ASK_FENGXING_TARGET), Vec::new(), None, Vec::new()),
        ("xuanfeng" | "olxuanfeng" | "rexuanfeng", _) => (Some(ASK_XUANFENG_TARGET), vec![2], None, Vec::new()),
        ("rewansha", EventData::Dying { player, .. }) => (None, Vec::new(), Some(*player), Vec::new()),
        _ => (None, Vec::new(), None, Vec::new()),
    };
    HostMachine {
        def_key: skill.name.clone(),
        behavior: HostBehavior::Q4BossPlaySkill,
        owner,
        target,
        card,
        cards,
        targets,
        stage: HostStage::Act,
    }
}

fn start_act(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.def_key.as_str() {
        "boss_jizhen" => {
            ctx.log(LogEvent::SkillTriggered { skill: "激阵", owner, presentation: None });
            for p in ctx.core.seat_order_from(owner) {
                let pl = ctx.core.player(p);
                if pl.alive && pl.hp < pl.max_hp as i32 && ctx.core.is_same_team_or_friend(ctx.catalog, owner, p) {
                    ctx.start_draw(p, 1);
                }
            }
            Flow::Done
        }
        "boss_gongshenjg" => {
            let mut healed = false;
            for p in ctx.core.seat_order_from(owner) {
                let pl = ctx.core.player(p);
                if pl.alive && is_mech_seat(ctx.core, ctx.catalog, p) && ctx.core.is_same_team_or_friend(ctx.catalog, owner, p) && pl.hp < pl.max_hp as i32 {
                    ctx.log(LogEvent::SkillTriggered { skill: "工神", owner, presentation: None });
                    ctx.recover(p, 1, None);
                    healed = true;
                    break;
                }
            }
            if !healed {
                if let Some(enemy) = ctx.core.seat_order_from(owner).into_iter().find(|&p| {
                    ctx.core.player(p).alive && is_mech_seat(ctx.core, ctx.catalog, p) && ctx.core.is_hostile_to(ctx.catalog, owner, p)
                }) {
                    ctx.log(LogEvent::SkillTriggered { skill: "工神", owner, presentation: None });
                    ctx.emit(EventData::Damage(damage_data_with(Some(owner), enemy, 1, DamageNature::Fire, None)));
                }
            }
            Flow::Done
        }
        "boss_jingmiao" => {
            if let Some(user) = m.target {
                ctx.log(LogEvent::SkillTriggered { skill: "精妙", owner, presentation: None });
                ctx.emit(EventData::LoseHp { player: user, amount: 1 });
            }
            Flow::Done
        }
        "boss_zhinang" => q4_boss_extra::start_boss_zhinang(m, ctx),
        "boss_biantian" => {
            for p in 0..ctx.core.players.len() as PlayerId {
                ctx.core.vm_aux.flags.remove(&(p, "q4_biantian_dawu".to_string()));
                ctx.core.vm_aux.flags.remove(&(p, "q4_biantian_kuangfeng".to_string()));
            }
            if !ctx.core.player(owner).alive {
                return Flow::Done;
            }
            ctx.log(LogEvent::SkillTriggered { skill: "变天", owner, presentation: None });
            let judged = ctx.draw_judge_card();
            let suit = ctx.catalog.cards[judged as usize].suit;
            ctx.to_discard(judged);
            for p in 0..ctx.core.players.len() as PlayerId {
                if !ctx.core.player(p).alive {
                    continue;
                }
                let is_red = matches!(suit, crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond);
                if is_red && ctx.core.is_hostile_to(ctx.catalog, owner, p) {
                    ctx.core.vm_aux.flags.insert((p, "q4_biantian_kuangfeng".to_string()), 1);
                } else if !is_red && ctx.core.is_same_team_or_friend(ctx.catalog, owner, p) {
                    ctx.core.vm_aux.flags.insert((p, "q4_biantian_dawu".to_string()), 1);
                }
            }
            Flow::Done
        }
        "boss_tianyujg" => q4_boss_extra::start_boss_tianyujg(m, ctx),
        "boss_fanshi" => {
            ctx.log(LogEvent::SkillTriggered { skill: "反噬", owner, presentation: None });
            ctx.emit(EventData::LoseHp { player: owner, amount: 1 });
            Flow::Done
        }
        "boss_xiaorui" => {
            if let Some(src) = m.target {
                ctx.log(LogEvent::SkillTriggered { skill: "骁锐", owner, presentation: None });
                *ctx.core.vm_aux.flags.entry((src, "skill_turn:q4_xiaorui_sha_plus".to_string())).or_insert(0) += 1;
            }
            Flow::Done
        }
        "boss_huchen" => {
            ctx.log(LogEvent::SkillTriggered { skill: "虎臣", owner, presentation: None });
            *ctx.core.vm_aux.flags.entry((owner, "q4_huchen_kills".to_string())).or_insert(0) += 1;
            Flow::Done
        }
        "boss_fengjian" => {
            if let Some(&tgt) = m.targets.first() {
                ctx.log(LogEvent::SkillTriggered { skill: "封缄", owner, presentation: None });
                ctx.core.vm_aux.flags.insert((tgt, format!("q4_fengjian_ban:{owner}")), 1);
            }
            Flow::Done
        }
        "boss_jingjia" => q4_boss_extra::start_boss_jingjia(m, ctx),
        "shenqu" if m.card.is_none() => {
            ctx.log(LogEvent::SkillTriggered { skill: "神躯", owner, presentation: None });
            ctx.start_draw(owner, 2);
            Flow::Done
        }
        "boss_lingfeng" => q4_boss_extra::start_boss_lingfeng(m, ctx),
        "boss_jueji" => {
            if let Some(tgt) = m.target {
                ctx.log(LogEvent::SkillTriggered { skill: "绝汲", owner, presentation: None });
                for frame in ctx.res.stack.iter_mut().rev() {
                    if let EventData::DrawPhase { player, pending, .. } = &mut frame.data {
                        if *player == tgt {
                            *pending = (*pending - 1).max(0);
                            break;
                        }
                    }
                }
            }
            Flow::Done
        }
        "boss_xuanlei" => q4_boss_extra::start_boss_xuanlei(m, ctx),
        "boss_skonghun" => q4_boss_extra::start_boss_skonghun(m, ctx),
        "xuanfeng" | "olxuanfeng" | "rexuanfeng" => {
            ctx.core.vm_aux.flags.remove(&(owner, "q4_xuanfeng_discard_ready".to_string()));
            if m.def_key == "rexuanfeng"
                && q4_boss_jiwu::can_move_other_equip(ctx.core, ctx.catalog, owner)
            {
                m.card = Some(ASK_REXUANFENG_MODE);
                m.stage = HostStage::SelectNature;
                wait_q4_boss(m, ctx)
            } else if q4_boss_jiwu::has_other_card_target(ctx.core, owner) {
                m.cards = vec![2];
                m.card = Some(ASK_XUANFENG_TARGET);
                m.stage = HostStage::TargetChoice;
                wait_q4_boss(m, ctx)
            } else {
                Flow::Done
            }
        }
        "rewansha" => q4_boss_jiwu::start_rewansha(m, ctx),
        "boss_qiwu" | "boss_huodi" | "boss_jiaoxie" | "boss_leili" | "boss_fengxing" | "qiangxix" => {
            m.stage = HostStage::TargetChoice;
            wait_q4_boss(m, ctx)
        }
        "boss_bashi" | "boss_danjing" | "retieji" => {
            m.stage = HostStage::CardsAsk;
            wait_q4_boss(m, ctx)
        }
        "boss_keding" | "xiuluo" | "shenqu" | "jiwu" => {
            m.stage = HostStage::SelectCard;
            wait_q4_boss(m, ctx)
        }
        "boss_baonu" => {
            let has_baonu = ctx
                .catalog
                .skill_by_name("boss_baonu")
                .is_some_and(|def| ctx.core.has_skill(owner, def));
            if !ctx.core.player(owner).alive || !has_baonu {
                return Flow::Done;
            }
            m.stage = HostStage::SelectNature;
            wait_q4_boss(m, ctx)
        }
        _ => Flow::Done,
    }
}

pub(in crate::content::host) fn handle_q4_boss(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if m.stage == HostStage::Act {
        return start_act(m, ctx);
    }
    if m.stage == HostStage::DrawSettle {
        return match m.card {
            Some(ASK_DANJING_SAVE) => q4_boss_extra::step_danjing_save(m, ctx),
            Some(ASK_LINGFENG_GAIN) => q4_boss_extra::finish_lingfeng_gain(m, ctx),
            Some(ASK_JIWU_RESUME) => q4_boss_extra::finish_jiwu_skill_step(m, ctx),
            Some(ASK_JINGJIA_NEXT) => q4_boss_extra::step_jingjia_next(m, ctx),
            Some(ASK_TIANYUJG_NEXT) => q4_boss_extra::step_tianyujg_next(m, ctx),
            Some(ASK_XUANLEI_NEXT) => q4_boss_extra::step_xuanlei_next(m, ctx),
            Some(ASK_SKONGHUN_NEXT) => q4_boss_extra::step_skonghun_next(m, ctx),
            Some(ASK_XUANFENG_TARGET) => q4_boss_extra::resume_xuanfeng_next(m, ctx),
            Some(ASK_QIANGXIX_CARD) => q4_boss_jiwu::finish_qiangxix_damage(m, ctx),
            Some(ASK_RETIEJI_JUDGE) => q4_boss_jiwu::step_retieji_judge_settled(m, ctx),
            _ => Flow::Done,
        };
    }
    let owner = m.owner;
    match m.card {
        Some(ASK_ZHINANG_TARGET) => q4_boss_extra::finish_boss_zhinang(m, ctx),
        Some(ASK_QIWU_TARGET) => {
            if let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() {
                ctx.log(LogEvent::SkillTriggered { skill: "栖梧", owner, presentation: None });
                ctx.recover(tgt, 1, None);
            }
            Flow::Done
        }
        Some(ASK_HUODI_TARGET) => {
            if let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() {
                ctx.log(LogEvent::SkillTriggered { skill: "惑敌", owner, presentation: None });
                ctx.emit(EventData::TurnOverBefore { player: tgt, cancelled: false });
            }
            Flow::Done
        }
        Some(ASK_BASHI_CONFIRM) => {
            if matches!(ctx.answer().first(), Some(ChoiceOption::Confirm)) && !ctx.core.player(owner).face_down {
                ctx.log(LogEvent::SkillTriggered { skill: "拔矢", owner, presentation: None });
                if ctx.current_card_use().is_ok() {
                    ctx.current_card_use_mut().targets.retain(|t| *t != owner);
                }
                ctx.emit(EventData::TurnOverBefore { player: owner, cancelled: false });
            }
            Flow::Done
        }
        Some(ASK_DANJING_CONFIRM) => {
            if let (Some(ChoiceOption::Confirm), Some(_dying_tgt)) = (ctx.answer().first().copied(), m.target) {
                ctx.log(LogEvent::SkillTriggered { skill: "啖睛", owner, presentation: None });
                m.card = Some(ASK_DANJING_SAVE);
                m.stage = HostStage::DrawSettle;
                ctx.emit(EventData::LoseHp { player: owner, amount: 1 });
                return Flow::Yield;
            }
            Flow::Done
        }
        Some(ASK_JIAOXIE_TARGET) => {
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            ctx.core.vm_aux.inc_once(owner, 1, "boss_jiaoxie");
            m.target = Some(tgt);
            m.card = Some(ASK_JIAOXIE_GIVE);
            m.stage = HostStage::SelectCard;
            wait_q4_boss(m, ctx)
        }
        Some(ASK_JIAOXIE_GIVE) => {
            let (Some(tgt), Some(ChoiceOption::Card(c))) = (m.target, ctx.answer().first().copied()) else {
                return Flow::Done;
            };
            ctx.log(LogEvent::SkillTriggered { skill: "缴械", owner, presentation: None });
            if ctx.core.player(tgt).hand.contains(&c) {
                ctx.take_hand_card(tgt, c);
            } else {
                ctx.core.player_mut(tgt).equips.retain(|x| *x != c);
                crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, tgt, c);
            }
            ctx.core.player_mut(owner).hand.push(c);
            ctx.log(LogEvent::CardsGiven { from: tgt, to: owner, cards: vec![c] });
            Flow::Done
        }
        Some(ASK_KEDING_CARDS) => q4_boss_extra::step_keding_cards(m, ctx),
        Some(ASK_KEDING_TARGETS) => q4_boss_extra::step_keding_targets(m, ctx),
        Some(ASK_BAONU_FORM) => q4_boss_extra::finish_boss_baonu(m, ctx),
        Some(ASK_XIULUO_HAND) => q4_boss_extra::step_xiuluo_hand(m, ctx),
        Some(ASK_XIULUO_JUDGE) => q4_boss_extra::step_xiuluo_judge(m, ctx),
        Some(ASK_SHENQU_TAO) => q4_boss_extra::finish_shenqu_tao(m, ctx),
        Some(ASK_JIWU_CARD) => q4_boss_extra::step_jiwu_card(m, ctx),
        Some(ASK_JIWU_SKILL) => q4_boss_extra::step_jiwu_skill(m, ctx),
        Some(ASK_LINGFENG_TARGET) => q4_boss_extra::step_lingfeng_target(m, ctx),
        Some(ASK_LEILI_TARGET) => {
            if let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() {
                ctx.log(LogEvent::SkillTriggered { skill: "雷厉", owner, presentation: None });
                ctx.emit(EventData::Damage(damage_data_with(
                    Some(owner),
                    tgt,
                    1,
                    DamageNature::Thunder,
                    None,
                )));
            }
            Flow::Done
        }
        Some(ASK_FENGXING_TARGET) => {
            if let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() {
                if q4_fengxing_targets(ctx.core, ctx.catalog, owner).contains(&tgt) {
                    ctx.log(LogEvent::SkillTriggered { skill: "风行", owner, presentation: None });
                    ctx.emit(EventData::CardUsed(crate::event::CardUseData {
                        card: None,
                        player: owner,
                        targets: vec![tgt],
                        nature: DamageNature::Normal,
                        use_as: Some(CardKind::Sha),
                        cancelled: false,
                    }));
                }
            }
            Flow::Done
        }
        Some(ASK_XUANFENG_TARGET) => q4_boss_extra::step_xuanfeng_target(m, ctx),
        Some(ASK_XUANFENG_CARD) => q4_boss_extra::step_xuanfeng_card(m, ctx),
        Some(ASK_QIANGXIX_TARGET) => q4_boss_jiwu::step_qiangxix_target(m, ctx),
        Some(ASK_QIANGXIX_CARD) => q4_boss_jiwu::step_qiangxix_card(m, ctx),
        Some(ASK_RETIEJI_CONFIRM) => q4_boss_jiwu::step_retieji_confirm(m, ctx),
        Some(ASK_RETIEJI_GIVE) => q4_boss_jiwu::step_retieji_give(m, ctx),
        Some(ASK_RETIEJI_DISCARD) => q4_boss_jiwu::step_retieji_discard(m, ctx),
        Some(ASK_REXUANFENG_MODE) => q4_boss_jiwu::step_rexuanfeng_mode(m, ctx),
        Some(ASK_REXUANFENG_MOVE_FROM) => q4_boss_jiwu::step_rexuanfeng_move_from(m, ctx),
        Some(ASK_REXUANFENG_MOVE_CARD) => q4_boss_jiwu::step_rexuanfeng_move_card(m, ctx),
        Some(ASK_REXUANFENG_MOVE_TO) => q4_boss_jiwu::step_rexuanfeng_move_to(m, ctx),
        _ => Flow::Done,
    }
}
