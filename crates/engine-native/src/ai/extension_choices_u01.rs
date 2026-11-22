//! U01 Sub-batch 1 挂起询问的公开信息集启发式策略。

use super::*;
use crate::choice::ChoiceOption;

#[path = "extension_choices_u01_2.rs"]
mod extension_choices_u01_2;

pub(super) fn pick_hostile_or_first(
    input: &crate::view::PolicyInput,
    options: &[ChoiceOption],
    allow_cancel: bool,
) -> Vec<ChoiceOption> {
    let seat = input.view.seat;
    let best = options
        .iter()
        .filter_map(|o| match o {
            ChoiceOption::Player(t) => Some(*t),
            _ => None,
        })
        .filter(|&t| public_attitude(&input.view, seat, t) < 0)
        .max_by_key(|&t| {
            let att = public_attitude(&input.view, seat, t) as i64;
            let hp = input
                .view
                .players
                .get(t as usize)
                .map(|p| p.hp as i64)
                .unwrap_or(0);
            (-att) * 10 - hp
        });
    match best {
        Some(t) => vec![ChoiceOption::Player(t)],
        None if allow_cancel => Vec::new(),
        None => options.iter().take(1).copied().collect(),
    }
}

pub(super) fn pick_ally_or_first(
    input: &crate::view::PolicyInput,
    options: &[ChoiceOption],
    allow_cancel: bool,
) -> Vec<ChoiceOption> {
    let seat = input.view.seat;
    let best = options
        .iter()
        .filter_map(|o| match o {
            ChoiceOption::Player(t) => Some(*t),
            _ => None,
        })
        .filter(|&t| public_attitude(&input.view, seat, t) > 0)
        .max_by_key(|&t| public_attitude(&input.view, seat, t));
    match best {
        Some(t) => vec![ChoiceOption::Player(t)],
        None if allow_cancel => Vec::new(),
        None => options.iter().take(1).copied().collect(),
    }
}

pub(super) fn pick_lowest_cards(
    catalog: &crate::catalog::Catalog,
    input: &crate::view::PolicyInput,
    options: &[ChoiceOption],
    count: usize,
) -> Vec<ChoiceOption> {
    let mut cards: Vec<(i64, ChoiceOption)> = options
        .iter()
        .copied()
        .filter_map(|o| match o {
            ChoiceOption::Card(c) => Some((extension_card_value(catalog, input, c), o)),
            _ => None,
        })
        .collect();
    cards.sort_by_key(|(v, _)| *v);
    cards.into_iter().take(count).map(|(_, o)| o).collect()
}

