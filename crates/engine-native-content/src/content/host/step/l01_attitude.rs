//! L01 `AI优化` 公开态度与残局追踪器（`extension.js:870-1817, 3503-3920`）：
//! 严格公开信息集追踪——绝不读取未明置对手身份或暗手牌。

use super::*;
use crate::catalog::CardKind;
use crate::state::{GameMode, GameStateCore, Role};

pub const L01_SCORE_BIAS: i64 = 1_000_000;
const MAX_SCORE: i32 = 10_000;
const MIN_SCORE: i32 = -10_000;

fn flag(core: &GameStateCore, p: PlayerId, key: &str) -> u32 {
    core.vm_aux.flags.get(&(p, key.to_string())).copied().unwrap_or(0)
}

fn set_flag(core: &mut GameStateCore, p: PlayerId, key: &str, val: u32) {
    if val == 0 {
        core.vm_aux.flags.remove(&(p, key.to_string()));
    } else {
        core.vm_aux.flags.insert((p, key.to_string()), val);
    }
}

pub fn decode_biased_score(raw: u32) -> i32 {
    if raw == 0 {
        0
    } else {
        (i64::from(raw) - L01_SCORE_BIAS) as i32
    }
}

fn encode_biased_score(score: i32) -> u32 {
    let clamped = score.clamp(MIN_SCORE, MAX_SCORE);
    if clamped == 0 {
        0
    } else {
        (L01_SCORE_BIAS + i64::from(clamped)) as u32
    }
}

pub(crate) fn is_role_public(core: &GameStateCore, seat: PlayerId) -> bool {
    if !core.player(seat).alive {
        return true;
    }
    if flag(core, seat, "l01_role_shown") > 0 {
        return true;
    }
    core.identity_roles()
        .and_then(|rs| rs.get(seat as usize))
        .is_some_and(|r| *r == Role::Lord)
}

fn known_rebel_quota(core: &GameStateCore) -> Option<usize> {
    match &core.mode {
        GameMode::Identity { .. } => {
            let seats = core.players.len();
            if !(4..=8).contains(&seats) {
                return None;
            }
            let fix_four = seats == 4 && flag(core, 0, "l01_fix_four_applied") > 0;
            Some(match seats {
                4 => {
                    if fix_four {
                        2
                    } else {
                        1
                    }
                }
                5 => 2,
                6 => 3,
                _ => 4,
            })
        }
        _ => None,
    }
}

/// 仅凭死亡公开记录统计已知死亡反贼数，零偷看存活隐藏身份。
pub(crate) fn update_public_endgame_state(core: &mut GameStateCore, dying_seat: Option<PlayerId>) {
    let Some(quota) = known_rebel_quota(core) else {
        return;
    };
    let Some(roles) = core.identity_roles() else {
        return;
    };
    let dead_rebels = roles
        .iter()
        .enumerate()
        .filter(|(idx, r)| {
            let s = *idx as PlayerId;
            **r == Role::Rebel
                && (!core.player(s).alive
                    || (Some(s) == dying_seat && !core.player(s).death_cancelled))
        })
        .count();
    if dead_rebels >= quota {
        set_flag(core, 0, "l01_endgame_no_fan", 1);
    }
}

fn adjust_public_score(core: &mut GameStateCore, key_prefix: &str, actor: PlayerId, delta: i32) {
    let key = format!("{key_prefix}:{actor}");
    let cur = decode_biased_score(flag(core, 0, &key));
    let next = cur.saturating_add(delta);
    set_flag(core, 0, &key, encode_biased_score(next));
}

pub(crate) fn filter_l01_att(
    skill_name: &str,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    if !super::l01_rules::is_primary_alive_holder_or_dying(
        ctx.state,
        ctx.catalog,
        skill_name,
        owner,
        event,
    ) {
        return false;
    }
    let any_alive_has = |name: &str| {
        ctx.catalog
            .skill_by_name(name)
            .is_some_and(|d| ctx.state.alive_players().any(|p| ctx.state.has_skill(p, d)))
    };
    match event {
        EventData::CardUsed(_) => {
            skill_name == "attcard" || (skill_name == "_attzhi" && !any_alive_has("attcard"))
        }
        EventData::Damage(d) => {
            d.card.is_none()
                && !d.prevented
                && d.amount > 0
                && (skill_name == "attdamage"
                    || (skill_name == "_attzhi" && !any_alive_has("attdamage")))
        }
        EventData::LoseHp { .. } => {
            skill_name == "attlosehp" || (skill_name == "_attzhi" && !any_alive_has("attlosehp"))
        }
        EventData::Recover { .. } => {
            skill_name == "attrecover" || (skill_name == "_attzhi" && !any_alive_has("attrecover"))
        }
        EventData::Death { .. } => {
            skill_name == "_yingshilanggu"
                || (skill_name == "_attzhi" && !any_alive_has("_yingshilanggu"))
        }
        _ => false,
    }
}

