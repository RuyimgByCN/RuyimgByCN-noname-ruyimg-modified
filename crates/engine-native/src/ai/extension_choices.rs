//! 扩展包挂起询问（`OpenChoice`）的公开信息集启发式策略（`plan_extension_choice_action`）。

use super::*;
use crate::choice::ChoiceOption;
use crate::view::ChoiceView;

fn public_attitude(view: &crate::view::PlayerView, seat: u8, target: u8) -> i32 {
    if target == seat {
        return 10;
    }
    let my_role = view.players.get(seat as usize).and_then(|p| p.role);
    let target_role = view.players.get(target as usize).and_then(|p| p.role);
    match my_role {
        Some(crate::state::Role::Lord) | Some(crate::state::Role::Loyalist) => match target_role {
            Some(crate::state::Role::Lord) | Some(crate::state::Role::Loyalist) => 6,
            _ => -5,
        },
        Some(crate::state::Role::Rebel) => match target_role {
            Some(crate::state::Role::Lord) => -6,
            Some(crate::state::Role::Rebel) => 6,
            _ => -4,
        },
        _ => -5,
    }
}

fn extension_card_value(input: &crate::view::PolicyInput, card: crate::ids::CardId) -> i64 {
    let kind = input
        .own_hand_kinds
        .iter()
        .find(|(c, _)| *c == card)
        .map(|(_, k)| *k);
    match kind {
        Some(crate::catalog::CardKind::Tao) => 80,
        Some(crate::catalog::CardKind::Jiu) | Some(crate::catalog::CardKind::Wuzhong) => 70,
        Some(crate::catalog::CardKind::Shan) => 40,
        Some(crate::catalog::CardKind::Sha) => 35,
        Some(_) => 45,
        None => 30,
    }
}

