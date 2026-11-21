//! L10（史诗卡牌）十常侍等扩展询问的 AI 公开信息决策策略。

use super::{extension_card_value, public_attitude};
use crate::choice::ChoiceOption;
use crate::engine::GameAction;
use crate::view::PolicyInput;

#[allow(clippy::too_many_arguments)]
pub(super) fn plan_l10_choice_action(
    catalog: &crate::catalog::Catalog,
    input: &PolicyInput,
    id: u64,
    prompt: &str,
    options: &[ChoiceOption],
    min: usize,
    _max: usize,
    allow_cancel: bool,
) -> Option<GameAction> {
    let seat = input.view.seat;
    let pick_player = |want_ally: bool| -> Option<ChoiceOption> {
        options
            .iter()
            .filter_map(|o| match *o {
                ChoiceOption::Player(t) => Some(t),
                _ => None,
            })
            .max_by_key(|&t| {
                let att = public_attitude(&input.view, seat, t);
                let hp = input
                    .view
                    .players
                    .get(t as usize)
                    .map(|p| p.hp)
                    .unwrap_or(0);
                if want_ally {
                    (att, -hp)
                } else {
                    (-att, -hp)
                }
            })
            .map(ChoiceOption::Player)
    };
    let pick_cards_low = |count: usize| -> Vec<ChoiceOption> {
        let mut cards: Vec<crate::ids::CardId> = options
            .iter()
            .filter_map(|o| match *o {
                ChoiceOption::Card(c) => Some(c),
                _ => None,
            })
            .collect();
        cards.sort_by_key(|&c| extension_card_value(catalog, input, c));
        cards
            .into_iter()
            .take(count)
            .map(ChoiceOption::Card)
            .collect()
    };
    let pick_reguhuo_name = || -> Vec<ChoiceOption> {
        for pref in [4u8, 6, 12, 13, 11, 0, 3, 19] {
            if let Some(opt) = options
                .iter()
                .find(|o| matches!(o, ChoiceOption::Tag(t) if *t == pref))
                .copied()
            {
                return vec![opt];
            }
        }
        options.iter().take(1).copied().collect()
    };
    let best_rangjie_move = || -> Option<(crate::ids::CardId, u8, i32)> {
        let mut best: Option<(crate::ids::CardId, u8, i32)> = None;
        for from in &input.view.players {
            if !from.alive {
                continue;
            }
            let from_att = if from.seat == seat {
                10
            } else {
                public_attitude(&input.view, seat, from.seat)
            };
            for &c in &from.judge {
                let kind = catalog.cards.get(c as usize).map(|d| d.kind);
                if matches!(kind, Some(crate::catalog::CardKind::ShanDian | crate::catalog::CardKind::HongShui)) {
                    continue;
                }
                for to in &input.view.players {
                    if !to.alive || to.seat == from.seat {
                        continue;
                    }
                    if to.judge.iter().any(|&jc| catalog.cards.get(jc as usize).map(|d| d.kind) == kind) {
                        continue;
                    }
                    let to_att = if to.seat == seat {
                        10
                    } else {
                        public_attitude(&input.view, seat, to.seat)
                    };
                    let score = (from_att - to_att) * 10;
                    if score > best.map(|b| b.2).unwrap_or(i32::MIN) {
                        best = Some((c, to.seat, score));
                    }
                }
            }
            for &c in &from.equips {
                let slot = catalog.cards.get(c as usize).and_then(|d| d.kind.equip_slot());
                let Some(slot) = slot else {
                    continue;
                };
                for to in &input.view.players {
                    if !to.alive || to.seat == from.seat {
                        continue;
                    }
                    if to
                        .equips
                        .iter()
                        .any(|&ec| catalog.cards.get(ec as usize).and_then(|d| d.kind.equip_slot()) == Some(slot))
                    {
                        continue;
                    }
                    let to_att = if to.seat == seat {
                        10
                    } else {
                        public_attitude(&input.view, seat, to.seat)
                    };
                    let score = (to_att - from_att) * 10;
                    if score > best.map(|b| b.2).unwrap_or(i32::MIN) {
                        best = Some((c, to.seat, score));
                    }
                }
            }
        }
        best
    };
    match prompt {
        "rangjie_choice" => {
            let beneficial_move = best_rangjie_move().is_some_and(|(_, _, score)| score > 0);
            let want = if beneficial_move && options.contains(&ChoiceOption::Tag(1)) {
                1
            } else {
                0
            };
            let selection = options
                .iter()
                .find(|o| matches!(o, ChoiceOption::Tag(t) if *t == want))
                .or_else(|| options.first())
                .copied()
                .into_iter()
                .collect();
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "rangjie_move_card" => {
            let preferred = best_rangjie_move().and_then(|(c, _, _)| {
                options
                    .iter()
                    .find(|o| matches!(o, ChoiceOption::Card(id) if *id == c))
                    .copied()
            });
            let selection = preferred
                .or_else(|| options.iter().find(|o| matches!(o, ChoiceOption::Card(_))).copied())
                .into_iter()
                .collect();
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "rangjie_move_target" => {
            let preferred = best_rangjie_move().and_then(|(_, to, _)| {
                options
                    .iter()
                    .find(|o| matches!(o, ChoiceOption::Player(p) if *p == to))
                    .copied()
            });
            let selection = preferred
                .or_else(|| pick_player(true))
                .into_iter()
                .collect();
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "reguhuo_card" => Some(GameAction::Answer {
            choice_id: id,
            selection: pick_cards_low(1),
        }),
        "reguhuo_name" | "scstaoluan_name" => Some(GameAction::Answer {
            choice_id: id,
            selection: pick_reguhuo_name(),
        }),
        "scsniqu_target"
        | "scsanruo_target"
        | "reguhuo_jiedao_victim"
        | "scstaoluan_jiedao_victim" => {
            let best = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Player(t) => Some(t),
                    _ => None,
                })
                .filter(|&t| public_attitude(&input.view, seat, t) < 0)
                .max_by_key(|&t| {
                    let hp = input
                        .view
                        .players
                        .get(t as usize)
                        .map(|p| p.hp)
                        .unwrap_or(0);
                    (-public_attitude(&input.view, seat, t), -hp)
                });
            let selection = match best {
                Some(t) => vec![ChoiceOption::Player(t)],
                None if allow_cancel => Vec::new(),
                None => pick_player(false).into_iter().collect(),
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "scstaoluan_card" | "scsanruo_card" => {
            let selection = if options.is_empty() && allow_cancel {
                Vec::new()
            } else {
                pick_cards_low(1)
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "scstaoluan_targets" | "reguhuo_targets" => {
            let mut players: Vec<u8> = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Player(t) => Some(t),
                    _ => None,
                })
                .collect();
            players.sort_by_key(|&t| {
                if t == seat {
                    i32::MAX
                } else {
                    public_attitude(&input.view, seat, t)
                }
            });
            let take_n = min.max(1);
            let selection = players
                .into_iter()
                .take(take_n)
                .map(ChoiceOption::Player)
                .collect();
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "scschiyan_pick" => {
            let selection = options
                .iter()
                .find(|o| matches!(o, ChoiceOption::Card(_)))
                .copied()
                .into_iter()
                .collect();
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "scschihe_confirm" | "scspicai_continue" => {
            let selection = options
                .iter()
                .find(|o| matches!(o, ChoiceOption::Confirm))
                .copied()
                .into_iter()
                .collect();
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "scspicai_target" | "scsxiaolu_target" => Some(GameAction::Answer {
            choice_id: id,
            selection: pick_player(true).into_iter().collect(),
        }),
        "scsyaozhuo_target" | "scskuiji_target" => {
            let best = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Player(t) => Some(t),
                    _ => None,
                })
                .filter(|&t| public_attitude(&input.view, seat, t) < 0)
                .max_by_key(|&t| {
                    let hc = input
                        .view
                        .players
                        .get(t as usize)
                        .map(|p| p.hand_count as i32)
                        .unwrap_or(0);
                    (-public_attitude(&input.view, seat, t), hc)
                });
            let selection = match best {
                Some(t) => vec![ChoiceOption::Player(t)],
                None if allow_cancel => Vec::new(),
                None => pick_player(false).into_iter().collect(),
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "scsyaozhuo_pindian_own" | "scsyaozhuo_pindian_target" => {
            let best = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Card(c) => Some(c),
                    _ => None,
                })
                .max_by_key(|&c| {
                    // 仅对观察者自己已知的手牌读取点数，严禁窥探未公开牌。
                    if input.own_hand_kinds.iter().any(|(id, _)| *id == c) {
                        catalog.cards[c as usize].rank
                    } else {
                        0
                    }
                });
            Some(GameAction::Answer {
                choice_id: id,
                selection: best.map(ChoiceOption::Card).into_iter().collect(),
            })
        }
        "scsyaozhuo_discard" | "scsxiaolu_cards" => Some(GameAction::Answer {
            choice_id: id,
            selection: pick_cards_low(min.max(2)),
        }),
        "scsxiaolu_mode" => {
            let has_ally = input.view.players.iter().any(|p| {
                p.alive && p.seat != seat && public_attitude(&input.view, seat, p.seat) > 0
            });
            let prefer = if has_ally { 0 } else { 1 };
            let chosen = options
                .iter()
                .find(|o| matches!(o, ChoiceOption::Tag(t) if *t == prefer))
                .or_else(|| options.first())
                .copied();
            Some(GameAction::Answer {
                choice_id: id,
                selection: chosen.into_iter().collect(),
            })
        }
        "scskuiji_cards" => {
            // 每花色仅选 1 张，优先弃置非己方（对手已公开的）手牌。
            let mut by_suit = [None::<(i32, crate::ids::CardId)>; 4];
            for opt in options {
                let ChoiceOption::Card(c) = *opt else {
                    continue;
                };
                let suit_idx = catalog
                    .cards
                    .get(c as usize)
                    .map(|d| match d.suit {
                        crate::catalog::Suit::Spade => 0,
                        crate::catalog::Suit::Heart => 1,
                        crate::catalog::Suit::Club => 2,
                        crate::catalog::Suit::Diamond => 3,
                    })
                    .unwrap_or(0);
                let is_own = input.own_hand_kinds.iter().any(|(id, _)| *id == c);
                let val = extension_card_value(catalog, input, c) as i32;
                let score = if is_own { -(100 + val) } else { 1000 + val };
                if by_suit[suit_idx].is_none_or(|(best_s, _)| score > best_s) {
                    by_suit[suit_idx] = Some((score, c));
                }
            }
            let picked: Vec<ChoiceOption> = by_suit
                .into_iter()
                .flatten()
                .map(|(_, c)| ChoiceOption::Card(c))
                .collect();
            let selection = if picked.len() == 4 {
                picked
            } else if allow_cancel {
                Vec::new()
            } else {
                picked
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "l10_scschiyan_scschihe_order" => {
            let selection = options
                .iter()
                .find(|o| matches!(o, ChoiceOption::Tag(0)))
                .or_else(|| options.first())
                .copied()
                .into_iter()
                .collect();
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "scsanruo_steal_confirm" => {
            let selection = options
                .iter()
                .find(|o| matches!(o, ChoiceOption::Confirm))
                .copied()
                .into_iter()
                .collect();
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "scsanruo_steal_target" => {
            let best = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Player(t) => Some(t),
                    _ => None,
                })
                .filter(|&t| public_attitude(&input.view, seat, t) < 0)
                .max_by_key(|&t| -public_attitude(&input.view, seat, t));
            let selection = match best {
                Some(t) => vec![ChoiceOption::Player(t)],
                None if allow_cancel => Vec::new(),
                None => pick_player(false).into_iter().collect(),
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "scsanruo_steal_card" => {
            let best = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Card(c) => Some(c),
                    _ => None,
                })
                .max_by_key(|&c| {
                    let is_public = input.view.players.iter().any(|p| p.equips.contains(&c));
                    if is_public {
                        extension_card_value(catalog, input, c)
                    } else {
                        1
                    }
                });
            Some(GameAction::Answer {
                choice_id: id,
                selection: best.map(ChoiceOption::Card).into_iter().collect(),
            })
        }
        _ => None,
    }
}