pub(crate) fn machine_l01_att(skill: &HostSkillBehavior, owner: PlayerId, event: &EventData) -> HostMachine {
    let mut target = None;
    let mut card = None;
    let mut targets = Vec::new();
    let mut meta = (99u32, 0u32);
    match event {
        EventData::CardUsed(u) => {
            target = Some(u.player);
            card = u.card;
            targets = u.targets.clone();
            let sub = u.use_as.map_or(0u32, |k| {
                if matches!(k, CardKind::SavageAttack | CardKind::ArcheryAttack) {
                    1
                } else if k == CardKind::Tao {
                    2
                } else if k.is_sha_like()
                    || matches!(
                        k,
                        CardKind::Duel | CardKind::Dismantle | CardKind::Snatch | CardKind::Le | CardKind::Bing
                    )
                {
                    3
                } else {
                    4
                }
            });
            meta = (0, sub);
        }
        EventData::Damage(d) => {
            target = d.source;
            targets.push(d.target);
            let amt = if !d.prevented && d.amount > 0 {
                d.amount as u32
            } else {
                0
            };
            meta = (1, amt);
        }
        EventData::LoseHp { player, amount } => {
            target = Some(*player);
            meta = (2, (*amount).max(0) as u32);
        }
        EventData::Recover { player, amount, .. } => {
            target = Some(*player);
            meta = (3, *amount);
        }
        EventData::Death { player, .. } => {
            target = Some(*player);
            meta = (4, 0);
        }
        _ => {}
    }
    HostMachine {
        def_key: format!("{}|{}|{}", skill.name, meta.0, meta.1),
        behavior: skill.behavior,
        owner,
        target,
        card,
        cards: Vec::new(),
        targets,
        stage: HostStage::Act,
    }
}

fn parse_att_meta(def_key: &str) -> (u32, u32) {
    let mut parts = def_key.rsplitn(3, '|');
    let extra = parts.next().and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
    let tag = parts.next().and_then(|s| s.parse::<u32>().ok()).unwrap_or(99);
    (tag, extra)
}

