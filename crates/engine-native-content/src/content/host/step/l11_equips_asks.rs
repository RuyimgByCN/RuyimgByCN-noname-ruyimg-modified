//! L11（太虚幻境）装备与专属宝物牌技能的询问构建（`l11_equips_asks.rs`）。

use super::l11_common::*;
use super::*;
use crate::catalog::{CardKind, Catalog, EquipSlot};
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::event::DamageNature;
use crate::ids::{CardId, PlayerId};
use crate::rvm_bridge::modifiers::eval_card_target_canceled;
use crate::snap::ExpectedAsk;
use crate::state::GameStateCore;

pub(super) const ASK_NODA_AXE_DISCARD: u32 = 1;
pub(super) const ASK_NODA_AXE_RAW_DISCARD: u32 = 2;
pub(super) const ASK_QILIN_CONFIRM: u32 = 3;
pub(super) const ASK_QILIN_SKILL_CONFIRM: u32 = 4;
pub(super) const ASK_FANGTIAN_TARGETS: u32 = 5;
pub(super) const ASK_PILICHE_CONFIRM: u32 = 6;
pub(super) const ASK_LEIJIMU_CONFIRM: u32 = 7;
pub(super) const ASK_FEILONG_CONFIRM: u32 = 8;
pub(super) const ASK_FEILONG_SKILL_CONFIRM: u32 = 9;
pub(super) const ASK_MINGZHU_DISCARD: u32 = 10;
pub(super) const ASK_TAIJI_DISCARD: u32 = 11;
pub(super) const ASK_GX_TAIJI_DISCARD: u32 = 12;

pub(super) fn target_qilin_equips(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    target: PlayerId,
) -> Vec<CardId> {
    core.player(target)
        .equips
        .iter()
        .copied()
        .filter(|&c| {
            matches!(
                catalog.card_slot_of(core, c),
                Some(EquipSlot::Treasure | EquipSlot::HorsePlus | EquipSlot::HorseMinus)
            ) && !crate::rvm_bridge::modifiers::eval_card_discard_protected(
                core, catalog, owner, target, c,
            )
        })
        .collect()
}

pub(super) fn is_qilin_sha_damage(
    state: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    d: &crate::event::DamageData,
) -> bool {
    match d.card {
        Some(c) => {
            catalog.card_kind(c).is_sha_like()
                || state.declared_kinds.get(&c).is_some_and(|k| k.is_sha_like())
        }
        None => state
            .vm_aux
            .flags
            .keys()
            .any(|(p, k)| *p == owner && k.starts_with("sha_damage_frame:")),
    }
}

