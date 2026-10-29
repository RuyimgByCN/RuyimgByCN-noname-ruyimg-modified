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

/// 濒死救援的桃判定扩面（与 kernel `is_tao_like` 一致）。
pub(in crate::snap) fn is_tao_like(
    core: &GameStateCore,
    catalog: &Catalog,
    responder: PlayerId,
    card: CardId,
    dying: PlayerId,
) -> bool {
    if catalog.card_kind(card) == CardKind::Hufu {
        return responder == dying;
    }
    let kind = catalog.card_kind(card);
    if matches!(
        kind,
        CardKind::Tao | CardKind::Jiu | CardKind::Dinvxuanshuang
    ) {
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
    false
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