pub(crate) fn step_l01_att(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let att_enabled = flag(ctx.core, 0, "l01_cfg:attceshi") > 0
        || ctx
            .catalog
            .skill_by_name(crate::content::host::L01_ATTZHI_KEY)
            .is_some_and(|def| {
                ctx.core
                    .skills
                    .iter()
                    .any(|((_, d), inst)| *d == def && inst.enabled)
            });
    if !att_enabled {
        return Flow::Done;
    }
    let lord_seat = ctx
        .core
        .identity_roles()
        .and_then(|rs| rs.iter().position(|r| *r == Role::Lord))
        .map(|i| i as PlayerId);
    let (tag, extra) = parse_att_meta(&m.def_key);
    match tag {
        0 => {
            let Some(actor) = m.target else { return Flow::Done; };
            let kind_opt = m
                .card
                .and_then(|cid| ctx.catalog.cards.get(cid as usize).map(|d| d.kind));
            let use_as_tag = (extra > 0).then_some(extra);
            let is_aoe = use_as_tag == Some(1)
                || kind_opt.is_some_and(|k| matches!(k, CardKind::SavageAttack | CardKind::ArcheryAttack));
            if is_aoe {
                return Flow::Done;
            }
            let is_tao = use_as_tag == Some(2) || (use_as_tag.is_none() && kind_opt == Some(CardKind::Tao));
            if is_tao {
                let ts = if m.targets.is_empty() {
                    vec![actor]
                } else {
                    m.targets.clone()
                };
                for t in ts {
                    if t == actor {
                        continue;
                    }
                    let target_lord_side = Some(t) == lord_seat
                        || (is_role_public(ctx.core, t)
                            && ctx.core.identity_roles().and_then(|rs| rs.get(t as usize))
                                == Some(&Role::Loyalist));
                    let target_rebel = is_role_public(ctx.core, t)
                        && ctx.core.identity_roles().and_then(|rs| rs.get(t as usize))
                            == Some(&Role::Rebel);
                    if target_lord_side {
                        adjust_public_score(ctx.core, "l01_pub_att", actor, 200);
                    } else if target_rebel {
                        adjust_public_score(ctx.core, "l01_pub_att", actor, -200);
                        adjust_public_score(ctx.core, "l01_pub_tiaofan", actor, 150);
                    }
                }
                return Flow::Done;
            }
            let is_offensive = use_as_tag == Some(3)
                || (use_as_tag.is_none()
                    && kind_opt.is_some_and(|kind| {
                        kind.is_sha_like()
                            || matches!(
                                kind,
                                CardKind::Duel
                                    | CardKind::Dismantle
                                    | CardKind::Snatch
                                    | CardKind::Le
                                    | CardKind::Bing
                            )
                    }));
            if !is_offensive {
                return Flow::Done;
            }
            for &t in &m.targets {
                if t == actor {
                    continue;
                }
                let t_lord_side = Some(t) == lord_seat
                    || (is_role_public(ctx.core, t)
                        && ctx.core.identity_roles().and_then(|rs| rs.get(t as usize))
                            == Some(&Role::Loyalist));
                let t_rebel = is_role_public(ctx.core, t)
                    && ctx.core.identity_roles().and_then(|rs| rs.get(t as usize))
                        == Some(&Role::Rebel);
                if t_lord_side {
                    adjust_public_score(ctx.core, "l01_pub_tiaofan", actor, 150);
                    adjust_public_score(ctx.core, "l01_pub_att", actor, -150);
                } else if t_rebel {
                    adjust_public_score(ctx.core, "l01_pub_att", actor, 150);
                }
                let exp_key = format!("l01_pub_exposure:{actor}");
                let cur = flag(ctx.core, 0, &exp_key);
                set_flag(ctx.core, 0, &exp_key, (cur + 15).min(100));
            }
        }
        1 => {
            let amount = extra as i32;
            if let (Some(src), Some(&t)) = (m.target, m.targets.first()) {
                if amount > 0 && src != t {
                    let is_lord_side = Some(t) == lord_seat
                        || (is_role_public(ctx.core, t)
                            && ctx.core.identity_roles().and_then(|rs| rs.get(t as usize))
                                == Some(&Role::Loyalist));
                    let is_rebel = is_role_public(ctx.core, t)
                        && ctx.core.identity_roles().and_then(|rs| rs.get(t as usize))
                            == Some(&Role::Rebel);
                    if is_lord_side {
                        adjust_public_score(ctx.core, "l01_pub_att", src, -200 * amount);
                        adjust_public_score(ctx.core, "l01_pub_tiaofan", src, 200 * amount);
                    } else if is_rebel {
                        adjust_public_score(ctx.core, "l01_pub_att", src, 200 * amount);
                    }
                    let exp_key = format!("l01_pub_exposure:{src}");
                    let cur = flag(ctx.core, 0, &exp_key);
                    set_flag(ctx.core, 0, &exp_key, (cur + 25).min(100));
                }
            }
        }
        2 => {
            let amount = extra;
            if let Some(p) = m.target {
                if amount > 0 {
                    let exp_key = format!("l01_pub_losehp:{p}");
                    let cur = flag(ctx.core, 0, &exp_key);
                    set_flag(ctx.core, 0, &exp_key, cur + amount);
                }
            }
        }
        3 => {
            let amount = extra;
            if let Some(p) = m.target {
                if amount > 0 {
                    let key = format!("l01_pub_recover:{p}");
                    let cur = flag(ctx.core, 0, &key);
                    set_flag(core_of(ctx), 0, &key, cur + amount);
                }
            }
        }
        4 => {
            update_public_endgame_state(ctx.core, m.target);
            if let Some(dead) = m.target {
                super::l01_rules::migrate_l01_trackers(ctx.core, ctx.catalog, dead);
            }
        }
        _ => {}
    }
    Flow::Done
}

fn core_of<'a>(ctx: &'a mut Ctx<'_>) -> &'a mut GameStateCore {
    ctx.core
}
