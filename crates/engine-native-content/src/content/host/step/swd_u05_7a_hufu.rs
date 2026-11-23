//! U05-7a 玉符转化 legality & execution (`g_hufu_sha` / `g_hufu_jiu`)——自 `swd_u05_7a` 拆出。

use super::*;
use crate::catalog::Catalog;
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow};
use crate::snap::{ask as build_ask, ExpectedAsk};
use crate::state::GameStateCore;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HufuShaSlot {
    Unlimited,
    Normal,
    LandBonus,
    Yuanbaorou,
    Huihuo,
    Lingshi,
    DcdichouEff,
    VmExtra,
}

pub(super) fn hufu_hand_cards(
    catalog: &Catalog,
    core: &GameStateCore,
    owner: PlayerId,
) -> Vec<CardId> {
    core.player(owner)
        .hand
        .iter()
        .copied()
        .filter(|c| catalog.card_kind(*c) == CardKind::Hufu)
        .collect()
}

fn is_red_physical(catalog: &Catalog, card: CardId) -> bool {
    matches!(
        catalog.cards[card as usize].suit,
        crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
    )
}

fn has_enabled_skill(catalog: &Catalog, core: &GameStateCore, p: PlayerId, name: &str) -> bool {
    catalog
        .skill_by_name(name)
        .is_some_and(|d| core.has_skill(p, d))
}

fn has_unused_turn_skill(catalog: &Catalog, core: &GameStateCore, p: PlayerId, name: &str) -> bool {
    catalog
        .skill_by_name(name)
        .and_then(|d| core.skills.get(&(p, d)))
        .is_some_and(|i| i.enabled && !i.once_used)
}