pub(super) fn plan_extension_choice_action(input: &crate::view::PolicyInput) -> Option<GameAction> {
    let seat = input.view.seat;
    let Some(ChoiceView::Own {
        id,
        prompt,
        ref options,
        allow_cancel,
        ..
    }) = input.choice
    else {
        return None;
    };
    match prompt {
        "boss_chuanyun_target" => {
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
        "boss_tianyun_target" => {
            let my_hp = input
                .view
                .players
                .get(seat as usize)
                .map(|p| p.hp)
                .unwrap_or(0);
            if my_hp <= 1 && allow_cancel {
                return Some(GameAction::Answer {
                    choice_id: id,
                    selection: Vec::new(),
                });
            }
            let best = options
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Player(t) => Some(*t),
                    _ => None,
                })
                .filter(|&t| public_attitude(&input.view, seat, t) <= -3)
                .max_by_key(|&t| {
                    let att = (-public_attitude(&input.view, seat, t)) as i64;
                    let equips = input
                        .view
                        .players
                        .get(t as usize)
                        .map(|p| p.equips.len() as i64)
                        .unwrap_or(0);
                    att * 10 + equips * 5
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
        "dddxiaxing_card" => {
            let beneficiary = input
                .view
                .players
                .iter()
                .find(|p| p.alive && p.hp <= 0)
                .map(|p| p.seat)
                .unwrap_or(input.view.turn);
            if public_attitude(&input.view, seat, beneficiary) <= 0 && allow_cancel {
                return Some(GameAction::Answer {
                    choice_id: id,
                    selection: Vec::new(),
                });
            }
            let best = options
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Card(c) => Some(*c),
                    _ => None,
                })
                .min_by_key(|&c| extension_card_value(input, c));
            let selection = match best {
                Some(c) => vec![ChoiceOption::Card(c)],
                None => Vec::new(),
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        // U06 欢乐卡牌·猴子（skill.js:20-22）：仅当使用【桃】的角色为敌方（public_attitude <= 0）时发动夺桃
        "huanle_monkey_confirm" => {
            let peach_user = input
                .view
                .players
                .iter()
                .find(|p| p.alive && p.hp <= 0)
                .map(|p| p.seat)
                .unwrap_or(input.view.turn);
            let confirm = public_attitude(&input.view, seat, peach_user) <= 0;
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
        "trigger_confirm" => match input.choice.as_ref().and_then(|c| match c {
            ChoiceView::Own { source_skill, .. } => *source_skill,
            _ => None,
        }) {
            Some("huanle:monkey" | "monkey") => {
                let peach_user = input
                    .view
                    .players
                    .iter()
                    .find(|p| p.alive && p.hp <= 0)
                    .map(|p| p.seat)
                    .unwrap_or(input.view.turn);
                let confirm = public_attitude(&input.view, seat, peach_user) <= 0;
                let selection = if confirm {
                    vec![ChoiceOption::Confirm]
                } else {
                    vec![ChoiceOption::Pass]
                };
                Some(GameAction::Answer {
                    choice_id: id,
                    selection,
                })
            }
            Some("huanle_jiuwei" | "jiuwei") => Some(GameAction::Answer {
                choice_id: id,
                selection: vec![ChoiceOption::Confirm],
            }),
            _ => None,
        },
        // U06 欢乐卡牌·手里剑（card.js:47-59）：背面朝上且装备旋涡面具时宁可受 1 伤保留面具，否则弃置最低价值装备
        "host_target_discard_equip" => {
            let me = input.view.players.get(seat as usize);
            let keep_mianju_when_turned = me.is_some_and(|p| {
                p.turned_over
                    && p.skills
                        .iter()
                        .any(|&s| s == "huanle:mianju" || s == "mianju")
            });
            if keep_mianju_when_turned {
                let selection = if options.contains(&ChoiceOption::Pass) {
                    vec![ChoiceOption::Pass]
                } else {
                    Vec::new()
                };
                return Some(GameAction::Answer {
                    choice_id: id,
                    selection,
                });
            }
            let best = options
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Card(c) => Some(*c),
                    _ => None,
                })
                .min_by_key(|&c| extension_card_value(input, c));
            let selection = match best {
                Some(c) => vec![ChoiceOption::Card(c)],
                None if options.contains(&ChoiceOption::Pass) => vec![ChoiceOption::Pass],
                None => Vec::new(),
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        // U06 欢乐卡牌·苦无（skill.js:66-69）/ L06 五行生克（wuxing/extension.js:77）：强制弃置 1 张价值最低的 he 牌
        "kuwu_discard" | "shengke_discard" => {
            let best = options
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Card(c) => Some(*c),
                    _ => None,
                })
                .min_by_key(|&c| extension_card_value(input, c));
            let selection = match best {
                Some(c) => vec![ChoiceOption::Card(c)],
                None => options.iter().take(1).copied().collect(),
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        // U06 欢乐卡牌·写轮眼（skill.js:80-112）：优先选择技能数最多的存活其他角色与首个可窃技能
        "huanle_xuelunyang_target" => {
            let best = options
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Player(t) => Some(*t),
                    _ => None,
                })
                .max_by_key(|&t| {
                    let skill_n = input
                        .view
                        .players
                        .get(t as usize)
                        .map(|p| p.skills.len() as i64)
                        .unwrap_or(0);
                    let enemy_bonus = if public_attitude(&input.view, seat, t) < 0 {
                        1i64
                    } else {
                        0
                    };
                    skill_n * 10 + enemy_bonus
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
        "huanle_xuelunyang_skill" => {
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
        "wudian_duoqi_card" => {
            let cur = input.view.turn;
            if public_attitude(&input.view, seat, cur) >= 0 && allow_cancel {
                return Some(GameAction::Answer {
                    choice_id: id,
                    selection: Vec::new(),
                });
            }
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
        "wudian_youzhan_discard" | "wudian_choudu_card" | "wudian_diaobing_swap" => {
            let best = options
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Card(c) => Some(*c),
                    _ => None,
                })
                .min_by_key(|&c| extension_card_value(input, c));
            let selection = match best {
                Some(c) => vec![ChoiceOption::Card(c)],
                None => Vec::new(),
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "wudian_choudu_target" => {
            let best = options
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Player(t) => Some(*t),
                    _ => None,
                })
                .max_by_key(|&t| {
                    let att = public_attitude(&input.view, seat, t) as i64;
                    let self_bonus = if t == seat { 5i64 } else { 0 };
                    att * 10 + self_bonus
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
        "wudian_liduan_give" => {
            if allow_cancel {
                return Some(GameAction::Answer {
                    choice_id: id,
                    selection: Vec::new(),
                });
            }
            let best = options
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Card(c) => Some(*c),
                    _ => None,
                })
                .min_by_key(|&c| extension_card_value(input, c));
            let selection = match best {
                Some(c) => vec![ChoiceOption::Card(c)],
                None => options.iter().take(1).copied().collect(),
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "wudian_youdishenru_sha" => {
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
        _ => None,
    }
}
