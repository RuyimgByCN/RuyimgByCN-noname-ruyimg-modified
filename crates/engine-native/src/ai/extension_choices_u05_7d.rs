//! U05-7d (`swd` 角色技能收口批) 挂起询问的公开信息集启发式策略。

use super::*;
use crate::choice::ChoiceOption;

fn pick_hostile_player(
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

fn pick_highest_rank_card(catalog: &crate::catalog::Catalog, options: &[ChoiceOption]) -> Vec<ChoiceOption> {
    options
        .iter()
        .filter_map(|o| match o {
            ChoiceOption::Card(c) => catalog.cards.get(*c as usize).map(|def| (def.rank, *o)),
            _ => None,
        })
        .max_by_key(|(r, _)| *r)
        .map(|(_, o)| vec![o])
        .unwrap_or_default()
}

fn pick_lowest_rank_card(catalog: &crate::catalog::Catalog, options: &[ChoiceOption]) -> Vec<ChoiceOption> {
    options
        .iter()
        .filter_map(|o| match o {
            ChoiceOption::Card(c) => catalog.cards.get(*c as usize).map(|def| (def.rank, *o)),
            _ => None,
        })
        .min_by_key(|(r, _)| *r)
        .map(|(_, o)| vec![o])
        .unwrap_or_default()
}

fn pick_same_suit_pair(
    catalog: &crate::catalog::Catalog,
    options: &[ChoiceOption],
    allow_cancel: bool,
    min: usize,
) -> Vec<ChoiceOption> {
    let cards: Vec<(crate::catalog::Suit, ChoiceOption)> = options
        .iter()
        .filter_map(|o| match o {
            ChoiceOption::Card(c) => catalog.cards.get(*c as usize).map(|def| (def.suit, *o)),
            _ => None,
        })
        .collect();
    for (idx, (suit, first)) in cards.iter().enumerate() {
        if let Some((_, second)) = cards.iter().skip(idx + 1).find(|(s, _)| s == suit) {
            return vec![*first, *second];
        }
    }
    if allow_cancel {
        Vec::new()
    } else {
        options.iter().take(min).copied().collect()
    }
}

fn pick_guanhu_cards(
    input: &crate::view::PolicyInput,
    options: &[ChoiceOption],
    max: usize,
    allow_cancel: bool,
) -> Vec<ChoiceOption> {
    let mut hand_pick = None;
    let mut equip_pick = None;
    for &opt in options {
        if let ChoiceOption::Card(c) = opt {
            let is_equip = input
                .view
                .players
                .iter()
                .any(|p| p.equips.contains(&c));
            if is_equip && equip_pick.is_none() {
                equip_pick = Some(opt);
            } else if !is_equip && hand_pick.is_none() {
                hand_pick = Some(opt);
            }
        }
    }
    let mut picked = Vec::new();
    if let Some(h) = hand_pick {
        picked.push(h);
    }
    if picked.len() < max {
        if let Some(e) = equip_pick {
            picked.push(e);
        }
    }
    if picked.is_empty() && !allow_cancel {
        options.iter().take(1).copied().collect()
    } else {
        picked
    }
}

fn pick_distinct_suit_cards(
    catalog: &crate::catalog::Catalog,
    options: &[ChoiceOption],
    min: usize,
    max: usize,
    allow_cancel: bool,
) -> Vec<ChoiceOption> {
    let mut seen_suits = Vec::new();
    let mut picked = Vec::new();
    for &opt in options {
        if let ChoiceOption::Card(c) = opt {
            if let Some(def) = catalog.cards.get(c as usize) {
                if !seen_suits.contains(&def.suit) {
                    seen_suits.push(def.suit);
                    picked.push(opt);
                    if picked.len() >= max {
                        break;
                    }
                }
            }
        }
    }
    if picked.len() >= min {
        picked
    } else if allow_cancel {
        Vec::new()
    } else {
        options.iter().take(min).copied().collect()
    }
}

pub(super) fn plan_u05_7d_choice_action(
    catalog: &crate::catalog::Catalog,
    input: &crate::view::PolicyInput,
    id: u64,
    prompt: &str,
    options: &[ChoiceOption],
    min: usize,
    max: usize,
    allow_cancel: bool,
) -> Option<GameAction> {
    let sel = match prompt {
        "duijue_target" | "tanlin_target" | "oldliaoyuan_target" | "oldliaoyuan2_target" => {
            pick_hostile_player(input, options, allow_cancel)
        }
        "tanlin_pindian" | "tanlin_target_pindian" => pick_highest_rank_card(catalog, options),
        "tanlin_defence_discard" => {
            let one = pick_lowest_rank_card(catalog, options);
            if one.is_empty() && allow_cancel {
                Vec::new()
            } else {
                one
            }
        }
        "oldliaoyuan_cards" | "oldliaoyuan2_cards" => {
            pick_same_suit_pair(catalog, options, allow_cancel, min)
        }
        "oldliaoyuan_target_discard" | "toudan_neighbor_discard" => {
            options.iter().take(min.max(1)).copied().collect()
        }
        "xuanzhou_delay_kind" => options
            .iter()
            .find(|o| matches!(o, ChoiceOption::Tag(_)))
            .copied()
            .into_iter()
            .collect(),
        "lqingcheng_continue" => {
            if options.contains(&ChoiceOption::Confirm) {
                vec![ChoiceOption::Confirm]
            } else {
                Vec::new()
            }
        }
        "guanhu_discard" => pick_guanhu_cards(input, options, max, allow_cancel),
        "shehun_discard" => pick_distinct_suit_cards(catalog, options, min, max, allow_cancel),
        "rexue_sha" => {
            if super::public_attitude(&input.view, input.view.seat, input.view.turn) <= 0 {
                options
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    })
                    .min_by_key(|&c| super::extension_card_value(catalog, input, c))
                    .into_iter()
                    .map(ChoiceOption::Card)
                    .collect()
            } else {
                Vec::new()
            }
        }
        _ => return None,
    };
    Some(GameAction::Answer {
        choice_id: id,
        selection: sel,
    })
}
