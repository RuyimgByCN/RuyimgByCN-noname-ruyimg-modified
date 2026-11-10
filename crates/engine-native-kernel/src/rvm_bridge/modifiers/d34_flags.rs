//! Authoritative kernel evaluation for Decision 34 (`U01`, `U05`, `L11`, `L19`, `L20`, `L25`, `L27`)
//! passive skills, equipment/servant modifiers, buff/debuff flags, and `VmViewAsSpec` conversions.

use std::sync::OnceLock;

use crate::catalog::{CardKind, Catalog, EquipSlot};
use crate::event::DamageNature;
use crate::ids::{CardId, PlayerId};
use crate::rvm_bridge::{VmViewAsFilter, VmViewAsSpec, VmViewAsWindow, VmViewAsZone};
use crate::state::GameStateCore;

#[inline]
fn flag_val(state: &GameStateCore, player: PlayerId, key: &str) -> u32 {
    if state.vm_aux.flags.is_empty() {
        return 0;
    }
    state
        .vm_aux
        .flags
        .get(&(player, key.to_string()))
        .copied()
        .unwrap_or(0)
}

#[inline]
fn has_ext_skill(state: &GameStateCore, catalog: &Catalog, player: PlayerId, name: &str) -> bool {
    if catalog.skills.len() <= 24 {
        return false;
    }
    catalog
        .skill_by_name(name)
        .is_some_and(|def| state.has_skill(player, def))
}

fn has_flag_prefix(state: &GameStateCore, player: PlayerId, prefix: &str) -> bool {
    if state.vm_aux.flags.is_empty() {
        return false;
    }
    state
        .vm_aux
        .flags
        .range((player, prefix.to_string())..)
        .take_while(|((p, k), _)| *p == player && k.starts_with(prefix))
        .any(|(_, &v)| v > 0)
}

pub fn apply_hand_limit(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    mut limit: i32,
) -> i32 {
    if flag_val(state, player, "l11_xuwang_hand_minus1") > 0
        || has_ext_skill(state, catalog, player, "txhj_xuwangzhimian")
    {
        limit -= 1;
    }
    if flag_val(state, player, "l25_yuxi_treasure_active") > 0 {
        limit += 1;
    }
    let crazy = flag_val(state, player, "l25_chanyuan_crazy");
    if crazy > 0 {
        limit -= crazy as i32;
    }
    if flag_val(state, player, "u05_mod:gw_qinpendayu") > 0 {
        limit -= 1;
    }
    if flag_val(state, player, "u05_mod:chunbing") > 0 {
        limit += 1;
    }
    if has_flag_prefix(state, player, "u05_buff:") {
        limit += 1;
    }
    if has_flag_prefix(state, player, "u05_debuff:") {
        limit -= 1;
    }
    limit
}

pub fn apply_attack_range(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    mut cur: i32,
) -> i32 {
    let qimou = flag_val(state, player, "l27_qimou_range_bonus");
    if qimou > 0 {
        cur += qimou as i32;
    }
    if has_ext_skill(state, catalog, player, "ssr_huchi") {
        cur += 1;
    }
    if has_flag_prefix(state, player, "u05_mod:") {
        cur += 1;
    }
    cur
}

pub fn apply_distance_total(
    state: &GameStateCore,
    catalog: &Catalog,
    a: PlayerId,
    b: PlayerId,
    mut cur: i32,
) -> i32 {
    if a == b {
        return cur;
    }
    // `ssr_biyue` (`祖安武将/extension.js:11950`): +1 distance to `b` for each empty zone (`h`, `e`, `j`)
    if has_ext_skill(state, catalog, b, "ssr_biyue") {
        let pb = state.player(b);
        if pb.hand.is_empty() {
            cur += 1;
        }
        if pb.equips.is_empty() {
            cur += 1;
        }
        if pb.judge.is_empty() {
            cur += 1;
        }
    }
    // `ssr_newyicong` (`祖安武将/extension.js:13604`): -99 against targets with mounts, else -1
    if has_ext_skill(state, catalog, a, "ssr_newyicong") {
        let target_has_mount = state.player(b).equips.iter().any(|&c| {
            catalog.cards.get(c as usize).is_some_and(|d| {
                matches!(
                    d.kind.equip_slot(),
                    Some(EquipSlot::HorsePlus | EquipSlot::HorseMinus)
                )
            })
        });
        if target_has_mount {
            cur -= 99;
        } else {
            cur -= 1;
        }
    }
    if flag_val(state, a, "l11_mount_active:txhj_liulongcanjia") > 0
        || flag_val(state, a, "l11_mount_active:txhj_baihu") > 0
        || flag_val(state, a, "u05_cross_mod:mashu") > 0
        || flag_val(state, a, "u05_cross_mod:yicong") > 0
    {
        cur -= 1;
    }
    if flag_val(state, b, "l11_mount_active:txhj_liulongcanjia") > 0
        || flag_val(state, b, "l11_mount_active:txhj_juechenjinge") > 0
        || flag_val(state, b, "u05_cross_mod:feiying") > 0
        || flag_val(state, b, "u05_mod:heilonglinpian") > 0
    {
        cur += 1;
    }
    cur
}

