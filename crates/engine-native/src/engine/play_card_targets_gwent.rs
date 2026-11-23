//! 昆特牌 (`U05-5a`) 出牌目标校验辅助。

use super::*;

pub(in crate::engine) fn clean_gwent_name(name: &str) -> &str {
    name.rsplit_once('_')
        .and_then(|(head, tail)| {
            (!tail.is_empty() && tail.chars().all(|c| c.is_ascii_digit())).then_some(head)
        })
        .unwrap_or(name)
}

pub(super) fn validate_u05_gwent_targets(
    core: &GameStateCore,
    catalog: &SharedCatalog,
    cname: &str,
    targets: &[PlayerId],
) -> Result<(), RejectReason> {
    let cname = clean_gwent_name(cname);
    let cur = core.turn.current;
    let has_skill = |p: PlayerId, k: &str| {
        catalog
            .skill_by_name(k)
            .is_some_and(|def| core.skills.contains_key(&(p, def)))
    };
    let distinct_alive = |ts: &[PlayerId]| {
        ts.iter()
            .all(|&p| (p as usize) < core.players.len() && core.player(p).alive)
            && {
                let mut seen = Vec::with_capacity(ts.len());
                ts.iter().all(|p| {
                    if seen.contains(p) {
                        false
                    } else {
                        seen.push(*p);
                        true
                    }
                })
            }
    };
    match cname {
        "gwjinli_jiu" | "gw_dieyi" | "gw_fuyuan" | "gw_dudayuanshuai1" | "gw_dudayuanshuai2" => {
            Err(RejectReason::WrongPhase)
        }
        "gw_xianzumaijiu" => {
            if !targets.is_empty() {
                return Err(RejectReason::IllegalTargets);
            }
            let used = core
                .vm_aux
                .flags
                .get(&(cur, "gw_xianzumaijiu_used_play".to_string()))
                .copied()
                .unwrap_or(0)
                > 0;
            if used || has_skill(cur, "gw_xianzumaijiu") {
                return Err(RejectReason::CardLimit);
            }
            Ok(())
        }
        "gw_guaiwuchaoxue" => {
            if !targets.is_empty() {
                return Err(RejectReason::IllegalTargets);
            }
            let used = core
                .vm_aux
                .flags
                .get(&(cur, "gw_guaiwuchaoxue_used_play".to_string()))
                .copied()
                .unwrap_or(0)
                > 0;
            if used {
                return Err(RejectReason::CardLimit);
            }
            Ok(())
        }
        "gw_hudiewu" => {
            if !targets.is_empty() {
                return Err(RejectReason::IllegalTargets);
            }
            let ok = (0..core.players.len() as PlayerId).any(|p| {
                p != cur
                    && core.player(p).alive
                    && (!core.player(p).equips.is_empty() || !core.player(p).judge.is_empty())
            });
            if ok { Ok(()) } else { Err(RejectReason::IllegalTargets) }
        }
        "gw_aerdeyin" => {
            if !targets.is_empty() {
                return Err(RejectReason::IllegalTargets);
            }
            let ok = (0..core.players.len() as PlayerId).any(|p| {
                p != cur && core.player(p).alive && !core.is_friend_of(catalog, cur, p)
            });
            if ok { Ok(()) } else { Err(RejectReason::IllegalTargets) }
        }
        "gw_ansha" => {
            if !targets.is_empty() {
                return Err(RejectReason::IllegalTargets);
            }
            let ok = (0..core.players.len() as PlayerId).any(|p| {
                p != cur
                    && core.player(p).alive
                    && !core.is_friend_of(catalog, cur, p)
                    && core.player(p).hp == 1
            });
            if ok { Ok(()) } else { Err(RejectReason::IllegalTargets) }
        }
        "gw_xinsheng" => {
            if !targets.is_empty() {
                return Err(RejectReason::IllegalTargets);
            }
            let ok = (0..core.players.len() as PlayerId).any(|p| {
                core.player(p).alive
                    && (!matches!(core.mode, crate::state::GameMode::Kingdoms)
                        || core.player(p).revealed_generals[0]
                        || core.player(p).revealed_generals[1])
            });
            if ok { Ok(()) } else { Err(RejectReason::IllegalTargets) }
        }
        "gw_niuquzhijing" => {
            if !targets.is_empty() {
                return Err(RejectReason::IllegalTargets);
            }
            let cur_hp = core.player(cur).hp;
            let ok = (0..core.players.len() as PlayerId)
                .any(|p| core.player(p).alive && core.player(p).hp != cur_hp);
            if ok { Ok(()) } else { Err(RejectReason::IllegalTargets) }
        }
        "gw_tunshi" => {
            if !targets.is_empty() {
                return Err(RejectReason::IllegalTargets);
            }
            if core.player(cur).max_hp <= 1 {
                return Err(RejectReason::IllegalTargets);
            }
            let ok = (0..core.players.len() as PlayerId).any(|p| {
                p != cur
                    && core.player(p).alive
                    && !core.is_friend_of(catalog, cur, p)
                    && core.skills.iter().any(|((o, def), _)| {
                        *o == p
                            && !catalog.is_charlotte_skill(*def)
                            && !core.player(p).equips.iter().any(|ec| {
                                catalog.cards[*ec as usize]
                                    .equip_skills
                                    .contains(&catalog.skills[*def as usize].name)
                            })
                    })
            });
            if ok { Ok(()) } else { Err(RejectReason::IllegalTargets) }
        }
        "gw_yigeniyin"
        | "gw_leizhoushu"
        | "gw_zhongmozhizhan"
        | "gw_ganhan"
        | "gw_huangjiashenpan"
        | "gw_chongci"
        | "gw_shizizhaohuan"
        | "gw_zuihouyuanwang"
        | "gw_zirankuizeng"
        | "gwmaoxian_yenaifa"
        | "gwmaoxian_telisi"
        | "gwmaoxian_hengsaite"
        | "gwmaoxian_bulanwang"
        | "gwmaoxian_haluo"
        | "gwmaoxian_airuiting"
        | "gwmaoxian_aisinie"
        | "gwmaoxian_falanxisika"
        | "gwmaoxian_huoge" => {
            if targets.is_empty() { Ok(()) } else { Err(RejectReason::IllegalTargets) }
        }
        "gw_youer" | "gw_tongdi" => {
            if targets.len() == 1
                && targets[0] != cur
                && core.player(targets[0]).alive
                && !core.player(targets[0]).hand.is_empty()
            {
                Ok(())
            } else {
                Err(RejectReason::IllegalTargets)
            }
        }
        "gw_nuhaifengbao" => {
            if targets.len() == 1
                && core.player(targets[0]).alive
                && !has_skill(targets[0], "gw_nuhaifengbao")
            {
                Ok(())
            } else {
                Err(RejectReason::IllegalTargets)
            }
        }
        "gw_butianshu" | "gwmaoxian_jieluote" | "gwmaoxian_laduoweide" | "gwmaoxian_gaier" => {
            if targets.len() == 1 && core.player(targets[0]).alive {
                Ok(())
            } else {
                Err(RejectReason::IllegalTargets)
            }
        }
        "gwmaoxian_luoqi" => {
            if targets.len() == 1 && targets[0] != cur && core.player(targets[0]).alive {
                Ok(())
            } else {
                Err(RejectReason::IllegalTargets)
            }
        }
        "gwmaoxian_enxier" => {
            let h0 = core.player(cur).hand.len() as i32;
            if targets.len() == 1
                && core.player(targets[0]).alive
                && (core.player(targets[0]).hand.len() as i32 - h0).abs() <= 1
            {
                Ok(())
            } else {
                Err(RejectReason::IllegalTargets)
            }
        }
        "gwmaoxian_kuite" => {
            let h0 = core.player(cur).hand.len();
            if targets.len() == 1
                && targets[0] != cur
                && core.player(targets[0]).alive
                && core.player(targets[0]).hand.len() >= h0
            {
                Ok(())
            } else {
                Err(RejectReason::IllegalTargets)
            }
        }
        "gwmaoxian_dagong" => {
            if targets.len() != 1 || !core.player(targets[0]).alive {
                return Err(RejectReason::IllegalTargets);
            }
            let t = targets[0];
            let has_cigu = has_skill(t, crate::content::host::GW_CIGU_STATE_KEY)
                || has_skill(t, "gw_ciguhanshuang");
            let has_birin = has_skill(t, crate::content::host::GW_BIRIN_STATE_KEY)
                || has_skill(t, "gw_birinongwu");
            let has_qinpen = has_skill(t, crate::content::host::GW_QINPEN_STATE_KEY)
                || has_skill(t, "gw_qinpendayu");
            if !has_cigu || !has_birin || !has_qinpen {
                Ok(())
            } else {
                Err(RejectReason::IllegalTargets)
            }
        }
        "gw_zhuoshao" => {
            let max_hp = core.alive_players().map(|p| core.player(p).hp).max();
            if !targets.is_empty()
                && distinct_alive(targets)
                && max_hp.is_some_and(|mhp| targets.iter().all(|&p| core.player(p).hp == mhp))
            {
                Ok(())
            } else {
                Err(RejectReason::IllegalTargets)
            }
        }
        "gw_baishuang" => {
            if (1..=3).contains(&targets.len())
                && distinct_alive(targets)
                && targets.iter().all(|&t| {
                    !has_skill(t, crate::content::host::GW_CIGU_STATE_KEY)
                        && !has_skill(t, "gw_ciguhanshuang")
                })
            {
                Ok(())
            } else {
                Err(RejectReason::IllegalTargets)
            }
        }
        "gw_baobaoshu" => {
            if (1..=2).contains(&targets.len())
                && distinct_alive(targets)
                && targets.iter().all(|&t| !has_skill(t, "gw_baobaoshu"))
            {
                Ok(())
            } else {
                Err(RejectReason::IllegalTargets)
            }
        }
        "gwmaoxian_yioufeisi" => {
            if core.alive_count() > 2
                && targets.len() == 2
                && distinct_alive(targets)
                && targets.iter().all(|&t| t != cur)
            {
                Ok(())
            } else {
                Err(RejectReason::IllegalTargets)
            }
        }
        "gwmaoxian_fuertaisite" => {
            if (1..=2).contains(&targets.len()) && distinct_alive(targets) {
                Ok(())
            } else {
                Err(RejectReason::IllegalTargets)
            }
        }
        "gwmaoxian_fulisi" => {
            if (1..=3).contains(&targets.len())
                && distinct_alive(targets)
                && targets
                    .iter()
                    .all(|&t| t != cur && !core.player(t).hand.is_empty())
            {
                Ok(())
            } else {
                Err(RejectReason::IllegalTargets)
            }
        }
        "gwmaoxian_kaerweite" => {
            if (1..=2).contains(&targets.len())
                && distinct_alive(targets)
                && targets
                    .iter()
                    .all(|&t| t != cur && !core.player(t).hand.is_empty())
            {
                Ok(())
            } else {
                Err(RejectReason::IllegalTargets)
            }
        }
        _ => {
            if targets.is_empty() || (targets.len() == 1 && core.player(targets[0]).alive) {
                Ok(())
            } else {
                Err(RejectReason::IllegalTargets)
            }
        }
    }
}

