//! 扩展包挂起询问（`OpenChoice`）的公开信息集启发式策略（`plan_extension_choice_action`）。

use super::*;
use crate::choice::ChoiceOption;
use crate::view::ChoiceView;

#[path = "extension_choices_l10.rs"]
mod extension_choices_l10;
#[path = "extension_choices_l28.rs"]
mod extension_choices_l28;
#[path = "extension_choices_u01.rs"]
mod extension_choices_u01;
#[path = "extension_choices_l11.rs"]
mod extension_choices_l11;
#[path = "extension_choices_misc.rs"]
mod extension_choices_misc;
#[path = "extension_choices_l11_b2.rs"]
mod extension_choices_l11_b2;
#[path = "extension_choices_l11_b3.rs"]
mod extension_choices_l11_b3;
#[path = "extension_choices_l11_b4.rs"]
mod extension_choices_l11_b4;
#[path = "extension_choices_l11_b5.rs"]
mod extension_choices_l11_b5;
#[path = "extension_choices_u05_1.rs"]
mod extension_choices_u05_1;

pub(super) fn public_attitude(view: &crate::view::PlayerView, seat: u8, target: u8) -> i32 {
    if target == seat {
        return 10;
    }
    if let (Some(my_team), Some(target_team)) = (
        view.players.get(seat as usize).and_then(|p| p.team),
        view.players.get(target as usize).and_then(|p| p.team),
    ) {
        return if my_team == target_team { 6 } else { -5 };
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

pub(super) fn extension_card_value(
    catalog: &crate::catalog::Catalog,
    input: &crate::view::PolicyInput,
    card: crate::ids::CardId,
) -> i64 {
    let kind = input
        .own_hand_kinds
        .iter()
        .find(|(c, _)| *c == card)
        .map(|(_, k)| *k)
        .or_else(|| catalog.cards.get(card as usize).map(|d| d.kind));
    match kind {
        Some(crate::catalog::CardKind::Tao) => 80,
        Some(crate::catalog::CardKind::Jiu) | Some(crate::catalog::CardKind::Wuzhong) => 70,
        Some(crate::catalog::CardKind::Shan) => 40,
        Some(crate::catalog::CardKind::Sha) => 35,
        Some(_) => 45,
        None => 30,
    }
}

pub(super) fn plan_extension_choice_action(
    catalog: &crate::catalog::Catalog,
    input: &crate::view::PolicyInput,
) -> Option<GameAction> {
    let seat = input.view.seat;
    let Some(ChoiceView::Own {
        id,
        prompt,
        ref options,
        min,
        max,
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
                .min_by_key(|&c| extension_card_value(catalog, input, c));
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
            Some("huanle_jiuwei" | "jiuwei" | "dddmoyan") => Some(GameAction::Answer {
                choice_id: id,
                selection: vec![ChoiceOption::Confirm],
            }),
            Some("zhucheng2") => {
                let confirm = public_attitude(&input.view, seat, input.view.turn) <= 0;
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
            _ => extension_choices_l11_b3::plan_l11_b3_choice_action(
                catalog,
                input,
                id,
                prompt,
                options,
                min,
                max,
                allow_cancel,
            ),
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
                .min_by_key(|&c| extension_card_value(catalog, input, c));
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
                .min_by_key(|&c| extension_card_value(catalog, input, c));
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
        "wudian_diaobing_pick" => {
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
        "wudian_diaobing_top" => {
            let next_seat = ((seat as usize + 1) % input.view.players.len().max(1)) as u8;
            let selection = if public_attitude(&input.view, seat, next_seat) > 0 {
                options
                    .iter()
                    .find(|o| matches!(o, ChoiceOption::Card(_)))
                    .copied()
                    .into_iter()
                    .collect()
            } else {
                Vec::new()
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "wudian_zhucheng_guard" => {
            let mut ranked: Vec<(i64, ChoiceOption)> = options
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Card(c) => Some((extension_card_value(catalog, input, *c), *o)),
                    _ => None,
                })
                .collect();
            ranked.sort_by_key(|(v, _)| *v);
            let selection: Vec<ChoiceOption> = if min > 0 && ranked.len() >= min {
                ranked.into_iter().take(min).map(|(_, o)| o).collect()
            } else {
                Vec::new()
            };
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
                .min_by_key(|&c| extension_card_value(catalog, input, c));
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
        "wudian_diaobing_dir" => {
            // 玩点论杀 card/card.js:19-24：下家友方选逆时针 (Tag(1))，否则选顺时针 (Tag(0))
            let n = input.view.players.len() as u8;
            let next_alive = (1..n)
                .map(|off| (seat + off) % n)
                .find(|&p| input.view.players.get(p as usize).is_some_and(|s| s.alive))
                .unwrap_or(seat);
            let tag = if public_attitude(&input.view, seat, next_alive) > 0 {
                1
            } else {
                0
            };
            let pick = if options.contains(&ChoiceOption::Tag(tag)) {
                ChoiceOption::Tag(tag)
            } else {
                options.first().copied().unwrap_or(ChoiceOption::Tag(0))
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection: vec![pick],
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
                .min_by_key(|&c| extension_card_value(catalog, input, c));
            let selection = match best {
                Some(c) => vec![ChoiceOption::Card(c)],
                None => options.iter().take(1).copied().collect(),
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "wudian_youdishenru_use" => {
            let att = public_attitude(&input.view, seat, input.view.turn);
            let selection = if att <= 0 {
                options
                    .iter()
                    .find(|o| matches!(o, ChoiceOption::Card(_)))
                    .copied()
                    .into_iter()
                    .collect()
            } else {
                Vec::new()
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "wudian_youdishenru_sha" | "wudian_fuchou2_sha" => {
            let best = options
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Card(c) => Some(*c),
                    _ => None,
                })
                .min_by_key(|&c| extension_card_value(catalog, input, c));
            let selection = match best {
                Some(c) => vec![ChoiceOption::Card(c)],
                None => Vec::new(),
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "mbcmjiushi_gain_confirm" => Some(GameAction::Answer {
            choice_id: id,
            selection: vec![ChoiceOption::Confirm],
        }),
        "mbcmjiushi_turnback_confirm" => {
            let face_down = input
                .view
                .players
                .get(seat as usize)
                .is_some_and(|p| p.turned_over);
            let selection = if face_down || !allow_cancel {
                vec![ChoiceOption::Confirm]
            } else {
                Vec::new()
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "mbcmqingzheng_suit" => {
            let has_enemy_hand = input
                .view
                .players
                .iter()
                .any(|p| p.alive && p.seat != seat && p.hand_count > 0 && public_attitude(&input.view, seat, p.seat) < 0);
            if !has_enemy_hand && allow_cancel {
                return Some(GameAction::Answer {
                    choice_id: id,
                    selection: Vec::new(),
                });
            }
            let best = options
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Tag(tag) => Some(*tag),
                    _ => None,
                })
                .min_by_key(|&tag| {
                    input
                        .own_hand_kinds
                        .iter()
                        .filter(|&(cid, _)| {
                            catalog.cards.get(*cid as usize).is_some_and(|d| match d.suit {
                                crate::catalog::Suit::Spade => tag == 0,
                                crate::catalog::Suit::Heart => tag == 1,
                                crate::catalog::Suit::Club => tag == 2,
                                crate::catalog::Suit::Diamond => tag == 3,
                            })
                        })
                        .map(|&(cid, _)| extension_card_value(catalog, input, cid))
                        .sum::<i64>()
                });
            let selection = match best {
                Some(t) => vec![ChoiceOption::Tag(t)],
                None if allow_cancel => Vec::new(),
                None => options.iter().take(1).copied().collect(),
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "mbcmqingzheng_target" => {
            let best = options
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Player(t) => Some(*t),
                    _ => None,
                })
                .filter(|&t| {
                    public_attitude(&input.view, seat, t) < 0
                        && input
                            .view
                            .players
                            .get(t as usize)
                            .is_some_and(|p| p.hand_count > 0)
                })
                .min_by_key(|&t| {
                    input
                        .view
                        .players
                        .get(t as usize)
                        .map(|p| p.hand_count as i64)
                        .unwrap_or(99)
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
        "mbcmqingzheng_target_suit" => Some(GameAction::Answer {
            choice_id: id,
            selection: options.iter().take(1).copied().collect(),
        }),
        "dying_save" => {
            let has_card = options.iter().any(|o| matches!(o, ChoiceOption::Card(_)));
            if !has_card {
                if let Some(def) = catalog.skill_by_name("mbcmjiushi") {
                    if options.contains(&ChoiceOption::Skill(def)) {
                        return Some(GameAction::Answer {
                            choice_id: id,
                            selection: vec![ChoiceOption::Skill(def)],
                        });
                    }
                }
            }
            None
        }
        "mbcmfangzhu_target" => {
            let best = options
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Player(t) => Some(*t),
                    _ => None,
                })
                .filter(|&t| public_attitude(&input.view, seat, t) < 0)
                .max_by_key(|&t| {
                    let att = (-public_attitude(&input.view, seat, t)) as i64;
                    let hand_n = input
                        .view
                        .players
                        .get(t as usize)
                        .map(|p| p.hand_count as i64)
                        .unwrap_or(0);
                    att * 10 + hand_n
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
        "mbcmfangzhu_option" => {
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
        "k_danggu_partner" => {
            let selection = options
                .iter()
                .find(|o| matches!(o, ChoiceOption::Tag(9)))
                .or_else(|| options.first())
                .copied()
                .into_iter()
                .collect();
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "reguhuo_guess_pick" => {
            let me_hp = input.view.players.get(seat as usize).map(|p| p.hp).unwrap_or(0);
            let ally_turn = public_attitude(&input.view, seat, input.view.turn) >= 0;
            let want = if me_hp <= 1 || ally_turn { 0 } else { 1 };
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
        "reguhuo_final" => {
            let selection = options
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Card(c) => Some((*c, extension_card_value(catalog, input, *c))),
                    _ => None,
                })
                .min_by_key(|&(_, v)| v)
                .map(|(c, _)| ChoiceOption::Card(c))
                .into_iter()
                .collect();
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "rangjie_choice" => extension_choices_l10::plan_l10_choice_action(
            catalog,
            input,
            id,
            prompt,
            options,
            min,
            max,
            allow_cancel,
        ),
        "rangjie_type" => {
            let me = input.view.players.get(seat as usize);
            let want = if me.is_some_and(|p| p.hp <= 3) {
                0
            } else if me.is_some_and(|p| p.equips.len() < 2) {
                2
            } else {
                1
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
        "wumou:choose_mode" => {
            let tag = if options.contains(&ChoiceOption::Tag(0)) {
                0
            } else {
                1
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection: vec![ChoiceOption::Tag(tag)],
            })
        }
        "new_guixin:choose_zone" => {
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
        "xumou:confirm" => {
            let me = input.view.players.get(seat as usize);
            let want = me.is_some_and(|p| p.hp + p.hand_count as i32 <= 3);
            let selection = if want {
                options
                    .iter()
                    .find(|o| matches!(o, ChoiceOption::Confirm))
                    .copied()
                    .into_iter()
                    .collect()
            } else {
                Vec::new()
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "l01_yiyi_discard" | "l01_neikey_recover" => {
            super::l01_policy::plan_l01_choice(
                catalog,
                input,
                &super::l01_policy::L01AiOptions::default(),
            )
        }
        _ => extension_choices_misc::plan_misc_choice_action(
            catalog,
            input,
            id,
            prompt,
            options,
            min,
            max,
            allow_cancel,
        )
        .or_else(|| {
            extension_choices_u05_1::plan_u05_b1_choice_action(
                catalog,
                input,
                id,
                prompt,
                options,
                min,
                max,
                allow_cancel,
            )
        })
        .or_else(|| {
            extension_choices_l10::plan_l10_choice_action(
                catalog,
                input,
                id,
                prompt,
                options,
                min,
                max,
                allow_cancel,
            )
        })
        .or_else(|| {
            extension_choices_l28::plan_l28_choice_action(
                catalog,
                input,
                id,
                prompt,
                options,
                min,
                max,
                allow_cancel,
            )
        })
        .or_else(|| {
            extension_choices_u01::plan_u01_choice_action(
                catalog,
                input,
                id,
                prompt,
                options,
                min,
                max,
                allow_cancel,
            )
        })
        .or_else(|| {
            extension_choices_l11::plan_l11_choice_action(
                catalog,
                input,
                id,
                prompt,
                options,
                min,
                max,
                allow_cancel,
            )
        })
        .or_else(|| {
            extension_choices_l11_b2::plan_l11_b2_choice_action(
                catalog,
                input,
                id,
                prompt,
                options,
                min,
                max,
                allow_cancel,
            )
        })
        .or_else(|| {
            extension_choices_l11_b3::plan_l11_b3_choice_action(
                catalog,
                input,
                id,
                prompt,
                options,
                min,
                max,
                allow_cancel,
            )
        })
        .or_else(|| {
            extension_choices_l11_b4::plan_l11_b4_choice_action(
                catalog,
                input,
                id,
                prompt,
                options,
                min,
                max,
                allow_cancel,
            )
        })
        .or_else(|| {
            extension_choices_l11_b5::plan_l11_b5_choice_action(
                catalog,
                input,
                id,
                prompt,
                options,
                min,
                max,
                allow_cancel,
            )
        }),
    }
}
