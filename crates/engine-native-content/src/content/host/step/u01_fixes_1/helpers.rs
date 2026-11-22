//! Target legality, Sha play-phase limit, and card-name/target helpers for `u01_fixes_1`.

use super::*;

pub(super) fn card_zh_name_len(catalog: &Catalog, card: CardId) -> usize {
    let def = &catalog.cards[card as usize];
    let raw = def.name.rsplit(':').next().unwrap_or(def.name);
    match raw {
        "sha" | "huosha" | "leisha" | "fire_sha" | "sha_fire" | "sha_thunder" | "shan" | "tao"
        | "jiu" => 1,
        "juedou" | "duel" | "shandian" | "shandian_red" | "tengjia" | "huogong" | "chitu"
        | "dayuan" | "zixing" | "jueying" | "dilu" | "hualiu" | "kuwu" | "juedi" | "dujian" => 2,
        "bagua" | "renwang" | "qinggang" | "hanbing" | "guanshi" | "qilin" | "guding"
        | "xuanyuanjian" | "shoulijian" | "wuxingpan" | "shenmiguo" | "lianyaohu" | "shuigong" => {
            3
        }
        "baiyin" | "zhuque" | "zhangba" | "fangtian" | "zhuge" | "muniu" | "bintie"
        | "zhuahuang" | "zhuahuangfeidian" | "wuzhong" | "taoyuan" | "wugu" | "nanman"
        | "wanjian" | "guohe" | "shunshou" | "wuxie" | "tiesuo" | "lebu" | "bingliang"
        | "jiedao" | "savage" | "archery" | "dismantle" | "snatch" | "nullify" | "chain" | "le"
        | "bing" | "shuchui" | "jinlianzhu" | "youdishenru" | "wangmeizhike" | "wangmei_zhike" => {
            4
        }
        "qinglong" | "cixiong" | "sanjian" => 5,
        _ => match def.kind {
            CardKind::Sha
            | CardKind::FireSha
            | CardKind::LeiSha
            | CardKind::Shan
            | CardKind::Tao
            | CardKind::Jiu => 1,
            CardKind::Duel | CardKind::ShanDian => 2,
            CardKind::ShouLiJian
            | CardKind::WuxingPan
            | CardKind::Xuanyuanjian
            | CardKind::Qilin
            | CardKind::Renwang
            | CardKind::Qinggang
            | CardKind::Hanbing
            | CardKind::Guanshi
            | CardKind::Shenmiguo
            | CardKind::Lianyaohu
            | CardKind::Shuigong => 3,
            CardKind::WangmeiZhike
            | CardKind::Zhuge
            | CardKind::Zhangba
            | CardKind::Fangtian
            | CardKind::Le
            | CardKind::Bing
            | CardKind::Dismantle
            | CardKind::Snatch
            | CardKind::Nullify
            | CardKind::Chain
            | CardKind::Wuzhong
            | CardKind::Taoyuan
            | CardKind::Wugu
            | CardKind::SavageAttack
            | CardKind::ArcheryAttack
            | CardKind::Jiedao
            | CardKind::Jinlianzhu
            | CardKind::Youdishenru
            | CardKind::Shuchui => 4,
            CardKind::Sanjian | CardKind::Cixiong | CardKind::Qinglong => 5,
            CardKind::HorsePlus | CardKind::HorseMinus => {
                if raw.contains("zhuahuang") {
                    4
                } else {
                    2
                }
            }
            CardKind::Weapon | CardKind::Armor => {
                if raw.contains("sanjian") || raw.contains("qinglong") || raw.contains("cixiong") {
                    5
                } else if raw.contains("fangtian")
                    || raw.contains("zhuge")
                    || raw.contains("bintie")
                    || raw.contains("zhangba")
                    || raw.contains("zhuque")
                    || raw.contains("baiyin")
                {
                    4
                } else if raw.contains("kuwu") || raw.contains("juedi") || raw.contains("tengjia") {
                    2
                } else {
                    3
                }
            }
            _ => 2,
        },
    }
}

pub(in crate::content::host::step) fn is_target_kongcheng(
    core: &GameStateCore,
    catalog: &Catalog,
    t: PlayerId,
) -> bool {
    catalog
        .skill_by_name(crate::vocabulary::names::KONG_CHENG)
        .is_some_and(|d| core.has_skill(t, d))
        && core.player(t).hand.is_empty()
}

pub(in crate::content::host::step) fn is_target_protected(
    core: &GameStateCore,
    t: PlayerId,
) -> bool {
    let p = core.player(t);
    p.jidong_active || p.yinshen_active || p.jileng_active || p.yinmi_active
}