pub(super) fn plan_u01_choice_action(
    catalog: &crate::catalog::Catalog,
    input: &crate::view::PolicyInput,
    id: u64,
    prompt: &'static str,
    options: &[ChoiceOption],
    min: usize,
    max: usize,
    allow_cancel: bool,
) -> Option<GameAction> {
    let seat = input.view.seat;
    let sel: Vec<ChoiceOption> = match prompt {
        "dddyeshen_card" => options
            .iter()
            .filter_map(|&o| match o {
                ChoiceOption::Card(c) => Some((
                    catalog.cards.get(c as usize).map(|d| d.rank).unwrap_or(0),
                    o,
                )),
                _ => None,
            })
            .max_by_key(|(r, _)| *r)
            .map(|(_, o)| vec![o])
            .unwrap_or_else(|| options.iter().take(1).copied().collect()),
        "dddyeshen_target" => {
            let hostile: Vec<ChoiceOption> = options
                .iter()
                .copied()
                .filter(|o| matches!(o, ChoiceOption::Player(t) if public_attitude(&input.view, seat, *t) < 0))
                .take(max)
                .collect();
            if hostile.len() >= min {
                hostile
            } else if allow_cancel {
                Vec::new()
            } else {
                options.iter().take(min.max(1)).copied().collect()
            }
        }
        "dddguiying_confirm" | "dddlangzhi_confirm" => vec![ChoiceOption::Confirm],
        "dddguiying_give_card"
        | "ddddiedang_discard"
        | "dddbingjian_discard"
        | "dddzhilian_discard" => pick_lowest_cards(catalog, input, options, min.max(1)),
        "dddguiying_gain_target" => pick_hostile_or_first(input, options, allow_cancel),
        "dddguiying_gain_card" | "dddjijian_huogong_show" | "dddjijian_huogong_discard" => {
            options.iter().take(1).copied().collect()
        }
        "dddfushi_target_pile" => options.iter().take(1).copied().collect(),
        "dddlangzhi_reveal_target_card" | "dddfuzong_discard" => {
            options.iter().take(1).copied().collect()
        }
        "dddlangzhi_own_card" => {
            let take = min.max(1).min(options.len());
            pick_lowest_cards(catalog, input, options, take)
        }
        "dddlangzhi_swap_cards" => options.iter().take(min.max(1)).copied().collect(),
        "dddbingjian_discard_target" => options.iter().take(min.max(1)).copied().collect(),
        "dddbingjian_sha_target" | "dddshilie_target" => {
            pick_hostile_or_first(input, options, allow_cancel)
        }
        "dddtongyu_cards" | "dddjungui_discard" => {
            let mut suits = Vec::new();
            let mut chosen = Vec::new();
            for &opt in options {
                if chosen.len() >= max {
                    break;
                }
                if let ChoiceOption::Card(c) = opt {
                    let s = catalog.cards[c as usize].suit;
                    if !suits.contains(&s) {
                        suits.push(s);
                        chosen.push(opt);
                    }
                }
            }
            if chosen.len() >= min {
                chosen
            } else if allow_cancel {
                Vec::new()
            } else {
                chosen
            }
        }
        "dddtongyu_direction" => options.iter().take(1).copied().collect(),
        "dddtongyu_remain_target" => pick_ally_or_first(input, options, allow_cancel),
        "dddshilie_cards" => {
            let own_hp = input.view.players.get(seat as usize).map(|p| p.hp).unwrap_or(0);
            let cur_hp = input
                .view
                .players
                .get(input.view.turn as usize)
                .map(|p| p.hp)
                .unwrap_or(0);
            let need = own_hp + cur_hp;
            let mut ranked: Vec<(u8, ChoiceOption)> = options
                .iter()
                .copied()
                .filter_map(|o| match o {
                    ChoiceOption::Card(c) => Some((catalog.cards[c as usize].rank, o)),
                    _ => None,
                })
                .collect();
            ranked.sort_by(|a, b| b.0.cmp(&a.0));
            let mut sum = 0i32;
            let mut picked = Vec::new();
            for (r, o) in ranked {
                picked.push(o);
                sum += i32::from(r);
                if sum >= need && picked.len() >= min {
                    break;
                }
            }
            if sum >= need && picked.len() >= min && picked.len() <= max {
                picked
            } else if allow_cancel {
                Vec::new()
            } else {
                options.iter().take(min).copied().collect()
            }
        }
        "dddjijian_show" => {
            let mut reds = Vec::new();
            let mut blacks = Vec::new();
            for &opt in options {
                if let ChoiceOption::Card(c) = opt {
                    let is_red = matches!(
                        catalog.cards[c as usize].suit,
                        crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                    );
                    if is_red {
                        reds.push(opt);
                    } else {
                        blacks.push(opt);
                    }
                }
            }
            let chosen = if reds.len() >= blacks.len() { reds } else { blacks };
            chosen.into_iter().take(max).collect()
        }
        "dddjijian_use_card" | "dddjijian_old_use_card" => {
            options.iter().take(1).copied().collect()
        }
        "dddjijian_use_target" | "dddjijian_old_use_target" => {
            pick_hostile_or_first(input, options, allow_cancel)
        }
        _ => {
            return extension_choices_u01_2::plan_u01_2_choice_action(
                catalog,
                input,
                id,
                prompt,
                options,
                min,
                max,
                allow_cancel,
            );
        }
    };
    Some(GameAction::Answer {
        choice_id: id,
        selection: sel,
    })
}
