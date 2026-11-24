//! Batch Q4 (`jiwu` 派生技：`qiangxix` / `retieji` / `rexuanfeng`) 的 AI 公开信息决策。

use super::{extension_card_value, public_attitude};
use crate::catalog::EquipSlot;
use crate::choice::ChoiceOption;
use crate::engine::GameAction;
use crate::view::PolicyInput;

#[allow(clippy::too_many_arguments)]
pub(super) fn plan_q4_choice_action(
    catalog: &crate::catalog::Catalog,
    input: &PolicyInput,
    id: u64,
    prompt: &str,
    options: &[ChoiceOption],
    min: usize,
    allow_cancel: bool,
) -> Option<GameAction> {
    let seat = input.view.seat;
    let me = input.view.players.get(seat as usize);
    let pick_enemy = || -> Option<u8> {
        options
            .iter()
            .filter_map(|o| match *o {
                ChoiceOption::Player(t) => Some(t),
                _ => None,
            })
            .filter(|&t| public_attitude(&input.view, seat, t) < 0)
            .min_by_key(|&t| {
                let p = input.view.players.get(t as usize);
                (
                    p.map(|x| x.hp).unwrap_or(99),
                    p.map(|x| x.hand_count).unwrap_or(99),
                    public_attitude(&input.view, seat, t),
                )
            })
    };
    match prompt {
        "qiangxix:target" => {
            let has_weapon = input
                .own_hand_kinds
                .iter()
                .any(|&(_, k)| k.equip_slot() == Some(EquipSlot::Weapon))
                || me.is_some_and(|p| {
                    p.equips.iter().any(|&c| {
                        catalog
                            .cards
                            .get(c as usize)
                            .and_then(|d| d.kind.equip_slot())
                            == Some(EquipSlot::Weapon)
                    })
                });
            if me.is_some_and(|p| p.hp <= 1) && !has_weapon && allow_cancel {
                return Some(GameAction::Answer {
                    choice_id: id,
                    selection: Vec::new(),
                });
            }
            let selection = match pick_enemy() {
                Some(t) => vec![ChoiceOption::Player(t)],
                None if allow_cancel => Vec::new(),
                None => options.iter().take(1).copied().collect(),
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "qiangxix:card" => {
            let best = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Card(c) => Some(c),
                    _ => None,
                })
                .min_by_key(|&c| {
                    let equipped_penalty = if me.is_some_and(|p| p.equips.contains(&c)) {
                        25
                    } else {
                        0
                    };
                    extension_card_value(catalog, input, c) + equipped_penalty
                });
            let selection = match best {
                Some(c) => vec![ChoiceOption::Card(c)],
                None if min == 0 || allow_cancel => Vec::new(),
                None => Vec::new(),
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "retieji:confirm" => {
            let explicit_target = options.iter().find_map(|o| match *o {
                ChoiceOption::Player(t) => Some(t),
                _ => None,
            });
            let confirm = match explicit_target {
                Some(t) => public_attitude(&input.view, seat, t) < 0,
                None => input
                    .view
                    .players
                    .iter()
                    .any(|p| p.alive && p.seat != seat && public_attitude(&input.view, seat, p.seat) < 0),
            };
            let selection = if confirm && options.contains(&ChoiceOption::Confirm) {
                vec![ChoiceOption::Confirm]
            } else if options.contains(&ChoiceOption::Pass) {
                vec![ChoiceOption::Pass]
            } else if allow_cancel {
                Vec::new()
            } else {
                options.iter().take(1).copied().collect()
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "retieji:give" | "retieji:discard" => {
            let best = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Card(c) => Some(c),
                    _ => None,
                })
                .min_by_key(|&c| extension_card_value(catalog, input, c));
            let selection = match best {
                Some(c) => vec![ChoiceOption::Card(c)],
                None if allow_cancel => Vec::new(),
                None => Vec::new(),
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "rexuanfeng:mode" => {
            let good_equip_move = options.contains(&ChoiceOption::Tag(1))
                && input.view.players.iter().any(|from| {
                    from.alive
                        && from.seat != seat
                        && public_attitude(&input.view, seat, from.seat) < 0
                        && from.equips.iter().any(|&eq| {
                            let Some(slot) = catalog
                                .cards
                                .get(eq as usize)
                                .and_then(|d| d.kind.equip_slot())
                            else {
                                return false;
                            };
                            input.view.players.iter().any(|to| {
                                to.alive
                                    && to.seat != seat
                                    && to.seat != from.seat
                                    && public_attitude(&input.view, seat, to.seat) >= 0
                                    && !to.equips.iter().any(|&oeq| {
                                        catalog
                                            .cards
                                            .get(oeq as usize)
                                            .and_then(|d| d.kind.equip_slot())
                                            == Some(slot)
                                    })
                            })
                        })
                });
            let enemy_has_cards = options.contains(&ChoiceOption::Tag(0))
                && input.view.players.iter().any(|p| {
                    p.alive
                        && p.seat != seat
                        && public_attitude(&input.view, seat, p.seat) < 0
                        && (p.hand_count > 0 || !p.equips.is_empty())
                });
            let selection = if good_equip_move {
                vec![ChoiceOption::Tag(1)]
            } else if enemy_has_cards {
                vec![ChoiceOption::Tag(0)]
            } else if allow_cancel {
                Vec::new()
            } else {
                options.iter().take(1).copied().collect()
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "rexuanfeng:move_from" => {
            let best = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Player(t) => Some(t),
                    _ => None,
                })
                .filter(|&t| public_attitude(&input.view, seat, t) < 0)
                .max_by_key(|&t| {
                    input
                        .view
                        .players
                        .get(t as usize)
                        .map(|p| p.equips.len())
                        .unwrap_or(0)
                });
            let selection = match best {
                Some(t) => vec![ChoiceOption::Player(t)],
                None if allow_cancel => Vec::new(),
                None => options.iter().take(1).copied().collect(),
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "rexuanfeng:move_card" => {
            let best = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Card(c) => Some(c),
                    _ => None,
                })
                .max_by_key(|&c| extension_card_value(catalog, input, c));
            let selection = match best {
                Some(c) => vec![ChoiceOption::Card(c)],
                None if allow_cancel => Vec::new(),
                None => options.iter().take(1).copied().collect(),
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "rexuanfeng:move_to" => {
            let best = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Player(t) => Some(t),
                    _ => None,
                })
                .max_by_key(|&t| public_attitude(&input.view, seat, t));
            let selection = match best {
                Some(t) if public_attitude(&input.view, seat, t) >= 0 || !allow_cancel => {
                    vec![ChoiceOption::Player(t)]
                }
                _ if allow_cancel => Vec::new(),
                _ => options.iter().take(1).copied().collect(),
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        _ => None,
    }
}