pub(super) fn axe_discardable_cards(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<CardId> {
    let wpn = equip_slot_card(core, catalog, owner, EquipSlot::Weapon).filter(|&c| {
        catalog
            .cards
            .get(c as usize)
            .is_some_and(|cd| cd.name == "txhj_xingtianpojunfu" || cd.name == "txhj_noda_axe")
    });
    he_cards(core, owner)
        .into_iter()
        .filter(|&c| Some(c) != wpn)
        .collect()
}

pub(super) fn mingzhu_discardable_cards(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<CardId> {
    let treasure = equip_slot_card(core, catalog, owner, EquipSlot::Treasure).filter(|&c| {
        catalog
            .cards
            .get(c as usize)
            .is_some_and(|cd| cd.name == "txhj_dinglanyemingzhu")
    });
    he_cards(core, owner)
        .into_iter()
        .filter(|&c| Some(c) != treasure)
        .collect()
}

pub(super) fn set_fangtian_meta(
    core: &mut GameStateCore,
    owner: PlayerId,
    kind: CardKind,
    nature: DamageNature,
) {
    let kind_code = match kind {
        CardKind::FireSha => 2,
        CardKind::LeiSha => 3,
        _ => 1,
    };
    let nature_code = match nature {
        DamageNature::Fire => 2,
        DamageNature::Thunder => 3,
        _ => 1,
    };
    set_flag(core, owner, "l11_fangtian_kind", kind_code);
    set_flag(core, owner, "l11_fangtian_nature", nature_code);
}

pub(super) fn clear_fangtian_meta(core: &mut GameStateCore, owner: PlayerId) {
    set_flag(core, owner, "l11_fangtian_kind", 0);
    set_flag(core, owner, "l11_fangtian_nature", 0);
}

pub(super) fn decode_fangtian_meta(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    sha_cid: Option<CardId>,
) -> (CardKind, DamageNature) {
    let kind = match flag(core, owner, "l11_fangtian_kind") {
        2 => CardKind::FireSha,
        3 => CardKind::LeiSha,
        1 => CardKind::Sha,
        _ => sha_cid.map(|c| catalog.card_kind(c)).unwrap_or(CardKind::Sha),
    };
    let nature = match flag(core, owner, "l11_fangtian_nature") {
        2 => DamageNature::Fire,
        3 => DamageNature::Thunder,
        1 => DamageNature::Normal,
        _ => match kind {
            CardKind::FireSha => DamageNature::Fire,
            CardKind::LeiSha => DamageNature::Thunder,
            _ => DamageNature::Normal,
        },
    };
    (kind, nature)
}

pub(super) fn fangtian_extra_candidates(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    already: &[PlayerId],
    sha_card: Option<CardId>,
    sha_kind: CardKind,
    sha_nature: DamageNature,
) -> Vec<PlayerId> {
    let card_for_mod = sha_card.unwrap_or(u32::MAX);
    let range_unlimited = crate::rvm_bridge::modifiers::eval_card_distance_unlimited(
        core,
        catalog,
        owner,
        card_for_mod,
        sha_kind,
        sha_nature,
    );
    let atk_range = core.attack_range(catalog, owner);
    let kongcheng_def = catalog.skill_by_name(crate::content::standard::KONG_CHENG);
    alive_others(core, owner)
        .into_iter()
        .filter(|&p| {
            !already.contains(&p)
                && (range_unlimited || core.computed_distance(catalog, owner, p) <= atk_range)
                && !kongcheng_def.is_some_and(|d| core.has_skill(p, d) && core.player(p).hand.is_empty())
                && !eval_card_target_canceled(
                    core,
                    catalog,
                    owner,
                    p,
                    card_for_mod,
                    sha_kind,
                    sha_nature,
                )
        })
        .collect()
}

pub(super) fn build_l11_equip_ask(
    ask_id: u32,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    _card: Option<CardId>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<(PlayerId, ChoiceSpec)> {
    let tgt = target.unwrap_or(owner);
    match ask_id {
        ASK_NODA_AXE_DISCARD => card_ask(
            owner,
            "txhj_xingtianpojunfu:discard_cards",
            axe_discardable_cards(core, catalog, owner),
            2,
            2,
            true,
        ),
        ASK_NODA_AXE_RAW_DISCARD => card_ask(
            owner,
            "txhj_noda_axe:discard_cards",
            axe_discardable_cards(core, catalog, owner),
            2,
            2,
            true,
        ),
        ASK_QILIN_CONFIRM => confirm_ask(owner, "txhj_qilingong:discard_horse"),
        ASK_QILIN_SKILL_CONFIRM => {
            let equips = target_qilin_equips(core, catalog, owner, tgt);
            if equips.is_empty() {
                return None;
            }
            card_ask(owner, "qilin_skill:discard_horse", equips, 0, 1, true)
        }
        ASK_FANGTIAN_TARGETS => {
            let sha_cid = cards.first().copied();
            let (sha_kind, sha_nature) = decode_fangtian_meta(core, catalog, owner, sha_cid);
            let cands = fangtian_extra_candidates(
                core,
                catalog,
                owner,
                targets,
                sha_cid,
                sha_kind,
                sha_nature,
            );
            let max_add = 2usize.min(cands.len()).max(1);
            player_ask(
                owner,
                "txhj_fangtianhuaji:extra_targets",
                cands,
                1,
                max_add,
                false,
            )
        }
        ASK_PILICHE_CONFIRM => confirm_ask(owner, "txhj_piliche:confirm"),
        ASK_LEIJIMU_CONFIRM => confirm_ask(owner, "txhj_leijimu:confirm"),
        ASK_FEILONG_CONFIRM => confirm_ask(owner, "txhj_feilong:confirm"),
        ASK_FEILONG_SKILL_CONFIRM => confirm_ask(owner, "txhj_feilong_skill:confirm"),
        ASK_MINGZHU_DISCARD => {
            let pool = mingzhu_discardable_cards(core, catalog, owner);
            let max = (core.player(owner).max_hp as usize).min(pool.len());
            card_ask(owner, "txhj_dinglanyemingzhu:discard_cards", pool, 1, max, true)
        }
        ASK_TAIJI_DISCARD => {
            let mut opts = vec![ChoiceOption::Pass];
            opts.extend(he_cards(core, tgt).into_iter().map(ChoiceOption::Card));
            spec_ask(
                tgt,
                ChoiceKind::Confirm,
                "txhj_taijifuchen:target_discard",
                opts,
                1,
                1,
                true,
            )
        }
        ASK_GX_TAIJI_DISCARD => card_ask(
            tgt,
            "gx_taijifuchen:target_discard",
            he_cards(core, tgt),
            1,
            1,
            true,
        ),
        _ => None,
    }
}

pub(in crate::content::host) fn ask_l11_equip_cards(
    stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    card: Option<CardId>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    if stage == HostStage::Act || stage == HostStage::DrawSettle {
        return None;
    }
    let ask_id = card?;
    let (observer, spec) =
        build_l11_equip_ask(ask_id, owner, target, targets, card, cards, core, catalog)?;
    Some(crate::snap::ask(
        observer,
        spec.prompt,
        spec.kind,
        spec.options,
        spec.min,
        spec.max,
        spec.allow_cancel,
    ))
}

#[cfg(test)]
mod tests {
    use super::is_qilin_sha_damage;
    use crate::catalog::{CardDef, CardKind, Catalog};
    use crate::event::{DamageData, DamageNature, DamageSettlement, Phase};
    use crate::rng::RuleRng;
    use crate::state::{GameMode, GameStateCore, PlayerState, TurnState, VmAux};
    use std::collections::BTreeMap;

    #[test]
    fn qilin_sha_damage_ignores_secondary_non_sha_card_even_during_sha_frame() {
        let catalog = Catalog::build(
            vec![
                CardDef::basic("sha", CardKind::Sha),
                CardDef::basic("dismantle", CardKind::Dismantle),
            ],
            Vec::new(),
        );
        let mut state = GameStateCore {
            revision: 1,
            ruleset_version: 333,
            mode: GameMode::Prototype,
            players: vec![PlayerState::new(4), PlayerState::new(4)],
            seat_ring: vec![0, 1],
            deck: Vec::new(),
            discard: Vec::new(),
            detached_cards: Vec::new(),
            table: Vec::new(),
            rng: RuleRng::from_seed(2026),
            turn: TurnState {
                current: 0,
                phase: Phase::Play,
                round: 1,
                sha_used_play_phase: false,
                mufeng_used_basic: false,
                wangchen_discard_basic: false,
                shenmiguo_used: false,
                shuchui_armed: false,
                shuchui_hit: false,
                lianda_used: false,
                land_used: false,
                land_sha_used: false,
                discard_phase_discarded: 0,
                ciqiu_dying: Vec::new(),
                kangyin2: None,
                huanxia_track: Vec::new(),
                cards_used_play_phase: 0,
                cards_used_turn: 0,
                liaoyuan_count: 0,
                xuanyuanjian_marked: false,
                kunlunjing_snapshot: Vec::new(),
                kunlunjing_hp: 0,
                extra_phases: Vec::new(),
                extra_phase_lingwu: false,
                bte_used_play_phase: 0,
                phase_number: 1,
                rende_given: 0,
                jiu_used: false,
                mujiaren_used: false,
                jiu_active: false,
                turn_opened: true,
                skip_draw_phase: false,
                skip_play_phase: false,
                guxing_armed: false,
                guxing_count: 0,
                mengtai_draw_pending: false,
                mengtai_use_pending: false,
                zhufangshenshi_target: None,
                yuedong_num: 0,
                yuedong_eff: 0,
                yuedong_recover: false,
                liangou_hook: None,
                liangou_bonus_used: false,
                feitiao_target: None,
                fanling_used: false,
                diewu_draw_used: false,
                lueying_used: false,
                txianqu_used: false,
                jtjubao_used: false,
                zhuyue_targets: Vec::new(),
                sheling_used: 0,
                sheying_used: false,
                zhangmu_used: false,
                yingfeng_chain: false,
                luanjian_card: None,
                minwan_targets: Vec::new(),
                yanyu_count: 0,
                xjyufeng_used: 0,
                qizuo_copy: None,
                jidian_chain: false,
            },
            skills: BTreeMap::new(),
            declared_kinds: BTreeMap::new(),
            declared_slots: BTreeMap::new(),
            card_natures: BTreeMap::new(),
            current_land: None,
            outcome: None,
            vm_aux: VmAux::default(),
        };
        state
            .vm_aux
            .flags
            .insert((0, "sha_damage_frame:42".to_string()), 1);

        let virtual_sha = DamageData {
            source: Some(0),
            target: 1,
            amount: 1,
            settlement: DamageSettlement::default(),
            nature: DamageNature::Normal,
            prevented: false,
            redirect_chain: Vec::new(),
            card: None,
            from_chain: false,
        };
        assert!(
            is_qilin_sha_damage(&state, &catalog, 0, &virtual_sha),
            "virtual Sha (card=None) during sha_damage_frame must match"
        );

        let secondary_non_sha = DamageData {
            card: Some(1),
            ..virtual_sha
        };
        assert!(
            !is_qilin_sha_damage(&state, &catalog, 0, &secondary_non_sha),
            "secondary non-Sha card damage during sha_damage_frame must NOT match Qilin"
        );
    }
}