pub fn apply_sha_use_limit(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    mut cur: i32,
    mut unlimited: bool,
) -> (i32, bool) {
    let zhuge_plus = flag_val(state, player, "l11_txhj_zhuge_sha_plus3");
    if zhuge_plus > 0 {
        cur += zhuge_plus as i32;
    } else if has_ext_skill(state, catalog, player, "txhj_zhuge") {
        cur += 3;
    }
    if has_ext_skill(state, catalog, player, "ssr_paoxiao")
        || flag_val(state, player, "l25_paoxiao_unlimited") > 0
        || flag_val(state, player, "l19_baozou_sha_unlimited") > 0
    {
        unlimited = true;
    }
    if has_ext_skill(state, catalog, player, "ssr_jijiang") {
        cur += 1;
    }
    if has_ext_skill(state, catalog, player, "ssr_weilong") {
        cur += 1;
    }
    if has_ext_skill(state, catalog, player, "ssr_huchi") {
        cur += 1;
    }
    for k in [
        "u05_mod:fengnu",
        "u05_mod:feiren",
        "u05_mod:feidan",
        "u05_mod:duijue2",
        "u05_buff:chilongya2",
        "u05_buff:chongzhuang",
        "u05_buff:aqianghua2",
    ] {
        if flag_val(state, player, k) > 0 {
            cur += 1;
        }
    }
    (cur, unlimited)
}

pub fn apply_sha_extra_targets(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    kind: CardKind,
    mut extra: i32,
) -> i32 {
    if kind.is_sha_like() {
        let fangtian = flag_val(state, player, "l11_fangtian_extra_targets");
        if fangtian > 0 {
            extra += fangtian as i32;
        } else if has_ext_skill(state, catalog, player, "txhj_fangtianhuaji") {
            extra += 2;
        }
    }
    extra
}

pub fn apply_card_distance_unlimited(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    kind: CardKind,
    mut unlimited: bool,
) -> bool {
    if !kind.is_basic_kind()
        && !kind.is_equip()
        && has_ext_skill(state, catalog, player, "ssr_qicai")
    {
        unlimited = true;
    }
    if flag_val(state, player, "l25_jiwu_boost") > 0 {
        unlimited = true;
    }
    unlimited
}

pub fn apply_card_target_canceled(
    state: &GameStateCore,
    catalog: &Catalog,
    actor: PlayerId,
    target: PlayerId,
    kind: CardKind,
    mut canceled: bool,
) -> bool {
    if actor != target
        && (has_ext_skill(state, catalog, target, "ssr_newjiahe")
            || flag_val(state, target, "l25_jiahe_qianxing") > 0
            || flag_val(state, target, "th_tiansuan2_immune") > 0)
    {
        canceled = true;
    }
    if matches!(kind, CardKind::Snatch | CardKind::Le)
        && flag_val(state, target, "u05_cross_mod:qianxun") > 0
    {
        canceled = true;
    }
    canceled
}

pub fn apply_card_use_canceled(
    state: &GameStateCore,
    actor: PlayerId,
    kind: CardKind,
    mut canceled: bool,
) -> bool {
    if flag_val(state, actor, "l11_xingtian_card_locked") > 0
        || flag_val(state, actor, "l25_yijue_locked") > 0
    {
        canceled = true;
    }
    if kind.is_sha_like()
        && (flag_val(state, actor, "l25_zhenhun_forbid_sha") > 0
            || flag_val(state, actor, "u05_mod:gw_birinongwu") > 0)
    {
        canceled = true;
    }
    canceled
}