fn hufu_sha_limit_slot(
    catalog: &Catalog,
    core: &GameStateCore,
    owner: PlayerId,
    card: CardId,
) -> Option<HufuShaSlot> {
    use crate::event::DamageNature;
    use crate::vocabulary::names::{
        BOSS_FENTIAN, BOSS_HUIHUO, BOSS_JIDING, BOSS_QINGZHU, PAO_XIAO,
    };
    if !core.player(owner).alive
        || catalog.blocks_all_cards(core, owner)
        || core.player(owner).wangshi_ban
        || core.player(owner).jileng_active
        || has_enabled_skill(catalog, core, owner, crate::content::host::GW_BIRIN_STATE_KEY)
        || has_enabled_skill(catalog, core, owner, crate::content::host::WUDIAN_SHOUHU_KEY)
        || (core.turn.current == owner
            && has_enabled_skill(catalog, core, owner, BOSS_QINGZHU)
            && !has_enabled_skill(catalog, core, owner, BOSS_JIDING))
        || crate::rvm_bridge::modifiers::eval_card_use_canceled(
            core,
            catalog,
            owner,
            card,
            CardKind::Sha,
            DamageNature::Normal,
        )
    {
        return None;
    }
    let is_red = is_red_physical(catalog, card);
    let (vm_sha_limit, vm_sha_unlimited) = crate::rvm_bridge::modifiers::eval_sha_use_limit(
        core,
        catalog,
        owner,
        Some(card),
        CardKind::Sha,
        DamageNature::Normal,
        1,
    );
    let paoxiao = !catalog.is_rechanyuan_sealed_name(core, owner, PAO_XIAO)
        && has_enabled_skill(catalog, core, owner, PAO_XIAO);
    let has_zhuge = !crate::kernel::mbcmfangzhu_baiban(core, owner)
        && core
            .player(owner)
            .equips
            .iter()
            .any(|c| catalog.card_kind(*c) == CardKind::Zhuge);
    let baolei = catalog
        .skill_by_name(crate::content::host::WUDIAN_BAOLEI_KEY)
        .is_some_and(|d| core.has_skill(owner, d) && core.player(owner).baolei_mode == 1);
    let sbzhaxiang = has_enabled_skill(catalog, core, owner, crate::content::host::L27_SBZHAXIANG_KEY)
        && crate::content::host::sbzhaxiang_active(core, owner);
    let fentian_red = is_red && has_enabled_skill(catalog, core, owner, BOSS_FENTIAN);
    let poyue_red_free = is_red && has_enabled_skill(catalog, core, owner, "poyue");
    if paoxiao
        || has_zhuge
        || baolei
        || fentian_red
        || poyue_red_free
        || sbzhaxiang
        || vm_sha_unlimited
    {
        return Some(HufuShaSlot::Unlimited);
    }
    let land_bonus = core.current_land == Some(CardKind::Longlushanfeng) && !core.turn.land_sha_used;
    if !core.turn.sha_used_play_phase && vm_sha_limit > 0 {
        return Some(HufuShaSlot::Normal);
    }
    if land_bonus {
        return Some(HufuShaSlot::LandBonus);
    }
    if vm_sha_limit == 0 {
        return None;
    }
    if has_unused_turn_skill(catalog, core, owner, crate::content::host::GJ_FOOD_YUANBAOROU_KEY) {
        return Some(HufuShaSlot::Yuanbaorou);
    }
    if has_unused_turn_skill(catalog, core, owner, BOSS_HUIHUO) {
        return Some(HufuShaSlot::Huihuo);
    }
    let has_treasure = core
        .player(owner)
        .equips
        .iter()
        .any(|c| catalog.card_slot_of(core, *c) == Some(crate::catalog::EquipSlot::Treasure));
    if has_unused_turn_skill(catalog, core, owner, "lingshi") && has_treasure {
        return Some(HufuShaSlot::Lingshi);
    }
    let dcdichou_eff_extra = {
        let has_eff = has_enabled_skill(catalog, core, owner, "dcdichou_eff")
            || crate::rvm_ops::resolve_skill_def(catalog, 2_754_298_115)
                .is_some_and(|d| core.has_skill(owner, d))
            || core
                .vm_aux
                .flags
                .get(&(owner, "skill:2754298115".to_string()))
                .copied()
                .unwrap_or(0)
                != 0;
        let bonus = core
            .vm_aux
            .marks
            .get(&(owner, 2_754_298_115))
            .copied()
            .unwrap_or(0);
        let used = core
            .vm_aux
            .flags
            .get(&(owner, "dcdichou_eff_sha_used".to_string()))
            .copied()
            .unwrap_or(0) as i32;
        has_eff && bonus > used
    };
    if dcdichou_eff_extra {
        return Some(HufuShaSlot::DcdichouEff);
    }
    let vm_sha_extra_used = core
        .vm_aux
        .flags
        .get(&(owner, "vm_sha_extra_used".to_string()))
        .copied()
        .unwrap_or(0);
    if vm_sha_limit > 1 + vm_sha_extra_used {
        return Some(HufuShaSlot::VmExtra);
    }
    None
}