/// 昆特牌 (`U05-5a`) 铜卡法术/天气出牌目标校验。
pub(super) fn validate_u05_gwent_bronze_targets(
    core: &GameStateCore,
    catalog: &SharedCatalog,
    kind: CardKind,
    targets: &[PlayerId],
) -> Result<(), RejectReason> {
    match kind {
        CardKind::Kunenfayin => {
            // 昆恩法印（`card/gwent/card.js:1758`）：恰 1 名未持有昆恩的存活角色（含自己）
            if targets.len() != 1 || !core.player(targets[0]).alive {
                return Err(RejectReason::IllegalTargets);
            }
            let has_kunen = [
                crate::content::host::GW_KUNEN_STATE_KEY,
                "gw_kunenfayin",
            ]
            .into_iter()
            .any(|k| {
                catalog
                    .skill_by_name(k)
                    .is_some_and(|def| core.skills.contains_key(&(targets[0], def)))
            });
            if has_kunen {
                return Err(RejectReason::IllegalTargets);
            }
        }
        CardKind::Poxiao => {
            // 破晓：无目标
            if !targets.is_empty() {
                return Err(RejectReason::IllegalTargets);
            }
        }
        CardKind::Zumoshoukao => {
            // 祖墨手铠（`card/gwent/card.js:1616`）：恰 1 名存活角色（含自己），需有护甲或未封印
            if targets.len() != 1 || !core.player(targets[0]).alive {
                return Err(RejectReason::IllegalTargets);
            }
            let t = targets[0];
            let sealed = catalog
                .skill_by_name(crate::content::host::GW_FENGYIN_STATE_KEY)
                .is_some_and(|def| core.skills.contains_key(&(t, def)));
            if core.player(t).hujia == 0 && sealed {
                return Err(RejectReason::IllegalTargets);
            }
        }
        CardKind::Shanbengshu => {
            // 山崩术（`card/gwent/card.js:1850`）：无目标；需存在至少一名有装备的敌方
            if !targets.is_empty() {
                return Err(RejectReason::IllegalTargets);
            }
            let current = core.turn.current;
            let any_enemy_equip = (0..core.players.len() as PlayerId).any(|p| {
                p != current
                    && core.player(p).alive
                    && !core.is_friend_of(catalog, current, p)
                    && !core.player(p).equips.is_empty()
            });
            if !any_enemy_equip {
                return Err(RejectReason::IllegalTargets);
            }
        }
        CardKind::Yanziyaoshui | CardKind::Baoxueyaoshui | CardKind::Liuxinghuoyu => {
            // 燕子药水/暴雪药水/流星火雨：恰 1 名存活目标
            if targets.len() != 1 || !core.player(targets[0]).alive {
                return Err(RejectReason::IllegalTargets);
            }
        }
        CardKind::Aozuzhilei => {
            // 艾祖之雷（`card/gwent/card.js:1648`）：恰 1 名体力 >= 使用者的存活角色
            if targets.len() != 1
                || !core.player(targets[0]).alive
                || core.player(targets[0]).hp < core.player(core.turn.current).hp
            {
                return Err(RejectReason::IllegalTargets);
            }
        }
        CardKind::Zhihuanjun => {
            // 致幻菌（`card/gwent/card.js:1585`）：恰 1 名已受伤存活角色
            if targets.len() != 1
                || !core.player(targets[0]).alive
                || core.player(targets[0]).hp >= core.player(targets[0]).max_hp as i32
            {
                return Err(RejectReason::IllegalTargets);
            }
        }
        CardKind::Wenyi => {
            // 瘟疫（`card/gwent/card.js:1814`）：全自动锁定全体体力最少者，`targets` 为空
            if !targets.is_empty() {
                return Err(RejectReason::IllegalTargets);
            }
        }
        CardKind::Qinpendayu | CardKind::Birinongwu | CardKind::Ciguhanshuang => {
            // gwent 天气（`card/gwent/card.js:1456-1545`）：恰 1 名未持有该天气的存活角色（含自己）
            if targets.len() != 1 || !core.player(targets[0]).alive {
                return Err(RejectReason::IllegalTargets);
            }
            let t = targets[0];
            let (k1, k2) = match kind {
                CardKind::Qinpendayu => (
                    crate::content::host::GW_QINPEN_STATE_KEY,
                    "gw_qinpendayu",
                ),
                CardKind::Birinongwu => (
                    crate::content::host::GW_BIRIN_STATE_KEY,
                    "gw_birinongwu",
                ),
                _ => (
                    crate::content::host::GW_CIGU_STATE_KEY,
                    "gw_ciguhanshuang",
                ),
            };
            let already = [k1, k2].into_iter().any(|k| {
                catalog
                    .skill_by_name(k)
                    .is_some_and(|def| core.skills.contains_key(&(t, def)))
            });
            if already {
                return Err(RejectReason::IllegalTargets);
            }
        }
        _ => {}
    }
    Ok(())
}