pub fn apply_damage_amount(
    state: &GameStateCore,
    catalog: &Catalog,
    target: PlayerId,
    card: Option<CardId>,
    nature: DamageNature,
    mut cur: i32,
) -> i32 {
    if cur <= 0 {
        return 0;
    }
    // `ssr_manjia` (`祖安武将/extension.js:22026`): Vine Armor — immune to Nanman/Wanjian/normal Sha, +1 Fire damage
    if has_ext_skill(state, catalog, target, "ssr_manjia") {
        let ckind = card.map(|c| catalog.card_kind(c));
        if matches!(ckind, Some(CardKind::SavageAttack | CardKind::ArcheryAttack))
            || (matches!(ckind, Some(CardKind::Sha)) && nature == DamageNature::Normal)
        {
            return 0;
        }
        if nature == DamageNature::Fire {
            cur += 1;
        }
    }
    if flag_val(state, target, "l20_qingyu_prevented") > 0
        || flag_val(state, target, "l11_yaya2_protected") > 0
    {
        return 0;
    }
    if flag_val(state, target, "u05_cross_mod:xinwuyan") > 0 {
        if let Some(c) = card {
            let k = catalog.card_kind(c);
            if !k.is_basic_kind() && !k.is_equip() {
                return 0;
            }
        }
    }
    if flag_val(state, target, "l25_tianxiang_deflected") > 0 {
        cur = (cur - 1).max(0);
    }
    if cur > 1
        && (flag_val(state, target, "l11_dundun_zhongzhi_reduce") > 0
            || has_ext_skill(state, catalog, target, "txhj_dundunSkill2")
            || flag_val(state, target, "u05_buff:bingjia") > 0
            || flag_val(state, target, "u05_buff:bingjia2") > 0
            || flag_val(state, target, "u05_buff:cyxianjiang") > 0
            || flag_val(state, target, "u05_buff:dangping") > 0)
    {
        cur -= 1;
    }
    cur
}

pub fn apply_ignore_armor(
    state: &GameStateCore,
    catalog: &Catalog,
    source: PlayerId,
    target: PlayerId,
    mut ignore: bool,
) -> bool {
    if flag_val(state, target, "l11_xingtian_armor_null") > 0
        || flag_val(state, target, "l25_yijue_locked") > 0
        || flag_val(state, target, "l25_pojia_broken") > 0
        || flag_val(state, target, "l25_hubu_unequip2") > 0
        || has_ext_skill(state, catalog, source, "ssr_pojia")
        || has_ext_skill(state, catalog, source, "ssr_hubu")
    {
        ignore = true;
    }
    ignore
}

pub fn apply_card_unrespondable(
    state: &GameStateCore,
    catalog: &Catalog,
    source: PlayerId,
    kind: CardKind,
    mut unrespondable: bool,
) -> bool {
    if kind.is_sha_like()
        && (flag_val(state, source, "l11_taiji_unrespondable") > 0
            || has_ext_skill(state, catalog, source, "ssr_wushuang")
            || flag_val(state, source, "l25_wushuang_unrespondable") > 0
            || flag_val(state, source, "u05_cross_mod:wushuang") > 0)
    {
        unrespondable = true;
    }
    if kind == CardKind::Duel
        && (has_ext_skill(state, catalog, source, "ssr_wushuang")
            || flag_val(state, source, "u05_cross_mod:wushuang") > 0)
    {
        unrespondable = true;
    }
    unrespondable
}

