//! Hearthstone (`U05-6a`) custom `CardKind::Wuzhong` target validation helper.

use super::*;

pub(super) fn validate_hearth_wuzhong_targets(
    core: &GameStateCore,
    catalog: &SharedCatalog,
    card: CardId,
    mut cname: &str,
    targets: &[PlayerId],
) -> Option<Result<(), RejectReason>> {
    if !super::is_hearth_formal(catalog) {
        return None;
    }
    let actor = core.turn.current;
    let mut b = super::strip_hearth_base(cname);
    if b == "hsfashu_anyingjingxiang" {
        if core.declared_kinds.get(&card).copied() != Some(CardKind::Wuzhong) {
            return Some(Err(RejectReason::CardLimit));
        }
        if let Some(&src_plus) =
            core.vm_aux.flags.get(&(actor, format!("hs_wxuying_src_card:{card}")))
        {
            if src_plus > 0 {
                if let Some(cdef) = catalog.cards.get((src_plus - 1) as usize) {
                    cname = cdef.name;
                    b = super::strip_hearth_base(cname);
                }
            }
        }
        if cname == "无中生有"
            || cname == "wuzhong"
            || cname.starts_with("wuzhong_")
            || b == "hsfashu_anyingjingxiang"
        {
            if targets.is_empty() {
                return Some(Ok(()));
            }
            return Some(Err(RejectReason::IllegalTargets));
        }
    }
    if b.starts_with("tuteng") {
        return Some(Err(RejectReason::CardLimit));
    }
    if b == "shandianjian" {
        if targets.len() == 1
            && core.player(targets[0]).alive
            && !core.player(targets[0]).hand.is_empty()
        {
            return Some(Ok(()));
        }
        return Some(Err(RejectReason::IllegalTargets));
    }
    if b == "hsqizhou_tu" {
        if !targets.is_empty()
            && targets.iter().all(|&t| t != actor && core.player(t).alive)
        {
            return Some(Ok(()));
        }
        return Some(Err(RejectReason::IllegalTargets));
    }
    if b == "hsqizhou_shui" {
        let injured = core.player(actor).hp < core.player(actor).max_hp as i32;
        if injured && (targets.is_empty() || targets == [actor]) {
            return Some(Ok(()));
        }
        return Some(Err(RejectReason::IllegalTargets));
    }
    if b == "hsmengjing_huanxiaojiemei" {
        if targets.len() == 1
            && core.player(targets[0]).alive
            && core.player(targets[0]).hp < core.player(targets[0]).max_hp as i32
        {
            return Some(Ok(()));
        }
        return Some(Err(RejectReason::IllegalTargets));
    }
    if b == "hsfashu_buwendingyibian" {
        let used = core
            .vm_aux
            .flags
            .get(&(actor, "hs_buwendingyibian_used_turn".to_string()))
            .copied()
            .unwrap_or(0);
        if used as i32 >= core.player(actor).hp {
            return Some(Err(RejectReason::CardLimit));
        }
        if targets.len() == 1 && core.player(targets[0]).alive {
            return Some(Ok(()));
        }
        return Some(Err(RejectReason::IllegalTargets));
    }
    if b == "hsdusu_xueji" {
        if targets.len() == 1
            && core.player(targets[0]).alive
            && !core.player(targets[0]).equips.is_empty()
        {
            return Some(Ok(()));
        }
        return Some(Err(RejectReason::IllegalTargets));
    }
    if b == "hsdusu_huoyanhua" {
        if targets.len() == 1
            && targets[0] != actor
            && core.player(targets[0]).alive
            && core.computed_distance(catalog, actor, targets[0])
                <= core.attack_range(catalog, actor)
        {
            return Some(Ok(()));
        }
        return Some(Err(RejectReason::IllegalTargets));
    }
    if b == "hsdusu_kuyecao" {
        if targets.len() == 1
            && core.player(targets[0]).alive
            && !core.player(targets[0]).yinshen_active
        {
            return Some(Ok(()));
        }
        return Some(Err(RejectReason::IllegalTargets));
    }
    if b == "hsdusu_shinancao" {
        let unsk = targets.len() == 1
            && core.player(targets[0]).alive
            && catalog
                .skill_by_name("hsdusu_shinancao")
                .is_none_or(|d| !core.has_skill(targets[0], d));
        if unsk {
            return Some(Ok(()));
        }
        return Some(Err(RejectReason::IllegalTargets));
    }
    if b == "hsshenqi_nengliangzhiguang" {
        if core.player(actor).face_down {
            return Some(Err(RejectReason::CardLimit));
        }
        if targets.len() == 1 && core.player(targets[0]).alive {
            return Some(Ok(()));
        }
        return Some(Err(RejectReason::IllegalTargets));
    }
    if b == "hsmengjing_mengjing" {
        let ok = targets.len() == 1
            && core.player(targets[0]).alive
            && (!core.player(targets[0]).judge.iter().any(|&jc| {
                core.declared_kinds
                    .get(&jc)
                    .copied()
                    .unwrap_or_else(|| catalog.card_kind(jc))
                    == CardKind::Le
            }) || !core.player(targets[0]).equips.is_empty());
        if ok {
            return Some(Ok(()));
        }
        return Some(Err(RejectReason::IllegalTargets));
    }
    if b == "hszuzhou_nvwudexuetu" {
        let ok = targets.len() == 1
            && core.player(targets[0]).alive
            && catalog
                .skill_by_name("zhoujiang")
                .is_none_or(|d| !core.has_skill(targets[0], d));
        if ok {
            return Some(Ok(()));
        }
        return Some(Err(RejectReason::IllegalTargets));
    }
    if b == "hszuzhou_guhuo" {
        if targets.len() == 1
            && targets[0] != actor
            && core.player(targets[0]).alive
            && (!core.player(targets[0]).hand.is_empty()
                || !core.player(targets[0]).equips.is_empty())
        {
            return Some(Ok(()));
        }
        return Some(Err(RejectReason::IllegalTargets));
    }
    if matches!(b, "hsqingyu_feibiao" | "hszuzhou_nvwudeganguo") {
        if targets.len() == 1
            && core.player(targets[0]).alive
            && (!core.player(targets[0]).hand.is_empty()
                || !core.player(targets[0]).equips.is_empty())
        {
            return Some(Ok(()));
        }
        return Some(Err(RejectReason::IllegalTargets));
    }
    if matches!(
        b,
        "hsqizhou_huo"
            | "hsqingyu_zhanfang"
            | "hsqingyu_hufu"
            | "hsqingyu_shandian"
            | "hsmengjing_feicuiyoulong"
            | "hsmengjing_mengye"
            | "hszuzhou_nvwudepingguo"
    ) {
        if targets.len() == 1 && core.player(targets[0]).alive {
            return Some(Ok(()));
        }
        return Some(Err(RejectReason::IllegalTargets));
    }
    if b == "hsdusu_huangxuecao" {
        if targets.is_empty() || targets == [actor] {
            return Some(Ok(()));
        }
        return Some(Err(RejectReason::IllegalTargets));
    }
    if matches!(b, "hsshenqi_kongbusangzhong" | "hsshenqi_morijingxiang") {
        if core.player(actor).face_down {
            return Some(Err(RejectReason::CardLimit));
        }
        if b == "hsshenqi_morijingxiang"
            && !(0..core.players.len() as PlayerId).any(|p| {
                p != actor && core.player(p).alive && {
                    let pl = core.player(p);
                    !pl.hand.is_empty() || !pl.equips.is_empty() || !pl.judge.is_empty()
                }
            })
        {
            return Some(Err(RejectReason::IllegalTargets));
        }
        if targets.is_empty() {
            return Some(Ok(()));
        }
        return Some(Err(RejectReason::IllegalTargets));
    }
    if matches!(
        b,
        "shenenshu"
            | "hsjixie_zhadan"
            | "hsqizhou_feng"
            | "hsbaowu_cangbaotu"
            | "hsbaowu_huangjinyuanhou"
            | "hsmengjing_suxing"
            | "hszuzhou_wushushike"
    ) {
        if targets.is_empty() {
            return Some(Ok(()));
        }
        return Some(Err(RejectReason::IllegalTargets));
    }
    None
}
