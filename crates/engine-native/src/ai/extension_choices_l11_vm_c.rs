//! Batch P3-E3（`HostBehavior::L11VmFixC` + canonical `taoluan`）的 AI 公开信息决策。

use super::{extension_card_value, public_attitude};
use crate::catalog::Suit;
use crate::choice::ChoiceOption;
use crate::engine::GameAction;
use crate::ids::CardId;
use crate::view::PolicyInput;

fn suit_idx(s: Suit) -> usize {
    match s {
        Suit::Spade => 0,
        Suit::Heart => 1,
        Suit::Club => 2,
        Suit::Diamond => 3,
    }
}

fn card_rank(catalog: &crate::catalog::Catalog, cid: CardId) -> u8 {
    catalog.cards.get(cid as usize).map_or(0, |d| d.rank)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn plan_l11_vm_c_choice_action(
    catalog: &crate::catalog::Catalog,
    input: &PolicyInput,
    id: u64,
    prompt: &str,
    options: &[ChoiceOption],
    min: usize,
    max: usize,
    allow_cancel: bool,
) -> Option<GameAction> {
    let seat = input.view.seat;
    let me_hp = input.view.players.get(seat as usize).map_or(1, |p| p.hp);
    let enemies_sorted = || -> Vec<u8> {
        let mut out: Vec<u8> = options
            .iter()
            .filter_map(|o| match *o {
                ChoiceOption::Player(t) if public_attitude(&input.view, seat, t) < 0 => Some(t),
                _ => None,
            })
            .collect();
        out.sort_by_key(|&t| {
            let p = input.view.players.get(t as usize);
            (
                p.map_or(99, |x| x.hp),
                p.map_or(99, |x| x.hand_count),
                public_attitude(&input.view, seat, t),
            )
        });
        out
    };
    let allies_sorted = || -> Vec<u8> {
        let mut out: Vec<u8> = options
            .iter()
            .filter_map(|o| match *o {
                ChoiceOption::Player(t) if public_attitude(&input.view, seat, t) >= 0 => Some(t),
                _ => None,
            })
            .collect();
        out.sort_by_key(|&t| -public_attitude(&input.view, seat, t));
        out
    };
    let pick_cards_by_value = |descending: bool, count: usize| -> Vec<ChoiceOption> {
        let mut ranked: Vec<(i64, ChoiceOption)> = options
            .iter()
            .filter_map(|o| match *o {
                ChoiceOption::Card(c) => Some((extension_card_value(catalog, input, c), *o)),
                _ => None,
            })
            .collect();
        if descending {
            ranked.sort_by_key(|(v, _)| -*v);
        } else {
            ranked.sort_by_key(|(v, _)| *v);
        }
        ranked.into_iter().take(count).map(|(_, o)| o).collect()
    };

    match prompt {
        "chongzhen:confirm"
        | "dujin:confirm"
        | "juece:confirm"
        | "rejizhi:confirm"
        | "retieji:confirm"
        | "rezhenxing:confirm" => Some(GameAction::Answer {
            choice_id: id,
            selection: vec![ChoiceOption::Confirm],
        }),
        "zhuikong:confirm" => {
            let has_high = input
                .own_hand_kinds
                .iter()
                .any(|&(c, _)| card_rank(catalog, c) >= 11);
            let selection = if me_hp <= 1 && !has_high && allow_cancel {
                Vec::new()
            } else {
                vec![ChoiceOption::Confirm]
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "shelie:pick_card" => {
            let mut best_by_suit: [Option<(i64, ChoiceOption)>; 4] = [None; 4];
            for opt in options {
                let ChoiceOption::Card(c) = *opt else {
                    continue;
                };
                let Some(def) = catalog.cards.get(c as usize) else {
                    continue;
                };
                let si = suit_idx(def.suit);
                let val = extension_card_value(catalog, input, c);
                if best_by_suit[si].is_none_or(|(prev, _)| val > prev) {
                    best_by_suit[si] = Some((val, *opt));
                }
            }
            let selection = best_by_suit.into_iter().flatten().map(|(_, o)| o).collect();
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "rezhenxing:pick_card" => {
            let selection = if max == 0 || options.is_empty() {
                Vec::new()
            } else {
                pick_cards_by_value(true, 1)
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "gnjinfan:choose_cards" => {
            if allow_cancel && input.own_hand_kinds.len() <= 2 {
                return Some(GameAction::Answer {
                    choice_id: id,
                    selection: Vec::new(),
                });
            }
            let mut by_suit: [Option<(i64, ChoiceOption)>; 4] = [None; 4];
            for opt in options {
                let ChoiceOption::Card(c) = *opt else {
                    continue;
                };
                let Some(def) = catalog.cards.get(c as usize) else {
                    continue;
                };
                let si = suit_idx(def.suit);
                let val = extension_card_value(catalog, input, c);
                if by_suit[si].is_none_or(|(prev, _)| val < prev) {
                    by_suit[si] = Some((val, *opt));
                }
            }
            let mut picks: Vec<(i64, ChoiceOption)> = by_suit.into_iter().flatten().collect();
            picks.sort_by_key(|(v, _)| *v);
            let want = max.min(2).max(min);
            let selection = picks.into_iter().take(want).map(|(_, o)| o).collect();
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "nsyice:order_cards" => {
            let count = if max > 0 { min.max(1).min(max) } else { min };
            Some(GameAction::Answer {
                choice_id: id,
                selection: options.iter().take(count).copied().collect(),
            })
        }
        "nsyice:top_card" => Some(GameAction::Answer {
            choice_id: id,
            selection: pick_cards_by_value(true, 1),
        }),
        "nsfeixiong:pindian_owner"
        | "nsfeixiong:pindian_target"
        | "qiaoshui:pindian_owner"
        | "qiaoshui:pindian_target"
        | "zhuikong:pindian_owner"
        | "zhuikong:pindian_target" => {
            let pick = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Card(c) => Some((
                        card_rank(catalog, c),
                        -extension_card_value(catalog, input, c),
                        *o,
                    )),
                    _ => None,
                })
                .max_by_key(|&(r, neg_v, _)| (r, neg_v))
                .map(|(_, _, o)| o);
            Some(GameAction::Answer {
                choice_id: id,
                selection: pick.into_iter().collect(),
            })
        }
        "chongzhen:choose_card" | "xinniluan:discard_target" => Some(GameAction::Answer {
            choice_id: id,
            selection: pick_cards_by_value(true, min.max(1)),
        }),
        "reshanxi:exile_cards" => Some(GameAction::Answer {
            choice_id: id,
            selection: pick_cards_by_value(true, max.max(min).max(1)),
        }),
        "taoluan:helper_give" => {
            let helper_friendly = public_attitude(&input.view, seat, input.view.turn) >= 0;
            let selection = if helper_friendly {
                pick_cards_by_value(false, 1)
            } else if allow_cancel && min == 0 {
                Vec::new()
            } else {
                pick_cards_by_value(false, 1)
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "rejizhi:discard" => {
            let selection = match pick_cards_by_value(false, 1).first().copied() {
                Some(ChoiceOption::Card(c))
                    if !allow_cancel || min > 0 || extension_card_value(catalog, input, c) <= 120 =>
                {
                    vec![ChoiceOption::Card(c)]
                }
                _ if allow_cancel && min == 0 => Vec::new(),
                _ => options.iter().take(min).copied().collect(),
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "huoji:choose_card"
        | "longhun:choose_cards"
        | "reqiangxi:choose_weapon"
        | "reshanxi:choose_cost"
        | "retieji:discard"
        | "rezhiheng:choose_cards"
        | "taoluan:choose_card"
        | "twjiange:choose_card"
        | "twkujian:choose_cards"
        | "twkujian:discard"
        | "xinniluan:choose_card" => {
            let want = min.max(1);
            let picked = pick_cards_by_value(false, want);
            let selection = if picked.len() >= min {
                picked
            } else if allow_cancel {
                Vec::new()
            } else {
                options.iter().take(want).copied().collect()
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "huoji:choose_target"
        | "longhun:choose_target"
        | "mieji:choose_target"
        | "nsfeixiong:choose_target"
        | "nsyice:damage_target"
        | "nzry_jieying:choose_target"
        | "qiaoshui:target"
        | "qiaoshui3:add_target"
        | "reqiangxi:choose_targets"
        | "reshanxi:choose_target"
        | "twjiange:choose_target"
        | "twkujian:choose_target" => {
            let enemies = enemies_sorted();
            let selection = if let Some(&t) = enemies.first() {
                vec![ChoiceOption::Player(t)]
            } else if allow_cancel && min == 0 {
                Vec::new()
            } else {
                options.iter().take(min.max(1)).copied().collect()
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "qiaoshui3:remove_target" => {
            let allies = allies_sorted();
            let selection = if let Some(&a) = allies.first() {
                vec![ChoiceOption::Player(a)]
            } else if allow_cancel && min == 0 {
                Vec::new()
            } else {
                options.iter().take(min).copied().collect()
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "qice:choose_targets" | "taoluan:choose_targets" => {
            let enemies = enemies_sorted();
            let selection = if !enemies.is_empty() {
                enemies
                    .into_iter()
                    .take(max.max(1))
                    .map(ChoiceOption::Player)
                    .collect()
            } else if options.contains(&ChoiceOption::Player(seat)) {
                vec![ChoiceOption::Player(seat)]
            } else if allow_cancel && min == 0 {
                Vec::new()
            } else {
                options.iter().take(min.max(1)).copied().collect()
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "xiongyi:choose_targets" => {
            let allies = allies_sorted();
            let selection = if allies.len() >= min {
                allies
                    .into_iter()
                    .take(max.max(min))
                    .map(ChoiceOption::Player)
                    .collect()
            } else {
                options.iter().take(min.max(1)).copied().collect()
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "relianji:choose_targets" => {
            let allies = allies_sorted();
            let selection = if allies.len() >= 2 {
                allies
                    .into_iter()
                    .take(2)
                    .map(ChoiceOption::Player)
                    .collect()
            } else if allow_cancel && min == 0 {
                Vec::new()
            } else {
                options.iter().take(min.max(1)).copied().collect()
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "taoluan:choose_helper" => {
            let allies = allies_sorted();
            let selection = if let Some(&a) = allies.first() {
                vec![ChoiceOption::Player(a)]
            } else {
                let best = options
                    .iter()
                    .filter_map(|o| match *o {
                        ChoiceOption::Player(p) => Some((public_attitude(&input.view, seat, p), *o)),
                        _ => None,
                    })
                    .max_by_key(|(att, _)| *att)
                    .map(|(_, o)| o);
                best.into_iter().collect()
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "xinkuanggu:choose_mode" => {
            let pick = if options.contains(&ChoiceOption::Tag(1)) {
                Some(ChoiceOption::Tag(1))
            } else {
                options.first().copied()
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection: pick.into_iter().collect(),
            })
        }
        "longhun:choose_use_kind"
        | "qice:choose_trick"
        | "taoluan:choose_virtual"
        | "xinanjian:choose_mode" => Some(GameAction::Answer {
            choice_id: id,
            selection: options.first().copied().into_iter().collect(),
        }),
        _ => None,
    }
}
