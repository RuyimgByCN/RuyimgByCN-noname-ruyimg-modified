//! U01 Sub-batch 3 挂起询问的公开信息集启发式策略。

use super::super::*;
use super::{pick_ally_or_first, pick_hostile_or_first, pick_lowest_cards};
use crate::choice::ChoiceOption;

fn pick_highest_cards(
    catalog: &crate::catalog::Catalog,
    _input: &crate::view::PolicyInput,
    options: &[ChoiceOption],
    count: usize,
) -> Vec<ChoiceOption> {
    let mut cards: Vec<crate::ids::CardId> = options
        .iter()
        .filter_map(|o| match o {
            ChoiceOption::Card(c) => Some(*c),
            _ => None,
        })
        .collect();
    cards.sort_by_key(|&c| {
        std::cmp::Reverse(catalog.cards.get(c as usize).map_or(0, |d| d.rank))
    });
    cards.into_iter().take(count).map(ChoiceOption::Card).collect()
}

fn find_sum_13(catalog: &crate::catalog::Catalog, options: &[ChoiceOption]) -> Vec<ChoiceOption> {
    let cards: Vec<crate::ids::CardId> = options
        .iter()
        .filter_map(|o| match o {
            ChoiceOption::Card(c) => Some(*c),
            _ => None,
        })
        .collect();
    let n = cards.len();
    fn dfs(
        catalog: &crate::catalog::Catalog,
        cards: &[crate::ids::CardId],
        idx: usize,
        sum: u32,
        cur: &mut Vec<ChoiceOption>,
    ) -> Option<Vec<ChoiceOption>> {
        if sum == 13 && !cur.is_empty() {
            return Some(cur.clone());
        }
        if sum > 13 || idx >= cards.len() {
            return None;
        }
        let c = cards[idx];
        let r = u32::from(catalog.cards.get(c as usize).map_or(0, |d| d.rank));
        if r > 0 && sum + r <= 13 {
            cur.push(ChoiceOption::Card(c));
            if let Some(res) = dfs(catalog, cards, idx + 1, sum + r, cur) {
                return Some(res);
            }
            cur.pop();
        }
        dfs(catalog, cards, idx + 1, sum, cur)
    }
    let _ = n;
    let mut cur = Vec::new();
    dfs(catalog, &cards, 0, 0, &mut cur).unwrap_or_default()
}

pub(super) fn plan_u01_3_choice_action(
    catalog: &crate::catalog::Catalog,
    input: &crate::view::PolicyInput,
    id: u64,
    prompt: &str,
    options: &[ChoiceOption],
    min: usize,
    max: usize,
    allow_cancel: bool,
) -> Option<GameAction> {
    let seat = input.view.seat;
    let sel = match prompt {
        "dddxiaheng_discard_target" | "dddxiaheng_damage_target" | "dddfenye_target"
        | "dddfenye_sha_target" | "dddduanbing_target" | "dddzhuanshe_add_target"
        | "dddchengjing_target" | "dddchengjing_jiedao_victim" | "dddbailei_target" => {
            pick_hostile_or_first(input, options, allow_cancel)
        }
        "dddxiaheng_discard_cards" | "dddfengzheng_give_cards" | "dddyulv_put"
        | "dddyulv_discard" | "dddyulv_swap" | "dddyulv_old_discard" | "dddshichao_discard"
        | "dddchengjing_card" | "dddduanbing_card" | "dddzhuanshe_give" | "dddxuanlun_cards"
        | "rezhiheng_cards" | "xiangle_discard" | "dddfenye_join_card" => {
            if prompt == "xiangle_discard" && allow_cancel && options.is_empty() {
                Vec::new()
            } else {
                pick_lowest_cards(catalog, input, options, min.max(1).min(max))
            }
        }
        "dddfengzheng_cards" => {
            let sub = find_sum_13(catalog, options);
            if !sub.is_empty() {
                sub
            } else if allow_cancel {
                Vec::new()
            } else {
                options.iter().take(min).copied().collect()
            }
        }
        "dddfengzheng_give_target" => pick_ally_or_first(input, options, allow_cancel),
        "dddfenye_owner_card" | "dddfenye_target_card" => {
            pick_highest_cards(catalog, input, options, 1)
        }
        "dddfenye_join_side" => pick_ally_or_first(input, options, allow_cancel),
        "dddyulv_mode" | "dddyulv_old_mode" => {
            let cur = input.view.turn;
            let friendly = cur == seat || public_attitude(&input.view, seat, cur) > 0;
            if friendly && options.contains(&ChoiceOption::Tag(0)) {
                vec![ChoiceOption::Tag(0)]
            } else if !friendly && options.contains(&ChoiceOption::Tag(1)) {
                vec![ChoiceOption::Tag(1)]
            } else {
                options.iter().take(1).copied().collect()
            }
        }
        "dddyulv_old_declare" => options.iter().take(1).copied().collect(),
        "dddshichao_target" => {
            let best = options
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Player(t) => Some(*t),
                    _ => None,
                })
                .max_by_key(|&t| {
                    let h = input.view.players.get(t as usize).map_or(0, |p| p.hand_count as i64);
                    let att = public_attitude(&input.view, seat, t) as i64;
                    h * 10 - att
                });
            best.map(|t| vec![ChoiceOption::Player(t)]).unwrap_or_default()
        }
        "dddshichao_avoid_confirm" => {
            if allow_cancel {
                Vec::new()
            } else {
                vec![ChoiceOption::Confirm]
            }
        }
        "dddyouxue_target" | "dddanzhi_target" => pick_ally_or_first(input, options, allow_cancel),
        "dddchengjing_give_confirm" | "dddduanbing_transfer" | "dddlianer_confirm"
        | "dddxuanlun_confirm" | "dddjiexing_confirm" | "dddlanghuai_confirm"
        | "jiushi_turnover" | "dddweiqiu_confirm" | "dddfengzheng_confirm" => {
            if options.contains(&ChoiceOption::Confirm) {
                vec![ChoiceOption::Confirm]
            } else {
                options.iter().take(1).copied().collect()
            }
        }
        "dddzhuanshe_damage" => {
            if options.contains(&ChoiceOption::Confirm) {
                vec![ChoiceOption::Confirm]
            } else if allow_cancel {
                Vec::new()
            } else {
                options.iter().take(1).copied().collect()
            }
        }
        "ddddanggu_targets" => {
            let mut ranked: Vec<u8> = options
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Player(p) => Some(*p),
                    _ => None,
                })
                .collect();
            ranked.sort_by_key(|&p| public_attitude(&input.view, seat, p));
            ranked.into_iter().take(min).map(ChoiceOption::Player).collect()
        }
        "dddxuanlun_end_mode" => {
            if options.contains(&ChoiceOption::Tag(0)) {
                vec![ChoiceOption::Tag(0)]
            } else {
                options.iter().take(1).copied().collect()
            }
        }
        "dddxuanlun_top" => pick_highest_cards(catalog, input, options, 2.min(max)),
        "dddxuanlun_bottom" => options.iter().take(max).copied().collect(),
        "dddbailei_pick_card" => options.iter().take(1).copied().collect(),
        _ => return None,
    };
    Some(GameAction::Answer {
        choice_id: id,
        selection: sel,
    })
}
