//! 开放选择期望形状（ExpectedAsk）与通用选项构造。

use super::*;

/// 开放选择的期望形状（由挂起机器的 stage/asked 派生）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpectedAsk {
    pub observer: PlayerId,
    pub prompt: &'static str,
    pub kind: ChoiceKind,
    pub options: Vec<ChoiceOption>,
    pub min: usize,
    pub max: usize,
    pub allow_cancel: bool,
    /// Derived ask metadata, excluded from snapshots and formal shape validation.
    pub source_skill: Option<&'static str>,
}

pub fn ask(
    observer: PlayerId,
    prompt: &'static str,
    kind: ChoiceKind,
    options: Vec<ChoiceOption>,
    min: usize,
    max: usize,
    allow_cancel: bool,
) -> ExpectedAsk {
    ExpectedAsk {
        observer,
        prompt,
        kind,
        options,
        min,
        max,
        allow_cancel,
        source_skill: None,
    }
}

pub(in crate::snap) fn confirm_ask(observer: PlayerId, prompt: &'static str) -> ExpectedAsk {
    ask(
        observer,
        prompt,
        ChoiceKind::Confirm,
        vec![ChoiceOption::Confirm],
        1,
        1,
        true,
    )
}

pub fn hand_card_options(hand: &[CardId]) -> Vec<ChoiceOption> {
    hand.iter().copied().map(ChoiceOption::Card).collect()
}

pub fn other_player_options(core: &GameStateCore, owner: PlayerId) -> Vec<ChoiceOption> {
    core.alive_players()
        .filter(|p| *p != owner)
        .map(ChoiceOption::Player)
        .collect()
}

/// 濒死救援的桃判定扩面（直接复用 kernel `is_tao_like_core`，消除运行时/快照漂移）。
pub(in crate::snap) fn is_tao_like(
    core: &GameStateCore,
    catalog: &Catalog,
    responder: PlayerId,
    card: CardId,
    dying: PlayerId,
) -> bool {
    crate::kernel::is_tao_like_core(core, catalog, responder, card, dying)
}

pub(in crate::snap) fn append_vm_shan_options(
    core: &GameStateCore,
    catalog: &Catalog,
    target: PlayerId,
    options: &mut Vec<ChoiceOption>,
) {
    let hand_banned = core
        .vm_aux
        .flags
        .get(&(target, "mbcmfangzhu_ban".to_string()))
        .copied()
        .unwrap_or(0)
        > 0
        || crate::rvm_bridge::modifiers::d34_flags::is_u01_hand_locked(core, catalog, target)
        || catalog.blocks_hand_cards(core, target);
    let p = core.player(target);
    if !hand_banned {
        for &c in &p.hand {
            if catalog
                .find_player_vm_view_as(core, target, c, false, Some(CardKind::Shan))
                .is_some()
                && !options.contains(&ChoiceOption::Card(c))
            {
                options.push(ChoiceOption::Card(c));
            }
        }
    }
    for c in catalog.player_vm_view_as_extra_cards(core, target, false, Some(CardKind::Shan)) {
        if (!(crate::rvm_bridge::modifiers::d34_flags::is_u01_hand_locked(core, catalog, target)
            || catalog.blocks_hand_cards(core, target))
            || !p.hand.contains(&c))
            && !options.contains(&ChoiceOption::Card(c))
        {
            options.push(ChoiceOption::Card(c));
        }
    }
}

pub(in crate::snap) fn dying_save_options(
    core: &GameStateCore,
    catalog: &Catalog,
    responder: PlayerId,
    dying: PlayerId,
) -> Vec<ChoiceOption> {
    if catalog.blocks_card_use_and_save(core, responder) {
        return vec![ChoiceOption::Pass];
    }
    let hand_locked =
        crate::rvm_bridge::modifiers::d34_flags::is_u01_hand_locked(core, catalog, responder)
            || catalog.blocks_hand_cards(core, responder);
    let mut options = vec![ChoiceOption::Pass];
    if !crate::kernel::mbcmfangzhu_banned(core, responder) {
        let p = core.player(responder);
        if !hand_locked {
            for &c in &p.hand {
                if !crate::kernel::jinyan_blocks_black_trick(core, catalog, responder, c)
                    && is_tao_like(core, catalog, responder, c, dying)
                {
                    options.push(ChoiceOption::Card(c));
                }
            }
        }
        let has_mizhilianou = catalog
            .skill_by_name(crate::vocabulary::host::GJ_FOOD_MIZHLIANOU_KEY)
            .is_some_and(|d| core.has_skill(responder, d))
            || catalog
                .skill_by_name("mizhilianou")
                .is_some_and(|d| core.has_skill(responder, d));
        if has_mizhilianou {
            for &c in &p.equips {
                if catalog.resolved_card_suit(core, c) == crate::catalog::Suit::Heart
                    && !crate::kernel::jinyan_blocks_black_trick(core, catalog, responder, c)
                    && is_tao_like(core, catalog, responder, c, dying)
                    && !options.contains(&ChoiceOption::Card(c))
                {
                    options.push(ChoiceOption::Card(c));
                }
            }
        }
        for c in catalog.player_vm_view_as_extra_cards(core, responder, false, None) {
            if (!hand_locked || !p.hand.contains(&c))
                && !crate::kernel::jinyan_blocks_black_trick(core, catalog, responder, c)
                && is_tao_like(core, catalog, responder, c, dying)
                && !options.contains(&ChoiceOption::Card(c))
            {
                options.push(ChoiceOption::Card(c));
            }
        }
        if let Some(def) = crate::kernel::mbcmjiushi_dying_skill(core, catalog, responder, dying) {
            options.push(ChoiceOption::Skill(def));
        }
        if let Some(def) = crate::kernel::gwfusheng_dying_skill(core, catalog, responder, dying) {
            if !options.contains(&ChoiceOption::Skill(def)) {
                options.push(ChoiceOption::Skill(def));
            }
        }
    }
    for rdef in crate::kernel::dying_response_skills(core, catalog, responder, dying) {
        options.push(ChoiceOption::Skill(rdef));
    }
    for opt in crate::kernel::dying_save_skill_options(core, catalog, responder, dying) {
        if !options.contains(&opt) {
            options.push(opt);
        }
    }
    for rdef in (catalog.hooks.hearth_dying_save_skills)(core, catalog, responder, dying) {
        if !options.contains(&ChoiceOption::Skill(rdef)) {
            options.push(ChoiceOption::Skill(rdef));
        }
    }
    options
}

pub(in crate::snap) fn response_kind_options(
    core: &GameStateCore,
    catalog: &Catalog,
    responder: PlayerId,
    need: CardKind,
) -> Vec<ChoiceOption> {
    let mut options = vec![ChoiceOption::Pass];
    options.extend(
        crate::kernel::response_kind_cards_core(core, catalog, responder, need)
            .into_iter()
            .map(ChoiceOption::Card),
    );
    options
}

/// 性别门控（与 standard.rs `seat_is_male_in` 一致）：双将槽全空 → 不适用。
pub(in crate::snap) fn seat_is_male_core(
    core: &GameStateCore,
    catalog: &Catalog,
    p: PlayerId,
) -> bool {
    let slots = core.player(p).general_slots;
    if slots.iter().all(|s| s.is_none()) {
        return true;
    }
    core.matches_gender(catalog, p, Gender::Male)
        || (core
            .vm_aux
            .flags
            .get(&(p, "effective_gender_override".to_string()))
            .copied()
            .unwrap_or(0)
            == 0
            && slots
                .iter()
                .flatten()
                .any(|g| general_gender(*g) == Some(Gender::Male)))
}