fn is_hufu_sha_target_legal(
    catalog: &Catalog,
    core: &GameStateCore,
    owner: PlayerId,
    card: CardId,
    t: PlayerId,
) -> bool {
    use crate::event::DamageNature;
    use crate::vocabulary::names::{
        BOSS_FENTIAN, BOSS_KUANGXIAO, BOSS_REMEN, BOSS_SHENEN, BOSS_SHOUYI, BOSS_ZAOYI,
    };
    if crate::kernel::sha_user_or_target_blocked(core, catalog, owner, t) {
        return false;
    }
    if !core.turn.zhuyue_targets.is_empty() && !core.turn.zhuyue_targets.contains(&t) {
        return false;
    }
    let is_red = is_red_physical(catalog, card);
    if !is_red
        && catalog
            .skill_by_name(crate::content::host::GJ_FOOD_MOLICHA_KEY)
            .is_some_and(|d| core.skills.contains_key(&(t, d)))
    {
        return false;
    }
    if t != owner {
        const YINGBA_MARK_RAW: u32 = 3_219_705_664;
        const YINGBA_RAW: u32 = 2_187_983_153;
        let used = core.turn.cards_used_turn as i32;
        let has_mark = |p: PlayerId| {
            has_enabled_skill(catalog, core, p, "yingba_mark")
                || crate::rvm_ops::resolve_skill_def(catalog, YINGBA_MARK_RAW)
                    .is_some_and(|d| core.has_skill(p, d))
                || core
                    .vm_aux
                    .flags
                    .get(&(p, format!("skill:{YINGBA_MARK_RAW}")))
                    .copied()
                    .unwrap_or(0)
                    != 0
        };
        let has_yb = |p: PlayerId| {
            has_enabled_skill(catalog, core, p, "yingba")
                || crate::rvm_ops::resolve_skill_def(catalog, YINGBA_RAW)
                    .is_some_and(|d| core.has_skill(p, d))
                || core
                    .vm_aux
                    .flags
                    .get(&(p, format!("skill:{YINGBA_RAW}")))
                    .copied()
                    .unwrap_or(0)
                    != 0
        };
        let tm = core.vm_aux.marks.get(&(t, YINGBA_MARK_RAW)).copied().unwrap_or(0);
        let um = core
            .vm_aux
            .marks
            .get(&(owner, YINGBA_MARK_RAW))
            .copied()
            .unwrap_or(0);
        if (has_mark(t) && tm > used) || (has_mark(owner) && has_yb(t) && um > used) {
            return false;
        }
    }
    if core.turn.current == owner
        && has_enabled_skill(catalog, core, owner, BOSS_ZAOYI)
        && core.is_boss_side(t) != core.is_boss_side(owner)
    {
        let base = catalog
            .general_count()
            .saturating_sub(crate::content::standard::BOSS_GENERALS.len()) as u16;
        let gonggong_alive = (base..catalog.general_count() as u16)
            .filter(|&d| catalog.general_meta(d).is_some_and(|m| m.0 == "共工"))
            .any(|gd| {
                (0..core.players.len() as u8)
                    .any(|p| core.player(p).alive && core.player(p).general_slots[0] == Some(gd))
            });
        if gonggong_alive {
            return false;
        }
    }
    if has_enabled_skill(catalog, core, t, BOSS_REMEN)
        && !core
            .player(t)
            .equips
            .iter()
            .any(|c| catalog.card_slot_of(core, *c) == Some(crate::catalog::EquipSlot::Armor))
    {
        return false;
    }
    if crate::rvm_bridge::modifiers::eval_card_target_canceled(
        core,
        catalog,
        owner,
        t,
        card,
        CardKind::Sha,
        DamageNature::Normal,
    ) {
        return false;
    }
    let baolei_free = (has_enabled_skill(catalog, core, owner, "bshaowei")
        || has_enabled_skill(catalog, core, owner, crate::content::host::WUDIAN_BAOLEI_KEY))
        && core.player(owner).baolei_mode == 1;
    let shenen_free = catalog
        .skill_by_name(BOSS_SHENEN)
        .is_some_and(|def| core.skills.keys().any(|(_, dd)| *dd == def))
        && core.is_boss_side(owner);
    let range_free = baolei_free
        || shenen_free
        || has_enabled_skill(catalog, core, owner, crate::content::host::WUDIAN_FEIREN_KEY)
        || has_enabled_skill(catalog, core, owner, crate::content::host::WUDIAN_FEIDAN_KEY)
        || has_enabled_skill(catalog, core, owner, crate::content::host::WUDIAN_XJ_TIANXIAN_KEY)
        || has_enabled_skill(catalog, core, owner, BOSS_KUANGXIAO)
        || has_enabled_skill(catalog, core, owner, "jiying")
        || has_enabled_skill(catalog, core, owner, BOSS_SHOUYI)
        || (is_red && has_enabled_skill(catalog, core, owner, BOSS_FENTIAN))
        || (!is_red && has_enabled_skill(catalog, core, owner, "poyue"))
        || (has_enabled_skill(catalog, core, owner, crate::content::host::L27_SBZHAXIANG_KEY)
            && crate::content::host::sbzhaxiang_active(core, owner))
        || crate::rvm_bridge::modifiers::eval_card_distance_unlimited(
            core,
            catalog,
            owner,
            card,
            CardKind::Sha,
            DamageNature::Normal,
        );
    let zfs_free = core.turn.zhufangshenshi_target == Some(t);
    let zy_free = core.turn.zhuyue_targets.contains(&t);
    let sliufeng_free = has_enabled_skill(catalog, core, owner, "sliufeng")
        && core.player(t).hp <= core.player(owner).hp;
    range_free
        || zfs_free
        || zy_free
        || sliufeng_free
        || core.computed_distance(catalog, owner, t) <= core.attack_range(catalog, owner)
}

