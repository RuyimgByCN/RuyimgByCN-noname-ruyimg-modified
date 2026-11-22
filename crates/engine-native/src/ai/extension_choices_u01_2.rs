//! U01 Sub-batch 2 挂起询问的公开信息集启发式策略。

use super::super::extension_choices_u01::{
    pick_ally_or_first, pick_hostile_or_first, pick_lowest_cards,
};
use super::*;
use crate::choice::ChoiceOption;

pub(super) fn plan_u01_2_choice_action(
    catalog: &crate::catalog::Catalog,
    input: &crate::view::PolicyInput,
    id: u64,
    prompt: &'static str,
    options: &[ChoiceOption],
    min: usize,
    _max: usize,
    allow_cancel: bool,
) -> Option<GameAction> {
    let seat = input.view.seat;
    let selection: Vec<ChoiceOption> = match prompt {
        "dddzhengjun_hp_choice" | "dddzhengjun_hs_choice" => {
            if options.contains(&ChoiceOption::Tag(1)) {
                vec![ChoiceOption::Tag(1)]
            } else {
                options.iter().take(1).copied().collect()
            }
        }
        "dddzhengjun_hp_target" | "dddzhengjun_hs_target" => {
            pick_hostile_or_first(input, options, allow_cancel)
        }
        "dddzhengjun_es_confirm"
        | "dddlingyong_confirm"
        | "dddlingyong_repeat"
        | "dddxuyu_confirm"
        | "dddxuyu_old_confirm"
        | "ddddongcha_confirm"
        | "dddxunxun_confirm"
        | "dddtuoji_confirm" => vec![ChoiceOption::Confirm],
        "dddshijian_old_confirm" => {
            let cur = input.view.turn;
            if public_attitude(&input.view, seat, cur) > 0 || !allow_cancel {
                vec![ChoiceOption::Confirm]
            } else {
                Vec::new()
            }
        }
        "dddzhengjun_hs_discard"
        | "dddxianxi_discard"
        | "dddxuyu_discard"
        | "dddxuyu_old_discard"
        | "dddchashi_discard"
        | "dddfenji_discard"
        | "dddmiaoxing_discard"
        | "dddquche_red_card"
        | "dddquche_black_sha"
        | "ddddongcha_owner_put"
        | "ddddongcha_target_put" => pick_lowest_cards(catalog, input, options, min.max(1)),
        "dddzhengjun_es_card" | "dddzhengjun_es_dest" => {
            options.iter().take(1).copied().collect()
        }
        "dddxianxi_extra_target" => {
            let hostile: Vec<ChoiceOption> = options
                .iter()
                .filter(|o| matches!(o, ChoiceOption::Player(t) if public_attitude(&input.view, seat, *t) < 0))
                .copied()
                .collect();
            if !hostile.is_empty() {
                hostile
            } else if allow_cancel {
                Vec::new()
            } else {
                options.iter().take(1).copied().collect()
            }
        }
        "dddlingyong_use_target" => {
            if options.iter().any(|o| matches!(o, ChoiceOption::Player(_))) {
                pick_hostile_or_first(input, options, allow_cancel)
            } else {
                vec![ChoiceOption::Confirm]
            }
        }
        "dddlingyong_jiedao_victim"
        | "dddquche_sha_target"
        | "dddfusi_target"
        | "dddfusi_jiedao_victim"
        | "dddfenji_target"
        | "dddfenji_jiedao_victim" => pick_hostile_or_first(input, options, allow_cancel),
        "dddxuxiao_card" => options.iter().take(1).copied().collect(),
        "dddxuxiao_target" | "dddquche_give_target" | "dddtaisi_target" => {
            pick_ally_or_first(input, options, allow_cancel)
        }
        "dddshijian_give" => {
            if public_attitude(&input.view, seat, input.view.turn) > 0 || !allow_cancel {
                pick_lowest_cards(catalog, input, options, 1)
            } else {
                Vec::new()
            }
        }
        "dddshijian_mode" => {
            if public_attitude(&input.view, seat, input.view.turn) > 0
                && options.contains(&ChoiceOption::Tag(0))
            {
                vec![ChoiceOption::Tag(0)]
            } else if options.contains(&ChoiceOption::Tag(1)) {
                vec![ChoiceOption::Tag(1)]
            } else {
                options.iter().take(1).copied().collect()
            }
        }
        "dddtaisi_card"
        | "ddddongcha_end_owner_pick"
        | "ddddongcha_end_target_pick" => options
            .iter()
            .filter_map(|o| match o {
                ChoiceOption::Card(c) => Some((extension_card_value(catalog, input, *c), *o)),
                _ => None,
            })
            .max_by_key(|(v, _)| *v)
            .map(|(_, o)| vec![o])
            .unwrap_or_else(|| options.iter().take(1).copied().collect()),
        "dddzhijie_cards" => options.iter().take(2).copied().collect(),
        "dddqiahua_show" => options.iter().take(min.max(1)).copied().collect(),
        "dddxunxun_top" => {
            let mut scored: Vec<(i64, ChoiceOption)> = options
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Card(c) => Some((extension_card_value(catalog, input, *c), *o)),
                    _ => None,
                })
                .collect();
            scored.sort_by_key(|(v, _)| -*v);
            scored.into_iter().take(min.max(1)).map(|(_, o)| o).collect()
        }
        "dddxunxun_bottom" => options.iter().take(min.max(1)).copied().collect(),
        "dddfusi_pick" | "dddmiaoxing_out_group" | "dddfenji_trick" => {
            options.iter().take(1).copied().collect()
        }
        "dddfusi_agree" => {
            let cur = input.view.turn;
            if public_attitude(&input.view, seat, cur) > 0 {
                vec![ChoiceOption::Tag(0)]
            } else {
                vec![ChoiceOption::Tag(1)]
            }
        }
        "dddmiaoxing_pile" => options.to_vec(),
        _ => return None,
    };
    Some(GameAction::Answer {
        choice_id: id,
        selection,
    })
}
