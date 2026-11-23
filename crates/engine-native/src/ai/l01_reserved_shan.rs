//! L01 `_aiyh_reserved_shan` / `aiyh_disabled_shan` 与公开运行态同步。
//!
//! 对应上游 `AI优化/extension.js:2075-2139`：仅依据公开对局状态（阶段、出牌方手牌数、
//! 公开余杀额度、酒杀/火杀/连环队友/防具及双方公开技能）决定是否惜闪留闪。

use super::{has_equip_named, l01_public_attitude, L01AiOptions};
use crate::catalog::{CardKind, Catalog, EquipSlot};
use crate::choice::ChoiceOption;
use crate::event::{DamageNature, Phase};
use crate::ids::{CardId, PlayerId};
use crate::snap::MachineSnapshot;
use crate::view::{PlayerCellView, PlayerView, PolicyInput};
use crate::{Engine, GameAction};

/// 同步 L01 AI 判定所需的公开对局运行态（`zhenlie_lose`、活跃【杀】/酒杀/火杀与攻击者公开余杀额度）。
pub fn sync_l01_public_runtime(engine: &Engine, opts: &mut L01AiOptions) {
    let core = engine.state_core();
    let catalog = engine.catalog();
    opts.zhenlie_lose_seats
        .retain(|p| core.players.get(*p as usize).is_some_and(|pl| pl.alive));
    for ((p, k), &v) in &core.vm_aux.flags {
        if v > 0
            && k == "l01_zhenlie_lose"
            && core.players.get(*p as usize).is_some_and(|pl| pl.alive)
            && !opts.zhenlie_lose_seats.contains(p)
        {
            opts.zhenlie_lose_seats.push(*p);
        }
    }
    let sha_info = engine.resolution().parks.iter().rev().find_map(|park| {
        let snap = park.machine.snapshot();
        let mut cur = &snap;
        loop {
            match cur {
                MachineSnapshot::Sha {
                    source,
                    nature,
                    asked: true,
                    jiu_bonus,
                    xiandan_bonus,
                    ..
                } => {
                    return Some((*source, *nature, *jiu_bonus || *xiandan_bonus));
                }
                MachineSnapshot::OptionalGate {
                    inner: Some(inner), ..
                }
                | MachineSnapshot::NullifyWindow {
                    effect: Some(inner),
                    ..
                } => cur = inner,
                _ => return None,
            }
        }
    });
    if let Some((source, nature, bonus)) = sha_info {
        opts.respond_shan_is_sha = true;
        opts.respond_shan_fire = matches!(nature, DamageNature::Fire);
        opts.jiu_active = core.turn.jiu_active || bonus;
        opts.respond_shan_attacker = Some(source);
    } else {
        opts.respond_shan_is_sha = false;
        opts.respond_shan_fire = false;
        opts.jiu_active = core.turn.jiu_active;
        opts.respond_shan_attacker = Some(core.turn.current);
    }
    let attacker = opts.respond_shan_attacker.unwrap_or(core.turn.current);
    opts.attacker_sha_usable = core.turn.phase == Phase::Play
        && attacker == core.turn.current
        && attacker_has_remaining_sha(core, catalog, attacker);
}

fn attacker_has_remaining_sha(
    core: &crate::state::GameStateCore,
    catalog: &Catalog,
    attacker: PlayerId,
) -> bool {
    let has_sk = |name: &str| {
        catalog
            .skill_by_name(name)
            .is_some_and(|d| core.has_skill(attacker, d))
    };
    let paoxiao = has_sk(crate::content::standard::PAO_XIAO) || has_sk("paoxiao");
    let has_zhuge = !crate::kernel::mbcmfangzhu_baiban(core, attacker)
        && core
            .player(attacker)
            .equips
            .iter()
            .any(|c| catalog.card_kind(*c) == CardKind::Zhuge);
    let baolei = has_sk(crate::content::host::WUDIAN_BAOLEI_KEY)
        && core.player(attacker).baolei_mode == 1;
    let limu = !core.player(attacker).judge.is_empty()
        && !catalog.is_tieji_sealed(core, attacker)
        && has_sk("xinfu_limu")
        && core.alive_players().any(|t| {
            t != attacker
                && core.computed_distance(catalog, attacker, t)
                    <= core.attack_range(catalog, attacker)
        });
    let (vm_limit, vm_unlim) = crate::rvm_bridge::modifiers::eval_sha_use_limit(
        core,
        catalog,
        attacker,
        None,
        CardKind::Sha,
        DamageNature::Normal,
        1,
    );
    let vm_used = core
        .vm_aux
        .flags
        .get(&(attacker, "vm_sha_extra_used".to_string()))
        .copied()
        .unwrap_or(0);
    if paoxiao
        || has_zhuge
        || baolei
        || limu
        || vm_unlim
        || (vm_limit as i32) > 1 + (vm_used as i32)
    {
        return true;
    }
    if vm_limit == 0 {
        return false;
    }
    if !core.turn.sha_used_play_phase {
        return true;
    }
    let unused_once = |name: &str| {
        catalog.skill_by_name(name).is_some_and(|def| {
            core.skills
                .get(&(attacker, def))
                .is_some_and(|i| i.enabled && !i.once_used)
        })
    };
    let lingshi = unused_once("lingshi")
        && core
            .player(attacker)
            .equips
            .iter()
            .any(|c| catalog.card_slot_of(core, *c) == Some(EquipSlot::Treasure));
    let sbzx = has_sk(crate::content::host::L27_SBZHAXIANG_KEY)
        && crate::content::host::sbzhaxiang_active(core, attacker);
    let dcdichou = core
        .vm_aux
        .marks
        .get(&(attacker, 2_754_298_115))
        .copied()
        .unwrap_or(0)
        > core
            .vm_aux
            .flags
            .get(&(attacker, "dcdichou_eff_sha_used".to_string()))
            .copied()
            .unwrap_or(0) as i32;
    (core.current_land == Some(CardKind::Longlushanfeng) && !core.turn.land_sha_used)
        || unused_once(crate::content::host::GJ_FOOD_YUANBAOROU_KEY)
        || unused_once(crate::content::standard::BOSS_HUIHUO)
        || lingshi
        || sbzx
        || dcdichou
}