pub(super) fn sha_targets_for_hufu(
    catalog: &Catalog,
    core: &GameStateCore,
    owner: PlayerId,
    card: CardId,
) -> Vec<PlayerId> {
    core.alive_players()
        .filter(|&t| is_hufu_sha_target_legal(catalog, core, owner, card, t))
        .collect()
}

pub(super) fn hufu_sha_eligible_cards(
    catalog: &Catalog,
    core: &GameStateCore,
    owner: PlayerId,
) -> Vec<CardId> {
    hufu_hand_cards(catalog, core, owner)
        .into_iter()
        .filter(|&c| {
            hufu_sha_limit_slot(catalog, core, owner, c).is_some()
                && !sha_targets_for_hufu(catalog, core, owner, c).is_empty()
        })
        .collect()
}

pub(super) fn hufu_jiu_eligible_cards(
    catalog: &Catalog,
    core: &GameStateCore,
    owner: PlayerId,
) -> Vec<CardId> {
    if !core.player(owner).alive
        || core.turn.jiu_used
        || catalog.blocks_all_cards(core, owner)
        || core.player(owner).wangshi_ban
        || core.player(owner).jileng_active
    {
        return Vec::new();
    }
    hufu_hand_cards(catalog, core, owner)
        .into_iter()
        .filter(|&c| {
            !crate::rvm_bridge::modifiers::eval_card_use_canceled(
                core,
                catalog,
                owner,
                c,
                CardKind::Jiu,
                crate::event::DamageNature::Normal,
            )
        })
        .collect()
}

