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

fn tao_save_canceled(
    core: &GameStateCore,
    catalog: &Catalog,
    responder: PlayerId,
    card: CardId,
    dying: PlayerId,
) -> bool {
    crate::rvm_bridge::modifiers::eval_dying_peach_target_canceled(
        core, catalog, responder, dying, card,
    ) || crate::rvm_bridge::modifiers::eval_card_use_canceled(
        core,
        catalog,
        responder,
        card,
        catalog.card_kind(card),
        crate::event::DamageNature::Normal,
    )
}

/// 濒死救援的桃判定扩面（与 kernel `is_tao_like` 一致）。
pub(in crate::snap) fn is_tao_like(
    core: &GameStateCore,
    catalog: &Catalog,
    responder: PlayerId,
    card: CardId,
    dying: PlayerId,
) -> bool {
    if tao_save_canceled(core, catalog, responder, card, dying) {
        return false;
    }
    let kind = catalog.card_kind(card);
    if kind == CardKind::Hufu {
        return responder == dying;
    }
    if kind == CardKind::Tao {
        return !crate::kernel::tao_futai2_banned(core, catalog, responder);
    }
    if matches!(kind, CardKind::Jiu | CardKind::Tianxianjiu | CardKind::Dinvxuanshuang) {
        return true;
    }
    let suit = catalog.cards[card as usize].suit;
    let jijiu = catalog
        .skill_by_name(JI_JIU)
        .map(|d| core.has_skill(responder, d))
        .unwrap_or(false);
    if jijiu
        && matches!(
            suit,
            crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
        )
    {
        return true;
    }
    let mizhilianou = catalog
        .skill_by_name(GJ_FOOD_MIZHLIANOU_KEY)
        .map(|d| core.has_skill(responder, d))
        .unwrap_or(false);
    if mizhilianou && suit == crate::catalog::Suit::Heart {
        return true;
    }
    if catalog
        .find_player_vm_view_as(core, responder, card, false, Some(CardKind::Tao))
        .is_some()
        && !crate::kernel::tao_futai2_banned(core, catalog, responder)
    {
        return true;
    }
    if responder == dying
        && catalog
            .find_player_vm_view_as(core, responder, card, false, Some(CardKind::Jiu))
            .is_some()
    {
        return true;
    }
    false
}

pub(in crate::snap) fn append_vm_shan_options(
    core: &GameStateCore,
    catalog: &Catalog,
    target: PlayerId,
    options: &mut Vec<ChoiceOption>,
) {
    let p = core.player(target);
    for &c in &p.hand {
        if catalog
            .find_player_vm_view_as(core, target, c, false, Some(CardKind::Shan))
            .is_some()
            && !options.contains(&ChoiceOption::Card(c))
        {
            options.push(ChoiceOption::Card(c));
        }
    }
    for c in catalog.player_vm_view_as_extra_cards(core, target, false, Some(CardKind::Shan)) {
        if !options.contains(&ChoiceOption::Card(c)) {
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
    let mut options = vec![ChoiceOption::Pass];
    let p = core.player(responder);
    for &c in &p.hand {
        if is_tao_like(core, catalog, responder, c, dying) {
            options.push(ChoiceOption::Card(c));
        }
    }
    for c in catalog.player_vm_view_as_extra_cards(core, responder, false, None) {
        if is_tao_like(core, catalog, responder, c, dying)
            && !options.contains(&ChoiceOption::Card(c))
        {
            options.push(ChoiceOption::Card(c));
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
    let p = core.player(responder);
    for &c in &p.hand {
        if catalog.card_kind(c) == need
            || catalog
                .find_player_vm_view_as(core, responder, c, false, Some(need))
                .is_some()
        {
            options.push(ChoiceOption::Card(c));
        }
    }
    for c in catalog.player_vm_view_as_extra_cards(core, responder, false, Some(need)) {
        if !options.contains(&ChoiceOption::Card(c)) {
            options.push(ChoiceOption::Card(c));
        }
    }
    options
}

/// 性别门控（与 standard.rs `seat_is_male` 一致）：双将槽全空 → 不适用。
pub(in crate::snap) fn seat_is_male_core(core: &GameStateCore, p: PlayerId) -> bool {
    let slots = core.player(p).general_slots;
    if slots.iter().all(|s| s.is_none()) {
        return true;
    }
    slots
        .iter()
        .flatten()
        .any(|g| general_gender(*g) == Some(Gender::Male))
}
