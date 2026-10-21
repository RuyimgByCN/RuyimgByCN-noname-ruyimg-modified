use super::*;

mod effect_choose_discard_one_from_each_linked;
mod effect_draw_cards;

pub(in super::super) fn items() -> Vec<LspCompletionItem> {
    let mut items = effect_choose_discard_one_from_each_linked::items();
    items.extend(effect_draw_cards::items());
    items
}