pub(in crate::content::host::step) fn is_sha_target_blocked(
    core: &GameStateCore,
    catalog: &Catalog,
    user: PlayerId,
    t: PlayerId,
    kind: CardKind,
    nature: DamageNature,
) -> bool {
    if is_target_kongcheng(core, catalog, t) || is_target_protected(core, t) {
        return true;
    }
    if catalog
        .skill_by_name(crate::content::standard::BOSS_REMEN)
        .is_some_and(|def| {
            core.has_skill(t, def)
                && !core
                    .player(t)
                    .equips
                    .iter()
                    .any(|c| catalog.card_slot_of(core, *c) == Some(EquipSlot::Armor))
        })
    {
        return true;
    }
    crate::rvm_bridge::modifiers::eval_card_target_canceled(
        core, catalog, user, t, 0, kind, nature,
    )
}

pub(crate) fn is_sha_legal_play_target(
    core: &GameStateCore,
    catalog: &Catalog,
    user: PlayerId,
    t: PlayerId,
) -> bool {
    if t == user || !core.player(t).alive {
        return false;
    }
    let atk = core.attack_range(catalog, user);
    let unlimited = crate::rvm_bridge::modifiers::eval_card_distance_unlimited(
        core,
        catalog,
        user,
        0,
        CardKind::Sha,
        DamageNature::Normal,
    );
    (unlimited || core.computed_distance(catalog, user, t) <= atk)
        && !is_sha_target_blocked(core, catalog, user, t, CardKind::Sha, DamageNature::Normal)
}

fn sha_play_has_unlimited(core: &GameStateCore, catalog: &Catalog, player: PlayerId) -> bool {
    let paoxiao = catalog
        .skill_by_name(crate::content::standard::PAO_XIAO)
        .is_some_and(|d| core.has_skill(player, d));
    let has_zhuge = core
        .player(player)
        .equips
        .iter()
        .any(|c| catalog.card_kind(*c) == CardKind::Zhuge);
    let baolei = catalog
        .skill_by_name(crate::content::host::WUDIAN_BAOLEI_KEY)
        .is_some_and(|d| core.has_skill(player, d))
        && core.player(player).baolei_mode == 1;
    let sbzhaxiang = catalog
        .skill_by_name(crate::content::host::L27_SBZHAXIANG_KEY)
        .is_some_and(|d| core.has_skill(player, d))
        && crate::content::host::sbzhaxiang_active(core, player);
    let (_, vm_unlimited) = crate::rvm_bridge::modifiers::eval_sha_use_limit(
        core,
        catalog,
        player,
        None,
        CardKind::Sha,
        DamageNature::Normal,
        1,
    );
    paoxiao || has_zhuge || baolei || sbzhaxiang || vm_unlimited
}

pub(crate) fn can_use_sha_play_phase(
    core: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
) -> bool {
    if core.turn.current != player || core.turn.phase != Phase::Play {
        return false;
    }
    if catalog.blocks_all_cards(core, player) {
        return false;
    }
    let has_enabled = |key: &str| {
        catalog
            .skill_by_name(key)
            .is_some_and(|d| core.skills.get(&(player, d)).is_some_and(|i| i.enabled))
    };
    if has_enabled(crate::content::host::GW_BIRIN_STATE_KEY)
        || has_enabled(crate::content::host::WUDIAN_SHOUHU_KEY)
        || (has_enabled(crate::content::standard::BOSS_QINGZHU)
            && !catalog
                .skill_by_name(crate::content::standard::BOSS_JIDING)
                .is_some_and(|d| core.has_skill(player, d)))
    {
        return false;
    }
    let (vm_limit, vm_unlimited) = crate::rvm_bridge::modifiers::eval_sha_use_limit(
        core,
        catalog,
        player,
        None,
        CardKind::Sha,
        DamageNature::Normal,
        1,
    );
    if sha_play_has_unlimited(core, catalog, player) || vm_unlimited {
        return true;
    }
    if vm_limit == 0 {
        return false;
    }
    if !core.turn.sha_used_play_phase {
        return true;
    }
    let land_bonus =
        core.current_land == Some(CardKind::Longlushanfeng) && !core.turn.land_sha_used;
    let unused = |key: &str| {
        catalog
            .skill_by_name(key)
            .is_some_and(|d| core.skills.get(&(player, d)).is_some_and(|i| i.enabled && !i.once_used))
    };
    let lingshi = unused("lingshi")
        && core
            .player(player)
            .equips
            .iter()
            .any(|c| catalog.card_slot_of(core, *c) == Some(EquipSlot::Treasure));
    let dcdichou = {
        let has = catalog
            .skill_by_name("dcdichou_eff")
            .is_some_and(|d| core.has_skill(player, d))
            || flag(core, player, "skill:2754298115") != 0;
        let bonus = core
            .vm_aux
            .marks
            .get(&(player, 2_754_298_115))
            .copied()
            .unwrap_or(0);
        let used = flag(core, player, "dcdichou_eff_sha_used") as i32;
        has && bonus > used
    };
    let vm_extra = vm_limit > 1 + flag(core, player, "vm_sha_extra_used");
    land_bonus
        || unused(crate::content::host::GJ_FOOD_YUANBAOROU_KEY)
        || unused(crate::content::standard::BOSS_HUIHUO)
        || lingshi
        || dcdichou
        || vm_extra
}