fn linked_favors_dodge(view: &PlayerView, opts: Option<&L01AiOptions>, seat: u8) -> bool {
    if !view.players.get(seat as usize).is_some_and(|p| p.linked) {
        return false;
    }
    let (mut cxdy, mut cxdr, mut hzdy, mut hzdr) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
    for p in &view.players {
        if !p.alive || p.seat == seat || !p.linked {
            continue;
        }
        let att = l01_public_attitude(view, opts, seat, p.seat);
        if att > 0 {
            hzdy += 1.0;
            if p.hp < 2 {
                cxdy += 1.0;
            }
        } else if att < 0 {
            hzdr += 1.0;
            if p.hp < 2 {
                cxdr += 1.0;
            }
        }
    }
    cxdy > cxdr || (hzdy + 2.1 * cxdy) > (hzdr + 2.1 * cxdr)
}

fn reserved_shan_blocked_by_skills(
    catalog: &Catalog,
    me: &PlayerCellView,
    attacker: Option<&PlayerCellView>,
    is_fire: bool,
) -> bool {
    let att_unequip = attacker.is_some_and(|a| {
        has_equip_named(catalog, a, "qinggang")
            || a.skills
                .iter()
                .any(|s| matches!(*s, "unequip" | "tieji" | "铁骑" | "无双"))
    });
    let def_unequip = me.skills.iter().any(|s| *s == "unequip2");
    if !att_unequip && !def_unequip {
        let has_bagua = me.equips.iter().any(|c| {
            crate::rvm_bridge::modifiers::d34_l28_mods::is_bagua_armor_card(catalog, *c)
        });
        if has_bagua || me.skills.iter().any(|s| matches!(*s, "bazhen" | "八阵")) {
            return true;
        }
        if is_fire
            && (has_equip_named(catalog, me, "tengjia")
                || me.skills.iter().any(|s| *s == "tengjia2"))
        {
            return true;
        }
    }
    if attacker.is_some_and(|a| {
        has_equip_named(catalog, a, "hanbing")
            || a.skills.iter().any(|s| {
                matches!(
                    *s,
                    "luoyi2"
                        | "reluoyi2"
                        | "luoyi"
                        | "裸衣"
                        | "repojun"
                        | "hanbing_skill"
                        | "xinfu_zengdao2"
                        | "xinliegong"
                        | "liegong"
                        | "烈弓"
                        | "anjian"
                        | "reanjian"
                        | "xinanjian"
                        | "lkzhongzhuang"
                        | "decadexianzhen2"
                        | "twxuhe_damage"
                        | "twchuanshu_effect"
                        | "xinqingxi"
                        | "twqingxi"
                        | "nsshijun"
                        | "liyong2"
                        | "wangong2"
                        | "pyzhuren_diamond"
                        | "tuxing2"
                        | "spjiedao"
                        | "dcyiyong"
                        | "rezhiman"
                )
            })
    }) {
        return true;
    }
    me.skills.iter().any(|s| {
        matches!(
            *s,
            "useShan"
                | "respondShan"
                | "zhenlie"
                | "贞烈"
                | "bazhen"
                | "八阵"
                | "juezhen"
                | "lingren_adddamage"
                | "cuijin"
                | "shouli_thunder"
                | "twyiju"
                | "twzhongchi_effect"
                | "reyanzhu2"
                | "twqiongji"
                | "yise_damage"
                | "dcshizhao_effect"
                | "xionghuo"
                | "dcfudao_deadmark"
        )
    })
}

pub(super) fn plan_l01_reserved_shan(
    catalog: &Catalog,
    input: &PolicyInput,
    me: &PlayerCellView,
    opts: &L01AiOptions,
    id: u64,
    options: &[ChoiceOption],
) -> Option<GameAction> {
    let cards: Vec<CardId> = options
        .iter()
        .filter_map(|o| match o {
            ChoiceOption::Card(c) => Some(*c),
            _ => None,
        })
        .collect();
    if cards.is_empty() {
        return options
            .contains(&ChoiceOption::Pass)
            .then(|| GameAction::Answer {
                choice_id: id,
                selection: vec![ChoiceOption::Pass],
            });
    }
    let seat = input.view.seat;
    let attacker_seat = opts.respond_shan_attacker.unwrap_or(input.view.turn);
    let attacker_opt = input.view.players.get(attacker_seat as usize);
    let should_hold = cards.len() == 1
        && opts.respond_shan_is_sha
        && input.view.phase == Phase::Play
        && attacker_seat == input.view.turn
        && opts.attacker_sha_usable
        && !opts.jiu_active
        && me.hp >= 2
        && attacker_opt.is_some_and(|a| a.hand_count >= 2)
        && options.contains(&ChoiceOption::Pass)
        && !linked_favors_dodge(&input.view, Some(opts), seat)
        && !reserved_shan_blocked_by_skills(catalog, me, attacker_opt, opts.respond_shan_fire);
    if should_hold {
        Some(GameAction::Answer {
            choice_id: id,
            selection: vec![ChoiceOption::Pass],
        })
    } else {
        Some(GameAction::Answer {
            choice_id: id,
            selection: vec![ChoiceOption::Card(cards[0])],
        })
    }
}