pub(super) fn step_hufu_view_as(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::SelectCard => {
            let cards = if m.def_key == "g_hufu_jiu" {
                hufu_jiu_eligible_cards(ctx.catalog, ctx.core, m.owner)
            } else {
                hufu_sha_eligible_cards(ctx.catalog, ctx.core, m.owner)
            };
            let opts: Vec<ChoiceOption> = cards.into_iter().map(ChoiceOption::Card).collect();
            if opts.is_empty() {
                return Flow::Done;
            }
            if m.def_key == "g_hufu_jiu" {
                m.stage = HostStage::Act;
                Flow::WaitChoice {
                    observer: m.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "u05_hufu_jiu_card",
                        options: opts,
                        min: 1,
                        max: 1,
                        allow_cancel: true,
                    },
                }
            } else {
                m.stage = HostStage::TargetChoice;
                Flow::WaitChoice {
                    observer: m.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "u05_hufu_sha_card",
                        options: opts,
                        min: 1,
                        max: 1,
                        allow_cancel: true,
                    },
                }
            }
        }
        HostStage::TargetChoice => {
            let Some(card) = ctx.answer().iter().find_map(|o| match o {
                ChoiceOption::Card(c) => Some(*c),
                _ => None,
            }) else {
                return Flow::Done;
            };
            if !ctx.core.player(m.owner).hand.contains(&card)
                || hufu_sha_limit_slot(ctx.catalog, ctx.core, m.owner, card).is_none()
            {
                return Flow::Done;
            }
            m.cards = vec![card];
            let targets = sha_targets_for_hufu(ctx.catalog, ctx.core, m.owner, card);
            if targets.is_empty() {
                return Flow::Done;
            }
            m.stage = HostStage::Act;
            Flow::WaitChoice {
                observer: m.owner,
                spec: ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "u05_hufu_sha_target",
                    options: targets.into_iter().map(ChoiceOption::Player).collect(),
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            }
        }
        HostStage::Act => {
            if m.def_key == "g_hufu_jiu" {
                let Some(card) = ctx.answer().iter().find_map(|o| match o {
                    ChoiceOption::Card(c) => Some(*c),
                    _ => None,
                }) else {
                    return Flow::Done;
                };
                if ctx.core.player(m.owner).hand.contains(&card)
                    && hufu_jiu_eligible_cards(ctx.catalog, ctx.core, m.owner).contains(&card)
                {
                    ctx.emit(EventData::CardUsed(crate::machine::card_use_as(
                        card,
                        m.owner,
                        Vec::new(),
                        Some(CardKind::Jiu),
                    )));
                }
                return Flow::Done;
            }
            let Some(card) = m.cards.first().copied() else {
                return Flow::Done;
            };
            let Some(t) = ctx.answer().iter().find_map(|o| match o {
                ChoiceOption::Player(p) => Some(*p),
                _ => None,
            }) else {
                return Flow::Done;
            };
            if ctx.core.player(m.owner).hand.contains(&card)
                && is_hufu_sha_target_legal(ctx.catalog, ctx.core, m.owner, card, t)
            {
                let Some(slot) = hufu_sha_limit_slot(ctx.catalog, ctx.core, m.owner, card) else {
                    return Flow::Done;
                };
                match slot {
                    HufuShaSlot::Yuanbaorou => {
                        if let Some(def) = ctx
                            .catalog
                            .skill_by_name(crate::content::host::GJ_FOOD_YUANBAOROU_KEY)
                        {
                            if let Some(inst) = ctx.core.skills.get_mut(&(m.owner, def)) {
                                inst.once_used = true;
                            }
                        }
                    }
                    HufuShaSlot::Huihuo => {
                        if let Some(def) =
                            ctx.catalog.skill_by_name(crate::vocabulary::names::BOSS_HUIHUO)
                        {
                            if let Some(inst) = ctx.core.skills.get_mut(&(m.owner, def)) {
                                inst.once_used = true;
                            }
                        }
                    }
                    HufuShaSlot::Lingshi => {
                        if let Some(def) = ctx.catalog.skill_by_name("lingshi") {
                            if let Some(inst) = ctx.core.skills.get_mut(&(m.owner, def)) {
                                inst.once_used = true;
                            }
                        }
                    }
                    HufuShaSlot::DcdichouEff => {
                        let key = (m.owner, "dcdichou_eff_sha_used".to_string());
                        let next = ctx.core.vm_aux.flags.get(&key).copied().unwrap_or(0) + 1;
                        ctx.core.vm_aux.flags.insert(key, next);
                    }
                    HufuShaSlot::VmExtra => {
                        let key = (m.owner, "vm_sha_extra_used".to_string());
                        let next = ctx.core.vm_aux.flags.get(&key).copied().unwrap_or(0) + 1;
                        ctx.core.vm_aux.flags.insert(key, next);
                    }
                    HufuShaSlot::Unlimited | HufuShaSlot::Normal | HufuShaSlot::LandBonus => {}
                }
                ctx.emit(EventData::CardUsed(crate::machine::card_use_as(
                    card,
                    m.owner,
                    vec![t],
                    Some(CardKind::Sha),
                )));
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn ask_hufu(
    stage: HostStage,
    owner: PlayerId,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    let cards_ask = |prompt: &'static str, opts: Vec<CardId>| {
        Some(build_ask(
            owner,
            prompt,
            ChoiceKind::SelectCards,
            opts.into_iter().map(ChoiceOption::Card).collect(),
            1,
            1,
            true,
        ))
    };
    match stage {
        HostStage::TargetChoice => cards_ask(
            "u05_hufu_sha_card",
            hufu_sha_eligible_cards(catalog, core, owner),
        ),
        HostStage::Act if cards.is_empty() => cards_ask(
            "u05_hufu_jiu_card",
            hufu_jiu_eligible_cards(catalog, core, owner),
        ),
        HostStage::Act => Some(build_ask(
            owner,
            "u05_hufu_sha_target",
            ChoiceKind::SelectPlayers,
            sha_targets_for_hufu(catalog, core, owner, cards[0])
                .into_iter()
                .map(ChoiceOption::Player)
                .collect(),
            1,
            1,
            true,
        )),
        _ => None,
    }
}