pub(crate) fn consume_sha_play_phase(
    core: &mut GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
) {
    if !core.turn.sha_used_play_phase || sha_play_has_unlimited(core, catalog, player) {
        core.turn.sha_used_play_phase = true;
        return;
    }
    if core.current_land == Some(CardKind::Longlushanfeng) && !core.turn.land_sha_used {
        return;
    }
    let has_treasure = core
        .player(player)
        .equips
        .iter()
        .any(|c| catalog.card_slot_of(core, *c) == Some(EquipSlot::Treasure));
    for key in [
        crate::content::host::GJ_FOOD_YUANBAOROU_KEY,
        crate::content::standard::BOSS_HUIHUO,
        "lingshi",
    ] {
        let extra_ok = key != "lingshi" || has_treasure;
        if !extra_ok {
            continue;
        }
        if let Some(def) = catalog.skill_by_name(key) {
            if let Some(inst) = core.skills.get_mut(&(player, def)) {
                if inst.enabled && !inst.once_used {
                    inst.once_used = true;
                    return;
                }
            }
        }
    }
    let has_dc = catalog
        .skill_by_name("dcdichou_eff")
        .is_some_and(|d| core.has_skill(player, d))
        || flag(core, player, "skill:2754298115") != 0;
    let dc_bonus = core
        .vm_aux
        .marks
        .get(&(player, 2_754_298_115))
        .copied()
        .unwrap_or(0);
    let dc_used = flag(core, player, "dcdichou_eff_sha_used");
    if has_dc && dc_bonus > dc_used as i32 {
        set_flag(core, player, "dcdichou_eff_sha_used", dc_used + 1);
        return;
    }
    let vm_used = flag(core, player, "vm_sha_extra_used");
    set_flag(core, player, "vm_sha_extra_used", vm_used + 1);
}

pub(in crate::content::host::step) fn jiedao_victims_for(
    core: &GameStateCore,
    catalog: &Catalog,
    holder: PlayerId,
) -> Vec<PlayerId> {
    let atk = core.attack_range(catalog, holder);
    core.alive_players()
        .filter(|&v| {
            v != holder
                && core.computed_distance(catalog, holder, v) <= atk
                && !is_sha_target_blocked(
                    core,
                    catalog,
                    holder,
                    v,
                    CardKind::Sha,
                    DamageNature::Normal,
                )
        })
        .collect()
}

pub(super) fn canonical_virt_kind(catalog: &Catalog, card: CardId) -> CardKind {
    let raw = catalog.cards[card as usize].name;
    let key = raw.rsplit(':').next().unwrap_or(raw);
    if matches!(key, "huogong" | "火攻") {
        CardKind::Linghunzhihuo
    } else {
        catalog.card_kind(card)
    }
}

pub(super) fn canonical_jijian_key(kind: CardKind) -> u32 {
    if kind.is_sha_like() {
        u32::from(CardKind::Sha as u16)
    } else {
        u32::from(kind as u16)
    }
}

