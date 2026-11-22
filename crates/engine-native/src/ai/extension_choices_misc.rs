//! U08/L27 等扩展挂起询问的公开信息集启发式策略（从 `extension_choices` 拆出以维持模块行数上限）。

use super::*;
use crate::choice::ChoiceOption;

pub(super) fn plan_misc_choice_action(
    catalog: &crate::catalog::Catalog,
    input: &crate::view::PolicyInput,
    id: u64,
    prompt: &str,
    options: &[ChoiceOption],
    min: usize,
    _max: usize,
    allow_cancel: bool,
) -> Option<GameAction> {
    let seat = input.view.seat;
    match prompt {
        "bolehuiyan:predict" => {
            let attacker = input.view.players.get(input.view.turn as usize);
            let direct_hit_attacker = attacker.is_some_and(|a| {
                a.skills.iter().any(|&s| {
                    matches!(
                        s,
                        "无双" | "wushuang" | "烈弓" | "liegong" | "铁骑" | "tieji" | "juji" | "retieji"
                    )
                })
            });
            let likely_hit = direct_hit_attacker
                || input.view.players.iter().any(|p| {
                    p.alive
                        && p.seat != seat
                        && p.seat != input.view.turn
                        && p.hand_count > 0
                        && (p.skills.iter().any(|&s| matches!(s, "yijue2" | "shanguang2"))
                            || (p.hand_count < 2 && p.equips.is_empty()))
                });
            let tag = if likely_hit { 1 } else { 2 };
            let selection = options
                .iter()
                .find(|o| matches!(o, ChoiceOption::Tag(t) if *t == tag))
                .copied()
                .into_iter()
                .collect();
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "cike:confirm" => Some(GameAction::Answer {
            choice_id: id,
            selection: vec![ChoiceOption::Confirm],
        }),
        "cike:discard_card" | "wange:choose_card" => {
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
        "reqimou:confirm" => {
            let me = input.view.players.get(seat as usize);
            let confirm = me.is_some_and(|p| p.hp >= 3) && {
                let my_hp = me.map(|p| p.hp).unwrap_or(0);
                let save_bonus = if input.own_hand_kinds.iter().any(|&(_, k)| {
                    matches!(k, crate::catalog::CardKind::Tao | crate::catalog::CardKind::Jiu)
                }) {
                    1
                } else {
                    0
                };
                let mindist = my_hp + save_bonus;
                let minus_horse = me.is_some_and(|p| {
                    p.equips.iter().any(|&c| {
                        catalog
                            .cards
                            .get(c as usize)
                            .is_some_and(|d| d.kind == crate::catalog::CardKind::HorseMinus)
                    })
                });
                let alive_seats: Vec<u8> = input
                    .view
                    .players
                    .iter()
                    .filter(|p| p.alive)
                    .map(|p| p.seat)
                    .collect();
                let my_pos = alive_seats.iter().position(|&s| s == seat);
                alive_seats.iter().enumerate().any(|(idx, &t)| {
                    if t == seat || public_attitude(&input.view, seat, t) >= 0 {
                        return false;
                    }
                    let Some(mpos) = my_pos else {
                        return false;
                    };
                    let n = alive_seats.len();
                    let cw = (idx + n - mpos) % n;
                    let ccw = (mpos + n - idx) % n;
                    let mut dist = cw.min(ccw) as i32;
                    if minus_horse {
                        dist -= 1;
                    }
                    if input.view.players.get(t as usize).is_some_and(|tp| {
                        tp.equips.iter().any(|&c| {
                            catalog
                                .cards
                                .get(c as usize)
                                .is_some_and(|d| d.kind == crate::catalog::CardKind::HorsePlus)
                        })
                    }) {
                        dist += 1;
                    }
                    dist.max(1) <= mindist
                })
            };
            let selection = if confirm {
                vec![ChoiceOption::Confirm]
            } else if options.contains(&ChoiceOption::Pass) {
                vec![ChoiceOption::Pass]
            } else {
                Vec::new()
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "dcxiaoxi:gain_card" => {
            let need = min.max(1);
            let mut ranked: Vec<(i64, ChoiceOption)> = options
                .iter()
                .filter_map(|&opt| match opt {
                    ChoiceOption::Card(c) => Some((100 + extension_card_value(catalog, input, c), opt)),
                    ChoiceOption::HiddenCard(_) => Some((50, opt)),
                    _ => None,
                })
                .collect();
            ranked.sort_by(|a, b| b.0.cmp(&a.0));
            let selection: Vec<ChoiceOption> =
                ranked.into_iter().take(need).map(|(_, o)| o).collect();
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "twzhouzu:target_discard" => {
            let need = min.max(1);
            let mut ranked: Vec<(i64, ChoiceOption)> = options
                .iter()
                .filter_map(|&opt| match opt {
                    ChoiceOption::Card(c) => Some((extension_card_value(catalog, input, c), opt)),
                    _ => None,
                })
                .collect();
            ranked.sort_by_key(|(v, _)| *v);
            let selection: Vec<ChoiceOption> =
                ranked.into_iter().take(need).map(|(_, o)| o).collect();
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "bolan_g:choose_target" => {
            let best = options
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Player(t) => Some(*t),
                    _ => None,
                })
                .filter(|&t| !allow_cancel || public_attitude(&input.view, seat, t) >= 0)
                .max_by_key(|&t| public_attitude(&input.view, seat, t));
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
        "sbqingzheng:choose_suits" => {
            let need = min.max(1);
            let selection = if options.len() < need && allow_cancel {
                Vec::new()
            } else {
                options.iter().take(need).copied().collect()
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "bolan:choose_skill" => {
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
        _ => None,
    }
}
