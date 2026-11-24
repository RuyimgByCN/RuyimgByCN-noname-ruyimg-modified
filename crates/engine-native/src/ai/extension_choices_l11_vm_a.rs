//! P3-E1 L11 (`L11VmFixA`) 挂起询问的公开信息集 AI 启发式策略。
//!
//! 严格只读取 `PolicyInput` / `PlayerView` 公开信息（身份、阵营、已装备/判定牌、弃牌堆、
//! 手牌数、体力与己方手牌），不窥视敌方隐藏手牌或牌堆顶未知随机状态。

use super::{extension_card_value, public_attitude};
use crate::catalog::{CardKind, EquipSlot};
use crate::choice::ChoiceOption;
use crate::engine::GameAction;
use crate::view::PolicyInput;

fn is_card_public_to_seat(input: &PolicyInput, card: u32) -> bool {
    input.own_hand_kinds.iter().any(|&(c, _)| c == card)
        || input.view.discard.contains(&card)
        || input
            .view
            .players
            .iter()
            .any(|p| p.equips.contains(&card) || p.judge.contains(&card))
}

fn public_card_value(
    catalog: &crate::catalog::Catalog,
    input: &PolicyInput,
    card: u32,
) -> i64 {
    if is_card_public_to_seat(input, card) {
        extension_card_value(catalog, input, card)
    } else {
        40
    }
}

fn pick_lowest_own_cards(
    catalog: &crate::catalog::Catalog,
    input: &PolicyInput,
    options: &[ChoiceOption],
    count: usize,
) -> Vec<ChoiceOption> {
    let mut cards: Vec<u32> = options
        .iter()
        .filter_map(|o| match *o {
            ChoiceOption::Card(c) => Some(c),
            _ => None,
        })
        .collect();
    cards.sort_by_key(|&c| (public_card_value(catalog, input, c), c));
    cards.into_iter().take(count).map(ChoiceOption::Card).collect()
}

fn best_player_by(
    options: &[ChoiceOption],
    mut pred: impl FnMut(u8) -> bool,
    mut score: impl FnMut(u8) -> (i32, i32, u8),
) -> Option<u8> {
    options
        .iter()
        .filter_map(|o| match *o {
            ChoiceOption::Player(p) => Some(p),
            _ => None,
        })
        .filter(|&p| pred(p))
        .min_by_key(|&p| score(p))
}