pub(in crate::content::host::step) fn virtual_legal_targets(
    core: &GameStateCore,
    catalog: &Catalog,
    user: PlayerId,
    kind: CardKind,
) -> Vec<PlayerId> {
    let nature = match kind {
        CardKind::FireSha | CardKind::Linghunzhihuo => DamageNature::Fire,
        CardKind::LeiSha => DamageNature::Thunder,
        _ => DamageNature::Normal,
    };
    if kind.is_sha_like() {
        let atk = core.attack_range(catalog, user);
        let unlimited = crate::rvm_bridge::modifiers::eval_card_distance_unlimited(
            core, catalog, user, 0, kind, nature,
        );
        return core
            .alive_players()
            .filter(|&t| {
                t != user
                    && (unlimited || core.computed_distance(catalog, user, t) <= atk)
                    && !is_sha_target_blocked(core, catalog, user, t, kind, nature)
            })
            .collect();
    }
    match kind {
        CardKind::Tao => (core.player(user).hp < core.player(user).max_hp as i32)
            .then_some(vec![user])
            .unwrap_or_default(),
        CardKind::Jiu => (!crate::rvm_bridge::modifiers::eval_card_target_canceled(
            core,
            catalog,
            user,
            user,
            0,
            kind,
            nature,
        ))
        .then_some(vec![user])
        .unwrap_or_default(),
        CardKind::Wuzhong => vec![user],
        CardKind::Taoyuan => {
            if core.alive_players().any(|p| core.player(p).hp < core.player(p).max_hp as i32) {
                core.alive_players().collect()
            } else {
                Vec::new()
            }
        }
        CardKind::Wugu => core.alive_players().collect(),
        CardKind::SavageAttack | CardKind::ArcheryAttack => {
            core.alive_players().filter(|&t| t != user).collect()
        }
        CardKind::Chain => core
            .alive_players()
            .filter(|&t| {
                !is_target_protected(core, t)
                    && !crate::rvm_bridge::modifiers::eval_card_target_canceled(
                        core, catalog, user, t, 0, kind, nature,
                    )
            })
            .collect(),
        CardKind::Duel => core
            .alive_players()
            .filter(|&t| {
                t != user
                    && !is_target_kongcheng(core, catalog, t)
                    && !is_target_protected(core, t)
                    && !crate::rvm_bridge::modifiers::eval_card_target_canceled(
                        core, catalog, user, t, 0, kind, nature,
                    )
            })
            .collect(),
        CardKind::Dismantle => core
            .alive_players()
            .filter(|&t| {
                t != user
                    && !is_target_protected(core, t)
                    && (!core.player(t).hand.is_empty()
                        || !core.player(t).equips.is_empty()
                        || !core.player(t).judge.is_empty())
                    && !crate::rvm_bridge::modifiers::eval_card_target_canceled(
                        core, catalog, user, t, 0, kind, nature,
                    )
            })
            .collect(),
        CardKind::Snatch => core
            .alive_players()
            .filter(|&t| {
                t != user
                    && !is_target_protected(core, t)
                    && core.computed_distance(catalog, user, t) <= 1
                    && (!core.player(t).hand.is_empty()
                        || !core.player(t).equips.is_empty()
                        || !core.player(t).judge.is_empty())
                    && !crate::rvm_bridge::modifiers::eval_card_target_canceled(
                        core, catalog, user, t, 0, kind, nature,
                    )
            })
            .collect(),
        CardKind::Linghunzhihuo => core
            .alive_players()
            .filter(|&t| {
                !core.player(t).hand.is_empty()
                    && !is_target_protected(core, t)
                    && !crate::rvm_bridge::modifiers::eval_card_target_canceled(
                        core, catalog, user, t, 0, kind, nature,
                    )
            })
            .collect(),
        CardKind::Jiedao => core
            .alive_players()
            .filter(|&t| {
                t != user
                    && core
                        .player(t)
                        .equips
                        .iter()
                        .any(|&c| catalog.card_slot_of(core, c) == Some(EquipSlot::Weapon))
                    && !jiedao_victims_for(core, catalog, t).is_empty()
                    && !crate::rvm_bridge::modifiers::eval_card_target_canceled(
                        core, catalog, user, t, 0, kind, nature,
                    )
            })
            .collect(),
        CardKind::ShouLiJian => core
            .alive_players()
            .filter(|&t| t != user && !is_target_protected(core, t))
            .collect(),
        _ => Vec::new(),
    }
}

pub(in crate::content::host::step) fn virtual_kind_can_use_play(
    core: &GameStateCore,
    catalog: &Catalog,
    user: PlayerId,
    kind: CardKind,
) -> bool {
    if catalog.blocks_all_cards(core, user) {
        return false;
    }
    if kind == CardKind::Shan || kind == CardKind::Nullify {
        return false;
    }
    if !(kind.is_basic_kind() || (!kind.is_equip() && kind.is_basic_trick_or_equip() && !is_delayed(kind))) {
        return false;
    }
    if crate::rvm_bridge::modifiers::eval_card_use_canceled(
        core,
        catalog,
        user,
        0,
        kind,
        DamageNature::Normal,
    ) {
        return false;
    }
    !virtual_legal_targets(core, catalog, user, kind).is_empty()
}

pub(in crate::content::host::step) fn is_delayed(kind: CardKind) -> bool {
    matches!(
        kind,
        CardKind::Le | CardKind::Bing | CardKind::ShanDian | CardKind::HongShui | CardKind::Guiyoujie
    )
}

pub(in crate::content::host::step) fn virtual_kind_needs_target_prompt(kind: CardKind) -> bool {
    kind.is_sha_like()
        || matches!(
            kind,
            CardKind::Duel
                | CardKind::Dismantle
                | CardKind::Snatch
                | CardKind::Chain
                | CardKind::Jiedao
                | CardKind::Linghunzhihuo
                | CardKind::ShouLiJian
        )
}
