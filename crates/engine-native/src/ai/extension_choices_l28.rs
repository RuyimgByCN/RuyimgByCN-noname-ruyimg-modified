//! L28（雷霆万钧）及神工 18 件专属装备询问的 AI 公开信息决策策略。

use super::{extension_card_value, public_attitude};
use crate::choice::ChoiceOption;
use crate::engine::GameAction;
use crate::view::PolicyInput;

#[allow(clippy::too_many_arguments)]
pub(super) fn plan_l28_choice_action(
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
    match prompt {
        "th_zhengjing:choose_count" => {
            let best = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Tag(n) => Some(n),
                    _ => None,
                })
                .max()
                .map(ChoiceOption::Tag);
            Some(GameAction::Answer {
                choice_id: id,
                selection: best.into_iter().collect(),
            })
        }
        "th_zhengjing:choose_target" => {
            let best = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Player(t) => Some(t),
                    _ => None,
                })
                .max_by_key(|&t| {
                    let has_yingshi = input
                        .view
                        .players
                        .get(t as usize)
                        .is_some_and(|p| p.skills.contains(&"xinfu_pdgyingshi"));
                    let score = if has_yingshi {
                        0i64
                    } else {
                        -(public_attitude(&input.view, seat, t) as i64)
                    };
                    let hp = input
                        .view
                        .players
                        .get(t as usize)
                        .map(|p| p.hp as i64)
                        .unwrap_or(0);
                    (score, -hp)
                });
            let selection = match best {
                Some(t) => vec![ChoiceOption::Player(t)],
                None => options.iter().take(1).copied().collect(),
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "th_zhengjing:choose_cards" | "th_caiyi:discard_cards" => {
            let mut cards: Vec<crate::ids::CardId> = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Card(c) => Some(c),
                    _ => None,
                })
                .collect();
            cards.sort_by_key(|&c| extension_card_value(catalog, input, c));
            let selection: Vec<ChoiceOption> = cards
                .into_iter()
                .take(min.max(1))
                .map(ChoiceOption::Card)
                .collect();
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "th_tianshu:choose_target" => {
            let best = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Player(t) => Some(t),
                    _ => None,
                })
                .filter(|&t| public_attitude(&input.view, seat, t) > 0)
                .max_by_key(|&t| {
                    let att = public_attitude(&input.view, seat, t) as i64;
                    let hp = input
                        .view
                        .players
                        .get(t as usize)
                        .map(|p| p.hp as i64)
                        .unwrap_or(0);
                    att * 10 - hp
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
        "th_tianshu:discard_card" => {
            let best = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Card(c) => Some(c),
                    _ => None,
                })
                .min_by_key(|&c| extension_card_value(catalog, input, c));
            let selection = match best {
                Some(c) if extension_card_value(catalog, input, c) < 75 || !allow_cancel => {
                    vec![ChoiceOption::Card(c)]
                }
                _ => Vec::new(),
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "th_buxu:discard_cards" => {
            let mut cards: Vec<crate::ids::CardId> = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Card(c) => Some(c),
                    _ => None,
                })
                .collect();
            cards.sort_by_key(|&c| extension_card_value(catalog, input, c));
            let selection = if cards.len() >= min && min > 0 {
                cards.into_iter().take(min).map(ChoiceOption::Card).collect()
            } else {
                Vec::new()
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "th_buxu:choose_chamber" => {
            let selection = options
                .iter()
                .find(|o| matches!(o, ChoiceOption::Tag(_)))
                .copied()
                .into_iter()
                .collect();
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "th_yeyan:choose_mode" => {
            let has_enemy = input
                .view
                .players
                .iter()
                .any(|p| p.alive && public_attitude(&input.view, seat, p.seat) < 0);
            let selection = if !has_enemy && allow_cancel {
                Vec::new()
            } else {
                options
                    .iter()
                    .find(|o| matches!(o, ChoiceOption::Tag(0)))
                    .or_else(|| options.first())
                    .copied()
                    .into_iter()
                    .collect()
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "th_yeyan:choose_targets" => {
            let mut enemies: Vec<u8> = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Player(t) => Some(t),
                    _ => None,
                })
                .filter(|&t| public_attitude(&input.view, seat, t) < 0)
                .collect();
            enemies.sort_by_key(|&t| {
                let hp = input
                    .view
                    .players
                    .get(t as usize)
                    .map(|p| p.hp as i64)
                    .unwrap_or(0);
                let att = public_attitude(&input.view, seat, t) as i64;
                (att, hp)
            });
            let selection = if !enemies.is_empty() {
                enemies
                    .into_iter()
                    .take(max.max(1))
                    .map(ChoiceOption::Player)
                    .collect()
            } else if allow_cancel {
                Vec::new()
            } else {
                options.iter().take(min.max(1)).copied().collect()
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "th_yeyan:choose_damage" => {
            let selection = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Tag(n) => Some(n),
                    _ => None,
                })
                .max()
                .map(ChoiceOption::Tag)
                .into_iter()
                .collect();
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "th_yeyan:discard_cards" => {
            let mut cards: Vec<crate::ids::CardId> = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Card(c) => Some(c),
                    _ => None,
                })
                .collect();
            cards.sort_by_key(|&c| extension_card_value(catalog, input, c));
            let mut distinct_four = Vec::new();
            let mut seen_suits = Vec::new();
            for &cid in &cards {
                if let Some(def) = catalog.cards.get(cid as usize) {
                    if !seen_suits.contains(&def.suit) {
                        seen_suits.push(def.suit);
                        distinct_four.push(ChoiceOption::Card(cid));
                        if distinct_four.len() == 4 {
                            break;
                        }
                    }
                }
            }
            let selection = if distinct_four.len() == 4 {
                distinct_four
            } else if allow_cancel {
                Vec::new()
            } else {
                cards.into_iter().take(min.max(4)).map(ChoiceOption::Card).collect()
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "th_shengong:assist" => {
            let actor = input.view.turn;
            let att = public_attitude(&input.view, seat, actor);
            let selection = if att > 0 && options.contains(&ChoiceOption::Tag(0)) {
                vec![ChoiceOption::Tag(0)]
            } else if att < 0 && options.contains(&ChoiceOption::Tag(1)) {
                vec![ChoiceOption::Tag(1)]
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
        "th_shengong:choose_card" => {
            let best = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Card(c) => Some(c),
                    _ => None,
                })
                .max_by_key(|&c| extension_card_value(catalog, input, c))
                .map(ChoiceOption::Card);
            let selection = match best {
                Some(c) => vec![c],
                None => options.iter().take(1).copied().collect(),
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "pro_qinyin:choose_mode" => {
            let self_max = input
                .view
                .players
                .get(seat as usize)
                .map(|p| p.max_hp)
                .unwrap_or(0);
            let selection = if options.contains(&ChoiceOption::Tag(2)) && self_max > 3 {
                vec![ChoiceOption::Tag(2)]
            } else {
                let (mut recover, mut lose) = (0i32, 0i32);
                for p in &input.view.players {
                    if !p.alive {
                        continue;
                    }
                    let att = public_attitude(&input.view, seat, p.seat);
                    if p.hp < p.max_hp as i32 {
                        if att > 0 {
                            if p.hp < 2 {
                                lose -= 1;
                                recover += 1;
                            }
                            lose -= 2;
                            recover += 2;
                        } else if att < 0 {
                            if p.hp < 2 {
                                lose += 1;
                                recover -= 1;
                            }
                            lose += 2;
                            recover -= 2;
                        }
                    } else if att > 0 {
                        lose -= 2;
                    } else if att < 0 {
                        lose += 2;
                    }
                }
                if lose > recover && lose > 0 && options.contains(&ChoiceOption::Tag(1)) {
                    vec![ChoiceOption::Tag(1)]
                } else if lose < recover && recover > 0 && options.contains(&ChoiceOption::Tag(0)) {
                    vec![ChoiceOption::Tag(0)]
                } else if allow_cancel {
                    Vec::new()
                } else {
                    options.iter().take(1).copied().collect()
                }
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "th_tiansuan:choose_lot" => {
            let allies = input
                .view
                .players
                .iter()
                .filter(|p| p.alive && public_attitude(&input.view, seat, p.seat) > 0)
                .count();
            let enemies = input
                .view
                .players
                .iter()
                .filter(|p| p.alive && public_attitude(&input.view, seat, p.seat) < 0)
                .count();
            let pref = if allies > enemies {
                ChoiceOption::Tag(0)
            } else {
                ChoiceOption::Tag(4)
            };
            let selection = if options.contains(&pref) {
                vec![pref]
            } else {
                options.iter().take(1).copied().collect()
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "th_tiansuan:choose_target" | "shufazijinguan:choose_target" | "zhaogujing:choose_target" => {
            let best = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Player(t) => Some(t),
                    _ => None,
                })
                .filter(|&t| prompt != "shufazijinguan:choose_target" || public_attitude(&input.view, seat, t) < 0)
                .max_by_key(|&t| {
                    let att = -(public_attitude(&input.view, seat, t) as i64);
                    let hp = input
                        .view
                        .players
                        .get(t as usize)
                        .map(|p| p.hp as i64)
                        .unwrap_or(0);
                    (att, -hp)
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
        "wushuangfangtianji:choose_mode" => {
            let selection = if options.contains(&ChoiceOption::Tag(1)) {
                vec![ChoiceOption::Tag(1)]
            } else if options.contains(&ChoiceOption::Tag(0)) {
                vec![ChoiceOption::Tag(0)]
            } else {
                options.iter().take(1).copied().collect()
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "wushuangfangtianji:discard_card" => {
            let selection = options.iter().take(1).copied().collect();
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "bintieshuangji:confirm" => {
            let hp = input.view.players.get(seat as usize).map(|p| p.hp).unwrap_or(0);
            let selection = if hp >= 2 && options.contains(&ChoiceOption::Confirm) {
                vec![ChoiceOption::Confirm]
            } else {
                Vec::new()
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "wuxinghelingshan:choose_nature" => Some(GameAction::Answer {
            choice_id: id,
            selection: Vec::new(),
        }),
        "linglongshimandai:confirm" | "huxinjing:confirm" => Some(GameAction::Answer {
            choice_id: id,
            selection: if options.contains(&ChoiceOption::Confirm) {
                vec![ChoiceOption::Confirm]
            } else {
                Vec::new()
            },
        }),
        "tianjitu:discard_card" | "taigongyinfu:recast_card" => {
            let best = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Card(c) => Some(c),
                    _ => None,
                })
                .min_by_key(|&c| extension_card_value(catalog, input, c));
            let selection = match best {
                Some(c) if prompt != "taigongyinfu:recast_card" || extension_card_value(catalog, input, c) < 55 => {
                    vec![ChoiceOption::Card(c)]
                }
                _ if allow_cancel => Vec::new(),
                _ => options.iter().take(1).copied().collect(),
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "taigongyinfu:choose_target" => {
            let best = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Player(t) => Some(t),
                    _ => None,
                })
                .filter(|&t| {
                    let att = public_attitude(&input.view, seat, t);
                    let linked = input
                        .view
                        .players
                        .get(t as usize)
                        .is_some_and(|p| p.linked);
                    (!linked && att < 0) || (linked && att > 0)
                })
                .max_by_key(|&t| public_attitude(&input.view, seat, t).abs());
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
        "zhaogujing:choose_card" => {
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
        _ => None,
    }
}