enum D34ViewAsActivation {
    Skill(&'static str),
    Flag(&'static str),
}

struct D34ViewAsRule {
    activation: D34ViewAsActivation,
    spec: VmViewAsSpec,
}

fn make_spec(
    id: u32,
    name: &str,
    as_kind: CardKind,
    window: VmViewAsWindow,
    zone: VmViewAsZone,
    filter: VmViewAsFilter,
    ignore_distance: bool,
) -> VmViewAsSpec {
    VmViewAsSpec {
        skill_def_id: id,
        skill_name: name.to_string(),
        as_kind,
        window,
        zone,
        filter,
        uses_per_turn: None,
        ignore_distance,
        red_unrespondable: false,
        black_no_use_count: false,
        no_use_count: false,
        target_lacks_source_suit_unrespondable: false,
        refund_use_count_if_no_damage: false,
        min_cards: 1,
        max_cards: 1,
        owner_hp_min_one_cards: false,
        owner_basic_uses_plus_one_cards: false,
        once_per_turn_per_branch: false,
        same_suit: false,
        multi_card_bonus: None,
        branch: None,
        once_per_round: false,
        next_branch: None,
        cost: None,
        mark_cost: None,
        post_draw: 0,
    }
}

fn d34_view_as_table() -> &'static Vec<D34ViewAsRule> {
    static TABLE: OnceLock<Vec<D34ViewAsRule>> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut v = vec![
            D34ViewAsRule {
                activation: D34ViewAsActivation::Skill("ssr_newlongdan"),
                spec: make_spec(
                    1_926_900_001,
                    "ssr_newlongdan",
                    CardKind::Sha,
                    VmViewAsWindow::Any,
                    VmViewAsZone::Hand,
                    VmViewAsFilter::Shan,
                    false,
                ),
            },
            D34ViewAsRule {
                activation: D34ViewAsActivation::Skill("ssr_newlongdan"),
                spec: make_spec(
                    1_926_900_002,
                    "ssr_newlongdan",
                    CardKind::Shan,
                    VmViewAsWindow::Any,
                    VmViewAsZone::Hand,
                    VmViewAsFilter::Sha,
                    false,
                ),
            },
            D34ViewAsRule {
                activation: D34ViewAsActivation::Skill("ssr_wusheng"),
                spec: make_spec(
                    1_926_900_003,
                    "ssr_wusheng",
                    CardKind::Sha,
                    VmViewAsWindow::Any,
                    VmViewAsZone::HandEquip,
                    VmViewAsFilter::Red,
                    true,
                ),
            },
            D34ViewAsRule {
                activation: D34ViewAsActivation::Skill("ssr_newguose"),
                spec: make_spec(
                    1_926_900_004,
                    "ssr_newguose",
                    CardKind::Le,
                    VmViewAsWindow::PlayPhase,
                    VmViewAsZone::HandEquip,
                    VmViewAsFilter::Diamond,
                    false,
                ),
            },
            D34ViewAsRule {
                activation: D34ViewAsActivation::Skill("ssr_newjijie"),
                spec: make_spec(
                    1_926_900_005,
                    "ssr_newjijie",
                    CardKind::Shan,
                    VmViewAsWindow::Response,
                    VmViewAsZone::HandExpansion,
                    VmViewAsFilter::Any,
                    false,
                ),
            },
            D34ViewAsRule {
                activation: D34ViewAsActivation::Skill("ssr_newjiang"),
                spec: make_spec(
                    1_926_900_006,
                    "ssr_newjiang",
                    CardKind::Sha,
                    VmViewAsWindow::Response,
                    VmViewAsZone::Hand,
                    VmViewAsFilter::Basic,
                    false,
                ),
            },
            D34ViewAsRule {
                activation: D34ViewAsActivation::Flag("dddxiaxing_viewas_taojiu"),
                spec: make_spec(
                    1_926_900_007,
                    "dddxiaxing",
                    CardKind::Tao,
                    VmViewAsWindow::Any,
                    VmViewAsZone::Hand,
                    VmViewAsFilter::Any,
                    false,
                ),
            },
            D34ViewAsRule {
                activation: D34ViewAsActivation::Flag("dddxiaxing_viewas_taojiu"),
                spec: make_spec(
                    1_926_900_008,
                    "dddxiaxing",
                    CardKind::Jiu,
                    VmViewAsWindow::Any,
                    VmViewAsZone::Hand,
                    VmViewAsFilter::Any,
                    false,
                ),
            },
            D34ViewAsRule {
                activation: D34ViewAsActivation::Flag("dddxiaxing_viewas_basic"),
                spec: make_spec(
                    1_926_900_009,
                    "dddxiaxing",
                    CardKind::Sha,
                    VmViewAsWindow::Any,
                    VmViewAsZone::Hand,
                    VmViewAsFilter::Any,
                    false,
                ),
            },
            D34ViewAsRule {
                activation: D34ViewAsActivation::Flag("dddxiaxing_viewas_basic"),
                spec: make_spec(
                    1_926_900_010,
                    "dddxiaxing",
                    CardKind::Shan,
                    VmViewAsWindow::Any,
                    VmViewAsZone::Hand,
                    VmViewAsFilter::Any,
                    false,
                ),
            },
            D34ViewAsRule {
                activation: D34ViewAsActivation::Flag("l25_viewas_wuxie"),
                spec: make_spec(
                    1_926_900_011,
                    "wuxie",
                    CardKind::Nullify,
                    VmViewAsWindow::Any,
                    VmViewAsZone::Hand,
                    VmViewAsFilter::Any,
                    false,
                ),
            },
            D34ViewAsRule {
                activation: D34ViewAsActivation::Flag("l25_viewas_jiu"),
                spec: make_spec(
                    1_926_900_012,
                    "ssr_zhujiu",
                    CardKind::Jiu,
                    VmViewAsWindow::Any,
                    VmViewAsZone::Hand,
                    VmViewAsFilter::Any,
                    false,
                ),
            },
            D34ViewAsRule {
                activation: D34ViewAsActivation::Flag("l25_basic_as_sha"),
                spec: make_spec(
                    1_926_900_013,
                    "res",
                    CardKind::Sha,
                    VmViewAsWindow::Any,
                    VmViewAsZone::Hand,
                    VmViewAsFilter::Basic,
                    false,
                ),
            },
            D34ViewAsRule {
                activation: D34ViewAsActivation::Flag("u05_viewas:longdan"),
                spec: make_spec(
                    1_926_900_014,
                    "u05_longdan",
                    CardKind::Sha,
                    VmViewAsWindow::Any,
                    VmViewAsZone::Hand,
                    VmViewAsFilter::Shan,
                    false,
                ),
            },
            D34ViewAsRule {
                activation: D34ViewAsActivation::Flag("u05_viewas:longdan"),
                spec: make_spec(
                    1_926_900_015,
                    "u05_longdan",
                    CardKind::Shan,
                    VmViewAsWindow::Any,
                    VmViewAsZone::Hand,
                    VmViewAsFilter::Sha,
                    false,
                ),
            },
        ];

        let u05_flag_map: &[(&str, CardKind)] = &[
            ("u05_viewas:sha", CardKind::Sha),
            ("u05_viewas:stonesha", CardKind::Sha),
            ("u05_viewas:shan", CardKind::Shan),
            ("u05_viewas:stoneshan", CardKind::Shan),
            ("u05_viewas:tao", CardKind::Tao),
            ("u05_viewas:caoyao", CardKind::Tao),
            ("u05_viewas:jiu", CardKind::Jiu),
            ("u05_viewas:jiuchi", CardKind::Jiu),
            ("u05_viewas:wuxie", CardKind::Nullify),
            ("u05_viewas:wuzhong", CardKind::Wuzhong),
            ("u05_viewas:juedou", CardKind::Duel),
            ("u05_viewas:nanman", CardKind::SavageAttack),
            ("u05_viewas:luanji", CardKind::ArcheryAttack),
            ("u05_viewas:shunshou", CardKind::Snatch),
            ("u05_viewas:taoyuan", CardKind::Taoyuan),
            ("u05_viewas:shuigong", CardKind::Shuigong),
            ("u05_viewas:toulianghuanzhu", CardKind::Toulianghuanzhu),
            ("u05_viewas:bingpotong", CardKind::Bingpotong),
            ("u05_viewas:qiankunbiao", CardKind::Qiankunbiao),
            ("u05_viewas:feibiao", CardKind::Feibiao),
            ("u05_viewas:shihuifen", CardKind::Shihuifen),
            ("u05_viewas:liuxinghuoyu", CardKind::FireSha),
            ("u05_viewas:jingleishan", CardKind::LeiSha),
            ("u05_viewas:chiyuxi", CardKind::FireSha),
        ];
        for (idx, &(flag_name, kind)) in u05_flag_map.iter().enumerate() {
            v.push(D34ViewAsRule {
                activation: D34ViewAsActivation::Flag(flag_name),
                spec: make_spec(
                    1_926_900_100 + idx as u32,
                    flag_name,
                    kind,
                    VmViewAsWindow::Any,
                    VmViewAsZone::HandEquip,
                    VmViewAsFilter::Any,
                    false,
                ),
            });
        }
        v
    })
}

pub fn find_d34_flag_view_as(
    core: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    card: CardId,
    in_play: bool,
    target_kind: Option<CardKind>,
) -> Option<&'static VmViewAsSpec> {
    if core.vm_aux.flags.is_empty() && catalog.skills.len() <= 24 {
        return None;
    }
    for entry in d34_view_as_table() {
        let active = match entry.activation {
            D34ViewAsActivation::Skill(sk) => has_ext_skill(core, catalog, player, sk),
            D34ViewAsActivation::Flag(fl) => flag_val(core, player, fl) > 0,
        };
        if active && entry.spec.matches_card(core, catalog, player, card, in_play, target_kind) {
            return Some(&entry.spec);
        }
    }
    None
}