#[allow(clippy::too_many_arguments)]
pub(super) fn plan_l11_vm_a_choice_action(
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
    let me = input.view.players.get(seat as usize);
    let answer = |selection: Vec<ChoiceOption>| {
        Some(GameAction::Answer {
            choice_id: id,
            selection,
        })
    };
    let cancel_or_first = || {
        if allow_cancel {
            answer(Vec::new())
        } else {
            answer(options.iter().take(min).copied().collect())
        }
    };
    let hostile_player = |require_neg: bool| {
        best_player_by(
            options,
            |p| !require_neg || public_attitude(&input.view, seat, p) < 0,
            |p| {
                let hp = input.view.players.get(p as usize).map_or(99, |pl| pl.hp);
                (public_attitude(&input.view, seat, p), hp, p)
            },
        )
    };
    let ally_player = || {
        best_player_by(
            options,
            |p| public_attitude(&input.view, seat, p) > 0,
            |p| {
                let pl = input.view.players.get(p as usize);
                (
                    pl.map_or(99, |x| x.hand_count as i32),
                    pl.map_or(99, |x| x.hp),
                    p,
                )
            },
        )
    };

    match prompt {
        "trigger_confirm" => match input.choice.as_ref().and_then(|c| match c {
            crate::view::ChoiceView::Own { source_skill, .. } => *source_skill,
            _ => None,
        }) {
            Some("miji") => {
                let has_ally = input
                    .view
                    .players
                    .iter()
                    .any(|p| p.alive && p.seat != seat && public_attitude(&input.view, seat, p.seat) > 0);
                let ok = has_ally || me.is_some_and(|p| p.hp <= 1);
                answer(vec![if ok {
                    ChoiceOption::Confirm
                } else {
                    ChoiceOption::Pass
                }])
            }
            _ => None,
        },
        "l11_vm_a_miji_cards" => {
            let has_ally = input
                .view
                .players
                .iter()
                .any(|p| p.alive && p.seat != seat && public_attitude(&input.view, seat, p.seat) > 0);
            if !has_ally && allow_cancel {
                return answer(Vec::new());
            }
            answer(pick_lowest_own_cards(catalog, input, options, min))
        }
        "l11_vm_a_miji_target" => match ally_player() {
            Some(t) => answer(vec![ChoiceOption::Player(t)]),
            None => cancel_or_first(),
        },
        "l11_vm_a_fuji_confirm" => {
            let confirm = public_attitude(&input.view, seat, input.view.turn) > 0
                || input
                    .view
                    .players
                    .iter()
                    .any(|p| p.alive && p.seat != seat && public_attitude(&input.view, seat, p.seat) < 0);
            answer(vec![if confirm {
                ChoiceOption::Confirm
            } else {
                ChoiceOption::Pass
            }])
        }
        "l11_vm_a_wurong_target"
        | "l11_vm_a_yingjian_target"
        | "l11_vm_a_xiongluan_target"
        | "l11_vm_a_lihuo_extra"
        | "l11_vm_a_mingce_sha_target" => match hostile_player(true) {
            Some(t) => answer(vec![ChoiceOption::Player(t)]),
            None => cancel_or_first(),
        },
        "l11_vm_a_wurong_card" => {
            let pick = if seat == input.view.turn {
                input
                    .own_hand_kinds
                    .iter()
                    .find(|&&(c, k)| k.is_sha_like() && options.contains(&ChoiceOption::Card(c)))
                    .map(|&(c, _)| c)
            } else {
                input
                    .own_hand_kinds
                    .iter()
                    .find(|&&(c, k)| {
                        k == CardKind::Shan && options.contains(&ChoiceOption::Card(c))
                    })
                    .map(|&(c, _)| c)
            };
            let sel = pick
                .map(|c| vec![ChoiceOption::Card(c)])
                .unwrap_or_else(|| pick_lowest_own_cards(catalog, input, options, 1));
            answer(sel)
        }
        "l11_vm_a_wurong_gain" | "l11_vm_a_refankui_gain" | "l11_vm_a_nsyangwu_gain" => {
            if prompt == "l11_vm_a_refankui_gain"
                && allow_cancel
                && public_attitude(&input.view, seat, input.view.turn) > 0
            {
                return answer(Vec::new());
            }
            let mut cards: Vec<u32> = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Card(c) => Some(c),
                    _ => None,
                })
                .collect();
            cards.sort_by_key(|&c| {
                let eq_bonus = if input.view.players.iter().any(|p| p.equips.contains(&c)) {
                    1
                } else {
                    0
                };
                (std::cmp::Reverse(eq_bonus), std::cmp::Reverse(public_card_value(catalog, input, c)), c)
            });
            answer(cards.into_iter().take(min).map(ChoiceOption::Card).collect())
        }
        "l11_vm_a_yeyan_mode" => {
            if options.contains(&ChoiceOption::Tag(0)) {
                answer(vec![ChoiceOption::Tag(0)])
            } else {
                cancel_or_first()
            }
        }
        "l11_vm_a_yeyan_card" => answer(pick_lowest_own_cards(catalog, input, options, 1)),
        "l11_vm_a_yeyan_small_targets" | "l11_vm_a_yeyan_great_targets" => {
            let mut enemies: Vec<u8> = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Player(p) if public_attitude(&input.view, seat, p) < 0 => Some(p),
                    _ => None,
                })
                .collect();
            enemies.sort_by_key(|&p| (input.view.players.get(p as usize).map_or(99, |x| x.hp), p));
            enemies.truncate(max);
            if enemies.len() >= min {
                answer(enemies.into_iter().map(ChoiceOption::Player).collect())
            } else {
                cancel_or_first()
            }
        }
        "l11_vm_a_yeyan_2dmg_target" => match hostile_player(false) {
            Some(t) => answer(vec![ChoiceOption::Player(t)]),
            None => cancel_or_first(),
        },
        "l11_vm_a_yeyan_dmg_amount" => {
            if options.contains(&ChoiceOption::Tag(3)) {
                answer(vec![ChoiceOption::Tag(3)])
            } else {
                answer(options.iter().take(min).copied().collect())
            }
        }
        "l11_vm_a_fenwei_targets" => {
            let allies: Vec<ChoiceOption> = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Player(p) if public_attitude(&input.view, seat, p) > 0 => {
                        Some(ChoiceOption::Player(p))
                    }
                    _ => None,
                })
                .take(max)
                .collect();
            if allies.len() >= min {
                answer(allies)
            } else {
                cancel_or_first()
            }
        }
        "l11_vm_a_kuangfu_equip" | "l11_vm_a_qiaomeng_discard" => {
            let mut enemy_eqs: Vec<u32> = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Card(c) => Some(c),
                    _ => None,
                })
                .filter(|&c| {
                    input.view.players.iter().any(|p| {
                        p.equips.contains(&c) && public_attitude(&input.view, seat, p.seat) <= 0
                    })
                })
                .collect();
            enemy_eqs.sort_by_key(|&c| {
                let horse = (prompt == "l11_vm_a_qiaomeng_discard")
                    && catalog.cards.get(c as usize).is_some_and(|cd| {
                        matches!(
                            cd.kind.equip_slot(),
                            Some(EquipSlot::HorsePlus | EquipSlot::HorseMinus)
                        )
                    });
                (std::cmp::Reverse(horse), std::cmp::Reverse(public_card_value(catalog, input, c)), c)
            });
            if let Some(c) = enemy_eqs.first().copied() {
                answer(vec![ChoiceOption::Card(c)])
            } else {
                cancel_or_first()
            }
        }
        "l11_vm_a_kuangfu_equip_self"
        | "l11_vm_a_shenxian_confirm"
        | "l11_vm_a_lianpo_confirm"
        | "l11_vm_a_fenyin_confirm"
        | "l11_vm_a_jianying_confirm"
        | "l11_vm_a_guzheng_gain"
        | "l11_vm_a_kaikang_use_equip" => answer(vec![ChoiceOption::Confirm]),
        "l11_vm_a_langmie_play_discard" => {
            let best = pick_lowest_own_cards(catalog, input, options, 1);
            if let Some(ChoiceOption::Card(c)) = best.first().copied() {
                if public_card_value(catalog, input, c) <= 55 || !allow_cancel {
                    return answer(best);
                }
            }
            cancel_or_first()
        }
        "l11_vm_a_langmie_end_discard" => {
            if public_attitude(&input.view, seat, input.view.turn) < 0 {
                let best = pick_lowest_own_cards(catalog, input, options, 1);
                if !best.is_empty() {
                    return answer(best);
                }
            }
            cancel_or_first()
        }
        "l11_vm_a_bihuo_confirm" => {
            let ally_low = input.view.players.iter().any(|p| {
                p.alive && p.hp <= 1 && public_attitude(&input.view, seat, p.seat) > 0
            });
            answer(vec![if ally_low {
                ChoiceOption::Confirm
            } else {
                ChoiceOption::Pass
            }])
        }
        "l11_vm_a_chunlao_store" => {
            let picks: Vec<ChoiceOption> = options
                .iter()
                .filter(|o| matches!(o, ChoiceOption::Card(_)))
                .take(max)
                .copied()
                .collect();
            if picks.len() >= min {
                answer(picks)
            } else {
                cancel_or_first()
            }
        }
        "l11_vm_a_chunlao_spend" => answer(options.iter().take(min).copied().collect()),
        "l11_vm_a_fenming_confirm" => {
            let mut enemy_cnt = 0i32;
            let mut ally_cnt = 0i32;
            for p in &input.view.players {
                if p.alive && p.linked && (p.hand_count > 0 || !p.equips.is_empty()) {
                    if public_attitude(&input.view, seat, p.seat) > 0 {
                        ally_cnt += 1;
                    } else {
                        enemy_cnt += 1;
                    }
                }
            }
            answer(vec![if enemy_cnt > ally_cnt {
                ChoiceOption::Confirm
            } else {
                ChoiceOption::Pass
            }])
        }
        "l11_vm_a_fenming_card" => answer(pick_lowest_own_cards(catalog, input, options, min)),
        "l11_vm_a_guzheng_confirm" => {
            let confirm = public_attitude(&input.view, seat, input.view.turn) > 0;
            answer(vec![if confirm {
                ChoiceOption::Confirm
            } else {
                ChoiceOption::Pass
            }])
        }
        "l11_vm_a_guzheng_keep" => {
            let ally = public_attitude(&input.view, seat, input.view.turn) > 0;
            let mut cards: Vec<u32> = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Card(c) => Some(c),
                    _ => None,
                })
                .collect();
            if ally {
                cards.sort_by_key(|&c| (std::cmp::Reverse(public_card_value(catalog, input, c)), c));
            } else {
                cards.sort_by_key(|&c| (public_card_value(catalog, input, c), c));
            }
            answer(cards.into_iter().take(1).map(ChoiceOption::Card).collect())
        }
        "l11_vm_a_gzyinghun_target" => {
            let lost = me.map_or(1, |p| (p.max_hp as i32 - p.hp).max(1));
            if lost <= 1 {
                if let Some(a) = ally_player() {
                    return answer(vec![ChoiceOption::Player(a)]);
                }
            }
            match hostile_player(true).or_else(ally_player) {
                Some(t) => answer(vec![ChoiceOption::Player(t)]),
                None => cancel_or_first(),
            }
        }
        "l11_vm_a_gzyinghun_mode" => {
            let has_enemy = input
                .view
                .players
                .iter()
                .any(|p| p.alive && p.seat != seat && public_attitude(&input.view, seat, p.seat) < 0);
            let tag = if has_enemy && options.contains(&ChoiceOption::Tag(1)) {
                1
            } else {
                0
            };
            answer(vec![ChoiceOption::Tag(tag)])
        }
        "l11_vm_a_gzyinghun_discard" | "l11_vm_a_kaikang_give" => {
            answer(pick_lowest_own_cards(catalog, input, options, min))
        }
        "l11_vm_a_kaikang_confirm" => {
            let confirm = public_attitude(&input.view, seat, input.view.turn) < 0
                || input.view.players.iter().any(|p| {
                    p.alive && p.seat != seat && public_attitude(&input.view, seat, p.seat) > 0
                });
            answer(vec![if confirm {
                ChoiceOption::Confirm
            } else {
                ChoiceOption::Pass
            }])
        }
        "l11_vm_a_kunfen_confirm" => {
            let ok = me.is_some_and(|p| p.hp > 2);
            answer(vec![if ok {
                ChoiceOption::Confirm
            } else {
                ChoiceOption::Pass
            }])
        }
        "l11_vm_a_lihuo_convert" => answer(vec![ChoiceOption::Pass]),
        "l11_vm_a_mingce_card" => {
            if ally_player().is_none()
                && !input
                    .view
                    .players
                    .iter()
                    .any(|p| p.alive && p.seat != seat && public_attitude(&input.view, seat, p.seat) > 0)
                && allow_cancel
            {
                return answer(Vec::new());
            }
            answer(pick_lowest_own_cards(catalog, input, options, 1))
        }
        "l11_vm_a_mingce_give_target" => match ally_player() {
            Some(t) => answer(vec![ChoiceOption::Player(t)]),
            None => cancel_or_first(),
        },
        "l11_vm_a_mingce_choice" => {
            let has_enemy = input
                .view
                .players
                .iter()
                .any(|p| p.alive && p.seat != seat && public_attitude(&input.view, seat, p.seat) < 0);
            let tag = if has_enemy && options.contains(&ChoiceOption::Tag(1)) {
                1
            } else {
                0
            };
            answer(vec![ChoiceOption::Tag(tag)])
        }
        "l11_vm_a_nsyangwu_target" => {
            let pick = best_player_by(
                options,
                |p| public_attitude(&input.view, seat, p) < 0,
                |p| {
                    let hc = input.view.players.get(p as usize).map_or(0, |x| x.hand_count as i32);
                    (-hc, public_attitude(&input.view, seat, p), p)
                },
            );
            match pick {
                Some(t) => answer(vec![ChoiceOption::Player(t)]),
                None => cancel_or_first(),
            }
        }
        "l11_vm_a_nsyangwu_card" => answer(pick_lowest_own_cards(catalog, input, options, 1)),
        _ => None,
    }
}
